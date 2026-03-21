#pragma once

#include "grpc/telemetry_server.h"
#include "grpc/command_server.h"

#include <memory>
#include <string>
#include <unordered_map>

namespace amr::sim {

class Simulation;  // forward declaration

/// High-level facade that owns the TelemetryServer and CommandServer and
/// wires incoming commands into the Simulation instance.
class SimServer {
public:
    /// Create the server pair.  The telemetry port streams robot state;
    /// the command port accepts JSON commands.
    SimServer(int telemetry_port, int command_port, Simulation& sim);
    ~SimServer();

    /// Start both servers on background threads.
    void start();

    /// Stop both servers.
    void stop();

    /// Convenience: broadcast current telemetry from the simulation.
    void broadcastCurrentTelemetry();

    TelemetryServer& telemetryServer() { return m_telemetry; }
    CommandServer&   commandServer()   { return m_command; }

private:
    /// Handler wired to the CommandServer; dispatches to Simulation methods.
    std::string handleCommand(const SimCommand& cmd);

    TelemetryServer m_telemetry;
    CommandServer   m_command;
    Simulation&     m_sim;
    std::unordered_map<std::string, uint64_t> m_robotIdMap;  // string ID → internal ID
};

}  // namespace amr::sim
