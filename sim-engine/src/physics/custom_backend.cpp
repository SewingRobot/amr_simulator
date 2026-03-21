#include "physics/custom_backend.h"

#include <spdlog/spdlog.h>
#include <cmath>
#include <stdexcept>

namespace amr::sim {

void CustomLightweightBackend::init() {
    spdlog::info("CustomLightweightBackend initialized");
}

void CustomLightweightBackend::shutdown() {
    bodies_.clear();
    collisions_.clear();
    spdlog::info("CustomLightweightBackend shut down");
}

void CustomLightweightBackend::step(double dt) {
    // Integrate velocities: update pose based on (linear, angular) velocity
    for (auto& [id, body] : bodies_) {
        double v = body.velocity.linear;
        double w = body.velocity.angular;

        // Update orientation
        body.pose.theta += w * dt;

        // Normalize theta to [-pi, pi]
        while (body.pose.theta > M_PI) body.pose.theta -= 2.0 * M_PI;
        while (body.pose.theta < -M_PI) body.pose.theta += 2.0 * M_PI;

        // Update position along current heading
        body.pose.x += v * std::cos(body.pose.theta) * dt;
        body.pose.y += v * std::sin(body.pose.theta) * dt;
    }

    clampToBounds();
    detectCollisions();
}

BodyHandle CustomLightweightBackend::addBody(double x, double y, double radius) {
    uint64_t id = next_id_++;
    Body body;
    body.pose = {x, y, 0.0};
    body.radius = radius;
    bodies_[id] = body;
    return {id};
}

void CustomLightweightBackend::removeBody(BodyHandle handle) {
    auto it = bodies_.find(handle.id);
    if (it == bodies_.end()) {
        spdlog::warn("removeBody: body {} not found", handle.id);
        return;
    }
    bodies_.erase(it);
}

void CustomLightweightBackend::setBodyVelocity(BodyHandle handle, Velocity2D vel) {
    auto it = bodies_.find(handle.id);
    if (it == bodies_.end()) {
        spdlog::warn("setBodyVelocity: body {} not found", handle.id);
        return;
    }
    it->second.velocity = vel;
}

Pose2D CustomLightweightBackend::getBodyPose(BodyHandle handle) const {
    auto it = bodies_.find(handle.id);
    if (it == bodies_.end()) {
        throw std::runtime_error("getBodyPose: body not found");
    }
    return it->second.pose;
}

Velocity2D CustomLightweightBackend::getBodyVelocity(BodyHandle handle) const {
    auto it = bodies_.find(handle.id);
    if (it == bodies_.end()) {
        throw std::runtime_error("getBodyVelocity: body not found");
    }
    return it->second.velocity;
}

std::vector<CollisionInfo> CustomLightweightBackend::getCollisions() const {
    return collisions_;
}

void CustomLightweightBackend::detectCollisions() {
    collisions_.clear();

    // Collect body IDs for pairwise iteration
    std::vector<uint64_t> ids;
    ids.reserve(bodies_.size());
    for (const auto& [id, _] : bodies_) {
        ids.push_back(id);
    }

    // Circle-circle collision detection
    for (size_t i = 0; i < ids.size(); ++i) {
        for (size_t j = i + 1; j < ids.size(); ++j) {
            const Body& a = bodies_.at(ids[i]);
            const Body& b = bodies_.at(ids[j]);

            double dx = a.pose.x - b.pose.x;
            double dy = a.pose.y - b.pose.y;
            double dist_sq = dx * dx + dy * dy;
            double min_dist = a.radius + b.radius;

            if (dist_sq < min_dist * min_dist) {
                collisions_.push_back({{ids[i]}, {ids[j]}});
            }
        }
    }
}

void CustomLightweightBackend::clampToBounds() {
    if (!m_hasBounds) return;

    for (auto& [id, body] : bodies_) {
        double r = body.radius + 0.05;  // small buffer so body visually stays inside wall
        bool hit = false;

        if (body.pose.x < r) {
            body.pose.x = r;
            hit = true;
        }
        if (body.pose.x > m_worldWidth - r) {
            body.pose.x = m_worldWidth - r;
            hit = true;
        }
        if (body.pose.y < r) {
            body.pose.y = r;
            hit = true;
        }
        if (body.pose.y > m_worldHeight - r) {
            body.pose.y = m_worldHeight - r;
            hit = true;
        }

        // When hitting a wall, reverse the heading slightly to nudge away
        if (hit) {
            body.pose.theta += M_PI * 0.5;  // turn 90 degrees on wall contact
            while (body.pose.theta > M_PI) body.pose.theta -= 2.0 * M_PI;
        }
    }
}

}  // namespace amr::sim
