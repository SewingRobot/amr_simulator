# Agent Assignment: backend-agent — A-02 Traffic Management

## Agent Profile
- **ID:** backend-agent (or backend-agent-traffic if split)
- **Team:** Team 2 | **Phase:** 2 | **Duration:** Week 9-16
- **Feature:** A-02 Traffic Management

## Objective
Implement zone locking, conflict detection, priority queues, and deadlock prevention for multi-robot coordination.

## Week-by-Week Tasks

### Week 9-10: Zone Locking
- [ ] Redis data structure: zone_id → {robot_id, acquired_at, ttl}
- [ ] Acquire/release lock API (internal service)
- [ ] Timeout-based auto-release (default 60s)

### Week 11-12: Conflict Detection
- [ ] Pre-assignment path conflict check (does path overlap locked zones?)
- [ ] Zone overlap calculation using roadmap edge → zone mapping

### Week 13-14: Priority & Deadlock
- [ ] Priority queue: higher-priority missions get zone access first
- [ ] Deadlock detection: cycle detection in zone wait graph
- [ ] Deadlock resolution: timeout + lower-priority robot yields

### Week 15-16: Testing
- [ ] 5-robot overlapping path scenario (no collision, no deadlock)
- [ ] Priority override scenario
- [ ] Stress test: 50 robots, 20 zones

## References
- `docs/features/advanced/A-02-traffic-management.md`

## Dependencies
- A-01 Mission System, C-04 Map Core (roadmap graph)

## Definition of Done
- [ ] Zone locking via Redis working
- [ ] Conflict detection prevents overlapping assignments
- [ ] No deadlocks in multi-robot scenarios
- [ ] Stress test passes
