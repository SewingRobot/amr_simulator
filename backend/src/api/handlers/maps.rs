use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::db::models::Map;
use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct CreateMapRequest {
    pub name: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMapRequest {
    pub name: Option<String>,
    pub version: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct MapResponse {
    pub id: Uuid,
    pub name: String,
    pub version: i32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Map> for MapResponse {
    fn from(m: Map) -> Self {
        Self {
            id: m.id,
            name: m.name,
            version: m.version,
            metadata: m.metadata,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

pub async fn list_maps(
    State(state): State<AppState>,
) -> Result<Json<Vec<MapResponse>>, AppError> {
    let maps: Vec<Map> = sqlx::query_as(
        "SELECT id, name, version, metadata, created_at, updated_at FROM maps ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(maps.into_iter().map(MapResponse::from).collect()))
}

pub async fn create_map(
    State(state): State<AppState>,
    Json(payload): Json<CreateMapRequest>,
) -> Result<Json<MapResponse>, AppError> {
    let metadata = payload.metadata.unwrap_or(serde_json::json!({}));

    let map: Map = sqlx::query_as(
        r#"INSERT INTO maps (name, metadata)
           VALUES ($1, $2)
           RETURNING id, name, version, metadata, created_at, updated_at"#,
    )
    .bind(&payload.name)
    .bind(&metadata)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(MapResponse::from(map)))
}

pub async fn get_map(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MapResponse>, AppError> {
    let map: Map = sqlx::query_as(
        "SELECT id, name, version, metadata, created_at, updated_at FROM maps WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Map {} not found", id)))?;

    Ok(Json(MapResponse::from(map)))
}

pub async fn update_map(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMapRequest>,
) -> Result<Json<MapResponse>, AppError> {
    let existing: Map = sqlx::query_as(
        "SELECT id, name, version, metadata, created_at, updated_at FROM maps WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Map {} not found", id)))?;

    let name = payload.name.unwrap_or(existing.name);
    let version = payload.version.unwrap_or(existing.version);
    let metadata = payload.metadata.unwrap_or(existing.metadata);

    let map: Map = sqlx::query_as(
        r#"UPDATE maps SET name = $1, version = $2, metadata = $3, updated_at = NOW()
           WHERE id = $4
           RETURNING id, name, version, metadata, created_at, updated_at"#,
    )
    .bind(&name)
    .bind(version)
    .bind(&metadata)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(MapResponse::from(map)))
}

pub async fn delete_map(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM maps WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Map {} not found", id)));
    }

    Ok(Json(serde_json::json!({ "deleted": true, "id": id.to_string() })))
}
