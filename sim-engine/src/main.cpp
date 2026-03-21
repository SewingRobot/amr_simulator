#include "core/simulation.h"
#include <spdlog/spdlog.h>

int main(int argc, char* argv[]) {
    spdlog::set_level(spdlog::level::info);
    spdlog::info("AMR Simulation Engine v0.1.0");

    amr::sim::SimConfig config;
    // TODO: Load config from file or CLI args

    amr::sim::Simulation sim(config);
    spdlog::info("Simulation initialized");

    // TODO: Start gRPC server
    // TODO: Run simulation loop

    return 0;
}
