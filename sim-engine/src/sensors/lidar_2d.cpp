#include "sensors/lidar_2d.h"
#include "environment/world.h"

#include <cmath>
#include <random>

namespace amr::sim {

Lidar2D::Lidar2D(const Lidar2DConfig& config)
    : config_(config)
    , ranges_(config.num_rays, config.max_range) {}

void Lidar2D::setWorldContext(const World* world, const std::vector<RobotCircle>& others) {
    world_ = world;
    other_robots_ = others;
}

void Lidar2D::update(double dt) {
    time_accumulator_ += dt;

    double update_period = 1.0 / config_.update_rate_hz;
    if (time_accumulator_ < update_period) {
        return;  // Not time to update yet
    }
    time_accumulator_ -= update_period;

    performScan();
}

void Lidar2D::performScan() {
    // Thread-local RNG for Gaussian noise
    thread_local std::mt19937 rng{std::random_device{}()};
    std::normal_distribution<double> noise_dist(0.0, config_.noise_stddev);

    double angle_step = (config_.num_rays > 1)
        ? (config_.max_angle - config_.min_angle) / config_.num_rays
        : 0.0;

    double ox = sensor_pose_.x;
    double oy = sensor_pose_.y;

    for (int i = 0; i < config_.num_rays; ++i) {
        double angle = sensor_pose_.theta + config_.min_angle + angle_step * (i + 0.5);
        double dx = std::cos(angle);
        double dy = std::sin(angle);

        double min_dist = config_.max_range;

        // Check walls
        if (world_) {
            for (const auto& wall : world_->getWalls()) {
                double t = raySegmentIntersect(ox, oy, dx, dy,
                                               wall.x1, wall.y1, wall.x2, wall.y2);
                if (t >= 0.0 && t < min_dist) {
                    min_dist = t;
                }
            }

            // Check obstacles (boxes treated as 4 line segments)
            for (const auto& obs : world_->getObstacles()) {
                double hx = obs.size_x * 0.5;
                double hy = obs.size_y * 0.5;

                // Four corners of the box
                double corners[4][2] = {
                    {obs.x - hx, obs.y - hy},
                    {obs.x + hx, obs.y - hy},
                    {obs.x + hx, obs.y + hy},
                    {obs.x - hx, obs.y + hy}
                };

                for (int e = 0; e < 4; ++e) {
                    int next = (e + 1) % 4;
                    double t = raySegmentIntersect(ox, oy, dx, dy,
                                                   corners[e][0], corners[e][1],
                                                   corners[next][0], corners[next][1]);
                    if (t >= 0.0 && t < min_dist) {
                        min_dist = t;
                    }
                }
            }
        }

        // Check other robots (circles)
        for (const auto& rc : other_robots_) {
            double t = rayCircleIntersect(ox, oy, dx, dy, rc.x, rc.y, rc.radius);
            if (t >= 0.0 && t < min_dist) {
                min_dist = t;
            }
        }

        // Add Gaussian noise (clamp to [0, max_range])
        double noisy = min_dist + noise_dist(rng);
        if (noisy < 0.0) noisy = 0.0;
        if (noisy > config_.max_range) noisy = config_.max_range;
        ranges_[i] = noisy;
    }
}

std::vector<double> Lidar2D::getData() const {
    return ranges_;
}

double Lidar2D::raySegmentIntersect(double ox, double oy, double dx, double dy,
                                     double x1, double y1, double x2, double y2) {
    // Ray: P = O + t * D,  t >= 0
    // Segment: Q = P1 + u * (P2 - P1),  u in [0,1]
    // Solve: O + t*D = P1 + u*(P2-P1)
    //   t*D - u*(P2-P1) = P1 - O
    // [dx  -(x2-x1)] [t]   [x1-ox]
    // [dy  -(y2-y1)] [u] = [y1-oy]

    double sx = x2 - x1;
    double sy = y2 - y1;

    double denom = dx * (-sy) - dy * (-sx);  // dx*(-sy) + dy*sx
    // = dy*sx - dx*sy  ... let me redo:
    // denom = dx * (-(y2-y1)) - dy * (-(x2-x1))
    //       = -dx*(y2-y1) + dy*(x2-x1)
    //       = dy*sx - dx*sy

    if (std::abs(denom) < 1e-12) {
        return -1.0;  // Parallel
    }

    double bx = x1 - ox;
    double by = y1 - oy;

    // Using Cramer's rule:
    // t = (bx*(-sy) - by*(-sx)) / denom = (-bx*sy + by*sx) / denom = (by*sx - bx*sy) / denom
    // u = (dx*by - dy*bx) / denom

    double t = (by * sx - bx * sy) / denom;
    double u = (dx * by - dy * bx) / denom;

    if (t >= 0.0 && u >= 0.0 && u <= 1.0) {
        return t;
    }
    return -1.0;
}

double Lidar2D::rayCircleIntersect(double ox, double oy, double dx, double dy,
                                    double cx, double cy, double r) {
    // Ray: P = O + t*D
    // Circle: |P - C|^2 = r^2
    // |O + t*D - C|^2 = r^2
    // Let f = O - C
    // |f + t*D|^2 = r^2
    // (D.D)*t^2 + 2*(f.D)*t + (f.f - r^2) = 0

    double fx = ox - cx;
    double fy = oy - cy;

    double a = dx * dx + dy * dy;  // Should be 1.0 for unit direction, but be safe
    double b = 2.0 * (fx * dx + fy * dy);
    double c = fx * fx + fy * fy - r * r;

    double discriminant = b * b - 4.0 * a * c;
    if (discriminant < 0.0) {
        return -1.0;
    }

    double sqrt_disc = std::sqrt(discriminant);
    double t1 = (-b - sqrt_disc) / (2.0 * a);
    double t2 = (-b + sqrt_disc) / (2.0 * a);

    // We want the smallest positive t
    if (t1 >= 0.0) return t1;
    if (t2 >= 0.0) return t2;
    return -1.0;  // Both behind the ray origin (inside circle or behind)
}

}  // namespace amr::sim
