use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::db::models::Robot;
use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct CreateRobotRequest {
    pub name: String,
    pub model_id: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRobotRequest {
    pub name: Option<String>,
    pub model_id: Option<String>,
    pub status: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RobotResponse {
    pub id: Uuid,
    pub name: String,
    pub model_id: Option<String>,
    pub status: String,
    pub config: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Robot> for RobotResponse {
    fn from(r: Robot) -> Self {
        Self {
            id: r.id,
            name: r.name,
            model_id: r.model_id,
            status: r.status,
            config: r.config,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

pub async fn list_robots(
    State(state): State<AppState>,
) -> Result<Json<Vec<RobotResponse>>, AppError> {
    let robots: Vec<Robot> = sqlx::query_as(
        "SELECT id, name, model_id, status, config, created_at, updated_at FROM robots ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(robots.into_iter().map(RobotResponse::from).collect()))
}

pub async fn create_robot(
    State(state): State<AppState>,
    Json(payload): Json<CreateRobotRequest>,
) -> Result<Json<RobotResponse>, AppError> {
    let config = payload.config.unwrap_or(serde_json::json!({}));

    let robot: Robot = sqlx::query_as(
        r#"INSERT INTO robots (name, model_id, config)
           VALUES ($1, $2, $3)
           RETURNING id, name, model_id, status, config, created_at, updated_at"#,
    )
    .bind(&payload.name)
    .bind(&payload.model_id)
    .bind(&config)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(RobotResponse::from(robot)))
}

pub async fn get_robot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RobotResponse>, AppError> {
    let robot: Robot = sqlx::query_as(
        "SELECT id, name, model_id, status, config, created_at, updated_at FROM robots WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Robot {} not found", id)))?;

    Ok(Json(RobotResponse::from(robot)))
}

pub async fn update_robot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRobotRequest>,
) -> Result<Json<RobotResponse>, AppError> {
    let existing: Robot = sqlx::query_as(
        "SELECT id, name, model_id, status, config, created_at, updated_at FROM robots WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Robot {} not found", id)))?;

    let name = payload.name.unwrap_or(existing.name);
    let model_id = payload.model_id.or(existing.model_id);
    let status = payload.status.unwrap_or(existing.status);
    let config = payload.config.unwrap_or(existing.config);

    let robot: Robot = sqlx::query_as(
        r#"UPDATE robots SET name = $1, model_id = $2, status = $3, config = $4, updated_at = NOW()
           WHERE id = $5
           RETURNING id, name, model_id, status, config, created_at, updated_at"#,
    )
    .bind(&name)
    .bind(&model_id)
    .bind(&status)
    .bind(&config)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(RobotResponse::from(robot)))
}

pub async fn delete_robot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM robots WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Robot {} not found", id)));
    }

    Ok(Json(serde_json::json!({ "deleted": true, "id": id.to_string() })))
}
