# CLAUDE.md

## Project Overview
AMR (Autonomous Mobile Robot) integrated framework. A platform for comprehensive setup, remote operation, fleet management, and simulation.
5 sub-projects (Frontend, Backend, Sim Engine, Asset Manager, Map Manager) developed in parallel using an AI agent swarm.

---

## Skills (Strategy Guides)
| Skill | Path | Purpose |
|-------|------|---------|
| Reference Navigation | `.claude/skills/find-references.md` | Finding documents/code locations, document map |
| API Design | `.claude/skills/api-design.md` | REST/gRPC/WS/MQTT design rules |
| Documentation | `.claude/skills/documentation.md` | Code/design/history documentation guidelines |
| Communication | `.claude/skills/communication.md` | RFC, reports, issues, checkpoints |
| Git Workflow | `.claude/skills/git-workflow.md` | Branches, commits, merges, tags |
| Coding Standards | `.claude/skills/coding-standards.md` | Per-language patterns, naming, error handling |
| Inter-Phase Cleanup | `.claude/skills/phase-cleanup.md` | Legacy/temporary code cleanup, dependency/documentation/build cleanup |

## Memory & Context
| File | Purpose |
|------|---------|
| `.claude/progress.md` | Overall development progress (per-feature %, blockers, milestones) |
| `.claude/ownership.md` | File ownership matrix & conflict prevention strategy |

---

## Project Documents
- **Architecture:** `docs/architecture.md`
- **Development Plan:** `docs/development-plan.md`
- **Core Features (Phase 1):** `docs/features/core/C-01~C-06`
- **Advanced Features (Phase 2-4):** `docs/features/advanced/A-01~A-10`
- **Agent Allocation:** `docs/governance/agent-allocation.md`
- **Cross-Team Interfaces:** `docs/integration/integration-spec.md`
- **Test Data:** `docs/integration/test-data-spec.md`
- **Data Pipeline:** `docs/integration/data-pipeline.md`
- **Team Technical Docs:** `docs/teams/team{1-5}-*.md`

## Sub-Project Structure
```
amr/
├── proto/           # Protobuf definitions (single source of truth)
├── frontend/        # React + TypeScript + Three.js
├── backend/         # Rust + Axum
├── sim-engine/      # C++17/20
├── asset-manager/   # Rust + Python
├── map-manager/     # Rust + Python
├── deploy/          # Docker, K8s
├── test-data/       # Test data generators and fixtures
└── docs/            # All documents
```

---

## Session Start Checklist
1. `.claude/progress.md` — Check current progress
2. `.claude/ownership.md` — Verify my ownership scope
3. Read assigned feature documents (`docs/features/`)
4. `docs/rfcs/` — Check for unprocessed RFCs
5. `docs/issues/` — Check assigned issues
6. `docs/worklog/` — Review previous session

## Session End Checklist
1. Commit (following `.claude/skills/git-workflow.md` rules)
2. `.claude/progress.md` — Update progress
3. `docs/worklog/` — Write work log
4. `CHANGELOG.md` — Update
5. Record unresolved issues/RFCs
