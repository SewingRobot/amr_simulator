#include "core/simulation.h"
#include "physics/backend_factory.h"

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
    auto it = robots_.find(robot_id);
    if (it == robots_.end()) {
        spdlog::warn("sendCommand: robot {} not found", robot_id);
        return;
    }
    it->second.getDrive().setCommand(linear, angular);
}

void Simulation::step() {
    double dt = config_.timestep_s;

    // Update robot kinematics
    for (auto& [id, robot] : robots_) {
        robot.update(dt);
        Pose2D pose = robot.getDrive().getPose();
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

void Simulation::start() {
    spdlog::info("Simulation starting");
    running_ = true;

    auto step_duration = std::chrono::duration<double>(config_.timestep_s / config_.realtime_factor);

    while (running_) {
        auto t0 = std::chrono::steady_clock::now();
        step();
        auto t1 = std::chrono::steady_clock::now();

        auto elapsed = t1 - t0;
        if (elapsed < step_duration) {
            std::this_thread::sleep_for(step_duration - elapsed);
        }
    }
}

void Simulation::stop() {
    if (running_) {
        spdlog::info("Simulation stopping");
        running_ = false;
    }
}

Pose2D Simulation::getRobotPose(uint64_t robot_id) const {
    auto it = robots_.find(robot_id);
    if (it == robots_.end()) {
        spdlog::warn("getRobotPose: robot {} not found", robot_id);
        return {0.0, 0.0, 0.0};
    }
    return it->second.getDrive().getPose();
}

}  // namespace amr::sim
