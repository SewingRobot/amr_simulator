#pragma once

#include "robots/differential_drive.h"
#include "sensors/lidar_2d.h"

#include <string>
#include <vector>

namespace amr::sim {

struct RobotConfig {
    std::string name = "amr_robot";
    DriveConfig drive;
    Lidar2DConfig lidar;
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

    /// Access the LiDAR sensor.
    Lidar2D& getLidar() { return lidar_; }
    const Lidar2D& getLidar() const { return lidar_; }

    /// Get the latest LiDAR range data.
    std::vector<double> getLidarData() const { return lidar_.getData(); }

    /// Battery level (0-100%).
    double getBattery() const { return battery_; }

    const RobotConfig& getConfig() const { return config_; }

private:
    RobotConfig config_;
    DifferentialDrive drive_;
    Lidar2D lidar_;
    double battery_;
};

}  // namespace amr::sim
