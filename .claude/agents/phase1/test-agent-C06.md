# Agent Assignment: test-agent — C-06 Telemetry Pipeline E2E

## Agent Profile
- **ID:** test-agent
- **Team:** Cross-team (Integration)
- **Phase:** 1 (Prototype)
- **Feature:** C-06 Telemetry Pipeline
- **Duration:** Week 1-8

## Objective
Generate test data, build mock servers, and validate the end-to-end telemetry flow (Sim→Backend→Frontend).

## Week-by-Week Tasks

### Week 1-2: Test Data & Mocks
- [ ] Test data generator scripts (`test-data/generators/`)
- [ ] Generate XS dataset (1 robot, 1K points, 3 nodes)
- [ ] Generate S dataset (3 robots, 10K points, 10 nodes)
- [ ] Sample robot config JSON, world JSON, telemetry sequence
- [ ] Mock gRPC servers for Sim Engine, Asset Manager, Map Manager
- **Deliverable:** Test fixtures ready, mock servers runnable

### Week 3-4: Mock Infrastructure
- [ ] MSW (Mock Service Worker) handlers for Frontend development
- [ ] Mock WebSocket server generating fake telemetry (10Hz)
- [ ] Mock Potree tiles (pre-generated small dataset)
- [ ] Mock glTF robot model (simple box)
- [ ] Seed-based generation (SEED=42 for reproducibility)
- **Deliverable:** All mocks operational for team dev

### Week 5-6: E2E Test Scripts
- [ ] Telemetry latency measurement tool (timestamp at each hop)
- [ ] Multi-robot stress test (10 robots × 10Hz × 60 seconds)
- [ ] WebSocket reconnection test (kill + auto-reconnect)
- [ ] Multiple client test (3 browsers receive same telemetry)
- [ ] Generate M dataset (10 robots, 1M points, 50 nodes)
- **Deliverable:** E2E test scripts ready

### Week 7-8: Integration Validation
- [ ] Docker Compose full-stack integration test
- [ ] Run all E2E scenarios against real services
- [ ] Measure: latency p50/p95/p99 (target: <100ms p95)
- [ ] Verify: no dropped messages over 60s
- [ ] Phase 1 checkpoint report (`docs/checkpoints/`)
- **Deliverable:** Phase 1 E2E validated, checkpoint report written

## Reference Documents
- Feature spec: `docs/features/core/C-06-telemetry-pipeline.md`
- Test data spec: `docs/integration/test-data-spec.md`
- Data pipeline: `docs/integration/data-pipeline.md`
- Docker Compose: `docs/integration/integration-spec.md` §docker-compose

## Dependencies
- Week 2: Proto freeze
- Week 5: All team services minimally functional
- Week 7: Full stack ready for integration

## Constraints
- Write to `test-data/` and `tests/` only
- Read-only access to all sub-project code
- Docker Compose changes via infra-agent

## Definition of Done
- [ ] Test data generators for all data types
- [ ] XS/S/M fixture datasets generated
- [ ] Mock servers for all 3 internal services
- [ ] E2E telemetry test: Sim→Backend→Frontend verified
- [ ] Latency <100ms p95 confirmed
- [ ] Phase 1 checkpoint report written
