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
- [ ] Test data generator scripts (`test-data/generators/`) — NOT DONE
- [ ] Generate XS dataset — NOT DONE
- [ ] Generate S dataset — NOT DONE
- [x] Sample world JSON (basic_warehouse.json)
- [ ] Mock gRPC servers for Sim Engine, Asset Manager, Map Manager — NOT DONE
- **Deliverable:** Test fixtures ready, mock servers runnable

### Week 3-4: Mock Infrastructure
- [ ] MSW (Mock Service Worker) handlers for Frontend development — NOT DONE
- [ ] Mock WebSocket server generating fake telemetry (10Hz) — NOT DONE
- [ ] Mock Potree tiles (pre-generated small dataset) — NOT DONE
- [ ] Mock glTF robot model (simple box) — NOT DONE
- [ ] Seed-based generation (SEED=42 for reproducibility) — NOT DONE
- **Deliverable:** All mocks operational for team dev

### Week 5-6: E2E Test Scripts
- [ ] Telemetry latency measurement tool — NOT DONE
- [ ] Multi-robot stress test — IMPLICIT (demo_runner.py)
- [ ] WebSocket reconnection test — NOT DONE (logic exists but no formal test)
- [ ] Multiple client test — NOT DONE
- [ ] Generate M dataset — NOT DONE
- **Deliverable:** E2E test scripts ready

### Week 7-8: Integration Validation
- [x] Docker Compose full-stack integration test (manual)
- [x] Run all E2E scenarios against real services (manual)
- [ ] Measure: latency p50/p95/p99 — NOT MEASURED
- [ ] Verify: no dropped messages over 60s — NOT VERIFIED
- [x] Phase 1 checkpoint report (`docs/checkpoints/`)
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
- [ ] Test data generators for all data types — NOT DONE
- [ ] XS/S/M fixture datasets generated — NOT DONE
- [ ] Mock servers for all 3 internal services — NOT DONE
- [x] E2E telemetry test: Sim→Backend→Frontend verified (manual)
- [ ] Latency <100ms p95 confirmed — NOT MEASURED
- [x] Phase 1 checkpoint report written
