use serde::{Deserialize, Serialize};

/// Telemetry message from Sim Engine (matches C++ JSON format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryMessage {
    pub robot_id: String,
    pub timestamp_ms: f64,
    pub pose: SimPose,
    pub velocity: SimVelocity,
    pub battery_percent: f64,
    pub status: String,
    #[serde(default)]
    pub lidar_ranges: Vec<f64>,
    #[serde(default)]
    pub errors: Vec<String>,
}

/// Sim engine sends 2D pose: {x, y, theta}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimPose {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

/// Sim engine sends velocity: {linear, angular}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimVelocity {
    pub linear: f64,
    pub angular: f64,
}

/// WebSocket telemetry envelope sent to Frontend
#[derive(Debug, Clone, Serialize)]
pub struct WsTelemetryEnvelope {
    pub r#type: String,
    pub topic: String,
    pub payload: WsTelemetryPayload,
    pub timestamp: f64,
}

/// Frontend-friendly telemetry payload (3D pose format)
#[derive(Debug, Clone, Serialize)]
pub struct WsTelemetryPayload {
    pub robot_id: String,
    pub timestamp_ms: f64,
    pub pose: WsPose,
    pub velocity: WsVelocity,
    pub battery_percent: f64,
    pub status: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsPose {
    pub position: WsPosition,
    pub orientation: WsOrientation,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsOrientation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WsVelocity {
    pub linear: WsPosition,
    pub angular: WsPosition,
}

impl TelemetryMessage {
    /// Convert sim 2D telemetry to frontend 3D WebSocket format
    pub fn to_ws_envelope(&self) -> WsTelemetryEnvelope {
        let half_theta = self.pose.theta / 2.0;
        WsTelemetryEnvelope {
            r#type: "telemetry".to_string(),
            topic: format!("telemetry:{}", self.robot_id),
            timestamp: self.timestamp_ms,
            payload: WsTelemetryPayload {
                robot_id: self.robot_id.clone(),
                timestamp_ms: self.timestamp_ms,
                pose: WsPose {
                    position: WsPosition {
                        x: self.pose.x,
                        y: self.pose.y,
                        z: 0.0,
                    },
                    orientation: WsOrientation {
                        x: 0.0,
                        y: 0.0,
                        z: half_theta.sin(),
                        w: half_theta.cos(),
                    },
                },
                velocity: WsVelocity {
                    linear: WsPosition {
                        x: self.velocity.linear,
                        y: 0.0,
                        z: 0.0,
                    },
                    angular: WsPosition {
                        x: 0.0,
                        y: 0.0,
                        z: self.velocity.angular,
                    },
                },
                battery_percent: self.battery_percent,
                status: self.status.clone(),
                errors: self.errors.clone(),
            },
        }
    }
}

/// Command sent from the backend to the sim engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimCommand {
    #[serde(rename = "type")]
    pub command_type: String,
    #[serde(default)]
    pub robot_id: String,
    #[serde(default)]
    pub linear: f64,
    #[serde(default)]
    pub angular: f64,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub theta: f64,
    #[serde(default)]
    pub config: serde_json::Value,
}
