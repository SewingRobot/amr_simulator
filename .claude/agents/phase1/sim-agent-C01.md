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
- [ ] CMake + Conan project scaffold
- [ ] Directory structure: `src/{core,physics,robots,sensors,grpc,python}`
- [ ] Build system with clang-format, clang-tidy, Google Test
- [ ] Basic `main.cpp` with sim loop skeleton
- **Deliverable:** Project builds and runs empty sim loop

### Week 3-4: Physics & Robot
- [ ] `CustomLightweightBackend` class (PhysicsBackend interface)
- [ ] 2D collision detection (AABB, circle-circle)
- [ ] `DifferentialDrive` kinematics (v_left/v_right → Δx, Δy, Δθ)
- [ ] Acceleration limits, battery model (linear depletion)
- [ ] gRPC server skeleton (CreateSim, StartSim, StopSim, SpawnRobot, SendCommand)
- **Deliverable:** Robot moves via gRPC velocity commands

### Week 5-6: Sensors & Multi-Robot
- [ ] 2D LiDAR sensor (360-ray sweep, batch ray casting)
- [ ] Gaussian noise (σ=0.01m), configurable update rate (10Hz)
- [ ] Multi-robot support (spawn/remove at runtime)
- [ ] StreamTelemetry gRPC (10Hz per robot)
- [ ] JSON world loading (walls, obstacles)
- **Deliverable:** 10 robots with LiDAR, streaming telemetry

### Week 7-8: Integration & Polish
- [ ] Verify telemetry matches proto TelemetryMessage format
- [ ] Performance optimization (10 robots real-time, step <10ms)
- [ ] Python scenario script (spawn, command, collect)
- [ ] Unit tests: kinematics, collision, LiDAR, gRPC E2E
- [ ] Integration test with Backend gRPC client
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
- [ ] Custom physics: differential drive robot moves correctly
- [ ] 2D LiDAR ray casting works
- [ ] gRPC telemetry streaming at 10Hz
- [ ] 10 robots simultaneous real-time
- [ ] All unit tests pass
- [ ] Backend integration confirmed
