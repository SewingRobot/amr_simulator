use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::db::models::Mission;
use crate::error::AppError;
use crate::services::mission_service;

#[derive(Debug, Deserialize)]
pub struct ListMissionsQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMissionRequest {
    pub start_node_id: Option<String>,
    pub end_node_id: Option<String>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
}

fn default_metadata() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize)]
pub struct AssignMissionRequest {
    pub robot_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct MissionResponse {
    pub id: Uuid,
    pub robot_id: Option<Uuid>,
    pub status: String,
    pub priority: i32,
    pub start_node_id: Option<String>,
    pub end_node_id: Option<String>,
    pub path: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Mission> for MissionResponse {
    fn from(m: Mission) -> Self {
        Self {
            id: m.id,
            robot_id: m.robot_id,
            status: m.status,
            priority: m.priority,
            start_node_id: m.start_node_id,
            end_node_id: m.end_node_id,
            path: m.path,
            metadata: m.metadata,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

fn broadcast_mission_update(state: &AppState, mission: &Mission) {
    let payload = serde_json::json!({
        "type": "mission_update",
        "topic": "missions",
        "payload": {
            "id": mission.id.to_string(),
            "status": mission.status,
            "robot_id": mission.robot_id.map(|id| id.to_string()),
        }
    });
    // Use telemetry_tx to broadcast; we reuse the existing broadcast channel
    // by wrapping the mission update as a TelemetryMessage.
    // Instead, we use a dedicated approach: send via the WS broadcast.
    let _ = state.mission_tx.send(payload);
}

pub async fn list_missions(
    State(state): State<AppState>,
    Query(query): Query<ListMissionsQuery>,
) -> Result<Json<Vec<MissionResponse>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).min(200);

    let missions = mission_service::list(
        &state.pool,
        query.status.as_deref(),
        page,
        limit,
    )
    .await?;

    Ok(Json(missions.into_iter().map(MissionResponse::from).collect()))
}

pub async fn create_mission(
    State(state): State<AppState>,
    Json(payload): Json<CreateMissionRequest>,
) -> Result<Json<MissionResponse>, AppError> {
    let mission = mission_service::create(
        &state.pool,
        payload.start_node_id,
        payload.end_node_id,
        payload.priority,
        payload.metadata,
    )
    .await?;

    broadcast_mission_update(&state, &mission);
    Ok(Json(MissionResponse::from(mission)))
}

pub async fn get_mission(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MissionResponse>, AppError> {
    let mission = mission_service::get_by_id(&state.pool, id).await?;
    Ok(Json(MissionResponse::from(mission)))
}

pub async fn assign_mission(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AssignMissionRequest>,
) -> Result<Json<MissionResponse>, AppError> {
    let mission = mission_service::assign(&state.pool, id, payload.robot_id).await?;
    broadcast_mission_update(&state, &mission);
    Ok(Json(MissionResponse::from(mission)))
}

pub async fn cancel_mission(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MissionResponse>, AppError> {
    let mission = mission_service::cancel(&state.pool, id).await?;
    broadcast_mission_update(&state, &mission);
    Ok(Json(MissionResponse::from(mission)))
}

pub async fn delete_mission(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    mission_service::delete(&state.pool, id).await?;
    Ok(Json(serde_json::json!({ "deleted": true, "id": id.to_string() })))
}
