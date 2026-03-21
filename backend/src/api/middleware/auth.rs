use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::Response,
};

use crate::api::AppState;
use crate::error::AppError;
use crate::services::auth_service::{self, Claims};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid authorization header format".into()))?;

    let claims = auth_service::validate_jwt(token, &state.config.jwt_secret)
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// Extractor for getting Claims from request extensions.
/// Use in handlers: `claims: Claims` after the auth middleware runs.
impl Claims {
    pub fn from_request_extensions(extensions: &axum::http::Extensions) -> Option<&Claims> {
        extensions.get::<Claims>()
    }
}
