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
- [ ] Cargo workspace scaffold (Axum, tonic, sqlx, serde)
- [ ] PostgreSQL migrations: users, robots, maps tables
- [ ] Configuration management (env vars, config.rs)
- [ ] Health check endpoint (`GET /api/health`)
- [ ] Docker Compose dev: postgres, redis, minio
- **Deliverable:** Server starts, DB migrations run

### Week 3-4: REST API & Auth
- [ ] JWT auth: POST /api/auth/login, POST /api/auth/refresh, GET /api/auth/me
- [ ] Auth middleware (JWT validation, role extraction)
- [ ] Robot CRUD: GET/POST /api/robots, GET/PUT/DELETE /api/robots/:id
- [ ] Map CRUD: GET/POST /api/maps, GET/PUT/DELETE /api/maps/:id
- [ ] WebSocket upgrade handler (`/api/ws?token=jwt`)
- [ ] WS session: topic subscription, message routing
- **Deliverable:** REST API + WS connection working

### Week 5-6: Telemetry Bridge
- [ ] gRPC client for Sim Engine (tonic)
- [ ] StreamTelemetry → normalize → fan-out to WS clients
- [ ] Robot status management (update from telemetry)
- [ ] WS topic filtering (subscribe `telemetry:{robot_id}` or `telemetry:*`)
- [ ] Heartbeat ping/pong (30s interval)
- **Deliverable:** Sim telemetry flows to browser via WS

### Week 7-8: Proxy & Integration
- [ ] Asset proxy endpoints: GET/POST /api/assets (→ Asset Manager gRPC)
- [ ] Map proxy: POST /api/maps/:id/pointcloud/upload, GET /api/maps/:id/tiles/:nodeId
- [ ] Roadmap proxy: GET/PUT /api/maps/:id/roadmap
- [ ] Integration tests (testcontainers: postgres, redis)
- [ ] API tests: auth flow, CRUD, WS telemetry
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
- [ ] Axum server with health check
- [ ] JWT authentication working
- [ ] Robot/Map CRUD APIs
- [ ] WebSocket telemetry fan-out
- [ ] Sim Engine gRPC bridge
- [ ] Asset/Map proxy endpoints
- [ ] Integration tests pass
