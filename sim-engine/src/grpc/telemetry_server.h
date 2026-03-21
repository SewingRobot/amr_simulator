#pragma once

#include "physics/physics_backend.h"  // Pose2D, Velocity2D

#include <nlohmann/json.hpp>

#include <atomic>
#include <mutex>
#include <string>
#include <thread>
#include <vector>

namespace amr::sim {

/// Telemetry snapshot for a single robot, broadcast to connected TCP clients.
struct RobotTelemetry {
    std::string robot_id;
    double timestamp_ms = 0.0;
    uint64_t sequence_number = 0;
    Pose2D pose{0.0, 0.0, 0.0};
    Velocity2D velocity{0.0, 0.0};
    double battery_percent = 100.0;
    std::string status = "idle";  // "idle", "busy", "error"
    std::vector<double> lidar_ranges;  // LiDAR range data (possibly downsampled)

    nlohmann::json toJson() const;
};

/// Simple TCP server that accepts connections and streams JSON telemetry.
///
/// Each connected client receives newline-delimited JSON messages containing
/// the telemetry for all robots.  The server runs its accept loop on a
/// background thread and is safe to call from any thread.
class TelemetryServer {
public:
    explicit TelemetryServer(int port);
    ~TelemetryServer();

    /// Start listening for connections on a background thread.
    void start();

    /// Stop the server and close all connections.
    void stop();

    /// Broadcast a telemetry snapshot to every connected client.
    /// Messages are newline-delimited JSON arrays.
    void broadcastTelemetry(const std::vector<RobotTelemetry>& telemetry);

    /// Return the number of currently connected clients.
    size_t clientCount() const;

    /// Return the port the server is configured to listen on.
    int port() const { return m_port; }

private:
    void acceptLoop();
    void removeDisconnectedClients();

    int m_port;
    int m_serverFd = -1;
    std::atomic<bool> m_running{false};
    std::thread m_acceptThread;

    mutable std::mutex m_clientsMutex;
    std::vector<int> m_clients;
};

}  // namespace amr::sim
