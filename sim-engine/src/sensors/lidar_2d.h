#pragma once

#include "sensors/sensor.h"
#include "physics/physics_backend.h"  // Pose2D

#include <vector>

namespace amr::sim {

struct Lidar2DConfig {
    double min_angle = -M_PI;        // radians
    double max_angle = M_PI;         // radians
    int num_rays = 360;
    double max_range = 12.0;         // meters
    double noise_stddev = 0.01;      // meters (Gaussian)
    double update_rate_hz = 10.0;    // Hz
};

/// 2D LiDAR sensor.
/// Currently a skeleton; ray casting will be implemented in Week 5-6.
class Lidar2D : public Sensor {
public:
    explicit Lidar2D(const Lidar2DConfig& config = {});

    void update(double dt) override;
    std::vector<double> getData() const override;
    std::string getType() const override { return "lidar_2d"; }

    /// Set the sensor's pose in the world (from the parent robot).
    void setSensorPose(const Pose2D& pose) { sensor_pose_ = pose; }

    const Lidar2DConfig& getConfig() const { return config_; }

private:
    Lidar2DConfig config_;
    Pose2D sensor_pose_{0.0, 0.0, 0.0};
    std::vector<double> ranges_;
    double time_accumulator_ = 0.0;
};

}  // namespace amr::sim
