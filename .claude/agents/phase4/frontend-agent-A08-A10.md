# Agent Assignment: frontend-agent — A-08 Dashboard + A-10 Digital Twin

## Agent Profile
- **ID:** frontend-agent | **Team:** Team 1 | **Phase:** 4 | **Duration:** Week 23-30
- **Features:** A-08 Dashboard & Analytics, A-10 Digital Twin

## Objective
Build fleet KPI dashboard, analytics, telemetry replay, and digital twin hybrid overlay.

## Week-by-Week Tasks

### Week 23-24: Dashboard KPIs
- [ ] FleetOverview: robot count by status (idle/busy/error/offline)
- [ ] KPICards: missions/day, avg mission time, fleet utilization % (Recharts)
- [ ] AlertFeed: real-time alerts from WebSocket
- [ ] MissionQueue: pending/active missions list

### Week 25-26: Analytics & Replay
- [ ] Heatmap: robot activity overlay on map (D3 or Deck.gl)
- [ ] Telemetry replay UI: time slider, playback controls, speed adjustment
- [ ] Historical query integration (TimescaleDB via Backend REST)
- [ ] Mission history table with filtering/sorting

### Week 27-28: Digital Twin
- [ ] Full hybrid overlay: real robot positions + sim predictions
- [ ] Near-future prediction display (sim runs 10s ahead)
- [ ] Visual indicators: predicted path vs actual path
- [ ] Toggle between real/sim/hybrid views

### Week 29-30: Commissioning & Polish
- [ ] New robot commissioning workflow (sim first → real transition UI)
- [ ] Plugin management settings page
- [ ] i18n preparation, accessibility audit
- [ ] Performance optimization: WebGPU renderer for 100M+ point clouds

## References
- `docs/features/advanced/A-08-dashboard-analytics.md`
- `docs/features/advanced/A-10-digital-twin.md`

## Dependencies
- A-01 Mission System, A-04 VDA5050, A-05 Advanced Sim

## Definition of Done
- [ ] Dashboard with live KPIs and alerts
- [ ] Telemetry replay working
- [ ] Digital twin hybrid mode functional
- [ ] Commissioning workflow complete
