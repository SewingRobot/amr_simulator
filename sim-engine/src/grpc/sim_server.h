#pragma once

namespace amr::sim {

/// Placeholder for the gRPC simulation server.
/// Will implement:
///   - CreateSim / StartSim / StopSim
///   - SpawnRobot / RemoveRobot
///   - SendCommand
///   - StreamTelemetry (server-streaming, 10Hz per robot)
///
/// Implementation deferred until proto definitions are frozen (Week 2+).

class SimServer {
public:
    // TODO: Implement when proto files are available.
};

}  // namespace amr::sim
