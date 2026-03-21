#include "core/simulation.h"
#include "physics/backend_factory.h"
#include "sensors/lidar_2d.h"

#include <spdlog/spdlog.h>
#include <chrono>
#include <thread>

namespace amr::sim {

Simulation::Simulation(const SimConfig& config)
    : config_(config)
    , physics_(BackendFactory::create(config.physics_backend)) {
    physics_->init();
    spdlog::info("Simulation created with backend '{}'", config.physics_backend);
}

Simulation::~Simulation() {
    stop();
    if (physics_) {
        physics_->shutdown();
    }
}

void Simulation::loadWorld(const std::string& path) {
    spdlog::info("Loading world from '{}'", path);
    world_.loadFromFile(path);
}

uint64_t Simulation::spawnRobot(double x, double y, double theta) {
    std::lock_guard<std::mutex> lock(robots_mutex_);

    uint64_t id = next_robot_id_++;

    RobotConfig rcfg;
    Robot robot(rcfg);
    robot.getDrive().setPose({x, y, theta});

    BodyHandle body = physics_->addBody(x, y, rcfg.collision_radius);

    robots_.emplace(id, std::move(robot));
    robot_bodies_.emplace(id, body);

    spdlog::info("Spawned robot {} at ({}, {}, {})", id, x, y, theta);
    return id;
}

void Simulation::removeRobot(uint64_t robot_id) {
    std::lock_guard<std::mutex> lock(robots_mutex_);

    auto it = robots_.find(robot_id);
    if (it == robots_.end()) {
        spdlog::warn("removeRobot: robot {} not found", robot_id);
        return;
    }
    physics_->removeBody(robot_bodies_.at(robot_id));
    robots_.erase(it);
    robot_bodies_.erase(robot_id);
    spdlog::info("Removed robot {}", robot_id);
}

void Simulation::sendCommand(uint64_t robot_id, double linear, double angular) {
    std::lock_guard<std::mutex> lock(robots_mutex_);

    auto it = robots_.find(robot_id);
    if (it == robots_.end()) {
        spdlog::warn("sendCommand: robot {} not found", robot_id);
        return;
    }
    it->second.getDrive().setCommand(linear, angular);
}

void Simulation::step() {
    auto step_t0 = std::chrono::steady_clock::now();

    double dt = config_.timestep_s;

    // Process any queued commands before stepping
    processCommandQueue();

    {
        std::lock_guard<std::mutex> lock(robots_mutex_);

        // Build world context for LiDAR: for each robot, provide circles of all other robots
        for (auto& [id, robot] : robots_) {
            std::vector<RobotCircle> others;
            others.reserve(robots_.size() > 0 ? robots_.size() - 1 : 0);
            for (const auto& [oid, other] : robots_) {
                if (oid == id) continue;
                Pose2D op = other.getDrive().getPose();
                others.push_back({op.x, op.y, other.getConfig().collision_radius});
            }
            robot.getLidar().setWorldContext(&world_, others);
        }

        // Update robot kinematics (also updates LiDAR)
        for (auto& [id, robot] : robots_) {
            robot.update(dt);
            Velocity2D vel = robot.getDrive().getVelocity();
            physics_->setBodyVelocity(robot_bodies_.at(id), vel);
        }

        // Step physics
        physics_->step(dt);

        // Sync physics poses back to robots
        for (auto& [id, robot] : robots_) {
            Pose2D pose = physics_->getBodyPose(robot_bodies_.at(id));
            robot.getDrive().setPose(pose);
        }
    }

    sim_time_ += dt;

    auto step_t1 = std::chrono::steady_clock::now();
    double elapsed_ms = std::chrono::duration<double, std::milli>(step_t1 - step_t0).count();
    spdlog::debug("Step time: {:.3f}ms", elapsed_ms);
}

void Simulation::start() {
    spdlog::info("Simulation starting");
    running_ = true;

    auto step_duration = std::chrono::duration<double>(config_.timestep_s / config_.realtime_factor);

    // Telemetry broadcast interval: 10 Hz
    constexpr double telemetry_interval_s = 0.1;
    double telemetry_accum = 0.0;

    while (running_.load()) {
        auto t0 = std::chrono::steady_clock::now();
        step();

        // Fire post-step callback at ~10 Hz
        telemetry_accum += config_.timestep_s;
        if (telemetry_accum >= telemetry_interval_s) {
            telemetry_accum -= telemetry_interval_s;
            if (post_step_callback_) {
                post_step_callback_();
            }
        }

        auto t1 = std::chrono::steady_clock::now();
        auto elapsed = t1 - t0;
        if (elapsed < step_duration) {
            std::this_thread::sleep_for(step_duration - elapsed);
        }
    }
}

void Simulation::startAsync() {
    if (running_.load()) return;

    sim_thread_ = std::thread([this]() {
        start();
    });
}

void Simulation::stop() {
    if (running_.load()) {
        spdlog::info("Simulation stopping");
        running_ = false;

        if (sim_thread_.joinable()) {
            sim_thread_.join();
        }
    }
}

Pose2D Simulation::getRobotPose(uint64_t robot_id) const {
    std::lock_guard<std::mutex> lock(robots_mutex_);

    auto it = robots_.find(robot_id);
    if (it == robots_.end()) {
        spdlog::warn("getRobotPose: robot {} not found", robot_id);
        return {0.0, 0.0, 0.0};
    }
    return it->second.getDrive().getPose();
}

std::vector<RobotTelemetry> Simulation::getTelemetry() const {
    std::lock_guard<std::mutex> lock(robots_mutex_);

    auto now = std::chrono::system_clock::now();
    double ts_ms = static_cast<double>(
        std::chrono::duration_cast<std::chrono::milliseconds>(
            now.time_since_epoch()).count());

    std::vector<RobotTelemetry> result;
    result.reserve(robots_.size());

    for (const auto& [id, robot] : robots_) {
        RobotTelemetry t;
        t.robot_id = std::to_string(id);
        t.timestamp_ms = ts_ms;
        t.sequence_number = telemetry_seq_;
        t.pose = robot.getDrive().getPose();
        t.velocity = robot.getDrive().getVelocity();
        t.battery_percent = robot.getBattery();

        t.lidar_ranges = robot.getLidarData();

        // Determine status based on velocity
        double speed = std::abs(t.velocity.linear) + std::abs(t.velocity.angular);
        t.status = (speed > 1e-6) ? "busy" : "idle";

        result.push_back(std::move(t));
    }

    // Increment sequence (mutable would be cleaner but const_cast is fine here)
    const_cast<Simulation*>(this)->telemetry_seq_++;

    return result;
}

void Simulation::setPostStepCallback(StepCallback cb) {
    post_step_callback_ = std::move(cb);
}

void Simulation::enqueueCommand(const QueuedCommand& cmd) {
    std::lock_guard<std::mutex> lock(cmd_queue_mutex_);
    cmd_queue_.push(cmd);
}

void Simulation::processCommandQueue() {
    std::lock_guard<std::mutex> lock(cmd_queue_mutex_);
    while (!cmd_queue_.empty()) {
        auto cmd = cmd_queue_.front();
        cmd_queue_.pop();

        switch (cmd.type) {
            case QueuedCommand::Type::SpawnRobot:
                spawnRobot(cmd.x, cmd.y, cmd.theta);
                break;
            case QueuedCommand::Type::RemoveRobot:
                removeRobot(cmd.robot_id);
                break;
            case QueuedCommand::Type::SendCommand:
                sendCommand(cmd.robot_id, cmd.linear, cmd.angular);
                break;
        }
    }
}

}  // namespace amr::sim
