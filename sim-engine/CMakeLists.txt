cmake_minimum_required(VERSION 3.28)
project(amr-sim-engine VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Dependencies via find_package (Conan provides them)
find_package(gRPC REQUIRED)
find_package(Protobuf REQUIRED)
find_package(spdlog REQUIRED)
find_package(nlohmann_json REQUIRED)
find_package(GTest REQUIRED)

# ─── Proto code generation ───────────────────────────────────────────
set(PROTO_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../proto")
set(PROTO_GENERATED_DIR "${CMAKE_BINARY_DIR}/proto_gen")
file(MAKE_DIRECTORY ${PROTO_GENERATED_DIR})

# Locate protoc and grpc_cpp_plugin
find_program(PROTOC protoc REQUIRED)
find_program(GRPC_CPP_PLUGIN grpc_cpp_plugin REQUIRED)

set(PROTO_FILES
    "${PROTO_DIR}/common/geometry.proto"
    "${PROTO_DIR}/common/identifiers.proto"
    "${PROTO_DIR}/common/errors.proto"
    "${PROTO_DIR}/common/telemetry.proto"
    "${PROTO_DIR}/simulation/sim_types.proto"
    "${PROTO_DIR}/simulation/sim_service.proto"
)

set(PROTO_GENERATED_SRCS "")
set(PROTO_GENERATED_HDRS "")
set(GRPC_GENERATED_SRCS "")
set(GRPC_GENERATED_HDRS "")

foreach(PROTO_FILE ${PROTO_FILES})
    # Compute relative path from PROTO_DIR for output naming
    file(RELATIVE_PATH PROTO_REL ${PROTO_DIR} ${PROTO_FILE})
    get_filename_component(PROTO_REL_DIR ${PROTO_REL} DIRECTORY)
    get_filename_component(PROTO_NAME_WE ${PROTO_FILE} NAME_WE)

    set(PB_CC "${PROTO_GENERATED_DIR}/${PROTO_REL_DIR}/${PROTO_NAME_WE}.pb.cc")
    set(PB_H  "${PROTO_GENERATED_DIR}/${PROTO_REL_DIR}/${PROTO_NAME_WE}.pb.h")

    list(APPEND PROTO_GENERATED_SRCS ${PB_CC})
    list(APPEND PROTO_GENERATED_HDRS ${PB_H})

    # Generate protobuf sources
    add_custom_command(
        OUTPUT ${PB_CC} ${PB_H}
        COMMAND ${PROTOC}
            --proto_path=${PROTO_DIR}
            --cpp_out=${PROTO_GENERATED_DIR}
            ${PROTO_FILE}
        DEPENDS ${PROTO_FILE}
        COMMENT "Generating protobuf C++ for ${PROTO_REL}"
    )

    # Generate gRPC sources only for service protos
    if(PROTO_REL MATCHES "sim_service\\.proto$")
        set(GRPC_CC "${PROTO_GENERATED_DIR}/${PROTO_REL_DIR}/${PROTO_NAME_WE}.grpc.pb.cc")
        set(GRPC_H  "${PROTO_GENERATED_DIR}/${PROTO_REL_DIR}/${PROTO_NAME_WE}.grpc.pb.h")

        list(APPEND GRPC_GENERATED_SRCS ${GRPC_CC})
        list(APPEND GRPC_GENERATED_HDRS ${GRPC_H})

        add_custom_command(
            OUTPUT ${GRPC_CC} ${GRPC_H}
            COMMAND ${PROTOC}
                --proto_path=${PROTO_DIR}
                --grpc_out=${PROTO_GENERATED_DIR}
                --plugin=protoc-gen-grpc=${GRPC_CPP_PLUGIN}
                ${PROTO_FILE}
            DEPENDS ${PROTO_FILE}
            COMMENT "Generating gRPC C++ for ${PROTO_REL}"
        )
    endif()
endforeach()

# Proto library target
add_library(proto_gen STATIC
    ${PROTO_GENERATED_SRCS}
    ${GRPC_GENERATED_SRCS}
)
target_include_directories(proto_gen PUBLIC ${PROTO_GENERATED_DIR})
target_link_libraries(proto_gen PUBLIC
    protobuf::protobuf
    gRPC::grpc++
)
# Suppress warnings in generated code
target_compile_options(proto_gen PRIVATE -w)

# ─── Sim engine library ─────────────────────────────────────────────
# Source files
file(GLOB_RECURSE SIM_SOURCES "src/*.cpp")
list(FILTER SIM_SOURCES EXCLUDE REGEX "src/main.cpp")

add_library(sim_engine_lib STATIC ${SIM_SOURCES})
target_include_directories(sim_engine_lib PUBLIC src/)
target_link_libraries(sim_engine_lib PUBLIC
    spdlog::spdlog
    nlohmann_json::nlohmann_json
    proto_gen
)

add_executable(sim-engine src/main.cpp)
target_link_libraries(sim-engine PRIVATE sim_engine_lib)

# Tests
enable_testing()
add_subdirectory(tests)
