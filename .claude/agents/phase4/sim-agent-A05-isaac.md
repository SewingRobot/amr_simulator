# Agent Assignment: sim-agent — A-05 Isaac Sim Bridge + OpenUSD Loading

## Agent Profile
- **ID:** sim-agent | **Team:** Team 3 | **Phase:** 4 | **Duration:** Week 23-30
- **Feature:** A-05 (Isaac Sim), A-09 (OpenUSD scene loading in engine)

## Objective
Implement Isaac Sim bridge for photorealistic simulation and OpenUSD scene loading in the engine.

## Week-by-Week Tasks

### Week 23-24: Isaac Sim Bridge
- [ ] IsaacSimBridge class implementing PhysicsBackend interface
- [ ] gRPC connection to Isaac Sim instance (separate process)
- [ ] Command forwarding: velocity commands → Isaac Sim
- [ ] Telemetry reception: Isaac Sim state → TelemetryMessage

### Week 25-26: Photorealistic Sensors
- [ ] Ray-traced LiDAR via Isaac Sim
- [ ] Synthetic RGB camera via Isaac Sim RTX renderer
- [ ] Sensor data streaming through gRPC bridge
- [ ] Config: `physics.backend: "isaac_sim"` + endpoint setting

### Week 27-28: OpenUSD Scene Loading
- [ ] OpenUSD SDK (pxr) C++ integration
- [ ] USD scene loading: parse Xform hierarchy, mesh prims
- [ ] UsdPhysics → PhysicsBackend body creation
- [ ] Support for both custom engine and MuJoCo backends

### Week 29-30: Polish & Benchmarks
- [ ] Performance benchmarks (Isaac Sim vs custom vs MuJoCo)
- [ ] Documentation: backend selection guide
- [ ] Integration test: load USD scene → simulate → stream telemetry
- [ ] CI pipeline for sim engine with optional backends

## References
- `docs/features/advanced/A-05-advanced-simulation.md`
- `docs/features/advanced/A-09-openusd-pipeline.md`

## Dependencies
- C-01 Sim Engine Core, A-09 OpenUSD Pipeline (asset-agent provides USD files)

## Definition of Done
- [ ] Isaac Sim bridge functional
- [ ] Photorealistic sensor data via Isaac Sim
- [ ] OpenUSD scene loading in sim engine
- [ ] All 3 backends switchable via config
