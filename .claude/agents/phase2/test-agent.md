# Agent Assignment: test-agent — Phase 2 Integration Tests

## Agent Profile
- **ID:** test-agent
- **Team:** Integration | **Phase:** 2 | **Duration:** Week 9-16
- **Feature:** Cross-team (A-01, A-02 integration validation)

## Objective
Validate mission E2E flow, traffic management scenarios, and remote control integration.

## Week-by-Week Tasks

### Week 9-12: Mission E2E Tests
- [ ] Scenario: create mission → assign robot → sim executes → status updates → complete
- [ ] Scenario: cancel mission mid-execution
- [ ] Scenario: mission with pathfinding (start→waypoints→end)
- [ ] Generate M dataset with mission data (20 missions, 10 robots, 50-node graph)

### Week 13-14: Traffic Scenarios
- [ ] 5 robots with overlapping paths — verify no collisions
- [ ] Priority override — high-priority mission preempts zone
- [ ] Deadlock scenario — verify timeout resolution

### Week 15-16: Phase 2 Checkpoint
- [ ] Full E2E: map edit → create roadmap → create mission → sim execute → monitor
- [ ] Remote control E2E: select robot → joystick drive → verify telemetry
- [ ] Performance baselines (mission throughput, zone lock latency)
- [ ] Phase 2 checkpoint report

## References
- `docs/integration/test-data-spec.md`, `docs/integration/integration-spec.md` §docker-compose

## Definition of Done
- [ ] All mission E2E scenarios pass
- [ ] Traffic scenarios pass (no deadlock, no collision)
- [ ] Phase 2 checkpoint report written
