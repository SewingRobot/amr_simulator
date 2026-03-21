# Agent Assignment: sim-agent — A-05 Advanced Simulation

## Agent Profile
- **ID:** sim-agent | **Team:** Team 3 | **Phase:** 3 | **Duration:** Week 17-22
- **Feature:** A-05 Advanced Simulation (MuJoCo + Sensors)

## Objective
Add MuJoCo physics backend plugin and advanced sensors (3D LiDAR, camera, IMU, encoder).

## Week-by-Week Tasks

### Week 17-18: MuJoCo Backend
- [ ] MuJoCoBackend class implementing PhysicsBackend interface
- [ ] Shared library dynamic loading (.so/.dylib)
- [ ] MuJoCo world setup (gravity, ground, contact parameters)
- [ ] Robot body creation from config (compound shapes)

### Week 19-20: Advanced Sensors
- [ ] 3D LiDAR (multi-beam ray casting, configurable vertical beams)
- [ ] IMU sensor (accelerometer + gyroscope, bias + noise models)
- [ ] Wheel encoder (tick counting, occasional missed ticks)
- [ ] Sensor data included in TelemetryMessage (extended fields)

### Week 21-22: Camera & Mecanum
- [ ] Vulkan headless rendering setup (offscreen framebuffer)
- [ ] RGB camera sensor (rasterization, configurable resolution/FOV)
- [ ] Depth camera sensor (float32 depth buffer)
- [ ] Mecanum drive kinematics (4-wheel omnidirectional)
- [ ] Backend selection via config (`physics.backend: "mujoco"`)

## References
- `docs/features/advanced/A-05-advanced-simulation.md`
- `docs/teams/team3-sim-engine.md`

## Dependencies
- C-01 Sim Engine Core (PhysicsBackend interface)

## Definition of Done
- [ ] MuJoCo backend loads and runs via plugin
- [ ] 3D LiDAR, IMU, encoder sensors working
- [ ] Camera rendering (RGB + depth) via Vulkan
- [ ] Mecanum drive kinematics
- [ ] Backend switchable via config
