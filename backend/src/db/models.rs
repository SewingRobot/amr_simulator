use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Robot {
    pub id: Uuid,
    pub name: String,
    pub model_id: Option<String>,
    pub status: String,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Map {
    pub id: Uuid,
    pub name: String,
    pub version: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Mission {
    pub id: Uuid,
    pub robot_id: Option<Uuid>,
    pub status: String,
    pub priority: i32,
    pub start_node_id: Option<String>,
    pub end_node_id: Option<String>,
    pub path: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoadmapNode {
    pub id: Uuid,
    pub map_id: Uuid,
    pub name: Option<String>,
    pub node_type: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub properties: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoadmapEdge {
    pub id: Uuid,
    pub map_id: Uuid,
    pub source_node_id: Uuid,
    pub target_node_id: Uuid,
    pub distance: f64,
    pub max_speed: f64,
    pub direction: String,
    pub cost_factor: f64,
    pub properties: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
