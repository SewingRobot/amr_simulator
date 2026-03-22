# Agent Assignment: sim-agent — C-01 Sim Engine Core

## Agent Profile
- **ID:** sim-agent
- **Team:** Team 3 (Simulation Engine)
- **Phase:** 1 (Prototype)
- **Feature:** C-01 Sim Engine Core
- **Duration:** Week 1-8

## Objective
Build custom lightweight physics engine with differential drive robots, 2D LiDAR sensors, and gRPC telemetry streaming.

## Week-by-Week Tasks

### Week 1-2: Project Setup
- [x] CMake + Conan project scaffold
- [x] Directory structure: `src/{core,physics,robots,sensors,grpc,python}`
- [x] Build system with clang-format, clang-tidy, Google Test
- [x] Basic `main.cpp` with sim loop skeleton
- **Deliverable:** Project builds and runs empty sim loop

### Week 3-4: Physics & Robot
- [x] `CustomLightweightBackend` class (PhysicsBackend interface)
- [x] 2D collision detection (circle-circle + wall boundary clamping; no AABB)
- [x] `DifferentialDrive` kinematics (v_left/v_right → Δx, Δy, Δθ)
- [x] Acceleration limits, battery model (linear depletion)
- [x] TCP server (telemetry + command) — TCP JSON instead of gRPC
- **Deliverable:** Robot moves via TCP velocity commands

### Week 5-6: Sensors & Multi-Robot
- [x] 2D LiDAR sensor (360-ray sweep, batch ray casting)
- [x] Gaussian noise (σ=0.01m), configurable update rate (10Hz)
- [x] Multi-robot support (spawn/remove at runtime)
- [x] StreamTelemetry TCP JSON (10Hz per robot) — TCP instead of gRPC
- [x] JSON world loading (walls, obstacles)
- **Deliverable:** 10 robots with LiDAR, streaming telemetry

### Week 7-8: Integration & Polish
- [x] Verify telemetry format (with 2D→3D conversion in backend)
- [x] Performance optimization (0.013ms/step for 10 robots)
- [x] Python scenario script (demo_runner.py)
- [x] Unit tests: 29 GTest (kinematics, collision, LiDAR)
- [x] Integration test with Backend TCP client
- **Deliverable:** All C-01 DoD items checked

## Reference Documents
- Feature spec: `docs/features/core/C-01-sim-engine-core.md`
- Team tech: `docs/teams/team3-sim-engine.md`
- Interfaces: `docs/integration/integration-spec.md` §simulation-service
- Test data: `docs/integration/test-data-spec.md` §sim-engine-test-data

## Dependencies
- Week 2: Proto freeze (proto-agent)
- Week 7-8: Backend gRPC client ready (or mock)

## Constraints
- Only modify `sim-engine/`. Proto changes require RFC.
- Use JSON world format (OpenUSD deferred to A-09)

## Definition of Done
- [x] Custom physics: differential drive robot moves correctly
- [x] 2D LiDAR ray casting works
- [x] TCP telemetry streaming at 10Hz (TCP JSON instead of gRPC)
- [x] 10 robots simultaneous real-time
- [x] All unit tests pass (29 GTest)
- [x] Backend integration confirmed
