#pragma once
#include <string>

namespace amr::sim {

struct SimConfig {
    std::string physics_backend = "custom";  // "custom" | "mujoco" | "isaac_sim"
    double realtime_factor = 1.0;
    double timestep_s = 0.01;  // 10ms = 100Hz
    int grpc_port = 50051;
    std::string world_file = "";
};

}  // namespace amr::sim
