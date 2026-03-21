use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::sync::broadcast;
use tokio::time::{self, Duration};

use crate::api::AppState;
use crate::grpc::telemetry::{SimCommand, TelemetryMessage};
use crate::services::auth_service;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum WsClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { topic: String },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { topic: String },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "command")]
    Command { payload: SimCommand },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum WsServerMessage {
    #[serde(rename = "subscribed")]
    Subscribed { topic: String },
    #[serde(rename = "unsubscribed")]
    Unsubscribed { topic: String },
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "telemetry")]
    Telemetry {
        topic: String,
        data: serde_json::Value,
    },
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    // Validate JWT token if provided
    if let Some(ref token) = query.token {
        match auth_service::validate_jwt(token, &state.config.jwt_secret) {
            Ok(claims) => {
                tracing::info!(
                    "WebSocket connection authenticated for user: {}",
                    claims.email
                );
            }
            Err(e) => {
                tracing::debug!("WebSocket auth skipped: {}", e);
            }
        }
    }

    let telemetry_rx = state.telemetry_tx.subscribe();
    let sim_client = state.sim_client.clone();

    ws.on_upgrade(move |socket| handle_socket(socket, telemetry_rx, sim_client))
}

async fn handle_socket(
    mut socket: WebSocket,
    mut telemetry_rx: broadcast::Receiver<TelemetryMessage>,
    sim_client: std::sync::Arc<crate::grpc::sim_client::SimEngineClient>,
) {
    let mut subscriptions: HashSet<String> = HashSet::new();
    let mut heartbeat_interval = time::interval(Duration::from_secs(30));

    tracing::info!("WebSocket client connected");

    loop {
        tokio::select! {
            // Incoming message from the WebSocket client
            maybe_msg = socket.recv() => {
                let msg = match maybe_msg {
                    Some(Ok(msg)) => msg,
                    Some(Err(e)) => {
                        tracing::error!("WebSocket receive error: {}", e);
                        break;
                    }
                    None => break, // stream ended
                };

                match msg {
                    Message::Text(text) => {
                        let response = match serde_json::from_str::<WsClientMessage>(&text) {
                            Ok(WsClientMessage::Subscribe { topic }) => {
                                subscriptions.insert(topic.clone());
                                tracing::debug!("Client subscribed to: {}", topic);
                                Some(WsServerMessage::Subscribed { topic })
                            }
                            Ok(WsClientMessage::Unsubscribe { topic }) => {
                                subscriptions.remove(&topic);
                                tracing::debug!("Client unsubscribed from: {}", topic);
                                Some(WsServerMessage::Unsubscribed { topic })
                            }
                            Ok(WsClientMessage::Ping) => Some(WsServerMessage::Pong),
                            Ok(WsClientMessage::Command { payload }) => {
                                let client = sim_client.clone();
                                // Fire and forget; report errors back to client
                                if let Err(e) = client.send_command(&payload).await {
                                    tracing::warn!("Failed to send command to sim engine: {}", e);
                                    Some(WsServerMessage::Error {
                                        message: format!("Command relay failed: {}", e),
                                    })
                                } else {
                                    None // no response needed on success
                                }
                            }
                            Err(e) => Some(WsServerMessage::Error {
                                message: format!("Invalid message: {}", e),
                            }),
                        };

                        if let Some(resp) = response {
                            let text = serde_json::to_string(&resp).unwrap();
                            if socket.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Message::Close(_) => {
                        tracing::info!("WebSocket client disconnected");
                        break;
                    }
                    _ => {}
                }
            }

            // Telemetry from the sim engine broadcast channel
            result = telemetry_rx.recv() => {
                match result {
                    Ok(msg) => {
                        if should_forward(&subscriptions, &msg) {
                            let envelope = msg.to_ws_envelope();
                            let text = serde_json::to_string(&envelope).unwrap();
                            if socket.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WebSocket client lagged, skipped {} telemetry messages", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::info!("Telemetry broadcast channel closed");
                        break;
                    }
                }
            }

            // Heartbeat ping
            _ = heartbeat_interval.tick() => {
                if socket.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
        }
    }

    tracing::info!(
        "WebSocket session ended, had {} subscriptions",
        subscriptions.len()
    );
}

/// Check whether a telemetry message matches any of the client's subscriptions.
///
/// Supported subscription patterns:
/// - `telemetry:*` — matches all robots
/// - `telemetry:{robot_id}` — matches a specific robot
fn should_forward(subscriptions: &HashSet<String>, msg: &TelemetryMessage) -> bool {
    if subscriptions.is_empty() {
        return false;
    }
    let wildcard = "telemetry:*";
    let specific = format!("telemetry:{}", msg.robot_id);
    subscriptions.contains(wildcard) || subscriptions.contains(&specific)
}
