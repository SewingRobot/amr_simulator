# Skill: Reference Navigation

Strategy for finding documents and code in the project.

---

## 1. Document Map

### 1.1 Architecture & Planning
| Purpose | Path |
|---------|------|
| Overall architecture, feature classification | `docs/architecture.md` |
| Phased development plan, Gantt | `docs/development-plan.md` |
| Agent allocation strategy | `docs/governance/agent-allocation.md` |

### 1.2 Feature Specs
| Purpose | Path |
|---------|------|
| Core features (Phase 1) | `docs/features/core/C-{01-06}-*.md` |
| Advanced features (Phase 2-4) | `docs/features/advanced/A-{01-10}-*.md` |

**Feature ID Quick Reference:**
- `C-01` Sim Engine Core, `C-02` 3D Viewer, `C-03` Backend Core
- `C-04` Map Core, `C-05` Asset Core, `C-06` Telemetry Pipeline
- `A-01` Mission, `A-02` Traffic, `A-03` Remote Control
- `A-04` VDA5050, `A-05` Advanced Sim, `A-06` Map Editor
- `A-07` Plugin, `A-08` Dashboard, `A-09` OpenUSD, `A-10` Digital Twin

### 1.3 Cross-Team Interfaces
| Purpose | Path |
|---------|------|
| Proto schemas, API specs, WS messages | `docs/integration/integration-spec.md` |
| Test data specification | `docs/integration/test-data-spec.md` |
| Data pipeline | `docs/integration/data-pipeline.md` |

### 1.4 Team Technical Details
| Team | Path |
|------|------|
| Frontend (React/Three.js) | `docs/teams/team1-frontend.md` |
| Backend (Rust/Axum) | `docs/teams/team2-backend.md` |
| Sim Engine (C++) | `docs/teams/team3-sim-engine.md` |
| Asset Manager (Rust/Python) | `docs/teams/team4-asset-manager.md` |
| Map Manager (Rust/Python) | `docs/teams/team5-map-manager.md` |

### 1.5 Operational Documents (created during development)
| Purpose | Path |
|---------|------|
| RFC (interface changes) | `docs/rfcs/RFC-{number}.md` |
| Issues | `docs/issues/ISSUE-{number}.md` |
| Work logs | `docs/worklog/{date}-{agent}-{feature}.md` |
| Status reports | `docs/status/{date}-team{N}.md` |
| ADR (design decisions) | `docs/decisions/ADR-{number}.md` |
| Completion reports | `docs/reports/{feature-id}-completion.md` |
| Checkpoints | `docs/checkpoints/{date}-phase{N}-checkpoint.md` |

---

## 2. Search Strategies

### "How should I implement this feature?"
1. `docs/features/{core|advanced}/{feature-id}.md` — scope, specs, modules, DoD
2. `docs/teams/team{N}.md` — tech stack, directory structure, code patterns
3. `docs/integration/integration-spec.md` — interfaces with other teams

### "What messages do I exchange with other teams?"
1. `docs/integration/integration-spec.md` — proto definitions, REST API, WS schemas
2. Follow `📋 Interface details: integration-spec.md` links in feature docs

### "How do I create test data?"
1. `docs/integration/test-data-spec.md` — per-type format/semantic requirements
2. `test-data/generators/` — generation scripts
3. `test-data/fixtures/` — pre-generated datasets

### "What changed recently?"
1. `CHANGELOG.md`, 2. `docs/rfcs/`, 3. `docs/worklog/`, 4. `git log --oneline -20`

### "Why was this decision made?"
1. `docs/decisions/ADR-{number}.md`, 2. Commit message `Refs:` field

---

## 3. Code Location Map

```
amr/
├── proto/                    # Protobuf definitions (all gRPC interfaces)
├── frontend/src/
│   ├── components/           # React components
│   ├── services/             # API/WS/WebRTC clients
│   ├── stores/               # Zustand state
│   └── types/                # TypeScript types (incl. proto-generated)
├── backend/src/
│   ├── api/                  # REST handlers, WebSocket
│   ├── services/             # Business logic
│   ├── grpc/                 # gRPC clients
│   ├── mqtt/                 # VDA5050 bridge
│   └── db/                   # PostgreSQL queries
├── sim-engine/src/
│   ├── core/                 # Sim loop, world
│   ├── physics/              # Physics backends (custom/mujoco/isaac)
│   ├── robots/               # Robot models, kinematics
│   ├── sensors/              # Sensor simulation
│   └── grpc/                 # gRPC server
├── asset-manager/src/        # Asset gRPC service
├── asset-manager/processing/ # Python conversion pipeline
├── map-manager/src/          # Map gRPC service
└── map-manager/processing/   # Python point cloud pipeline
```
