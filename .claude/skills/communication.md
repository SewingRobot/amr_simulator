# Skill: Agent Communication

---

## 1. Communication Channels

| Channel | Location | Timing |
|---------|----------|--------|
| RFC | `docs/rfcs/RFC-{N}.md` | When interface change needed |
| Issue | `docs/issues/ISSUE-{N}.md` | When bug/blocker found |
| Work Log | `docs/worklog/{date}-{agent}-{feature}.md` | Every session end |
| Status | `docs/status/{date}-team{N}.md` | Weekly |
| Checkpoint | `docs/checkpoints/{date}-phase{N}.md` | Phase milestones |

---

## 2. Session Protocol

**Start:** Check rfcs/ → issues/ → dependent team status/ → progress.md
**End:** commit → update progress.md → write worklog/ → update CHANGELOG → record issues/RFCs

---

## 3. RFC Process

**Requires RFC:** proto/ changes, REST API changes, WS message changes, shared convention changes
**No RFC needed:** Sub-project internal changes, tests, doc typos

**Format:** `docs/rfcs/RFC-{N}.md`
```markdown
# RFC-{N}: {title}
**Author:** {agent} | **Status:** draft→open→accepted→implemented
**Affected teams:** ... | **Features:** ...
## Summary → Motivation → Before/After → Impact → Migration → Approval checklist
```

**Timing:** urgent=next session (auto-approve after 24h), normal=2 sessions, low=weekly review

---

## 4. Issue Tracker

`docs/issues/ISSUE-{N}.md` — Status: open→in-progress→resolved | Severity: critical/high/medium/low

---

## 5. Integration Checkpoints

| Week | Checkpoint | Verification |
|------|-----------|-------------|
| 2 | Proto Freeze | Proto finalized + code gen |
| 3 | Mock Ready | Team mock servers working |
| 8 | Phase 1 E2E | Sim→Backend→Frontend |
| 16 | Phase 2 E2E | Mission→Sim→Monitor |
| 22 | Phase 3 E2E | Real robot + sim hybrid |
| 30 | Phase 4 Final | Production ready |

---

## 6. Priority

| Priority | Scope | Timing |
|----------|-------|--------|
| Immediate | proto/ changes, build breaks, critical issues | Same session |
| Next session | RFC review, high issues | Next day |
| Weekly | Status reports, medium/low issues | Weekly |
