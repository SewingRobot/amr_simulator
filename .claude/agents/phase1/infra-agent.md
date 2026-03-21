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
- [ ] Root directory structure (proto/, frontend/, backend/, sim-engine/, etc.)
- [ ] Root `.gitignore` (per git-workflow.md)
- [ ] Docker Compose dev: PostgreSQL 16 + PostGIS, Redis 7, MinIO, Mosquitto
- [ ] Docker Compose health checks for all services
- [ ] CI pipeline skeleton (GitHub Actions or similar)
- [ ] Lint/format checks per sub-project in CI
- **Deliverable:** `docker compose up` starts all infra services

### Week 3-4: Sub-project Dockerfiles
- [ ] `Dockerfile.backend` (multi-stage Rust build)
- [ ] `Dockerfile.sim-engine` (multi-stage C++ build)
- [ ] `Dockerfile.asset-manager` (Rust + Python)
- [ ] `Dockerfile.map-manager` (Rust + Python)
- [ ] `Dockerfile.frontend` (Node build + nginx serve)
- [ ] Docker Compose profiles (dev, test, full)
- **Deliverable:** All sub-projects containerized

### Week 5-6: CI Pipeline
- [ ] CI: build all sub-projects on push
- [ ] CI: run unit tests per sub-project
- [ ] CI: lint/format checks (rustfmt, clippy, eslint, clang-format, ruff)
- [ ] CI: proto lint (buf lint)
- [ ] CI: proto breaking change detection (buf breaking)
- **Deliverable:** CI green on develop branch

### Week 7-8: Integration Environment
- [ ] Docker Compose integration profile (all services)
- [ ] Wait-for-services script (health check polling)
- [ ] Integration test runner script
- [ ] Environment variable templates (`.env.example`)
- [ ] Phase 1 full-stack smoke test
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
- [ ] Docker Compose dev environment working
- [ ] All sub-project Dockerfiles built
- [ ] CI pipeline running (build + test + lint)
- [ ] Integration test environment operational
- [ ] Phase 1 full-stack smoke test passes
