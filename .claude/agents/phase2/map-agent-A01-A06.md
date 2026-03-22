# Agent Assignment: map-agent — A-01 Pathfinding + A-06 Semantic Layer

## Agent Profile
- **ID:** map-agent
- **Team:** Team 5 | **Phase:** 2 | **Duration:** Week 9-16
- **Features:** A-01 (pathfinding), A-06 (semantic layer, obstacles)

## Objective
Implement A* pathfinding on roadmap graph, semantic region CRUD with PostGIS, and obstacle management.

## Week-by-Week Tasks

### Week 9-10: A* Pathfinding + Phase 1 Carryover
- [ ] Graph data structure (adjacency list from roadmap)
- [ ] A* algorithm with Euclidean heuristic
- [ ] Robot type filtering (edges with allowed_robot_types)
- [ ] FindPath gRPC RPC, FindNearestNode RPC
- [ ] **Carryover C-04:** Complete Potree tile serving with MinIO integration (point cloud upload/process/serve)

### Week 11-12: Semantic Layer
- [ ] DB schema: semantic_regions (PostGIS POLYGON geometry)
- [ ] CRUD RPCs: AddRegion, UpdateRegion, DeleteRegion, GetSemanticLayer
- [ ] Region types: no_go_zone, speed_limit_zone, charging_area, loading_dock
- [ ] Spatial queries: point-in-polygon, path-crosses-region

### Week 13-14: Obstacles
- [ ] Static obstacles: PostgreSQL with PostGIS geometry
- [ ] Dynamic obstacles: Redis with TTL (ephemeral, from robot detection)
- [ ] GetObstacles, AddObstacle, UpdateDynamicObstacles RPCs
- [ ] Obstacles-near-path query

### Week 15-16: Conflict Check & Performance
- [ ] CheckPathConflict RPC (path + time window vs active robots)
- [ ] A* benchmark: <5ms for 10K node graph
- [ ] Spatial query benchmark: <10ms for 1000 regions
- [ ] Integration tests

## References
- `docs/features/advanced/A-01-mission-system.md` §pathfinding
- `docs/features/advanced/A-06-map-editor.md` §semantic

## Dependencies
- C-04 Map Core (roadmap graph)

## Definition of Done
- [ ] A* pathfinding correct and fast (<5ms)
- [ ] Semantic layer CRUD with PostGIS
- [ ] Obstacle management (static + dynamic)
- [ ] Conflict detection API working
