#pragma once

#include "sensors/sensor.h"
#include "physics/physics_backend.h"  // Pose2D

#include <vector>

namespace amr::sim {

class World;

struct RobotCircle {
    double x, y, radius;
};

struct Lidar2DConfig {
    double min_angle = -M_PI;        // radians
    double max_angle = M_PI;         // radians
    int num_rays = 360;
    double max_range = 12.0;         // meters
    double noise_stddev = 0.01;      // meters (Gaussian)
    double update_rate_hz = 10.0;    // Hz
};

/// 2D LiDAR sensor with ray casting against walls, obstacles, and other robots.
class Lidar2D : public Sensor {
public:
    explicit Lidar2D(const Lidar2DConfig& config = {});

    void update(double dt) override;
    std::vector<double> getData() const override;
    std::string getType() const override { return "lidar_2d"; }

    /// Set the sensor's pose in the world (from the parent robot).
    void setSensorPose(const Pose2D& pose) { sensor_pose_ = pose; }

    /// Set the world context for ray casting.
    /// @param world       Pointer to the world (walls + obstacles).
    /// @param others      Circles representing other robots (excluding self).
    void setWorldContext(const World* world, const std::vector<RobotCircle>& others);

    const Lidar2DConfig& getConfig() const { return config_; }

private:
    /// Perform the ray cast scan.
    void performScan();

    /// Ray-line-segment intersection. Returns distance or negative if no hit.
    static double raySegmentIntersect(double ox, double oy, double dx, double dy,
                                      double x1, double y1, double x2, double y2);

    /// Ray-circle intersection. Returns distance or negative if no hit.
    static double rayCircleIntersect(double ox, double oy, double dx, double dy,
                                     double cx, double cy, double r);

    Lidar2DConfig config_;
    Pose2D sensor_pose_{0.0, 0.0, 0.0};
    std::vector<double> ranges_;
    double time_accumulator_ = 0.0;

    const World* world_ = nullptr;
    std::vector<RobotCircle> other_robots_;
};

}  // namespace amr::sim
