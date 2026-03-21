#pragma once

#include "core/config.h"
#include "physics/physics_backend.h"
#include "robots/robot.h"
#include "environment/world.h"
#include "grpc/telemetry_server.h"  // RobotTelemetry

#include <atomic>
#include <chrono>
#include <cstdint>
#include <functional>
#include <memory>
#include <mutex>
#include <queue>
#include <string>
#include <thread>
#include <unordered_map>
#include <vector>

namespace amr::sim {

/// Queued command from an external source (e.g. CommandServer).
struct QueuedCommand {
    enum class Type { SpawnRobot, RemoveRobot, SendCommand };
    Type type;
    uint64_t robot_id = 0;
    double x = 0, y = 0, theta = 0;
    double linear = 0, angular = 0;
};

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

    /// Start the simulation loop on a background thread (non-blocking).
    void startAsync();

    /// Stop the simulation loop.
    void stop();

    /// Query the pose of a robot.
    Pose2D getRobotPose(uint64_t robot_id) const;

    /// Get a telemetry snapshot for every robot.
    std::vector<RobotTelemetry> getTelemetry() const;

    /// Access the world.
    const World& getWorld() const { return world_; }

    /// Check if simulation is running.
    bool isRunning() const { return running_.load(); }

    /// Get current simulation time in seconds.
    double simTime() const { return sim_time_; }

    /// Set a callback invoked after every step (e.g. to broadcast telemetry).
    using StepCallback = std::function<void()>;
    void setPostStepCallback(StepCallback cb);

    /// Enqueue a command for thread-safe processing during the next step.
    void enqueueCommand(const QueuedCommand& cmd);

private:
    void processCommandQueue();

    SimConfig config_;
    std::unique_ptr<PhysicsBackend> physics_;
    World world_;

    mutable std::mutex robots_mutex_;
    std::unordered_map<uint64_t, Robot> robots_;
    std::unordered_map<uint64_t, BodyHandle> robot_bodies_;
    uint64_t next_robot_id_ = 1;

    std::atomic<bool> running_{false};
    double sim_time_ = 0.0;
    uint64_t telemetry_seq_ = 0;

    std::thread sim_thread_;

    std::mutex cmd_queue_mutex_;
    std::queue<QueuedCommand> cmd_queue_;

    StepCallback post_step_callback_;
};

}  // namespace amr::sim
