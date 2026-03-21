#pragma once

#include "physics/physics_backend.h"
#include "physics/custom_backend.h"

#include <memory>
#include <stdexcept>
#include <string>

namespace amr::sim {

class BackendFactory {
public:
    static std::unique_ptr<PhysicsBackend> create(const std::string& name) {
        if (name == "custom") {
            return std::make_unique<CustomLightweightBackend>();
        }
        // Future: "mujoco", "isaac_sim"
        throw std::runtime_error("Unknown physics backend: " + name);
    }
};

}  // namespace amr::sim
