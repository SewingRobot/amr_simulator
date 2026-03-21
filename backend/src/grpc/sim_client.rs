//! TCP client for the Simulation Engine.
//!
//! The Sim Engine exposes two TCP ports:
//! - Telemetry port (default 50051): streams newline-delimited JSON telemetry
//! - Command port (default 50052): accepts newline-delimited JSON commands

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast;

use super::telemetry::{SimCommand, TelemetryMessage};

/// Client for communicating with the Sim Engine over TCP.
#[derive(Debug, Clone)]
pub struct SimEngineClient {
    telemetry_addr: String,
    command_addr: String,
}

impl SimEngineClient {
    /// Create a new SimEngineClient with the given telemetry and command addresses.
    pub fn new(telemetry_addr: String, command_addr: String) -> Self {
        Self {
            telemetry_addr,
            command_addr,
        }
    }

    /// Connect to the sim engine telemetry port and stream parsed messages
    /// into the provided broadcast channel.
    ///
    /// This method runs indefinitely (until the TCP connection drops or
    /// the task is cancelled). It should be spawned as a background task.
    pub async fn stream_telemetry(
        &self,
        tx: broadcast::Sender<TelemetryMessage>,
    ) -> Result<()> {
        tracing::info!(
            "Connecting to Sim Engine telemetry at {}",
            self.telemetry_addr
        );

        let stream = TcpStream::connect(&self.telemetry_addr).await?;
        tracing::info!("Connected to Sim Engine telemetry stream");

        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }

            match serde_json::from_str::<TelemetryMessage>(&line) {
                Ok(msg) => {
                    tracing::trace!(
                        robot_id = %msg.robot_id,
                        "Received telemetry"
                    );
                    // send returns Err only when there are no receivers; that's fine
                    let _ = tx.send(msg);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse telemetry JSON: {}", e);
                }
            }
        }

        tracing::warn!("Sim Engine telemetry stream closed");
        Ok(())
    }

    /// Send a command to the sim engine via its command port.
    pub async fn send_command(&self, cmd: &SimCommand) -> Result<()> {
        tracing::debug!(
            "Sending command to Sim Engine at {}: {:?}",
            self.command_addr,
            cmd
        );

        let mut stream = TcpStream::connect(&self.command_addr).await?;
        let mut payload = serde_json::to_vec(cmd)?;
        payload.push(b'\n');
        stream.write_all(&payload).await?;
        stream.flush().await?;

        Ok(())
    }
}
