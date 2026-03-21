use axum::{
    extract::Path,
    http::header,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct AssetInfo {
    pub id: String,
    pub name: String,
    pub format: String,
}

/// GET /api/assets — return hardcoded list of available models.
pub async fn list_assets() -> Json<Vec<AssetInfo>> {
    let assets = vec![
        AssetInfo {
            id: "default-amr".to_string(),
            name: "Default AMR".to_string(),
            format: "glb".to_string(),
        },
        AssetInfo {
            id: "forklift".to_string(),
            name: "Forklift".to_string(),
            format: "glb".to_string(),
        },
        AssetInfo {
            id: "tugbot".to_string(),
            name: "Tugbot".to_string(),
            format: "glb".to_string(),
        },
    ];
    Json(assets)
}

/// GET /api/assets/:id/download — serve a static .glb file from a local directory.
/// For Phase 1: serves from local filesystem.
/// Later: proxy to Asset Manager gRPC.
pub async fn download_asset(
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Sanitize id to prevent path traversal
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err(AppError::Validation("Invalid asset id".to_string()));
    }

    let path = format!("static/models/{}.glb", id);
    match tokio::fs::read(&path).await {
        Ok(data) => Ok((
            [(header::CONTENT_TYPE, "model/gltf-binary")],
            data,
        )),
        Err(_) => Err(AppError::NotFound(format!("Asset {} not found", id))),
    }
}
