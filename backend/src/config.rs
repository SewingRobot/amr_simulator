use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_secs: u64,
    pub port: u16,
    pub grpc_sim_engine_url: String,
    pub grpc_asset_manager_url: String,
    pub grpc_map_manager_url: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-in-production".into()),
            jwt_expiration_secs: std::env::var("JWT_EXPIRATION_SECS")
                .unwrap_or_else(|_| "3600".into())
                .parse()
                .context("JWT_EXPIRATION_SECS must be a number")?,
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .context("PORT must be a number")?,
            grpc_sim_engine_url: std::env::var("GRPC_SIM_ENGINE_URL")
                .unwrap_or_else(|_| "http://localhost:50051".into()),
            grpc_asset_manager_url: std::env::var("GRPC_ASSET_MANAGER_URL")
                .unwrap_or_else(|_| "http://localhost:50052".into()),
            grpc_map_manager_url: std::env::var("GRPC_MAP_MANAGER_URL")
                .unwrap_or_else(|_| "http://localhost:50053".into()),
        })
    }
}
