use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod db;
mod error;
mod grpc;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,amr_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::AppConfig::from_env()?;
    tracing::info!("AMR Backend v0.1.0 starting");

    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    let app_state = api::AppState::new(pool, config.clone());

    // Spawn the Sim Engine telemetry bridge as a background task.
    // It will reconnect-loop so the server keeps running even when the
    // sim engine is not available yet.
    let telemetry_tx = app_state.telemetry_tx.clone();
    let sim_client = app_state.sim_client.clone();
    tokio::spawn(async move {
        loop {
            tracing::info!("Starting Sim Engine telemetry bridge...");
            match sim_client.stream_telemetry(telemetry_tx.clone()).await {
                Ok(()) => {
                    tracing::warn!("Sim Engine telemetry stream ended, reconnecting in 5s...");
                }
                Err(e) => {
                    tracing::warn!(
                        "Sim Engine telemetry connection failed: {}. Retrying in 5s...",
                        e
                    );
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    });

    let app = api::router::create_router(app_state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("AMR Backend shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("Shutdown signal received, stopping server...");
}
