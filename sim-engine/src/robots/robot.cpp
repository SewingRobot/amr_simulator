#include "robots/robot.h"

#include <algorithm>
#include <cmath>

namespace amr::sim {

Robot::Robot(const RobotConfig& config)
    : config_(config)
    , drive_(config.drive)
    , lidar_(config.lidar)
    , battery_(config.battery_capacity) {}

void Robot::update(double dt) {
    drive_.update(dt);

    // Sync lidar pose with drive pose
    lidar_.setSensorPose(drive_.getPose());
    lidar_.update(dt);

    // Linear battery depletion when moving
    auto vel = drive_.getVelocity();
    double speed = std::abs(vel.linear) + std::abs(vel.angular);
    if (speed > 1e-6) {
        battery_ -= config_.battery_drain_rate * dt;
        battery_ = std::max(0.0, battery_);
    }
}

}  // namespace amr::sim
