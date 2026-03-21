# Agent Assignment: asset-agent — C-05 Asset Core

## Agent Profile
- **ID:** asset-agent
- **Team:** Team 4 (Asset Manager)
- **Phase:** 1 (Prototype)
- **Feature:** C-05 Asset Core
- **Duration:** Week 1-8

## Objective
Build asset upload/download service with metadata CRUD and basic URDF→glTF conversion pipeline.

## Week-by-Week Tasks

### Week 1-2: Project Setup
- [ ] Cargo scaffold (tonic, sqlx, aws-sdk-s3)
- [ ] PostgreSQL schema: assets, asset_files (with SHA256 checksums)
- [ ] MinIO bucket setup (`assets/{type}/{id}/{format}/`)
- [ ] gRPC server skeleton (AssetService)
- [ ] Python processing environment (urdfpy, trimesh, pygltflib)
- **Deliverable:** gRPC server starts, DB + MinIO ready

### Week 3-4: Upload/Download & Metadata
- [ ] CreateAsset RPC (chunked gRPC stream → MinIO + DB metadata)
- [ ] DownloadAsset RPC (MinIO → gRPC stream)
- [ ] ListAssets RPC (filter by type, tags, name)
- [ ] GetAsset RPC (metadata + file info)
- [ ] DeleteAsset RPC (cascade: DB + MinIO cleanup)
- [ ] Asset types: robot_model, static_object (Phase 1 minimum)
- **Deliverable:** Upload/download/list/delete working

### Week 5-6: URDF→glTF Conversion
- [ ] Python URDF parser (urdfpy): extract links, joints, meshes
- [ ] Mesh loading (STL/DAE/OBJ via trimesh)
- [ ] glTF export (pygltflib): combine meshes into single GLB
- [ ] ConvertAsset RPC trigger → Python subprocess
- [ ] Conversion job status tracking (pending/processing/completed/failed)
- **Deliverable:** Upload URDF → auto-convert → download glTF

### Week 7-8: Integration
- [ ] Backend proxy integration (REST /api/assets/*)
- [ ] Validation: mesh integrity check, file size limits
- [ ] Integration tests (testcontainers)
- [ ] Test with sample URDF (TurtleBot3-like)
- **Deliverable:** E2E: upload URDF → convert → Frontend renders glTF

## Reference Documents
- Feature spec: `docs/features/core/C-05-asset-core.md`
- Team tech: `docs/teams/team4-asset-manager.md`
- Interfaces: `docs/integration/integration-spec.md` §asset-service
- Test data: `docs/integration/test-data-spec.md` §robot-model-data

## Dependencies
- Week 2: Proto freeze (proto-agent)
- Week 7: Backend proxy ready (C-03)

## Constraints
- Only modify `asset-manager/`. Proto changes require RFC.
- Phase 1: URDF→glTF only. OpenUSD conversion deferred to A-09.

## Definition of Done
- [ ] Asset upload/download (chunked streaming)
- [ ] Metadata CRUD with filtering
- [ ] URDF→glTF conversion pipeline
- [ ] Conversion job status tracking
- [ ] Integration tests pass
