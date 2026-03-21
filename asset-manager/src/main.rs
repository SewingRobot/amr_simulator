use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod grpc;
mod models;
mod storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,amr_asset_manager=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::AppConfig::from_env()?;
    tracing::info!(
        "AMR Asset Manager v0.1.0 starting on port {}",
        config.grpc_port
    );

    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // TODO: Initialize S3 client
    // TODO: Start gRPC server

    tracing::info!("Asset Manager ready");

    // Keep running
    tokio::signal::ctrl_c().await?;
    Ok(())
}
