#include "sensors/lidar_2d.h"

#include <cmath>

namespace amr::sim {

Lidar2D::Lidar2D(const Lidar2DConfig& config)
    : config_(config)
    , ranges_(config.num_rays, config.max_range) {}

void Lidar2D::update(double dt) {
    time_accumulator_ += dt;

    double update_period = 1.0 / config_.update_rate_hz;
    if (time_accumulator_ < update_period) {
        return;  // Not time to update yet
    }
    time_accumulator_ -= update_period;

    // TODO (Week 5-6): Perform actual ray casting against world geometry.
    // For now, all rays return max_range (no obstacles).
    for (int i = 0; i < config_.num_rays; ++i) {
        ranges_[i] = config_.max_range;
    }
}

std::vector<double> Lidar2D::getData() const {
    return ranges_;
}

}  // namespace amr::sim
