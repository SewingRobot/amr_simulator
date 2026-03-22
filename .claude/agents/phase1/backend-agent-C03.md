# Agent Assignment: backend-agent — C-03 Backend Core

## Agent Profile
- **ID:** backend-agent
- **Team:** Team 2 (Backend)
- **Phase:** 1 (Prototype)
- **Feature:** C-03 Backend Core
- **Duration:** Week 1-8

## Objective
Build Axum REST API server with JWT auth, WebSocket telemetry relay, and gRPC bridge to Sim Engine.

## Week-by-Week Tasks

### Week 1-2: Project Setup
- [x] Cargo workspace scaffold (Axum, tonic, sqlx, serde)
- [x] PostgreSQL migrations: users, robots, maps tables
- [x] Configuration management (env vars, config.rs)
- [x] Health check endpoint (`GET /api/health`)
- [x] Docker Compose dev: postgres, redis, minio (in deploy/)
- **Deliverable:** Server starts, DB migrations run

### Week 3-4: REST API & Auth
- [x] JWT auth: POST /api/auth/login, POST /api/auth/refresh, GET /api/auth/me
- [x] Auth middleware (JWT validation, role extraction)
- [x] Robot CRUD: GET/POST /api/robots, GET/PUT/DELETE /api/robots/:id
- [x] Map CRUD: GET/POST /api/maps, GET/PUT/DELETE /api/maps/:id
- [x] WebSocket upgrade handler (`/api/ws?token=jwt`)
- [x] WS session: topic subscription, message routing
- **Deliverable:** REST API + WS connection working

### Week 5-6: Telemetry Bridge
- [x] TCP client for Sim Engine — TCP JSON instead of gRPC
- [x] StreamTelemetry → normalize → fan-out to WS clients
- [x] Robot status management (update from telemetry)
- [x] WS topic filtering (subscribe `telemetry:{robot_id}` or `telemetry:*`)
- [x] Heartbeat ping/pong (30s interval)
- **Deliverable:** Sim telemetry flows to browser via WS

### Week 7-8: Proxy & Integration
- [x] Asset proxy endpoints (basic static file serving)
- [x] Map tile proxy (stub)
- [x] Roadmap proxy (hardcoded; now being replaced with DB in Phase 2)
- [ ] Integration tests (testcontainers: postgres, redis) — NO testcontainers yet
- [x] API tests: auth flow, CRUD, WS telemetry (10 auth tests)
- **Deliverable:** Full Backend proxy working, E2E tested

## Reference Documents
- Feature spec: `docs/features/core/C-03-backend-core.md`
- Team tech: `docs/teams/team2-backend.md`
- REST API: `docs/integration/integration-spec.md` §rest-api
- WS schema: `docs/integration/integration-spec.md` §websocket-messages

## Dependencies
- Week 2: Proto freeze (proto-agent)
- Week 5: Sim Engine gRPC server (C-01) or mock
- Week 7: Asset/Map Manager gRPC servers (C-04, C-05) or mocks

## Constraints
- Only modify `backend/`. Proto changes require RFC.
- Use sqlx compile-time checked queries

## Definition of Done
- [x] Axum server with health check
- [x] JWT authentication working
- [x] Robot/Map CRUD APIs
- [x] WebSocket telemetry fan-out
- [x] Sim Engine TCP bridge (TCP JSON instead of gRPC)
- [x] Asset/Map proxy endpoints (basic static file serving / stubs)
- [ ] Integration tests pass — API tests only, no testcontainers
