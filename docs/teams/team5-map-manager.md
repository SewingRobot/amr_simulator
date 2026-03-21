# Team 5: Map Manager

## 1. 팀 개요

### 1.1 팀 명칭
**Map Manager Team** (팀 5)

### 1.2 담당 범위
Map Manager 팀은 AMR(Autonomous Mobile Robot) 통합 프레임워크에서 **모든 공간 데이터의 생명주기를 관리**한다. 구체적으로 다음 데이터 도메인을 소유한다:

- **포인트 클라우드(Point Cloud)**: LAS/LAZ/PLY/PCD 포맷의 3D 스캔 데이터를 수집·처리·Potree 옥트리 타일로 변환하여 저장·서빙
- **씬 그래프(Scene Graph)**: 맵 단위의 메타데이터 및 좌표계 관리
- **로드맵 그래프(Roadmap Graph)**: AMR이 주행하는 경유점(waypoint) 노드와 경로 엣지로 구성된 방향 그래프
- **시맨틱 객체/영역(Semantic Layer)**: 금지 구역, 속도 제한 구역, 충전 구역, 적재 구역 등 의미 기반 공간 영역
- **장애물 레이어(Obstacle Layer)**: 정적 장애물(PostgreSQL)과 동적 장애물(Redis TTL 기반) 관리
- **경로 탐색(Pathfinding)**: 로드맵 그래프 위에서 A*/Dijkstra 기반 최단 경로 계산 및 충돌 감지

### 1.3 핵심 목표

| 목표 | 설명 |
|------|------|
| 대용량 포인트 클라우드 처리 | 수억 포인트(500M+) 규모의 LAS 파일을 OOM 없이 스트리밍 업로드·처리·Potree 옥트리 변환 |
| 고성능 타일 서빙 | MinIO에서 10ms 이내, Redis 캐시 적중 시 1ms 이내로 Potree 타일 제공 |
| 로드맵 기반 경로 계획 | 10,000 노드 그래프에서 5ms 이내 A* 경로 탐색 |
| 시맨틱 공간 정보 관리 | PostGIS 공간 인덱스를 활용한 10ms 이내 point-in-polygon 쿼리 |
| 동적 장애물 실시간 반영 | Redis 기반 TTL 캐시로 로봇 감지 장애물 실시간 관리 |

### 1.4 의존 관계

```
┌─────────────────────────────────────────────────────────────┐
│                     Map Manager (Team 5)                     │
│                                                              │
│  gRPC Server (Tonic) ◄──── Backend (Team 2) gRPC Client     │
│  Potree Tiles (MinIO) ────► Frontend (Team 1) Potree Viewer │
│  Map Data Export ────────► Sim Engine (Team 3) 맵 로드       │
│                                                              │
│  PostgreSQL + PostGIS ◄──── 메타데이터, 로드맵, 시맨틱 저장   │
│  MinIO (S3 호환) ◄───────── 포인트 클라우드 원본 및 타일 저장  │
│  Redis ◄─────────────────── 동적 장애물 캐시, 타일 캐시       │
└─────────────────────────────────────────────────────────────┘
```

| 의존 대상 | 방향 | 프로토콜 | 설명 |
|-----------|------|----------|------|
| Backend (Team 2) | Backend → MapManager | gRPC | Backend가 MapService의 gRPC 클라이언트로 동작 |
| Frontend (Team 1) | MapManager → Frontend | HTTP/gRPC | Potree 타일 서빙, 로드맵/시맨틱 JSON 제공 |
| Sim Engine (Team 3) | MapManager → SimEngine | gRPC/File | 맵 데이터 export (SDF 또는 커스텀 포맷) |
| PostgreSQL + PostGIS | MapManager → DB | SQL (SQLx) | 메타데이터, 로드맵, 시맨틱, 장애물 영구 저장 |
| MinIO | MapManager → Storage | S3 API | 포인트 클라우드 원본·Potree 타일 오브젝트 스토리지 |
| Redis | MapManager → Cache | Redis Protocol | 동적 장애물, 타일 캐시 |
| Asset Manager (Team 6) | 양방향 | gRPC | 맵 내 에셋(로봇 모델 등) 참조 |

---

## 2. 기술 스택 상세

### 2.1 Rust (core service)

| 항목 | 선택 | 버전 | 용도 |
|------|------|------|------|
| Rust Edition | 2024 | 1.85+ | 메인 서비스 언어 |
| Tonic | 0.12+ | gRPC 서버/클라이언트 |
| Prost | 0.13+ | Protobuf 코드 생성 |
| SQLx | 0.8+ | PostgreSQL 비동기 드라이버 (PostGIS 지원) |
| aws-sdk-s3 | 1.x | MinIO S3 호환 오브젝트 스토리지 |
| fred | 9.x | Redis 비동기 클라이언트 |
| tokio | 1.x | 비동기 런타임 |
| serde / serde_json | 1.x | JSON 직렬화 |
| uuid | 1.x | UUID v4 생성 |
| tracing / tracing-subscriber | 0.1+ / 0.3+ | 구조화 로깅 |
| config | 0.14+ | 설정 파일 로드 |
| thiserror | 2.x | 에러 타입 정의 |
| petgraph | 0.7+ | 그래프 자료구조 (A* 참조 구현) |
| geo / geo-types | 0.28+ | Rust 측 기하 연산 보조 |

**Cargo.toml 예시:**

```toml
[package]
name = "map-manager"
version = "0.1.0"
edition = "2024"

[dependencies]
tokio = { version = "1", features = ["full"] }
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }
aws-sdk-s3 = "1"
aws-config = "1"
fred = { version = "9", features = ["tokio-runtime"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
config = "0.14"
thiserror = "2"
petgraph = "0.7"
geo = "0.28"
geo-types = "0.7"
chrono = { version = "0.4", features = ["serde"] }
bytes = "1"

[build-dependencies]
tonic-build = "0.12"
```

### 2.2 Python (point cloud processing)

| 항목 | 버전 | 용도 |
|------|------|------|
| Python | 3.12+ | 포인트 클라우드 처리 파이프라인 |
| Open3D | 0.18+ | 포인트 클라우드 읽기/쓰기, 다운샘플링, 필터링, 노멀 추정 |
| laspy | 2.x | LAS/LAZ 파일 읽기 (대용량 청크 처리) |
| numpy | 1.26+ | 수치 연산, 좌표 변환 |
| scipy | 1.12+ | 공간 알고리즘 (KDTree 등) |
| PotreeConverter | 2.1+ | Potree 옥트리 생성 (외부 바이너리) |
| boto3 | 1.34+ | MinIO S3 업/다운로드 |
| psycopg2-binary | 2.9+ | PostgreSQL 메타데이터 기록 |
| pydantic | 2.x | 데이터 검증 |

**requirements.txt:**

```
open3d>=0.18.0
laspy[lazrs]>=2.5.0
numpy>=1.26.0
scipy>=1.12.0
boto3>=1.34.0
psycopg2-binary>=2.9.0
pydantic>=2.0
tqdm>=4.66.0
```

### 2.3 인프라

| 항목 | 용도 |
|------|------|
| PostgreSQL 16 + PostGIS 3.4 | 메타데이터, 로드맵, 시맨틱, 장애물 영구 저장 |
| MinIO (S3 호환) | 포인트 클라우드 원본, Potree 타일 오브젝트 스토리지 |
| Redis 7.2+ | 동적 장애물 캐시, Potree 타일 캐시 |
| Docker + Docker Compose | 로컬 개발 및 배포 |

---

## 3. 디렉토리 구조 상세

```
map-manager/
├── Cargo.toml                   # Rust 프로젝트 설정
├── build.rs                     # tonic-build protobuf 컴파일
├── Dockerfile                   # 멀티스테이지 빌드 (Rust + Python)
├── docker-compose.dev.yml       # 로컬 개발용 (PostgreSQL, MinIO, Redis)
├── config/
│   ├── default.toml             # 기본 설정
│   ├── development.toml         # 개발 환경 오버라이드
│   └── production.toml          # 프로덕션 환경
├── proto/
│   └── map_service.proto        # gRPC 서비스 정의
├── src/
│   ├── main.rs                  # 서버 엔트리포인트, gRPC 서버 시작
│   ├── config.rs                # 설정 구조체 및 로딩
│   ├── error.rs                 # 통합 에러 타입 (thiserror)
│   ├── grpc/
│   │   ├── mod.rs               # gRPC 모듈 re-export
│   │   ├── server.rs            # Tonic gRPC 서버 셋업
│   │   ├── map_handlers.rs      # Map CRUD RPC 핸들러
│   │   ├── pointcloud_handlers.rs  # 포인트 클라우드 업로드/타일 서빙 핸들러
│   │   ├── roadmap_handlers.rs  # 로드맵 그래프 CRUD 핸들러
│   │   ├── semantic_handlers.rs # 시맨틱 레이어 핸들러
│   │   ├── obstacle_handlers.rs # 장애물 레이어 핸들러
│   │   └── pathfinding_handlers.rs # 경로 탐색 핸들러
│   ├── storage/
│   │   ├── mod.rs               # 스토리지 모듈 re-export
│   │   ├── postgres.rs          # PostgreSQL 연결 풀 관리
│   │   ├── s3_tiles.rs          # MinIO S3 타일 업로드/다운로드
│   │   ├── metadata.rs          # maps, pointcloud_data 테이블 CRUD
│   │   ├── roadmap_repo.rs      # roadmap_nodes, roadmap_edges 테이블 CRUD
│   │   ├── semantic_repo.rs     # semantic_regions 테이블 CRUD
│   │   ├── obstacle_repo.rs     # obstacles 테이블 CRUD
│   │   └── cache.rs             # Redis 캐시 (동적 장애물, 타일)
│   ├── models/
│   │   ├── mod.rs               # 모델 re-export
│   │   ├── map.rs               # Map 도메인 모델
│   │   ├── roadmap.rs           # RoadmapNode, RoadmapEdge
│   │   ├── semantic.rs          # SemanticRegion
│   │   ├── obstacle.rs          # Obstacle, DynamicObstacle
│   │   └── pointcloud.rs        # PointCloudData, ProcessingJob
│   ├── pathfinding/
│   │   ├── mod.rs               # 경로 탐색 모듈 re-export
│   │   ├── graph.rs             # 인메모리 그래프 자료구조 (인접 리스트)
│   │   ├── astar.rs             # A* 알고리즘 구현
│   │   ├── dijkstra.rs          # Dijkstra 알고리즘 (대안)
│   │   ├── heuristics.rs        # 거리 휴리스틱 (유클리드, 맨해튼)
│   │   └── conflict.rs          # 경로 충돌 감지
│   └── processing/
│       ├── mod.rs               # 처리 모듈 re-export
│       └── pipeline.rs          # Python 파이프라인 트리거 (subprocess)
├── processing/                  # Python 포인트 클라우드 처리 파이프라인
│   ├── __init__.py
│   ├── pipeline.py              # 메인 처리 오케스트레이터
│   ├── ingest.py                # LAS/PLY/PCD → Open3D PointCloud 변환
│   ├── preprocess.py            # 다운샘플링, 필터링
│   ├── normals.py               # 노멀 벡터 추정
│   ├── potree_convert.py        # PotreeConverter 실행 래퍼
│   ├── metadata_extract.py      # 바운딩 박스, 밀도, 통계 추출
│   ├── coordinate_system.py     # 좌표계 정규화 (ENU 변환)
│   ├── s3_upload.py             # Potree 타일 MinIO 업로드
│   ├── config.py                # Python 파이프라인 설정
│   └── requirements.txt
├── migrations/
│   ├── 001_enable_postgis.sql
│   ├── 002_create_maps.sql
│   ├── 003_create_pointcloud_data.sql
│   ├── 004_create_roadmap.sql
│   ├── 005_create_semantic.sql
│   ├── 006_create_obstacles.sql
│   └── 007_create_processing_jobs.sql
├── tests/
│   ├── rust/
│   │   ├── test_pathfinding.rs  # A*/Dijkstra 정확성 테스트
│   │   ├── test_roadmap.rs      # 로드맵 그래프 CRUD 테스트
│   │   ├── test_grpc.rs         # gRPC 핸들러 통합 테스트
│   │   ├── test_spatial.rs      # PostGIS 공간 쿼리 테스트
│   │   └── common/
│   │       └── mod.rs           # 테스트 유틸리티, 픽스처
│   └── python/
│       ├── test_pipeline.py     # 파이프라인 E2E 테스트
│       ├── test_ingest.py       # 포맷별 인제스트 테스트
│       ├── test_preprocess.py   # 다운샘플링/필터링 테스트
│       ├── test_potree.py       # PotreeConverter 출력 검증
│       ├── test_coordinates.py  # 좌표 변환 테스트
│       └── fixtures/
│           ├── small_room.ply   # 10K 포인트 테스트 데이터
│           ├── sample.las       # 작은 LAS 테스트 파일
│           └── sample.pcd       # PCD 포맷 테스트 파일
└── scripts/
    ├── init_minio_buckets.sh    # MinIO 버킷 초기 생성
    └── seed_test_data.py        # 테스트 데이터 시딩
```

---

## 4. 구성 모듈 상세 스펙

### 4.1 gRPC Service Interface

#### 4.1.1 Protobuf 서비스 정의

아래는 `proto/map_service.proto`의 **완전한** 정의이다. 모든 RPC와 메시지 타입을 포함한다.

```protobuf
syntax = "proto3";

package mapservice;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

// ─────────────────────────────────────────
// 공통 타입
// ─────────────────────────────────────────

message Point3D {
  double x = 1;
  double y = 2;
  double z = 3;
}

message BoundingBox3D {
  Point3D min = 1;
  Point3D max = 2;
}

message Polygon3D {
  repeated Point3D vertices = 1;  // 닫힌 폴리곤 (첫 점 = 마지막 점)
}

// ─────────────────────────────────────────
// Map CRUD 메시지
// ─────────────────────────────────────────

message MapDescriptor {
  string id = 1;                             // UUID
  string name = 2;
  string description = 3;
  int32 version = 4;
  Point3D coordinate_offset = 5;             // 처리 시 적용된 좌표 오프셋
  BoundingBox3D bounds = 6;
  google.protobuf.Struct metadata = 7;       // 임의 메타데이터
  google.protobuf.Timestamp created_at = 8;
  google.protobuf.Timestamp updated_at = 9;
}

message CreateMapRequest {
  string name = 1;
  string description = 2;
  google.protobuf.Struct metadata = 3;
}

message GetMapRequest {
  string map_id = 1;
}

message ListMapsRequest {
  int32 page = 1;          // 0-based
  int32 page_size = 2;     // default 20, max 100
  string name_filter = 3;  // 이름 부분 일치 필터 (optional)
}

message ListMapsResponse {
  repeated MapDescriptor maps = 1;
  int32 total_count = 2;
  int32 page = 3;
  int32 page_size = 4;
}

message UpdateMapRequest {
  string map_id = 1;
  string name = 2;
  string description = 3;
  google.protobuf.Struct metadata = 4;
}

message DeleteMapRequest {
  string map_id = 1;
}

message DeleteMapResponse {
  bool success = 1;
  string message = 2;
}

// ─────────────────────────────────────────
// Point Cloud 메시지
// ─────────────────────────────────────────

message PointCloudChunk {
  string map_id = 1;           // 첫 청크에만 필수, 이후 생략 가능
  string filename = 2;         // 원본 파일명 (첫 청크에만)
  string format = 3;           // "las", "laz", "ply", "pcd" (첫 청크에만)
  bytes data = 4;              // 청크 데이터 (최대 4MB per chunk)
  int64 total_size = 5;        // 전체 파일 크기 (첫 청크에만, 검증용)
  int32 chunk_index = 6;       // 0-based 청크 인덱스
  bool is_last = 7;            // 마지막 청크 여부
}

message ProcessingJob {
  string job_id = 1;
  string map_id = 2;
  string job_type = 3;         // "pointcloud_processing"
  string status = 4;           // "pending", "processing", "completed", "failed"
  float progress = 5;          // 0.0 ~ 100.0
  string error_message = 6;    // 실패 시 에러 메시지
  google.protobuf.Timestamp created_at = 7;
  google.protobuf.Timestamp completed_at = 8;
}

message GetProcessingStatusRequest {
  string job_id = 1;
}

message ProcessingStatus {
  string job_id = 1;
  string status = 2;
  float progress = 3;
  string current_step = 4;     // "ingestion", "validation", "normalization", "downsampling", "filtering", "normals", "potree_conversion", "upload", "metadata"
  string error_message = 5;
  google.protobuf.Timestamp updated_at = 6;
}

message GetPointCloudMetadataRequest {
  string map_id = 1;
}

message PointCloudMetadata {
  string id = 1;
  string map_id = 2;
  int64 point_count = 3;
  string format = 4;
  int32 lod_levels = 5;
  float density_per_sqm = 6;
  BoundingBox3D bounds = 7;
  Point3D coordinate_offset = 8;
  string tiles_path = 9;        // MinIO 경로 프리픽스
  string processing_status = 10;
  google.protobuf.Timestamp created_at = 11;
}

message GetTileRequest {
  string map_id = 1;
  string node_id = 2;           // Potree 옥트리 노드 ID (e.g., "r", "r0", "r01")
  int32 lod = 3;                // LOD 레벨 (0 = 최고 해상도)
}

message TileData {
  string node_id = 1;
  bytes data = 2;               // 바이너리 타일 데이터
  int32 point_count = 3;        // 이 타일의 포인트 수
  BoundingBox3D bounds = 4;     // 이 타일의 바운딩 박스
}

message StreamTilesRequest {
  string map_id = 1;
  BoundingBox3D frustum = 2;    // 카메라 프러스텀 (AABB 근사)
  int32 max_lod = 3;            // 요청할 최대 LOD 레벨
  float screen_size = 4;        // 화면 크기 (LOD 판단용)
  repeated string loaded_nodes = 5;  // 이미 로드된 노드 (중복 방지)
}

// ─────────────────────────────────────────
// Roadmap Graph 메시지
// ─────────────────────────────────────────

enum NodeType {
  NODE_TYPE_UNSPECIFIED = 0;
  NODE_TYPE_WAYPOINT = 1;
  NODE_TYPE_CHARGING_STATION = 2;
  NODE_TYPE_LOADING_DOCK = 3;
  NODE_TYPE_ELEVATOR = 4;
  NODE_TYPE_WAITING_POINT = 5;
  NODE_TYPE_PARKING = 6;
}

enum EdgeDirection {
  EDGE_DIRECTION_UNSPECIFIED = 0;
  EDGE_DIRECTION_UNIDIRECTIONAL = 1;
  EDGE_DIRECTION_BIDIRECTIONAL = 2;
}

message RoadmapNode {
  string id = 1;
  string map_id = 2;
  string name = 3;
  NodeType node_type = 4;
  Point3D position = 5;
  google.protobuf.Struct properties = 6;
  google.protobuf.Timestamp created_at = 7;
}

message RoadmapEdge {
  string id = 1;
  string map_id = 2;
  string source_node_id = 3;
  string target_node_id = 4;
  float distance = 5;             // 미터 단위
  float max_speed = 6;            // m/s
  EdgeDirection direction = 7;
  repeated string allowed_robot_types = 8;
  float cost_factor = 9;          // 1.0 = 기본, >1.0 = 더 비싼 경로
  repeated Point3D path_points = 10; // 경로 시각화용 중간 점
  google.protobuf.Struct properties = 11;
  google.protobuf.Timestamp created_at = 12;
}

message RoadmapGraph {
  string map_id = 1;
  repeated RoadmapNode nodes = 2;
  repeated RoadmapEdge edges = 3;
  int32 version = 4;
}

message GetRoadmapRequest {
  string map_id = 1;
}

message UpdateRoadmapRequest {
  string map_id = 1;
  RoadmapGraph graph = 2;       // 전체 그래프 교체
}

message AddNodeRequest {
  string map_id = 1;
  string name = 2;
  NodeType node_type = 3;
  Point3D position = 4;
  google.protobuf.Struct properties = 5;
}

message UpdateNodeRequest {
  string node_id = 1;
  string name = 2;
  NodeType node_type = 3;
  Point3D position = 4;
  google.protobuf.Struct properties = 5;
}

message DeleteNodeRequest {
  string node_id = 1;
}

message DeleteNodeResponse {
  bool success = 1;
  string message = 2;
  int32 deleted_edges_count = 3; // 연쇄 삭제된 엣지 수
}

message AddEdgeRequest {
  string map_id = 1;
  string source_node_id = 2;
  string target_node_id = 3;
  float max_speed = 4;
  EdgeDirection direction = 5;
  repeated string allowed_robot_types = 6;
  float cost_factor = 7;
  repeated Point3D path_points = 8;
  google.protobuf.Struct properties = 9;
}

message UpdateEdgeRequest {
  string edge_id = 1;
  float max_speed = 2;
  EdgeDirection direction = 3;
  repeated string allowed_robot_types = 4;
  float cost_factor = 5;
  repeated Point3D path_points = 6;
  google.protobuf.Struct properties = 7;
}

message DeleteEdgeRequest {
  string edge_id = 1;
}

message DeleteEdgeResponse {
  bool success = 1;
  string message = 2;
}

// ─────────────────────────────────────────
// Semantic Layer 메시지
// ─────────────────────────────────────────

enum RegionType {
  REGION_TYPE_UNSPECIFIED = 0;
  REGION_TYPE_NO_GO_ZONE = 1;
  REGION_TYPE_SPEED_LIMIT_ZONE = 2;
  REGION_TYPE_CHARGING_AREA = 3;
  REGION_TYPE_LOADING_DOCK = 4;
  REGION_TYPE_WAITING_AREA = 5;
  REGION_TYPE_ONE_WAY_ZONE = 6;
  REGION_TYPE_RESTRICTED_AREA = 7;
}

message SemanticRegion {
  string id = 1;
  string map_id = 2;
  string name = 3;
  RegionType region_type = 4;
  Polygon3D geometry = 5;
  google.protobuf.Struct properties = 6;  // region_type별 추가 속성
  google.protobuf.Timestamp created_at = 7;
}

message SemanticLayer {
  string map_id = 1;
  repeated SemanticRegion regions = 2;
}

message GetSemanticRequest {
  string map_id = 1;
  RegionType type_filter = 2;   // 0이면 전체
}

message UpdateSemanticRequest {
  string map_id = 1;
  SemanticLayer layer = 2;      // 전체 교체
}

message AddRegionRequest {
  string map_id = 1;
  string name = 2;
  RegionType region_type = 3;
  Polygon3D geometry = 4;
  google.protobuf.Struct properties = 5;
}

message UpdateRegionRequest {
  string region_id = 1;
  string name = 2;
  RegionType region_type = 3;
  Polygon3D geometry = 4;
  google.protobuf.Struct properties = 5;
}

message DeleteRegionRequest {
  string region_id = 1;
}

message DeleteRegionResponse {
  bool success = 1;
  string message = 2;
}

// ─────────────────────────────────────────
// Obstacle 메시지
// ─────────────────────────────────────────

enum ObstacleType {
  OBSTACLE_TYPE_UNSPECIFIED = 0;
  OBSTACLE_TYPE_STATIC = 1;       // 영구 장애물 (PostgreSQL)
  OBSTACLE_TYPE_TEMPORARY = 2;    // 임시 장애물 (PostgreSQL, 수동 관리)
  OBSTACLE_TYPE_DYNAMIC = 3;      // 동적 장애물 (Redis, 자동 TTL 만료)
}

message Obstacle {
  string id = 1;
  string map_id = 2;
  ObstacleType obstacle_type = 3;
  Polygon3D footprint = 4;        // 2D 바닥 면적 (PolygonZ의 z는 바닥 높이)
  Point3D position = 5;           // 중심 위치
  float length = 6;               // X축 크기 (m)
  float width = 7;                // Y축 크기 (m)
  float height = 8;               // Z축 크기 (m)
  google.protobuf.Struct properties = 9;
  google.protobuf.Timestamp created_at = 10;
}

message ObstacleList {
  repeated Obstacle obstacles = 1;
}

message GetObstaclesRequest {
  string map_id = 1;
  BoundingBox3D query_bounds = 2; // 이 영역 내 장애물 조회 (optional)
  ObstacleType type_filter = 3;   // 0이면 전체
  bool include_dynamic = 4;       // Redis 동적 장애물 포함 여부
}

message AddObstacleRequest {
  string map_id = 1;
  ObstacleType obstacle_type = 2;
  Polygon3D footprint = 3;
  Point3D position = 4;
  float length = 5;
  float width = 6;
  float height = 7;
  google.protobuf.Struct properties = 8;
}

message UpdateObstacleRequest {
  string obstacle_id = 1;
  Polygon3D footprint = 2;
  Point3D position = 3;
  float length = 4;
  float width = 5;
  float height = 6;
  google.protobuf.Struct properties = 7;
}

message DeleteObstacleRequest {
  string obstacle_id = 1;
}

message DeleteObstacleResponse {
  bool success = 1;
  string message = 2;
}

message DynamicObstacle {
  string obstacle_id = 1;        // 클라이언트가 생성한 고유 ID
  string map_id = 2;
  Point3D position = 3;
  float length = 4;
  float width = 5;
  float height = 6;
  float heading = 7;             // 라디안 (0 = East)
  int32 ttl_seconds = 8;        // Redis TTL (기본 10초)
  string source_robot_id = 9;   // 감지한 로봇 ID
  google.protobuf.Timestamp detected_at = 10;
}

message DynamicObstacleResponse {
  int32 accepted_count = 1;
  int32 rejected_count = 2;
  repeated string rejected_ids = 3;
}

// ─────────────────────────────────────────
// Pathfinding 메시지
// ─────────────────────────────────────────

message FindPathRequest {
  string map_id = 1;
  string start_node_id = 2;      // 출발 노드 UUID
  string goal_node_id = 3;       // 도착 노드 UUID
  string robot_type = 4;         // 로봇 유형 (엣지 필터링)
  bool avoid_no_go_zones = 5;    // 금지 구역 회피 (default true)
  bool consider_dynamic_obstacles = 6; // 동적 장애물 고려
  float max_cost = 7;            // 최대 비용 제한 (0이면 무제한)
}

message PathResult {
  bool found = 1;
  repeated string node_ids = 2;       // 경로 노드 ID 순서 목록
  repeated Point3D path_points = 3;   // 시각화용 경로 좌표
  float total_distance = 4;          // 총 거리 (m)
  float total_cost = 5;              // 총 비용 (cost_factor 적용)
  float estimated_time_seconds = 6;  // 예상 소요 시간 (초)
  repeated PathSegment segments = 7; // 구간별 상세
}

message PathSegment {
  string from_node_id = 1;
  string to_node_id = 2;
  string edge_id = 3;
  float distance = 4;
  float max_speed = 5;
  float estimated_time = 6;
}

message FindNearestNodeRequest {
  string map_id = 1;
  Point3D position = 2;          // 검색 기준 위치
  NodeType type_filter = 3;     // 특정 타입만 (0이면 전체)
  int32 max_results = 4;        // 최대 결과 수 (default 1)
  float max_distance = 5;       // 최대 검색 거리 (m, 0이면 무제한)
}

message CheckPathConflictRequest {
  string map_id = 1;
  repeated string path_node_ids = 2;    // 확인할 경로의 노드 ID 목록
  string robot_id = 3;                  // 요청한 로봇 ID (자기 자신 제외)
  google.protobuf.Timestamp start_time = 4;
  google.protobuf.Timestamp end_time = 5;
}

message ConflictResult {
  bool has_conflict = 1;
  repeated PathConflict conflicts = 2;
}

message PathConflict {
  string conflicting_robot_id = 1;
  string node_id = 2;                   // 충돌 지점 노드
  google.protobuf.Timestamp conflict_time = 3;
  string conflict_type = 4;             // "node_overlap", "edge_crossing", "head_on"
}

// ─────────────────────────────────────────
// 서비스 정의
// ─────────────────────────────────────────

service MapService {
  // ── Map CRUD ──
  rpc CreateMap(CreateMapRequest) returns (MapDescriptor);
  rpc GetMap(GetMapRequest) returns (MapDescriptor);
  rpc ListMaps(ListMapsRequest) returns (ListMapsResponse);
  rpc UpdateMap(UpdateMapRequest) returns (MapDescriptor);
  rpc DeleteMap(DeleteMapRequest) returns (DeleteMapResponse);

  // ── Point Cloud ──
  rpc UploadPointCloud(stream PointCloudChunk) returns (ProcessingJob);
  rpc GetProcessingStatus(GetProcessingStatusRequest) returns (ProcessingStatus);
  rpc GetPointCloudMetadata(GetPointCloudMetadataRequest) returns (PointCloudMetadata);
  rpc GetTile(GetTileRequest) returns (TileData);
  rpc StreamTiles(StreamTilesRequest) returns (stream TileData);

  // ── Roadmap Graph ──
  rpc GetRoadmapGraph(GetRoadmapRequest) returns (RoadmapGraph);
  rpc UpdateRoadmapGraph(UpdateRoadmapRequest) returns (RoadmapGraph);
  rpc AddNode(AddNodeRequest) returns (RoadmapNode);
  rpc UpdateNode(UpdateNodeRequest) returns (RoadmapNode);
  rpc DeleteNode(DeleteNodeRequest) returns (DeleteNodeResponse);
  rpc AddEdge(AddEdgeRequest) returns (RoadmapEdge);
  rpc UpdateEdge(UpdateEdgeRequest) returns (RoadmapEdge);
  rpc DeleteEdge(DeleteEdgeRequest) returns (DeleteEdgeResponse);

  // ── Semantic Layer ──
  rpc GetSemanticLayer(GetSemanticRequest) returns (SemanticLayer);
  rpc UpdateSemanticLayer(UpdateSemanticRequest) returns (SemanticLayer);
  rpc AddRegion(AddRegionRequest) returns (SemanticRegion);
  rpc UpdateRegion(UpdateRegionRequest) returns (SemanticRegion);
  rpc DeleteRegion(DeleteRegionRequest) returns (DeleteRegionResponse);

  // ── Obstacles ──
  rpc GetObstacles(GetObstaclesRequest) returns (ObstacleList);
  rpc AddObstacle(AddObstacleRequest) returns (Obstacle);
  rpc UpdateObstacle(UpdateObstacleRequest) returns (Obstacle);
  rpc DeleteObstacle(DeleteObstacleRequest) returns (DeleteObstacleResponse);
  rpc UpdateDynamicObstacles(stream DynamicObstacle) returns (DynamicObstacleResponse);

  // ── Pathfinding ──
  rpc FindPath(FindPathRequest) returns (PathResult);
  rpc FindNearestNode(FindNearestNodeRequest) returns (RoadmapNode);
  rpc CheckPathConflict(CheckPathConflictRequest) returns (ConflictResult);
}
```

#### 4.1.2 build.rs (protobuf 컴파일)

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true) // 통합 테스트용
        .out_dir("src/generated")
        .compile_protos(&["proto/map_service.proto"], &["proto/"])?;
    Ok(())
}
```

### 4.2 Rust 도메인 모델 상세

#### 4.2.1 설정 (`src/config.rs`)

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub minio: MinioConfig,
    pub redis: RedisConfig,
    pub processing: ProcessingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,          // "0.0.0.0"
    pub port: u16,             // 50053
    pub max_message_size: usize, // 16MB (gRPC 메시지 크기 제한)
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,           // "postgresql://user:pass@localhost:5432/amr_maps"
    pub max_connections: u32,  // 20
    pub min_connections: u32,  // 5
}

#[derive(Debug, Deserialize, Clone)]
pub struct MinioConfig {
    pub endpoint: String,      // "http://localhost:9000"
    pub access_key: String,
    pub secret_key: String,
    pub region: String,        // "us-east-1" (MinIO default)
    pub raw_bucket: String,    // "pointcloud-raw"
    pub tiles_bucket: String,  // "pointcloud-tiles"
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,           // "redis://localhost:6379"
    pub tile_cache_ttl_secs: u64,    // 3600 (1시간)
    pub dynamic_obstacle_default_ttl_secs: u64, // 10
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessingConfig {
    pub python_executable: String,   // "python3"
    pub pipeline_script: String,     // "./processing/pipeline.py"
    pub potree_converter_path: String, // "/usr/local/bin/PotreeConverter"
    pub temp_dir: String,            // "/tmp/map-manager"
    pub default_voxel_size: f64,     // 0.02
    pub outlier_neighbors: usize,    // 20
    pub outlier_std_ratio: f64,      // 2.0
    pub normal_search_radius: f64,   // 0.1
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name(&format!("config/{}", env)).required(false))
            .add_source(config::Environment::with_prefix("MAP_MANAGER").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}
```

#### 4.2.2 에러 타입 (`src/error.rs`)

```rust
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum MapManagerError {
    #[error("Map not found: {0}")]
    MapNotFound(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Edge not found: {0}")]
    EdgeNotFound(String),

    #[error("Region not found: {0}")]
    RegionNotFound(String),

    #[error("Obstacle not found: {0}")]
    ObstacleNotFound(String),

    #[error("Processing job not found: {0}")]
    JobNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    #[error("Tile not found: map={map_id}, node={node_id}")]
    TileNotFound { map_id: String, node_id: String },

    #[error("Path not found from {from} to {to}")]
    PathNotFound { from: String, to: String },

    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("S3 error: {0}")]
    S3(String),

    #[error("Redis error: {0}")]
    Redis(#[from] fred::error::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<MapManagerError> for Status {
    fn from(err: MapManagerError) -> Status {
        match &err {
            MapManagerError::MapNotFound(_)
            | MapManagerError::NodeNotFound(_)
            | MapManagerError::EdgeNotFound(_)
            | MapManagerError::RegionNotFound(_)
            | MapManagerError::ObstacleNotFound(_)
            | MapManagerError::JobNotFound(_)
            | MapManagerError::TileNotFound { .. } => {
                Status::not_found(err.to_string())
            }
            MapManagerError::InvalidInput(_)
            | MapManagerError::InvalidGeometry(_) => {
                Status::invalid_argument(err.to_string())
            }
            MapManagerError::PathNotFound { .. } => {
                Status::not_found(err.to_string())
            }
            _ => Status::internal(err.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, MapManagerError>;
```

#### 4.2.3 Map 도메인 모델 (`src/models/map.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Map {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub coordinate_offset: Vec<f64>,  // [x, y, z]
    pub bounds_min: Vec<f64>,         // [x, y, z]
    pub bounds_max: Vec<f64>,         // [x, y, z]
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMap {
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMap {
    pub name: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMapsFilter {
    pub page: i32,
    pub page_size: i32,
    pub name_filter: Option<String>,
}
```

#### 4.2.4 Roadmap 모델 (`src/models/roadmap.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Waypoint,
    ChargingStation,
    LoadingDock,
    Elevator,
    WaitingPoint,
    Parking,
}

impl NodeType {
    pub fn as_str(&self) -> &str {
        match self {
            NodeType::Waypoint => "waypoint",
            NodeType::ChargingStation => "charging_station",
            NodeType::LoadingDock => "loading_dock",
            NodeType::Elevator => "elevator",
            NodeType::WaitingPoint => "waiting_point",
            NodeType::Parking => "parking",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "waypoint" => Some(NodeType::Waypoint),
            "charging_station" => Some(NodeType::ChargingStation),
            "loading_dock" => Some(NodeType::LoadingDock),
            "elevator" => Some(NodeType::Elevator),
            "waiting_point" => Some(NodeType::WaitingPoint),
            "parking" => Some(NodeType::Parking),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeDirection {
    Unidirectional,
    Bidirectional,
}

/// 3D 위치 (PostGIS PointZ로 저장)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapNode {
    pub id: Uuid,
    pub map_id: Uuid,
    pub name: Option<String>,
    pub node_type: NodeType,
    pub position: Position3D,
    pub properties: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapEdge {
    pub id: Uuid,
    pub map_id: Uuid,
    pub source_node_id: Uuid,
    pub target_node_id: Uuid,
    pub distance: f32,
    pub max_speed: f32,
    pub direction: EdgeDirection,
    pub allowed_robot_types: Vec<String>,
    pub cost_factor: f32,
    pub path_points: Vec<Position3D>,
    pub properties: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// 인메모리 그래프 구축용 구조체
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: Uuid,
    pub position: Position3D,
    pub node_type: NodeType,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub id: Uuid,
    pub source: Uuid,
    pub target: Uuid,
    pub distance: f32,
    pub max_speed: f32,
    pub direction: EdgeDirection,
    pub allowed_robot_types: Vec<String>,
    pub cost_factor: f32,
}
```

#### 4.2.5 Semantic 모델 (`src/models/semantic.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RegionType {
    NoGoZone,
    SpeedLimitZone,
    ChargingArea,
    LoadingDock,
    WaitingArea,
    OneWayZone,
    RestrictedArea,
}

impl RegionType {
    pub fn as_str(&self) -> &str {
        match self {
            RegionType::NoGoZone => "no_go_zone",
            RegionType::SpeedLimitZone => "speed_limit_zone",
            RegionType::ChargingArea => "charging_area",
            RegionType::LoadingDock => "loading_dock",
            RegionType::WaitingArea => "waiting_area",
            RegionType::OneWayZone => "one_way_zone",
            RegionType::RestrictedArea => "restricted_area",
        }
    }
}

/// Region type별 properties 스키마:
///
/// - speed_limit_zone: { "max_speed_mps": 0.5 }
/// - charging_area:    { "charger_count": 4, "charger_type": "wireless" }
/// - loading_dock:     { "dock_type": "pallet", "capacity": 2 }
/// - one_way_zone:     { "allowed_direction_deg": 90.0 }
/// - no_go_zone:       {} (추가 속성 없음)
/// - waiting_area:     { "max_robots": 3 }
/// - restricted_area:  { "allowed_robot_types": ["agv_large"], "schedule": "08:00-18:00" }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRegion {
    pub id: Uuid,
    pub map_id: Uuid,
    pub name: Option<String>,
    pub region_type: RegionType,
    pub geometry: Vec<[f64; 3]>,  // 폴리곤 꼭짓점 [(x,y,z), ...]
    pub properties: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
```

#### 4.2.6 Obstacle 모델 (`src/models/obstacle.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::roadmap::Position3D;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObstacleType {
    Static,
    Temporary,
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obstacle {
    pub id: Uuid,
    pub map_id: Uuid,
    pub obstacle_type: ObstacleType,
    pub footprint: Vec<[f64; 3]>,   // 2D 폴리곤 (z = 바닥 높이)
    pub position: Position3D,
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub properties: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Redis에 저장되는 동적 장애물
/// Key: "dynamic_obstacle:{map_id}:{obstacle_id}"
/// TTL: ttl_seconds (기본 10초)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicObstacle {
    pub obstacle_id: String,
    pub map_id: Uuid,
    pub position: Position3D,
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub heading: f32,              // 라디안
    pub source_robot_id: String,
    pub detected_at: DateTime<Utc>,
}
```

#### 4.2.7 PointCloud 모델 (`src/models/pointcloud.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCloudData {
    pub id: Uuid,
    pub map_id: Uuid,
    pub raw_file_path: Option<String>,
    pub tiles_path: Option<String>,
    pub point_count: Option<i64>,
    pub format: Option<String>,
    pub lod_levels: Option<i32>,
    pub density_per_sqm: Option<f32>,
    pub processing_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingJob {
    pub id: Uuid,
    pub map_id: Uuid,
    pub job_type: String,
    pub status: ProcessingStatus,
    pub progress: f32,
    pub current_step: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl ProcessingStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ProcessingStatus::Pending => "pending",
            ProcessingStatus::Processing => "processing",
            ProcessingStatus::Completed => "completed",
            ProcessingStatus::Failed => "failed",
        }
    }
}
```

### 4.3 스토리지 레이어 상세

#### 4.3.1 PostgreSQL 연결 관리 (`src/storage/postgres.rs`)

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::DatabaseConfig;
use crate::error::Result;

pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect(&config.url)
        .await?;

    // 마이그레이션 실행
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

#### 4.3.2 Map 메타데이터 저장소 (`src/storage/metadata.rs`)

```rust
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{MapManagerError, Result};
use crate::models::map::{CreateMap, ListMapsFilter, Map, UpdateMap};

pub struct MapRepository {
    pool: PgPool,
}

impl MapRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: CreateMap) -> Result<Map> {
        let map = sqlx::query_as::<_, Map>(
            r#"
            INSERT INTO maps (name, description, metadata)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await?;

        Ok(map)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Map> {
        sqlx::query_as::<_, Map>("SELECT * FROM maps WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(MapManagerError::MapNotFound(id.to_string()))
    }

    pub async fn list(&self, filter: ListMapsFilter) -> Result<(Vec<Map>, i32)> {
        let offset = filter.page * filter.page_size;

        let maps = if let Some(ref name_filter) = filter.name_filter {
            sqlx::query_as::<_, Map>(
                "SELECT * FROM maps WHERE name ILIKE $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
            )
            .bind(format!("%{}%", name_filter))
            .bind(filter.page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Map>(
                "SELECT * FROM maps ORDER BY created_at DESC LIMIT $1 OFFSET $2"
            )
            .bind(filter.page_size as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await?
        };

        let total: (i64,) = if let Some(ref name_filter) = filter.name_filter {
            sqlx::query_as("SELECT COUNT(*) FROM maps WHERE name ILIKE $1")
                .bind(format!("%{}%", name_filter))
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM maps")
                .fetch_one(&self.pool)
                .await?
        };

        Ok((maps, total.0 as i32))
    }

    pub async fn update(&self, id: Uuid, input: UpdateMap) -> Result<Map> {
        let existing = self.get_by_id(id).await?;

        let map = sqlx::query_as::<_, Map>(
            r#"
            UPDATE maps
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                metadata = COALESCE($4, metadata),
                version = version + 1,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(input.name.as_deref().or(Some(&existing.name)))
        .bind(input.description.as_deref().or(existing.description.as_deref()))
        .bind(input.metadata.as_ref().unwrap_or(&existing.metadata))
        .fetch_one(&self.pool)
        .await?;

        Ok(map)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM maps WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
```

#### 4.3.3 Roadmap 저장소 (`src/storage/roadmap_repo.rs`)

```rust
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{MapManagerError, Result};
use crate::models::roadmap::*;

pub struct RoadmapRepository {
    pool: PgPool,
}

impl RoadmapRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Nodes ──

    pub async fn add_node(
        &self,
        map_id: Uuid,
        name: Option<&str>,
        node_type: &NodeType,
        position: &Position3D,
        properties: &serde_json::Value,
    ) -> Result<RoadmapNode> {
        // PostGIS ST_MakePoint로 PointZ 생성
        let row = sqlx::query_as::<_, (Uuid, Option<String>, String, f64, f64, f64, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
            r#"
            INSERT INTO roadmap_nodes (map_id, name, node_type, position, properties)
            VALUES ($1, $2, $3, ST_SetSRID(ST_MakePoint($4, $5, $6), 4326), $7)
            RETURNING id, name, node_type,
                      ST_X(position), ST_Y(position), ST_Z(position),
                      properties, created_at
            "#,
        )
        .bind(map_id)
        .bind(name)
        .bind(node_type.as_str())
        .bind(position.x)
        .bind(position.y)
        .bind(position.z)
        .bind(properties)
        .fetch_one(&self.pool)
        .await?;

        Ok(RoadmapNode {
            id: row.0,
            map_id,
            name: row.1,
            node_type: NodeType::from_str(&row.2).unwrap_or(NodeType::Waypoint),
            position: Position3D { x: row.3, y: row.4, z: row.5 },
            properties: row.6,
            created_at: row.7,
        })
    }

    pub async fn get_nodes_by_map(&self, map_id: Uuid) -> Result<Vec<RoadmapNode>> {
        let rows = sqlx::query_as::<_, (Uuid, Option<String>, String, f64, f64, f64, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, name, node_type,
                   ST_X(position), ST_Y(position), ST_Z(position),
                   properties, created_at
            FROM roadmap_nodes
            WHERE map_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(map_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| RoadmapNode {
            id: row.0,
            map_id,
            name: row.1,
            node_type: NodeType::from_str(&row.2).unwrap_or(NodeType::Waypoint),
            position: Position3D { x: row.3, y: row.4, z: row.5 },
            properties: row.6,
            created_at: row.7,
        }).collect())
    }

    /// PostGIS 기반 최근접 노드 검색
    pub async fn find_nearest_nodes(
        &self,
        map_id: Uuid,
        position: &Position3D,
        type_filter: Option<&NodeType>,
        max_results: i32,
        max_distance: Option<f64>,
    ) -> Result<Vec<RoadmapNode>> {
        // ST_3DDistance로 3D 유클리드 거리 계산
        let query = r#"
            SELECT id, name, node_type,
                   ST_X(position), ST_Y(position), ST_Z(position),
                   properties, created_at,
                   ST_3DDistance(position, ST_SetSRID(ST_MakePoint($2, $3, $4), 4326)) as dist
            FROM roadmap_nodes
            WHERE map_id = $1
              AND ($5::text IS NULL OR node_type = $5)
              AND ($6::float IS NULL OR ST_3DDistance(position, ST_SetSRID(ST_MakePoint($2, $3, $4), 4326)) <= $6)
            ORDER BY dist
            LIMIT $7
        "#;

        let rows = sqlx::query_as::<_, (Uuid, Option<String>, String, f64, f64, f64, serde_json::Value, chrono::DateTime<chrono::Utc>, f64)>(query)
            .bind(map_id)
            .bind(position.x)
            .bind(position.y)
            .bind(position.z)
            .bind(type_filter.map(|t| t.as_str()))
            .bind(max_distance)
            .bind(max_results)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|row| RoadmapNode {
            id: row.0,
            map_id,
            name: row.1,
            node_type: NodeType::from_str(&row.2).unwrap_or(NodeType::Waypoint),
            position: Position3D { x: row.3, y: row.4, z: row.5 },
            properties: row.6,
            created_at: row.7,
        }).collect())
    }

    pub async fn delete_node(&self, node_id: Uuid) -> Result<i32> {
        // 연쇄 삭제: 연결된 엣지도 삭제 (FK CASCADE)
        let edge_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM roadmap_edges WHERE source_node_id = $1 OR target_node_id = $1"
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query("DELETE FROM roadmap_nodes WHERE id = $1")
            .bind(node_id)
            .execute(&self.pool)
            .await?;

        Ok(edge_count as i32)
    }

    // ── Edges ──

    pub async fn add_edge(
        &self,
        map_id: Uuid,
        source_node_id: Uuid,
        target_node_id: Uuid,
        max_speed: f32,
        direction: &EdgeDirection,
        allowed_robot_types: &[String],
        cost_factor: f32,
        path_points: &[Position3D],
        properties: &serde_json::Value,
    ) -> Result<RoadmapEdge> {
        // 두 노드 위치로 distance 자동 계산
        let distance: (f64,) = sqlx::query_as(
            r#"
            SELECT ST_3DDistance(
                (SELECT position FROM roadmap_nodes WHERE id = $1),
                (SELECT position FROM roadmap_nodes WHERE id = $2)
            )
            "#,
        )
        .bind(source_node_id)
        .bind(target_node_id)
        .fetch_one(&self.pool)
        .await?;

        // path_points로 LineStringZ 생성 (비어있으면 두 노드 직선)
        let linestring_wkt = if path_points.is_empty() {
            format!(
                "SRID=4326;LINESTRING Z(({src_x} {src_y} {src_z}),({tgt_x} {tgt_y} {tgt_z}))",
                src_x = 0.0, src_y = 0.0, src_z = 0.0,
                tgt_x = 0.0, tgt_y = 0.0, tgt_z = 0.0
            )
            // 실제 구현에서는 두 노드의 좌표를 조회하여 사용
        } else {
            let coords: Vec<String> = path_points
                .iter()
                .map(|p| format!("{} {} {}", p.x, p.y, p.z))
                .collect();
            format!("SRID=4326;LINESTRING Z({})", coords.join(","))
        };

        let dir_str = match direction {
            EdgeDirection::Unidirectional => "uni",
            EdgeDirection::Bidirectional => "bi",
        };

        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO roadmap_edges
                (id, map_id, source_node_id, target_node_id, distance, max_speed,
                 direction, allowed_robot_types, cost_factor, path, properties)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    ST_GeomFromEWKT($10), $11)
            "#,
        )
        .bind(id)
        .bind(map_id)
        .bind(source_node_id)
        .bind(target_node_id)
        .bind(distance.0 as f32)
        .bind(max_speed)
        .bind(dir_str)
        .bind(allowed_robot_types)
        .bind(cost_factor)
        .bind(&linestring_wkt)
        .bind(properties)
        .execute(&self.pool)
        .await?;

        Ok(RoadmapEdge {
            id,
            map_id,
            source_node_id,
            target_node_id,
            distance: distance.0 as f32,
            max_speed,
            direction: direction.clone(),
            allowed_robot_types: allowed_robot_types.to_vec(),
            cost_factor,
            path_points: path_points.to_vec(),
            properties: properties.clone(),
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn get_edges_by_map(&self, map_id: Uuid) -> Result<Vec<RoadmapEdge>> {
        let rows = sqlx::query_as::<_, (
            Uuid, Uuid, Uuid, f32, f32, String, Vec<String>, f32,
            serde_json::Value, chrono::DateTime<chrono::Utc>,
        )>(
            r#"
            SELECT id, source_node_id, target_node_id, distance, max_speed,
                   direction, allowed_robot_types, cost_factor,
                   properties, created_at
            FROM roadmap_edges
            WHERE map_id = $1
            "#,
        )
        .bind(map_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| RoadmapEdge {
            id: row.0,
            map_id,
            source_node_id: row.1,
            target_node_id: row.2,
            distance: row.3,
            max_speed: row.4,
            direction: if row.5 == "bi" { EdgeDirection::Bidirectional } else { EdgeDirection::Unidirectional },
            allowed_robot_types: row.6,
            cost_factor: row.7,
            path_points: vec![], // LineString에서 추출 시 별도 쿼리 필요
            properties: row.8,
            created_at: row.9,
        }).collect())
    }
}
```

#### 4.3.4 Semantic 저장소 (`src/storage/semantic_repo.rs`)

```rust
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;
use crate::models::semantic::*;

pub struct SemanticRepository {
    pool: PgPool,
}

impl SemanticRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 특정 좌표가 포함된 시맨틱 영역 조회 (point-in-polygon)
    /// PostGIS ST_Contains 사용
    pub async fn find_regions_containing_point(
        &self,
        map_id: Uuid,
        x: f64,
        y: f64,
        z: f64,
    ) -> Result<Vec<SemanticRegion>> {
        let rows = sqlx::query_as::<_, (Uuid, Option<String>, String, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, name, region_type, properties, created_at
            FROM semantic_regions
            WHERE map_id = $1
              AND ST_Contains(
                geometry,
                ST_SetSRID(ST_MakePoint($2, $3, $4), 4326)
              )
            "#,
        )
        .bind(map_id)
        .bind(x)
        .bind(y)
        .bind(z)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| SemanticRegion {
            id: row.0,
            map_id,
            name: row.1,
            region_type: RegionType::from_str(&row.2),
            geometry: vec![], // 별도 ST_AsText로 추출
            properties: row.3,
            created_at: row.4,
        }).collect())
    }

    /// 경로(LineString)와 교차하는 시맨틱 영역 조회
    /// 경로가 금지 구역을 통과하는지 확인하는 데 사용
    pub async fn find_regions_intersecting_path(
        &self,
        map_id: Uuid,
        path_wkt: &str,  // "LINESTRING Z(x1 y1 z1, x2 y2 z2, ...)"
        region_type_filter: Option<&str>,
    ) -> Result<Vec<SemanticRegion>> {
        let rows = sqlx::query_as::<_, (Uuid, Option<String>, String, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, name, region_type, properties, created_at
            FROM semantic_regions
            WHERE map_id = $1
              AND ST_Intersects(geometry, ST_GeomFromEWKT($2))
              AND ($3::text IS NULL OR region_type = $3)
            "#,
        )
        .bind(map_id)
        .bind(format!("SRID=4326;{}", path_wkt))
        .bind(region_type_filter)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| SemanticRegion {
            id: row.0,
            map_id,
            name: row.1,
            region_type: RegionType::from_str(&row.2),
            geometry: vec![],
            properties: row.3,
            created_at: row.4,
        }).collect())
    }

    /// 폴리곤 꼭짓점을 WKT로 변환하여 INSERT
    pub async fn add_region(
        &self,
        map_id: Uuid,
        name: Option<&str>,
        region_type: &RegionType,
        vertices: &[[f64; 3]],
        properties: &serde_json::Value,
    ) -> Result<SemanticRegion> {
        let coords: Vec<String> = vertices.iter()
            .map(|v| format!("{} {} {}", v[0], v[1], v[2]))
            .collect();
        let polygon_wkt = format!("SRID=4326;POLYGON Z(({}))", coords.join(","));

        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO semantic_regions (id, map_id, name, region_type, geometry, properties)
            VALUES ($1, $2, $3, $4, ST_GeomFromEWKT($5), $6)
            "#,
        )
        .bind(id)
        .bind(map_id)
        .bind(name)
        .bind(region_type.as_str())
        .bind(&polygon_wkt)
        .bind(properties)
        .execute(&self.pool)
        .await?;

        Ok(SemanticRegion {
            id,
            map_id,
            name: name.map(String::from),
            region_type: region_type.clone(),
            geometry: vertices.to_vec(),
            properties: properties.clone(),
            created_at: chrono::Utc::now(),
        })
    }
}
```

#### 4.3.5 MinIO S3 타일 스토리지 (`src/storage/s3_tiles.rs`)

```rust
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use bytes::Bytes;

use crate::config::MinioConfig;
use crate::error::{MapManagerError, Result};

pub struct TileStorage {
    client: S3Client,
    raw_bucket: String,
    tiles_bucket: String,
}

impl TileStorage {
    pub async fn new(config: &MinioConfig) -> Result<Self> {
        let s3_config = aws_config::from_env()
            .endpoint_url(&config.endpoint)
            .region(aws_config::Region::new(config.region.clone()))
            .load()
            .await;

        let client = S3Client::new(&s3_config);

        Ok(Self {
            client,
            raw_bucket: config.raw_bucket.clone(),
            tiles_bucket: config.tiles_bucket.clone(),
        })
    }

    /// 청크 업로드: raw 포인트 클라우드 파일을 MinIO에 저장
    /// Multipart upload 사용하여 대용량 파일 지원
    pub async fn upload_raw_file(
        &self,
        map_id: &str,
        filename: &str,
        data: ByteStream,
    ) -> Result<String> {
        let key = format!("raw/{}/{}", map_id, filename);

        self.client
            .put_object()
            .bucket(&self.raw_bucket)
            .key(&key)
            .body(data)
            .send()
            .await
            .map_err(|e| MapManagerError::S3(e.to_string()))?;

        Ok(key)
    }

    /// Potree 타일 업로드 (처리 완료 후)
    pub async fn upload_tile(
        &self,
        map_id: &str,
        tile_path: &str,  // 예: "hierarchy.bin", "octree.bin", "r/r0/r01.bin"
        data: Bytes,
    ) -> Result<()> {
        let key = format!("tiles/{}/{}", map_id, tile_path);

        self.client
            .put_object()
            .bucket(&self.tiles_bucket)
            .key(&key)
            .body(ByteStream::from(data))
            .send()
            .await
            .map_err(|e| MapManagerError::S3(e.to_string()))?;

        Ok(())
    }

    /// 타일 다운로드: node_id 기반
    pub async fn get_tile(&self, map_id: &str, node_id: &str) -> Result<Bytes> {
        let key = format!("tiles/{}/{}.bin", map_id, node_id);

        let response = self.client
            .get_object()
            .bucket(&self.tiles_bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| MapManagerError::TileNotFound {
                map_id: map_id.to_string(),
                node_id: node_id.to_string(),
            })?;

        let data = response.body.collect().await
            .map_err(|e| MapManagerError::S3(e.to_string()))?;

        Ok(data.into_bytes())
    }

    /// Potree metadata.json 다운로드
    pub async fn get_potree_metadata(&self, map_id: &str) -> Result<Bytes> {
        let key = format!("tiles/{}/metadata.json", map_id);

        let response = self.client
            .get_object()
            .bucket(&self.tiles_bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| MapManagerError::S3(e.to_string()))?;

        let data = response.body.collect().await
            .map_err(|e| MapManagerError::S3(e.to_string()))?;

        Ok(data.into_bytes())
    }

    /// 맵 삭제 시 관련 모든 S3 객체 삭제
    pub async fn delete_map_data(&self, map_id: &str) -> Result<()> {
        // raw 파일 삭제
        self.delete_prefix(&self.raw_bucket, &format!("raw/{}/", map_id)).await?;
        // 타일 삭제
        self.delete_prefix(&self.tiles_bucket, &format!("tiles/{}/", map_id)).await?;
        Ok(())
    }

    async fn delete_prefix(&self, bucket: &str, prefix: &str) -> Result<()> {
        let mut continuation_token = None;

        loop {
            let mut request = self.client
                .list_objects_v2()
                .bucket(bucket)
                .prefix(prefix);

            if let Some(token) = &continuation_token {
                request = request.continuation_token(token);
            }

            let response = request.send().await
                .map_err(|e| MapManagerError::S3(e.to_string()))?;

            if let Some(contents) = response.contents {
                for obj in contents {
                    if let Some(key) = obj.key {
                        self.client
                            .delete_object()
                            .bucket(bucket)
                            .key(&key)
                            .send()
                            .await
                            .map_err(|e| MapManagerError::S3(e.to_string()))?;
                    }
                }
            }

            if response.is_truncated.unwrap_or(false) {
                continuation_token = response.next_continuation_token;
            } else {
                break;
            }
        }

        Ok(())
    }
}
```

#### 4.3.6 Redis 캐시 (`src/storage/cache.rs`)

```rust
use std::time::Duration;
use bytes::Bytes;
use fred::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::config::RedisConfig;
use crate::error::Result;
use crate::models::obstacle::DynamicObstacle;

pub struct CacheManager {
    client: Client,
    tile_cache_ttl: Duration,
    dynamic_obstacle_ttl: Duration,
}

impl CacheManager {
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        let redis_config = fred::types::Config::from_url(&config.url)?;
        let client = Client::new(redis_config, None, None, None);
        client.connect();
        client.wait_for_connect().await?;

        Ok(Self {
            client,
            tile_cache_ttl: Duration::from_secs(config.tile_cache_ttl_secs),
            dynamic_obstacle_ttl: Duration::from_secs(config.dynamic_obstacle_default_ttl_secs),
        })
    }

    // ── 타일 캐시 ──

    pub async fn get_cached_tile(&self, map_id: &str, node_id: &str) -> Result<Option<Bytes>> {
        let key = format!("tile:{}:{}", map_id, node_id);
        let data: Option<Bytes> = self.client.get(&key).await?;
        Ok(data)
    }

    pub async fn cache_tile(&self, map_id: &str, node_id: &str, data: &[u8]) -> Result<()> {
        let key = format!("tile:{}:{}", map_id, node_id);
        self.client.set(
            &key,
            data,
            Some(Expiration::EX(self.tile_cache_ttl.as_secs() as i64)),
            None,
            false,
        ).await?;
        Ok(())
    }

    // ── 동적 장애물 ──

    /// 동적 장애물 저장 (TTL 기반 자동 만료)
    pub async fn set_dynamic_obstacle(&self, obstacle: &DynamicObstacle, ttl_secs: Option<u64>) -> Result<()> {
        let key = format!(
            "dynamic_obstacle:{}:{}",
            obstacle.map_id, obstacle.obstacle_id
        );
        let value = serde_json::to_string(obstacle)?;
        let ttl = ttl_secs.unwrap_or(self.dynamic_obstacle_ttl.as_secs());

        self.client.set(
            &key,
            value.as_str(),
            Some(Expiration::EX(ttl as i64)),
            None,
            false,
        ).await?;

        // 맵별 동적 장애물 셋에 추가 (SCAN 대신 SET 사용)
        let set_key = format!("dynamic_obstacles_set:{}", obstacle.map_id);
        self.client.sadd(&set_key, &obstacle.obstacle_id).await?;

        Ok(())
    }

    /// 특정 맵의 모든 동적 장애물 조회
    pub async fn get_dynamic_obstacles(&self, map_id: &str) -> Result<Vec<DynamicObstacle>> {
        let set_key = format!("dynamic_obstacles_set:{}", map_id);
        let obstacle_ids: Vec<String> = self.client.smembers(&set_key).await?;

        let mut obstacles = Vec::new();
        let mut expired_ids = Vec::new();

        for obstacle_id in &obstacle_ids {
            let key = format!("dynamic_obstacle:{}:{}", map_id, obstacle_id);
            let data: Option<String> = self.client.get(&key).await?;

            match data {
                Some(json) => {
                    if let Ok(obstacle) = serde_json::from_str::<DynamicObstacle>(&json) {
                        obstacles.push(obstacle);
                    }
                }
                None => {
                    // TTL 만료됨 → 셋에서 제거
                    expired_ids.push(obstacle_id.clone());
                }
            }
        }

        // 만료된 ID를 셋에서 정리
        if !expired_ids.is_empty() {
            for id in &expired_ids {
                self.client.srem(&set_key, id).await?;
            }
        }

        Ok(obstacles)
    }

    /// 동적 장애물 삭제
    pub async fn delete_dynamic_obstacle(&self, map_id: &str, obstacle_id: &str) -> Result<()> {
        let key = format!("dynamic_obstacle:{}:{}", map_id, obstacle_id);
        self.client.del(&key).await?;

        let set_key = format!("dynamic_obstacles_set:{}", map_id);
        self.client.srem(&set_key, obstacle_id).await?;

        Ok(())
    }
}
```

### 4.4 경로 탐색 모듈 상세

#### 4.4.1 그래프 자료구조 (`src/pathfinding/graph.rs`)

```rust
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::roadmap::*;

/// 인접 리스트 기반 방향 그래프
/// 양방향 엣지는 양쪽 방향으로 두 개의 엣지로 저장
#[derive(Debug, Clone)]
pub struct RoadmapGraphData {
    pub nodes: HashMap<Uuid, GraphNode>,
    pub adjacency: HashMap<Uuid, Vec<AdjacencyEntry>>,
}

#[derive(Debug, Clone)]
pub struct AdjacencyEntry {
    pub edge_id: Uuid,
    pub target_node_id: Uuid,
    pub distance: f32,
    pub max_speed: f32,
    pub cost_factor: f32,
    pub allowed_robot_types: Vec<String>,
}

impl RoadmapGraphData {
    /// DB에서 로드한 노드/엣지로 인메모리 그래프 구축
    pub fn build(nodes: Vec<RoadmapNode>, edges: Vec<RoadmapEdge>) -> Self {
        let mut node_map = HashMap::new();
        let mut adjacency: HashMap<Uuid, Vec<AdjacencyEntry>> = HashMap::new();

        for node in &nodes {
            node_map.insert(node.id, GraphNode {
                id: node.id,
                position: node.position,
                node_type: node.node_type.clone(),
            });
            adjacency.entry(node.id).or_default();
        }

        for edge in &edges {
            let entry = AdjacencyEntry {
                edge_id: edge.id,
                target_node_id: edge.target_node_id,
                distance: edge.distance,
                max_speed: edge.max_speed,
                cost_factor: edge.cost_factor,
                allowed_robot_types: edge.allowed_robot_types.clone(),
            };

            adjacency.entry(edge.source_node_id).or_default().push(entry);

            // 양방향 엣지: 반대 방향도 추가
            if edge.direction == EdgeDirection::Bidirectional {
                let reverse_entry = AdjacencyEntry {
                    edge_id: edge.id,
                    target_node_id: edge.source_node_id,
                    distance: edge.distance,
                    max_speed: edge.max_speed,
                    cost_factor: edge.cost_factor,
                    allowed_robot_types: edge.allowed_robot_types.clone(),
                };
                adjacency.entry(edge.target_node_id).or_default().push(reverse_entry);
            }
        }

        Self {
            nodes: node_map,
            adjacency,
        }
    }

    pub fn neighbors(&self, node_id: &Uuid) -> &[AdjacencyEntry] {
        self.adjacency
            .get(node_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|v| v.len()).sum()
    }
}
```

#### 4.4.2 A* 알고리즘 (`src/pathfinding/astar.rs`)

```rust
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use uuid::Uuid;

use crate::error::{MapManagerError, Result};
use crate::models::roadmap::Position3D;
use super::graph::{RoadmapGraphData, AdjacencyEntry};
use super::heuristics::euclidean_distance_3d;

/// A* 경로 탐색 결과
#[derive(Debug)]
pub struct AStarResult {
    pub path: Vec<Uuid>,          // 노드 ID 순서 목록
    pub total_distance: f32,
    pub total_cost: f32,
    pub edge_ids: Vec<Uuid>,      // 경유한 엣지 ID 목록
}

/// 우선순위 큐용 노드 (min-heap)
#[derive(Debug)]
struct OpenNode {
    node_id: Uuid,
    f_score: f32,   // g + h
}

impl Eq for OpenNode {}
impl PartialEq for OpenNode {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}
impl Ord for OpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // 역순: BinaryHeap은 max-heap이므로 f_score가 작은 것이 우선
        other.f_score.partial_cmp(&self.f_score)
            .unwrap_or(Ordering::Equal)
    }
}
impl PartialOrd for OpenNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A* 경로 탐색
///
/// # 매개변수
/// - `graph`: 인메모리 그래프
/// - `start`: 출발 노드 UUID
/// - `goal`: 도착 노드 UUID
/// - `robot_type`: 로봇 유형 (엣지 필터링, None이면 무시)
/// - `max_cost`: 최대 비용 제한 (0이면 무제한)
pub fn astar(
    graph: &RoadmapGraphData,
    start: Uuid,
    goal: Uuid,
    robot_type: Option<&str>,
    max_cost: f32,
) -> Result<AStarResult> {
    let goal_node = graph.nodes.get(&goal)
        .ok_or(MapManagerError::NodeNotFound(goal.to_string()))?;
    let start_node = graph.nodes.get(&start)
        .ok_or(MapManagerError::NodeNotFound(start.to_string()))?;

    let goal_pos = goal_node.position;

    // g_score: 시작 노드에서의 실제 비용
    let mut g_score: HashMap<Uuid, f32> = HashMap::new();
    g_score.insert(start, 0.0);

    // came_from: 경로 추적용
    let mut came_from: HashMap<Uuid, (Uuid, Uuid)> = HashMap::new(); // node → (prev_node, edge_id)

    // Open set
    let mut open_set = BinaryHeap::new();
    let h = euclidean_distance_3d(&start_node.position, &goal_pos);
    open_set.push(OpenNode { node_id: start, f_score: h });

    // Closed set
    let mut closed: HashMap<Uuid, bool> = HashMap::new();

    while let Some(current) = open_set.pop() {
        let current_id = current.node_id;

        if current_id == goal {
            // 경로 재구성
            return Ok(reconstruct_path(&came_from, &g_score, start, goal));
        }

        if closed.contains_key(&current_id) {
            continue;
        }
        closed.insert(current_id, true);

        let current_g = *g_score.get(&current_id).unwrap_or(&f32::MAX);

        // max_cost 제한
        if max_cost > 0.0 && current_g > max_cost {
            continue;
        }

        for neighbor in graph.neighbors(&current_id) {
            if closed.contains_key(&neighbor.target_node_id) {
                continue;
            }

            // 로봇 타입 필터링
            if let Some(rt) = robot_type {
                if !neighbor.allowed_robot_types.is_empty()
                    && !neighbor.allowed_robot_types.iter().any(|t| t == rt)
                {
                    continue;
                }
            }

            let edge_cost = neighbor.distance * neighbor.cost_factor;
            let tentative_g = current_g + edge_cost;

            let existing_g = *g_score.get(&neighbor.target_node_id).unwrap_or(&f32::MAX);

            if tentative_g < existing_g {
                g_score.insert(neighbor.target_node_id, tentative_g);
                came_from.insert(
                    neighbor.target_node_id,
                    (current_id, neighbor.edge_id),
                );

                let neighbor_pos = &graph.nodes[&neighbor.target_node_id].position;
                let h = euclidean_distance_3d(neighbor_pos, &goal_pos);
                let f = tentative_g + h;

                open_set.push(OpenNode {
                    node_id: neighbor.target_node_id,
                    f_score: f,
                });
            }
        }
    }

    Err(MapManagerError::PathNotFound {
        from: start.to_string(),
        to: goal.to_string(),
    })
}

fn reconstruct_path(
    came_from: &HashMap<Uuid, (Uuid, Uuid)>,
    g_score: &HashMap<Uuid, f32>,
    start: Uuid,
    goal: Uuid,
) -> AStarResult {
    let mut path = vec![goal];
    let mut edge_ids = Vec::new();
    let mut current = goal;

    while current != start {
        if let Some((prev, edge_id)) = came_from.get(&current) {
            path.push(*prev);
            edge_ids.push(*edge_id);
            current = *prev;
        } else {
            break;
        }
    }

    path.reverse();
    edge_ids.reverse();

    let total_cost = *g_score.get(&goal).unwrap_or(&0.0);

    // total_distance는 cost_factor를 제외한 순수 거리
    // (간단히 total_cost와 같이 사용하거나 별도 계산)
    AStarResult {
        total_distance: total_cost, // 별도 distance 합산 필요 시 수정
        total_cost,
        path,
        edge_ids,
    }
}
```

#### 4.4.3 휴리스틱 (`src/pathfinding/heuristics.rs`)

```rust
use crate::models::roadmap::Position3D;

/// 3D 유클리드 거리 (A* 기본 휴리스틱)
/// admissible하고 consistent한 휴리스틱 (최적 경로 보장)
pub fn euclidean_distance_3d(a: &Position3D, b: &Position3D) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    ((dx * dx + dy * dy + dz * dz) as f64).sqrt() as f32
}

/// 2D 유클리드 거리 (Z축 무시, 평면 로봇용)
pub fn euclidean_distance_2d(a: &Position3D, b: &Position3D) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    ((dx * dx + dy * dy) as f64).sqrt() as f32
}

/// 맨해튼 거리 (격자형 로드맵에 적합)
pub fn manhattan_distance_3d(a: &Position3D, b: &Position3D) -> f32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    let dz = (a.z - b.z).abs();
    (dx + dy + dz) as f32
}
```

#### 4.4.4 충돌 감지 (`src/pathfinding/conflict.rs`)

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

use crate::models::roadmap::Position3D;

/// 활성 로봇 경로 (다른 로봇이 예약한 경로)
#[derive(Debug, Clone)]
pub struct ActiveRobotPath {
    pub robot_id: String,
    pub node_ids: Vec<Uuid>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub estimated_node_times: Vec<DateTime<Utc>>, // 각 노드 도착 예상 시각
}

#[derive(Debug, Clone)]
pub struct PathConflict {
    pub conflicting_robot_id: String,
    pub node_id: Uuid,
    pub conflict_time: DateTime<Utc>,
    pub conflict_type: ConflictType,
}

#[derive(Debug, Clone)]
pub enum ConflictType {
    /// 같은 노드에 동시 도달
    NodeOverlap,
    /// 같은 엣지를 반대 방향으로 동시 이동
    HeadOn,
    /// 엣지 교차
    EdgeCrossing,
}

impl ConflictType {
    pub fn as_str(&self) -> &str {
        match self {
            ConflictType::NodeOverlap => "node_overlap",
            ConflictType::HeadOn => "head_on",
            ConflictType::EdgeCrossing => "edge_crossing",
        }
    }
}

/// 경로 충돌 감지
///
/// 주어진 경로가 다른 활성 로봇 경로와 시간적·공간적으로 충돌하는지 확인한다.
/// - 충돌 시간 윈도우: 같은 노드에 ±2초 이내로 도달하면 충돌로 판정
pub fn check_conflicts(
    proposed_path: &[Uuid],
    proposed_times: &[DateTime<Utc>],
    requesting_robot_id: &str,
    active_paths: &[ActiveRobotPath],
    time_buffer_secs: i64,  // 기본 2초
) -> Vec<PathConflict> {
    let mut conflicts = Vec::new();

    let proposed_node_times: HashMap<Uuid, &DateTime<Utc>> = proposed_path.iter()
        .zip(proposed_times.iter())
        .collect();

    for active in active_paths {
        if active.robot_id == requesting_robot_id {
            continue;
        }

        let active_node_times: HashMap<Uuid, &DateTime<Utc>> = active.node_ids.iter()
            .zip(active.estimated_node_times.iter())
            .collect();

        // 노드 겹침 검사
        for (node_id, proposed_time) in &proposed_node_times {
            if let Some(active_time) = active_node_times.get(node_id) {
                let diff = (**proposed_time - ***active_time).num_seconds().abs();
                if diff <= time_buffer_secs {
                    conflicts.push(PathConflict {
                        conflicting_robot_id: active.robot_id.clone(),
                        node_id: **node_id,
                        conflict_time: **proposed_time,
                        conflict_type: ConflictType::NodeOverlap,
                    });
                }
            }
        }

        // Head-on 검사: 연속 두 노드가 반대 순서로 나타나는 경우
        for i in 0..proposed_path.len().saturating_sub(1) {
            let a = proposed_path[i];
            let b = proposed_path[i + 1];

            for j in 0..active.node_ids.len().saturating_sub(1) {
                if active.node_ids[j] == b && active.node_ids[j + 1] == a {
                    let proposed_mid = proposed_times[i];
                    let active_mid = active.estimated_node_times[j];
                    let diff = (proposed_mid - active_mid).num_seconds().abs();
                    if diff <= time_buffer_secs * 2 {
                        conflicts.push(PathConflict {
                            conflicting_robot_id: active.robot_id.clone(),
                            node_id: a,
                            conflict_time: proposed_mid,
                            conflict_type: ConflictType::HeadOn,
                        });
                    }
                }
            }
        }
    }

    conflicts
}
```

### 4.5 gRPC 핸들러 구현 패턴

#### 4.5.1 서버 셋업 (`src/grpc/server.rs`)

```rust
use std::sync::Arc;
use tonic::transport::Server;

use crate::config::AppConfig;
use crate::storage::cache::CacheManager;
use crate::storage::metadata::MapRepository;
use crate::storage::roadmap_repo::RoadmapRepository;
use crate::storage::semantic_repo::SemanticRepository;
use crate::storage::obstacle_repo::ObstacleRepository;
use crate::storage::s3_tiles::TileStorage;

/// 모든 핸들러가 공유하는 애플리케이션 상태
pub struct AppState {
    pub config: AppConfig,
    pub map_repo: MapRepository,
    pub roadmap_repo: RoadmapRepository,
    pub semantic_repo: SemanticRepository,
    pub obstacle_repo: ObstacleRepository,
    pub tile_storage: TileStorage,
    pub cache: CacheManager,
}

pub async fn start_server(config: AppConfig) -> anyhow::Result<()> {
    let pool = crate::storage::postgres::create_pool(&config.database).await?;
    let tile_storage = TileStorage::new(&config.minio).await?;
    let cache = CacheManager::new(&config.redis).await?;

    let state = Arc::new(AppState {
        map_repo: MapRepository::new(pool.clone()),
        roadmap_repo: RoadmapRepository::new(pool.clone()),
        semantic_repo: SemanticRepository::new(pool.clone()),
        obstacle_repo: ObstacleRepository::new(pool.clone()),
        tile_storage,
        cache,
        config: config.clone(),
    });

    let map_service = crate::grpc::map_handlers::MapServiceImpl::new(state.clone());

    let addr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    tracing::info!("Map Manager gRPC server listening on {}", addr);

    Server::builder()
        .max_frame_size(Some((config.server.max_message_size / 1024) as u32)) // bytes → KB
        .add_service(
            crate::generated::map_service::map_service_server::MapServiceServer::new(map_service)
                .max_decoding_message_size(config.server.max_message_size)
                .max_encoding_message_size(config.server.max_message_size)
        )
        .serve(addr)
        .await?;

    Ok(())
}
```

#### 4.5.2 핸들러 예시: UploadPointCloud (`src/grpc/pointcloud_handlers.rs`)

```rust
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tonic::{Request, Response, Status, Streaming};

use crate::grpc::server::AppState;
use crate::generated::map_service::*;

impl MapServiceImpl {
    /// 청크 스트림으로 포인트 클라우드 업로드
    /// 1. 스트림에서 청크를 수신하여 임시 파일에 기록
    /// 2. 임시 파일을 MinIO raw 버킷에 업로드
    /// 3. processing_job 생성
    /// 4. Python 파이프라인 비동기 실행 (tokio::spawn)
    pub async fn upload_point_cloud_impl(
        &self,
        request: Request<Streaming<PointCloudChunk>>,
    ) -> std::result::Result<Response<ProcessingJob>, Status> {
        let state = self.state.clone();
        let mut stream = request.into_inner();

        let mut map_id = String::new();
        let mut filename = String::new();
        let mut format = String::new();
        let mut total_bytes: u64 = 0;

        // 임시 파일 생성
        let temp_dir = &state.config.processing.temp_dir;
        tokio::fs::create_dir_all(temp_dir).await
            .map_err(|e| Status::internal(format!("Failed to create temp dir: {}", e)))?;

        let temp_id = uuid::Uuid::new_v4().to_string();
        let temp_path = format!("{}/{}", temp_dir, temp_id);
        let mut temp_file = tokio::fs::File::create(&temp_path).await
            .map_err(|e| Status::internal(format!("Failed to create temp file: {}", e)))?;

        // 스트림에서 청크 수신
        while let Some(chunk) = stream.message().await? {
            if chunk.chunk_index == 0 {
                map_id = chunk.map_id;
                filename = chunk.filename;
                format = chunk.format;
            }

            temp_file.write_all(&chunk.data).await
                .map_err(|e| Status::internal(format!("Write error: {}", e)))?;

            total_bytes += chunk.data.len() as u64;
        }

        temp_file.flush().await
            .map_err(|e| Status::internal(format!("Flush error: {}", e)))?;

        if map_id.is_empty() {
            return Err(Status::invalid_argument("map_id is required in the first chunk"));
        }

        // MinIO에 raw 파일 업로드
        let raw_data = tokio::fs::read(&temp_path).await
            .map_err(|e| Status::internal(e.to_string()))?;
        let raw_path = state.tile_storage
            .upload_raw_file(&map_id, &filename, raw_data.into())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Processing job 생성 (DB)
        let job_id = uuid::Uuid::new_v4();
        let map_uuid = uuid::Uuid::parse_str(&map_id)
            .map_err(|_| Status::invalid_argument("Invalid map_id UUID"))?;

        // DB에 job 레코드 삽입 (구현은 metadata.rs에서)
        // ...

        // Python 파이프라인 비동기 실행
        let config = state.config.processing.clone();
        let job_id_clone = job_id.to_string();
        let map_id_clone = map_id.clone();

        tokio::spawn(async move {
            let result = tokio::process::Command::new(&config.python_executable)
                .arg(&config.pipeline_script)
                .arg("--job-id").arg(&job_id_clone)
                .arg("--map-id").arg(&map_id_clone)
                .arg("--input-file").arg(&temp_path)
                .arg("--format").arg(&format)
                .arg("--voxel-size").arg(config.default_voxel_size.to_string())
                .arg("--potree-converter").arg(&config.potree_converter_path)
                .status()
                .await;

            match result {
                Ok(status) if status.success() => {
                    tracing::info!("Processing job {} completed", job_id_clone);
                }
                Ok(status) => {
                    tracing::error!("Processing job {} failed with exit code: {:?}", job_id_clone, status.code());
                }
                Err(e) => {
                    tracing::error!("Processing job {} failed to start: {}", job_id_clone, e);
                }
            }

            // 임시 파일 정리
            let _ = tokio::fs::remove_file(&temp_path).await;
        });

        let response = ProcessingJob {
            job_id: job_id.to_string(),
            map_id,
            job_type: "pointcloud_processing".to_string(),
            status: "pending".to_string(),
            progress: 0.0,
            error_message: String::new(),
            created_at: None,
            completed_at: None,
        };

        Ok(Response::new(response))
    }
}
```

#### 4.5.3 핸들러 예시: FindPath (`src/grpc/pathfinding_handlers.rs`)

```rust
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::grpc::server::AppState;
use crate::generated::map_service::*;
use crate::pathfinding::astar;
use crate::pathfinding::graph::RoadmapGraphData;

impl MapServiceImpl {
    pub async fn find_path_impl(
        &self,
        request: Request<FindPathRequest>,
    ) -> std::result::Result<Response<PathResult>, Status> {
        let req = request.into_inner();
        let state = &self.state;

        let map_id = uuid::Uuid::parse_str(&req.map_id)
            .map_err(|_| Status::invalid_argument("Invalid map_id"))?;
        let start_id = uuid::Uuid::parse_str(&req.start_node_id)
            .map_err(|_| Status::invalid_argument("Invalid start_node_id"))?;
        let goal_id = uuid::Uuid::parse_str(&req.goal_node_id)
            .map_err(|_| Status::invalid_argument("Invalid goal_node_id"))?;

        // DB에서 그래프 로드 (캐시 도입 가능)
        let nodes = state.roadmap_repo.get_nodes_by_map(map_id).await
            .map_err(|e| Status::internal(e.to_string()))?;
        let edges = state.roadmap_repo.get_edges_by_map(map_id).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let graph = RoadmapGraphData::build(nodes, edges);

        // A* 실행
        let robot_type = if req.robot_type.is_empty() { None } else { Some(req.robot_type.as_str()) };

        let result = astar::astar(&graph, start_id, goal_id, robot_type, req.max_cost)
            .map_err(|e| Status::from(e))?;

        // 경로 좌표 추출
        let path_points: Vec<Point3d> = result.path.iter()
            .filter_map(|node_id| graph.nodes.get(node_id))
            .map(|node| Point3d {
                x: node.position.x,
                y: node.position.y,
                z: node.position.z,
            })
            .collect();

        // 구간별 상세 정보
        let mut segments = Vec::new();
        for i in 0..result.path.len().saturating_sub(1) {
            let from = result.path[i];
            let to = result.path[i + 1];
            let edge_id = if i < result.edge_ids.len() { result.edge_ids[i] } else { uuid::Uuid::nil() };

            // 엣지 정보에서 distance, max_speed 추출
            let (distance, max_speed) = graph.neighbors(&from).iter()
                .find(|e| e.target_node_id == to)
                .map(|e| (e.distance, e.max_speed))
                .unwrap_or((0.0, 1.0));

            segments.push(PathSegment {
                from_node_id: from.to_string(),
                to_node_id: to.to_string(),
                edge_id: edge_id.to_string(),
                distance,
                max_speed,
                estimated_time: if max_speed > 0.0 { distance / max_speed } else { 0.0 },
            });
        }

        let estimated_time: f32 = segments.iter().map(|s| s.estimated_time).sum();

        // 금지 구역 회피 검사 (optional)
        if req.avoid_no_go_zones {
            // PostGIS 쿼리로 경로가 no_go_zone과 교차하는지 확인
            // 교차 시 Status::failed_precondition 반환 또는 대안 경로 탐색
        }

        let response = PathResult {
            found: true,
            node_ids: result.path.iter().map(|id| id.to_string()).collect(),
            path_points,
            total_distance: result.total_distance,
            total_cost: result.total_cost,
            estimated_time_seconds: estimated_time,
            segments,
        };

        Ok(Response::new(response))
    }
}
```

### 4.6 Python 포인트 클라우드 처리 파이프라인

#### 4.6.1 메인 파이프라인 오케스트레이터 (`processing/pipeline.py`)

```python
"""
포인트 클라우드 처리 파이프라인 메인 오케스트레이터.

실행 순서:
1. Ingestion: 원본 파일 읽기
2. Validation: 포맷, 포인트 수, 좌표 범위 검증
3. Coordinate Normalization: ENU 좌표계로 변환, 원점 근처로 오프셋
4. Downsampling: Voxel grid 필터
5. Filtering: 통계적 이상치 제거
6. Normal Estimation: 노멀 벡터 추정
7. Potree Conversion: PotreeConverter로 옥트리 생성
8. Upload: 생성된 타일을 MinIO에 업로드
9. Metadata: 바운딩 박스, 포인트 수, LOD 레벨 등을 DB에 기록
10. Cleanup: 임시 파일 삭제

사용법:
    python pipeline.py \
        --job-id <uuid> \
        --map-id <uuid> \
        --input-file /tmp/map-manager/<uuid> \
        --format las \
        --voxel-size 0.02 \
        --potree-converter /usr/local/bin/PotreeConverter
"""

import argparse
import logging
import sys
import traceback
from pathlib import Path

import psycopg2

from config import PipelineConfig
from ingest import ingest_pointcloud
from preprocess import voxel_downsample, statistical_outlier_removal
from normals import estimate_normals
from coordinate_system import normalize_coordinates
from metadata_extract import extract_metadata
from potree_convert import run_potree_converter
from s3_upload import upload_potree_tiles

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
)
logger = logging.getLogger("pipeline")


def update_job_status(
    db_url: str,
    job_id: str,
    status: str,
    progress: float,
    current_step: str,
    error_message: str | None = None,
):
    """DB의 processing_jobs 테이블 업데이트."""
    conn = psycopg2.connect(db_url)
    try:
        with conn.cursor() as cur:
            if status == "completed":
                cur.execute(
                    """
                    UPDATE processing_jobs
                    SET status = %s, progress = %s, completed_at = NOW()
                    WHERE id = %s::uuid
                    """,
                    (status, progress, job_id),
                )
            elif status == "failed":
                cur.execute(
                    """
                    UPDATE processing_jobs
                    SET status = %s, progress = %s, error_message = %s
                    WHERE id = %s::uuid
                    """,
                    (status, progress, error_message, job_id),
                )
            else:
                cur.execute(
                    """
                    UPDATE processing_jobs
                    SET status = %s, progress = %s
                    WHERE id = %s::uuid
                    """,
                    (status, progress, job_id),
                )
        conn.commit()
    finally:
        conn.close()


def run_pipeline(config: PipelineConfig):
    """전체 파이프라인 실행."""
    db_url = config.db_url
    job_id = config.job_id

    try:
        # ── Step 1: Ingestion (0-10%) ──
        logger.info("Step 1/9: Ingestion")
        update_job_status(db_url, job_id, "processing", 5.0, "ingestion")
        pcd = ingest_pointcloud(config.input_file, config.format)
        logger.info(f"  Loaded {len(pcd.points)} points")

        # ── Step 2: Validation (10-15%) ──
        logger.info("Step 2/9: Validation")
        update_job_status(db_url, job_id, "processing", 12.0, "validation")
        import numpy as np
        points = np.asarray(pcd.points)
        if len(points) == 0:
            raise ValueError("Point cloud is empty")
        if len(points) > 2_000_000_000:
            raise ValueError(f"Too many points: {len(points)} (max 2B)")

        # 좌표 범위 검증 (극단적인 값 거부)
        coord_max = np.abs(points).max()
        if coord_max > 1e9:
            raise ValueError(f"Coordinate range too large: {coord_max}")

        # ── Step 3: Coordinate Normalization (15-25%) ──
        logger.info("Step 3/9: Coordinate normalization")
        update_job_status(db_url, job_id, "processing", 20.0, "normalization")
        pcd, offset = normalize_coordinates(pcd)
        logger.info(f"  Applied offset: [{offset[0]:.2f}, {offset[1]:.2f}, {offset[2]:.2f}]")

        # ── Step 4: Downsampling (25-40%) ──
        logger.info(f"Step 4/9: Downsampling (voxel={config.voxel_size}m)")
        update_job_status(db_url, job_id, "processing", 30.0, "downsampling")
        original_count = len(pcd.points)
        pcd = voxel_downsample(pcd, config.voxel_size)
        logger.info(f"  {original_count} → {len(pcd.points)} points ({len(pcd.points)/original_count*100:.1f}%)")

        # ── Step 5: Filtering (40-55%) ──
        logger.info("Step 5/9: Statistical outlier removal")
        update_job_status(db_url, job_id, "processing", 45.0, "filtering")
        before_filter = len(pcd.points)
        pcd = statistical_outlier_removal(
            pcd,
            nb_neighbors=config.outlier_neighbors,
            std_ratio=config.outlier_std_ratio,
        )
        logger.info(f"  Removed {before_filter - len(pcd.points)} outliers")

        # ── Step 6: Normal Estimation (55-65%) ──
        logger.info("Step 6/9: Normal estimation")
        update_job_status(db_url, job_id, "processing", 58.0, "normals")
        pcd = estimate_normals(pcd, search_radius=config.normal_search_radius)

        # ── Step 7: Potree Conversion (65-85%) ──
        logger.info("Step 7/9: Potree conversion")
        update_job_status(db_url, job_id, "processing", 70.0, "potree_conversion")

        # 중간 PLY 파일 저장 (PotreeConverter 입력용)
        import open3d as o3d
        intermediate_ply = config.temp_dir / f"{config.job_id}_processed.ply"
        o3d.io.write_point_cloud(str(intermediate_ply), pcd)

        potree_output_dir = config.temp_dir / f"{config.job_id}_potree"
        run_potree_converter(
            input_file=str(intermediate_ply),
            output_dir=str(potree_output_dir),
            converter_path=config.potree_converter_path,
        )

        # ── Step 8: Upload to MinIO (85-95%) ──
        logger.info("Step 8/9: Uploading tiles to MinIO")
        update_job_status(db_url, job_id, "processing", 88.0, "upload")
        tiles_path = f"tiles/{config.map_id}"
        upload_potree_tiles(
            potree_dir=str(potree_output_dir),
            bucket=config.tiles_bucket,
            prefix=tiles_path,
            endpoint=config.s3_endpoint,
            access_key=config.s3_access_key,
            secret_key=config.s3_secret_key,
        )

        # ── Step 9: Metadata (95-100%) ──
        logger.info("Step 9/9: Saving metadata")
        update_job_status(db_url, job_id, "processing", 95.0, "metadata")
        metadata = extract_metadata(pcd, str(potree_output_dir))

        conn = psycopg2.connect(db_url)
        try:
            with conn.cursor() as cur:
                # pointcloud_data 레코드 업데이트
                cur.execute(
                    """
                    UPDATE pointcloud_data
                    SET tiles_path = %s,
                        point_count = %s,
                        lod_levels = %s,
                        density_per_sqm = %s,
                        processing_status = 'completed'
                    WHERE map_id = %s::uuid
                    """,
                    (
                        tiles_path,
                        metadata["point_count"],
                        metadata["lod_levels"],
                        metadata["density_per_sqm"],
                        config.map_id,
                    ),
                )
                # maps 테이블에 bounds, offset 업데이트
                cur.execute(
                    """
                    UPDATE maps
                    SET coordinate_offset = ARRAY[%s, %s, %s],
                        bounds_min = ARRAY[%s, %s, %s],
                        bounds_max = ARRAY[%s, %s, %s],
                        updated_at = NOW()
                    WHERE id = %s::uuid
                    """,
                    (
                        offset[0], offset[1], offset[2],
                        metadata["bounds_min"][0], metadata["bounds_min"][1], metadata["bounds_min"][2],
                        metadata["bounds_max"][0], metadata["bounds_max"][1], metadata["bounds_max"][2],
                        config.map_id,
                    ),
                )
            conn.commit()
        finally:
            conn.close()

        # 완료
        update_job_status(db_url, job_id, "completed", 100.0, "done")
        logger.info("Pipeline completed successfully!")

    except Exception as e:
        logger.error(f"Pipeline failed: {e}")
        logger.error(traceback.format_exc())
        update_job_status(db_url, job_id, "failed", 0.0, "error", str(e))
        sys.exit(1)

    finally:
        # Cleanup
        import shutil
        for path in [
            config.temp_dir / f"{config.job_id}_processed.ply",
            config.temp_dir / f"{config.job_id}_potree",
        ]:
            if path.exists():
                if path.is_dir():
                    shutil.rmtree(path)
                else:
                    path.unlink()


def main():
    parser = argparse.ArgumentParser(description="Point cloud processing pipeline")
    parser.add_argument("--job-id", required=True)
    parser.add_argument("--map-id", required=True)
    parser.add_argument("--input-file", required=True)
    parser.add_argument("--format", required=True, choices=["las", "laz", "ply", "pcd"])
    parser.add_argument("--voxel-size", type=float, default=0.02)
    parser.add_argument("--potree-converter", default="/usr/local/bin/PotreeConverter")
    parser.add_argument("--db-url", default="postgresql://amr:amr@localhost:5432/amr_maps")
    parser.add_argument("--s3-endpoint", default="http://localhost:9000")
    parser.add_argument("--s3-access-key", default="minioadmin")
    parser.add_argument("--s3-secret-key", default="minioadmin")
    parser.add_argument("--tiles-bucket", default="pointcloud-tiles")
    args = parser.parse_args()

    config = PipelineConfig(
        job_id=args.job_id,
        map_id=args.map_id,
        input_file=Path(args.input_file),
        format=args.format,
        voxel_size=args.voxel_size,
        potree_converter_path=args.potree_converter,
        db_url=args.db_url,
        s3_endpoint=args.s3_endpoint,
        s3_access_key=args.s3_access_key,
        s3_secret_key=args.s3_secret_key,
        tiles_bucket=args.tiles_bucket,
        outlier_neighbors=20,
        outlier_std_ratio=2.0,
        normal_search_radius=0.1,
        temp_dir=Path("/tmp/map-manager"),
    )

    run_pipeline(config)


if __name__ == "__main__":
    main()
```

#### 4.6.2 파일 인제스트 (`processing/ingest.py`)

```python
"""
다양한 포인트 클라우드 포맷을 Open3D PointCloud 객체로 변환한다.

지원 포맷:
- LAS/LAZ: laspy로 읽기 (대용량 청크 처리 지원)
- PLY: Open3D 네이티브
- PCD: Open3D 네이티브
"""

import logging
from pathlib import Path

import numpy as np
import open3d as o3d

logger = logging.getLogger("pipeline.ingest")


def ingest_pointcloud(filepath: Path, format: str) -> o3d.geometry.PointCloud:
    """원본 포인트 클라우드 파일을 Open3D PointCloud로 변환."""
    filepath = Path(filepath)
    if not filepath.exists():
        raise FileNotFoundError(f"Input file not found: {filepath}")

    if format in ("las", "laz"):
        return _read_las(filepath)
    elif format == "ply":
        return _read_ply(filepath)
    elif format == "pcd":
        return _read_pcd(filepath)
    else:
        raise ValueError(f"Unsupported format: {format}")


def _read_las(filepath: Path) -> o3d.geometry.PointCloud:
    """LAS/LAZ 파일을 청크 단위로 읽어 메모리 효율적 처리."""
    import laspy

    pcd = o3d.geometry.PointCloud()
    all_points = []
    all_colors = []

    # 대용량 파일을 위한 청크 리더
    chunk_size = 5_000_000  # 500만 포인트씩

    with laspy.open(str(filepath)) as reader:
        logger.info(f"  LAS header: {reader.header.point_count} points, "
                     f"format {reader.header.point_format.id}")

        for chunk in reader.read_points_in_chunks(chunk_size):
            points = np.stack([
                chunk.x, chunk.y, chunk.z
            ], axis=-1).astype(np.float64)
            all_points.append(points)

            # 컬러 정보 (있으면)
            if hasattr(chunk, 'red') and hasattr(chunk, 'green') and hasattr(chunk, 'blue'):
                colors = np.stack([
                    chunk.red, chunk.green, chunk.blue
                ], axis=-1).astype(np.float64) / 65535.0  # 16bit → [0,1]
                all_colors.append(colors)

    all_points = np.concatenate(all_points, axis=0)
    pcd.points = o3d.utility.Vector3dVector(all_points)

    if all_colors:
        all_colors = np.concatenate(all_colors, axis=0)
        pcd.colors = o3d.utility.Vector3dVector(all_colors)

    logger.info(f"  Loaded {len(pcd.points)} points from LAS")
    return pcd


def _read_ply(filepath: Path) -> o3d.geometry.PointCloud:
    """PLY 파일 읽기 (Open3D 네이티브)."""
    pcd = o3d.io.read_point_cloud(str(filepath))
    logger.info(f"  Loaded {len(pcd.points)} points from PLY")
    return pcd


def _read_pcd(filepath: Path) -> o3d.geometry.PointCloud:
    """PCD 파일 읽기 (Open3D 네이티브)."""
    pcd = o3d.io.read_point_cloud(str(filepath))
    logger.info(f"  Loaded {len(pcd.points)} points from PCD")
    return pcd
```

#### 4.6.3 전처리 (`processing/preprocess.py`)

```python
"""
포인트 클라우드 전처리: 다운샘플링, 이상치 제거.
"""

import logging

import numpy as np
import open3d as o3d

logger = logging.getLogger("pipeline.preprocess")


def voxel_downsample(
    pcd: o3d.geometry.PointCloud,
    voxel_size: float = 0.02,
) -> o3d.geometry.PointCloud:
    """
    Voxel grid 다운샘플링.

    각 voxel 내의 포인트를 하나의 대표 포인트(centroid)로 교체한다.
    voxel_size가 작을수록 더 많은 포인트를 유지한다.

    Args:
        pcd: 입력 포인트 클라우드
        voxel_size: 복셀 크기 (미터). 기본 0.02m (디스플레이용).
                    네비게이션용은 0.05m 권장.

    Returns:
        다운샘플링된 포인트 클라우드
    """
    logger.info(f"  Voxel downsample: voxel_size={voxel_size}m, input={len(pcd.points)} points")
    downsampled = pcd.voxel_down_sample(voxel_size=voxel_size)
    logger.info(f"  Output: {len(downsampled.points)} points")
    return downsampled


def statistical_outlier_removal(
    pcd: o3d.geometry.PointCloud,
    nb_neighbors: int = 20,
    std_ratio: float = 2.0,
) -> o3d.geometry.PointCloud:
    """
    통계적 이상치 제거.

    각 포인트에 대해 nb_neighbors개의 최근접 이웃까지의 평균 거리를 계산하고,
    전체 평균 + std_ratio * 표준편차를 초과하는 포인트를 이상치로 제거한다.

    Args:
        pcd: 입력 포인트 클라우드
        nb_neighbors: 이웃 포인트 수 (기본 20)
        std_ratio: 표준편차 배수 (기본 2.0, 작을수록 공격적 제거)

    Returns:
        필터링된 포인트 클라우드
    """
    logger.info(f"  Statistical outlier removal: neighbors={nb_neighbors}, std_ratio={std_ratio}")
    filtered, indices = pcd.remove_statistical_outlier(
        nb_neighbors=nb_neighbors,
        std_ratio=std_ratio,
    )
    removed = len(pcd.points) - len(filtered.points)
    logger.info(f"  Removed {removed} outliers ({removed/len(pcd.points)*100:.2f}%)")
    return filtered
```

#### 4.6.4 좌표 정규화 (`processing/coordinate_system.py`)

```python
"""
좌표계 정규화: ENU(East-North-Up) 좌표계로 변환하고 원점 근처로 오프셋을 적용한다.

대규모 포인트 클라우드는 종종 절대 좌표(UTM 등)를 사용하므로,
부동소수점 정밀도 문제를 피하기 위해 centroid를 원점으로 이동한다.
"""

import logging
from typing import Tuple

import numpy as np
import open3d as o3d

logger = logging.getLogger("pipeline.coordinate_system")


def normalize_coordinates(
    pcd: o3d.geometry.PointCloud,
) -> Tuple[o3d.geometry.PointCloud, np.ndarray]:
    """
    포인트 클라우드의 centroid를 원점으로 이동한다.

    Returns:
        (정규화된 포인트 클라우드, 적용된 오프셋 [x, y, z])
        원본 좌표 = 정규화된 좌표 + 오프셋
    """
    points = np.asarray(pcd.points)

    # Centroid 계산
    centroid = points.mean(axis=0)
    offset = centroid.copy()

    # Z축은 바닥을 0으로 (최소값을 0으로)
    offset[2] = points[:, 2].min()

    # 오프셋 적용 (centroid → 원점)
    normalized_points = points - offset

    pcd.points = o3d.utility.Vector3dVector(normalized_points)

    logger.info(f"  Original centroid: [{centroid[0]:.2f}, {centroid[1]:.2f}, {centroid[2]:.2f}]")
    logger.info(f"  Applied offset: [{offset[0]:.2f}, {offset[1]:.2f}, {offset[2]:.2f}]")
    logger.info(f"  New bounds: [{normalized_points.min(axis=0)}] to [{normalized_points.max(axis=0)}]")

    return pcd, offset
```

#### 4.6.5 노멀 추정 (`processing/normals.py`)

```python
"""
포인트 클라우드 노멀 벡터 추정.
"""

import logging

import open3d as o3d

logger = logging.getLogger("pipeline.normals")


def estimate_normals(
    pcd: o3d.geometry.PointCloud,
    search_radius: float = 0.1,
    max_nn: int = 30,
) -> o3d.geometry.PointCloud:
    """
    각 포인트의 노멀 벡터를 추정한다.

    KD-tree 기반 이웃 검색으로 로컬 평면을 피팅하고 노멀을 계산한다.

    Args:
        pcd: 입력 포인트 클라우드
        search_radius: 검색 반경 (미터, 기본 0.1m)
        max_nn: 최대 이웃 수 (기본 30)

    Returns:
        노멀이 추정된 포인트 클라우드
    """
    logger.info(f"  Estimating normals: radius={search_radius}m, max_nn={max_nn}")

    pcd.estimate_normals(
        search_param=o3d.geometry.KDTreeSearchParamHybrid(
            radius=search_radius,
            max_nn=max_nn,
        )
    )

    # 노멀 방향 일관성 보정 (카메라 방향으로)
    pcd.orient_normals_consistent_tangent_plane(k=15)

    logger.info(f"  Normals estimated for {len(pcd.normals)} points")
    return pcd
```

#### 4.6.6 PotreeConverter 실행 (`processing/potree_convert.py`)

```python
"""
PotreeConverter 2.x를 실행하여 Potree 옥트리 구조를 생성한다.

출력 구조:
  output_dir/
  ├── metadata.json    # 옥트리 메타데이터 (bounds, spacing, hierarchy 정보)
  ├── hierarchy.bin    # 노드 계층 구조 바이너리
  └── octree.bin       # 포인트 데이터 바이너리 (모든 LOD)
"""

import json
import logging
import subprocess
from pathlib import Path

logger = logging.getLogger("pipeline.potree_convert")


def run_potree_converter(
    input_file: str,
    output_dir: str,
    converter_path: str = "/usr/local/bin/PotreeConverter",
) -> dict:
    """
    PotreeConverter를 실행하여 PLY → Potree 옥트리를 생성한다.

    Args:
        input_file: 처리된 PLY 파일 경로
        output_dir: Potree 출력 디렉토리
        converter_path: PotreeConverter 실행 파일 경로

    Returns:
        생성된 metadata.json의 내용 (dict)
    """
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    cmd = [
        converter_path,
        input_file,
        "-o", str(output_path),
        "--generate-page", "false",  # HTML 페이지 생성 안 함
        "--overwrite",
    ]

    logger.info(f"  Running PotreeConverter: {' '.join(cmd)}")

    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        timeout=3600,  # 1시간 타임아웃
    )

    if result.returncode != 0:
        logger.error(f"  PotreeConverter stderr: {result.stderr}")
        raise RuntimeError(f"PotreeConverter failed (exit code {result.returncode}): {result.stderr}")

    logger.info(f"  PotreeConverter stdout: {result.stdout}")

    # 출력 검증
    metadata_path = output_path / "metadata.json"
    if not metadata_path.exists():
        raise RuntimeError("PotreeConverter did not produce metadata.json")

    hierarchy_path = output_path / "hierarchy.bin"
    octree_path = output_path / "octree.bin"
    if not hierarchy_path.exists() or not octree_path.exists():
        raise RuntimeError("PotreeConverter did not produce hierarchy.bin or octree.bin")

    with open(metadata_path) as f:
        metadata = json.load(f)

    logger.info(f"  Potree output: {metadata.get('points', 'unknown')} points, "
                f"hierarchy nodes in hierarchy.bin")

    return metadata
```

#### 4.6.7 메타데이터 추출 (`processing/metadata_extract.py`)

```python
"""
처리된 포인트 클라우드와 Potree 출력에서 메타데이터를 추출한다.
"""

import json
import logging
from pathlib import Path

import numpy as np
import open3d as o3d

logger = logging.getLogger("pipeline.metadata_extract")


def extract_metadata(
    pcd: o3d.geometry.PointCloud,
    potree_output_dir: str,
) -> dict:
    """
    메타데이터를 추출하여 DB 저장에 필요한 딕셔너리를 반환한다.

    Returns:
        {
            "point_count": int,
            "lod_levels": int,
            "density_per_sqm": float,
            "bounds_min": [x, y, z],
            "bounds_max": [x, y, z],
        }
    """
    points = np.asarray(pcd.points)
    bounds_min = points.min(axis=0).tolist()
    bounds_max = points.max(axis=0).tolist()

    # 2D 바닥 면적으로 밀도 계산
    x_range = bounds_max[0] - bounds_min[0]
    y_range = bounds_max[1] - bounds_min[1]
    floor_area = max(x_range * y_range, 1e-6)  # 0 방지
    density = len(points) / floor_area

    # Potree metadata에서 LOD 레벨 추출
    potree_metadata_path = Path(potree_output_dir) / "metadata.json"
    lod_levels = 0
    if potree_metadata_path.exists():
        with open(potree_metadata_path) as f:
            potree_meta = json.load(f)
            # PotreeConverter 2.x의 hierarchy depth
            hierarchy = potree_meta.get("hierarchy", {})
            lod_levels = hierarchy.get("depth", potree_meta.get("hierarchyStepSize", 5))

    metadata = {
        "point_count": len(points),
        "lod_levels": lod_levels,
        "density_per_sqm": round(density, 2),
        "bounds_min": bounds_min,
        "bounds_max": bounds_max,
    }

    logger.info(f"  Metadata: {json.dumps(metadata, indent=2)}")
    return metadata
```

#### 4.6.8 S3 업로드 (`processing/s3_upload.py`)

```python
"""
Potree 옥트리 파일을 MinIO(S3 호환)에 업로드한다.
"""

import logging
import os
from pathlib import Path

import boto3
from botocore.config import Config

logger = logging.getLogger("pipeline.s3_upload")


def upload_potree_tiles(
    potree_dir: str,
    bucket: str,
    prefix: str,
    endpoint: str,
    access_key: str,
    secret_key: str,
):
    """
    Potree 출력 디렉토리의 모든 파일을 MinIO에 업로드한다.

    파일 구조:
        potree_dir/metadata.json    → s3://bucket/prefix/metadata.json
        potree_dir/hierarchy.bin    → s3://bucket/prefix/hierarchy.bin
        potree_dir/octree.bin       → s3://bucket/prefix/octree.bin
    """
    s3 = boto3.client(
        "s3",
        endpoint_url=endpoint,
        aws_access_key_id=access_key,
        aws_secret_access_key=secret_key,
        config=Config(signature_version="s3v4"),
        region_name="us-east-1",
    )

    # 버킷 존재 확인 (없으면 생성)
    try:
        s3.head_bucket(Bucket=bucket)
    except Exception:
        logger.info(f"  Creating bucket: {bucket}")
        s3.create_bucket(Bucket=bucket)

    potree_path = Path(potree_dir)
    uploaded_count = 0

    for file_path in potree_path.rglob("*"):
        if file_path.is_file():
            relative = file_path.relative_to(potree_path)
            s3_key = f"{prefix}/{relative}"

            # MIME type 설정
            content_type = "application/octet-stream"
            if file_path.suffix == ".json":
                content_type = "application/json"

            logger.debug(f"  Uploading {relative} → s3://{bucket}/{s3_key}")

            s3.upload_file(
                str(file_path),
                bucket,
                s3_key,
                ExtraArgs={"ContentType": content_type},
            )
            uploaded_count += 1

    logger.info(f"  Uploaded {uploaded_count} files to s3://{bucket}/{prefix}/")
```

#### 4.6.9 파이프라인 설정 (`processing/config.py`)

```python
"""파이프라인 설정 구조체."""

from dataclasses import dataclass
from pathlib import Path


@dataclass
class PipelineConfig:
    job_id: str
    map_id: str
    input_file: Path
    format: str                       # "las", "laz", "ply", "pcd"
    voxel_size: float                 # 기본 0.02m
    potree_converter_path: str
    db_url: str
    s3_endpoint: str
    s3_access_key: str
    s3_secret_key: str
    tiles_bucket: str
    outlier_neighbors: int = 20
    outlier_std_ratio: float = 2.0
    normal_search_radius: float = 0.1
    temp_dir: Path = Path("/tmp/map-manager")
```

### 4.7 타일 서빙 상세

#### 4.7.1 타일 서빙 흐름

```
클라이언트(Frontend) ──GetTile(map_id, node_id)──► Map Manager
                                                      │
                                          ┌───────────┼───────────┐
                                          ▼           ▼           ▼
                                       Redis      (cache miss)   응답
                                       cache         │
                                     (hit → 응답)    │
                                                     ▼
                                                   MinIO
                                                   (S3 get)
                                                     │
                                          ┌──────────┘
                                          ▼
                                       Redis에 캐시
                                       (TTL 1시간)
                                          │
                                          ▼
                                        응답
```

#### 4.7.2 StreamTiles 상세 로직

```
1. 클라이언트가 카메라 frustum(AABB)과 화면 크기를 전송
2. 서버에서 Potree metadata.json 로드 (옥트리 구조 파악)
3. 옥트리 루트("r")부터 시작하여 DFS 순회
4. 각 노드에 대해:
   a. 노드 bounds가 frustum과 교차하는지 검사
   b. 교차하면:
      - 화면에서의 투영 크기 계산
      - 투영 크기가 임계값보다 크면 → 이 노드의 타일 데이터 전송
      - 자식 노드도 순회 (더 높은 LOD)
   c. 교차하지 않으면 → 이 서브트리 스킵
5. 이미 로드된 노드(loaded_nodes)는 스킵
6. 스트림으로 TileData를 순차 전송
```

---

## 5. 데이터베이스 스키마 상세

### 5.1 전체 마이그레이션 SQL

#### `migrations/001_enable_postgis.sql`

```sql
-- PostGIS 확장 활성화
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS postgis_topology;

-- UUID 생성 함수 (PostgreSQL 13+는 내장)
-- CREATE EXTENSION IF NOT EXISTS "pgcrypto";
```

#### `migrations/002_create_maps.sql`

```sql
CREATE TABLE maps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    coordinate_offset DOUBLE PRECISION[3] DEFAULT ARRAY[0.0, 0.0, 0.0],
    bounds_min DOUBLE PRECISION[3] DEFAULT ARRAY[0.0, 0.0, 0.0],
    bounds_max DOUBLE PRECISION[3] DEFAULT ARRAY[0.0, 0.0, 0.0],
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 이름 검색용 인덱스
CREATE INDEX idx_maps_name ON maps USING gin (name gin_trgm_ops);
-- gin_trgm_ops 사용 시 pg_trgm 확장 필요:
-- CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- 업데이트 시 updated_at 자동 갱신 트리거
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER maps_updated_at
    BEFORE UPDATE ON maps
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

#### `migrations/003_create_pointcloud_data.sql`

```sql
CREATE TABLE pointcloud_data (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    raw_file_path VARCHAR(512),          -- MinIO 원본 파일 경로
    tiles_path VARCHAR(512),             -- MinIO Potree 타일 프리픽스
    point_count BIGINT,                  -- 처리 후 포인트 수
    format VARCHAR(50),                  -- "las", "laz", "ply", "pcd"
    lod_levels INTEGER,                  -- Potree LOD 단계 수
    density_per_sqm REAL,                -- 평방미터당 포인트 밀도
    processing_status VARCHAR(50) NOT NULL DEFAULT 'pending'
        CHECK (processing_status IN ('pending', 'processing', 'completed', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pointcloud_data_map ON pointcloud_data(map_id);
CREATE INDEX idx_pointcloud_data_status ON pointcloud_data(processing_status);
```

#### `migrations/004_create_roadmap.sql`

```sql
-- 로드맵 노드 (경유점)
CREATE TABLE roadmap_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    name VARCHAR(255),
    node_type VARCHAR(50) NOT NULL DEFAULT 'waypoint'
        CHECK (node_type IN (
            'waypoint', 'charging_station', 'loading_dock',
            'elevator', 'waiting_point', 'parking'
        )),
    position geometry(PointZ, 4326) NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_roadmap_nodes_map ON roadmap_nodes(map_id);
CREATE INDEX idx_roadmap_nodes_geom ON roadmap_nodes USING GIST(position);
CREATE INDEX idx_roadmap_nodes_type ON roadmap_nodes(map_id, node_type);

-- 로드맵 엣지 (경로 세그먼트)
CREATE TABLE roadmap_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    source_node_id UUID NOT NULL REFERENCES roadmap_nodes(id) ON DELETE CASCADE,
    target_node_id UUID NOT NULL REFERENCES roadmap_nodes(id) ON DELETE CASCADE,
    distance REAL NOT NULL,                -- 미터 단위 거리
    max_speed REAL NOT NULL DEFAULT 1.0,   -- m/s 최대 속도
    direction VARCHAR(10) NOT NULL DEFAULT 'bi'
        CHECK (direction IN ('uni', 'bi')),
    allowed_robot_types TEXT[] DEFAULT '{}',  -- 허용 로봇 유형
    cost_factor REAL NOT NULL DEFAULT 1.0,   -- 비용 배수 (1.0 = 기본)
    path geometry(LineStringZ, 4326),        -- 경로 시각화용 중간 점
    properties JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 자기 자신으로의 엣지 방지
    CONSTRAINT no_self_loop CHECK (source_node_id != target_node_id)
);

CREATE INDEX idx_roadmap_edges_map ON roadmap_edges(map_id);
CREATE INDEX idx_roadmap_edges_source ON roadmap_edges(source_node_id);
CREATE INDEX idx_roadmap_edges_target ON roadmap_edges(target_node_id);
CREATE INDEX idx_roadmap_edges_path ON roadmap_edges USING GIST(path);
```

#### `migrations/005_create_semantic.sql`

```sql
CREATE TABLE semantic_regions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    name VARCHAR(255),
    region_type VARCHAR(50) NOT NULL
        CHECK (region_type IN (
            'no_go_zone', 'speed_limit_zone', 'charging_area',
            'loading_dock', 'waiting_area', 'one_way_zone', 'restricted_area'
        )),
    geometry geometry(PolygonZ, 4326) NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',
    /*
     * properties 스키마 (region_type별):
     *
     * speed_limit_zone: {"max_speed_mps": 0.5}
     * charging_area:    {"charger_count": 4, "charger_type": "wireless"}
     * loading_dock:     {"dock_type": "pallet", "capacity": 2}
     * one_way_zone:     {"allowed_direction_deg": 90.0}
     * waiting_area:     {"max_robots": 3}
     * restricted_area:  {"allowed_robot_types": ["agv_large"], "schedule": "08:00-18:00"}
     * no_go_zone:       {}
     */
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_semantic_regions_map ON semantic_regions(map_id);
CREATE INDEX idx_semantic_regions_geom ON semantic_regions USING GIST(geometry);
CREATE INDEX idx_semantic_regions_type ON semantic_regions(map_id, region_type);
```

#### `migrations/006_create_obstacles.sql`

```sql
CREATE TABLE obstacles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    obstacle_type VARCHAR(50) NOT NULL DEFAULT 'static'
        CHECK (obstacle_type IN ('static', 'temporary')),
        -- dynamic 장애물은 Redis에만 저장 (PostgreSQL에는 static/temporary만)
    geometry geometry(PolygonZ, 4326) NOT NULL,  -- 2D 바닥 footprint
    dimensions JSONB,   -- {"length": 1.0, "width": 0.5, "height": 1.8}
    properties JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_obstacles_map ON obstacles(map_id);
CREATE INDEX idx_obstacles_geom ON obstacles USING GIST(geometry);
CREATE INDEX idx_obstacles_type ON obstacles(map_id, obstacle_type);
```

#### `migrations/007_create_processing_jobs.sql`

```sql
CREATE TABLE processing_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    job_type VARCHAR(50) NOT NULL
        CHECK (job_type IN ('pointcloud_processing', 'map_export', 'roadmap_import')),
    status VARCHAR(50) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    progress REAL DEFAULT 0.0 CHECK (progress >= 0 AND progress <= 100),
    current_step VARCHAR(100),
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_processing_jobs_map ON processing_jobs(map_id);
CREATE INDEX idx_processing_jobs_status ON processing_jobs(status);
```

### 5.2 유용한 PostGIS 쿼리 예시

```sql
-- 1. 특정 좌표가 포함된 시맨틱 영역 조회 (point-in-polygon)
SELECT id, name, region_type, properties
FROM semantic_regions
WHERE map_id = $1
  AND ST_Contains(geometry, ST_SetSRID(ST_MakePoint($2, $3, $4), 4326));

-- 2. 경로(LineString)와 교차하는 금지 구역 조회
SELECT id, name
FROM semantic_regions
WHERE map_id = $1
  AND region_type = 'no_go_zone'
  AND ST_Intersects(geometry, ST_GeomFromEWKT($2));

-- 3. 바운딩 박스 내 장애물 조회
SELECT id, obstacle_type, dimensions, properties,
       ST_AsGeoJSON(geometry) as geojson
FROM obstacles
WHERE map_id = $1
  AND ST_Intersects(
    geometry,
    ST_MakeEnvelope($2, $3, $4, $5, 4326)  -- xmin, ymin, xmax, ymax
  );

-- 4. 최근접 노드 K개 조회 (KNN)
SELECT id, name, node_type,
       ST_X(position) as x, ST_Y(position) as y, ST_Z(position) as z,
       ST_3DDistance(position, ST_SetSRID(ST_MakePoint($2, $3, $4), 4326)) as dist
FROM roadmap_nodes
WHERE map_id = $1
ORDER BY position <-> ST_SetSRID(ST_MakePoint($2, $3, $4), 4326)
LIMIT $5;

-- 5. 두 노드 간 직선 거리 계산
SELECT ST_3DDistance(
    (SELECT position FROM roadmap_nodes WHERE id = $1),
    (SELECT position FROM roadmap_nodes WHERE id = $2)
) as distance;

-- 6. 특정 영역 내 모든 노드 조회
SELECT id, name, node_type
FROM roadmap_nodes
WHERE map_id = $1
  AND ST_Within(
    position,
    (SELECT geometry FROM semantic_regions WHERE id = $2)
  );

-- 7. 맵의 전체 로드맵 그래프 통계
SELECT
    (SELECT COUNT(*) FROM roadmap_nodes WHERE map_id = $1) as node_count,
    (SELECT COUNT(*) FROM roadmap_edges WHERE map_id = $1) as edge_count,
    (SELECT AVG(distance) FROM roadmap_edges WHERE map_id = $1) as avg_edge_distance,
    (SELECT SUM(distance) FROM roadmap_edges WHERE map_id = $1) as total_edge_distance;
```

### 5.3 Redis 키 스키마

```
# 타일 캐시
tile:{map_id}:{node_id}           → 바이너리 타일 데이터 (TTL: 3600초)

# 동적 장애물
dynamic_obstacle:{map_id}:{obstacle_id}    → JSON 직렬화된 DynamicObstacle (TTL: 10초 기본)
dynamic_obstacles_set:{map_id}             → SET of obstacle_id (만료 정리용)

# 그래프 캐시 (optional, Phase 4)
graph_cache:{map_id}:{version}             → 직렬화된 인메모리 그래프 (TTL: 600초)

# 처리 진행 상태 (optional, 폴링 대신 pub/sub 가능)
processing_progress:{job_id}               → JSON {status, progress, step} (TTL: 86400초)
```

---

## 6. 성능 요구사항

### 6.1 상세 성능 목표

| 항목 | 목표 | 측정 방법 | 비고 |
|------|------|-----------|------|
| 포인트 클라우드 업로드 | 10GB LAS 파일 OOM 없이 처리 | 청크 스트리밍 + 청크 크기 4MB | 피크 메모리 < 2GB |
| Potree 변환 | 500M 포인트 < 30분 | PotreeConverter 벤치마크 | 8코어 CPU 기준 |
| 타일 서빙 (cache miss) | < 10ms p99 | MinIO latency + 네트워크 | 같은 호스트 기준 |
| 타일 서빙 (cache hit) | < 1ms p99 | Redis GET latency | 같은 호스트 기준 |
| A* 경로 탐색 | < 5ms (10K 노드) | 벤치마크 테스트 | 인메모리 그래프 |
| A* 경로 탐색 | < 50ms (100K 노드) | 벤치마크 테스트 | 인메모리 그래프 |
| 공간 쿼리 (point-in-polygon) | < 10ms (1000 영역) | PostGIS GIST 인덱스 | 인덱스 웜업 후 |
| 최근접 노드 조회 | < 5ms | PostGIS KNN | `<->` 연산자 사용 |
| 동적 장애물 업데이트 | < 2ms per obstacle | Redis SET + TTL | |
| gRPC 메시지 크기 | < 16MB | Tonic max_message_size | |

### 6.2 성능 최적화 전략

**포인트 클라우드 처리:**
- LAS 파일을 500만 포인트 청크로 분할 읽기 (laspy chunked reader)
- numpy 벡터 연산으로 좌표 변환 수행
- PotreeConverter의 멀티쓰레드 모드 활용

**타일 서빙:**
- Redis L1 캐시 (TTL 1시간) + MinIO L2 저장소
- 자주 접근하는 상위 LOD 노드(r, r0~r7)를 프리워밍

**경로 탐색:**
- 그래프를 인메모리에 캐시 (맵 버전 변경 시 갱신)
- BinaryHeap 기반 우선순위 큐 (Rust std)
- 노드 수 10K 이하에서 HashMap 룩업 O(1) 보장

**공간 쿼리:**
- PostGIS GIST 인덱스 필수
- 자주 사용하는 쿼리에 대해 prepared statement 사용
- 시맨틱 레이어는 변경이 적으므로 애플리케이션 레벨 캐시 가능

---

## 7. 개발 계획

> 📋 Phase별 상세 일정 및 팀별 할당: [`development-plan.md`](../development-plan.md) 참조
>
> 본 팀의 기능별 상세 스펙은 아래 기능 문서 참조:
> - C-XX: `docs/features/core/C-XX-*.md`
> - A-XX: `docs/features/advanced/A-XX-*.md`

---

## 8. 테스트 계획

### 8.1 단위 테스트 (Rust)

#### 경로 탐색 테스트 (`tests/rust/test_pathfinding.rs`)

```rust
#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::pathfinding::graph::*;
    use crate::pathfinding::astar::*;
    use crate::models::roadmap::*;

    /// 간단한 다이아몬드 그래프에서 A* 정확성 검증
    ///
    ///   A ──(1)──► B ──(1)──► D
    ///   │                     ▲
    ///   └──(2)──► C ──(1)──┘
    ///
    /// A→D 최단 경로: A→B→D (비용 2)
    #[test]
    fn test_astar_diamond_graph() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        let nodes = vec![
            RoadmapNode {
                id: a, map_id: Uuid::nil(), name: Some("A".into()),
                node_type: NodeType::Waypoint,
                position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
                properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
            RoadmapNode {
                id: b, map_id: Uuid::nil(), name: Some("B".into()),
                node_type: NodeType::Waypoint,
                position: Position3D { x: 1.0, y: 1.0, z: 0.0 },
                properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
            RoadmapNode {
                id: c, map_id: Uuid::nil(), name: Some("C".into()),
                node_type: NodeType::Waypoint,
                position: Position3D { x: 1.0, y: -1.0, z: 0.0 },
                properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
            RoadmapNode {
                id: d, map_id: Uuid::nil(), name: Some("D".into()),
                node_type: NodeType::Waypoint,
                position: Position3D { x: 2.0, y: 0.0, z: 0.0 },
                properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
        ];

        let edges = vec![
            RoadmapEdge {
                id: Uuid::new_v4(), map_id: Uuid::nil(),
                source_node_id: a, target_node_id: b,
                distance: 1.0, max_speed: 1.0,
                direction: EdgeDirection::Unidirectional,
                allowed_robot_types: vec![], cost_factor: 1.0,
                path_points: vec![], properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
            RoadmapEdge {
                id: Uuid::new_v4(), map_id: Uuid::nil(),
                source_node_id: a, target_node_id: c,
                distance: 2.0, max_speed: 1.0,
                direction: EdgeDirection::Unidirectional,
                allowed_robot_types: vec![], cost_factor: 1.0,
                path_points: vec![], properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
            RoadmapEdge {
                id: Uuid::new_v4(), map_id: Uuid::nil(),
                source_node_id: b, target_node_id: d,
                distance: 1.0, max_speed: 1.0,
                direction: EdgeDirection::Unidirectional,
                allowed_robot_types: vec![], cost_factor: 1.0,
                path_points: vec![], properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
            RoadmapEdge {
                id: Uuid::new_v4(), map_id: Uuid::nil(),
                source_node_id: c, target_node_id: d,
                distance: 1.0, max_speed: 1.0,
                direction: EdgeDirection::Unidirectional,
                allowed_robot_types: vec![], cost_factor: 1.0,
                path_points: vec![], properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
        ];

        let graph = RoadmapGraphData::build(nodes, edges);
        let result = astar(&graph, a, d, None, 0.0).unwrap();

        assert_eq!(result.path, vec![a, b, d]);
        assert!((result.total_cost - 2.0).abs() < 0.001);
    }

    /// 경로가 없는 경우 에러 반환 확인
    #[test]
    fn test_astar_no_path() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        let nodes = vec![
            RoadmapNode {
                id: a, map_id: Uuid::nil(), name: Some("A".into()),
                node_type: NodeType::Waypoint,
                position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
                properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
            RoadmapNode {
                id: b, map_id: Uuid::nil(), name: Some("B".into()),
                node_type: NodeType::Waypoint,
                position: Position3D { x: 1.0, y: 0.0, z: 0.0 },
                properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
        ];

        let graph = RoadmapGraphData::build(nodes, vec![]);
        let result = astar(&graph, a, b, None, 0.0);

        assert!(result.is_err());
    }

    /// 로봇 타입 필터링 테스트
    #[test]
    fn test_astar_robot_type_filter() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        let nodes = vec![
            RoadmapNode { id: a, map_id: Uuid::nil(), name: Some("A".into()),
                node_type: NodeType::Waypoint,
                position: Position3D { x: 0.0, y: 0.0, z: 0.0 },
                properties: serde_json::json!({}), created_at: chrono::Utc::now() },
            RoadmapNode { id: b, map_id: Uuid::nil(), name: Some("B".into()),
                node_type: NodeType::Waypoint,
                position: Position3D { x: 1.0, y: 0.0, z: 0.0 },
                properties: serde_json::json!({}), created_at: chrono::Utc::now() },
            RoadmapNode { id: c, map_id: Uuid::nil(), name: Some("C".into()),
                node_type: NodeType::Waypoint,
                position: Position3D { x: 2.0, y: 0.0, z: 0.0 },
                properties: serde_json::json!({}), created_at: chrono::Utc::now() },
        ];

        let edges = vec![
            // A→B: agv_small만 허용
            RoadmapEdge {
                id: Uuid::new_v4(), map_id: Uuid::nil(),
                source_node_id: a, target_node_id: b,
                distance: 1.0, max_speed: 1.0,
                direction: EdgeDirection::Unidirectional,
                allowed_robot_types: vec!["agv_small".into()],
                cost_factor: 1.0, path_points: vec![],
                properties: serde_json::json!({}), created_at: chrono::Utc::now(),
            },
            // A→C: 모든 로봇 허용 (빈 배열)
            RoadmapEdge {
                id: Uuid::new_v4(), map_id: Uuid::nil(),
                source_node_id: a, target_node_id: c,
                distance: 3.0, max_speed: 1.0,
                direction: EdgeDirection::Unidirectional,
                allowed_robot_types: vec![], cost_factor: 1.0,
                path_points: vec![], properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
            // B→C
            RoadmapEdge {
                id: Uuid::new_v4(), map_id: Uuid::nil(),
                source_node_id: b, target_node_id: c,
                distance: 1.0, max_speed: 1.0,
                direction: EdgeDirection::Unidirectional,
                allowed_robot_types: vec![], cost_factor: 1.0,
                path_points: vec![], properties: serde_json::json!({}),
                created_at: chrono::Utc::now(),
            },
        ];

        let graph = RoadmapGraphData::build(nodes, edges);

        // agv_large는 A→B 사용 불가 → A→C 직행 (비용 3)
        let result = astar(&graph, a, c, Some("agv_large"), 0.0).unwrap();
        assert_eq!(result.path, vec![a, c]);
        assert!((result.total_cost - 3.0).abs() < 0.001);

        // agv_small은 A→B→C 사용 가능 (비용 2)
        let result = astar(&graph, a, c, Some("agv_small"), 0.0).unwrap();
        assert_eq!(result.path, vec![a, b, c]);
        assert!((result.total_cost - 2.0).abs() < 0.001);
    }
}
```

### 8.2 단위 테스트 (Python)

#### 파이프라인 테스트 (`tests/python/test_pipeline.py`)

```python
"""포인트 클라우드 처리 파이프라인 E2E 테스트."""

import json
import tempfile
from pathlib import Path

import numpy as np
import open3d as o3d
import pytest

from processing.ingest import ingest_pointcloud
from processing.preprocess import voxel_downsample, statistical_outlier_removal
from processing.normals import estimate_normals
from processing.coordinate_system import normalize_coordinates
from processing.metadata_extract import extract_metadata


@pytest.fixture
def sample_pointcloud():
    """10K 포인트의 간단한 방 형태 테스트 포인트 클라우드 생성."""
    pcd = o3d.geometry.PointCloud()

    # 바닥 (5m x 5m)
    floor_points = np.random.uniform([0, 0, 0], [5, 5, 0.01], size=(3000, 3))
    # 벽 4개
    wall1 = np.random.uniform([0, 0, 0], [0.01, 5, 2.5], size=(1750, 3))
    wall2 = np.random.uniform([5, 0, 0], [5.01, 5, 2.5], size=(1750, 3))
    wall3 = np.random.uniform([0, 0, 0], [5, 0.01, 2.5], size=(1750, 3))
    wall4 = np.random.uniform([0, 5, 0], [5, 5.01, 2.5], size=(1750, 3))

    points = np.concatenate([floor_points, wall1, wall2, wall3, wall4])
    pcd.points = o3d.utility.Vector3dVector(points)
    return pcd


@pytest.fixture
def sample_ply_file(sample_pointcloud):
    """임시 PLY 파일로 저장."""
    with tempfile.NamedTemporaryFile(suffix=".ply", delete=False) as f:
        o3d.io.write_point_cloud(f.name, sample_pointcloud)
        return Path(f.name)


class TestIngest:
    def test_read_ply(self, sample_ply_file):
        pcd = ingest_pointcloud(sample_ply_file, "ply")
        assert len(pcd.points) == 10000

    def test_invalid_format(self, sample_ply_file):
        with pytest.raises(ValueError, match="Unsupported format"):
            ingest_pointcloud(sample_ply_file, "xyz")

    def test_file_not_found(self):
        with pytest.raises(FileNotFoundError):
            ingest_pointcloud(Path("/nonexistent/file.ply"), "ply")


class TestPreprocess:
    def test_voxel_downsample(self, sample_pointcloud):
        downsampled = voxel_downsample(sample_pointcloud, voxel_size=0.1)
        assert len(downsampled.points) < len(sample_pointcloud.points)
        assert len(downsampled.points) > 0

    def test_statistical_outlier_removal(self, sample_pointcloud):
        # 이상치 추가
        outliers = np.array([[100, 100, 100], [-50, -50, -50]], dtype=np.float64)
        all_points = np.concatenate([np.asarray(sample_pointcloud.points), outliers])
        sample_pointcloud.points = o3d.utility.Vector3dVector(all_points)

        filtered = statistical_outlier_removal(sample_pointcloud, nb_neighbors=20, std_ratio=2.0)
        assert len(filtered.points) < len(all_points)


class TestCoordinateSystem:
    def test_normalize_coordinates(self, sample_pointcloud):
        pcd, offset = normalize_coordinates(sample_pointcloud)
        points = np.asarray(pcd.points)

        # 정규화 후 centroid가 원점 근처
        centroid = points.mean(axis=0)
        assert abs(centroid[0]) < 1.0
        assert abs(centroid[1]) < 1.0

        # Z 최소값이 0 근처
        assert abs(points[:, 2].min()) < 0.1

    def test_offset_recovery(self, sample_pointcloud):
        original_points = np.asarray(sample_pointcloud.points).copy()
        pcd, offset = normalize_coordinates(sample_pointcloud)
        recovered = np.asarray(pcd.points) + offset

        np.testing.assert_allclose(recovered, original_points, atol=1e-10)


class TestNormals:
    def test_estimate_normals(self, sample_pointcloud):
        pcd = estimate_normals(sample_pointcloud, search_radius=0.5)
        assert len(pcd.normals) == len(pcd.points)

        # 노멀 벡터가 단위 벡터인지 확인
        normals = np.asarray(pcd.normals)
        norms = np.linalg.norm(normals, axis=1)
        np.testing.assert_allclose(norms, 1.0, atol=0.01)


class TestMetadataExtract:
    def test_extract_metadata(self, sample_pointcloud):
        with tempfile.TemporaryDirectory() as tmpdir:
            # 가짜 Potree metadata.json 생성
            potree_meta = {"hierarchy": {"depth": 5}, "points": 10000}
            with open(Path(tmpdir) / "metadata.json", "w") as f:
                json.dump(potree_meta, f)

            metadata = extract_metadata(sample_pointcloud, tmpdir)

            assert metadata["point_count"] == 10000
            assert metadata["lod_levels"] == 5
            assert metadata["density_per_sqm"] > 0
            assert len(metadata["bounds_min"]) == 3
            assert len(metadata["bounds_max"]) == 3
```

### 8.3 통합 테스트

```rust
/// gRPC 통합 테스트 (testcontainers 사용)
///
/// Docker로 PostgreSQL + PostGIS, MinIO, Redis를 실행하고
/// 전체 gRPC 서비스를 테스트한다.
#[cfg(test)]
mod integration_tests {
    use testcontainers::*;
    use tonic::transport::Channel;

    // 전체 업로드 → 처리 → 타일 서빙 흐름 테스트
    #[tokio::test]
    async fn test_full_pointcloud_pipeline() {
        // 1. 테스트 컨테이너 시작 (PostgreSQL + PostGIS, MinIO, Redis)
        // 2. gRPC 서버 시작
        // 3. CreateMap
        // 4. UploadPointCloud (작은 PLY 파일)
        // 5. GetProcessingStatus 폴링 (completed 될 때까지)
        // 6. GetPointCloudMetadata 확인
        // 7. GetTile("r") → 바이너리 데이터 확인
    }

    #[tokio::test]
    async fn test_roadmap_crud_and_pathfinding() {
        // 1. CreateMap
        // 2. AddNode (5개)
        // 3. AddEdge (7개, 양방향 포함)
        // 4. GetRoadmapGraph → 5 노드, 7 엣지
        // 5. FindPath(A, E) → 경로 확인
        // 6. FindNearestNode(임의 좌표) → 가장 가까운 노드
        // 7. DeleteNode → 연쇄 삭제 확인
    }

    #[tokio::test]
    async fn test_semantic_and_obstacles() {
        // 1. CreateMap
        // 2. AddRegion (no_go_zone, speed_limit_zone)
        // 3. AddObstacle (static)
        // 4. GetObstacles(query_bounds) → 영역 내 장애물 확인
        // 5. UpdateDynamicObstacles → Redis 저장 확인
        // 6. GetObstacles(include_dynamic=true) → 동적 장애물 포함
        // 7. TTL 만료 후 → 동적 장애물 사라짐
    }
}
```

### 8.4 성능 벤치마크

```rust
/// A* 벤치마크
/// 실행: cargo bench
#[cfg(test)]
mod benchmarks {
    use criterion::{criterion_group, criterion_main, Criterion};
    use uuid::Uuid;
    use crate::pathfinding::graph::*;
    use crate::pathfinding::astar::*;
    use crate::models::roadmap::*;

    fn generate_grid_graph(size: usize) -> (RoadmapGraphData, Uuid, Uuid) {
        // size x size 격자 그래프 생성
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_ids = Vec::new();

        for i in 0..size {
            for j in 0..size {
                let id = Uuid::new_v4();
                node_ids.push(id);
                nodes.push(RoadmapNode {
                    id,
                    map_id: Uuid::nil(),
                    name: None,
                    node_type: NodeType::Waypoint,
                    position: Position3D {
                        x: i as f64,
                        y: j as f64,
                        z: 0.0,
                    },
                    properties: serde_json::json!({}),
                    created_at: chrono::Utc::now(),
                });

                // 오른쪽 이웃
                if j + 1 < size {
                    // 엣지는 build 시점에 추가
                }
                // 아래쪽 이웃
                if i + 1 < size {
                    // 엣지는 build 시점에 추가
                }
            }
        }

        // 격자 엣지 생성
        for i in 0..size {
            for j in 0..size {
                let idx = i * size + j;
                if j + 1 < size {
                    edges.push(RoadmapEdge {
                        id: Uuid::new_v4(), map_id: Uuid::nil(),
                        source_node_id: node_ids[idx],
                        target_node_id: node_ids[idx + 1],
                        distance: 1.0, max_speed: 1.0,
                        direction: EdgeDirection::Bidirectional,
                        allowed_robot_types: vec![], cost_factor: 1.0,
                        path_points: vec![], properties: serde_json::json!({}),
                        created_at: chrono::Utc::now(),
                    });
                }
                if i + 1 < size {
                    edges.push(RoadmapEdge {
                        id: Uuid::new_v4(), map_id: Uuid::nil(),
                        source_node_id: node_ids[idx],
                        target_node_id: node_ids[(i + 1) * size + j],
                        distance: 1.0, max_speed: 1.0,
                        direction: EdgeDirection::Bidirectional,
                        allowed_robot_types: vec![], cost_factor: 1.0,
                        path_points: vec![], properties: serde_json::json!({}),
                        created_at: chrono::Utc::now(),
                    });
                }
            }
        }

        let start = node_ids[0];
        let goal = node_ids[node_ids.len() - 1];
        let graph = RoadmapGraphData::build(nodes, edges);
        (graph, start, goal)
    }

    fn bench_astar(c: &mut Criterion) {
        // 100x100 = 10,000 노드
        let (graph_10k, start_10k, goal_10k) = generate_grid_graph(100);

        c.bench_function("astar_10k_nodes", |b| {
            b.iter(|| {
                astar(&graph_10k, start_10k, goal_10k, None, 0.0).unwrap()
            })
        });

        // 316x316 ≈ 100,000 노드
        let (graph_100k, start_100k, goal_100k) = generate_grid_graph(316);

        c.bench_function("astar_100k_nodes", |b| {
            b.iter(|| {
                astar(&graph_100k, start_100k, goal_100k, None, 0.0).unwrap()
            })
        });
    }

    criterion_group!(benches, bench_astar);
    criterion_main!(benches);
}
```

---

## 9. 목 데이터

### 9.1 테스트용 포인트 클라우드 생성 스크립트

```python
"""
테스트용 포인트 클라우드 생성 스크립트.
간단한 방(room) 형태의 PLY 파일을 생성한다.

실행: python scripts/seed_test_data.py
"""

import numpy as np
import open3d as o3d
from pathlib import Path


def create_room_pointcloud(
    width: float = 5.0,
    depth: float = 5.0,
    height: float = 2.5,
    density: int = 2000,  # 면당 포인트 수
) -> o3d.geometry.PointCloud:
    """간단한 방 형태의 포인트 클라우드 생성."""
    pcd = o3d.geometry.PointCloud()

    # 바닥
    floor = np.random.uniform(
        [0, 0, 0], [width, depth, 0.001], size=(density, 3)
    )
    # 천장
    ceiling = np.random.uniform(
        [0, 0, height - 0.001], [width, depth, height], size=(density, 3)
    )
    # 벽 4개
    wall_n = np.random.uniform([0, 0, 0], [width, 0.001, height], size=(density, 3))
    wall_s = np.random.uniform([0, depth - 0.001, 0], [width, depth, height], size=(density, 3))
    wall_e = np.random.uniform([width - 0.001, 0, 0], [width, depth, height], size=(density, 3))
    wall_w = np.random.uniform([0, 0, 0], [0.001, depth, height], size=(density, 3))

    all_points = np.concatenate([floor, ceiling, wall_n, wall_s, wall_e, wall_w])

    # 컬러 (면별 다른 색)
    colors = np.concatenate([
        np.tile([0.8, 0.8, 0.8], (density, 1)),  # 바닥: 회색
        np.tile([0.9, 0.9, 0.9], (density, 1)),  # 천장: 밝은 회색
        np.tile([0.7, 0.5, 0.3], (density, 1)),  # 벽: 갈색 계열
        np.tile([0.7, 0.5, 0.3], (density, 1)),
        np.tile([0.7, 0.5, 0.3], (density, 1)),
        np.tile([0.7, 0.5, 0.3], (density, 1)),
    ])

    pcd.points = o3d.utility.Vector3dVector(all_points)
    pcd.colors = o3d.utility.Vector3dVector(colors)

    return pcd


def create_sample_roadmap() -> dict:
    """샘플 로드맵 그래프 (20 노드, 30 엣지)."""
    import uuid

    nodes = []
    # 5x4 격자형 경유점
    for i in range(5):
        for j in range(4):
            node_type = "waypoint"
            if i == 0 and j == 0:
                node_type = "charging_station"
            elif i == 4 and j == 3:
                node_type = "loading_dock"
            elif i == 2 and j == 2:
                node_type = "waiting_point"

            nodes.append({
                "id": str(uuid.uuid4()),
                "name": f"WP_{i}_{j}",
                "node_type": node_type,
                "position": {"x": i * 1.2, "y": j * 1.2, "z": 0.0},
            })

    edges = []
    # 수평 엣지
    for i in range(5):
        for j in range(3):
            src_idx = i * 4 + j
            tgt_idx = i * 4 + j + 1
            edges.append({
                "id": str(uuid.uuid4()),
                "source": nodes[src_idx]["id"],
                "target": nodes[tgt_idx]["id"],
                "distance": 1.2,
                "max_speed": 1.0,
                "direction": "bi",
                "cost_factor": 1.0,
            })

    # 수직 엣지
    for i in range(4):
        for j in range(4):
            src_idx = i * 4 + j
            tgt_idx = (i + 1) * 4 + j
            edges.append({
                "id": str(uuid.uuid4()),
                "source": nodes[src_idx]["id"],
                "target": nodes[tgt_idx]["id"],
                "distance": 1.2,
                "max_speed": 1.0,
                "direction": "bi",
                "cost_factor": 1.0,
            })

    # 대각선 단축 엣지 (일부)
    edges.append({
        "id": str(uuid.uuid4()),
        "source": nodes[0]["id"],
        "target": nodes[5]["id"],
        "distance": 1.7,
        "max_speed": 0.8,
        "direction": "uni",
        "cost_factor": 1.2,
    })

    return {"nodes": nodes, "edges": edges}


def create_sample_semantic_regions() -> list:
    """샘플 시맨틱 영역 (5개)."""
    import uuid
    return [
        {
            "id": str(uuid.uuid4()),
            "name": "Charging Zone A",
            "region_type": "charging_area",
            "vertices": [[0, 0, 0], [1.5, 0, 0], [1.5, 1.5, 0], [0, 1.5, 0], [0, 0, 0]],
            "properties": {"charger_count": 2, "charger_type": "contact"},
        },
        {
            "id": str(uuid.uuid4()),
            "name": "No-Go Zone (Maintenance)",
            "region_type": "no_go_zone",
            "vertices": [[2, 2, 0], [3, 2, 0], [3, 3, 0], [2, 3, 0], [2, 2, 0]],
            "properties": {},
        },
        {
            "id": str(uuid.uuid4()),
            "name": "Speed Limit Corridor",
            "region_type": "speed_limit_zone",
            "vertices": [[0, 1.8, 0], [5, 1.8, 0], [5, 2.2, 0], [0, 2.2, 0], [0, 1.8, 0]],
            "properties": {"max_speed_mps": 0.3},
        },
        {
            "id": str(uuid.uuid4()),
            "name": "Loading Dock B",
            "region_type": "loading_dock",
            "vertices": [[4, 3, 0], [5, 3, 0], [5, 4.5, 0], [4, 4.5, 0], [4, 3, 0]],
            "properties": {"dock_type": "pallet", "capacity": 1},
        },
        {
            "id": str(uuid.uuid4()),
            "name": "One-Way Corridor East",
            "region_type": "one_way_zone",
            "vertices": [[3, 0, 0], [5, 0, 0], [5, 0.5, 0], [3, 0.5, 0], [3, 0, 0]],
            "properties": {"allowed_direction_deg": 0.0},
        },
    ]


if __name__ == "__main__":
    import json

    output_dir = Path("tests/python/fixtures")
    output_dir.mkdir(parents=True, exist_ok=True)

    # 포인트 클라우드
    pcd = create_room_pointcloud()
    o3d.io.write_point_cloud(str(output_dir / "small_room.ply"), pcd)
    print(f"Created small_room.ply ({len(pcd.points)} points)")

    # 로드맵
    roadmap = create_sample_roadmap()
    with open(output_dir / "sample_roadmap.json", "w") as f:
        json.dump(roadmap, f, indent=2)
    print(f"Created sample_roadmap.json ({len(roadmap['nodes'])} nodes, {len(roadmap['edges'])} edges)")

    # 시맨틱 영역
    regions = create_sample_semantic_regions()
    with open(output_dir / "sample_semantic.json", "w") as f:
        json.dump(regions, f, indent=2)
    print(f"Created sample_semantic.json ({len(regions)} regions)")
```

---

## 10. 다른 팀과의 인터페이스 계약

### 10.1 Backend (Team 2) ↔ Map Manager

**프로토콜:** gRPC (Map Manager가 서버, Backend가 클라이언트)

**연결 정보:**
- Host: `map-manager` (Docker 네트워크) 또는 `localhost`
- Port: `50053`
- TLS: 개발 환경에서는 비활성화, 프로덕션에서는 mTLS 적용

**Backend가 호출하는 주요 RPC:**

| RPC | 사용 시나리오 |
|-----|-------------|
| `CreateMap` | 관리자가 새 맵을 생성할 때 |
| `ListMaps` | 맵 목록 조회 (페이지네이션) |
| `UploadPointCloud` | 사용자가 포인트 클라우드 파일을 업로드할 때 |
| `GetProcessingStatus` | 처리 진행 상태 폴링 |
| `GetRoadmapGraph` | 편집기에서 로드맵 그래프 로드 |
| `AddNode/AddEdge` | 편집기에서 노드/엣지 추가 |
| `FindPath` | 로봇에게 경로 할당 시 |
| `GetObstacles` | 시뮬레이션에서 장애물 정보 필요 시 |
| `UpdateDynamicObstacles` | 로봇의 실시간 장애물 감지 데이터 전달 |

### 10.2 Frontend (Team 1) ↔ Map Manager

**Potree 타일 서빙:**

Frontend의 Potree 뷰어는 타일을 다음 URL 패턴으로 요청한다:

```
GET /api/maps/{map_id}/tiles/metadata.json     → Potree 메타데이터
GET /api/maps/{map_id}/tiles/hierarchy.bin      → 계층 구조
GET /api/maps/{map_id}/tiles/octree.bin         → 포인트 데이터 (range request 지원)
```

이 HTTP 요청은 Backend가 프록시하거나, Map Manager가 별도 HTTP 서버를 운영할 수 있다.

**로드맵/시맨틱 JSON 스키마:**

Frontend가 소비하는 JSON 응답 구조:

```json
{
  "roadmap": {
    "nodes": [
      {
        "id": "uuid",
        "name": "WP_01",
        "type": "waypoint",
        "position": {"x": 1.0, "y": 2.0, "z": 0.0}
      }
    ],
    "edges": [
      {
        "id": "uuid",
        "source": "node_uuid",
        "target": "node_uuid",
        "distance": 1.5,
        "direction": "bi",
        "pathPoints": [{"x": 1.0, "y": 2.0, "z": 0.0}, ...]
      }
    ]
  }
}
```

```json
{
  "semanticLayer": {
    "regions": [
      {
        "id": "uuid",
        "name": "No-Go Zone A",
        "type": "no_go_zone",
        "geometry": {
          "type": "Polygon",
          "coordinates": [[[x1,y1,z1], [x2,y2,z2], ...]]
        },
        "properties": {}
      }
    ]
  }
}
```

### 10.3 Sim Engine (Team 3) ↔ Map Manager

**맵 데이터 Export:**

Sim Engine은 시뮬레이션 시나리오 설정 시 Map Manager에서 맵 데이터를 로드한다.

```protobuf
// Sim Engine이 호출하는 RPC
rpc GetMap(GetMapRequest) returns (MapDescriptor);
rpc GetRoadmapGraph(GetRoadmapRequest) returns (RoadmapGraph);
rpc GetSemanticLayer(GetSemanticRequest) returns (SemanticLayer);
rpc GetObstacles(GetObstaclesRequest) returns (ObstacleList);
```

**SDF Export (향후):**

Sim Engine의 물리 엔진(예: Gazebo)을 위한 SDF(Simulation Description Format) 파일 생성:

```xml
<!-- 맵에서 자동 생성되는 SDF 구조 -->
<sdf version="1.9">
  <world name="warehouse_map">
    <include>
      <uri>model://ground_plane</uri>
    </include>

    <!-- 정적 장애물을 SDF 모델로 변환 -->
    <model name="obstacle_1">
      <static>true</static>
      <pose>x y z 0 0 0</pose>
      <link name="link">
        <collision name="collision">
          <geometry>
            <box><size>length width height</size></box>
          </geometry>
        </collision>
        <visual name="visual">
          <geometry>
            <box><size>length width height</size></box>
          </geometry>
        </visual>
      </link>
    </model>
  </world>
</sdf>
```

### 10.4 Asset Manager (Team 6) ↔ Map Manager

**맵 내 에셋 참조 방식:**

맵의 `metadata` JSONB 필드에 에셋 참조를 저장한다:

```json
{
  "assets": {
    "floor_texture": "asset://textures/warehouse_floor_01",
    "wall_material": "asset://materials/concrete_wall",
    "markers": [
      {
        "position": {"x": 1.0, "y": 2.0, "z": 0.0},
        "asset_id": "asset://markers/qr_code_001"
      }
    ]
  }
}
```

Asset Manager의 gRPC를 통해 에셋 메타데이터를 조회:

```protobuf
// Asset Manager에 요청
rpc GetAsset(GetAssetRequest) returns (AssetDescriptor);
```

---

## 11. Docker 및 배포 설정

### 11.1 Dockerfile

```dockerfile
# ── Stage 1: Rust 빌드 ──
FROM rust:1.85-bookworm AS rust-builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY proto/ proto/
COPY src/ src/

RUN cargo build --release

# ── Stage 2: Python 환경 ──
FROM python:3.12-slim-bookworm AS python-env

WORKDIR /app/processing
COPY processing/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# PotreeConverter 설치
RUN apt-get update && apt-get install -y --no-install-recommends \
    wget cmake build-essential libtbb-dev && \
    wget -q https://github.com/potree/PotreeConverter/archive/refs/tags/2.1.1.tar.gz && \
    tar xzf 2.1.1.tar.gz && \
    cd PotreeConverter-2.1.1 && mkdir build && cd build && \
    cmake .. && make -j$(nproc) && \
    cp PotreeConverter /usr/local/bin/ && \
    cd /app && rm -rf PotreeConverter-2.1.1 2.1.1.tar.gz && \
    apt-get purge -y cmake build-essential && apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

# ── Stage 3: 런타임 ──
FROM python:3.12-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 libtbb12 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Python 패키지 복사
COPY --from=python-env /usr/local/lib/python3.12/site-packages /usr/local/lib/python3.12/site-packages
COPY --from=python-env /usr/local/bin/PotreeConverter /usr/local/bin/PotreeConverter

# Rust 바이너리 복사
COPY --from=rust-builder /app/target/release/map-manager /app/map-manager

# Python 처리 스크립트 복사
COPY processing/ /app/processing/
COPY config/ /app/config/
COPY migrations/ /app/migrations/

# 임시 디렉토리 생성
RUN mkdir -p /tmp/map-manager

ENV APP_ENV=production
ENV RUST_LOG=info

EXPOSE 50053

ENTRYPOINT ["/app/map-manager"]
```

### 11.2 docker-compose.dev.yml

```yaml
version: "3.8"

services:
  map-manager:
    build: .
    ports:
      - "50053:50053"
    environment:
      APP_ENV: development
      MAP_MANAGER__DATABASE__URL: postgresql://amr:amr@postgres:5432/amr_maps
      MAP_MANAGER__MINIO__ENDPOINT: http://minio:9000
      MAP_MANAGER__MINIO__ACCESS_KEY: minioadmin
      MAP_MANAGER__MINIO__SECRET_KEY: minioadmin
      MAP_MANAGER__REDIS__URL: redis://redis:6379
      RUST_LOG: debug
    depends_on:
      postgres:
        condition: service_healthy
      minio:
        condition: service_started
      redis:
        condition: service_started
    volumes:
      - ./config:/app/config:ro
      - tile-temp:/tmp/map-manager

  postgres:
    image: postgis/postgis:16-3.4
    environment:
      POSTGRES_USER: amr
      POSTGRES_PASSWORD: amr
      POSTGRES_DB: amr_maps
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U amr"]
      interval: 5s
      timeout: 5s
      retries: 5

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    volumes:
      - miniodata:/data

  redis:
    image: redis:7.2-alpine
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes
    volumes:
      - redisdata:/data

volumes:
  pgdata:
  miniodata:
  redisdata:
  tile-temp:
```

### 11.3 기본 설정 파일 (`config/default.toml`)

```toml
[server]
host = "0.0.0.0"
port = 50053
max_message_size = 16777216  # 16MB

[database]
url = "postgresql://amr:amr@localhost:5432/amr_maps"
max_connections = 20
min_connections = 5

[minio]
endpoint = "http://localhost:9000"
access_key = "minioadmin"
secret_key = "minioadmin"
region = "us-east-1"
raw_bucket = "pointcloud-raw"
tiles_bucket = "pointcloud-tiles"

[redis]
url = "redis://localhost:6379"
tile_cache_ttl_secs = 3600
dynamic_obstacle_default_ttl_secs = 10

[processing]
python_executable = "python3"
pipeline_script = "./processing/pipeline.py"
potree_converter_path = "/usr/local/bin/PotreeConverter"
temp_dir = "/tmp/map-manager"
default_voxel_size = 0.02
outlier_neighbors = 20
outlier_std_ratio = 2.0
normal_search_radius = 0.1
```

---

## 12. 서버 엔트리포인트 (`src/main.rs`)

```rust
use tracing_subscriber::EnvFilter;

mod config;
mod error;
mod generated;
mod grpc;
mod models;
mod pathfinding;
mod processing;
mod storage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 로깅 초기화
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    tracing::info!("Starting Map Manager service");

    // 설정 로드
    let config = config::AppConfig::load()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

    tracing::info!(
        "Config loaded: server={}:{}, db=***",
        config.server.host,
        config.server.port,
    );

    // gRPC 서버 시작
    grpc::server::start_server(config).await?;

    Ok(())
}
```

---

## 13. 보안 고려사항

| 항목 | 대응 |
|------|------|
| gRPC 인증 | 프로덕션에서 mTLS 또는 JWT interceptor 적용 |
| MinIO 접근 | IAM 정책으로 버킷별 접근 제한 |
| SQL Injection | SQLx prepared statement 사용 (파라미터 바인딩) |
| 파일 업로드 | 파일 크기 제한 (10GB), 포맷 검증, 임시 파일 자동 정리 |
| Redis 접근 | 프로덕션에서 AUTH 비밀번호 설정, TLS 연결 |
| 에러 노출 | 내부 에러 메시지를 클라이언트에 그대로 전달하지 않음 |

---

## 14. 모니터링 및 관찰성

| 항목 | 도구 | 메트릭 |
|------|------|--------|
| 로깅 | tracing + JSON 포맷 | 요청 ID, 소요 시간, 에러 |
| 메트릭 | Prometheus (향후) | gRPC 요청 수, 지연 시간, 에러율 |
| 타일 서빙 | 커스텀 메트릭 | 캐시 적중률, MinIO 지연 시간 |
| 처리 파이프라인 | processing_jobs 테이블 | 단계별 소요 시간, 성공/실패율 |
| 헬스체크 | gRPC Health Check Protocol | 서비스 상태 |

---

이 문서는 Map Manager 팀이 전체 모듈을 구현하는 데 필요한 모든 상세를 포함한다. 각 코드 예시는 프로덕션 구현의 기초가 되며, 실제 구현 시 에러 처리와 엣지 케이스를 보강해야 한다.
