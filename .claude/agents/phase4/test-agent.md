# Agent Assignment: test-agent — Phase 4 Final Integration

## Agent Profile
- **ID:** test-agent | **Team:** Integration | **Phase:** 4 | **Duration:** Week 23-30
- **Feature:** Final E2E validation + production readiness

## Objective
Run comprehensive E2E tests, load tests with large datasets, and validate production readiness.

## Week-by-Week Tasks

### Week 23-24: Phase 3 Regression
- [ ] Re-run all Phase 1-3 E2E test suites
- [ ] Plugin E2E: WASM plugin receives events, creates missions
- [ ] ROS 2 Bridge: DDS↔MQTT message translation test
- [ ] Generate L dataset (50 robots, 10M points, 200 nodes, 100 missions)

### Week 25-26: Full System E2E
- [ ] Scenario: 50 robots, 100 missions, full traffic management
- [ ] Scenario: real robot (VDA5050 sim) + sim robots hybrid
- [ ] Scenario: map update while missions running
- [ ] Dashboard data accuracy verification

### Week 27-28: Load & Stress Testing
- [ ] Generate XL dataset (200 robots, 100M points, 1000 nodes, 1000 missions)
- [ ] WebSocket: 1000 concurrent connections
- [ ] Telemetry: 200 robots × 10Hz throughput
- [ ] Point cloud: 100M point Potree rendering performance
- [ ] Database: query latency under load (TimescaleDB, PostGIS)

### Week 29-30: Production Readiness
- [ ] Security test: auth bypass attempts, injection attacks
- [ ] Failover test: kill services, verify recovery
- [ ] Data integrity: verify no data loss during high load
- [ ] Final checkpoint report
- [ ] Production readiness sign-off checklist

## References
- `docs/integration/test-data-spec.md`
- `docs/integration/data-pipeline.md`

## Dependencies
- All features complete (Phase 1-4)

## Definition of Done
- [ ] All regression tests pass
- [ ] Full system E2E with 50+ robots passes
- [ ] Load test: 200 robots sustained
- [ ] Security tests pass
- [ ] Production readiness report written
- [ ] Final checkpoint approved
