# Agent Assignment: infra-agent — Infrastructure & DevOps

## Agent Profile
- **ID:** infra-agent
- **Team:** Integration
- **Phase:** 1 (Prototype)
- **Feature:** Cross-cutting (deploy/, CI/CD)
- **Duration:** Week 1-8

## Objective
Set up monorepo structure, Docker Compose development environment, CI pipeline, and integration test infrastructure.

## Week-by-Week Tasks

### Week 1-2: Monorepo & Dev Environment
- [x] Root directory structure (proto/, frontend/, backend/, sim-engine/, etc.)
- [x] Root `.gitignore` (per git-workflow.md)
- [x] Docker Compose dev: PostgreSQL 16 + PostGIS, Redis 7, MinIO, Mosquitto
- [x] Docker Compose health checks for all services
- [ ] CI pipeline skeleton (GitHub Actions or similar) — NOT DONE
- [ ] Lint/format checks per sub-project in CI — NOT DONE
- **Deliverable:** `docker compose up` starts all infra services

### Week 3-4: Sub-project Dockerfiles
- [ ] `Dockerfile.backend` (multi-stage Rust build) — NOT DONE
- [ ] `Dockerfile.sim-engine` (multi-stage C++ build) — NOT DONE
- [ ] `Dockerfile.asset-manager` (Rust + Python) — NOT DONE
- [ ] `Dockerfile.map-manager` (Rust + Python) — NOT DONE
- [ ] `Dockerfile.frontend` (Node build + nginx serve) — NOT DONE
- [ ] Docker Compose profiles (dev, test, full) — NOT DONE
- **Deliverable:** All sub-projects containerized

### Week 5-6: CI Pipeline
- [ ] CI: build all sub-projects on push — NOT DONE
- [ ] CI: run unit tests per sub-project — NOT DONE
- [ ] CI: lint/format checks — NOT DONE
- [ ] CI: proto lint (buf lint) — NOT DONE
- [ ] CI: proto breaking change detection (buf breaking) — NOT DONE
- **Deliverable:** CI green on develop branch

### Week 7-8: Integration Environment
- [ ] Docker Compose integration profile (all services) — NOT DONE
- [x] Wait-for-services script (health check polling)
- [ ] Integration test runner script — NOT DONE
- [ ] Environment variable templates (`.env.example`) — NOT DONE
- [ ] Phase 1 full-stack smoke test — NOT DONE
- **Deliverable:** `docker compose --profile integration up` works E2E

## Reference Documents
- Architecture: `docs/architecture.md`
- Integration spec: `docs/integration/integration-spec.md` §docker-compose
- Agent allocation: `docs/governance/agent-allocation.md`

## Dependencies
- Week 3+: Sub-project Dockerfiles depend on initial project scaffolds from team agents

## Constraints
- Exclusive write access to `deploy/`, `docker-compose*.yml`, `.github/`
- CI changes should not break existing builds

## Definition of Done
- [x] Docker Compose dev environment working
- [ ] All sub-project Dockerfiles built — NOT DONE
- [ ] CI pipeline running (build + test + lint) — NOT DONE
- [ ] Integration test environment operational — NOT DONE
- [ ] Phase 1 full-stack smoke test passes — NOT DONE
