#include "core/simulation.h"
#include "grpc/sim_server.h"

#include <spdlog/spdlog.h>

#include <csignal>
#include <cstdlib>
#include <string>

static std::function<void()> g_shutdown;

static void signalHandler(int /*sig*/) {
    if (g_shutdown) g_shutdown();
}

int main(int argc, char* argv[]) {
    spdlog::set_level(spdlog::level::info);
    spdlog::info("AMR Simulation Engine v0.1.0");

    // ── Parse config from CLI args or defaults ──────────────────────
    amr::sim::SimConfig config;
    config.timestep_s = 0.01;       // 100 Hz physics
    config.realtime_factor = 1.0;

    int telemetry_port = 50051;
    int command_port   = 50052;
    std::string world_file;

    for (int i = 1; i < argc; ++i) {
        std::string arg = argv[i];
        if (arg == "--world" && i + 1 < argc) {
            world_file = argv[++i];
        } else if (arg == "--telemetry-port" && i + 1 < argc) {
            telemetry_port = std::atoi(argv[++i]);
        } else if (arg == "--command-port" && i + 1 < argc) {
            command_port = std::atoi(argv[++i]);
        } else if (arg == "--timestep" && i + 1 < argc) {
            config.timestep_s = std::atof(argv[++i]);
        } else if (arg == "--realtime-factor" && i + 1 < argc) {
            config.realtime_factor = std::atof(argv[++i]);
        } else if (arg == "--help") {
            spdlog::info("Usage: sim-engine [options]");
            spdlog::info("  --world <path>           World JSON file");
            spdlog::info("  --telemetry-port <port>  Telemetry TCP port (default 50051)");
            spdlog::info("  --command-port <port>    Command TCP port   (default 50052)");
            spdlog::info("  --timestep <seconds>     Physics timestep   (default 0.01)");
            spdlog::info("  --realtime-factor <f>    Realtime factor    (default 1.0)");
            return 0;
        }
    }

    // ── Create simulation ───────────────────────────────────────────
    amr::sim::Simulation sim(config);
    spdlog::info("Simulation initialized (dt={}s, rtf={})", config.timestep_s, config.realtime_factor);

    // ── Load world if specified ─────────────────────────────────────
    if (!world_file.empty()) {
        sim.loadWorld(world_file);
    }

    // ── Start network servers ───────────────────────────────────────
    amr::sim::SimServer server(telemetry_port, command_port, sim);
    server.start();

    // ── Wire telemetry broadcast into the sim loop ──────────────────
    sim.setPostStepCallback([&server]() {
        server.broadcastCurrentTelemetry();
    });

    // ── Handle SIGINT/SIGTERM for clean shutdown ────────────────────
    g_shutdown = [&]() {
        spdlog::info("Shutdown signal received");
        sim.stop();
    };
    std::signal(SIGINT,  signalHandler);
    std::signal(SIGTERM, signalHandler);

    // ── Run the simulation (blocking) ───────────────────────────────
    spdlog::info("Starting simulation loop...");
    sim.start();

    // ── Cleanup ─────────────────────────────────────────────────────
    server.stop();
    spdlog::info("AMR Simulation Engine shut down");

    return 0;
}
