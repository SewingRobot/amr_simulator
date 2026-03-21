#include "robots/differential_drive.h"

#include <gtest/gtest.h>
#include <cmath>

using namespace amr::sim;

constexpr double kEpsilon = 1e-9;

TEST(DifferentialDrive, ZeroVelocityStaysInPlace) {
    DifferentialDrive drive;
    drive.setCommand(0.0, 0.0);

    Pose2D initial = drive.getPose();
    drive.update(1.0);
    Pose2D after = drive.getPose();

    EXPECT_NEAR(after.x, initial.x, kEpsilon);
    EXPECT_NEAR(after.y, initial.y, kEpsilon);
    EXPECT_NEAR(after.theta, initial.theta, kEpsilon);
}

TEST(DifferentialDrive, StraightLineForward) {
    DifferentialDrive drive;
    // Starting at origin, facing +x (theta=0)
    drive.setPose({0.0, 0.0, 0.0});
    drive.setCommand(1.0, 0.0);  // 1 m/s forward

    drive.update(1.0);  // 1 second
    Pose2D pose = drive.getPose();

    EXPECT_NEAR(pose.x, 1.0, kEpsilon);
    EXPECT_NEAR(pose.y, 0.0, kEpsilon);
    EXPECT_NEAR(pose.theta, 0.0, kEpsilon);
}

TEST(DifferentialDrive, StraightLineBackward) {
    DifferentialDrive drive;
    drive.setPose({0.0, 0.0, 0.0});
    drive.setCommand(-0.5, 0.0);

    drive.update(2.0);
    Pose2D pose = drive.getPose();

    EXPECT_NEAR(pose.x, -1.0, kEpsilon);
    EXPECT_NEAR(pose.y, 0.0, kEpsilon);
}

TEST(DifferentialDrive, PureRotation) {
    DifferentialDrive drive;
    drive.setPose({0.0, 0.0, 0.0});
    drive.setCommand(0.0, 1.0);  // 1 rad/s rotation

    drive.update(1.0);
    Pose2D pose = drive.getPose();

    // Should stay in place but rotate
    EXPECT_NEAR(pose.x, 0.0, kEpsilon);
    EXPECT_NEAR(pose.y, 0.0, kEpsilon);
    EXPECT_NEAR(pose.theta, 1.0, kEpsilon);
}

TEST(DifferentialDrive, FullRotation) {
    DifferentialDrive drive;
    drive.setPose({5.0, 5.0, 0.0});
    drive.setCommand(0.0, 2.0);  // 2 rad/s

    // Rotate for pi seconds -> theta = 2*pi, normalizes to ~0
    drive.update(M_PI);
    Pose2D pose = drive.getPose();

    EXPECT_NEAR(pose.x, 5.0, kEpsilon);
    EXPECT_NEAR(pose.y, 5.0, kEpsilon);
    // 2*pi normalized to approximately 0
    EXPECT_NEAR(std::abs(pose.theta), 0.0, 1e-6);
}

TEST(DifferentialDrive, WheelSpeedsSymmetric) {
    DifferentialDrive drive;
    drive.setCommand(1.0, 0.0);  // pure forward

    auto [vl, vr] = drive.getWheelSpeeds();
    EXPECT_NEAR(vl, vr, kEpsilon);
    EXPECT_NEAR(vl, 1.0, kEpsilon);
}

TEST(DifferentialDrive, WheelSpeedsDifferential) {
    DriveConfig cfg;
    cfg.wheel_separation = 0.4;
    DifferentialDrive drive(cfg);
    drive.setCommand(0.0, 1.0);  // pure rotation at 1 rad/s

    auto [vl, vr] = drive.getWheelSpeeds();
    // v_left = 0 - 1 * 0.4/2 = -0.2
    // v_right = 0 + 1 * 0.4/2 = 0.2
    EXPECT_NEAR(vl, -0.2, kEpsilon);
    EXPECT_NEAR(vr, 0.2, kEpsilon);
}

TEST(DifferentialDrive, VelocityClamping) {
    DriveConfig cfg;
    cfg.max_linear_speed = 0.5;
    cfg.max_angular_speed = 1.0;
    DifferentialDrive drive(cfg);

    drive.setCommand(10.0, 10.0);  // way above limits
    auto vel = drive.getVelocity();
    EXPECT_NEAR(vel.linear, 0.5, kEpsilon);
    EXPECT_NEAR(vel.angular, 1.0, kEpsilon);

    drive.setCommand(-10.0, -10.0);
    vel = drive.getVelocity();
    EXPECT_NEAR(vel.linear, -0.5, kEpsilon);
    EXPECT_NEAR(vel.angular, -1.0, kEpsilon);
}

TEST(DifferentialDrive, DiagonalMotion) {
    DifferentialDrive drive;
    drive.setPose({0.0, 0.0, M_PI / 4.0});  // facing 45 degrees
    drive.setCommand(1.0, 0.0);

    drive.update(1.0);
    Pose2D pose = drive.getPose();

    double expected = std::sqrt(2.0) / 2.0;
    EXPECT_NEAR(pose.x, expected, 1e-9);
    EXPECT_NEAR(pose.y, expected, 1e-9);
}

TEST(DifferentialDrive, MultipleSmallSteps) {
    // Verify that many small steps produce similar result to one large step
    // for straight-line motion (where there's no rotation coupling).
    DifferentialDrive drive1;
    drive1.setPose({0.0, 0.0, 0.0});
    drive1.setCommand(1.0, 0.0);
    drive1.update(1.0);

    DifferentialDrive drive2;
    drive2.setPose({0.0, 0.0, 0.0});
    drive2.setCommand(1.0, 0.0);
    for (int i = 0; i < 1000; ++i) {
        drive2.update(0.001);
    }

    EXPECT_NEAR(drive1.getPose().x, drive2.getPose().x, 1e-6);
    EXPECT_NEAR(drive1.getPose().y, drive2.getPose().y, 1e-6);
}
