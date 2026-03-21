# Agent Assignment: frontend-agent — A-01 Mission UI + A-03 Remote Control

## Agent Profile
- **ID:** frontend-agent
- **Team:** Team 1 | **Phase:** 2 | **Duration:** Week 9-16
- **Features:** A-01 (Mission UI), A-03 (Remote Control)

## Objective
Build mission management UI (create, list, monitor) and remote teleoperation with joystick/camera.

## Week-by-Week Tasks

### Week 9-10: Mission List & Detail
- [ ] MissionList component: filterable/sortable table (status, robot, time)
- [ ] MissionDetail: step-by-step progress, timeline visualization
- [ ] Real-time status updates via WebSocket (topic: "missions")
- [ ] Mission status badges (color-coded)

### Week 11-12: Mission Creation
- [ ] MissionCreator wizard: select robot → pick start/end waypoints on map → set priority
- [ ] Waypoint picker: click on 3D roadmap nodes
- [ ] Path preview overlay on 3D viewer

### Week 13-14: Remote Control
- [ ] Virtual joystick (nipplejs library) + WASD keyboard support
- [ ] Velocity command via WebSocket → Backend relay
- [ ] Emergency stop button (prominent, red)
- [ ] Connection status indicator
- [ ] Telemetry panel (speed, battery, position)

### Week 15-16: WebRTC Camera & Integration
- [ ] WebRTC signaling integration
- [ ] Camera feed display (low-latency video)
- [ ] Integration with Backend mission/teleop endpoints
- [ ] E2E test: create mission → monitor in UI

## References
- `docs/features/advanced/A-01-mission-system.md`, `docs/features/advanced/A-03-remote-control.md`

## Dependencies
- C-02 3D Viewer, C-03 Backend (mission API, WS)

## Definition of Done
- [ ] Mission CRUD UI working
- [ ] Real-time mission status in UI
- [ ] Joystick/keyboard teleop functional
- [ ] WebRTC camera feed displayed
- [ ] Emergency stop works
