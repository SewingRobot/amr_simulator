# C-04: Map Core

> **Phase**: 1
> **담당 팀**: Team 5 (Map Manager, 주도) + Team 2 (Backend, HTTP 프록시)
> **상태**: Draft
> **최종 수정일**: 2026-03-22

---

## 1. 개요

포인트 클라우드 데이터의 업로드·처리·Potree 타일 변환 파이프라인과,
로드맵 그래프의 기본 CRUD를 구현하여 AMR 맵 관리의 핵심 기반을 확립한다.

**핵심 흐름:**
```
Frontend → Backend REST → Backend gRPC Client → Map Manager gRPC Server → MinIO / PostgreSQL
```

---

## 2. 스코프

### 2.1 In-Scope

| # | 항목 | 설명 |
|---|------|------|
| 1 | gRPC MapService 스켈레톤 | CreateMap, GetMap, ListMaps, UpdateMap, DeleteMap |
| 2 | 포인트 클라우드 업로드 | chunked gRPC stream (4MB/chunk) → MinIO 임시 저장 |
| 3 | Python 처리 파이프라인 | LAS/PLY → validate → downsample → PotreeConverter → MinIO 타일 저장 |
| 4 | Potree 타일 서빙 | GetTile RPC + Backend HTTP proxy (`/api/maps/:id/tiles/:node_id`) |
| 5 | 로드맵 그래프 CRUD | AddNode, AddEdge, GetRoadmapGraph, UpdateNode, DeleteNode, UpdateEdge, DeleteEdge |
| 6 | PostgreSQL + PostGIS 스키마 | maps, pointcloud_data, roadmap_nodes, roadmap_edges |
| 7 | 처리 상태 폴링 | GetProcessingStatus RPC (pending → processing → completed/failed) |

### 2.2 Out-of-Scope (A-06에서 구현)

- 시맨틱 레이어 (영역, 장애물)
- A* 경로 계획 (FindPath, FindNearestNode)
- 맵 버전 관리
- 동적 장애물 Redis 캐시
- 맵 편집 UI (Frontend)
- Redis 타일 캐시

---

## 3. 상세 스펙

### 3.1 gRPC 서비스 정의 (Phase 1 범위)

```protobuf
service MapService {
  // Map CRUD
  rpc CreateMap(CreateMapRequest) returns (MapDescriptor);
  rpc GetMap(GetMapRequest) returns (MapDescriptor);
  rpc ListMaps(ListMapsRequest) returns (ListMapsResponse);
  rpc UpdateMap(UpdateMapRequest) returns (MapDescriptor);
  rpc DeleteMap(DeleteMapRequest) returns (DeleteMapResponse);

  // Point Cloud
  rpc UploadPointCloud(stream PointCloudChunk) returns (ProcessingJob);
  rpc GetProcessingStatus(GetProcessingStatusRequest) returns (ProcessingStatus);
  rpc GetPointCloudMetadata(GetPointCloudMetadataRequest) returns (PointCloudMetadata);
  rpc GetTile(GetTileRequest) returns (TileData);

  // Roadmap Graph
  rpc GetRoadmapGraph(GetRoadmapRequest) returns (RoadmapGraph);
  rpc AddNode(AddNodeRequest) returns (RoadmapNode);
  rpc UpdateNode(UpdateNodeRequest) returns (RoadmapNode);
  rpc DeleteNode(DeleteNodeRequest) returns (DeleteNodeResponse);
  rpc AddEdge(AddEdgeRequest) returns (RoadmapEdge);
  rpc UpdateEdge(UpdateEdgeRequest) returns (RoadmapEdge);
  rpc DeleteEdge(DeleteEdgeRequest) returns (DeleteEdgeResponse);
}
```

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#map-service) 참조

### 3.2 포인트 클라우드 처리 파이프라인

Python 기반 비동기 처리 파이프라인. Rust 서비스에서 `subprocess`로 호출한다.

**파이프라인 단계:**

| 단계 | 처리 내용 | 도구/라이브러리 | 실패 시 |
|------|----------|---------------|--------|
| 1. ingestion | 청크 수신 → MinIO 임시 파일 조합 | aws-sdk-s3 (Rust) | job failed |
| 2. validation | 파일 포맷 검증, 포인트 수 확인 | laspy / plyfile | job failed |
| 3. normalization | 좌표 오프셋 적용 (원점 근처로 이동) | numpy | job failed |
| 4. downsampling | Voxel downsampling (0.02m grid) | Open3D | job failed |
| 5. normals | 법선 벡터 추정 (k=30 neighbors) | Open3D | 경고, 계속 |
| 6. potree_conversion | PotreeConverter 2.1 실행 → 옥트리 타일 | PotreeConverter CLI | job failed |
| 7. upload | 타일 파일 MinIO 업로드 (`tiles/{map_id}/`) | boto3 | job failed |
| 8. metadata | DB 메타데이터 업데이트 (point_count, bounds, lod_levels) | psycopg2 | job failed |

**처리 상태:**
```
pending → processing (progress 0-100%) → completed | failed
```

각 단계 완료 시 `processing_jobs.progress`와 `current_step`을 업데이트한다.

### 3.3 Potree 타일 서빙

- **GetTile RPC**: `(map_id, node_id, lod)` → MinIO에서 바이너리 옥트리 노드 반환
- **node_id 형식**: Potree 옥트리 경로 (`"r"`, `"r0"`, `"r01"`, ...)
- **응답**: `TileData { format: "potree_v2", data: bytes, point_count, bounds }`
- **Backend HTTP 프록시**: `GET /api/maps/:map_id/tiles/:node_id?lod=0`
  - Backend가 gRPC로 Map Manager 호출 후 바이너리를 HTTP 응답으로 전달

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#tile-serving) 참조

### 3.4 로드맵 그래프 데이터 모델

**Node:**
```
{id: UUID, map_id: UUID, name: string, node_type: enum, position: PointZ, properties: JSONB}
```

- `node_type`: `waypoint` | `charging_station` | `loading_dock` | `elevator` | `waiting_point` | `parking`
- `position`: PostGIS `geometry(PointZ, 4326)` — 3D 좌표

**Edge:**
```
{id: UUID, map_id: UUID, source_node_id: UUID, target_node_id: UUID,
 distance: float(m), max_speed: float(m/s), direction: uni|bi, properties: JSONB}
```

- `direction`: `uni` (단방향, source→target) | `bi` (양방향)
- 자기 자신으로의 엣지 방지 (`source_node_id != target_node_id` CHECK)

### 3.5 데이터베이스 스키마

**필요 테이블:**

| 테이블 | 설명 | 인덱스 |
|--------|------|--------|
| `maps` | 맵 메타데이터 (name, description, version, bounds, metadata JSONB) | `idx_maps_name` (GIN) |
| `pointcloud_data` | 포인트 클라우드 처리 결과 (map_id FK, point_count, tiles_path, status) | map_id, status |
| `processing_jobs` | 처리 작업 상태 추적 (job_type, status, progress, current_step) | map_id, status |
| `roadmap_nodes` | 경유점 노드 (position PointZ, node_type, properties) | map_id, GIST(position), (map_id, node_type) |
| `roadmap_edges` | 경로 엣지 (source, target, distance, max_speed, direction) | map_id, source, target |

PostGIS 확장 필수: `CREATE EXTENSION IF NOT EXISTS postgis;`

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#map-data) 참조

---

## 4. 구현 모듈

### 4.1 Map Manager (Team 5, Rust)

| 모듈 | 파일 경로 | 설명 |
|------|----------|------|
| gRPC 서버 | `src/grpc/server.rs` | Tonic gRPC 서버 부트스트랩 |
| Map 핸들러 | `src/grpc/map_handlers.rs` | CreateMap, GetMap, ListMaps, UpdateMap, DeleteMap |
| PointCloud 핸들러 | `src/grpc/pointcloud_handlers.rs` | UploadPointCloud, GetProcessingStatus, GetTile |
| Roadmap 핸들러 | `src/grpc/roadmap_handlers.rs` | AddNode, UpdateNode, DeleteNode, AddEdge, etc. |
| Map 리포지토리 | `src/db/map_repository.rs` | maps 테이블 CRUD (SQLx) |
| Roadmap 리포지토리 | `src/db/roadmap_repository.rs` | roadmap_nodes, roadmap_edges CRUD |
| S3 클라이언트 | `src/storage/s3_client.rs` | MinIO 업로드/다운로드 래퍼 |
| 처리 파이프라인 | `python/pipeline.py` | 포인트 클라우드 처리 전체 파이프라인 |
| 파이프라인 러너 | `src/pipeline/runner.rs` | Python subprocess 실행 및 진행률 파싱 |

### 4.2 Backend (Team 2, Rust)

| 모듈 | 파일 경로 | 설명 |
|------|----------|------|
| Map REST 프록시 | `src/api/routes/maps.rs` | `/api/maps/*` REST 엔드포인트 |
| Tile 프록시 | `src/api/routes/map_tiles.rs` | `/api/maps/:id/tiles/:node_id` 바이너리 프록시 |
| gRPC 클라이언트 | `src/grpc/map_client.rs` | MapService gRPC 클라이언트 래퍼 |

---

## 5. 의존성

### 5.1 인프라 의존성

| 구성 요소 | 용도 | 비고 |
|-----------|------|------|
| PostgreSQL 16 + PostGIS 3.4 | 메타데이터, 로드맵 그래프 저장 | `postgis` 확장 필수 |
| MinIO | 포인트 클라우드 원본 + Potree 타일 저장 | S3 호환 API |
| PotreeConverter 2.1 | LAS → Potree 옥트리 변환 | Docker 이미지에 포함 |
| Python 3.12+ | 처리 파이프라인 | Open3D, laspy, numpy, boto3 |

### 5.2 팀 간 의존성

| 의존 대상 | 방향 | 내용 |
|-----------|------|------|
| Team 6 (Proto) | 공유 | `.proto` 파일 정의 (`map_service.proto`) |
| Team 2 (Backend) | Backend → Map Manager | gRPC 클라이언트로 Map Manager 호출 |
| Team 1 (Frontend) | 소비자 | REST API + Potree 타일 소비 |

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#team5-interfaces) 참조

---

## 6. 테스트 기준

### 6.1 단위 테스트

| 대상 | 테스트 항목 | 기대 결과 |
|------|-----------|----------|
| Map CRUD | CreateMap → GetMap 왕복 | 생성된 맵 데이터 일치 |
| Map CRUD | ListMaps 페이지네이션 | page_size 준수, total_count 정확 |
| Map CRUD | DeleteMap 후 GetMap | NOT_FOUND 에러 |
| Roadmap | AddNode → GetRoadmapGraph | 노드 포함된 그래프 반환 |
| Roadmap | AddEdge (존재하지 않는 노드) | INVALID_ARGUMENT 에러 |
| Roadmap | 자기 자신으로의 엣지 | 거부 (no_self_loop) |
| PointCloud | 빈 청크 스트림 | INVALID_ARGUMENT 에러 |

### 6.2 통합 테스트

| 시나리오 | 설명 | 성공 기준 |
|---------|------|----------|
| 포인트 클라우드 E2E | LAS 파일 업로드 → 처리 완료 → GetTile 성공 | processing_status == "completed", 타일 바이너리 반환 |
| 로드맵 그래프 구축 | 5 노드 + 6 엣지 생성 → GetRoadmapGraph | 그래프 구조 일치 |
| Backend 프록시 | REST `/api/maps` CRUD → gRPC 전달 확인 | HTTP 200, 응답 JSON 일치 |
| 대용량 업로드 | 100MB LAS 파일 chunked 업로드 | OOM 없이 완료, progress 추적 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#pointcloud-samples) 참조

### 6.3 성능 기준

| 항목 | 목표 |
|------|------|
| Map CRUD 응답 | < 50ms (p95) |
| GetTile 응답 (MinIO 직접) | < 10ms |
| 로드맵 노드 조회 (1,000 노드) | < 100ms |
| 포인트 클라우드 처리 (10M points) | < 5분 |

---

## 7. 완료 조건

- [ ] gRPC MapService 전체 Phase 1 RPC 구현 및 테스트 통과
- [ ] 포인트 클라우드 업로드 (chunked stream) → MinIO 저장 동작
- [ ] Python 파이프라인: LAS → Potree 타일 변환 E2E 성공
- [ ] GetTile로 Potree 타일 바이너리 서빙 확인
- [ ] 로드맵 그래프 CRUD (노드/엣지) 전체 동작
- [ ] PostgreSQL + PostGIS 마이그레이션 적용 완료
- [ ] Backend REST 프록시 (`/api/maps/*`) 동작 확인
- [ ] GetProcessingStatus 진행률 폴링 동작
- [ ] Docker Compose로 Map Manager + PostgreSQL + MinIO 통합 테스트 통과
- [ ] 100MB LAS 파일 업로드 및 처리 E2E 성공
