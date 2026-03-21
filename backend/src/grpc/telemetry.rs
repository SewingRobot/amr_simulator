use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryMessage {
    pub robot_id: String,
    pub timestamp_ms: f64,
    pub pose: Pose,
    pub velocity: Velocity,
    pub battery_percent: f64,
    pub status: String,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    pub position: Position,
    pub orientation: Orientation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orientation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Velocity {
    pub linear: LinearVelocity,
    pub angular: AngularVelocity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearVelocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AngularVelocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Command sent from the backend to the sim engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimCommand {
    pub robot_id: String,
    pub command_type: String,
    #[serde(default)]
    pub linear: f64,
    #[serde(default)]
    pub angular: f64,
    #[serde(default)]
    pub payload: serde_json::Value,
}
