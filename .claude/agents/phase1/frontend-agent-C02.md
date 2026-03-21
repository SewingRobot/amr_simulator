# Agent Assignment: frontend-agent — C-02 3D Viewer Core

## Agent Profile
- **ID:** frontend-agent
- **Team:** Team 1 (Frontend)
- **Phase:** 1 (Prototype)
- **Feature:** C-02 3D Viewer Core
- **Duration:** Week 1-8

## Objective
Build browser-based 3D viewer with real-time robot visualization using Three.js/R3F and WebSocket telemetry.

## Week-by-Week Tasks

### Week 1-2: Project Setup
- [ ] Vite + React 19 + TypeScript strict scaffold
- [ ] Three.js + React Three Fiber + Drei setup
- [ ] Basic scene: ambient light, directional light, GridHelper, AxesHelper
- [ ] Storybook configuration, Tailwind CSS, Radix UI
- [ ] ESLint + Prettier config
- **Deliverable:** Running app with basic 3D scene

### Week 3-4: Robot Rendering & Camera
- [ ] GLTFLoader for robot models (fallback: colored box 0.5×0.3×0.2m)
- [ ] Camera controls: OrbitControls (default), top-down, follow mode
- [ ] Keyboard shortcuts (1=orbit, 2=top-down, 3=follow)
- [ ] Robot selection via Raycaster click → info panel (Drei Html)
- [ ] Robot label overlay (name, status)
- **Deliverable:** Load glTF model, orbit camera, click-to-select

### Week 5-6: WebSocket & Real-time
- [ ] WebSocket client (connect, subscribe to telemetry topics)
- [ ] Zustand store: `useRobotStore` (robots Map, selectedRobotId)
- [ ] Pose interpolation (10Hz data → smooth 60fps rendering via lerp)
- [ ] Environment rendering (walls=gray BoxGeometry, obstacles=orange)
- [ ] Basic point cloud display (BufferGeometry, no LOD yet)
- **Deliverable:** Robots update in real-time from WebSocket

### Week 7-8: Integration & Polish
- [ ] Connect to real Backend WebSocket (not mock)
- [ ] Load glTF from Asset Manager via Backend proxy
- [ ] Trajectory trail (last N positions as dotted line)
- [ ] Storybook stories for all viewer components
- [ ] E2E test: Sim→Backend→Frontend pose rendering
- **Deliverable:** Full E2E 3D visualization working

## Reference Documents
- Feature spec: `docs/features/core/C-02-3d-viewer-core.md`
- Team tech: `docs/teams/team1-frontend.md`
- WS messages: `docs/integration/integration-spec.md` §websocket-messages
- Coordinates: `docs/integration/integration-spec.md` §coordinate-system

## Dependencies
- Week 2: Proto freeze (proto-agent)
- Week 5: Backend WebSocket server ready (C-03)
- Week 7: Asset Manager glTF download (C-05)

## Constraints
- Only modify `frontend/`. Proto changes require RFC.
- ENU→Three.js coordinate conversion (swap Y↔Z)

## Definition of Done
- [ ] 3D scene with environment (walls, obstacles, grid)
- [ ] glTF robot model or fallback box rendering
- [ ] Real-time pose update via WebSocket (10Hz)
- [ ] Camera controls (orbit/top-down/follow)
- [ ] Robot click selection + info panel
- [ ] Storybook components registered
