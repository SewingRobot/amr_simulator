use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::api::AppState;
use crate::error::AppError;
use crate::grpc::telemetry::SimCommand;

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub status: String,
}

/// POST /api/sim/command
///
/// Forwards a control command to the Sim Engine via TCP.
pub async fn send_command(
    State(state): State<AppState>,
    Json(cmd): Json<SimCommand>,
) -> Result<(StatusCode, Json<CommandResponse>), AppError> {
    state
        .sim_client
        .send_command(&cmd)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to send command to sim engine: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(CommandResponse {
            status: "sent".to_string(),
        }),
    ))
}
