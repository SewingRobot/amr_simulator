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

    /// Set world boundaries for wall collision clamping
    void setWorldBounds(double width, double height) {
        m_worldWidth = width;
        m_worldHeight = height;
        m_hasBounds = true;
    }

private:
    struct Body {
        Pose2D pose{0.0, 0.0, 0.0};
        Velocity2D velocity{0.0, 0.0};
        double radius = 0.0;
    };

    std::unordered_map<uint64_t, Body> bodies_;
    std::vector<CollisionInfo> collisions_;
    uint64_t next_id_ = 1;
    double m_worldWidth = 0.0;
    double m_worldHeight = 0.0;
    bool m_hasBounds = false;

    void detectCollisions();
    void clampToBounds();
};

}  // namespace amr::sim
