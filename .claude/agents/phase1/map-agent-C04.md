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
- [ ] Cargo scaffold (tonic, sqlx, aws-sdk-s3)
- [ ] PostgreSQL + PostGIS schema: maps, pointcloud_data, processing_jobs
- [ ] MinIO bucket setup (`tiles/`, `raw/`)
- [ ] gRPC server skeleton (MapService)
- [ ] Python processing environment (Open3D, laspy, numpy)
- **Deliverable:** gRPC server starts, DB ready

### Week 3-4: Point Cloud Pipeline
- [ ] UploadPointCloud RPC (chunked gRPC stream → MinIO raw/)
- [ ] Python pipeline: validate → coordinate normalize → voxel downsample (0.02m)
- [ ] Statistical outlier removal, normal estimation
- [ ] PotreeConverter integration (octree generation)
- [ ] Upload octree tiles to MinIO tiles/{map_id}/
- [ ] GetProcessingStatus RPC (pending/processing/completed/failed, progress %)
- **Deliverable:** Upload PLY → process → Potree tiles in MinIO

### Week 5-6: Tile Serving & Roadmap
- [ ] GetTile RPC (map_id, node_id, lod → binary tile from MinIO)
- [ ] Roadmap graph CRUD: AddNode, UpdateNode, DeleteNode, AddEdge, DeleteEdge
- [ ] GetRoadmapGraph RPC (return all nodes + edges)
- [ ] PostGIS PointZ for nodes, LineStringZ for edge paths
- [ ] Roadmap node types: waypoint, charging_station, loading_dock
- **Deliverable:** Tiles served, roadmap editable

### Week 7-8: Integration
- [ ] Backend proxy integration (tile HTTP proxy, roadmap REST proxy)
- [ ] Point cloud metadata endpoint (bounds, point_count, LOD levels)
- [ ] Integration tests (testcontainers: postgres+postgis, minio)
- [ ] Performance: tile serving <10ms, pipeline handles 1M points
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
- [ ] Point cloud upload + Potree conversion pipeline
- [ ] Tile serving via gRPC
- [ ] Roadmap graph CRUD (nodes + edges)
- [ ] Processing status tracking
- [ ] Integration tests pass
