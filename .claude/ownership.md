# File Ownership & Conflict Prevention

> Ownership matrix and rules to prevent file/directory conflicts during parallel agent work.

---

## 1. Directory Ownership Matrix

### 1.1 Code Directories (Exclusive Write)

| Directory | Owner Agent | Others |
|-----------|------------|--------|
| `frontend/` | frontend-agent | Read-only |
| `backend/` | backend-agent | Read-only |
| `sim-engine/` | sim-agent | Read-only |
| `asset-manager/` | asset-agent | Read-only |
| `map-manager/` | map-agent | Read-only |
| `proto/` | proto-agent | Read-only (RFC required for changes) |
| `deploy/` | infra-agent | Read-only |
| `test-data/` | test-agent | Read-only |

### 1.2 Shared Write Directories

| Directory | Write Access | Rule |
|-----------|-------------|------|
| `docs/rfcs/` | All agents | Anyone can create RFCs |
| `docs/issues/` | All agents | Anyone can report issues |
| `docs/worklog/` | All agents | Each writes own files (`{date}-{agent}-*`) |
| `docs/status/` | All agents | Each team writes own report |
| `docs/decisions/` | All agents | Anyone can write ADRs |
| `docs/reports/` | All agents | Assigned agent writes completion report |
| `docs/checkpoints/` | test-agent | Integration test results |
| `docs/features/` | docs-agent | Feature spec changes via RFC |
| `docs/teams/` | Each team agent | Own team doc only |

### 1.3 Config Files

| File | Owner |
|------|-------|
| `CLAUDE.md` | docs-agent |
| `.claude/progress.md` | All (own section only) |
| `.claude/ownership.md` | docs-agent |
| `CHANGELOG.md` | All (own team entries only) |
| `.gitignore` | infra-agent |
| `docker-compose*.yml` | infra-agent |

---

## 2. Conflict Prevention Rules

### 2.1 Exclusive Ownership
Each agent modifies only its owned directories. Violations are flagged by review-agent at merge time.

### 2.2 Shared File Protocol
For `CHANGELOG.md`, `.claude/progress.md`:
- **Section-based separation:** Each agent edits only its team/feature section
- **Append-only:** Don't modify existing content, only add new entries
- **On conflict:** Rebase on latest develop, reapply own changes only

### 2.3 Proto Change Flow
1. Team agent writes RFC → 2. Affected teams approve → 3. proto-agent modifies proto/ → 4. proto-agent notifies all → 5. Each team regenerates code

---

## 3. Concurrency Matrix

```
              frontend  backend  sim-engine  asset-mgr  map-mgr  proto  deploy  test-data
frontend-agent   ✅        👁️       👁️         👁️        👁️      👁️     👁️       👁️
backend-agent    👁️        ✅       👁️         👁️        👁️      👁️     👁️       👁️
sim-agent        👁️        👁️       ✅         👁️        👁️      👁️     👁️       👁️
asset-agent      👁️        👁️       👁️         ✅        👁️      👁️     👁️       👁️
map-agent        👁️        👁️       👁️         👁️        ✅      👁️     👁️       👁️
proto-agent      👁️        👁️       👁️         👁️        👁️      ✅     👁️       👁️
infra-agent      👁️        👁️       👁️         👁️        👁️      👁️     ✅       👁️
test-agent       👁️        👁️       👁️         👁️        👁️      👁️     👁️       ✅
```
✅ = write, 👁️ = read-only

### Parallelization
- **Always safe:** Agents modifying different directories
- **Caution:** Shared files (CHANGELOG, progress.md) — use section-based separation
- **Serial only:** Proto changes → all teams regenerate (wait for proto-agent)

---

## 4. Branch Ownership

| Branch Pattern | Owner | Example |
|---------------|-------|---------|
| `feature/p*-C01-*` | sim-agent | `feature/p1-C01-physics-engine` |
| `feature/p*-C02-*` | frontend-agent | `feature/p1-C02-3d-viewer` |
| `feature/p*-C03-*` | backend-agent | `feature/p1-C03-rest-api` |
| `feature/p*-C04-*` | map-agent | `feature/p1-C04-pointcloud` |
| `feature/p*-C05-*` | asset-agent | `feature/p1-C05-upload` |
| `feature/p*-C06-*` | test-agent | `feature/p1-C06-telemetry-e2e` |
| `integration/*` | proto-agent / infra-agent | `integration/p1-proto-v1` |
