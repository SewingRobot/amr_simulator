#include "environment/world.h"

#include <nlohmann/json.hpp>
#include <spdlog/spdlog.h>
#include <fstream>
#include <stdexcept>

namespace amr::sim {

void World::loadFromFile(const std::string& path) {
    std::ifstream file(path);
    if (!file.is_open()) {
        throw std::runtime_error("Failed to open world file: " + path);
    }

    nlohmann::json j;
    file >> j;

    auto& world = j.at("world");

    name_ = world.at("name").get<std::string>();

    auto size = world.at("size");
    width_ = size[0].get<double>();
    height_ = size[1].get<double>();

    walls_.clear();
    if (world.contains("walls")) {
        for (auto& w : world["walls"]) {
            Wall wall;
            wall.x1 = w["from"][0].get<double>();
            wall.y1 = w["from"][1].get<double>();
            wall.x2 = w["to"][0].get<double>();
            wall.y2 = w["to"][1].get<double>();
            walls_.push_back(wall);
        }
    }

    obstacles_.clear();
    if (world.contains("obstacles")) {
        for (auto& o : world["obstacles"]) {
            Obstacle obs;
            obs.type = o.at("type").get<std::string>();
            obs.x = o["position"][0].get<double>();
            obs.y = o["position"][1].get<double>();
            obs.size_x = o["size"][0].get<double>();
            obs.size_y = o["size"][1].get<double>();
            obstacles_.push_back(obs);
        }
    }

    spdlog::info("Loaded world '{}' ({}x{}) with {} walls, {} obstacles",
                 name_, width_, height_, walls_.size(), obstacles_.size());
}

}  // namespace amr::sim
