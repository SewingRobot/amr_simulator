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
- [x] Vite + React 18 + TypeScript strict scaffold (React 18, not 19)
- [x] Three.js + React Three Fiber + Drei setup
- [x] Basic scene: ambient light, directional light, GridHelper, AxesHelper
- [ ] Storybook configuration, Tailwind CSS, Radix UI — Storybook NOT DONE
- [x] ESLint + Prettier config (partially — Vite default)
- **Deliverable:** Running app with basic 3D scene

### Week 3-4: Robot Rendering & Camera
- [x] GLTFLoader for robot models (fallback: colored box 0.5×0.3×0.2m)
- [x] Camera controls: OrbitControls (default), top-down, follow mode
- [x] Keyboard shortcuts (1=orbit, 2=top-down, 3=follow)
- [x] Robot selection via Raycaster click → info panel (Drei Html)
- [x] Robot label overlay (name, status)
- **Deliverable:** Load glTF model, orbit camera, click-to-select

### Week 5-6: WebSocket & Real-time
- [x] WebSocket client (connect, subscribe to telemetry topics)
- [x] Zustand store: `useRobotStore` (robots Map, selectedRobotId)
- [x] Pose interpolation (10Hz data → smooth 60fps rendering via lerp/slerp)
- [x] Environment rendering (walls=gray BoxGeometry, obstacles=orange)
- [ ] Basic point cloud display — NOT DONE (no real point cloud data)
- **Deliverable:** Robots update in real-time from WebSocket

### Week 7-8: Integration & Polish
- [x] Connect to real Backend WebSocket (not mock)
- [x] Load glTF from Asset Manager via Backend proxy (useModelLoader hook)
- [ ] Trajectory trail (last N positions as dotted line) — NOT DONE
- [ ] Storybook stories for all viewer components — NOT DONE
- [x] E2E test: Sim→Backend→Frontend pose rendering
- **Deliverable:** Full E2E 3D visualization working

### Additional completed items (not in original plan)
- [x] 2D/3D view toggle (V key)
- [x] ConnectionStatus indicator
- [x] 30 Vitest unit tests

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
- [x] 3D scene with environment (walls, obstacles, grid)
- [x] glTF robot model or fallback box rendering
- [x] Real-time pose update via WebSocket (10Hz)
- [x] Camera controls (orbit/top-down/follow)
- [x] Robot click selection + info panel
- [ ] Storybook components registered — NOT DONE
