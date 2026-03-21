use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetRecord {
    pub id: Uuid,
    pub asset_type: String,
    pub name: String,
    pub description: Option<String>,
    pub properties: serde_json::Value,
    pub tags: Vec<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetFileRecord {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub format: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub created_at: DateTime<Utc>,
}
