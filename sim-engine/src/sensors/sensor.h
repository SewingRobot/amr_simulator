#pragma once

#include <string>
#include <vector>

namespace amr::sim {

/// Abstract base class for all sensors.
class Sensor {
public:
    virtual ~Sensor() = default;

    /// Update the sensor reading for the given timestep.
    virtual void update(double dt) = 0;

    /// Get the latest sensor data as a flat vector of doubles.
    virtual std::vector<double> getData() const = 0;

    /// Sensor type name.
    virtual std::string getType() const = 0;
};

}  // namespace amr::sim
