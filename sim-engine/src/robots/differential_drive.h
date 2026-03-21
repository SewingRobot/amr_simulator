#pragma once

#include "physics/physics_backend.h"  // Pose2D, Velocity2D

namespace amr::sim {

struct DriveConfig {
    double wheel_radius = 0.05;       // meters
    double wheel_separation = 0.3;    // meters (track width L)
    double max_linear_speed = 1.0;    // m/s
    double max_angular_speed = 2.0;   // rad/s
};

/// Differential drive kinematics model.
///
/// Converts (v, omega) commands into wheel velocities and integrates pose.
/// Kinematics:
///   v_left  = v - omega * L / 2
///   v_right = v + omega * L / 2
///   dx/dt = v * cos(theta)
///   dy/dt = v * sin(theta)
///   dtheta/dt = omega
class DifferentialDrive {
public:
    explicit DifferentialDrive(const DriveConfig& config = {});

    /// Set desired linear and angular velocity command.
    void setCommand(double linear, double angular);

    /// Integrate kinematics for dt seconds.
    void update(double dt);

    /// Get current pose.
    Pose2D getPose() const { return pose_; }

    /// Set pose (e.g., from physics sync).
    void setPose(const Pose2D& pose) { pose_ = pose; }

    /// Get current velocity command (clamped).
    Velocity2D getVelocity() const { return {cmd_linear_, cmd_angular_}; }

    /// Get left/right wheel speeds (m/s at wheel rim).
    std::pair<double, double> getWheelSpeeds() const;

    const DriveConfig& getConfig() const { return config_; }

private:
    DriveConfig config_;
    Pose2D pose_{0.0, 0.0, 0.0};
    double cmd_linear_ = 0.0;
    double cmd_angular_ = 0.0;

    double clamp(double val, double limit) const;
};

}  // namespace amr::sim
