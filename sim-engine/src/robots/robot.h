#pragma once

#include "robots/differential_drive.h"

#include <string>

namespace amr::sim {

struct RobotConfig {
    std::string name = "amr_robot";
    DriveConfig drive;
    double collision_radius = 0.25;  // meters
    double battery_capacity = 100.0; // percent
    double battery_drain_rate = 0.01; // percent per second while moving
};

class Robot {
public:
    explicit Robot(const RobotConfig& config = {});

    /// Update robot state for one timestep.
    void update(double dt);

    /// Access the drive model.
    DifferentialDrive& getDrive() { return drive_; }
    const DifferentialDrive& getDrive() const { return drive_; }

    /// Battery level (0-100%).
    double getBattery() const { return battery_; }

    const RobotConfig& getConfig() const { return config_; }

private:
    RobotConfig config_;
    DifferentialDrive drive_;
    double battery_;
};

}  // namespace amr::sim
