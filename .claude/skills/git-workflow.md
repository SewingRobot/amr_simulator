# Skill: Git Workflow

---

## 1. Branch Strategy

**Main branches:** `main` (production, no direct push), `develop` (integration point)

**Naming:** `{type}/{phase}-{feature-id}-{short-description}`

| Type | Purpose | Example |
|------|---------|---------|
| `feature/` | New feature | `feature/p1-C01-sim-engine-core` |
| `fix/` | Bug fix | `fix/p1-C06-websocket-reconnect` |
| `refactor/` | Refactoring | `refactor/p2-backend-service-layer` |
| `docs/` | Documentation | `docs/integration-spec-update` |
| `test/` | Tests | `test/p1-C06-telemetry-e2e` |
| `infra/` | CI/CD, Docker | `infra/docker-compose-setup` |
| `integration/` | Cross-team (proto/) | `integration/p1-proto-schema-v1` |

**Sub-project independence:** Each team modifies only its own directory. Cross-team changes go in `integration/` branches.

**Phase releases:** `develop` → `release/phase-{N}` → stabilize → `main` + tag

---

## 2. Commit Rules

**Format:**
```
{type}({scope}): {subject}

{body}

{footer}
```
- **Type:** `feat`, `fix`, `refactor`, `test`, `docs`, `style`, `perf`, `ci`, `chore`
- **Scope:** `sim`, `frontend`, `backend`, `asset`, `map`, `proto`, `infra`
- **Subject:** ≤50 chars, present tense, lowercase, no period
- **Footer:** `Refs: C-01-sim-engine-core.md`, `BREAKING CHANGE: ...`

**Principles:** Atomic (1 commit = 1 logical change), buildable, tests pass, no WIP/temp commits

---

## 3. Merge Rules

**Checklist:** Build passes + tests pass + lint clean + integration-spec checked (if cross-team) + proto codegen verified (if proto/ changed)

| Target | Method |
|--------|--------|
| feature → develop | Squash and Merge |
| develop → release | Merge Commit |
| release → main | Merge Commit + Tag |

**Conflict resolution:** proto/ → proto-agent, sub-project → team agent, cross-team → both teams collaborate

---

## 4. Tags & Protection

**Tags:** Phase `v0.1.0-phase1`, feature `C-01-done`, checkpoint `integration-check-20260401`

**Protection:** `main`, `develop`, `release/*` — no force push, no direct push

---

## 5. .gitignore

```gitignore
target/ dist/ build/ node_modules/ __pycache__/ *.pyc
.env .env.local *.local
.idea/ .vscode/ *.swp
.DS_Store
data/ *.las *.laz *.ply *.pcd
*.glb *.gltf *.usd *.usda *.usdc *.stl *.dae
**/generated/ **/proto_gen/
test-data/fixtures/m/ test-data/fixtures/l/ test-data/fixtures/xl/
```
