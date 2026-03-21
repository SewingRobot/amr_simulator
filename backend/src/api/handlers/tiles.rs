use axum::{
    extract::Path,
    Json,
};
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct RoadmapNode {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub neighbors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RoadmapResponse {
    pub map_id: String,
    pub nodes: Vec<RoadmapNode>,
}

/// GET /api/maps/:map_id/tiles/:node_id — point cloud tile proxy.
/// For Phase 1: returns 404 (point cloud tiles not yet available).
pub async fn get_tile(
    Path((map_id, node_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    Err(AppError::NotFound(format!(
        "Tile {}/{} not yet available — point cloud tiles are not implemented in Phase 1",
        map_id, node_id
    )))
}

/// GET /api/maps/:map_id/roadmap — return hardcoded sample roadmap JSON.
pub async fn get_roadmap(
    Path(map_id): Path<String>,
) -> Json<RoadmapResponse> {
    let roadmap = RoadmapResponse {
        map_id,
        nodes: vec![
            RoadmapNode {
                id: "node-a".to_string(),
                x: 0.0,
                y: 0.0,
                z: 0.0,
                neighbors: vec!["node-b".to_string(), "node-c".to_string()],
            },
            RoadmapNode {
                id: "node-b".to_string(),
                x: 5.0,
                y: 0.0,
                z: 0.0,
                neighbors: vec!["node-a".to_string(), "node-d".to_string()],
            },
            RoadmapNode {
                id: "node-c".to_string(),
                x: 0.0,
                y: 5.0,
                z: 0.0,
                neighbors: vec!["node-a".to_string(), "node-d".to_string()],
            },
            RoadmapNode {
                id: "node-d".to_string(),
                x: 5.0,
                y: 5.0,
                z: 0.0,
                neighbors: vec!["node-b".to_string(), "node-c".to_string()],
            },
        ],
    };
    Json(roadmap)
}
