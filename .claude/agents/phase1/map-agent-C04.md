# Agent Assignment: map-agent — C-04 Map Core

## Agent Profile
- **ID:** map-agent
- **Team:** Team 5 (Map Manager)
- **Phase:** 1 (Prototype)
- **Feature:** C-04 Map Core
- **Duration:** Week 1-8

## Objective
Build point cloud upload/processing pipeline with Potree conversion, roadmap graph CRUD, and tile serving.

## Week-by-Week Tasks

### Week 1-2: Project Setup
- [x] Cargo scaffold (tonic, sqlx, aws-sdk-s3)
- [x] PostGIS schema: maps, pointcloud_data, processing_jobs
- [ ] MinIO bucket setup (`tiles/`, `raw/`) — NOT TESTED (Docker needed)
- [x] gRPC server skeleton (MapService)
- [x] Python processing environment (Open3D, laspy, numpy)
- **Deliverable:** gRPC server starts, DB ready

### Week 3-4: Point Cloud Pipeline
- [ ] UploadPointCloud RPC — SKELETON ONLY
- [x] Python pipeline code (ingest, preprocess, potree_convert)
- [ ] Statistical outlier removal — code exists, NOT TESTED
- [ ] PotreeConverter integration — PLACEHOLDER
- [ ] Upload octree tiles to MinIO tiles/{map_id}/ — NOT DONE
- [ ] GetProcessingStatus RPC — SKELETON ONLY
- **Deliverable:** Upload PLY → process → Potree tiles in MinIO

### Week 5-6: Tile Serving & Roadmap
- [ ] GetTile RPC — SKELETON ONLY
- [x] Roadmap graph CRUD: AddNode, UpdateNode, DeleteNode, AddEdge, DeleteEdge (A* + graph, 11 tests)
- [x] GetRoadmapGraph RPC (return all nodes + edges)
- [ ] PostGIS PointZ for nodes — NOT DONE (using plain x,y,z columns)
- [x] Roadmap node types: waypoint, charging_station, loading_dock
- **Deliverable:** Tiles served, roadmap editable

### Week 7-8: Integration
- [x] Backend proxy integration (roadmap REST proxy — hardcoded, now DB in Phase 2)
- [ ] Point cloud metadata endpoint — NOT DONE
- [ ] Integration tests — NOT DONE
- [ ] Performance benchmarks — NOT DONE (only A* benchmark)
- **Deliverable:** E2E: upload → process → serve tiles to Frontend

## Reference Documents
- Feature spec: `docs/features/core/C-04-map-core.md`
- Team tech: `docs/teams/team5-map-manager.md`
- Interfaces: `docs/integration/integration-spec.md` §map-service
- Test data: `docs/integration/test-data-spec.md` §pointcloud-data

## Dependencies
- Week 2: Proto freeze (proto-agent)
- Week 7: Backend proxy endpoints ready (C-03)

## Constraints
- Only modify `map-manager/`. Proto changes require RFC.
- Coordinate system: ENU (X-East, Y-North, Z-Up)

## Definition of Done
- [ ] Point cloud upload + Potree conversion pipeline — pipeline code exists but not tested end-to-end
- [ ] Tile serving via gRPC — SKELETON ONLY
- [x] Roadmap graph CRUD (nodes + edges) — with A* pathfinding, 11 tests
- [ ] Processing status tracking — SKELETON ONLY
- [ ] Integration tests pass — NOT DONE
