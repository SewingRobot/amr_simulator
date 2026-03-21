# Skill: Documentation & Development History

---

## 1. Code Documentation

**Public API — must document:** JSDoc (TS), rustdoc (Rust), Doxygen (C++), docstring (Python)

**Internal:** Comment only non-obvious algorithms/logic

**Markers:**
```
// TODO(team-id): description [feature-ref]
// FIXME(team-id): description [ISSUE-number]
// CROSS-TEAM: message [target-team] [feature-ref]
```

---

## 2. CHANGELOG

Location: `CHANGELOG.md` (root) + per sub-project
```markdown
## [Unreleased]
### Added
- feat(sim): Differential drive kinematics (C-01) [2026-03-25]
### Fixed
- fix(frontend): WebSocket reconnection (C-06) [2026-03-28]
```
- Update when feature branch merges to develop
- Include date + feature reference

---

## 3. ADR (Architecture Decision Records)

Location: `docs/decisions/ADR-{number}.md`
```markdown
# ADR-{number}: {title}
**Date:** ... | **Status:** accepted | **Features:** C-01, A-05
## Context → Decision → Alternatives → Consequences (positive/negative/neutral)
```
When to write: tech stack selection, architecture changes, interface decisions, trade-offs

---

## 4. Work Log

Location: `docs/worklog/{date}-{agent-id}-{feature}.md`
- **Mandatory at every session end**
- Structure: Goals → Completed → Incomplete → Issues → Cross-team notes → Next plan

---

## 5. Feature Completion Report

Location: `docs/reports/{feature-id}-completion.md`
- DoD checklist → Test results table → Known limitations → Follow-up features

---

## 6. Status Report

Location: `docs/status/{date}-team{N}.md`
- Completed / In-progress (%) / Blockers / Cross-team requests / Next week plan
- Weekly cadence

---

## 7. Quality Rules
1. English technical terms, keep code/paths as-is
2. No duplication — use `📋 See: [doc](path)` cross-references
3. Update docs in same commit as implementation changes
4. Be specific with metrics ("error < 1cm" not "accurate")
