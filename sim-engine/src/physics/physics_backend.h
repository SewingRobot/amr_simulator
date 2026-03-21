#pragma once
#include <cstdint>
#include <string>
#include <vector>

namespace amr::sim {

struct BodyHandle {
    uint64_t id;
    bool operator==(const BodyHandle& other) const { return id == other.id; }
};

struct Pose2D {
    double x, y, theta;
};

struct Velocity2D {
    double linear, angular;
};

struct CollisionInfo {
    BodyHandle a, b;
};

class PhysicsBackend {
public:
    virtual ~PhysicsBackend() = default;
    virtual void init() = 0;
    virtual void shutdown() = 0;
    virtual void step(double dt) = 0;
    virtual BodyHandle addBody(double x, double y, double radius) = 0;
    virtual void removeBody(BodyHandle handle) = 0;
    virtual void setBodyVelocity(BodyHandle handle, Velocity2D vel) = 0;
    virtual Pose2D getBodyPose(BodyHandle handle) const = 0;
    virtual Velocity2D getBodyVelocity(BodyHandle handle) const = 0;
    virtual std::vector<CollisionInfo> getCollisions() const = 0;
    virtual std::string getName() const = 0;
};

}  // namespace amr::sim
