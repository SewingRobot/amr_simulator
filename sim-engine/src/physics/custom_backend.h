#pragma once

#include "physics/physics_backend.h"

#include <cstdint>
#include <unordered_map>
#include <vector>

namespace amr::sim {

/// Lightweight custom 2D physics backend.
/// Manages circular rigid bodies with velocity integration and AABB collision detection.
class CustomLightweightBackend : public PhysicsBackend {
public:
    void init() override;
    void shutdown() override;
    void step(double dt) override;
    BodyHandle addBody(double x, double y, double radius) override;
    void removeBody(BodyHandle handle) override;
    void setBodyVelocity(BodyHandle handle, Velocity2D vel) override;
    Pose2D getBodyPose(BodyHandle handle) const override;
    Velocity2D getBodyVelocity(BodyHandle handle) const override;
    std::vector<CollisionInfo> getCollisions() const override;
    std::string getName() const override { return "custom_lightweight"; }

private:
    struct Body {
        Pose2D pose{0.0, 0.0, 0.0};
        Velocity2D velocity{0.0, 0.0};
        double radius = 0.0;
    };

    std::unordered_map<uint64_t, Body> bodies_;
    std::vector<CollisionInfo> collisions_;
    uint64_t next_id_ = 1;

    void detectCollisions();
};

}  // namespace amr::sim
