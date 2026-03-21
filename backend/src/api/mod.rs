pub mod handlers;
pub mod middleware;
pub mod router;
pub mod ws;

use crate::config::AppConfig;
use crate::grpc::sim_client::SimEngineClient;
use crate::grpc::telemetry::TelemetryMessage;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Channel capacity for the telemetry broadcast. Messages are dropped for
/// slow receivers once this buffer is exceeded.
const TELEMETRY_CHANNEL_CAPACITY: usize = 256;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub telemetry_tx: broadcast::Sender<TelemetryMessage>,
    pub sim_client: Arc<SimEngineClient>,
}

impl AppState {
    pub fn new(pool: PgPool, config: AppConfig) -> Self {
        let (telemetry_tx, _) = broadcast::channel(TELEMETRY_CHANNEL_CAPACITY);
        let sim_client = Arc::new(SimEngineClient::new(
            config.sim_telemetry_addr.clone(),
            config.sim_command_addr.clone(),
        ));
        Self {
            pool,
            config,
            telemetry_tx,
            sim_client,
        }
    }
}
