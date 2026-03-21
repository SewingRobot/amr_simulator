//! Placeholder gRPC client for the Simulation Engine service.
//!
//! This module will contain the tonic-generated client code and helper
//! functions for communicating with the Sim Engine once the proto files
//! are finalized and compiled via `build.rs`.

use anyhow::Result;

/// Placeholder struct for the simulation engine gRPC client.
#[derive(Debug, Clone)]
pub struct SimEngineClient {
    endpoint: String,
}

impl SimEngineClient {
    /// Create a new SimEngineClient pointing at the given gRPC endpoint.
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
        }
    }

    /// Connect to the simulation engine.
    /// TODO: Replace with actual tonic client connection when proto files are ready.
    pub async fn connect(&self) -> Result<()> {
        tracing::info!("SimEngineClient: would connect to {}", self.endpoint);
        Ok(())
    }

    /// Start a simulation run.
    /// TODO: Implement with generated proto types.
    pub async fn start_simulation(&self, _scenario_id: &str) -> Result<()> {
        tracing::info!("SimEngineClient: start_simulation (placeholder)");
        Ok(())
    }

    /// Stop a running simulation.
    /// TODO: Implement with generated proto types.
    pub async fn stop_simulation(&self, _run_id: &str) -> Result<()> {
        tracing::info!("SimEngineClient: stop_simulation (placeholder)");
        Ok(())
    }

    /// Stream telemetry data from the simulation engine.
    /// TODO: Implement with actual gRPC streaming when proto files are ready.
    pub async fn stream_telemetry(&self, _run_id: &str) -> Result<()> {
        tracing::info!("SimEngineClient: stream_telemetry (placeholder)");
        Ok(())
    }
}
