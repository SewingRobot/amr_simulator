# Agent Assignment: infra-agent — Production Deployment

## Agent Profile
- **ID:** infra-agent | **Team:** Integration | **Phase:** 4 | **Duration:** Week 23-30
- **Feature:** Production infrastructure

## Objective
Prepare production deployment with Kubernetes, security hardening, and monitoring.

## Week-by-Week Tasks

### Week 23-24: Kubernetes
- [ ] Helm charts for all services (backend, sim-engine, asset-mgr, map-mgr, frontend)
- [ ] K8s namespace and resource limits
- [ ] ConfigMaps and Secrets management
- [ ] Ingress controller (nginx) with TLS

### Week 25-26: CI/CD
- [ ] CI pipeline: build → test → Docker push → K8s deploy (staging)
- [ ] Health check endpoints for all services
- [ ] Prometheus metrics endpoints
- [ ] Grafana dashboard templates

### Week 27-28: Security & Monitoring
- [ ] TLS everywhere (inter-service mTLS)
- [ ] Secrets rotation strategy
- [ ] Rate limiting configuration
- [ ] Log aggregation (ELK or Loki)
- [ ] OpenTelemetry trace propagation

### Week 29-30: Production Readiness
- [ ] Load testing with XL dataset (200 robots, 100M points)
- [ ] Backup automation (PostgreSQL WAL, MinIO versioning)
- [ ] Disaster recovery runbook
- [ ] Production deployment documentation
- [ ] Final smoke test on staging environment

## References
- `docs/architecture.md`, `docs/integration/integration-spec.md` §docker-compose

## Dependencies
- All services functional (Phase 1-3 complete)

## Definition of Done
- [ ] K8s Helm charts for all services
- [ ] CI/CD pipeline deploys to staging
- [ ] Monitoring (Prometheus + Grafana) operational
- [ ] TLS + secrets management configured
- [ ] Load test passes
- [ ] Deployment documentation complete
