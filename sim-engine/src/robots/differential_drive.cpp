#include "robots/differential_drive.h"

#include <cmath>
#include <algorithm>

namespace amr::sim {

DifferentialDrive::DifferentialDrive(const DriveConfig& config)
    : config_(config) {}

void DifferentialDrive::setCommand(double linear, double angular) {
    cmd_linear_ = clamp(linear, config_.max_linear_speed);
    cmd_angular_ = clamp(angular, config_.max_angular_speed);
}

void DifferentialDrive::update(double dt) {
    double v = cmd_linear_;
    double w = cmd_angular_;

    // Update orientation
    pose_.theta += w * dt;

    // Normalize theta to [-pi, pi]
    while (pose_.theta > M_PI) pose_.theta -= 2.0 * M_PI;
    while (pose_.theta < -M_PI) pose_.theta += 2.0 * M_PI;

    // Update position along heading
    pose_.x += v * std::cos(pose_.theta) * dt;
    pose_.y += v * std::sin(pose_.theta) * dt;
}

std::pair<double, double> DifferentialDrive::getWheelSpeeds() const {
    double L = config_.wheel_separation;
    double v_left  = cmd_linear_ - cmd_angular_ * L / 2.0;
    double v_right = cmd_linear_ + cmd_angular_ * L / 2.0;
    return {v_left, v_right};
}

double DifferentialDrive::clamp(double val, double limit) const {
    return std::clamp(val, -limit, limit);
}

}  // namespace amr::sim
