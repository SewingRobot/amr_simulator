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
- [x] Cargo scaffold (tonic, sqlx, aws-sdk-s3)
- [x] PostgreSQL schema: assets, asset_files (with SHA256 checksums)
- [ ] MinIO bucket setup — NOT TESTED (Docker needed)
- [x] gRPC server skeleton (AssetService)
- [x] Python processing environment (urdfpy, trimesh, pygltflib)
- **Deliverable:** gRPC server starts, DB + MinIO ready

### Week 3-4: Upload/Download & Metadata
- [ ] CreateAsset RPC — SKELETON ONLY
- [ ] DownloadAsset RPC — SKELETON ONLY
- [ ] ListAssets RPC — SKELETON ONLY
- [ ] GetAsset RPC — SKELETON ONLY
- [ ] DeleteAsset RPC — SKELETON ONLY
- [x] Asset types: robot_model, static_object (Phase 1 minimum)
- **Deliverable:** Upload/download/list/delete working

### Week 5-6: URDF→glTF Conversion
- [x] Python URDF parser (urdfpy): extract links, joints, meshes
- [x] Mesh loading (STL/DAE/OBJ via trimesh)
- [x] glTF export (pygltflib): combine meshes into single GLB
- [ ] ConvertAsset RPC trigger → Python subprocess — SKELETON ONLY
- [ ] Conversion job status tracking — SKELETON ONLY
- **Deliverable:** Upload URDF → auto-convert → download glTF

### Week 7-8: Integration
- [x] Backend proxy integration (basic static file serving)
- [x] Validation: mesh integrity check, file size limits
- [ ] Integration tests — NOT DONE
- [ ] Test with sample URDF — NOT DONE
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
- [ ] Asset upload/download (chunked streaming) — SKELETON ONLY
- [ ] Metadata CRUD with filtering — SKELETON ONLY
- [x] URDF→glTF conversion pipeline (Python code: urdfpy + trimesh + glTF export)
- [ ] Conversion job status tracking — SKELETON ONLY
- [ ] Integration tests pass — NOT DONE
