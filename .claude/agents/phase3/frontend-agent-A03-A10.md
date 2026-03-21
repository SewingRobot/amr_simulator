# Agent Assignment: frontend-agent — A-03 WebRTC Camera + A-10 Hybrid Mode Prep

## Agent Profile
- **ID:** frontend-agent | **Team:** Team 1 | **Phase:** 3 | **Duration:** Week 17-22
- **Features:** A-03 (WebRTC camera), A-10 (hybrid mode prep)

## Objective
Integrate WebRTC camera feed and build hybrid view mode (real + sim overlay).

## Week-by-Week Tasks

### Week 17-18: WebRTC Integration
- [ ] WebRTC peer connection setup (STUN/TURN config)
- [ ] Signaling flow via Backend WebSocket
- [ ] Camera feed display component (low-latency video element)
- [ ] Camera feed overlay in remote control panel

### Week 19-20: Hybrid View Mode
- [ ] View mode toggle: real-only / sim-only / hybrid
- [ ] Dual data source management (real telemetry + sim telemetry)
- [ ] Visual differentiation (real robots = solid, sim robots = transparent/outlined)
- [ ] Zustand store: `useViewerStore` mode state

### Week 21-22: Sensor Visualization
- [ ] LiDAR scan overlay (fan shape visualization in 3D)
- [ ] Camera frustum visualization
- [ ] Sensor data panel (when robot selected)
- [ ] Integration with Backend real robot telemetry

## References
- `docs/features/advanced/A-03-remote-control.md` §camera
- `docs/features/advanced/A-10-digital-twin.md`

## Dependencies
- C-02 3D Viewer, A-04 VDA5050 (real robot telemetry)

## Definition of Done
- [ ] WebRTC camera feed displayed in browser
- [ ] Hybrid view mode toggle working
- [ ] Real + sim robots visually differentiated
- [ ] Sensor visualization overlays
