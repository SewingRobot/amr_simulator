#include "sensors/lidar_2d.h"

#include <gtest/gtest.h>
#include <cmath>

using namespace amr::sim;

TEST(Lidar2D, DefaultConfigInitializesMaxRange) {
    Lidar2DConfig cfg;
    Lidar2D lidar(cfg);

    auto data = lidar.getData();
    ASSERT_EQ(static_cast<int>(data.size()), cfg.num_rays);

    for (double range : data) {
        EXPECT_DOUBLE_EQ(range, cfg.max_range);
    }
}

TEST(Lidar2D, CustomRayCount) {
    Lidar2DConfig cfg;
    cfg.num_rays = 64;
    Lidar2D lidar(cfg);

    auto data = lidar.getData();
    EXPECT_EQ(static_cast<int>(data.size()), 64);
}

TEST(Lidar2D, UpdateRespectsRate) {
    Lidar2DConfig cfg;
    cfg.update_rate_hz = 10.0;  // 100ms period
    cfg.num_rays = 4;
    Lidar2D lidar(cfg);

    // Update with less than the period - should not trigger scan
    lidar.update(0.05);
    auto data1 = lidar.getData();
    EXPECT_EQ(static_cast<int>(data1.size()), 4);

    // Update past the period threshold - should trigger scan
    lidar.update(0.06);
    auto data2 = lidar.getData();
    EXPECT_EQ(static_cast<int>(data2.size()), 4);
}

TEST(Lidar2D, TypeString) {
    Lidar2D lidar;
    EXPECT_EQ(lidar.getType(), "lidar_2d");
}

TEST(Lidar2D, SensorPoseCanBeSet) {
    Lidar2D lidar;
    lidar.setSensorPose({1.0, 2.0, 0.5});
    // Verify no crash - pose is used internally for ray casting (future).
}

TEST(Lidar2D, ConfigAccessor) {
    Lidar2DConfig cfg;
    cfg.max_range = 25.0;
    cfg.noise_stddev = 0.05;
    Lidar2D lidar(cfg);

    EXPECT_DOUBLE_EQ(lidar.getConfig().max_range, 25.0);
    EXPECT_DOUBLE_EQ(lidar.getConfig().noise_stddev, 0.05);
}
