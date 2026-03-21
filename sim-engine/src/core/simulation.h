#pragma once

#include "core/config.h"
#include "physics/physics_backend.h"
#include "robots/robot.h"
#include "environment/world.h"

#include <cstdint>
#include <memory>
#include <string>
#include <unordered_map>
#include <vector>

namespace amr::sim {

class Simulation {
public:
    explicit Simulation(const SimConfig& config);
    ~Simulation();

    /// Load a world definition from a JSON file.
    void loadWorld(const std::string& path);

    /// Spawn a robot at the given pose; returns a unique robot ID.
    uint64_t spawnRobot(double x, double y, double theta);

    /// Remove a robot by ID.
    void removeRobot(uint64_t robot_id);

    /// Send a velocity command to a robot.
    void sendCommand(uint64_t robot_id, double linear, double angular);

    /// Advance the simulation by one timestep.
    void step();

    /// Start the simulation loop (blocking).
    void start();

    /// Stop the simulation loop.
    void stop();

    /// Query the pose of a robot.
    Pose2D getRobotPose(uint64_t robot_id) const;

    /// Access the world.
    const World& getWorld() const { return world_; }

    /// Check if simulation is running.
    bool isRunning() const { return running_; }

private:
    SimConfig config_;
    std::unique_ptr<PhysicsBackend> physics_;
    World world_;
    std::unordered_map<uint64_t, Robot> robots_;
    std::unordered_map<uint64_t, BodyHandle> robot_bodies_;
    uint64_t next_robot_id_ = 1;
    bool running_ = false;
};

}  // namespace amr::sim
