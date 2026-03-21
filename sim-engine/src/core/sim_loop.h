#pragma once

#include <atomic>
#include <chrono>
#include <functional>

namespace amr::sim {

/// SimLoop encapsulates a fixed-timestep simulation loop with real-time pacing.
class SimLoop {
public:
    using StepCallback = std::function<void(double dt)>;

    SimLoop(double timestep_s, double realtime_factor);

    /// Set the function called each tick.
    void setStepCallback(StepCallback cb);

    /// Run the loop (blocking). Call stop() from another thread to exit.
    void run();

    /// Signal the loop to stop.
    void stop();

    /// Get the current simulation time in seconds.
    double simTime() const { return sim_time_; }

    /// Get the number of steps executed.
    uint64_t stepCount() const { return step_count_; }

private:
    double timestep_s_;
    double realtime_factor_;
    std::atomic<bool> running_{false};
    double sim_time_ = 0.0;
    uint64_t step_count_ = 0;
    StepCallback step_callback_;
};

}  // namespace amr::sim
