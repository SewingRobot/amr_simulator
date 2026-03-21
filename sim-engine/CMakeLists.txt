cmake_minimum_required(VERSION 3.28)
project(amr-sim-engine VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(ENABLE_PROTO "Enable gRPC proto code generation" OFF)

# Dependencies via find_package (Conan provides them)
find_package(spdlog REQUIRED)
find_package(nlohmann_json REQUIRED)
find_package(GTest REQUIRED)

if(ENABLE_PROTO)
    find_package(gRPC REQUIRED)
    find_package(Protobuf REQUIRED)
endif()

# ─── Sim engine library ─────────────────────────────────────────────
file(GLOB_RECURSE SIM_SOURCES "src/*.cpp")
list(FILTER SIM_SOURCES EXCLUDE REGEX "src/main.cpp")

add_library(sim_engine_lib STATIC ${SIM_SOURCES})
target_include_directories(sim_engine_lib PUBLIC src/)
target_link_libraries(sim_engine_lib PUBLIC
    spdlog::spdlog
    nlohmann_json::nlohmann_json
)

if(ENABLE_PROTO)
    # Proto code generation would go here
    target_link_libraries(sim_engine_lib PUBLIC
        protobuf::protobuf
        gRPC::grpc++
    )
endif()

add_executable(sim-engine src/main.cpp)
target_link_libraries(sim-engine PRIVATE sim_engine_lib)

# Tests
enable_testing()
add_subdirectory(tests)
