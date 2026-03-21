use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PointCloudRecord {
    pub id: Uuid,
    pub map_id: Uuid,
    pub raw_file_path: Option<String>,
    pub tiles_path: Option<String>,
    pub point_count: Option<i64>,
    pub format: Option<String>,
    pub lod_levels: Option<i32>,
    /// Processing status: pending, processing, completed, failed.
    pub processing_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProcessingJobRecord {
    pub id: Uuid,
    pub map_id: Uuid,
    /// Job type: potree_convert, downsample, etc.
    pub job_type: String,
    /// Status: pending, running, completed, failed.
    pub status: String,
    /// Progress percentage (0.0 to 100.0).
    pub progress: f32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
