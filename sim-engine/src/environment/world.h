#pragma once

#include <string>
#include <vector>

namespace amr::sim {

struct Wall {
    double x1, y1;  // from
    double x2, y2;  // to
};

struct Obstacle {
    std::string type;          // "box" or "circle"
    double x, y;               // center position
    double size_x, size_y;     // dimensions (for box)
};

/// World environment loaded from a JSON scenario file.
class World {
public:
    World() = default;

    /// Load world definition from a JSON file.
    void loadFromFile(const std::string& path);

    /// Get all walls.
    const std::vector<Wall>& getWalls() const { return walls_; }

    /// Get all obstacles.
    const std::vector<Obstacle>& getObstacles() const { return obstacles_; }

    /// World name.
    const std::string& getName() const { return name_; }

    /// World dimensions.
    double getWidth() const { return width_; }
    double getHeight() const { return height_; }

private:
    std::string name_;
    double width_ = 0.0;
    double height_ = 0.0;
    std::vector<Wall> walls_;
    std::vector<Obstacle> obstacles_;
};

}  // namespace amr::sim
