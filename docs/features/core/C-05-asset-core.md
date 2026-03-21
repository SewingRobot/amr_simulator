# C-05: Asset Core

> **Phase**: 1
> **담당 팀**: Team 4 (Asset Manager, 주도) + Team 2 (Backend, REST 프록시)
> **상태**: Draft
> **최종 수정일**: 2026-03-22

---

## 1. 개요

로봇 모델 및 환경 객체 에셋의 업로드·다운로드·메타데이터 CRUD와
기본 URDF→glTF 변환 기능을 구현하여, 시뮬레이션 및 3D 시각화에 필요한
에셋 관리 기반을 확립한다.

**핵심 흐름:**
```
Frontend → Backend REST (multipart) → Backend gRPC Client → Asset Manager gRPC Server → MinIO + PostgreSQL
```

---

## 2. 스코프

### 2.1 In-Scope

| # | 항목 | 설명 |
|---|------|------|
| 1 | gRPC AssetService 스켈레톤 | ListAssets, GetAsset, CreateAsset, DeleteAsset, DownloadAsset |
| 2 | 에셋 업로드 | chunked gRPC stream → MinIO 저장 + PostgreSQL 메타데이터 |
| 3 | 에셋 다운로드 | MinIO → gRPC stream → Backend REST 응답 |
| 4 | 메타데이터 CRUD | type, name, description, format, tags, properties (JSONB) |
| 5 | URDF→glTF 기본 변환 | Python: urdfpy + trimesh → glTF export |
| 6 | 에셋 유형 (Phase 1) | `robot_model`, `static_object` (2종만) |
| 7 | PostgreSQL 스키마 | assets, asset_files 테이블 |
| 8 | Backend REST 프록시 | `/api/assets/*` 엔드포인트 |

### 2.2 Out-of-Scope (A-09에서 구현)

- OpenUSD 변환 파이프라인 (URDF→USD, SDF→USD)
- Blender 헤드리스 변환
- 메시 최적화 (LOD 생성, Draco 압축, 텍스처 아틀라스)
- 에셋 버전 관리
- `sensor_definition`, `mission_template`, `traffic_rule`, `dynamic_object` 에셋 유형
- 에셋 번들 다운로드 (GetAssetBundle)

---

## 3. 상세 스펙

### 3.1 gRPC 서비스 정의 (Phase 1 범위)

```protobuf
service AssetService {
  // 에셋 목록 조회 (필터링, 페이지네이션)
  rpc ListAssets(ListAssetsRequest) returns (ListAssetsResponse);

  // 단일 에셋 메타데이터 조회
  rpc GetAsset(GetAssetRequest) returns (AssetDescriptor);

  // 에셋 생성 (메타데이터 + 파일 스트림)
  rpc CreateAsset(stream CreateAssetRequest) returns (AssetDescriptor);

  // 에셋 삭제
  rpc DeleteAsset(DeleteAssetRequest) returns (DeleteAssetResponse);

  // 에셋 파일 다운로드 (바이너리 스트림)
  rpc DownloadAsset(DownloadAssetRequest) returns (stream AssetChunk);
}
```

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#asset-service) 참조

### 3.2 업로드 흐름

```
1. Frontend: multipart/form-data POST → Backend REST
2. Backend: 메타데이터 추출 → gRPC CreateAsset stream 시작
3. 첫 번째 메시지: CreateAssetMetadata {name, type, description, format, tags, properties}
4. 이후 메시지: AssetChunk {data(bytes, 4MB max), chunk_index, is_last}
5. Asset Manager: MinIO에 파일 저장 → DB에 메타데이터 삽입
6. 응답: AssetDescriptor (생성된 에셋의 전체 메타데이터)
```

**파일 저장 경로 규칙:**
```
assets/{asset_type}/{asset_id}/{filename}
예: assets/robot_model/550e8400-.../turtlebot3.urdf
```

### 3.3 다운로드 흐름

```
1. Frontend: GET /api/assets/:id/download?format=glTF → Backend REST
2. Backend: gRPC DownloadAsset(asset_id, format) 호출
3. Asset Manager: MinIO에서 파일 읽기 → gRPC stream으로 4MB 청크 전송
4. Backend: 청크를 HTTP 응답 body로 스트리밍 (Content-Type 설정)
```

**지원 다운로드 포맷:**
- 원본 포맷 (URDF, STL, DAE 등)
- 변환된 포맷 (glTF/GLB — 변환 완료 시)

### 3.4 URDF→glTF 변환

Python 기반 변환 파이프라인. 에셋 업로드 시 `robot_model` 타입이면 자동 트리거.

**변환 단계:**

| 단계 | 처리 내용 | 라이브러리 | 비고 |
|------|----------|-----------|------|
| 1. URDF 파싱 | XML 파싱, 링크/조인트 트리 구성 | urdfpy | joint 정보 추출 |
| 2. 메시 로드 | STL/DAE/OBJ 메시 파일 로드 | trimesh | 상대 경로 해석 |
| 3. 트랜스폼 적용 | URDF origin → mesh transform | numpy | 좌표계 변환 |
| 4. 씬 결합 | 개별 메시 → 단일 씬 구성 | trimesh.Scene | 계층 구조 유지 |
| 5. glTF 내보내기 | 씬 → glTF 2.0 / GLB export | trimesh.export | 바이너리 GLB 선호 |
| 6. 결과 저장 | GLB 파일 → MinIO 업로드 | boto3 | asset_files에 추가 포맷 기록 |

**변환 상태 추적:**
```
conversion_status: pending → processing → completed | failed
```

에셋 메타데이터의 `properties.conversion_status` 필드로 조회 가능.

### 3.5 메타데이터 스키마

```json
{
  "id": "UUID",
  "type": "robot_model | static_object",
  "name": "TurtleBot3 Burger",
  "description": "TurtleBot3 Burger URDF model",
  "formats": [
    {"format": "urdf", "storage_path": "assets/robot_model/.../turtlebot3.urdf", "size_bytes": 24560},
    {"format": "glb",  "storage_path": "assets/robot_model/.../turtlebot3.glb",  "size_bytes": 102400}
  ],
  "tags": ["turtlebot", "differential-drive", "indoor"],
  "properties": {
    "conversion_status": "completed",
    "original_format": "urdf",
    "mesh_count": 5,
    "total_vertices": 12340
  },
  "created_at": "2026-03-22T10:00:00Z",
  "updated_at": "2026-03-22T10:05:00Z"
}
```

### 3.6 데이터베이스 스키마

| 테이블 | 주요 컬럼 | 인덱스 |
|--------|----------|--------|
| `assets` | id (UUID PK), asset_type (CHECK), name, description, tags (TEXT[]), properties (JSONB), created_at, updated_at | type, name (GIN), tags (GIN) |
| `asset_files` | id (UUID PK), asset_id (FK CASCADE), format, storage_path, size_bytes, checksum_sha256, created_at | asset_id, UNIQUE(asset_id, format) |

`pg_trgm` 확장 필요 (이름 부분 검색용).

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#asset-data) 참조

---

## 4. 구현 모듈

### 4.1 Asset Manager (Team 4, Rust)

| 모듈 | 파일 경로 | 설명 |
|------|----------|------|
| gRPC 서버 | `src/grpc/server.rs` | Tonic gRPC 서버 부트스트랩 |
| Asset 핸들러 | `src/grpc/asset_handlers.rs` | ListAssets, GetAsset, CreateAsset, DeleteAsset, DownloadAsset |
| Asset 리포지토리 | `src/db/asset_repository.rs` | assets, asset_files 테이블 CRUD (SQLx) |
| S3 스토리지 | `src/storage/s3_client.rs` | MinIO 업로드/다운로드 래퍼, 스트리밍 지원 |
| 변환 러너 | `src/pipeline/conversion_runner.rs` | Python 변환 subprocess 실행, 상태 추적 |
| URDF→glTF | `python/urdf_to_gltf.py` | urdfpy + trimesh 기반 변환 스크립트 |

### 4.2 Backend (Team 2, Rust)

| 모듈 | 파일 경로 | 설명 |
|------|----------|------|
| Asset REST 프록시 | `src/api/routes/assets.rs` | `/api/assets/*` REST 엔드포인트 (multipart 업로드 포함) |
| gRPC 클라이언트 | `src/grpc/asset_client.rs` | AssetService gRPC 클라이언트 래퍼 |

**REST 엔드포인트:** `GET /api/assets`, `GET /api/assets/:id`, `POST /api/assets` (multipart), `DELETE /api/assets/:id`, `GET /api/assets/:id/download`

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#team4-interfaces) 참조

---

## 5. 의존성

### 5.1 인프라 의존성

| 구성 요소 | 용도 | 비고 |
|-----------|------|------|
| PostgreSQL 16 | 에셋 메타데이터, 파일 정보 저장 | `pg_trgm` 확장 필요 (이름 검색) |
| MinIO | 에셋 바이너리 파일 저장 | S3 호환 API |
| Python 3.12+ | URDF→glTF 변환 파이프라인 | urdfpy, trimesh, numpy, pygltflib |

### 5.2 팀 간 의존성

| 의존 대상 | 방향 | 내용 |
|-----------|------|------|
| Team 6 (Proto) | 공유 | `.proto` 파일 정의 (`asset_service.proto`) |
| Team 2 (Backend) | Backend → Asset Manager | gRPC 클라이언트로 Asset Manager 호출 |
| Team 1 (Frontend) | 소비자 | REST API로 에셋 조회/업로드, glTF 모델 렌더링 |
| Team 3 (Sim Engine) | 소비자 | URDF/SDF 모델 로드, 시뮬레이션에 활용 |

---

## 6. 테스트 기준

### 6.1 단위 테스트

| 대상 | 테스트 항목 | 기대 결과 |
|------|-----------|----------|
| ListAssets | 빈 DB 조회 | 빈 배열 반환, total_count = 0 |
| CreateAsset | 메타데이터 + 파일 업로드 | AssetDescriptor 반환, MinIO 파일 존재 |
| GetAsset | 존재하지 않는 ID 조회 | NOT_FOUND 에러 |
| DeleteAsset | 에셋 삭제 후 조회 | DB + MinIO 모두 삭제 확인 |
| CreateAsset | 지원하지 않는 타입 | INVALID_ARGUMENT 에러 |
| ListAssets | type 필터 적용 | 해당 타입만 반환 |

### 6.2 통합 테스트

| 시나리오 | 설명 | 성공 기준 |
|---------|------|----------|
| 업로드 E2E | URDF 파일 업로드 → 메타데이터 확인 → 다운로드 → 내용 일치 | SHA256 체크섬 일치 |
| URDF→glTF 변환 | TurtleBot3 URDF 업로드 → 자동 glTF 변환 | formats에 "glb" 추가, 다운로드 가능 |
| Backend 프록시 | REST multipart 업로드 → gRPC 전달 확인 | HTTP 201, 응답 JSON 일치 |
| 대용량 업로드 | 50MB 메시 파일 chunked 업로드 | OOM 없이 완료 |
| 태그 필터링 | tags=["indoor"] 필터 적용 | 매칭 에셋만 반환 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#urdf-samples) 참조

### 6.3 성능 기준

| 항목 | 목표 |
|------|------|
| ListAssets 응답 | < 50ms (p95, 1000개 에셋) |
| GetAsset 응답 | < 20ms (p95) |
| 파일 다운로드 처리량 | > 50MB/s (로컬 MinIO) |
| URDF→glTF 변환 (단순 모델) | < 30초 |

---

## 7. 완료 조건

- [ ] gRPC AssetService Phase 1 RPC 전체 구현 및 테스트 통과
- [ ] 에셋 업로드 (chunked stream) → MinIO + DB 저장 동작
- [ ] 에셋 다운로드 (gRPC stream) → 바이너리 파일 반환 동작
- [ ] 메타데이터 CRUD (type, name, tags 필터링 포함) 동작
- [ ] URDF→glTF 기본 변환 성공 (TurtleBot3 등 레퍼런스 모델)
- [ ] PostgreSQL 마이그레이션 (assets, asset_files) 적용 완료
- [ ] Backend REST 프록시 (`/api/assets/*`) 동작 확인
- [ ] SHA256 체크섬 검증 동작
- [ ] Docker Compose로 Asset Manager + PostgreSQL + MinIO 통합 테스트 통과
- [ ] 50MB 파일 업로드/다운로드 E2E 성공
