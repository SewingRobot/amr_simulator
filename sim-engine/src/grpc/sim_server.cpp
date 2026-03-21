#include "grpc/sim_server.h"
#include "core/simulation.h"

#include <nlohmann/json.hpp>
#include <spdlog/spdlog.h>

namespace amr::sim {

SimServer::SimServer(int telemetry_port, int command_port, Simulation& sim)
    : m_telemetry(telemetry_port)
    , m_command(command_port)
    , m_sim(sim) {}

SimServer::~SimServer() {
    stop();
}

void SimServer::start() {
    m_command.setHandler([this](const SimCommand& cmd) {
        return handleCommand(cmd);
    });
    m_telemetry.start();
    m_command.start();
    spdlog::info("SimServer started (telemetry={}, command={})",
                 m_telemetry.port(), m_command.port());
}

void SimServer::stop() {
    m_telemetry.stop();
    m_command.stop();
    spdlog::info("SimServer stopped");
}

void SimServer::broadcastCurrentTelemetry() {
    auto telemetry = m_sim.getTelemetry();
    // Map internal IDs back to string IDs
    for (auto& t : telemetry) {
        for (const auto& [str_id, internal_id] : m_robotIdMap) {
            if (std::to_string(internal_id) == t.robot_id) {
                t.robot_id = str_id;
                break;
            }
        }
    }
    m_telemetry.broadcastTelemetry(telemetry);
}

std::string SimServer::handleCommand(const SimCommand& cmd) {
    nlohmann::json resp;

    if (cmd.type == "spawn_robot") {
        uint64_t internal_id = m_sim.spawnRobot(cmd.x, cmd.y, cmd.theta);
        std::string str_id = cmd.robot_id.empty() ? std::to_string(internal_id) : cmd.robot_id;
        m_robotIdMap[str_id] = internal_id;
        resp["success"] = true;
        resp["robot_id"] = str_id;
        resp["internal_id"] = internal_id;

    } else if (cmd.type == "remove_robot") {
        auto it = m_robotIdMap.find(cmd.robot_id);
        if (it != m_robotIdMap.end()) {
            m_sim.removeRobot(it->second);
            m_robotIdMap.erase(it);
            resp["success"] = true;
        } else {
            resp["success"] = false;
            resp["error"] = "robot not found: " + cmd.robot_id;
        }

    } else if (cmd.type == "send_command") {
        auto it = m_robotIdMap.find(cmd.robot_id);
        if (it != m_robotIdMap.end()) {
            m_sim.sendCommand(it->second, cmd.linear, cmd.angular);
            resp["success"] = true;
        } else {
            resp["success"] = false;
            resp["error"] = "robot not found: " + cmd.robot_id;
        }

    } else if (cmd.type == "start_sim") {
        m_sim.startAsync();
        resp["success"] = true;

    } else if (cmd.type == "stop_sim") {
        m_sim.stop();
        resp["success"] = true;

    } else if (cmd.type == "get_state") {
        auto telemetry = m_sim.getTelemetry();
        resp["running"] = m_sim.isRunning();
        resp["robot_count"] = telemetry.size();
        nlohmann::json robots = nlohmann::json::array();
        for (const auto& t : telemetry) {
            robots.push_back(t.toJson());
        }
        resp["robots"] = robots;

    } else {
        resp["error"] = "unknown command type: " + cmd.type;
    }

    return resp.dump();
}

}  // namespace amr::sim
