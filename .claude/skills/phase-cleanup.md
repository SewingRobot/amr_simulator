# Skill: Inter-Phase Cleanup

Mandatory cleanup tasks between development phases, before starting the next phase.

---

## 1. Timing

```
Phase N features complete → Phase N integration test passes → ★ CLEANUP ★ → Phase N+1 starts
```

---

## 2. Legacy File Cleanup

- [ ] Remove temp files (`*_temp.*`, `*_backup.*`, `*_old.*`)
- [ ] Remove unused mocks (replaced by real implementations)
- [ ] Remove empty skeleton/placeholder files
- [ ] Delete commented-out code blocks (git history preserves them)
- [ ] Remove deprecated files from previous phases

**Verification:**
```bash
# Unused dependencies
npx depcheck              # frontend
cargo udeps               # rust projects
pip-audit && pipreqs --diff  # python
include-what-you-use      # C++
```

---

## 3. Temporary Code Cleanup

- [ ] `// TODO(temp)`, `// HACK`, `// WORKAROUND` → implement properly or convert to next-phase TODO
- [ ] Hardcoded values → move to config/env vars
- [ ] `console.log`, `println!`, `dbg!`, `std::cout` → remove or convert to proper logging
- [ ] `#[allow(...)]`, `// @ts-ignore`, `#pragma warning(disable:...)` → fix root cause
- [ ] Disabled tests (`#[ignore]`, `.skip`, `xit(`) → fix or delete

**Verification:**
```bash
grep -rn "console\.log\|println!\|dbg!\|std::cout" --include="*.ts" --include="*.rs" --include="*.cpp"
grep -rn "TODO(temp)\|HACK\|WORKAROUND" --include="*.ts" --include="*.rs" --include="*.cpp" --include="*.py"
grep -rn "#\[ignore\]\|\.skip\|xit(" --include="*.ts" --include="*.rs" --include="*.cpp"
```

---

## 4. Dependency Cleanup

- [ ] Remove unused packages/crates/libraries
- [ ] Verify lockfiles are current
- [ ] Security vulnerability scan (`npm audit`, `cargo audit`, `pip-audit`)

---

## 5. Documentation Cleanup

- [ ] Verify all DoD checklist items checked for completed features
- [ ] Write feature completion reports (`docs/reports/`)
- [ ] Finalize CHANGELOG section (`[Unreleased]` → `[x.y.z-phaseN]`)
- [ ] Close resolved issues (status → `closed`)
- [ ] Mark implemented RFCs (status → `implemented`)
- [ ] Update `.claude/progress.md`
- [ ] Archive previous phase work logs

---

## 6. Build & Test Cleanup

- [ ] Clean rebuild all sub-projects (`cargo clean && cargo build`, `rm -rf dist && npm run build`, etc.)
- [ ] Run full test suite (all sub-projects)
- [ ] Measure and record test coverage
- [ ] Rebuild Docker images (`docker compose build --no-cache`)
- [ ] Run integration tests (Docker Compose full stack)

---

## 7. Git Cleanup

- [ ] Delete merged feature branches
- [ ] Create Phase tag (`v0.1.0-phase1`)
- [ ] Create release branch (`release/phase-N`)
- [ ] Merge to main after stabilization

---

## 8. Checklist Summary

```
- [ ] Legacy/temp files deleted
- [ ] Debug code removed
- [ ] Temp TODO/HACK resolved or converted
- [ ] Unused dependencies removed + security scan
- [ ] Docs cleaned (reports, CHANGELOG, issues/RFCs)
- [ ] Clean build + all tests pass
- [ ] Docker rebuild + integration tests
- [ ] Feature branches deleted + Phase tag
- [ ] progress.md updated
```
