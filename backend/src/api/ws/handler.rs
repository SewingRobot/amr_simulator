use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::api::AppState;
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
                tracing::info!("WebSocket connection authenticated for user: {}", claims.email);
            }
            Err(e) => {
                tracing::warn!("WebSocket auth failed: {}", e);
                // Still allow connection but log the warning
                // In production, you might reject unauthenticated connections
            }
        }
    }

    ws.on_upgrade(|socket| handle_socket(socket))
}

async fn handle_socket(mut socket: WebSocket) {
    let mut subscriptions: HashSet<String> = HashSet::new();

    tracing::info!("WebSocket client connected");

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("WebSocket receive error: {}", e);
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                let response = match serde_json::from_str::<WsClientMessage>(&text) {
                    Ok(WsClientMessage::Subscribe { topic }) => {
                        subscriptions.insert(topic.clone());
                        tracing::debug!("Client subscribed to: {}", topic);
                        WsServerMessage::Subscribed { topic }
                    }
                    Ok(WsClientMessage::Unsubscribe { topic }) => {
                        subscriptions.remove(&topic);
                        tracing::debug!("Client unsubscribed from: {}", topic);
                        WsServerMessage::Unsubscribed { topic }
                    }
                    Ok(WsClientMessage::Ping) => WsServerMessage::Pong,
                    Err(e) => WsServerMessage::Error {
                        message: format!("Invalid message: {}", e),
                    },
                };

                let response_text = serde_json::to_string(&response).unwrap();
                if socket.send(Message::Text(response_text.into())).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => {
                tracing::info!("WebSocket client disconnected");
                break;
            }
            _ => {}
        }
    }

    tracing::info!(
        "WebSocket session ended, had {} subscriptions",
        subscriptions.len()
    );
}
