use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoadmapNodeRecord {
    pub id: Uuid,
    pub map_id: Uuid,
    pub name: Option<String>,
    /// Node type: waypoint, charging_station, loading_dock, etc.
    pub node_type: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub properties: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoadmapEdgeRecord {
    pub id: Uuid,
    pub map_id: Uuid,
    pub source_node_id: Uuid,
    pub target_node_id: Uuid,
    /// Edge length in meters.
    pub distance: f64,
    /// Maximum speed in m/s.
    pub max_speed: f64,
    /// Direction: "uni" (unidirectional) or "bi" (bidirectional).
    pub direction: String,
    /// Cost multiplier for pathfinding (1.0 = normal).
    pub cost_factor: f64,
    pub properties: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
