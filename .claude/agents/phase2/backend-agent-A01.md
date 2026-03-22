# Agent Assignment: backend-agent — A-01 Mission System

## Agent Profile
- **ID:** backend-agent
- **Team:** Team 2 (Backend)
- **Phase:** 2 | **Duration:** Week 9-16
- **Feature:** A-01 Mission System

## Objective
Build mission CRUD, state machine, VDA5050 order generation, and pathfinding integration.

## Week-by-Week Tasks

### Week 9-10: Mission Service Foundation + Phase 1 Carryover
- [x] DB schema: missions table (migration 004) — mission_steps not yet added
- [x] Mission CRUD: GET/POST/PUT/DELETE /api/missions
- [x] State machine: CREATED → ASSIGNED → EXECUTING → COMPLETED/FAILED/CANCELLED
- [x] Transition validation (only valid state changes allowed)
- [x] **Carryover:** Wire roadmap DB + A* pathfinding in backend (migrations 005, 006)
- [ ] **Carryover C-04/C-05:** Wire Asset Manager and Map Manager gRPC proxies with MinIO (complete actual file storage flow) — not started

### Week 11-12: VDA5050 & Pathfinding
- [ ] VDA5050 Order message generation from mission steps
- [ ] A* path request to Map Manager (FindPath gRPC)
- [ ] Mission assignment: POST /api/missions/:id/assign
- [ ] Mission cancel: POST /api/missions/:id/cancel

### Week 13-14: Real-time & WebSocket
- [ ] Mission status change → WebSocket notification (topic: "missions")
- [ ] Mission assignment logic (find available robot, check path feasibility)
- [ ] Mission monitoring endpoint with step-by-step progress

### Week 15-16: Integration & Edge Cases
- [ ] Cancel mid-execution (send cancel order to sim/robot)
- [ ] Retry/timeout handling
- [ ] Integration test: create → assign → sim executes → status updates → complete
- [ ] Load test: 100 concurrent missions

## References
- `docs/features/advanced/A-01-mission-system.md`
- `docs/integration/integration-spec.md` §mission-types

## Dependencies
- C-03 Backend Core, C-04 Map Core (pathfinding)

## Definition of Done
- [ ] Mission CRUD + state machine + VDA5050 order generation
- [ ] Path planning integration with Map Manager
- [ ] WebSocket status notifications
- [ ] Integration tests pass
