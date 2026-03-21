use axum::{extract::State, http::Extensions, Json};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::db::models::User;
use crate::error::AppError;
use crate::services::auth_service::{self, Claims};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub email: String,
    pub role: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user: User = sqlx::query_as(
        "SELECT id, email, password_hash, role, created_at, updated_at FROM users WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

    auth_service::verify_password(&payload.password, &user.password_hash)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".into()))?;

    let token = auth_service::generate_jwt(
        &user.id.to_string(),
        &user.email,
        &user.role,
        &state.config.jwt_secret,
        state.config.jwt_expiration_secs,
    )?;

    Ok(Json(LoginResponse {
        token,
        token_type: "Bearer".into(),
        expires_in: state.config.jwt_expiration_secs,
    }))
}

pub async fn me(extensions: Extensions) -> Result<Json<MeResponse>, AppError> {
    let claims = extensions
        .get::<Claims>()
        .ok_or_else(|| AppError::Unauthorized("Missing claims".into()))?;

    Ok(Json(MeResponse {
        user_id: claims.sub.clone(),
        email: claims.email.clone(),
        role: claims.role.clone(),
    }))
}
