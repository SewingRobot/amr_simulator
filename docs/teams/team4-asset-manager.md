# Team 4: Asset Manager

## 1. 팀 개요

### 1.1 팀 명칭
**Asset Manager Team** (팀 4)

### 1.2 담당 범위
로봇 모델, 환경 객체, 센서 정의, 미션 템플릿, 트래픽 규칙 등 **모든 재사용 가능 에셋**의 저장, 변환, 조회, 관리를 총괄한다. 구체적으로 다음 6가지 에셋 유형을 다룬다:

| 에셋 유형 | 설명 |
|---|---|
| `robot_model` | AMR 로봇의 URDF/SDF/glTF 모델 및 물리 파라미터 |
| `static_object` | 벽, 선반, 충전소 등 정적 환경 객체 |
| `dynamic_object` | 문, 컨베이어 벨트 등 동적 환경 객체 |
| `sensor_definition` | LiDAR, 카메라, IMU 등 센서 스펙 정의 |
| `mission_template` | 반복 사용 가능한 미션 패턴 템플릿 |
| `traffic_rule` | 일방통행, 우선순위 등 교통 규칙 정의 |

### 1.3 핵심 목표
1. 다양한 포맷(URDF, SDF, glTF/GLB)의 에셋을 **통합 관리**하고 메타데이터를 일관되게 유지
2. **자동 포맷 변환 파이프라인** 제공 (URDF → glTF, SDF → glTF, URDF ↔ SDF)
3. 메시 최적화(LOD 생성, Draco 압축, 텍스처 아틀라스) 자동화
4. 에셋 유효성 검증(mesh integrity, 참조 무결성, 파일 크기 제한)
5. 버전 관리 및 번들 다운로드 지원

### 1.4 의존 관계

```
┌─────────────┐     gRPC      ┌──────────────┐
│   Backend   │──────────────▶│ Asset Manager│
│  (Team 2)   │   클라이언트    │   (Team 4)   │
└─────────────┘               └──────┬───────┘
                                     │
                    ┌────────────────┼────────────────┐
                    │                │                │
                    ▼                ▼                ▼
              ┌──────────┐   ┌────────────┐   ┌──────────┐
              │  MinIO   │   │ PostgreSQL │   │ Python   │
              │(S3 호환)  │   │ (메타데이터) │   │Converter │
              └──────────┘   └────────────┘   └──────────┘

- Backend (Team 2): gRPC AssetService의 클라이언트. 에셋 CRUD 및 변환 요청
- Sim Engine (Team 3): URDF/SDF 모델 로드, 센서 정의 참조
- Frontend (Team 5): glTF 모델 소비, 메타데이터 JSON 표시
- Map Manager (Team 6): 맵에 배치된 에셋 참조 관리
- MinIO: 바이너리 파일 스토리지 (S3 호환 API)
- PostgreSQL: 에셋 메타데이터 및 변환 작업 상태 저장
```

---

## 2. 기술 스택 상세

### 2.1 Rust 서비스 (Core)

| 항목 | 기술 | 버전 | 용도 |
|---|---|---|---|
| 언어 | Rust | edition 2024 | gRPC 서비스 메인 구현 |
| gRPC | tonic | 0.12+ | gRPC 서버/클라이언트 |
| Protobuf | prost | 0.13+ | protobuf 코드 생성 |
| DB | sqlx | 0.8+ | PostgreSQL async 드라이버 |
| S3 | aws-sdk-s3 | 1.x | MinIO/S3 연동 |
| Serialization | serde, serde_json | 1.x | JSON 직렬화 |
| UUID | uuid | 1.x | 에셋 ID 생성 |
| SHA256 | sha2 | 0.10+ | 파일 체크섬 |
| Config | config | 0.14+ | 환경별 설정 관리 |
| Logging | tracing, tracing-subscriber | 0.1+ | 구조화 로깅 |
| Tokio | tokio | 1.x | 비동기 런타임 |
| Error | thiserror, anyhow | 1.x / 1.x | 에러 처리 |
| Testing | testcontainers | 0.20+ | 통합 테스트용 컨테이너 |

### 2.2 Python 변환 파이프라인

| 항목 | 기술 | 버전 | 용도 |
|---|---|---|---|
| 언어 | Python | 3.12+ | 변환 파이프라인 구현 |
| Mesh | trimesh | 4.x | 메시 로드/처리/최적화 |
| URDF | urdfpy | 0.0.22+ | URDF 파싱 |
| glTF | pygltflib | 1.16+ | glTF 생성/수정 |
| SDF | lxml | 5.x | SDF XML 파싱 |
| Blender | Blender | 4.x | 복잡한 모델 변환 (headless) |
| Validation | gltf-validator (npm) | 2.x | glTF 유효성 검증 (subprocess) |
| Image | Pillow | 10.x | 텍스처 리사이즈/변환 |
| gRPC | grpcio | 1.60+ | Rust 서비스와 내부 통신 |
| Testing | pytest | 8.x | 단위/통합 테스트 |
| NumPy | numpy | 1.26+ | 메시 데이터 처리 |

### 2.3 인프라

| 항목 | 기술 | 용도 |
|---|---|---|
| Container | Docker | 서비스 패키징 |
| Storage | MinIO | S3 호환 오브젝트 스토리지 |
| Database | PostgreSQL 16+ | 메타데이터 저장 |
| Build | cargo, pip | 빌드 도구 |

---

## 3. 디렉토리 구조 상세

```
asset-manager/
├── Cargo.toml                     # Rust 프로젝트 설정
├── build.rs                       # protobuf 컴파일 빌드 스크립트
├── Dockerfile                     # 멀티스테이지 빌드 (Rust + Python)
├── docker-compose.dev.yml         # 개발용 MinIO + PostgreSQL
├── proto/
│   └── asset_service.proto        # gRPC 서비스 정의
├── src/
│   ├── main.rs                    # 서비스 진입점, 서버 시작
│   ├── config.rs                  # 환경 변수 기반 설정 로드
│   ├── error.rs                   # 에러 타입 정의 및 gRPC Status 변환
│   ├── grpc/
│   │   ├── mod.rs                 # gRPC 모듈 공개 인터페이스
│   │   ├── server.rs              # AssetService trait 구현체
│   │   └── handlers.rs            # 각 RPC 메서드 핸들러 로직
│   ├── storage/
│   │   ├── mod.rs                 # 스토리지 모듈 공개 인터페이스
│   │   ├── s3.rs                  # MinIO/S3 업로드/다운로드/삭제
│   │   └── metadata.rs            # PostgreSQL 메타데이터 CRUD
│   ├── models/
│   │   ├── mod.rs                 # 모델 모듈 공개 인터페이스
│   │   ├── asset.rs               # Asset 도메인 모델 (공통)
│   │   ├── robot_model.rs         # 로봇 모델 전용 속성
│   │   ├── sensor_def.rs          # 센서 정의 모델
│   │   ├── environment_object.rs  # 정적/동적 환경 객체
│   │   └── template.rs            # 미션 템플릿 / 트래픽 규칙
│   └── conversion/
│       ├── mod.rs                 # 변환 모듈 공개 인터페이스
│       └── pipeline.rs            # Python 변환 프로세스 트리거 및 상태 관리
├── processing/                    # Python 변환 파이프라인
│   ├── __init__.py
│   ├── converter.py               # 메인 변환 오케스트레이터
│   ├── urdf_parser.py             # URDF → 내부 표현
│   ├── sdf_parser.py              # SDF → 내부 표현
│   ├── gltf_exporter.py           # 내부 표현 → glTF/GLB 내보내기
│   ├── mesh_processor.py          # 메시 단순화, LOD 생성, Draco 압축
│   ├── blender_converter.py       # Blender headless 변환
│   ├── texture_processor.py       # 텍스처 리사이즈, 포맷 변환 (WebP/KTX2)
│   ├── validator.py               # 에셋 유효성 검증
│   ├── models.py                  # Python 내부 데이터 모델 (dataclass)
│   ├── config.py                  # Python 파이프라인 설정
│   ├── requirements.txt           # Python 의존성
│   └── Dockerfile.converter       # Python 변환기 전용 이미지
├── migrations/
│   ├── 001_create_assets.sql      # assets 테이블
│   ├── 002_create_asset_files.sql # asset_files 테이블
│   ├── 003_create_conversions.sql # conversion_jobs 테이블
│   └── 004_create_indexes.sql     # 인덱스 생성
├── tests/
│   ├── rust/
│   │   ├── test_grpc.rs           # gRPC 핸들러 단위 테스트
│   │   ├── test_storage.rs        # S3/메타데이터 단위 테스트
│   │   └── test_integration.rs    # testcontainers 통합 테스트
│   └── python/
│       ├── conftest.py            # pytest fixture 설정
│       ├── test_converter.py      # 변환 오케스트레이터 테스트
│       ├── test_urdf_parser.py    # URDF 파서 테스트
│       ├── test_sdf_parser.py     # SDF 파서 테스트
│       ├── test_gltf_exporter.py  # glTF 내보내기 테스트
│       ├── test_mesh_processor.py # 메시 처리 테스트
│       ├── test_validator.py      # 유효성 검증 테스트
│       └── fixtures/              # 테스트 데이터
│           ├── sample_robot.urdf
│           ├── sample_world.sdf
│           ├── sample_model.glb
│           ├── sample_mesh.stl
│           ├── sample_texture.png
│           ├── sensor_lidar2d.json
│           ├── sensor_camera.json
│           ├── mission_patrol.json
│           └── traffic_oneway.json
└── scripts/
    ├── init_minio.sh              # MinIO 초기 버킷 생성
    └── run_migration.sh           # DB 마이그레이션 실행
```

---

## 4. 구성 모듈 상세 스펙

### 4.1 gRPC Service Interface

#### 4.1.1 Proto 정의 전체

```protobuf
syntax = "proto3";
package amr.asset;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

// ============================================================
// AssetService: 에셋 관리의 모든 기능을 제공하는 핵심 서비스
// ============================================================
service AssetService {
  // 에셋 목록 조회 (필터링, 페이징, 정렬 지원)
  rpc ListAssets(ListAssetsRequest) returns (ListAssetsResponse);

  // 단일 에셋 상세 조회
  rpc GetAsset(GetAssetRequest) returns (AssetDescriptor);

  // 에셋 생성 (스트리밍 업로드: 메타데이터 + 파일 청크)
  rpc CreateAsset(stream CreateAssetRequest) returns (AssetDescriptor);

  // 에셋 메타데이터 업데이트
  rpc UpdateAssetMetadata(UpdateAssetRequest) returns (AssetDescriptor);

  // 에셋 삭제 (관련 파일 포함)
  rpc DeleteAsset(DeleteAssetRequest) returns (DeleteAssetResponse);

  // 에셋 파일 다운로드 (스트리밍)
  rpc DownloadAsset(DownloadAssetRequest) returns (stream AssetChunk);

  // 포맷 변환 요청 (비동기)
  rpc ConvertAsset(ConvertAssetRequest) returns (ConversionJob);

  // 변환 작업 상태 조회
  rpc GetConversionStatus(GetConversionStatusRequest) returns (ConversionStatus);

  // 에셋 번들 다운로드 (모든 포맷 + 메타데이터를 ZIP으로)
  rpc GetAssetBundle(GetAssetBundleRequest) returns (stream AssetChunk);

  // 에셋 유효성 검증
  rpc ValidateAsset(ValidateAssetRequest) returns (ValidationResult);
}

// ============================================================
// Common Enums
// ============================================================
enum AssetType {
  ASSET_TYPE_UNSPECIFIED = 0;
  ASSET_TYPE_ROBOT_MODEL = 1;
  ASSET_TYPE_STATIC_OBJECT = 2;
  ASSET_TYPE_DYNAMIC_OBJECT = 3;
  ASSET_TYPE_SENSOR_DEFINITION = 4;
  ASSET_TYPE_MISSION_TEMPLATE = 5;
  ASSET_TYPE_TRAFFIC_RULE = 6;
}

enum AssetFormat {
  ASSET_FORMAT_UNSPECIFIED = 0;
  ASSET_FORMAT_URDF = 1;
  ASSET_FORMAT_SDF = 2;
  ASSET_FORMAT_GLTF = 3;
  ASSET_FORMAT_GLB = 4;
  ASSET_FORMAT_STL = 5;
  ASSET_FORMAT_DAE = 6;
  ASSET_FORMAT_OBJ = 7;
  ASSET_FORMAT_JSON = 8;
}

enum ConversionStatusEnum {
  CONVERSION_STATUS_UNSPECIFIED = 0;
  CONVERSION_STATUS_PENDING = 1;
  CONVERSION_STATUS_PROCESSING = 2;
  CONVERSION_STATUS_COMPLETED = 3;
  CONVERSION_STATUS_FAILED = 4;
}

enum SortOrder {
  SORT_ORDER_UNSPECIFIED = 0;
  SORT_ORDER_ASC = 1;
  SORT_ORDER_DESC = 2;
}

// ============================================================
// Message Types
// ============================================================

// --- ListAssets ---
message ListAssetsRequest {
  AssetType type_filter = 1;            // 선택적 유형 필터
  repeated string tags = 2;             // 태그 필터 (AND 조건)
  string name_pattern = 3;              // 이름 패턴 (LIKE 검색, 예: "%AMR%")
  int32 page_size = 4;                  // 페이지 크기 (기본 20, 최대 100)
  string page_token = 5;               // 다음 페이지 토큰 (base64 encoded cursor)
  string sort_by = 6;                   // 정렬 필드: "name", "created_at", "updated_at"
  SortOrder sort_order = 7;            // 정렬 방향
}

message ListAssetsResponse {
  repeated AssetSummary assets = 1;    // 에셋 요약 목록
  string next_page_token = 2;          // 다음 페이지 토큰 (없으면 마지막 페이지)
  int32 total_count = 3;               // 전체 매칭 개수
}

message AssetSummary {
  string id = 1;                        // UUID
  AssetType type = 2;
  string name = 3;
  string description = 4;
  repeated string tags = 5;
  repeated AssetFormat available_formats = 6;  // 사용 가능한 포맷 목록
  int32 version = 7;
  google.protobuf.Timestamp created_at = 8;
  google.protobuf.Timestamp updated_at = 9;
}

// --- GetAsset ---
message GetAssetRequest {
  string id = 1;                        // 에셋 UUID
  int32 version = 2;                    // 선택적 버전 (0이면 최신)
}

message AssetDescriptor {
  string id = 1;
  AssetType type = 2;
  string name = 3;
  string description = 4;
  google.protobuf.Struct properties = 5;  // 유형별 속성 (JSONB)
  repeated string tags = 6;
  repeated AssetFileInfo files = 7;       // 관련 파일 목록
  int32 version = 8;
  google.protobuf.Timestamp created_at = 9;
  google.protobuf.Timestamp updated_at = 10;
}

message AssetFileInfo {
  string file_id = 1;                   // 파일 UUID
  AssetFormat format = 2;
  string filename = 3;                  // 원본 파일명
  int64 size_bytes = 4;
  string checksum_sha256 = 5;
  string storage_path = 6;             // MinIO 내 경로
  google.protobuf.Timestamp created_at = 7;
}

// --- CreateAsset ---
message CreateAssetRequest {
  oneof payload {
    CreateAssetMetadata metadata = 1;   // 첫 번째 메시지: 메타데이터
    AssetChunk chunk = 2;               // 후속 메시지: 파일 청크
  }
}

message CreateAssetMetadata {
  AssetType type = 1;
  string name = 2;
  string description = 3;
  google.protobuf.Struct properties = 4;
  repeated string tags = 5;
  string filename = 6;                  // 업로드할 파일의 원본 이름
  AssetFormat format = 7;              // 업로드 파일 포맷
}

message AssetChunk {
  bytes data = 1;                       // 파일 청크 데이터 (최대 64KB)
  int64 offset = 2;                     // 청크 오프셋
}

// --- UpdateAsset ---
message UpdateAssetRequest {
  string id = 1;
  string name = 2;                      // 빈 문자열이면 변경 안 함
  string description = 3;
  google.protobuf.Struct properties = 4;  // null이면 변경 안 함
  repeated string tags = 5;
  bool replace_tags = 6;               // true: 태그 교체, false: 태그 추가
}

// --- DeleteAsset ---
message DeleteAssetRequest {
  string id = 1;
  bool delete_all_versions = 2;        // true: 모든 버전 삭제
}

message DeleteAssetResponse {
  bool success = 1;
  int32 files_deleted = 2;             // 삭제된 파일 수
}

// --- DownloadAsset ---
message DownloadAssetRequest {
  string id = 1;
  AssetFormat format = 2;              // 원하는 포맷
  int32 version = 3;                   // 0이면 최신
}

// --- ConvertAsset ---
message ConvertAssetRequest {
  string asset_id = 1;
  AssetFormat target_format = 2;
  ConversionOptions options = 3;
}

message ConversionOptions {
  int32 max_triangles = 1;            // 최대 삼각형 수 (0이면 제한 없음)
  int32 max_texture_size = 2;         // 최대 텍스처 크기 (픽셀, 0이면 제한 없음)
  bool enable_draco = 3;              // Draco 압축 활성화
  int32 draco_compression_level = 4;  // Draco 압축 레벨 (1-10, 기본 7)
  bool generate_lod = 5;             // LOD 레벨 자동 생성
  int32 lod_levels = 6;              // LOD 레벨 수 (기본 3)
  bool embed_textures = 7;           // 텍스처를 GLB에 임베드
  string texture_format = 8;         // "webp", "ktx2", "png" (기본 "webp")
}

message ConversionJob {
  string job_id = 1;
  string asset_id = 2;
  AssetFormat source_format = 3;
  AssetFormat target_format = 4;
  ConversionStatusEnum status = 5;
  google.protobuf.Timestamp created_at = 6;
}

// --- GetConversionStatus ---
message GetConversionStatusRequest {
  string job_id = 1;
}

message ConversionStatus {
  string job_id = 1;
  ConversionStatusEnum status = 2;
  float progress = 3;                 // 0.0 ~ 1.0
  string error_message = 4;           // 실패 시 에러 메시지
  string result_file_id = 5;          // 완료 시 결과 파일 ID
  google.protobuf.Timestamp created_at = 6;
  google.protobuf.Timestamp completed_at = 7;
}

// --- GetAssetBundle ---
message GetAssetBundleRequest {
  string id = 1;
  repeated AssetFormat formats = 2;   // 포함할 포맷 (빈 배열이면 모두)
  int32 version = 3;
}

// --- ValidateAsset ---
message ValidateAssetRequest {
  string id = 1;
  AssetFormat format = 2;             // 검증할 특정 포맷 (UNSPECIFIED이면 모두)
}

message ValidationResult {
  bool is_valid = 1;
  repeated ValidationIssue errors = 2;
  repeated ValidationIssue warnings = 3;
  ValidationStats stats = 4;
}

message ValidationIssue {
  string code = 1;                    // 예: "MESH_NON_MANIFOLD", "URDF_MISSING_LINK"
  string message = 2;
  string file_path = 3;              // 문제가 발견된 파일
  string severity = 4;               // "error" | "warning"
}

message ValidationStats {
  int32 total_vertices = 1;
  int32 total_triangles = 2;
  int32 total_textures = 3;
  int64 total_size_bytes = 4;
  BoundingBox bounding_box = 5;
}

message BoundingBox {
  float min_x = 1;
  float min_y = 2;
  float min_z = 3;
  float max_x = 4;
  float max_y = 5;
  float max_z = 6;
}
```

### 4.2 Asset Types and Schemas

#### 4.2.1 Robot Model (`robot_model`)

로봇 모델은 AMR의 기구학적, 물리적, 시각적 속성을 모두 포함한다.

**메타데이터 JSON 스키마 (properties JSONB):**
```json
{
  "dimensions": {
    "length": 0.5,
    "width": 0.3,
    "height": 0.2,
    "unit": "meters"
  },
  "weight_kg": 25.0,
  "max_payload_kg": 50.0,
  "max_speed_mps": 2.0,
  "max_acceleration_mps2": 1.5,
  "max_angular_velocity_rps": 3.14,
  "drive_type": "differential",
  "wheel_config": {
    "wheel_radius": 0.05,
    "wheel_width": 0.02,
    "wheel_separation": 0.3,
    "caster_offset": 0.04
  },
  "joints": [
    {
      "name": "wheel_left_joint",
      "type": "continuous",
      "parent_link": "base_link",
      "child_link": "wheel_left_link",
      "axis": [0, 0, 1],
      "origin": {"xyz": [0.0, 0.15, 0.0], "rpy": [0.0, 0.0, 0.0]},
      "limits": {
        "effort": 10.0,
        "velocity": 6.28
      }
    }
  ],
  "sensors": [
    {
      "name": "front_lidar",
      "sensor_def_id": "uuid-of-sensor-definition",
      "type": "lidar_2d",
      "pose": {
        "position": {"x": 0.2, "y": 0.0, "z": 0.15},
        "orientation": {"roll": 0.0, "pitch": 0.0, "yaw": 0.0}
      },
      "mount_link": "base_link"
    },
    {
      "name": "front_camera",
      "sensor_def_id": "uuid-of-camera-def",
      "type": "camera_rgb",
      "pose": {
        "position": {"x": 0.22, "y": 0.0, "z": 0.18},
        "orientation": {"roll": 0.0, "pitch": 0.0, "yaw": 0.0}
      },
      "mount_link": "base_link"
    }
  ],
  "collision_shapes": [
    {
      "name": "base_collision",
      "type": "box",
      "dimensions": {"x": 0.5, "y": 0.3, "z": 0.15},
      "origin": {"xyz": [0.0, 0.0, 0.075], "rpy": [0.0, 0.0, 0.0]}
    }
  ],
  "inertial": {
    "mass": 25.0,
    "origin": {"xyz": [0.0, 0.0, 0.05], "rpy": [0.0, 0.0, 0.0]},
    "inertia": {
      "ixx": 0.5, "ixy": 0.0, "ixz": 0.0,
      "iyy": 0.3, "iyz": 0.0, "izz": 0.4
    }
  },
  "battery": {
    "capacity_wh": 500,
    "voltage": 48.0,
    "charging_rate_w": 200
  },
  "footprint_polygon": [
    {"x": 0.25, "y": 0.15},
    {"x": -0.25, "y": 0.15},
    {"x": -0.25, "y": -0.15},
    {"x": 0.25, "y": -0.15}
  ]
}
```

#### 4.2.2 Static Object (`static_object`)

벽, 선반, 충전소 등 움직이지 않는 환경 요소.

```json
{
  "category": "furniture",
  "sub_category": "shelf",
  "dimensions": {
    "length": 2.0,
    "width": 0.6,
    "height": 2.0,
    "unit": "meters"
  },
  "weight_kg": 80.0,
  "material": "steel",
  "color": {"r": 0.7, "g": 0.7, "b": 0.7, "a": 1.0},
  "is_navigable": false,
  "is_intractable": false,
  "collision_shapes": [
    {
      "type": "box",
      "dimensions": {"x": 2.0, "y": 0.6, "z": 2.0},
      "origin": {"xyz": [0.0, 0.0, 1.0], "rpy": [0.0, 0.0, 0.0]}
    }
  ],
  "snap_points": [
    {
      "name": "front_center",
      "position": {"x": 0.0, "y": -0.3, "z": 0.0},
      "orientation": {"roll": 0.0, "pitch": 0.0, "yaw": 1.5708},
      "purpose": "robot_docking"
    }
  ],
  "physics": {
    "static_friction": 0.5,
    "dynamic_friction": 0.3,
    "restitution": 0.1
  }
}
```

#### 4.2.3 Dynamic Object (`dynamic_object`)

문, 컨베이어 벨트, 엘리베이터 등 상태가 변하는 환경 요소.

```json
{
  "category": "door",
  "sub_category": "sliding_door",
  "dimensions": {
    "length": 1.2,
    "width": 0.1,
    "height": 2.1,
    "unit": "meters"
  },
  "weight_kg": 30.0,
  "states": [
    {
      "name": "closed",
      "joint_positions": {"door_slide_joint": 0.0},
      "is_default": true
    },
    {
      "name": "open",
      "joint_positions": {"door_slide_joint": 1.2}
    }
  ],
  "joints": [
    {
      "name": "door_slide_joint",
      "type": "prismatic",
      "axis": [1, 0, 0],
      "limits": {
        "lower": 0.0,
        "upper": 1.2,
        "effort": 100.0,
        "velocity": 0.5
      }
    }
  ],
  "triggers": [
    {
      "type": "proximity",
      "radius": 2.0,
      "action": "open",
      "auto_close_delay_sec": 5.0
    }
  ],
  "collision_shapes": [
    {
      "type": "box",
      "dimensions": {"x": 1.2, "y": 0.1, "z": 2.1},
      "origin": {"xyz": [0.0, 0.0, 1.05], "rpy": [0.0, 0.0, 0.0]}
    }
  ],
  "physics": {
    "static_friction": 0.4,
    "dynamic_friction": 0.2,
    "restitution": 0.0
  }
}
```

#### 4.2.4 Sensor Definition (`sensor_definition`)

센서의 물리적 스펙과 시뮬레이션 파라미터를 정의한다.

```json
{
  "sensor_type": "lidar_2d",
  "manufacturer": "SICK",
  "model_number": "TIM571",
  "specifications": {
    "range_min": 0.05,
    "range_max": 25.0,
    "angle_min": -2.356,
    "angle_max": 2.356,
    "angle_increment": 0.00436,
    "scan_frequency_hz": 15,
    "samples_per_scan": 1081,
    "accuracy_mm": 30,
    "range_noise_stddev": 0.01
  },
  "physical": {
    "dimensions": {"length": 0.06, "width": 0.06, "height": 0.086},
    "weight_kg": 0.25,
    "power_consumption_w": 4.0,
    "interface": "ethernet",
    "ip_default": "192.168.0.1",
    "data_port": 2112
  },
  "simulation_params": {
    "update_rate_hz": 15,
    "ray_count": 1081,
    "noise_type": "gaussian",
    "noise_stddev": 0.01,
    "min_intensity": 0.0,
    "max_intensity": 1.0,
    "visualize_rays": false
  },
  "ros_config": {
    "topic_name": "/scan",
    "frame_id": "laser_frame",
    "message_type": "sensor_msgs/LaserScan"
  }
}
```

**Camera RGB 센서:**
```json
{
  "sensor_type": "camera_rgb",
  "manufacturer": "Intel",
  "model_number": "RealSense D435",
  "specifications": {
    "resolution_width": 1920,
    "resolution_height": 1080,
    "fov_horizontal_deg": 69.4,
    "fov_vertical_deg": 42.5,
    "fps": 30,
    "depth_enabled": true,
    "depth_range_min": 0.1,
    "depth_range_max": 10.0,
    "depth_resolution_width": 1280,
    "depth_resolution_height": 720
  },
  "physical": {
    "dimensions": {"length": 0.09, "width": 0.025, "height": 0.025},
    "weight_kg": 0.072,
    "power_consumption_w": 2.5,
    "interface": "usb3"
  },
  "simulation_params": {
    "update_rate_hz": 30,
    "image_format": "rgb8",
    "depth_format": "float32",
    "noise_type": "gaussian",
    "noise_stddev": 0.005,
    "lens_distortion": {
      "k1": 0.0, "k2": 0.0, "k3": 0.0,
      "p1": 0.0, "p2": 0.0
    },
    "clip_near": 0.1,
    "clip_far": 100.0
  },
  "ros_config": {
    "image_topic": "/camera/color/image_raw",
    "depth_topic": "/camera/depth/image_rect_raw",
    "camera_info_topic": "/camera/color/camera_info",
    "frame_id": "camera_link"
  }
}
```

**IMU 센서:**
```json
{
  "sensor_type": "imu",
  "manufacturer": "Bosch",
  "model_number": "BNO055",
  "specifications": {
    "accelerometer_range_g": 16,
    "gyroscope_range_dps": 2000,
    "magnetometer_range_ut": 1300,
    "update_rate_hz": 100
  },
  "physical": {
    "dimensions": {"length": 0.02, "width": 0.02, "height": 0.005},
    "weight_kg": 0.003,
    "power_consumption_w": 0.05,
    "interface": "i2c"
  },
  "simulation_params": {
    "update_rate_hz": 100,
    "accel_noise_stddev": 0.02,
    "gyro_noise_stddev": 0.001,
    "accel_bias_stddev": 0.001,
    "gyro_bias_stddev": 0.0001
  },
  "ros_config": {
    "topic_name": "/imu/data",
    "frame_id": "imu_link",
    "message_type": "sensor_msgs/Imu"
  }
}
```

#### 4.2.5 Mission Template (`mission_template`)

반복 사용 가능한 미션 패턴을 정의한다.

```json
{
  "template_name": "warehouse_patrol",
  "description": "창고 내 정해진 경로를 순찰하는 미션 템플릿",
  "category": "patrol",
  "required_robot_capabilities": ["navigation", "lidar_2d"],
  "parameters": [
    {
      "name": "patrol_points",
      "type": "array",
      "item_type": "pose2d",
      "description": "순찰 지점 리스트",
      "required": true,
      "min_items": 2,
      "max_items": 50
    },
    {
      "name": "loop_count",
      "type": "integer",
      "description": "반복 횟수 (0이면 무한)",
      "required": false,
      "default": 0,
      "min": 0,
      "max": 1000
    },
    {
      "name": "wait_time_sec",
      "type": "float",
      "description": "각 지점에서 대기 시간(초)",
      "required": false,
      "default": 2.0,
      "min": 0.0,
      "max": 300.0
    },
    {
      "name": "speed_factor",
      "type": "float",
      "description": "속도 배율 (1.0 = 기본 속도)",
      "required": false,
      "default": 1.0,
      "min": 0.1,
      "max": 2.0
    }
  ],
  "task_sequence": [
    {
      "step": 1,
      "action": "navigate_to",
      "params": {"target": "${patrol_points[i]}"},
      "on_failure": "skip_and_continue",
      "timeout_sec": 120
    },
    {
      "step": 2,
      "action": "wait",
      "params": {"duration_sec": "${wait_time_sec}"},
      "on_failure": "continue",
      "timeout_sec": "${wait_time_sec + 5}"
    },
    {
      "step": 3,
      "action": "loop",
      "params": {"goto_step": 1, "iterations": "${loop_count}"}
    }
  ],
  "abort_conditions": [
    {"type": "battery_low", "threshold": 15},
    {"type": "obstacle_stuck", "timeout_sec": 60},
    {"type": "manual_cancel"}
  ],
  "completion_actions": [
    {"action": "navigate_to_charger"},
    {"action": "send_report"}
  ],
  "estimated_duration_sec": 3600,
  "priority": "normal"
}
```

#### 4.2.6 Traffic Rule (`traffic_rule`)

에셋으로 저장되는 교통 규칙 정의.

```json
{
  "rule_name": "corridor_oneway",
  "description": "복도 구간 일방통행 규칙",
  "rule_type": "one_way",
  "priority": 10,
  "conditions": {
    "applicable_zones": ["zone_corridor_a", "zone_corridor_b"],
    "applicable_robot_types": ["all"],
    "time_window": {
      "start": "00:00",
      "end": "23:59",
      "days": ["mon", "tue", "wed", "thu", "fri", "sat", "sun"]
    }
  },
  "parameters": {
    "direction": {
      "type": "vector2d",
      "value": {"x": 1.0, "y": 0.0}
    },
    "max_speed_mps": 1.0,
    "min_following_distance_m": 1.5,
    "yield_to": ["emergency_robot"]
  },
  "enforcement": {
    "mode": "strict",
    "violation_action": "reroute",
    "notification": true,
    "log_violations": true
  },
  "visual_indicators": {
    "floor_marking_color": {"r": 0.0, "g": 0.0, "b": 1.0, "a": 0.5},
    "arrow_spacing_m": 2.0,
    "display_in_frontend": true
  }
}
```

### 4.3 Storage Layer

#### 4.3.1 MinIO Bucket 구조

```
amr-assets/                           # 메인 버킷
├── robot_model/
│   ├── {uuid}/
│   │   ├── v1/
│   │   │   ├── urdf/
│   │   │   │   ├── robot.urdf
│   │   │   │   └── meshes/
│   │   │   │       ├── base_link.stl
│   │   │   │       ├── wheel_left.stl
│   │   │   │       └── wheel_right.stl
│   │   │   ├── sdf/
│   │   │   │   └── model.sdf
│   │   │   └── gltf/
│   │   │       ├── robot.gltf
│   │   │       ├── robot.bin
│   │   │       └── textures/
│   │   │           └── base_color.webp
│   │   └── v2/
│   │       └── ...
├── static_object/
│   └── {uuid}/v1/...
├── dynamic_object/
│   └── {uuid}/v1/...
├── sensor_definition/
│   └── {uuid}/v1/
│       └── definition.json
├── mission_template/
│   └── {uuid}/v1/
│       └── template.json
└── traffic_rule/
    └── {uuid}/v1/
        └── rule.json
```

#### 4.3.2 S3 Operations (Rust)

```rust
// src/storage/s3.rs

use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use tokio::io::AsyncRead;
use sha2::{Sha256, Digest};

/// MinIO/S3 스토리지 클라이언트 래퍼
/// 모든 에셋 바이너리 파일의 업로드, 다운로드, 삭제를 담당한다.
pub struct AssetStorage {
    client: S3Client,
    bucket: String,
}

/// 업로드 결과 정보
pub struct UploadResult {
    pub storage_path: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub etag: String,
}

/// 다운로드 스트림 청크
pub struct DownloadChunk {
    pub data: Vec<u8>,
    pub offset: u64,
}

impl AssetStorage {
    /// 새 AssetStorage 인스턴스 생성
    /// MinIO 엔드포인트, 자격증명, 버킷명은 Config에서 주입받는다.
    pub fn new(client: S3Client, bucket: String) -> Self {
        Self { client, bucket }
    }

    /// 스토리지 경로 생성 규칙:
    /// `{asset_type}/{asset_id}/v{version}/{format}/{filename}`
    pub fn build_storage_path(
        asset_type: &str,
        asset_id: &str,
        version: i32,
        format: &str,
        filename: &str,
    ) -> String {
        format!("{}/{}/v{}/{}/{}", asset_type, asset_id, version, format, filename)
    }

    /// 파일 업로드 (스트리밍)
    /// - 5MB 미만: 단일 PutObject
    /// - 5MB 이상: Multipart Upload (파트 크기 5MB)
    /// 업로드 중 SHA256 체크섬을 계산하여 반환한다.
    pub async fn upload(
        &self,
        storage_path: &str,
        data: Vec<u8>,
    ) -> Result<UploadResult, StorageError> {
        let size = data.len() as i64;
        let checksum = Self::compute_sha256(&data);

        if size < 5 * 1024 * 1024 {
            // 단일 업로드
            self.client
                .put_object()
                .bucket(&self.bucket)
                .key(storage_path)
                .body(ByteStream::from(data))
                .send()
                .await
                .map_err(StorageError::S3Error)?;
        } else {
            // Multipart 업로드
            self.multipart_upload(storage_path, &data).await?;
        }

        Ok(UploadResult {
            storage_path: storage_path.to_string(),
            size_bytes: size,
            checksum_sha256: checksum,
            etag: String::new(), // S3에서 반환된 ETag
        })
    }

    /// 스트리밍 업로드: gRPC 스트림에서 직접 받아 MinIO에 저장
    /// 메모리 사용을 최소화하기 위해 청크 단위로 처리한다.
    pub async fn upload_stream(
        &self,
        storage_path: &str,
        mut stream: impl futures::Stream<Item = Result<Vec<u8>, tonic::Status>> + Unpin,
    ) -> Result<UploadResult, StorageError> {
        let mut hasher = Sha256::new();
        let mut total_size: i64 = 0;
        let mut buffer = Vec::new();

        // Multipart 업로드 시작
        let multipart = self.client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
            .map_err(StorageError::S3Error)?;

        let upload_id = multipart.upload_id().unwrap();
        let mut parts: Vec<CompletedPart> = Vec::new();
        let mut part_number = 1;
        let part_size = 5 * 1024 * 1024; // 5MB

        use futures::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(StorageError::GrpcError)?;
            hasher.update(&chunk);
            total_size += chunk.len() as i64;
            buffer.extend_from_slice(&chunk);

            while buffer.len() >= part_size {
                let part_data: Vec<u8> = buffer.drain(..part_size).collect();
                let upload_part = self.client
                    .upload_part()
                    .bucket(&self.bucket)
                    .key(storage_path)
                    .upload_id(upload_id)
                    .part_number(part_number)
                    .body(ByteStream::from(part_data))
                    .send()
                    .await
                    .map_err(StorageError::S3Error)?;

                parts.push(
                    CompletedPart::builder()
                        .part_number(part_number)
                        .e_tag(upload_part.e_tag().unwrap_or_default())
                        .build(),
                );
                part_number += 1;
            }
        }

        // 남은 데이터 업로드
        if !buffer.is_empty() {
            let upload_part = self.client
                .upload_part()
                .bucket(&self.bucket)
                .key(storage_path)
                .upload_id(upload_id)
                .part_number(part_number)
                .body(ByteStream::from(buffer))
                .send()
                .await
                .map_err(StorageError::S3Error)?;

            parts.push(
                CompletedPart::builder()
                    .part_number(part_number)
                    .e_tag(upload_part.e_tag().unwrap_or_default())
                    .build(),
            );
        }

        // Multipart 업로드 완료
        let completed = CompletedMultipartUpload::builder()
            .set_parts(Some(parts))
            .build();

        self.client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(storage_path)
            .upload_id(upload_id)
            .multipart_upload(completed)
            .send()
            .await
            .map_err(StorageError::S3Error)?;

        let checksum = format!("{:x}", hasher.finalize());

        Ok(UploadResult {
            storage_path: storage_path.to_string(),
            size_bytes: total_size,
            checksum_sha256: checksum,
            etag: String::new(),
        })
    }

    /// 파일 다운로드 (전체)
    pub async fn download(&self, storage_path: &str) -> Result<Vec<u8>, StorageError> {
        let output = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
            .map_err(StorageError::S3Error)?;

        let data = output.body.collect().await
            .map_err(StorageError::StreamError)?
            .into_bytes()
            .to_vec();

        Ok(data)
    }

    /// 파일 스트리밍 다운로드 (청크 단위 반환)
    /// gRPC 스트리밍 응답에 직접 연결할 수 있도록 Iterator 반환
    pub async fn download_stream(
        &self,
        storage_path: &str,
        chunk_size: usize, // 권장: 65536 (64KB)
    ) -> Result<impl futures::Stream<Item = Result<DownloadChunk, StorageError>>, StorageError> {
        let output = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
            .map_err(StorageError::S3Error)?;

        let stream = output.body;
        let mut offset: u64 = 0;

        Ok(async_stream::stream! {
            let mut reader = stream.into_async_read();
            let mut buf = vec![0u8; chunk_size];
            loop {
                use tokio::io::AsyncReadExt;
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        yield Ok(DownloadChunk {
                            data: buf[..n].to_vec(),
                            offset,
                        });
                        offset += n as u64;
                    }
                    Err(e) => {
                        yield Err(StorageError::IoError(e));
                        break;
                    }
                }
            }
        })
    }

    /// 파일 삭제
    pub async fn delete(&self, storage_path: &str) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
            .map_err(StorageError::S3Error)?;
        Ok(())
    }

    /// 접두사(prefix)로 여러 파일 삭제 (에셋 전체 삭제 시 사용)
    pub async fn delete_prefix(&self, prefix: &str) -> Result<u32, StorageError> {
        let mut count = 0u32;
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self.client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix);

            if let Some(token) = &continuation_token {
                request = request.continuation_token(token);
            }

            let output = request.send().await.map_err(StorageError::S3Error)?;

            if let Some(objects) = output.contents() {
                for obj in objects {
                    if let Some(key) = obj.key() {
                        self.delete(key).await?;
                        count += 1;
                    }
                }
            }

            if output.is_truncated() == Some(true) {
                continuation_token = output.next_continuation_token().map(String::from);
            } else {
                break;
            }
        }

        Ok(count)
    }

    /// 파일 존재 여부 확인
    pub async fn exists(&self, storage_path: &str) -> Result<bool, StorageError> {
        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// SHA256 체크섬 계산
    fn compute_sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Multipart 업로드 내부 구현
    async fn multipart_upload(
        &self,
        storage_path: &str,
        data: &[u8],
    ) -> Result<(), StorageError> {
        let part_size = 5 * 1024 * 1024; // 5MB
        let multipart = self.client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
            .map_err(StorageError::S3Error)?;

        let upload_id = multipart.upload_id().unwrap();
        let mut parts = Vec::new();
        let mut part_number = 1;

        for chunk in data.chunks(part_size) {
            let upload_part = self.client
                .upload_part()
                .bucket(&self.bucket)
                .key(storage_path)
                .upload_id(upload_id)
                .part_number(part_number)
                .body(ByteStream::from(chunk.to_vec()))
                .send()
                .await
                .map_err(StorageError::S3Error)?;

            parts.push(
                CompletedPart::builder()
                    .part_number(part_number)
                    .e_tag(upload_part.e_tag().unwrap_or_default())
                    .build(),
            );
            part_number += 1;
        }

        let completed = CompletedMultipartUpload::builder()
            .set_parts(Some(parts))
            .build();

        self.client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(storage_path)
            .upload_id(upload_id)
            .multipart_upload(completed)
            .send()
            .await
            .map_err(StorageError::S3Error)?;

        Ok(())
    }
}

/// 스토리지 에러 타입
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("S3 operation failed: {0}")]
    S3Error(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Stream error: {0}")]
    StreamError(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::Status),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File not found: {0}")]
    NotFound(String),
}
```

#### 4.3.3 Metadata CRUD (Rust)

```rust
// src/storage/metadata.rs

use sqlx::{PgPool, Row, FromRow, types::Json};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// 에셋 메타데이터 데이터베이스 접근 레이어
/// PostgreSQL JSONB를 활용하여 유형별로 유연한 속성을 저장한다.
pub struct MetadataStore {
    pool: PgPool,
}

/// 에셋 레코드 (DB 행 매핑)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetRecord {
    pub id: Uuid,
    pub r#type: String,
    pub name: String,
    pub description: Option<String>,
    pub properties: Json<serde_json::Value>,
    pub tags: Vec<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 에셋 파일 레코드
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetFileRecord {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub format: String,
    pub storage_path: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub created_at: DateTime<Utc>,
}

/// 변환 작업 레코드
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversionJobRecord {
    pub id: Uuid,
    pub source_asset_id: Uuid,
    pub target_format: String,
    pub status: String,
    pub progress: f32,
    pub error_message: Option<String>,
    pub result_file_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// 목록 조회 필터
pub struct ListFilter {
    pub type_filter: Option<String>,
    pub tags: Vec<String>,
    pub name_pattern: Option<String>,
    pub page_size: i32,
    pub offset: i64,
    pub sort_by: String,
    pub sort_order: String, // "ASC" | "DESC"
}

impl MetadataStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Asset CRUD ──────────────────────────────────────────

    /// 에셋 생성
    pub async fn create_asset(
        &self,
        r#type: &str,
        name: &str,
        description: Option<&str>,
        properties: serde_json::Value,
        tags: &[String],
    ) -> Result<AssetRecord, sqlx::Error> {
        let record = sqlx::query_as::<_, AssetRecord>(
            r#"
            INSERT INTO assets (type, name, description, properties, tags)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, type, name, description, properties, tags, version, created_at, updated_at
            "#,
        )
        .bind(r#type)
        .bind(name)
        .bind(description)
        .bind(Json(properties))
        .bind(tags)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// 에셋 조회 (ID)
    pub async fn get_asset(&self, id: Uuid) -> Result<Option<AssetRecord>, sqlx::Error> {
        let record = sqlx::query_as::<_, AssetRecord>(
            r#"
            SELECT id, type, name, description, properties, tags, version, created_at, updated_at
            FROM assets
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// 에셋 조회 (ID + 버전)
    pub async fn get_asset_version(
        &self,
        id: Uuid,
        version: i32,
    ) -> Result<Option<AssetRecord>, sqlx::Error> {
        let record = sqlx::query_as::<_, AssetRecord>(
            r#"
            SELECT id, type, name, description, properties, tags, version, created_at, updated_at
            FROM assets
            WHERE id = $1 AND version = $2
            "#,
        )
        .bind(id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// 에셋 목록 조회 (필터링 + 페이징)
    pub async fn list_assets(
        &self,
        filter: &ListFilter,
    ) -> Result<(Vec<AssetRecord>, i64), sqlx::Error> {
        // 동적 쿼리 빌드
        let mut conditions = vec!["1=1".to_string()];
        let mut bind_index = 1;

        if let Some(ref type_filter) = filter.type_filter {
            conditions.push(format!("type = ${}", bind_index));
            bind_index += 1;
        }

        if !filter.tags.is_empty() {
            conditions.push(format!("tags @> ${}", bind_index));
            bind_index += 1;
        }

        if let Some(ref name_pattern) = filter.name_pattern {
            conditions.push(format!("name ILIKE ${}", bind_index));
            bind_index += 1;
        }

        let where_clause = conditions.join(" AND ");

        // 정렬 필드 검증 (SQL injection 방지)
        let sort_by = match filter.sort_by.as_str() {
            "name" => "name",
            "created_at" => "created_at",
            "updated_at" => "updated_at",
            "type" => "type",
            _ => "created_at",
        };

        let sort_order = match filter.sort_order.as_str() {
            "ASC" => "ASC",
            _ => "DESC",
        };

        let query = format!(
            r#"
            SELECT id, type, name, description, properties, tags, version, created_at, updated_at
            FROM assets
            WHERE {}
            ORDER BY {} {}
            LIMIT ${}
            OFFSET ${}
            "#,
            where_clause,
            sort_by,
            sort_order,
            bind_index,
            bind_index + 1,
        );

        let count_query = format!(
            "SELECT COUNT(*) as count FROM assets WHERE {}",
            where_clause
        );

        // 참고: 실제 구현에서는 sqlx의 QueryBuilder 또는 동적 바인딩을 사용해야 한다.
        // 위 코드는 구조를 보여주기 위한 의사 코드이다.

        todo!("동적 쿼리 바인딩 구현 필요 - sqlx::QueryBuilder 사용 권장")
    }

    /// 에셋 메타데이터 업데이트
    pub async fn update_asset(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        properties: Option<serde_json::Value>,
        tags: Option<&[String]>,
        replace_tags: bool,
    ) -> Result<AssetRecord, sqlx::Error> {
        // 트랜잭션으로 업데이트 수행
        let mut tx = self.pool.begin().await?;

        if let Some(name) = name {
            sqlx::query("UPDATE assets SET name = $1, updated_at = NOW() WHERE id = $2")
                .bind(name)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        if let Some(description) = description {
            sqlx::query("UPDATE assets SET description = $1, updated_at = NOW() WHERE id = $2")
                .bind(description)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        if let Some(properties) = properties {
            sqlx::query(
                "UPDATE assets SET properties = properties || $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(Json(properties))
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }

        if let Some(tags) = tags {
            if replace_tags {
                sqlx::query("UPDATE assets SET tags = $1, updated_at = NOW() WHERE id = $2")
                    .bind(tags)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            } else {
                // 기존 태그에 추가 (중복 제거)
                sqlx::query(
                    r#"
                    UPDATE assets
                    SET tags = (SELECT array_agg(DISTINCT t) FROM unnest(tags || $1) AS t),
                        updated_at = NOW()
                    WHERE id = $2
                    "#,
                )
                .bind(tags)
                .bind(id)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        // 업데이트된 레코드 반환
        self.get_asset(id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    /// 에셋 삭제
    pub async fn delete_asset(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM assets WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 에셋 버전 증가 (새 버전 생성)
    pub async fn increment_version(&self, id: Uuid) -> Result<i32, sqlx::Error> {
        let row = sqlx::query(
            r#"
            UPDATE assets SET version = version + 1, updated_at = NOW()
            WHERE id = $1
            RETURNING version
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("version"))
    }

    // ── Asset File CRUD ─────────────────────────────────────

    /// 에셋 파일 레코드 생성
    pub async fn create_asset_file(
        &self,
        asset_id: Uuid,
        format: &str,
        storage_path: &str,
        size_bytes: i64,
        checksum_sha256: &str,
    ) -> Result<AssetFileRecord, sqlx::Error> {
        let record = sqlx::query_as::<_, AssetFileRecord>(
            r#"
            INSERT INTO asset_files (asset_id, format, storage_path, size_bytes, checksum_sha256)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, asset_id, format, storage_path, size_bytes, checksum_sha256, created_at
            "#,
        )
        .bind(asset_id)
        .bind(format)
        .bind(storage_path)
        .bind(size_bytes)
        .bind(checksum_sha256)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// 에셋의 모든 파일 조회
    pub async fn list_asset_files(
        &self,
        asset_id: Uuid,
    ) -> Result<Vec<AssetFileRecord>, sqlx::Error> {
        let records = sqlx::query_as::<_, AssetFileRecord>(
            r#"
            SELECT id, asset_id, format, storage_path, size_bytes, checksum_sha256, created_at
            FROM asset_files
            WHERE asset_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// 특정 포맷의 파일 조회
    pub async fn get_asset_file_by_format(
        &self,
        asset_id: Uuid,
        format: &str,
    ) -> Result<Option<AssetFileRecord>, sqlx::Error> {
        let record = sqlx::query_as::<_, AssetFileRecord>(
            r#"
            SELECT id, asset_id, format, storage_path, size_bytes, checksum_sha256, created_at
            FROM asset_files
            WHERE asset_id = $1 AND format = $2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(asset_id)
        .bind(format)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// 에셋 파일 삭제
    pub async fn delete_asset_file(&self, file_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM asset_files WHERE id = $1")
            .bind(file_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ── Conversion Job CRUD ─────────────────────────────────

    /// 변환 작업 생성
    pub async fn create_conversion_job(
        &self,
        source_asset_id: Uuid,
        target_format: &str,
    ) -> Result<ConversionJobRecord, sqlx::Error> {
        let record = sqlx::query_as::<_, ConversionJobRecord>(
            r#"
            INSERT INTO conversion_jobs (source_asset_id, target_format)
            VALUES ($1, $2)
            RETURNING id, source_asset_id, target_format, status, progress, error_message,
                      result_file_id, created_at, completed_at
            "#,
        )
        .bind(source_asset_id)
        .bind(target_format)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// 변환 작업 상태 조회
    pub async fn get_conversion_job(
        &self,
        job_id: Uuid,
    ) -> Result<Option<ConversionJobRecord>, sqlx::Error> {
        let record = sqlx::query_as::<_, ConversionJobRecord>(
            r#"
            SELECT id, source_asset_id, target_format, status, progress, error_message,
                   result_file_id, created_at, completed_at
            FROM conversion_jobs
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await
        .ok();

        Ok(record)
    }

    /// 변환 작업 상태 업데이트
    pub async fn update_conversion_status(
        &self,
        job_id: Uuid,
        status: &str,
        progress: f32,
        error_message: Option<&str>,
        result_file_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        let completed_at = if status == "completed" || status == "failed" {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE conversion_jobs
            SET status = $1, progress = $2, error_message = $3,
                result_file_id = $4, completed_at = $5
            WHERE id = $6
            "#,
        )
        .bind(status)
        .bind(progress)
        .bind(error_message)
        .bind(result_file_id)
        .bind(completed_at)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

#### 4.3.4 버전 관리 정책

- **새 파일 업로드 시**: 현재 버전의 `storage_path`에 저장. 같은 포맷의 기존 파일이 있으면 `asset_files` 레코드를 교체하되 MinIO의 기존 파일은 유지.
- **명시적 버전 증가**: `increment_version` 호출 시 `version` 필드 +1, 새 스토리지 경로 (`v{N+1}/`) 사용.
- **이전 버전 접근**: `GetAsset`에서 `version` 파라미터로 특정 버전 조회 가능.
- **전체 삭제**: `delete_all_versions=true`이면 모든 버전의 MinIO 파일 + DB 레코드 삭제.

### 4.4 Conversion Pipeline

#### 4.4.1 아키텍처 개요

Rust 서비스가 변환 요청을 받으면 Python 변환 프로세스를 subprocess로 실행한다. 변환 진행 상태는 DB의 `conversion_jobs` 테이블을 통해 추적한다.

```
┌─────────────┐    ConvertAsset RPC    ┌───────────────┐
│   Backend   │───────────────────────▶│  Rust Service  │
└─────────────┘                        └───────┬───────┘
                                               │
                                    1. DB에 job 생성 (pending)
                                    2. Python subprocess 실행
                                               │
                                               ▼
                                       ┌──────────────┐
                                       │   Python     │
                                       │  converter   │
                                       └──────┬───────┘
                                              │
                              ┌───────────────┼───────────────┐
                              │               │               │
                              ▼               ▼               ▼
                        ┌──────────┐   ┌──────────┐   ┌──────────┐
                        │  URDF    │   │   SDF    │   │  Blender │
                        │  Parser  │   │  Parser  │   │ Headless │
                        └────┬─────┘   └────┬─────┘   └────┬─────┘
                             │              │              │
                             └──────────────┼──────────────┘
                                            │
                                            ▼
                                     ┌──────────────┐
                                     │ Mesh         │
                                     │ Processor    │
                                     │ (simplify,   │
                                     │  LOD, Draco) │
                                     └──────┬───────┘
                                            │
                                            ▼
                                     ┌──────────────┐
                                     │ glTF         │
                                     │ Exporter     │
                                     └──────┬───────┘
                                            │
                                  3. MinIO에 결과 업로드
                                  4. DB status → completed
```

#### 4.4.2 Rust 변환 트리거

```rust
// src/conversion/pipeline.rs

use std::process::Command;
use tokio::process::Command as AsyncCommand;
use uuid::Uuid;
use crate::storage::metadata::MetadataStore;

/// Python 변환 파이프라인을 관리하는 구조체
pub struct ConversionPipeline {
    metadata: MetadataStore,
    python_bin: String,          // Python 인터프리터 경로
    converter_script: String,    // converter.py 경로
    work_dir: String,            // 임시 작업 디렉토리
}

impl ConversionPipeline {
    pub fn new(
        metadata: MetadataStore,
        python_bin: String,
        converter_script: String,
        work_dir: String,
    ) -> Self {
        Self {
            metadata,
            python_bin,
            converter_script,
            work_dir,
        }
    }

    /// 비동기 변환 작업 시작
    /// 1. DB에 conversion_job 생성 (pending)
    /// 2. 백그라운드 태스크로 Python subprocess 실행
    /// 3. job_id 즉시 반환 (클라이언트는 GetConversionStatus로 폴링)
    pub async fn start_conversion(
        &self,
        asset_id: Uuid,
        target_format: &str,
        options: ConversionOptions,
    ) -> Result<Uuid, ConversionError> {
        // 소스 에셋 존재 여부 확인
        let asset = self.metadata.get_asset(asset_id).await?
            .ok_or(ConversionError::AssetNotFound(asset_id))?;

        // 소스 포맷의 파일 존재 여부 확인
        let source_files = self.metadata.list_asset_files(asset_id).await?;
        if source_files.is_empty() {
            return Err(ConversionError::NoSourceFile(asset_id));
        }

        // 변환 작업 레코드 생성
        let job = self.metadata
            .create_conversion_job(asset_id, target_format)
            .await?;

        let job_id = job.id;

        // 변환 옵션을 JSON으로 직렬화
        let options_json = serde_json::to_string(&options)
            .map_err(ConversionError::SerializationError)?;

        // 백그라운드에서 Python 프로세스 실행
        let python_bin = self.python_bin.clone();
        let converter_script = self.converter_script.clone();
        let work_dir = self.work_dir.clone();
        let metadata = self.metadata.clone();

        tokio::spawn(async move {
            // 상태를 processing으로 업데이트
            let _ = metadata
                .update_conversion_status(job_id, "processing", 0.0, None, None)
                .await;

            // Python subprocess 실행
            let result = AsyncCommand::new(&python_bin)
                .arg(&converter_script)
                .arg("--job-id").arg(job_id.to_string())
                .arg("--asset-id").arg(asset_id.to_string())
                .arg("--target-format").arg(target_format)
                .arg("--options").arg(&options_json)
                .arg("--work-dir").arg(&work_dir)
                .arg("--db-url").arg(std::env::var("DATABASE_URL").unwrap_or_default())
                .arg("--s3-endpoint").arg(std::env::var("S3_ENDPOINT").unwrap_or_default())
                .arg("--s3-bucket").arg(std::env::var("S3_BUCKET").unwrap_or_default())
                .output()
                .await;

            match result {
                Ok(output) => {
                    if output.status.success() {
                        // Python 스크립트가 DB를 직접 업데이트하므로
                        // 여기서는 추가 처리 불필요
                        tracing::info!("Conversion job {} completed successfully", job_id);
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::error!("Conversion job {} failed: {}", job_id, stderr);
                        let _ = metadata
                            .update_conversion_status(
                                job_id,
                                "failed",
                                0.0,
                                Some(&stderr),
                                None,
                            )
                            .await;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to start conversion process: {}", e);
                    let _ = metadata
                        .update_conversion_status(
                            job_id,
                            "failed",
                            0.0,
                            Some(&e.to_string()),
                            None,
                        )
                        .await;
                }
            }
        });

        Ok(job_id)
    }
}

/// 변환 옵션 (gRPC ConversionOptions 대응)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionOptions {
    pub max_triangles: u32,
    pub max_texture_size: u32,
    pub enable_draco: bool,
    pub draco_compression_level: u32,
    pub generate_lod: bool,
    pub lod_levels: u32,
    pub embed_textures: bool,
    pub texture_format: String,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            max_triangles: 100_000,
            max_texture_size: 2048,
            enable_draco: true,
            draco_compression_level: 7,
            generate_lod: false,
            lod_levels: 3,
            embed_textures: true,
            texture_format: "webp".to_string(),
        }
    }
}

/// 변환 에러 타입
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Asset not found: {0}")]
    AssetNotFound(Uuid),

    #[error("No source file for asset: {0}")]
    NoSourceFile(Uuid),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[source] serde_json::Error),

    #[error("Process error: {0}")]
    ProcessError(String),
}
```

#### 4.4.3 Python 변환 오케스트레이터

```python
# processing/converter.py
"""
메인 변환 오케스트레이터.
Rust 서비스에서 subprocess로 호출되며, 변환 전 과정을 조율한다.
DB에 직접 진행 상태를 업데이트한다.
"""

import argparse
import json
import sys
import os
import tempfile
import shutil
from pathlib import Path
from uuid import UUID
from dataclasses import dataclass

import psycopg2
import boto3

from urdf_parser import URDFParser
from sdf_parser import SDFParser
from gltf_exporter import GLTFExporter
from mesh_processor import MeshProcessor
from texture_processor import TextureProcessor
from validator import AssetValidator
from models import ConversionContext, ConversionResult


@dataclass
class ConversionConfig:
    """변환 작업 설정"""
    job_id: str
    asset_id: str
    target_format: str
    options: dict
    work_dir: str
    db_url: str
    s3_endpoint: str
    s3_bucket: str


class Converter:
    """
    변환 파이프라인의 메인 오케스트레이터.

    지원하는 변환 경로:
    - URDF → glTF/GLB
    - URDF → SDF
    - SDF → glTF/GLB
    - STL/DAE/OBJ → glTF/GLB

    변환 과정:
    1. MinIO에서 소스 파일 다운로드
    2. 소스 포맷 파싱
    3. 메시 처리 (단순화, LOD)
    4. 텍스처 처리 (리사이즈, 포맷 변환)
    5. 타겟 포맷으로 내보내기
    6. 결과 파일을 MinIO에 업로드
    7. DB 업데이트 (완료/실패)
    """

    def __init__(self, config: ConversionConfig):
        self.config = config
        self.db_conn = psycopg2.connect(config.db_url)
        self.s3_client = boto3.client(
            's3',
            endpoint_url=config.s3_endpoint,
            aws_access_key_id=os.environ.get('AWS_ACCESS_KEY_ID', 'minioadmin'),
            aws_secret_access_key=os.environ.get('AWS_SECRET_ACCESS_KEY', 'minioadmin'),
        )
        self.temp_dir = tempfile.mkdtemp(dir=config.work_dir)

    def run(self) -> ConversionResult:
        """변환 실행 메인 엔트리포인트"""
        try:
            self._update_progress(0.05, "소스 파일 다운로드 중...")

            # 1. 소스 파일 정보 조회 및 다운로드
            source_files = self._get_source_files()
            if not source_files:
                raise ValueError(f"에셋 {self.config.asset_id}에 소스 파일이 없습니다")

            local_files = self._download_source_files(source_files)
            self._update_progress(0.15, "소스 파일 다운로드 완료")

            # 2. 소스 포맷 감지 및 파싱
            source_format = self._detect_source_format(source_files)
            self._update_progress(0.20, f"{source_format} 파싱 중...")

            context = self._parse_source(source_format, local_files)
            self._update_progress(0.35, "파싱 완료")

            # 3. 메시 처리
            self._update_progress(0.40, "메시 처리 중...")
            context = self._process_meshes(context)
            self._update_progress(0.55, "메시 처리 완료")

            # 4. 텍스처 처리
            self._update_progress(0.60, "텍스처 처리 중...")
            context = self._process_textures(context)
            self._update_progress(0.70, "텍스처 처리 완료")

            # 5. 타겟 포맷으로 내보내기
            self._update_progress(0.75, f"{self.config.target_format}으로 변환 중...")
            output_path = self._export(context)
            self._update_progress(0.85, "변환 완료")

            # 6. 결과를 MinIO에 업로드
            self._update_progress(0.90, "결과 파일 업로드 중...")
            file_record = self._upload_result(output_path)
            self._update_progress(0.95, "업로드 완료")

            # 7. DB 업데이트 (완료)
            self._mark_completed(file_record['id'])
            self._update_progress(1.0, "변환 완료")

            return ConversionResult(
                success=True,
                result_file_id=file_record['id'],
                output_path=output_path,
            )

        except Exception as e:
            self._mark_failed(str(e))
            return ConversionResult(success=False, error=str(e))

        finally:
            # 임시 디렉토리 정리
            shutil.rmtree(self.temp_dir, ignore_errors=True)
            self.db_conn.close()

    def _get_source_files(self) -> list[dict]:
        """DB에서 에셋의 소스 파일 목록 조회"""
        with self.db_conn.cursor() as cur:
            cur.execute(
                """
                SELECT id, format, storage_path, size_bytes, checksum_sha256
                FROM asset_files
                WHERE asset_id = %s
                ORDER BY created_at DESC
                """,
                (self.config.asset_id,),
            )
            columns = [desc[0] for desc in cur.description]
            return [dict(zip(columns, row)) for row in cur.fetchall()]

    def _download_source_files(self, source_files: list[dict]) -> dict[str, Path]:
        """MinIO에서 소스 파일 다운로드"""
        local_files = {}
        for f in source_files:
            filename = Path(f['storage_path']).name
            local_path = Path(self.temp_dir) / f['format'] / filename
            local_path.parent.mkdir(parents=True, exist_ok=True)

            self.s3_client.download_file(
                self.config.s3_bucket,
                f['storage_path'],
                str(local_path),
            )
            local_files[f['format']] = local_path
        return local_files

    def _detect_source_format(self, source_files: list[dict]) -> str:
        """변환에 사용할 소스 포맷 결정 (우선순위: URDF > SDF > STL)"""
        formats = {f['format'] for f in source_files}
        for preferred in ['urdf', 'sdf', 'dae', 'obj', 'stl']:
            if preferred in formats:
                return preferred
        return source_files[0]['format']

    def _parse_source(self, source_format: str, local_files: dict) -> ConversionContext:
        """소스 파일을 파싱하여 내부 표현으로 변환"""
        if source_format == 'urdf':
            parser = URDFParser()
            return parser.parse(local_files['urdf'])
        elif source_format == 'sdf':
            parser = SDFParser()
            return parser.parse(local_files['sdf'])
        else:
            # 단일 메시 파일
            import trimesh
            mesh = trimesh.load(str(local_files[source_format]))
            return ConversionContext(
                meshes={'main': mesh},
                materials={},
                textures={},
                joints=[],
                links=[],
            )

    def _process_meshes(self, context: ConversionContext) -> ConversionContext:
        """메시 최적화 처리"""
        processor = MeshProcessor()
        options = self.config.options

        max_triangles = options.get('max_triangles', 100_000)
        generate_lod = options.get('generate_lod', False)
        lod_levels = options.get('lod_levels', 3)

        processed_meshes = {}
        for name, mesh in context.meshes.items():
            # 삼각형 수 제한 적용
            if hasattr(mesh, 'faces') and len(mesh.faces) > max_triangles:
                mesh = processor.simplify(mesh, target_faces=max_triangles)

            processed_meshes[name] = mesh

            # LOD 생성
            if generate_lod:
                for level in range(1, lod_levels + 1):
                    ratio = 1.0 / (2 ** level)
                    target = max(int(len(mesh.faces) * ratio), 100)
                    lod_mesh = processor.simplify(mesh, target_faces=target)
                    processed_meshes[f"{name}_lod{level}"] = lod_mesh

        context.meshes = processed_meshes
        return context

    def _process_textures(self, context: ConversionContext) -> ConversionContext:
        """텍스처 최적화 처리"""
        processor = TextureProcessor()
        options = self.config.options

        max_size = options.get('max_texture_size', 2048)
        target_format = options.get('texture_format', 'webp')

        processed_textures = {}
        for name, texture_path in context.textures.items():
            processed = processor.process(
                texture_path,
                max_size=max_size,
                output_format=target_format,
                output_dir=Path(self.temp_dir) / 'textures',
            )
            processed_textures[name] = processed

        context.textures = processed_textures
        return context

    def _export(self, context: ConversionContext) -> Path:
        """타겟 포맷으로 내보내기"""
        target = self.config.target_format
        options = self.config.options

        if target in ('gltf', 'glb'):
            exporter = GLTFExporter()
            output_path = Path(self.temp_dir) / f"output.{target}"
            exporter.export(
                context,
                output_path=output_path,
                binary=(target == 'glb'),
                enable_draco=options.get('enable_draco', True),
                draco_level=options.get('draco_compression_level', 7),
                embed_textures=options.get('embed_textures', True),
            )
            return output_path
        elif target == 'sdf':
            from sdf_parser import SDFExporter
            exporter = SDFExporter()
            output_path = Path(self.temp_dir) / "model.sdf"
            exporter.export(context, output_path)
            return output_path
        else:
            raise ValueError(f"지원하지 않는 타겟 포맷: {target}")

    def _upload_result(self, output_path: Path) -> dict:
        """결과 파일을 MinIO에 업로드하고 DB에 파일 레코드 생성"""
        import hashlib

        # 체크섬 계산
        sha256 = hashlib.sha256()
        with open(output_path, 'rb') as f:
            for chunk in iter(lambda: f.read(8192), b''):
                sha256.update(chunk)
        checksum = sha256.hexdigest()
        file_size = output_path.stat().st_size

        # 스토리지 경로 결정
        asset_info = self._get_asset_info()
        storage_path = (
            f"{asset_info['type']}/{self.config.asset_id}"
            f"/v{asset_info['version']}/{self.config.target_format}/{output_path.name}"
        )

        # MinIO에 업로드
        self.s3_client.upload_file(
            str(output_path),
            self.config.s3_bucket,
            storage_path,
        )

        # DB에 파일 레코드 생성
        import uuid as uuid_mod
        file_id = str(uuid_mod.uuid4())
        with self.db_conn.cursor() as cur:
            cur.execute(
                """
                INSERT INTO asset_files (id, asset_id, format, storage_path, size_bytes, checksum_sha256)
                VALUES (%s, %s, %s, %s, %s, %s)
                RETURNING id
                """,
                (file_id, self.config.asset_id, self.config.target_format,
                 storage_path, file_size, checksum),
            )
            self.db_conn.commit()

        return {'id': file_id, 'storage_path': storage_path}

    def _get_asset_info(self) -> dict:
        """에셋 기본 정보 조회"""
        with self.db_conn.cursor() as cur:
            cur.execute(
                "SELECT type, version FROM assets WHERE id = %s",
                (self.config.asset_id,),
            )
            row = cur.fetchone()
            return {'type': row[0], 'version': row[1]}

    def _update_progress(self, progress: float, message: str = ""):
        """DB에 변환 진행률 업데이트"""
        with self.db_conn.cursor() as cur:
            cur.execute(
                """
                UPDATE conversion_jobs
                SET progress = %s, status = 'processing'
                WHERE id = %s
                """,
                (progress, self.config.job_id),
            )
            self.db_conn.commit()

    def _mark_completed(self, result_file_id: str):
        """변환 완료 표시"""
        with self.db_conn.cursor() as cur:
            cur.execute(
                """
                UPDATE conversion_jobs
                SET status = 'completed', progress = 1.0,
                    result_file_id = %s, completed_at = NOW()
                WHERE id = %s
                """,
                (result_file_id, self.config.job_id),
            )
            self.db_conn.commit()

    def _mark_failed(self, error_message: str):
        """변환 실패 표시"""
        with self.db_conn.cursor() as cur:
            cur.execute(
                """
                UPDATE conversion_jobs
                SET status = 'failed', error_message = %s, completed_at = NOW()
                WHERE id = %s
                """,
                (error_message, self.config.job_id),
            )
            self.db_conn.commit()


def main():
    parser = argparse.ArgumentParser(description='Asset Converter')
    parser.add_argument('--job-id', required=True)
    parser.add_argument('--asset-id', required=True)
    parser.add_argument('--target-format', required=True)
    parser.add_argument('--options', required=True, help='JSON string of conversion options')
    parser.add_argument('--work-dir', required=True)
    parser.add_argument('--db-url', required=True)
    parser.add_argument('--s3-endpoint', required=True)
    parser.add_argument('--s3-bucket', required=True)

    args = parser.parse_args()

    config = ConversionConfig(
        job_id=args.job_id,
        asset_id=args.asset_id,
        target_format=args.target_format,
        options=json.loads(args.options),
        work_dir=args.work_dir,
        db_url=args.db_url,
        s3_endpoint=args.s3_endpoint,
        s3_bucket=args.s3_bucket,
    )

    converter = Converter(config)
    result = converter.run()

    if not result.success:
        print(f"Conversion failed: {result.error}", file=sys.stderr)
        sys.exit(1)

    print(f"Conversion completed: {result.result_file_id}")


if __name__ == '__main__':
    main()
```

#### 4.4.4 URDF Parser

```python
# processing/urdf_parser.py
"""
URDF 파일을 파싱하여 내부 표현(ConversionContext)으로 변환한다.
URDF의 link, joint, visual, collision 요소를 모두 추출한다.
"""

from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional
import numpy as np

import trimesh
from urdfpy import URDF

from models import (
    ConversionContext, LinkInfo, JointInfo, MeshInfo,
    MaterialInfo, Pose, InertialInfo
)


class URDFParser:
    """
    URDF 파일 파서.

    지원하는 메시 포맷: STL, DAE (Collada), OBJ
    URDF 내 모든 visual/collision geometry를 추출하고,
    joint tree 구조를 보존하면서 내부 표현으로 변환한다.
    """

    def parse(self, urdf_path: Path) -> ConversionContext:
        """
        URDF 파일을 파싱하여 ConversionContext를 반환한다.

        Args:
            urdf_path: URDF 파일 경로

        Returns:
            ConversionContext: 메시, 머티리얼, 조인트 정보를 포함하는 변환 컨텍스트
        """
        urdf_dir = urdf_path.parent
        robot = URDF.load(str(urdf_path))

        meshes = {}
        materials = {}
        textures = {}
        links = []
        joints = []

        # Link 파싱
        for link in robot.links:
            link_info = LinkInfo(
                name=link.name,
                visual_meshes=[],
                collision_meshes=[],
                inertial=None,
            )

            # Visual geometry 추출
            if link.visuals:
                for i, visual in enumerate(link.visuals):
                    mesh_name = f"{link.name}_visual_{i}"
                    mesh = self._extract_geometry(visual.geometry, urdf_dir)
                    if mesh is not None:
                        # Visual origin 적용
                        if visual.origin is not None:
                            mesh.apply_transform(visual.origin)

                        meshes[mesh_name] = mesh
                        link_info.visual_meshes.append(mesh_name)

                        # 머티리얼 추출
                        if visual.material is not None:
                            mat_info = self._extract_material(visual.material, urdf_dir)
                            materials[mesh_name] = mat_info
                            if mat_info.texture_path:
                                textures[mesh_name] = mat_info.texture_path

            # Collision geometry 추출
            if link.collisions:
                for i, collision in enumerate(link.collisions):
                    mesh_name = f"{link.name}_collision_{i}"
                    mesh = self._extract_geometry(collision.geometry, urdf_dir)
                    if mesh is not None:
                        if collision.origin is not None:
                            mesh.apply_transform(collision.origin)
                        meshes[mesh_name] = mesh
                        link_info.collision_meshes.append(mesh_name)

            # Inertial 정보
            if link.inertial is not None:
                link_info.inertial = InertialInfo(
                    mass=link.inertial.mass,
                    origin=link.inertial.origin.tolist() if link.inertial.origin is not None else None,
                    inertia=link.inertial.inertia.tolist() if link.inertial.inertia is not None else None,
                )

            links.append(link_info)

        # Joint 파싱
        for joint in robot.joints:
            joint_info = JointInfo(
                name=joint.name,
                joint_type=joint.joint_type,
                parent_link=joint.parent,
                child_link=joint.child,
                axis=joint.axis.tolist() if joint.axis is not None else [0, 0, 1],
                origin=joint.origin.tolist() if joint.origin is not None else None,
                limits=None,
            )

            if joint.limit is not None:
                joint_info.limits = {
                    'lower': joint.limit.lower or 0.0,
                    'upper': joint.limit.upper or 0.0,
                    'effort': joint.limit.effort or 0.0,
                    'velocity': joint.limit.velocity or 0.0,
                }

            joints.append(joint_info)

        return ConversionContext(
            meshes=meshes,
            materials=materials,
            textures=textures,
            links=links,
            joints=joints,
            source_format='urdf',
            robot_name=robot.name or urdf_path.stem,
        )

    def _extract_geometry(
        self,
        geometry,
        base_dir: Path,
    ) -> Optional[trimesh.Trimesh]:
        """
        URDF geometry 요소에서 trimesh 객체를 추출한다.

        지원하는 geometry 타입:
        - mesh: STL, DAE, OBJ 파일 로드
        - box: 직육면체 생성
        - cylinder: 원기둥 생성
        - sphere: 구 생성
        """
        if hasattr(geometry, 'mesh') and geometry.mesh is not None:
            mesh_path = self._resolve_mesh_path(geometry.mesh.filename, base_dir)
            if mesh_path and mesh_path.exists():
                mesh = trimesh.load(str(mesh_path), force='mesh')
                # 스케일 적용
                if geometry.mesh.scale is not None:
                    mesh.apply_scale(geometry.mesh.scale)
                return mesh
            return None

        elif hasattr(geometry, 'box') and geometry.box is not None:
            size = geometry.box.size
            return trimesh.creation.box(extents=size)

        elif hasattr(geometry, 'cylinder') and geometry.cylinder is not None:
            return trimesh.creation.cylinder(
                radius=geometry.cylinder.radius,
                height=geometry.cylinder.length,
            )

        elif hasattr(geometry, 'sphere') and geometry.sphere is not None:
            return trimesh.creation.icosphere(
                radius=geometry.sphere.radius,
                subdivisions=3,
            )

        return None

    def _resolve_mesh_path(self, filename: str, base_dir: Path) -> Optional[Path]:
        """
        URDF 내 mesh filename을 실제 파일 경로로 변환한다.
        - 'package://' 접두사 처리
        - 상대 경로 처리
        """
        if filename.startswith('package://'):
            # package:// URI에서 패키지 이름 이후 경로 추출
            parts = filename[len('package://'):].split('/', 1)
            if len(parts) == 2:
                relative = parts[1]
            else:
                relative = parts[0]
            resolved = base_dir / relative
            if resolved.exists():
                return resolved
            # 패키지 루트에서도 탐색
            return base_dir / Path(filename[len('package://'):])
        elif filename.startswith('file://'):
            return Path(filename[len('file://'):])
        else:
            return base_dir / filename

    def _extract_material(self, material, base_dir: Path) -> MaterialInfo:
        """URDF material에서 색상 및 텍스처 정보 추출"""
        color = None
        texture_path = None

        if material.color is not None:
            color = {
                'r': float(material.color[0]),
                'g': float(material.color[1]),
                'b': float(material.color[2]),
                'a': float(material.color[3]) if len(material.color) > 3 else 1.0,
            }

        if material.texture is not None and material.texture.filename:
            tex_path = self._resolve_mesh_path(material.texture.filename, base_dir)
            if tex_path and tex_path.exists():
                texture_path = tex_path

        return MaterialInfo(
            name=material.name or 'default',
            color=color,
            texture_path=texture_path,
        )
```

#### 4.4.5 SDF Parser

```python
# processing/sdf_parser.py
"""
SDF (Simulation Description Format) 파일을 파싱한다.
Gazebo에서 사용하는 SDF 포맷을 내부 표현으로 변환한다.
"""

from pathlib import Path
from typing import Optional
from lxml import etree
import trimesh
import numpy as np

from models import ConversionContext, LinkInfo, JointInfo, MaterialInfo, Pose


class SDFParser:
    """SDF 파일 파서"""

    def parse(self, sdf_path: Path) -> ConversionContext:
        """
        SDF 파일을 파싱하여 ConversionContext를 반환한다.

        SDF는 <world> 또는 <model> 루트 요소를 가질 수 있다.
        <model> 내의 <link>, <joint>를 추출한다.
        """
        tree = etree.parse(str(sdf_path))
        root = tree.getroot()
        sdf_dir = sdf_path.parent

        meshes = {}
        materials = {}
        textures = {}
        links = []
        joints = []

        # <model> 요소 찾기
        models = root.findall('.//model')
        if not models:
            # 루트가 직접 model인 경우
            models = [root] if root.tag == 'model' else []

        model_name = "unknown"
        for model in models:
            model_name = model.get('name', 'model')

            # Link 파싱
            for link_elem in model.findall('link'):
                link_name = link_elem.get('name', 'link')
                link_info = LinkInfo(
                    name=link_name,
                    visual_meshes=[],
                    collision_meshes=[],
                    inertial=None,
                )

                # Visual 파싱
                for i, visual in enumerate(link_elem.findall('visual')):
                    mesh_name = f"{link_name}_visual_{i}"
                    geom = visual.find('geometry')
                    if geom is not None:
                        mesh = self._parse_geometry(geom, sdf_dir)
                        if mesh is not None:
                            pose = self._parse_pose(visual.find('pose'))
                            if pose:
                                mesh.apply_transform(pose.to_matrix())
                            meshes[mesh_name] = mesh
                            link_info.visual_meshes.append(mesh_name)

                    # Material
                    mat_elem = visual.find('material')
                    if mat_elem is not None:
                        materials[mesh_name] = self._parse_material(mat_elem, sdf_dir)

                # Collision 파싱
                for i, collision in enumerate(link_elem.findall('collision')):
                    mesh_name = f"{link_name}_collision_{i}"
                    geom = collision.find('geometry')
                    if geom is not None:
                        mesh = self._parse_geometry(geom, sdf_dir)
                        if mesh is not None:
                            meshes[mesh_name] = mesh
                            link_info.collision_meshes.append(mesh_name)

                links.append(link_info)

            # Joint 파싱
            for joint_elem in model.findall('joint'):
                joint_name = joint_elem.get('name', 'joint')
                joint_type = joint_elem.get('type', 'fixed')

                parent = joint_elem.find('parent')
                child = joint_elem.find('child')
                axis_elem = joint_elem.find('axis/xyz')

                axis = [0, 0, 1]
                if axis_elem is not None and axis_elem.text:
                    axis = [float(x) for x in axis_elem.text.strip().split()]

                joint_info = JointInfo(
                    name=joint_name,
                    joint_type=joint_type,
                    parent_link=parent.text.strip() if parent is not None else '',
                    child_link=child.text.strip() if child is not None else '',
                    axis=axis,
                    origin=None,
                    limits=None,
                )

                # Limits
                limit_elem = joint_elem.find('axis/limit')
                if limit_elem is not None:
                    joint_info.limits = {
                        'lower': float(limit_elem.findtext('lower', '0')),
                        'upper': float(limit_elem.findtext('upper', '0')),
                        'effort': float(limit_elem.findtext('effort', '0')),
                        'velocity': float(limit_elem.findtext('velocity', '0')),
                    }

                joints.append(joint_info)

        return ConversionContext(
            meshes=meshes,
            materials=materials,
            textures=textures,
            links=links,
            joints=joints,
            source_format='sdf',
            robot_name=model_name,
        )

    def _parse_geometry(self, geom_elem, base_dir: Path) -> Optional[trimesh.Trimesh]:
        """SDF <geometry> 요소 파싱"""
        mesh_elem = geom_elem.find('mesh')
        if mesh_elem is not None:
            uri = mesh_elem.findtext('uri', '')
            scale_text = mesh_elem.findtext('scale', '1 1 1')
            scale = [float(s) for s in scale_text.strip().split()]

            mesh_path = self._resolve_uri(uri, base_dir)
            if mesh_path and mesh_path.exists():
                mesh = trimesh.load(str(mesh_path), force='mesh')
                mesh.apply_scale(scale)
                return mesh
            return None

        box_elem = geom_elem.find('box')
        if box_elem is not None:
            size_text = box_elem.findtext('size', '1 1 1')
            size = [float(s) for s in size_text.strip().split()]
            return trimesh.creation.box(extents=size)

        cylinder_elem = geom_elem.find('cylinder')
        if cylinder_elem is not None:
            radius = float(cylinder_elem.findtext('radius', '0.5'))
            length = float(cylinder_elem.findtext('length', '1.0'))
            return trimesh.creation.cylinder(radius=radius, height=length)

        sphere_elem = geom_elem.find('sphere')
        if sphere_elem is not None:
            radius = float(sphere_elem.findtext('radius', '0.5'))
            return trimesh.creation.icosphere(radius=radius, subdivisions=3)

        return None

    def _parse_pose(self, pose_elem) -> Optional[Pose]:
        """SDF <pose> 요소 파싱. 형식: 'x y z roll pitch yaw'"""
        if pose_elem is None or not pose_elem.text:
            return None
        values = [float(v) for v in pose_elem.text.strip().split()]
        if len(values) == 6:
            return Pose(
                x=values[0], y=values[1], z=values[2],
                roll=values[3], pitch=values[4], yaw=values[5],
            )
        return None

    def _parse_material(self, mat_elem, base_dir: Path) -> MaterialInfo:
        """SDF <material> 요소 파싱"""
        color = None
        diffuse = mat_elem.find('diffuse')
        if diffuse is not None and diffuse.text:
            vals = [float(v) for v in diffuse.text.strip().split()]
            color = {'r': vals[0], 'g': vals[1], 'b': vals[2], 'a': vals[3] if len(vals) > 3 else 1.0}

        return MaterialInfo(
            name=mat_elem.findtext('name', 'default'),
            color=color,
            texture_path=None,
        )

    def _resolve_uri(self, uri: str, base_dir: Path) -> Optional[Path]:
        """SDF URI를 실제 파일 경로로 변환"""
        if uri.startswith('model://'):
            model_path = uri[len('model://'):]
            # Gazebo 모델 경로 탐색
            gazebo_model_path = Path.home() / '.gazebo' / 'models' / model_path
            if gazebo_model_path.exists():
                return gazebo_model_path
            return base_dir / model_path
        elif uri.startswith('file://'):
            return Path(uri[len('file://'):])
        else:
            return base_dir / uri
```

#### 4.4.6 glTF Exporter

```python
# processing/gltf_exporter.py
"""
내부 표현(ConversionContext)을 glTF 2.0 또는 GLB 포맷으로 내보낸다.
Draco 압축, 텍스처 임베딩, LOD 지원을 포함한다.
"""

from pathlib import Path
from typing import Optional
import struct
import json
import base64

import numpy as np
import trimesh
from pygltflib import (
    GLTF2, Scene, Node, Mesh, Primitive, Accessor, BufferView,
    Buffer, Material, PbrMetallicRoughness, Image, Texture,
    TextureInfo, Asset,
)

from models import ConversionContext


class GLTFExporter:
    """glTF 2.0 / GLB 포맷 내보내기"""

    # glTF component type constants
    FLOAT = 5126
    UNSIGNED_SHORT = 5123
    UNSIGNED_INT = 5125

    # glTF buffer view targets
    ARRAY_BUFFER = 34962
    ELEMENT_ARRAY_BUFFER = 34963

    def export(
        self,
        context: ConversionContext,
        output_path: Path,
        binary: bool = True,
        enable_draco: bool = True,
        draco_level: int = 7,
        embed_textures: bool = True,
    ):
        """
        ConversionContext를 glTF/GLB 파일로 내보낸다.

        Args:
            context: 변환 컨텍스트 (메시, 머티리얼, 조인트 정보)
            output_path: 출력 파일 경로
            binary: True이면 GLB, False이면 glTF + .bin
            enable_draco: Draco 메시 압축 활성화
            draco_level: Draco 압축 레벨 (1-10)
            embed_textures: 텍스처를 GLB에 임베드
        """
        # trimesh의 Scene을 활용하여 glTF 구성
        scene = trimesh.Scene()

        # 메시를 Scene에 추가 (visual mesh만, collision은 제외)
        for link in context.links:
            for mesh_name in link.visual_meshes:
                if mesh_name in context.meshes:
                    mesh = context.meshes[mesh_name]

                    # 머티리얼 적용
                    if mesh_name in context.materials:
                        mat = context.materials[mesh_name]
                        if mat.color:
                            visual = trimesh.visual.ColorVisuals(
                                mesh=mesh,
                                face_colors=np.array([
                                    [
                                        int(mat.color['r'] * 255),
                                        int(mat.color['g'] * 255),
                                        int(mat.color['b'] * 255),
                                        int(mat.color.get('a', 1.0) * 255),
                                    ]
                                ] * len(mesh.faces)),
                            )
                            mesh.visual = visual

                    scene.add_geometry(
                        mesh,
                        node_name=mesh_name,
                        geom_name=mesh_name,
                    )

        # Joint 계층 구조 구성
        # trimesh Scene의 graph를 사용하여 joint 트리를 표현
        for joint in context.joints:
            if joint.origin is not None:
                transform = np.array(joint.origin)
                if transform.shape == (4, 4):
                    scene.graph.update(
                        frame_from=joint.parent_link,
                        frame_to=joint.child_link,
                        matrix=transform,
                    )

        # glTF/GLB로 내보내기
        if binary:
            glb_data = scene.export(file_type='glb')
            with open(output_path, 'wb') as f:
                f.write(glb_data)
        else:
            gltf_data = scene.export(file_type='gltf')
            with open(output_path, 'wb') as f:
                f.write(gltf_data)

        # Draco 압축 (후처리)
        if enable_draco:
            self._apply_draco_compression(output_path, draco_level)

    def _apply_draco_compression(self, file_path: Path, compression_level: int):
        """
        Draco 압축 적용.
        gltf-pipeline 또는 gltfpack CLI 도구를 subprocess로 호출한다.
        """
        import subprocess

        output_path = file_path.with_suffix('.compressed' + file_path.suffix)

        try:
            # gltfpack 사용 (meshoptimizer)
            result = subprocess.run(
                [
                    'gltfpack',
                    '-i', str(file_path),
                    '-o', str(output_path),
                    '-cc',  # Draco compression
                ],
                capture_output=True,
                text=True,
                timeout=120,
            )

            if result.returncode == 0 and output_path.exists():
                # 원본 교체
                output_path.replace(file_path)
            else:
                # gltfpack 실패 시 원본 유지 (경고만 출력)
                print(f"Warning: Draco compression failed: {result.stderr}")
                if output_path.exists():
                    output_path.unlink()

        except FileNotFoundError:
            print("Warning: gltfpack not found, skipping Draco compression")
        except subprocess.TimeoutExpired:
            print("Warning: Draco compression timed out")
            if output_path.exists():
                output_path.unlink()
```

#### 4.4.7 Mesh Processor

```python
# processing/mesh_processor.py
"""
메시 최적화 처리: 단순화(decimation), LOD 생성, 무결성 검사.
"""

from pathlib import Path
from typing import Optional
from dataclasses import dataclass

import numpy as np
import trimesh


@dataclass
class MeshStats:
    """메시 통계 정보"""
    vertices: int
    faces: int
    edges: int
    bounding_box_min: list[float]
    bounding_box_max: list[float]
    is_watertight: bool
    is_manifold: bool
    volume: Optional[float]
    surface_area: float


class MeshProcessor:
    """메시 처리 유틸리티"""

    def simplify(
        self,
        mesh: trimesh.Trimesh,
        target_faces: int,
        preserve_borders: bool = True,
    ) -> trimesh.Trimesh:
        """
        메시를 지정된 삼각형 수로 단순화한다.

        Quadric Edge Collapse Decimation 알고리즘을 사용한다.
        UV 좌표와 법선 벡터를 최대한 보존한다.

        Args:
            mesh: 원본 메시
            target_faces: 목표 삼각형 수
            preserve_borders: 경계 엣지 보존 여부

        Returns:
            단순화된 메시
        """
        if len(mesh.faces) <= target_faces:
            return mesh.copy()

        # trimesh의 simplify_quadric_decimation 사용
        simplified = mesh.simplify_quadric_decimation(target_faces)
        return simplified

    def generate_lod_levels(
        self,
        mesh: trimesh.Trimesh,
        levels: int = 3,
        ratios: Optional[list[float]] = None,
    ) -> list[trimesh.Trimesh]:
        """
        LOD (Level of Detail) 레벨을 생성한다.

        Args:
            mesh: 원본 메시 (LOD 0)
            levels: 생성할 LOD 레벨 수
            ratios: 각 레벨별 삼각형 비율 (없으면 자동 계산)

        Returns:
            LOD 메시 리스트 (LOD 0 = 원본, LOD N = 가장 단순)
        """
        if ratios is None:
            # 기본 비율: 50%, 25%, 12.5%, ...
            ratios = [1.0 / (2 ** (i + 1)) for i in range(levels)]

        lod_meshes = [mesh.copy()]  # LOD 0 = 원본

        for ratio in ratios:
            target = max(int(len(mesh.faces) * ratio), 100)
            lod = self.simplify(mesh, target_faces=target)
            lod_meshes.append(lod)

        return lod_meshes

    def get_stats(self, mesh: trimesh.Trimesh) -> MeshStats:
        """메시 통계 정보를 반환한다."""
        bounds = mesh.bounds
        return MeshStats(
            vertices=len(mesh.vertices),
            faces=len(mesh.faces),
            edges=len(mesh.edges),
            bounding_box_min=bounds[0].tolist(),
            bounding_box_max=bounds[1].tolist(),
            is_watertight=mesh.is_watertight,
            is_manifold=bool(mesh.is_volume),
            volume=float(mesh.volume) if mesh.is_watertight else None,
            surface_area=float(mesh.area),
        )

    def check_integrity(self, mesh: trimesh.Trimesh) -> list[str]:
        """
        메시 무결성 검사.

        검사 항목:
        - Non-manifold edges (하나의 엣지가 3개 이상의 면에 공유)
        - Degenerate triangles (면적이 0인 삼각형)
        - Duplicate vertices
        - Unreferenced vertices
        - Inverted normals

        Returns:
            발견된 문제점 목록
        """
        issues = []

        # Degenerate faces (면적이 극히 작은 삼각형)
        face_areas = mesh.area_faces
        degenerate_count = np.sum(face_areas < 1e-10)
        if degenerate_count > 0:
            issues.append(
                f"MESH_DEGENERATE_FACES: {degenerate_count}개의 퇴화 삼각형 발견"
            )

        # Duplicate vertices
        unique_vertices = np.unique(mesh.vertices, axis=0)
        dup_count = len(mesh.vertices) - len(unique_vertices)
        if dup_count > 0:
            issues.append(
                f"MESH_DUPLICATE_VERTICES: {dup_count}개의 중복 정점 발견"
            )

        # Non-watertight
        if not mesh.is_watertight:
            issues.append("MESH_NOT_WATERTIGHT: 메시가 밀폐되지 않음 (구멍 있음)")

        # Non-manifold
        if not mesh.is_volume:
            issues.append("MESH_NON_MANIFOLD: 비매니폴드 메시")

        return issues

    def merge_meshes(self, meshes: list[trimesh.Trimesh]) -> trimesh.Trimesh:
        """여러 메시를 하나로 병합한다."""
        if not meshes:
            return trimesh.Trimesh()
        if len(meshes) == 1:
            return meshes[0].copy()

        return trimesh.util.concatenate(meshes)

    def center_mesh(self, mesh: trimesh.Trimesh) -> trimesh.Trimesh:
        """메시를 원점 중심으로 이동한다."""
        centered = mesh.copy()
        centered.vertices -= centered.centroid
        return centered
```

#### 4.4.8 Texture Processor

```python
# processing/texture_processor.py
"""
텍스처 이미지 처리: 리사이즈, 포맷 변환 (PNG → WebP/KTX2).
"""

from pathlib import Path
from PIL import Image


class TextureProcessor:
    """텍스처 이미지 처리 유틸리티"""

    SUPPORTED_FORMATS = {'png', 'jpg', 'jpeg', 'webp', 'tga', 'bmp'}
    OUTPUT_FORMATS = {'webp', 'png', 'ktx2'}

    def process(
        self,
        texture_path: Path,
        max_size: int = 2048,
        output_format: str = 'webp',
        output_dir: Path = None,
        quality: int = 85,
    ) -> Path:
        """
        텍스처를 처리하여 최적화된 이미지를 생성한다.

        Args:
            texture_path: 원본 텍스처 경로
            max_size: 최대 텍스처 크기 (가로/세로 중 큰 값)
            output_format: 출력 포맷 ('webp', 'png', 'ktx2')
            output_dir: 출력 디렉토리 (None이면 원본 디렉토리)
            quality: 압축 품질 (1-100, WebP/JPEG에 적용)

        Returns:
            처리된 텍스처 파일 경로
        """
        if output_dir is None:
            output_dir = texture_path.parent
        output_dir.mkdir(parents=True, exist_ok=True)

        img = Image.open(texture_path)

        # RGBA로 변환 (알파 채널 보존)
        if img.mode not in ('RGBA', 'RGB'):
            img = img.convert('RGBA')

        # 리사이즈 (비율 유지)
        if max(img.size) > max_size:
            ratio = max_size / max(img.size)
            new_size = (int(img.size[0] * ratio), int(img.size[1] * ratio))
            img = img.resize(new_size, Image.Resampling.LANCZOS)

        # 2의 거듭제곱 크기로 조정 (GPU 최적화)
        img = self._to_power_of_two(img)

        # 출력 파일 경로
        output_name = texture_path.stem + '.' + output_format
        output_path = output_dir / output_name

        if output_format == 'webp':
            img.save(output_path, 'WEBP', quality=quality)
        elif output_format == 'png':
            img.save(output_path, 'PNG', optimize=True)
        elif output_format == 'ktx2':
            # KTX2는 별도 도구가 필요 (toktx)
            # 먼저 PNG로 저장 후 toktx로 변환
            temp_png = output_dir / (texture_path.stem + '_temp.png')
            img.save(temp_png, 'PNG')
            self._convert_to_ktx2(temp_png, output_path)
            temp_png.unlink(missing_ok=True)
        else:
            raise ValueError(f"지원하지 않는 출력 포맷: {output_format}")

        return output_path

    def _to_power_of_two(self, img: Image.Image) -> Image.Image:
        """이미지 크기를 2의 거듭제곱으로 조정"""
        def next_pot(v: int) -> int:
            v -= 1
            v |= v >> 1
            v |= v >> 2
            v |= v >> 4
            v |= v >> 8
            v |= v >> 16
            return v + 1

        w, h = img.size
        new_w = next_pot(w)
        new_h = next_pot(h)

        if new_w != w or new_h != h:
            img = img.resize((new_w, new_h), Image.Resampling.LANCZOS)

        return img

    def _convert_to_ktx2(self, input_path: Path, output_path: Path):
        """PNG를 KTX2로 변환 (toktx CLI 사용)"""
        import subprocess

        try:
            subprocess.run(
                [
                    'toktx',
                    '--t2',
                    '--encode', 'uastc',
                    '--uastc_quality', '2',
                    '--zcmp', '5',
                    str(output_path),
                    str(input_path),
                ],
                capture_output=True,
                check=True,
                timeout=60,
            )
        except (FileNotFoundError, subprocess.CalledProcessError) as e:
            # toktx 사용 불가 시 WebP로 폴백
            img = Image.open(input_path)
            fallback_path = output_path.with_suffix('.webp')
            img.save(fallback_path, 'WEBP', quality=85)
            raise RuntimeError(f"KTX2 변환 실패, WebP로 대체: {e}")
```

#### 4.4.9 Validator

```python
# processing/validator.py
"""
에셋 유효성 검증.
메시 무결성, URDF/SDF 구조, glTF 표준 준수, 파일 크기 제한을 검사한다.
"""

from pathlib import Path
from dataclasses import dataclass, field
import json
import subprocess

import trimesh
from lxml import etree

from mesh_processor import MeshProcessor


@dataclass
class ValidationIssue:
    """검증 이슈"""
    code: str
    message: str
    file_path: str = ""
    severity: str = "error"  # "error" | "warning"


@dataclass
class ValidationStats:
    """검증 통계"""
    total_vertices: int = 0
    total_triangles: int = 0
    total_textures: int = 0
    total_size_bytes: int = 0
    bounding_box: dict = field(default_factory=dict)


@dataclass
class ValidationResult:
    """검증 결과"""
    is_valid: bool = True
    errors: list[ValidationIssue] = field(default_factory=list)
    warnings: list[ValidationIssue] = field(default_factory=list)
    stats: ValidationStats = field(default_factory=ValidationStats)


class AssetValidator:
    """에셋 유효성 검증기"""

    # 파일 크기 제한
    WARN_SIZE_BYTES = 50 * 1024 * 1024   # 50MB
    MAX_SIZE_BYTES = 500 * 1024 * 1024   # 500MB

    # 메시 복잡도 제한
    WARN_TRIANGLE_COUNT = 500_000
    MAX_TRIANGLE_COUNT = 5_000_000

    def validate_urdf(self, urdf_path: Path) -> ValidationResult:
        """
        URDF 파일의 유효성을 검증한다.

        검증 항목:
        1. XML 문법 유효성
        2. 필수 요소 존재 (robot, link, joint)
        3. link/joint 참조 무결성 (parent/child가 존재하는 link를 참조)
        4. 메시 파일 참조 유효성 (참조된 STL/DAE 파일이 존재)
        5. 메시 무결성 (non-manifold, degenerate faces)
        6. 파일 크기 제한
        """
        result = ValidationResult()
        urdf_dir = urdf_path.parent

        # XML 파싱
        try:
            tree = etree.parse(str(urdf_path))
            root = tree.getroot()
        except etree.XMLSyntaxError as e:
            result.errors.append(ValidationIssue(
                code="URDF_XML_INVALID",
                message=f"XML 문법 오류: {e}",
                file_path=str(urdf_path),
            ))
            result.is_valid = False
            return result

        # 루트 요소 확인
        if root.tag != 'robot':
            result.errors.append(ValidationIssue(
                code="URDF_MISSING_ROBOT",
                message="루트 요소가 <robot>이 아닙니다",
                file_path=str(urdf_path),
            ))
            result.is_valid = False
            return result

        # Link 목록 수집
        links = {}
        for link in root.findall('.//link'):
            name = link.get('name')
            if name:
                if name in links:
                    result.errors.append(ValidationIssue(
                        code="URDF_DUPLICATE_LINK",
                        message=f"중복 link 이름: '{name}'",
                        file_path=str(urdf_path),
                    ))
                links[name] = link

        if not links:
            result.errors.append(ValidationIssue(
                code="URDF_NO_LINKS",
                message="link 요소가 없습니다",
                file_path=str(urdf_path),
            ))
            result.is_valid = False
            return result

        # Joint 참조 무결성
        for joint in root.findall('.//joint'):
            joint_name = joint.get('name', 'unknown')
            parent = joint.find('parent')
            child = joint.find('child')

            if parent is not None:
                parent_link = parent.get('link')
                if parent_link and parent_link not in links:
                    result.errors.append(ValidationIssue(
                        code="URDF_MISSING_PARENT_LINK",
                        message=f"Joint '{joint_name}'의 parent link '{parent_link}'가 존재하지 않습니다",
                        file_path=str(urdf_path),
                    ))
            else:
                result.errors.append(ValidationIssue(
                    code="URDF_MISSING_PARENT",
                    message=f"Joint '{joint_name}'에 parent 요소가 없습니다",
                    file_path=str(urdf_path),
                ))

            if child is not None:
                child_link = child.get('link')
                if child_link and child_link not in links:
                    result.errors.append(ValidationIssue(
                        code="URDF_MISSING_CHILD_LINK",
                        message=f"Joint '{joint_name}'의 child link '{child_link}'가 존재하지 않습니다",
                        file_path=str(urdf_path),
                    ))
            else:
                result.errors.append(ValidationIssue(
                    code="URDF_MISSING_CHILD",
                    message=f"Joint '{joint_name}'에 child 요소가 없습니다",
                    file_path=str(urdf_path),
                ))

        # 메시 파일 참조 검증
        mesh_processor = MeshProcessor()
        total_vertices = 0
        total_faces = 0

        for mesh_elem in root.findall('.//mesh'):
            filename = mesh_elem.get('filename', '')
            if not filename:
                continue

            # 경로 해석
            if filename.startswith('package://'):
                parts = filename[len('package://'):].split('/', 1)
                rel_path = parts[1] if len(parts) > 1 else parts[0]
                mesh_path = urdf_dir / rel_path
            else:
                mesh_path = urdf_dir / filename

            if not mesh_path.exists():
                result.errors.append(ValidationIssue(
                    code="URDF_MISSING_MESH",
                    message=f"메시 파일을 찾을 수 없습니다: {filename}",
                    file_path=str(urdf_path),
                ))
                continue

            # 메시 무결성 검사
            try:
                mesh = trimesh.load(str(mesh_path), force='mesh')
                issues = mesh_processor.check_integrity(mesh)
                for issue in issues:
                    result.warnings.append(ValidationIssue(
                        code=issue.split(':')[0],
                        message=issue,
                        file_path=str(mesh_path),
                        severity="warning",
                    ))
                total_vertices += len(mesh.vertices)
                total_faces += len(mesh.faces)

                # 파일 크기 확인
                file_size = mesh_path.stat().st_size
                if file_size > self.MAX_SIZE_BYTES:
                    result.errors.append(ValidationIssue(
                        code="FILE_TOO_LARGE",
                        message=f"파일 크기가 제한을 초과합니다: {file_size / 1024 / 1024:.1f}MB > 500MB",
                        file_path=str(mesh_path),
                    ))
                elif file_size > self.WARN_SIZE_BYTES:
                    result.warnings.append(ValidationIssue(
                        code="FILE_SIZE_WARNING",
                        message=f"파일 크기가 큽니다: {file_size / 1024 / 1024:.1f}MB",
                        file_path=str(mesh_path),
                        severity="warning",
                    ))

            except Exception as e:
                result.errors.append(ValidationIssue(
                    code="MESH_LOAD_FAILED",
                    message=f"메시 로드 실패: {e}",
                    file_path=str(mesh_path),
                ))

        # 삼각형 수 제한 확인
        if total_faces > self.MAX_TRIANGLE_COUNT:
            result.errors.append(ValidationIssue(
                code="MESH_TOO_COMPLEX",
                message=f"총 삼각형 수가 제한을 초과합니다: {total_faces:,} > {self.MAX_TRIANGLE_COUNT:,}",
            ))
        elif total_faces > self.WARN_TRIANGLE_COUNT:
            result.warnings.append(ValidationIssue(
                code="MESH_COMPLEXITY_WARNING",
                message=f"총 삼각형 수가 많습니다: {total_faces:,}",
                severity="warning",
            ))

        # 통계 업데이트
        result.stats.total_vertices = total_vertices
        result.stats.total_triangles = total_faces

        # 에러가 있으면 유효하지 않음
        result.is_valid = len(result.errors) == 0

        return result

    def validate_gltf(self, gltf_path: Path) -> ValidationResult:
        """
        glTF/GLB 파일의 유효성을 검증한다.
        외부 도구(gltf-validator)를 사용하여 glTF 2.0 표준 준수를 확인한다.
        """
        result = ValidationResult()

        # gltf-validator 실행
        try:
            proc = subprocess.run(
                ['gltf_validator', str(gltf_path), '-o', '-'],
                capture_output=True,
                text=True,
                timeout=30,
            )

            if proc.returncode == 0 and proc.stdout:
                report = json.loads(proc.stdout)

                # 에러 추출
                for issue in report.get('issues', {}).get('messages', []):
                    severity = issue.get('severity', 0)
                    vi = ValidationIssue(
                        code=issue.get('code', 'GLTF_UNKNOWN'),
                        message=issue.get('message', ''),
                        file_path=str(gltf_path),
                        severity="error" if severity == 0 else "warning",
                    )
                    if severity == 0:
                        result.errors.append(vi)
                    else:
                        result.warnings.append(vi)

                # 통계
                info = report.get('info', {})
                result.stats.total_vertices = info.get('totalVertexCount', 0)
                result.stats.total_triangles = info.get('totalTriangleCount', 0)

        except FileNotFoundError:
            result.warnings.append(ValidationIssue(
                code="VALIDATOR_NOT_FOUND",
                message="gltf_validator를 찾을 수 없습니다. glTF 표준 검증을 건너뜁니다.",
                severity="warning",
            ))

        except subprocess.TimeoutExpired:
            result.warnings.append(ValidationIssue(
                code="VALIDATOR_TIMEOUT",
                message="glTF 검증 시간 초과",
                severity="warning",
            ))

        # 파일 크기 확인
        file_size = gltf_path.stat().st_size
        result.stats.total_size_bytes = file_size

        if file_size > self.MAX_SIZE_BYTES:
            result.errors.append(ValidationIssue(
                code="FILE_TOO_LARGE",
                message=f"파일 크기 제한 초과: {file_size / 1024 / 1024:.1f}MB > 500MB",
                file_path=str(gltf_path),
            ))
        elif file_size > self.WARN_SIZE_BYTES:
            result.warnings.append(ValidationIssue(
                code="FILE_SIZE_WARNING",
                message=f"파일 크기 주의: {file_size / 1024 / 1024:.1f}MB",
                file_path=str(gltf_path),
                severity="warning",
            ))

        result.is_valid = len(result.errors) == 0
        return result

    def validate_json_asset(self, json_path: Path, schema_type: str) -> ValidationResult:
        """
        JSON 에셋(sensor_definition, mission_template, traffic_rule)의 유효성 검증.
        필수 필드 존재 여부와 데이터 타입을 확인한다.
        """
        result = ValidationResult()

        try:
            with open(json_path) as f:
                data = json.load(f)
        except json.JSONDecodeError as e:
            result.errors.append(ValidationIssue(
                code="JSON_INVALID",
                message=f"JSON 파싱 오류: {e}",
                file_path=str(json_path),
            ))
            result.is_valid = False
            return result

        required_fields = {
            'sensor_definition': ['sensor_type', 'specifications', 'simulation_params'],
            'mission_template': ['template_name', 'parameters', 'task_sequence'],
            'traffic_rule': ['rule_name', 'rule_type', 'conditions', 'parameters'],
        }

        for field_name in required_fields.get(schema_type, []):
            if field_name not in data:
                result.errors.append(ValidationIssue(
                    code=f"MISSING_FIELD_{field_name.upper()}",
                    message=f"필수 필드 '{field_name}'이(가) 없습니다",
                    file_path=str(json_path),
                ))

        result.is_valid = len(result.errors) == 0
        return result
```

#### 4.4.10 Python 데이터 모델

```python
# processing/models.py
"""변환 파이프라인 내부 데이터 모델"""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional, Any

import numpy as np
import trimesh


@dataclass
class Pose:
    """3D 위치 + 방향 (RPY)"""
    x: float = 0.0
    y: float = 0.0
    z: float = 0.0
    roll: float = 0.0
    pitch: float = 0.0
    yaw: float = 0.0

    def to_matrix(self) -> np.ndarray:
        """4x4 동차 변환 행렬로 변환"""
        cr, sr = np.cos(self.roll), np.sin(self.roll)
        cp, sp = np.cos(self.pitch), np.sin(self.pitch)
        cy, sy = np.cos(self.yaw), np.sin(self.yaw)

        rotation = np.array([
            [cy*cp, cy*sp*sr - sy*cr, cy*sp*cr + sy*sr],
            [sy*cp, sy*sp*sr + cy*cr, sy*sp*cr - cy*sr],
            [-sp,   cp*sr,            cp*cr],
        ])

        matrix = np.eye(4)
        matrix[:3, :3] = rotation
        matrix[:3, 3] = [self.x, self.y, self.z]
        return matrix


@dataclass
class MaterialInfo:
    """머티리얼(재질) 정보"""
    name: str
    color: Optional[dict] = None      # {"r": 0-1, "g": 0-1, "b": 0-1, "a": 0-1}
    texture_path: Optional[Path] = None
    metallic: float = 0.0
    roughness: float = 0.8


@dataclass
class InertialInfo:
    """관성 정보"""
    mass: float
    origin: Optional[list] = None     # 4x4 transform matrix as list
    inertia: Optional[list] = None    # 3x3 inertia matrix as list


@dataclass
class JointInfo:
    """조인트 정보"""
    name: str
    joint_type: str                    # "revolute", "continuous", "prismatic", "fixed", "floating", "planar"
    parent_link: str
    child_link: str
    axis: list[float] = field(default_factory=lambda: [0, 0, 1])
    origin: Optional[Any] = None       # 4x4 transform matrix or list
    limits: Optional[dict] = None      # {"lower", "upper", "effort", "velocity"}


@dataclass
class LinkInfo:
    """링크 정보"""
    name: str
    visual_meshes: list[str] = field(default_factory=list)
    collision_meshes: list[str] = field(default_factory=list)
    inertial: Optional[InertialInfo] = None


@dataclass
class ConversionContext:
    """
    변환 파이프라인의 중간 표현.
    모든 파서가 이 구조로 변환하고, 모든 익스포터가 이 구조에서 내보낸다.
    """
    meshes: dict[str, trimesh.Trimesh] = field(default_factory=dict)
    materials: dict[str, MaterialInfo] = field(default_factory=dict)
    textures: dict[str, Path] = field(default_factory=dict)
    links: list[LinkInfo] = field(default_factory=list)
    joints: list[JointInfo] = field(default_factory=list)
    source_format: str = ""
    robot_name: str = ""
    metadata: dict = field(default_factory=dict)


@dataclass
class ConversionResult:
    """변환 결과"""
    success: bool
    result_file_id: Optional[str] = None
    output_path: Optional[Path] = None
    error: Optional[str] = None
```

### 4.5 Validation

검증 시스템의 전체 검증 항목 요약:

| 검증 항목 | 대상 포맷 | 심각도 | 설명 |
|---|---|---|---|
| `URDF_XML_INVALID` | URDF | error | XML 문법 오류 |
| `URDF_MISSING_ROBOT` | URDF | error | `<robot>` 루트 요소 없음 |
| `URDF_DUPLICATE_LINK` | URDF | error | 중복 link 이름 |
| `URDF_NO_LINKS` | URDF | error | link 요소 없음 |
| `URDF_MISSING_PARENT_LINK` | URDF | error | joint의 parent link 미존재 |
| `URDF_MISSING_CHILD_LINK` | URDF | error | joint의 child link 미존재 |
| `URDF_MISSING_MESH` | URDF | error | 참조된 메시 파일 미존재 |
| `MESH_NON_MANIFOLD` | STL/DAE/OBJ | warning | 비매니폴드 메시 |
| `MESH_DEGENERATE_FACES` | STL/DAE/OBJ | warning | 퇴화 삼각형 존재 |
| `MESH_DUPLICATE_VERTICES` | STL/DAE/OBJ | warning | 중복 정점 |
| `MESH_NOT_WATERTIGHT` | STL/DAE/OBJ | warning | 밀폐되지 않은 메시 |
| `MESH_TOO_COMPLEX` | all | error | 삼각형 수 > 5,000,000 |
| `MESH_COMPLEXITY_WARNING` | all | warning | 삼각형 수 > 500,000 |
| `MESH_LOAD_FAILED` | STL/DAE/OBJ | error | 메시 로드 실패 |
| `FILE_TOO_LARGE` | all | error | 파일 크기 > 500MB |
| `FILE_SIZE_WARNING` | all | warning | 파일 크기 > 50MB |
| `GLTF_*` | glTF/GLB | varies | glTF 표준 미준수 (gltf-validator) |
| `JSON_INVALID` | JSON | error | JSON 파싱 오류 |
| `MISSING_FIELD_*` | JSON | error | 필수 필드 누락 |

---

## 5. 데이터베이스 스키마

### 5.1 마이그레이션 파일

#### 001_create_assets.sql

```sql
-- 에셋 메인 테이블
-- 모든 에셋 유형의 공통 메타데이터를 저장한다.
-- 유형별 세부 속성은 properties JSONB 컬럼에 저장한다.

CREATE TABLE assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 에셋 유형 (6가지)
    type VARCHAR(50) NOT NULL
        CHECK (type IN (
            'robot_model',
            'static_object',
            'dynamic_object',
            'sensor_definition',
            'mission_template',
            'traffic_rule'
        )),

    -- 에셋 이름 (사용자가 지정, 유형 내에서 유일하지 않아도 됨)
    name VARCHAR(255) NOT NULL,

    -- 에셋 설명
    description TEXT,

    -- 유형별 세부 속성 (JSON)
    -- robot_model: dimensions, weight_kg, max_speed_mps, drive_type, joints, sensors, ...
    -- static_object: category, dimensions, material, collision_shapes, ...
    -- dynamic_object: category, states, joints, triggers, ...
    -- sensor_definition: sensor_type, specifications, simulation_params, ros_config, ...
    -- mission_template: template_name, parameters, task_sequence, abort_conditions, ...
    -- traffic_rule: rule_name, rule_type, conditions, parameters, enforcement, ...
    properties JSONB NOT NULL DEFAULT '{}',

    -- 태그 (검색/필터링용)
    tags TEXT[] DEFAULT '{}',

    -- 버전 번호 (1부터 시작, 업데이트 시 증가)
    version INTEGER NOT NULL DEFAULT 1,

    -- 감사 타임스탬프
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- updated_at 자동 갱신 트리거
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_assets_updated_at
    BEFORE UPDATE ON assets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

#### 002_create_asset_files.sql

```sql
-- 에셋 파일 테이블
-- 하나의 에셋은 여러 포맷의 파일을 가질 수 있다.
-- 예: robot_model은 URDF + SDF + glTF 파일을 동시에 가질 수 있음.

CREATE TABLE asset_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 소속 에셋 (CASCADE 삭제)
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,

    -- 파일 포맷
    format VARCHAR(50) NOT NULL
        CHECK (format IN ('urdf', 'sdf', 'gltf', 'glb', 'stl', 'dae', 'obj', 'json')),

    -- MinIO 내 스토리지 경로
    -- 형식: {type}/{asset_id}/v{version}/{format}/{filename}
    storage_path VARCHAR(512) NOT NULL,

    -- 파일 크기 (바이트)
    size_bytes BIGINT NOT NULL CHECK (size_bytes >= 0),

    -- 파일 SHA256 체크섬 (무결성 검증용)
    checksum_sha256 VARCHAR(64) NOT NULL,

    -- 원본 파일명 (업로드 시 이름)
    original_filename VARCHAR(255),

    -- 감사 타임스탬프
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### 003_create_conversions.sql

```sql
-- 변환 작업 테이블
-- 비동기 포맷 변환 작업의 상태를 추적한다.

CREATE TABLE conversion_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 소스 에셋
    source_asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,

    -- 소스 포맷 (변환 시작 시 자동 감지)
    source_format VARCHAR(50),

    -- 타겟 포맷
    target_format VARCHAR(50) NOT NULL
        CHECK (target_format IN ('urdf', 'sdf', 'gltf', 'glb')),

    -- 변환 상태
    status VARCHAR(50) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processing', 'completed', 'failed')),

    -- 진행률 (0.0 ~ 1.0)
    progress REAL NOT NULL DEFAULT 0.0 CHECK (progress >= 0.0 AND progress <= 1.0),

    -- 변환 옵션 (JSON)
    options JSONB DEFAULT '{}',

    -- 에러 메시지 (실패 시)
    error_message TEXT,

    -- 결과 파일 (완료 시)
    result_file_id UUID REFERENCES asset_files(id) ON DELETE SET NULL,

    -- 감사 타임스탬프
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
```

#### 004_create_indexes.sql

```sql
-- ============================================================
-- 성능 최적화 인덱스
-- ============================================================

-- assets 테이블 인덱스

-- 유형별 조회 (매우 빈번)
CREATE INDEX idx_assets_type ON assets(type);

-- 이름 검색 (ILIKE 패턴 매칭)
CREATE INDEX idx_assets_name_trgm ON assets USING gin (name gin_trgm_ops);
-- 참고: pg_trgm 확장 필요: CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- 태그 검색 (배열 포함 연산 @>)
CREATE INDEX idx_assets_tags ON assets USING gin (tags);

-- 생성일 정렬 (페이징)
CREATE INDEX idx_assets_created_at ON assets(created_at DESC);

-- 수정일 정렬 (페이징)
CREATE INDEX idx_assets_updated_at ON assets(updated_at DESC);

-- 유형 + 생성일 복합 인덱스 (유형별 최신순 조회)
CREATE INDEX idx_assets_type_created ON assets(type, created_at DESC);

-- JSONB properties 내 특정 필드 검색
-- 예: properties->>'sensor_type' = 'lidar_2d'
CREATE INDEX idx_assets_properties ON assets USING gin (properties jsonb_path_ops);


-- asset_files 테이블 인덱스

-- 에셋별 파일 조회
CREATE INDEX idx_asset_files_asset_id ON asset_files(asset_id);

-- 에셋 + 포맷 복합 인덱스 (특정 포맷 파일 조회)
CREATE INDEX idx_asset_files_asset_format ON asset_files(asset_id, format);

-- 스토리지 경로 유니크 (동일 경로에 두 파일 불가)
CREATE UNIQUE INDEX idx_asset_files_storage_path ON asset_files(storage_path);


-- conversion_jobs 테이블 인덱스

-- 소스 에셋별 변환 작업 조회
CREATE INDEX idx_conversion_jobs_asset ON conversion_jobs(source_asset_id);

-- 상태별 조회 (pending 작업 처리용)
CREATE INDEX idx_conversion_jobs_status ON conversion_jobs(status);

-- 생성일 정렬
CREATE INDEX idx_conversion_jobs_created ON conversion_jobs(created_at DESC);


-- ============================================================
-- 확장 설치 (필요 시)
-- ============================================================
CREATE EXTENSION IF NOT EXISTS pg_trgm;     -- 트라이그램 유사도 검색
CREATE EXTENSION IF NOT EXISTS "uuid-ossp"; -- UUID 생성 (gen_random_uuid 대체)
```

### 5.2 ER Diagram (텍스트)

```
┌─────────────────────┐        ┌─────────────────────────┐
│       assets        │        │      asset_files        │
├─────────────────────┤        ├─────────────────────────┤
│ id (PK, UUID)       │───┐    │ id (PK, UUID)           │
│ type (VARCHAR)      │   │    │ asset_id (FK) ──────────┤──┐
│ name (VARCHAR)      │   │    │ format (VARCHAR)        │  │
│ description (TEXT)  │   │    │ storage_path (VARCHAR)  │  │
│ properties (JSONB)  │   │    │ size_bytes (BIGINT)     │  │
│ tags (TEXT[])       │   │    │ checksum_sha256 (VARCHAR│  │
│ version (INTEGER)   │   │    │ original_filename       │  │
│ created_at          │   │    │ created_at              │  │
│ updated_at          │   │    └─────────────────────────┘  │
└─────────────────────┘   │                                  │
          │               │    ┌─────────────────────────┐  │
          │               │    │   conversion_jobs       │  │
          │               │    ├─────────────────────────┤  │
          │               └───▶│ id (PK, UUID)           │  │
          │                    │ source_asset_id (FK) ───┤──┘
          └───────────────────▶│ source_format           │
                               │ target_format           │
                               │ status                  │
                               │ progress                │
                               │ options (JSONB)         │
                               │ error_message           │
                               │ result_file_id (FK) ───┤──▶ asset_files.id
                               │ created_at              │
                               │ completed_at            │
                               └─────────────────────────┘
```

---

## 6. 개발 계획

> 📋 Phase별 상세 일정 및 팀별 할당: [`development-plan.md`](../development-plan.md) 참조
>
> 본 팀의 기능별 상세 스펙은 아래 기능 문서 참조:
> - C-XX: `docs/features/core/C-XX-*.md`
> - A-XX: `docs/features/advanced/A-XX-*.md`

---

## 7. 테스트 계획

### 7.1 Rust 단위 테스트

```rust
// tests/rust/test_grpc.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Request;

    /// ListAssets: 빈 결과 반환 테스트
    #[tokio::test]
    async fn test_list_assets_empty() {
        let service = create_test_service().await;
        let request = Request::new(ListAssetsRequest {
            type_filter: AssetType::Unspecified as i32,
            page_size: 20,
            ..Default::default()
        });

        let response = service.list_assets(request).await.unwrap();
        let body = response.into_inner();
        assert_eq!(body.total_count, 0);
        assert!(body.assets.is_empty());
    }

    /// CreateAsset: 메타데이터 + 파일 업로드 테스트
    #[tokio::test]
    async fn test_create_asset() {
        let service = create_test_service().await;

        // 메타데이터 + 파일 청크 스트림 생성
        let metadata = CreateAssetMetadata {
            r#type: AssetType::RobotModel as i32,
            name: "Test Robot".to_string(),
            description: "A test robot model".to_string(),
            filename: "robot.urdf".to_string(),
            format: AssetFormat::Urdf as i32,
            ..Default::default()
        };

        let chunks = vec![
            CreateAssetRequest {
                payload: Some(Payload::Metadata(metadata)),
            },
            CreateAssetRequest {
                payload: Some(Payload::Chunk(AssetChunk {
                    data: b"<robot name='test'></robot>".to_vec(),
                    offset: 0,
                })),
            },
        ];

        let stream = tokio_stream::iter(chunks);
        let request = Request::new(stream);
        let response = service.create_asset(request).await.unwrap();
        let asset = response.into_inner();

        assert_eq!(asset.name, "Test Robot");
        assert_eq!(asset.r#type, AssetType::RobotModel as i32);
        assert!(!asset.id.is_empty());
        assert_eq!(asset.version, 1);
    }

    /// GetAsset: 존재하지 않는 에셋 조회 시 NOT_FOUND 에러
    #[tokio::test]
    async fn test_get_asset_not_found() {
        let service = create_test_service().await;
        let request = Request::new(GetAssetRequest {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            version: 0,
        });

        let result = service.get_asset(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
    }

    /// DeleteAsset: 에셋 삭제 시 관련 파일도 삭제 확인
    #[tokio::test]
    async fn test_delete_asset_cascade() {
        let service = create_test_service().await;

        // 에셋 생성
        let asset_id = create_test_asset(&service).await;

        // 삭제
        let request = Request::new(DeleteAssetRequest {
            id: asset_id.clone(),
            delete_all_versions: true,
        });
        let response = service.delete_asset(request).await.unwrap();
        assert!(response.into_inner().success);

        // 재조회 시 NOT_FOUND
        let get_request = Request::new(GetAssetRequest {
            id: asset_id,
            version: 0,
        });
        assert!(service.get_asset(get_request).await.is_err());
    }
}
```

```rust
// tests/rust/test_storage.rs

#[cfg(test)]
mod tests {
    use testcontainers::{clients, images};

    /// MinIO 업로드/다운로드 라운드트립 테스트
    #[tokio::test]
    async fn test_upload_download_roundtrip() {
        let docker = clients::Cli::default();
        let minio = docker.run(images::minio::MinIO::default());
        let storage = create_test_storage(&minio).await;

        let data = b"test file content".to_vec();
        let path = "test/file.txt";

        // 업로드
        let result = storage.upload(path, data.clone()).await.unwrap();
        assert_eq!(result.size_bytes, data.len() as i64);

        // 다운로드
        let downloaded = storage.download(path).await.unwrap();
        assert_eq!(downloaded, data);

        // 체크섬 확인
        assert!(!result.checksum_sha256.is_empty());
    }

    /// 존재하지 않는 파일 다운로드 시 에러
    #[tokio::test]
    async fn test_download_not_found() {
        let docker = clients::Cli::default();
        let minio = docker.run(images::minio::MinIO::default());
        let storage = create_test_storage(&minio).await;

        let result = storage.download("nonexistent/file.txt").await;
        assert!(result.is_err());
    }

    /// 접두사 삭제 테스트
    #[tokio::test]
    async fn test_delete_prefix() {
        let docker = clients::Cli::default();
        let minio = docker.run(images::minio::MinIO::default());
        let storage = create_test_storage(&minio).await;

        // 여러 파일 업로드
        storage.upload("prefix/a.txt", b"a".to_vec()).await.unwrap();
        storage.upload("prefix/b.txt", b"b".to_vec()).await.unwrap();
        storage.upload("other/c.txt", b"c".to_vec()).await.unwrap();

        // prefix 삭제
        let count = storage.delete_prefix("prefix/").await.unwrap();
        assert_eq!(count, 2);

        // prefix 파일은 삭제됨
        assert!(!storage.exists("prefix/a.txt").await.unwrap());
        // other 파일은 유지
        assert!(storage.exists("other/c.txt").await.unwrap());
    }
}
```

### 7.2 Python 단위 테스트

```python
# tests/python/conftest.py

import pytest
from pathlib import Path

FIXTURES_DIR = Path(__file__).parent / 'fixtures'


@pytest.fixture
def sample_urdf() -> Path:
    return FIXTURES_DIR / 'sample_robot.urdf'


@pytest.fixture
def sample_sdf() -> Path:
    return FIXTURES_DIR / 'sample_world.sdf'


@pytest.fixture
def sample_stl() -> Path:
    return FIXTURES_DIR / 'sample_mesh.stl'


@pytest.fixture
def sample_glb() -> Path:
    return FIXTURES_DIR / 'sample_model.glb'


@pytest.fixture
def tmp_output(tmp_path) -> Path:
    return tmp_path / 'output'
```

```python
# tests/python/test_urdf_parser.py

import pytest
from pathlib import Path

from processing.urdf_parser import URDFParser
from processing.models import ConversionContext


class TestURDFParser:
    """URDF 파서 단위 테스트"""

    def test_parse_basic_urdf(self, sample_urdf):
        """기본 URDF 파싱: link, joint 추출 확인"""
        parser = URDFParser()
        context = parser.parse(sample_urdf)

        assert isinstance(context, ConversionContext)
        assert context.source_format == 'urdf'
        assert len(context.links) > 0
        assert len(context.meshes) > 0

    def test_parse_extracts_joints(self, sample_urdf):
        """URDF에서 joint 정보 추출 확인"""
        parser = URDFParser()
        context = parser.parse(sample_urdf)

        # TurtleBot3-like 로봇은 최소 2개 wheel joint를 가짐
        assert len(context.joints) >= 2

        for joint in context.joints:
            assert joint.name
            assert joint.joint_type
            assert joint.parent_link
            assert joint.child_link

    def test_parse_extracts_visual_meshes(self, sample_urdf):
        """URDF에서 visual mesh 추출 확인"""
        parser = URDFParser()
        context = parser.parse(sample_urdf)

        visual_meshes = []
        for link in context.links:
            visual_meshes.extend(link.visual_meshes)

        assert len(visual_meshes) > 0

        # 모든 visual mesh가 context.meshes에 존재
        for mesh_name in visual_meshes:
            assert mesh_name in context.meshes

    def test_parse_materials(self, sample_urdf):
        """URDF에서 material 정보 추출 확인"""
        parser = URDFParser()
        context = parser.parse(sample_urdf)

        # 최소한 일부 mesh에는 material이 있어야 함
        # (모든 mesh에 material이 없을 수도 있으므로 빈 dict도 허용)
        assert isinstance(context.materials, dict)

    def test_parse_nonexistent_file(self):
        """존재하지 않는 URDF 파일 파싱 시 예외 발생"""
        parser = URDFParser()
        with pytest.raises(Exception):
            parser.parse(Path('/nonexistent/robot.urdf'))


class TestURDFParserMeshResolution:
    """URDF 파서의 mesh 경로 해석 테스트"""

    def test_resolve_package_uri(self, tmp_path):
        """package:// URI 해석"""
        parser = URDFParser()
        mesh_dir = tmp_path / 'meshes'
        mesh_dir.mkdir()
        (mesh_dir / 'base.stl').write_bytes(b'dummy')

        resolved = parser._resolve_mesh_path(
            'package://robot_description/meshes/base.stl',
            tmp_path,
        )
        # base_dir에서 상대 경로로 탐색
        assert resolved is not None

    def test_resolve_relative_path(self, tmp_path):
        """상대 경로 해석"""
        parser = URDFParser()
        (tmp_path / 'mesh.stl').write_bytes(b'dummy')

        resolved = parser._resolve_mesh_path('mesh.stl', tmp_path)
        assert resolved == tmp_path / 'mesh.stl'
```

```python
# tests/python/test_converter.py

import pytest
import json
from pathlib import Path
from unittest.mock import MagicMock, patch

from processing.converter import Converter, ConversionConfig
from processing.models import ConversionResult


class TestConverter:
    """변환 오케스트레이터 통합 테스트"""

    @pytest.fixture
    def mock_config(self, tmp_path):
        return ConversionConfig(
            job_id='test-job-id',
            asset_id='test-asset-id',
            target_format='glb',
            options={
                'max_triangles': 50000,
                'max_texture_size': 1024,
                'enable_draco': False,
                'embed_textures': True,
            },
            work_dir=str(tmp_path),
            db_url='postgresql://test:test@localhost/test',
            s3_endpoint='http://localhost:9000',
            s3_bucket='test-bucket',
        )

    @patch('processing.converter.psycopg2.connect')
    @patch('processing.converter.boto3.client')
    def test_converter_creation(self, mock_boto, mock_pg, mock_config):
        """Converter 인스턴스 생성 테스트"""
        mock_pg.return_value = MagicMock()
        mock_boto.return_value = MagicMock()

        converter = Converter(mock_config)
        assert converter.config == mock_config

    def test_conversion_config_parsing(self):
        """변환 옵션 JSON 파싱 테스트"""
        options_json = json.dumps({
            'max_triangles': 100000,
            'enable_draco': True,
            'draco_compression_level': 7,
        })
        options = json.loads(options_json)
        assert options['max_triangles'] == 100000
        assert options['enable_draco'] is True
```

### 7.3 통합 테스트

통합 테스트는 testcontainers를 사용하여 실제 PostgreSQL + MinIO 컨테이너에서 수행한다.

```rust
// tests/rust/test_integration.rs

#[cfg(test)]
mod integration_tests {
    use testcontainers::{clients, GenericImage};

    /// 전체 에셋 라이프사이클 테스트:
    /// 생성 → 조회 → 업데이트 → 변환 요청 → 다운로드 → 삭제
    #[tokio::test]
    async fn test_full_asset_lifecycle() {
        // 1. 테스트 환경 준비 (PostgreSQL + MinIO 컨테이너)
        let docker = clients::Cli::default();
        let (service, _containers) = setup_test_environment(&docker).await;

        // 2. 에셋 생성 (URDF 파일 업로드)
        let asset = create_asset_via_grpc(&service, "Test Robot", "urdf", SAMPLE_URDF).await;
        assert_eq!(asset.name, "Test Robot");

        // 3. 에셋 조회
        let fetched = get_asset_via_grpc(&service, &asset.id).await;
        assert_eq!(fetched.id, asset.id);
        assert_eq!(fetched.files.len(), 1);

        // 4. 메타데이터 업데이트
        let updated = update_asset_via_grpc(&service, &asset.id, "Updated Robot").await;
        assert_eq!(updated.name, "Updated Robot");

        // 5. 파일 다운로드
        let downloaded = download_asset_via_grpc(&service, &asset.id, "urdf").await;
        assert_eq!(downloaded, SAMPLE_URDF);

        // 6. 유효성 검증
        let validation = validate_asset_via_grpc(&service, &asset.id).await;
        assert!(validation.is_valid);

        // 7. 삭제
        let deleted = delete_asset_via_grpc(&service, &asset.id).await;
        assert!(deleted.success);
    }

    /// 대용량 파일 스트리밍 업로드 테스트
    #[tokio::test]
    async fn test_large_file_streaming_upload() {
        let docker = clients::Cli::default();
        let (service, _containers) = setup_test_environment(&docker).await;

        // 10MB 더미 데이터 생성
        let large_data = vec![0u8; 10 * 1024 * 1024];

        let asset = create_asset_via_grpc(&service, "Large Model", "stl", &large_data).await;
        assert_eq!(asset.files[0].size_bytes, large_data.len() as i64);

        // 다운로드 후 크기 확인
        let downloaded = download_asset_via_grpc(&service, &asset.id, "stl").await;
        assert_eq!(downloaded.len(), large_data.len());
    }

    /// 동시 접근 테스트
    #[tokio::test]
    async fn test_concurrent_operations() {
        let docker = clients::Cli::default();
        let (service, _containers) = setup_test_environment(&docker).await;

        // 10개 에셋을 동시에 생성
        let mut handles = Vec::new();
        for i in 0..10 {
            let svc = service.clone();
            let handle = tokio::spawn(async move {
                create_asset_via_grpc(&svc, &format!("Robot {}", i), "urdf", SAMPLE_URDF).await
            });
            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles).await;
        for result in &results {
            assert!(result.is_ok());
        }

        // 목록 조회 확인
        let list = list_assets_via_grpc(&service, None, 20).await;
        assert_eq!(list.total_count, 10);
    }
}
```

### 7.4 성능 벤치마크

| 테스트 항목 | 목표 | 측정 방법 |
|---|---|---|
| 1MB 파일 업로드 | < 500ms | gRPC 스트리밍 업로드 완료 시간 |
| 1MB 파일 다운로드 | < 300ms | gRPC 스트리밍 다운로드 완료 시간 |
| ListAssets (1000건) | < 100ms | 페이징 조회 응답 시간 |
| GetAsset | < 50ms | 단일 에셋 조회 응답 시간 |
| URDF → glTF 변환 (10MB) | < 60s | 변환 완료까지 총 시간 |
| 메시 단순화 (100K → 10K faces) | < 5s | trimesh simplify 실행 시간 |

---

## 8. 목 데이터

### 8.1 Sample URDF (TurtleBot3-like)

```xml
<?xml version="1.0" ?>
<robot name="amr_sample" xmlns:xacro="http://www.ros.org/wiki/xacro">

  <!-- Base Link -->
  <link name="base_footprint"/>

  <link name="base_link">
    <visual>
      <origin xyz="0 0 0.05" rpy="0 0 0"/>
      <geometry>
        <box size="0.4 0.3 0.1"/>
      </geometry>
      <material name="dark_grey">
        <color rgba="0.3 0.3 0.3 1.0"/>
      </material>
    </visual>
    <collision>
      <origin xyz="0 0 0.05" rpy="0 0 0"/>
      <geometry>
        <box size="0.4 0.3 0.1"/>
      </geometry>
    </collision>
    <inertial>
      <mass value="8.0"/>
      <origin xyz="0 0 0.05" rpy="0 0 0"/>
      <inertia ixx="0.04" ixy="0" ixz="0" iyy="0.06" iyz="0" izz="0.08"/>
    </inertial>
  </link>

  <joint name="base_joint" type="fixed">
    <parent link="base_footprint"/>
    <child link="base_link"/>
    <origin xyz="0 0 0.05" rpy="0 0 0"/>
  </joint>

  <!-- Left Wheel -->
  <link name="wheel_left_link">
    <visual>
      <origin xyz="0 0 0" rpy="1.5708 0 0"/>
      <geometry>
        <cylinder radius="0.05" length="0.02"/>
      </geometry>
      <material name="black">
        <color rgba="0.1 0.1 0.1 1.0"/>
      </material>
    </visual>
    <collision>
      <origin xyz="0 0 0" rpy="1.5708 0 0"/>
      <geometry>
        <cylinder radius="0.05" length="0.02"/>
      </geometry>
    </collision>
    <inertial>
      <mass value="0.5"/>
      <inertia ixx="0.0001" ixy="0" ixz="0" iyy="0.0001" iyz="0" izz="0.0001"/>
    </inertial>
  </link>

  <joint name="wheel_left_joint" type="continuous">
    <parent link="base_link"/>
    <child link="wheel_left_link"/>
    <origin xyz="0.0 0.15 0.0" rpy="0 0 0"/>
    <axis xyz="0 1 0"/>
    <limit effort="10.0" velocity="6.28"/>
  </joint>

  <!-- Right Wheel -->
  <link name="wheel_right_link">
    <visual>
      <origin xyz="0 0 0" rpy="1.5708 0 0"/>
      <geometry>
        <cylinder radius="0.05" length="0.02"/>
      </geometry>
      <material name="black">
        <color rgba="0.1 0.1 0.1 1.0"/>
      </material>
    </visual>
    <collision>
      <origin xyz="0 0 0" rpy="1.5708 0 0"/>
      <geometry>
        <cylinder radius="0.05" length="0.02"/>
      </geometry>
    </collision>
    <inertial>
      <mass value="0.5"/>
      <inertia ixx="0.0001" ixy="0" ixz="0" iyy="0.0001" iyz="0" izz="0.0001"/>
    </inertial>
  </link>

  <joint name="wheel_right_joint" type="continuous">
    <parent link="base_link"/>
    <child link="wheel_right_link"/>
    <origin xyz="0.0 -0.15 0.0" rpy="0 0 0"/>
    <axis xyz="0 1 0"/>
    <limit effort="10.0" velocity="6.28"/>
  </joint>

  <!-- Caster Wheel -->
  <link name="caster_link">
    <visual>
      <geometry>
        <sphere radius="0.025"/>
      </geometry>
      <material name="grey">
        <color rgba="0.5 0.5 0.5 1.0"/>
      </material>
    </visual>
    <collision>
      <geometry>
        <sphere radius="0.025"/>
      </geometry>
    </collision>
    <inertial>
      <mass value="0.1"/>
      <inertia ixx="0.00001" ixy="0" ixz="0" iyy="0.00001" iyz="0" izz="0.00001"/>
    </inertial>
  </link>

  <joint name="caster_joint" type="fixed">
    <parent link="base_link"/>
    <child link="caster_link"/>
    <origin xyz="-0.15 0 -0.025" rpy="0 0 0"/>
  </joint>

  <!-- LiDAR -->
  <link name="lidar_link">
    <visual>
      <origin xyz="0 0 0" rpy="0 0 0"/>
      <geometry>
        <cylinder radius="0.03" length="0.04"/>
      </geometry>
      <material name="blue">
        <color rgba="0.0 0.0 0.8 1.0"/>
      </material>
    </visual>
    <collision>
      <origin xyz="0 0 0" rpy="0 0 0"/>
      <geometry>
        <cylinder radius="0.03" length="0.04"/>
      </geometry>
    </collision>
    <inertial>
      <mass value="0.2"/>
      <inertia ixx="0.00005" ixy="0" ixz="0" iyy="0.00005" iyz="0" izz="0.00005"/>
    </inertial>
  </link>

  <joint name="lidar_joint" type="fixed">
    <parent link="base_link"/>
    <child link="lidar_link"/>
    <origin xyz="0.15 0 0.12" rpy="0 0 0"/>
  </joint>

</robot>
```

### 8.2 Sample SDF

```xml
<?xml version="1.0" ?>
<sdf version="1.6">
  <world name="sample_warehouse">
    <model name="warehouse_floor">
      <static>true</static>
      <link name="floor_link">
        <visual name="floor_visual">
          <geometry>
            <box>
              <size>20 20 0.01</size>
            </box>
          </geometry>
          <material>
            <diffuse>0.8 0.8 0.8 1.0</diffuse>
          </material>
        </visual>
        <collision name="floor_collision">
          <geometry>
            <box>
              <size>20 20 0.01</size>
            </box>
          </geometry>
        </collision>
      </link>
    </model>

    <model name="shelf_unit_1">
      <static>true</static>
      <pose>5 3 0 0 0 0</pose>
      <link name="shelf_link">
        <visual name="shelf_visual">
          <geometry>
            <box>
              <size>2.0 0.6 2.0</size>
            </box>
          </geometry>
          <material>
            <diffuse>0.6 0.4 0.2 1.0</diffuse>
          </material>
        </visual>
        <collision name="shelf_collision">
          <geometry>
            <box>
              <size>2.0 0.6 2.0</size>
            </box>
          </geometry>
        </collision>
      </link>
    </model>

    <model name="charging_station">
      <static>true</static>
      <pose>0 -8 0 0 0 0</pose>
      <link name="charger_link">
        <visual name="charger_visual">
          <geometry>
            <box>
              <size>0.5 0.3 0.8</size>
            </box>
          </geometry>
          <material>
            <diffuse>0.0 0.6 0.0 1.0</diffuse>
          </material>
        </visual>
        <collision name="charger_collision">
          <geometry>
            <box>
              <size>0.5 0.3 0.8</size>
            </box>
          </geometry>
        </collision>
      </link>
    </model>
  </world>
</sdf>
```

### 8.3 Sample Sensor Definitions (JSON)

**LiDAR 2D:**
```json
{
  "sensor_type": "lidar_2d",
  "manufacturer": "SICK",
  "model_number": "TIM571",
  "specifications": {
    "range_min": 0.05,
    "range_max": 25.0,
    "angle_min": -2.356,
    "angle_max": 2.356,
    "angle_increment": 0.00436,
    "scan_frequency_hz": 15,
    "samples_per_scan": 1081
  },
  "simulation_params": {
    "update_rate_hz": 15,
    "ray_count": 1081,
    "noise_type": "gaussian",
    "noise_stddev": 0.01
  },
  "ros_config": {
    "topic_name": "/scan",
    "frame_id": "laser_frame",
    "message_type": "sensor_msgs/LaserScan"
  }
}
```

**Camera RGB-D:**
```json
{
  "sensor_type": "camera_rgbd",
  "manufacturer": "Intel",
  "model_number": "RealSense D435",
  "specifications": {
    "resolution_width": 1920,
    "resolution_height": 1080,
    "fov_horizontal_deg": 69.4,
    "fov_vertical_deg": 42.5,
    "fps": 30,
    "depth_enabled": true,
    "depth_range_min": 0.1,
    "depth_range_max": 10.0
  },
  "simulation_params": {
    "update_rate_hz": 30,
    "image_format": "rgb8",
    "depth_format": "float32",
    "noise_type": "gaussian",
    "noise_stddev": 0.005
  },
  "ros_config": {
    "image_topic": "/camera/color/image_raw",
    "depth_topic": "/camera/depth/image_rect_raw",
    "frame_id": "camera_link"
  }
}
```

### 8.4 Sample Mission Template (JSON)

```json
{
  "template_name": "warehouse_patrol",
  "description": "창고 내 정해진 경로를 순찰하는 미션",
  "category": "patrol",
  "required_robot_capabilities": ["navigation", "lidar_2d"],
  "parameters": [
    {
      "name": "patrol_points",
      "type": "array",
      "item_type": "pose2d",
      "required": true,
      "min_items": 2,
      "max_items": 50
    },
    {
      "name": "loop_count",
      "type": "integer",
      "default": 0,
      "min": 0,
      "max": 1000
    },
    {
      "name": "wait_time_sec",
      "type": "float",
      "default": 2.0,
      "min": 0.0,
      "max": 300.0
    }
  ],
  "task_sequence": [
    {"step": 1, "action": "navigate_to", "params": {"target": "${patrol_points[i]}"}},
    {"step": 2, "action": "wait", "params": {"duration_sec": "${wait_time_sec}"}},
    {"step": 3, "action": "loop", "params": {"goto_step": 1, "iterations": "${loop_count}"}}
  ],
  "abort_conditions": [
    {"type": "battery_low", "threshold": 15},
    {"type": "manual_cancel"}
  ]
}
```

### 8.5 Sample Traffic Rule (JSON)

```json
{
  "rule_name": "corridor_oneway",
  "description": "복도 A-B 구간 일방통행 규칙",
  "rule_type": "one_way",
  "priority": 10,
  "conditions": {
    "applicable_zones": ["zone_corridor_a"],
    "applicable_robot_types": ["all"],
    "time_window": {
      "start": "00:00",
      "end": "23:59",
      "days": ["mon", "tue", "wed", "thu", "fri", "sat", "sun"]
    }
  },
  "parameters": {
    "direction": {"x": 1.0, "y": 0.0},
    "max_speed_mps": 1.0,
    "min_following_distance_m": 1.5
  },
  "enforcement": {
    "mode": "strict",
    "violation_action": "reroute",
    "notification": true,
    "log_violations": true
  }
}
```

---

## 9. 다른 팀과의 인터페이스 계약

### 9.1 Backend (Team 2) ↔ Asset Manager

**통신 방식**: gRPC (Backend가 클라이언트, Asset Manager가 서버)

**서비스 엔드포인트**: `asset-manager:50051`

**사용 시나리오:**

| 시나리오 | RPC 메서드 | 설명 |
|---|---|---|
| 에셋 목록 표시 | `ListAssets` | Frontend에서 에셋 브라우저 표시 시 Backend가 호출 |
| 에셋 상세 조회 | `GetAsset` | 에셋 선택 시 상세 정보 반환 |
| 에셋 업로드 | `CreateAsset` | 사용자가 URDF/SDF 파일을 업로드할 때 |
| 에셋 수정 | `UpdateAssetMetadata` | 이름, 태그, 속성 수정 |
| 에셋 삭제 | `DeleteAsset` | 에셋 및 관련 파일 삭제 |
| 에셋 다운로드 | `DownloadAsset` | 특정 포맷의 파일 다운로드 |
| 포맷 변환 | `ConvertAsset` | URDF → glTF 등 변환 요청 |
| 변환 상태 | `GetConversionStatus` | 변환 진행률 폴링 |
| 에셋 검증 | `ValidateAsset` | 업로드된 에셋의 유효성 검사 |

**에러 처리 계약:**

| gRPC Status Code | 의미 | Backend 대응 |
|---|---|---|
| `NOT_FOUND` | 요청한 에셋/파일이 없음 | 404 응답 반환 |
| `INVALID_ARGUMENT` | 잘못된 파라미터 | 400 응답 반환 |
| `ALREADY_EXISTS` | 중복 에셋 | 409 응답 반환 |
| `RESOURCE_EXHAUSTED` | 파일 크기 제한 초과 | 413 응답 반환 |
| `INTERNAL` | 서버 내부 오류 | 500 응답, 재시도 가능 |
| `UNAVAILABLE` | 서비스 이용 불가 | 503 응답, 재시도 |

### 9.2 Frontend (Team 5) ↔ Asset Manager

**직접 통신은 없음.** Backend를 통해 간접 접근.

**데이터 계약:**

1. **glTF 출력 포맷 스펙**:
   - glTF 2.0 표준 준수
   - GLB (Binary) 포맷 기본 사용
   - Draco 압축 선택적 적용
   - 텍스처: WebP 포맷, 최대 2048x2048
   - 최대 삼각형 수: 100,000 (웹 렌더링 최적화)

2. **메타데이터 JSON 스키마**: 본 문서 4.2절의 스키마를 따름

3. **에셋 미리보기**: Frontend는 glTF 포맷이 있는 에셋에 대해 3D 미리보기를 표시. glTF가 없는 에셋에는 변환 요청 버튼을 표시.

### 9.3 Sim Engine (Team 3) ↔ Asset Manager

**통신 방식**: gRPC 또는 파일 시스템 마운트

**데이터 계약:**

1. **URDF 출력**: Sim Engine은 로봇 물리 시뮬레이션에 URDF를 사용.
   - URDF 1.0 표준 준수
   - 모든 mesh 참조는 상대 경로 사용
   - 물리 파라미터(inertial, collision) 필수 포함
   - `<gazebo>` 확장 태그 선택적 포함

2. **SDF 출력**: Sim Engine은 환경 시뮬레이션에 SDF를 사용.
   - SDF 1.6+ 표준 준수
   - 정적 객체 (`<static>true</static>`) 포함
   - 물리 파라미터(surface, friction) 포함

3. **센서 정의 스키마**: Sim Engine은 `sensor_definition` 에셋의 `simulation_params`를 사용하여 센서를 구성.
   - `update_rate_hz`: 센서 업데이트 주기
   - `noise_type`, `noise_stddev`: 노이즈 모델
   - 센서 유형별 파라미터 (`ray_count`, `image_format` 등)

4. **사용 시나리오:**

| 시나리오 | 설명 |
|---|---|
| 로봇 스폰 | `GetAsset` → URDF 다운로드 → 물리 엔진에 로드 |
| 환경 구성 | `ListAssets(static_object)` → 각 객체의 SDF/URDF 다운로드 |
| 센서 설정 | `GetAsset(sensor_definition)` → `simulation_params` 적용 |

### 9.4 Map Manager (Team 6) ↔ Asset Manager

**통신 방식**: gRPC (Map Manager가 Asset Manager의 클라이언트)

**데이터 계약:**

1. **맵-에셋 참조 방식**:
   - 맵에 배치된 객체는 `asset_id`로 에셋을 참조
   - 맵 데이터 내에 에셋 전체 데이터를 포함하지 않음 (ID 참조만)
   - 맵 로드 시 Map Manager가 필요한 에셋을 Asset Manager에서 가져옴

2. **맵 객체 배치 정보 예시:**
   ```json
   {
     "placed_objects": [
       {
         "instance_id": "shelf_001",
         "asset_id": "uuid-of-shelf-asset",
         "asset_type": "static_object",
         "pose": {
           "position": {"x": 5.0, "y": 3.0, "z": 0.0},
           "orientation": {"roll": 0.0, "pitch": 0.0, "yaw": 0.0}
         },
         "scale": {"x": 1.0, "y": 1.0, "z": 1.0}
       }
     ]
   }
   ```

3. **사용 시나리오:**

| 시나리오 | 설명 |
|---|---|
| 맵에 객체 배치 | Frontend에서 에셋 선택 → `asset_id` 기록 → 맵 데이터에 저장 |
| 맵 렌더링 | 맵 내 `asset_id` 목록 → `GetAsset` 호출 → glTF 다운로드 → 렌더링 |
| 맵 시뮬레이션 | 맵 내 에셋 → URDF/SDF 다운로드 → Sim Engine에 전달 |
| 에셋 삭제 영향 | 에셋 삭제 전 참조 여부 확인 필요 (Map Manager에 질의) |

---

## 10. 배포 설정

### 10.1 Dockerfile

```dockerfile
# ============================================================
# Stage 1: Rust 빌드
# ============================================================
FROM rust:1.82-bookworm AS rust-builder

WORKDIR /app

# 의존성 캐싱을 위해 Cargo.toml만 먼저 복사
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY proto/ proto/

# 더미 main.rs로 의존성만 빌드
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# 실제 소스 복사 및 빌드
COPY src/ src/
RUN touch src/main.rs && cargo build --release

# ============================================================
# Stage 2: Python 환경
# ============================================================
FROM python:3.12-slim-bookworm AS python-env

WORKDIR /app/processing

COPY processing/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY processing/ .

# ============================================================
# Stage 3: 런타임
# ============================================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    python3 \
    python3-pip \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Rust 바이너리 복사
COPY --from=rust-builder /app/target/release/asset-manager /app/asset-manager

# Python 환경 복사
COPY --from=python-env /usr/local/lib/python3.12 /usr/local/lib/python3.12
COPY --from=python-env /usr/local/bin/python3 /usr/local/bin/python3
COPY --from=python-env /app/processing /app/processing

# 임시 작업 디렉토리
RUN mkdir -p /tmp/asset-converter

ENV GRPC_PORT=50051
ENV PYTHON_BIN=/usr/local/bin/python3
ENV CONVERTER_SCRIPT=/app/processing/converter.py
ENV WORK_DIR=/tmp/asset-converter

EXPOSE 50051

ENTRYPOINT ["/app/asset-manager"]
```

### 10.2 docker-compose.dev.yml

```yaml
version: '3.8'

services:
  asset-manager:
    build: .
    ports:
      - "50051:50051"
    environment:
      - GRPC_PORT=50051
      - DATABASE_URL=postgresql://amr:amr_password@postgres:5432/amr_assets
      - S3_ENDPOINT=http://minio:9000
      - AWS_ACCESS_KEY_ID=minioadmin
      - AWS_SECRET_ACCESS_KEY=minioadmin
      - S3_BUCKET=amr-assets
      - RUST_LOG=info
    depends_on:
      postgres:
        condition: service_healthy
      minio:
        condition: service_healthy

  postgres:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_USER=amr
      - POSTGRES_PASSWORD=amr_password
      - POSTGRES_DB=amr_assets
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U amr -d amr_assets"]
      interval: 5s
      timeout: 5s
      retries: 5

  minio:
    image: minio/minio:latest
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    volumes:
      - minio_data:/data
    command: server /data --console-address ":9001"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 10s
      timeout: 5s
      retries: 3

  # MinIO 초기 버킷 생성
  minio-init:
    image: minio/mc:latest
    depends_on:
      minio:
        condition: service_healthy
    entrypoint: >
      /bin/sh -c "
      mc alias set local http://minio:9000 minioadmin minioadmin;
      mc mb --ignore-existing local/amr-assets;
      echo 'MinIO bucket created';
      "

volumes:
  postgres_data:
  minio_data:
```

### 10.3 Cargo.toml

```toml
[package]
name = "asset-manager"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "asset-manager"
path = "src/main.rs"

[dependencies]
# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# Async runtime
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
async-stream = "0.3"
futures = "0.3"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# S3 / MinIO
aws-sdk-s3 = "1"
aws-config = "1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
config = "0.14"

# Error handling
thiserror = "1"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

[dev-dependencies]
testcontainers = "0.20"
tempfile = "3"
tokio-test = "0.4"

[build-dependencies]
tonic-build = "0.12"
```

### 10.4 build.rs

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/asset_service.proto"], &["proto"])?;
    Ok(())
}
```

### 10.5 Python requirements.txt

```
# processing/requirements.txt

# 메시 처리
trimesh>=4.0.0
numpy>=1.26.0

# URDF 파싱
urdfpy>=0.0.22

# glTF 처리
pygltflib>=1.16.0

# SDF XML 파싱
lxml>=5.0.0

# 이미지 처리
Pillow>=10.0.0

# DB 연결 (변환 진행률 업데이트용)
psycopg2-binary>=2.9.0

# S3 연결 (파일 다운로드/업로드)
boto3>=1.34.0

# gRPC (선택적, 향후 내부 통신용)
grpcio>=1.60.0
grpcio-tools>=1.60.0

# 테스트
pytest>=8.0.0
pytest-asyncio>=0.23.0
```

---

## 11. 모니터링 및 관측성

### 11.1 로깅 표준

모든 로그는 구조화된 JSON 형식으로 출력한다.

```json
{
  "timestamp": "2026-03-21T10:30:00Z",
  "level": "INFO",
  "target": "asset_manager::grpc::handlers",
  "message": "Asset created",
  "fields": {
    "asset_id": "uuid",
    "asset_type": "robot_model",
    "file_size_bytes": 12345,
    "duration_ms": 150
  }
}
```

**로그 레벨 가이드:**

| 레벨 | 용도 |
|---|---|
| `ERROR` | 요청 실패, 변환 실패, DB/S3 연결 오류 |
| `WARN` | 파일 크기 경고, 검증 경고, 비정상적 파라미터 |
| `INFO` | 에셋 생성/삭제, 변환 시작/완료, 서비스 시작 |
| `DEBUG` | 쿼리 실행, S3 요청 상세, 변환 단계 진행 |
| `TRACE` | gRPC 메시지 전체 내용, 메시 정점 데이터 |

### 11.2 메트릭 (Prometheus)

```
# gRPC 요청 메트릭
asset_manager_grpc_requests_total{method, status}
asset_manager_grpc_request_duration_seconds{method}

# 스토리지 메트릭
asset_manager_upload_bytes_total
asset_manager_download_bytes_total
asset_manager_storage_operations_total{operation, status}

# 변환 메트릭
asset_manager_conversion_jobs_total{status}
asset_manager_conversion_duration_seconds{source_format, target_format}
asset_manager_conversion_active_jobs

# 에셋 메트릭
asset_manager_assets_total{type}
asset_manager_asset_files_total{format}
asset_manager_total_storage_bytes
```

### 11.3 헬스체크

```protobuf
// gRPC Health Check (grpc.health.v1)
service Health {
  rpc Check(HealthCheckRequest) returns (HealthCheckResponse);
}
```

헬스체크 항목:
- PostgreSQL 연결 상태
- MinIO 연결 상태
- 디스크 공간 (작업 디렉토리)

---

## 12. 보안 고려사항

### 12.1 파일 업로드 보안

1. **파일 크기 제한**: 최대 500MB (gRPC 메시지 크기 제한과 별도)
2. **파일 타입 검증**: 확장자 + MIME 타입 + 매직 바이트 검사
3. **경로 순회 방지**: 파일명에서 `..`, `/`, `\` 제거
4. **체크섬 검증**: 업로드 완료 후 SHA256 체크섬 일치 확인

### 12.2 접근 제어

- 현재 버전에서는 gRPC 수준의 인증/인가를 별도로 구현하지 않음
- Backend(Team 2)가 인증을 담당하고, Asset Manager는 내부 서비스로만 접근 가능
- 네트워크 정책: Asset Manager는 내부 네트워크에서만 접근 가능

### 12.3 데이터 무결성

- 모든 파일에 SHA256 체크섬 저장 및 다운로드 시 검증
- DB 트랜잭션으로 메타데이터-파일 일관성 보장
- MinIO 버킷 버전 관리로 실수 삭제 방지 가능 (선택적 활성화)
