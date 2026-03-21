#include "core/sim_loop.h"

#include <spdlog/spdlog.h>
#include <thread>

namespace amr::sim {

SimLoop::SimLoop(double timestep_s, double realtime_factor)
    : timestep_s_(timestep_s)
    , realtime_factor_(realtime_factor) {}

void SimLoop::setStepCallback(StepCallback cb) {
    step_callback_ = std::move(cb);
}

void SimLoop::run() {
    running_ = true;
    spdlog::info("SimLoop started: dt={}s, realtime_factor={}", timestep_s_, realtime_factor_);

    auto wall_step = std::chrono::duration<double>(timestep_s_ / realtime_factor_);

    while (running_) {
        auto t0 = std::chrono::steady_clock::now();

        if (step_callback_) {
            step_callback_(timestep_s_);
        }
        sim_time_ += timestep_s_;
        step_count_++;

        auto t1 = std::chrono::steady_clock::now();
        auto elapsed = t1 - t0;
        if (elapsed < wall_step) {
            std::this_thread::sleep_for(wall_step - elapsed);
        }
    }

    spdlog::info("SimLoop stopped after {} steps (sim_time={}s)", step_count_, sim_time_);
}

void SimLoop::stop() {
    running_ = false;
}

}  // namespace amr::sim
