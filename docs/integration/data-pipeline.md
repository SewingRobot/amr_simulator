# 데이터 관리 파이프라인

이 문서는 AMR 통합 프레임워크 전체의 데이터 생명주기(수집 → 저장 → 처리 → 서빙 → 아카이빙)를 정의한다.

> **관련 문서**
> - [아키텍처 개요](../architecture.md) — 전체 시스템 아키텍처 및 데이터 흐름
> - [테스트 데이터 명세서](test-data-spec.md) — 더미 데이터 구성 및 생성 방법
> - [통합 인터페이스 명세](integration-spec.md) — 크로스팀 인터페이스 계약

---

## 1. 데이터 흐름 전체 맵

```
[Data Sources]                    [Processing]              [Storage]               [Serving]

Real Robot Sensors ──┐
                     ├─ MQTT ──→ Backend ──→ TimescaleDB ──→ REST API (historical)
Sim Engine ──────────┤         (normalize)    (telemetry)    WebSocket (real-time)
                     │
3D Scanner ──────────┼─ Upload ──→ Map Manager ──→ MinIO ──→ Potree Tiles (HTTP)
                     │         (Open3D pipeline) (octree)    gRPC (metadata)
                     │
CAD/URDF Files ──────┼─ Upload ──→ Asset Manager ──→ MinIO ──→ gRPC (download)
                     │         (conversion)       (models)    REST (Frontend proxy)
                     │
User Input ──────────┘─ REST ──→ Backend ──→ PostgreSQL ──→ REST API
  (missions, maps,                          (relational)    WebSocket (updates)
   settings)
```

---

## 2. 데이터 유형별 파이프라인

### 2.1 텔레메트리 데이터 파이프라인

텔레메트리는 로봇의 실시간 상태 데이터이며, 시스템에서 가장 높은 빈도로 흐르는 데이터이다. 실제 로봇과 시뮬레이션 엔진 모두 동일한 `TelemetryMessage` 포맷을 생성한다 (mode-agnostic).

```
[수집]                  [정규화]            [팬아웃]              [저장]           [조회]
Robot ─── MQTT ──┐                     ┌── Redis Pub/Sub ──→ WebSocket ──→ Browser
                 ├─→ Backend ──────────┤
Sim ──── gRPC ───┘   (TelemetryMsg     ├── TimescaleDB ──→ REST API (시간 범위)
          Stream      통일 포맷 변환)    └── 메모리 캐시 ───→ 최신 상태 조회
```

**단계별 상세:**

| 단계 | 구현 | 설명 |
|------|------|------|
| **수집** | MQTT (VDA5050 실로봇) 또는 gRPC Stream (Sim Engine) | Sim Engine은 `StreamTelemetry` RPC로 서버 스트리밍. 실로봇은 VDA5050 State 토픽 구독 |
| **정규화** | Backend Telemetry Service | VDA5050 State → `TelemetryMessage` 변환. gRPC 스트림은 이미 동일 포맷 |
| **실시간 팬아웃** | Redis Pub/Sub → WebSocket Relay | `telemetry:{robot_id}` 채널로 발행. WebSocket 릴레이가 구독하여 브라우저 클라이언트에 전달 |
| **저장** | TimescaleDB hypertable | `robot_telemetry` 테이블, `timestamp` 기준 시간 파티셔닝 |
| **보존 정책** | 원본 30일 보관, 1분 평균 다운샘플은 영구 보존 | TimescaleDB continuous aggregate 활용 |
| **압축** | 7일 이후 TimescaleDB 네이티브 압축 적용 | 스토리지 60~90% 절감 |
| **조회** | 시간 범위 쿼리 (REST API), 리플레이 (WebSocket) | `GET /api/telemetry?robot_id=X&from=T1&to=T2` |

**TimescaleDB 스키마:**

```sql
CREATE TABLE robot_telemetry (
    time        TIMESTAMPTZ NOT NULL,
    robot_id    TEXT NOT NULL,
    pos_x       DOUBLE PRECISION,
    pos_y       DOUBLE PRECISION,
    pos_z       DOUBLE PRECISION,
    orient_x    DOUBLE PRECISION,
    orient_y    DOUBLE PRECISION,
    orient_z    DOUBLE PRECISION,
    orient_w    DOUBLE PRECISION,
    vel_linear_x  DOUBLE PRECISION,
    vel_linear_y  DOUBLE PRECISION,
    vel_angular_z DOUBLE PRECISION,
    battery_pct   REAL,
    battery_volt  REAL,
    odometry_dist DOUBLE PRECISION
);

SELECT create_hypertable('robot_telemetry', 'time');

-- 보존 정책: 원본 30일
SELECT add_retention_policy('robot_telemetry', INTERVAL '30 days');

-- 압축: 7일 이후
ALTER TABLE robot_telemetry SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'robot_id',
    timescaledb.compress_orderby = 'time DESC'
);
SELECT add_compression_policy('robot_telemetry', INTERVAL '7 days');

-- 1분 평균 다운샘플 (영구 보존)
CREATE MATERIALIZED VIEW telemetry_1min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 minute', time) AS bucket,
    robot_id,
    AVG(pos_x) AS avg_pos_x,
    AVG(pos_y) AS avg_pos_y,
    AVG(battery_pct) AS avg_battery_pct,
    MAX(odometry_dist) AS max_odometry_dist
FROM robot_telemetry
GROUP BY bucket, robot_id;
```

### 2.2 포인트 클라우드 데이터 파이프라인

포인트 클라우드는 3D 스캔 데이터이며, 수억 포인트 규모의 대용량 파일을 스트리밍 처리한다.

```
[수집]              [임시 저장]      [처리 파이프라인]           [최종 저장]        [서빙]
LAS/PLY/PCD ──→ gRPC Stream ──→ MinIO temp/ ──→ Python Pipeline ──→ MinIO tiles/ ──→ HTTP Range
                (chunked 4MB)                   (Open3D + Potree)     (octree)        (Potree Viewer)
```

**Python 처리 파이프라인 단계:**

| 단계 | 모듈 | 설명 | 실패 시 |
|------|------|------|--------|
| 1. Ingestion | `ingest.py` | LAS/PLY/PCD → Open3D PointCloud 변환 | Job 실패, 원본 보존 |
| 2. Validation | `validate.py` | 좌표 범위 체크 (±10,000m), 포인트 수 확인 | Job 실패, 에러 메시지 반환 |
| 3. Coordinate Normalize | `normalize.py` | 좌표 오프셋 제거, ENU 변환 (필요 시) | Job 실패 |
| 4. Downsample | `downsample.py` | Voxel 다운샘플링 (옵션, 원본 보존) | 스킵 가능 |
| 5. Filter | `filter.py` | Statistical outlier 제거 | 스킵 가능 |
| 6. Normals | `normals.py` | 법선 벡터 추정 | 스킵 가능 |
| 7. Potree Convert | `potree_convert.py` | PotreeConverter 2.1 실행 → 옥트리 타일 생성 | Job 실패 |
| 8. Upload | `s3_upload.py` | Potree 타일 → MinIO `tiles/{map_id}/` 업로드 | 재시도 3회 |
| 9. Metadata | `metadata.py` | PostgreSQL에 bounds, point_count, LOD levels, offset 기록 | 재시도 3회 |

**MinIO 경로 구조:**

```
pointclouds/
├── raw/{map_id}/{upload_id}/           # 원본 파일
├── temp/{map_id}/{job_id}/             # 처리 중 임시 파일
└── tiles/{map_id}/v{version}/          # Potree 옥트리 타일
    ├── metadata.json
    ├── octree.bin
    └── hierarchy.bin
```

**서빙:** Frontend의 Potree Viewer가 Backend를 통해 HTTP Range 요청으로 LOD별 타일을 프로그레시브 로딩한다. Backend는 MinIO에서 타일을 프록시하며, Redis 캐시를 통해 10ms 이내 응답을 보장한다.

**버전 관리:** 새 업로드 시 새 `v{version}` 경로를 생성하고, 이전 버전의 타일은 보존한다. PostgreSQL의 `pointcloud_data` 테이블에서 활성 버전을 관리한다.

### 2.3 에셋 데이터 파이프라인

에셋은 로봇 모델(URDF/SDF/glTF/USD), 환경 객체 메시 등의 3D 파일이다.

```
[수집]              [검증]          [변환]              [저장]              [서빙]
URDF/SDF ──→ Upload API ──→ Format ──→ Conversion ──→ MinIO + PostgreSQL ──→ gRPC Stream
glTF/USD       (gRPC)      Validate    Pipeline         assets/{type}/       (download)
                                                        {id}/{format}/       REST (proxy)
```

**검증 규칙:**
- URDF: XML 스키마 검증, 모든 mesh 파일 참조 존재 확인
- SDF: XML 스키마 검증, 물리 속성 유효성 확인
- glTF: glTF 2.0 스펙 준수, 바이너리 버퍼 유효성
- 파일 크기: 500MB 이하

**변환 파이프라인:**

| Phase | 변환 | 도구 | 용도 |
|-------|------|------|------|
| Phase 1 | URDF → glTF | 커스텀 Python 변환기 | 웹 3D 렌더링 (Three.js/R3F) |
| Phase 1 | URDF → USD | libsdformat + USD SDK | Sim Engine 물리 시뮬레이션 |
| Phase 3 | SDF → USD | libsdformat + USD SDK | 확장 씬 포맷 지원 |
| Phase 3 | USD → glTF | USD SDK + glTF exporter | 웹 렌더링 (USD 네이티브 에셋) |

**MinIO 경로 구조:**

```
assets/
├── robot_model/{asset_id}/
│   ├── urdf/     # 원본 URDF + mesh 파일
│   ├── gltf/     # 변환된 glTF
│   └── usd/      # 변환된 USD
├── environment/{asset_id}/
│   ├── usd/
│   └── gltf/
└── thumbnail/{asset_id}.png  # 미리보기 이미지 (자동 생성)
```

### 2.4 맵 구조 데이터 파이프라인

맵 데이터는 로드맵 그래프, 시맨틱 레이어, 장애물의 3가지 하위 유형으로 구성된다.

```
[수집]                [저장]               [서빙]              [소비자]
REST/gRPC CRUD ──→ PostgreSQL + PostGIS ──→ gRPC/REST ──→ Backend/Frontend
  (로드맵 그래프)     roadmap_nodes           (메타데이터)      Mission Service
  (시맨틱 영역)       roadmap_edges                            Traffic Service
  (정적 장애물)       semantic_regions                         Sim Engine
                     obstacles

로봇 센서 ──→ gRPC ──→ Redis (TTL) ──→ gRPC Stream ──→ Traffic Service
  (동적 장애물)        dynamic:{map_id}:     (실시간 조회)      Pathfinding
                      {obstacle_id}
```

**로드맵 그래프:**
- CRUD: `AddNode`, `UpdateNode`, `DeleteNode`, `AddEdge`, `UpdateEdge`, `DeleteEdge` gRPC RPC
- 버전 관리: 그래프 변경 시 `version` 필드 증가. 이전 버전 조회 가능
- 경로 탐색: A*/Dijkstra 기반. 10,000 노드 그래프에서 5ms 이내 목표
- 외부 import: JSON 포맷으로 로드맵 일괄 import 지원 (`UpdateRoadmapRequest`)

**시맨틱 레이어:**
- PostGIS `POLYGON` 타입으로 저장
- 공간 인덱스(GiST)를 활용한 10ms 이내 point-in-polygon 쿼리
- 영역 유형: no_go_zone, speed_limit_zone, charging_area, loading_dock, waiting_area, one_way_zone, restricted_area

**장애물 레이어:**
- 정적 장애물: PostgreSQL 영구 저장, CRUD API
- 동적 장애물: Redis에 TTL 기반 저장 (기본 10초), 로봇 센서가 감지 → 자동 갱신/소멸
- 통합 조회: `GetObstacles` RPC에서 `include_dynamic=true`로 Redis + PostgreSQL 합산 결과 반환

### 2.5 미션/운영 데이터 파이프라인

```
[생성]          [할당]           [실행]              [완료]          [이력]
REST API ──→ Mission ──→ Robot ──→ VDA5050 Order ──→ State ──→ PostgreSQL
  (create)    Service    Assign    MQTT publish      Update    (영구 보존)
              (Backend)            ↓                  ↑
                                  Real Robot ─── MQTT State ──┘
                                  Sim Engine ─── gRPC Stream ─┘
```

**미션 상태 머신:**

```
PENDING → ASSIGNED → IN_PROGRESS → COMPLETED
                   ↘ PAUSED ↗    ↘ FAILED
                   ↘ CANCELLED
```

**VDA5050 Order 변환:**
- Mission의 `waypoints`를 VDA5050 `nodes`/`edges`로 변환
- 로드맵 그래프의 노드 좌표를 VDA5050 `nodePosition`으로 매핑
- `sequenceId`는 0, 1, 2, ... 순서 (nodes: 짝수, edges: 홀수)

**상태 변경 알림:**
- PostgreSQL UPDATE 트리거 → Backend WebSocket → Frontend 실시간 갱신
- 미션 완료/실패 시 분석용 이력 영구 보존

---

## 3. 스토리지 계층 요약

| 스토리지 | 데이터 유형 | 접근 패턴 | 보존 기간 | 용량 추정 (L 규모) |
|----------|-----------|----------|----------|-------------------|
| **PostgreSQL 16** | 관계형 메타데이터, 맵, 로드맵, 시맨틱, 미션, 사용자 | CRUD, PostGIS 공간 쿼리 | 영구 | ~10 GB |
| **TimescaleDB** | 텔레메트리 시계열 | 시간 범위 쿼리, continuous aggregate | 원본 30일, 다운샘플 영구 | ~50 GB/month (원본) |
| **Redis 7.2+** | 실시간 상태, 존 잠금, 동적 장애물, 타일 캐시 | 키-값 조회, pub/sub, TTL | 일시적 | ~1 GB |
| **MinIO (S3)** | 바이너리 파일 (에셋, Potree 타일, 원본 스캔) | 파일 업/다운로드, HTTP Range | 영구 (버전별) | ~100 GB+ |

### 3.1 PostgreSQL 핵심 테이블

```
maps                  ← 맵 메타데이터
pointcloud_data       ← 포인트 클라우드 처리 결과 메타데이터
roadmap_nodes         ← 로드맵 경유점 (PostGIS POINT)
roadmap_edges         ← 로드맵 경로 (거리, 속도, 방향)
semantic_regions      ← 시맨틱 영역 (PostGIS POLYGON)
obstacles             ← 정적 장애물
assets                ← 에셋 메타데이터
missions              ← 미션 정의 및 상태
robots                ← 로봇 등록 정보
users                 ← 사용자/인증
```

### 3.2 Redis 키 구조

```
telemetry:{robot_id}              ← 최신 텔레메트리 (HASH)
robot:status:{robot_id}           ← 로봇 상태 (STRING, JSON)
zone:lock:{zone_id}               ← 존 잠금 (STRING, robot_id, TTL)
dynamic:{map_id}:{obstacle_id}    ← 동적 장애물 (HASH, TTL 10초)
tile:cache:{map_id}:{node_id}     ← Potree 타일 캐시 (BYTES, TTL 1시간)
mission:active:{robot_id}         ← 활성 미션 ID (STRING)
```

---

## 4. 데이터 무결성 및 검증

### 4.1 입력 검증 규칙

모든 데이터는 수집 시점에 검증되며, 검증 실패 시 명확한 에러 메시지와 함께 거부된다.

| 데이터 유형 | 검증 항목 | 제한 |
|------------|----------|------|
| 포인트 클라우드 | 좌표 범위 | ±10,000m |
| 포인트 클라우드 | 파일 크기 | < 10GB |
| 포인트 클라우드 | 지원 포맷 | LAS, LAZ, PLY, PCD |
| URDF | XML 스키마 | 유효한 URDF XML |
| URDF | mesh 참조 | 모든 참조 mesh 파일이 패키지 내 존재 |
| URDF | 물리 속성 | mass > 0, inertia 양의 정부호 |
| 로드맵 | edge 참조 | source/target node 존재 확인 |
| 로드맵 | distance | 양수, 노드 간 유클리드 거리와 일치 (오차 1%) |
| 미션 | robot_id | 등록된 로봇 ID 존재 확인 |
| 미션 | map_id | 등록된 맵 ID 존재 확인 |
| 미션 | node_id | start/goal 노드가 로드맵에 존재 |
| VDA5050 | 스펙 버전 | v2.0.0 |

### 4.2 검증 API

```
POST /api/validate/pointcloud    → ValidationResult
POST /api/validate/urdf          → ValidationResult
POST /api/validate/roadmap       → ValidationResult
POST /api/validate/mission       → ValidationResult
```

**ValidationResult:**

```json
{
  "valid": false,
  "errors": [
    {"field": "edges[2].distance", "message": "Distance 3.5 does not match Euclidean distance 4.0 between nodes", "severity": "error"},
    {"field": "nodes[0].position.z", "message": "Z coordinate should be 0 for 2D roadmap", "severity": "warning"}
  ]
}
```

### 4.3 실 데이터 온보딩 체크리스트

| 항목 | 확인 내용 | 자동 검증 가능 |
|------|----------|:-------------:|
| 좌표계 | 포인트 클라우드가 ENU 좌표계인지 확인. 아닌 경우 변환 필요 | 부분 |
| 단위 | URDF/SDF 모델이 미터 단위인지 확인. 밀리미터일 경우 스케일 변환 | ✅ |
| 범위 정합성 | 로드맵 노드 좌표가 포인트 클라우드 바운딩 박스 내에 있는지 | ✅ |
| 파일 크기 | 에셋 500MB 이하, 포인트 클라우드 10GB 이하 | ✅ |
| 참조 무결성 | 에지의 source/target 노드 존재, 미션의 노드 참조 유효 | ✅ |
| mesh 파일 | URDF가 참조하는 모든 mesh(STL/DAE) 파일이 패키지에 포함 | ✅ |

---

## 5. 실시간 데이터 흐름 성능 요구사항

| 구간 | 지표 | 목표값 |
|------|------|--------|
| Sim Engine → Backend | gRPC 스트림 지연 | < 5ms (로컬), < 20ms (네트워크) |
| Backend → Frontend | WebSocket 지연 | < 50ms (E2E) |
| Backend → TimescaleDB | 쓰기 처리량 | 10,000 rows/sec (50대 로봇 @ 200Hz) |
| Potree 타일 서빙 | 응답 시간 | < 10ms (Redis 캐시), < 50ms (MinIO) |
| A* 경로 탐색 | 응답 시간 | < 5ms (10K 노드) |
| PostGIS 공간 쿼리 | 응답 시간 | < 10ms (point-in-polygon) |
| 동적 장애물 갱신 | Redis 쓰기/읽기 | < 1ms |

---

## 6. 백업 및 재해 복구

### 6.1 백업 전략

| 스토리지 | 백업 방식 | 주기 | 보존 기간 |
|----------|----------|------|----------|
| PostgreSQL | WAL 아카이빙 + 일일 풀 백업 (pg_dump) | 연속 WAL + 일일 | WAL 7일, 풀 백업 30일 |
| TimescaleDB | PostgreSQL과 동일 (동일 인스턴스) | 동일 | 동일 |
| MinIO | 버전 관리 활성화, 크로스-리전 복제 (프로덕션) | 실시간 | 영구 (버전별) |
| Redis | RDB 스냅샷 | 1시간 | 24시간 |

### 6.2 복구 절차

```
1. PostgreSQL 복원 (WAL replay 또는 pg_restore)
2. MinIO 데이터 확인 (버전 관리로 자동 보존)
3. Redis 재시작 (비필수 — 유실 시 재생성 가능)
   - 동적 장애물: 로봇 센서가 재감지
   - 타일 캐시: 요청 시 자동 캐시 재구축
   - 실시간 상태: 텔레메트리 스트림에서 자동 복원
4. 서비스 재시작 순서: PostgreSQL → MinIO → Redis → Backend → Sim Engine
5. 검증: E2E 텔레메트리 흐름 확인, 타일 서빙 확인
```

### 6.3 데이터 유실 영향도

| 스토리지 | 유실 시 영향 | 복구 난이도 |
|----------|-------------|:----------:|
| PostgreSQL | 맵, 로드맵, 미션, 에셋 메타데이터 손실. 서비스 불가 | 높음 |
| TimescaleDB | 텔레메트리 이력 손실. 분석/리플레이 불가, 운영에는 영향 없음 | 중간 |
| MinIO | 에셋 파일, Potree 타일 손실. 재업로드 필요 | 높음 |
| Redis | 일시적 데이터만 유실. 자동 복원 가능 | 낮음 |

---

## 7. 테스트 데이터와 파이프라인 연동

### 7.1 테스트 데이터 흐름

테스트 데이터 명세서(`test-data-spec.md`)에서 정의한 더미 데이터는 실 데이터와 동일한 파이프라인을 통과한다. 이를 통해 파이프라인의 정확성을 검증한다.

```
test-data/fixtures/s/ ──→ 동일한 수집 API ──→ 동일한 처리 ──→ 동일한 저장소
                          (gRPC, REST)        파이프라인       (PostgreSQL, MinIO, Redis)
```

### 7.2 환경별 데이터 소스 전환

```bash
# 테스트 데이터 (CI/개발 환경)
DATA_SOURCE=test     # test-data/fixtures/ 에서 로드
DATA_SIZE=s          # Small 등급 사용

# 실 데이터 (스테이징/프로덕션)
DATA_SOURCE=real     # data/ 디렉토리 또는 외부 스토리지
```

### 7.3 파이프라인 테스트 시나리오

| 시나리오 | 데이터 등급 | 검증 포인트 |
|----------|-----------|------------|
| 포인트 클라우드 업로드 → Potree 변환 | S (10K pts) | 처리 완료, 타일 생성, 메타데이터 기록 |
| 텔레메트리 E2E 흐름 | S (600 frames) | gRPC → WebSocket → 브라우저 수신 확인 |
| 로드맵 CRUD + 경로 탐색 | M (50 nodes) | CRUD 정상, A* 경로 결과 유효 |
| 다중 로봇 미션 실행 | M (10 robots) | 미션 상태 전이, VDA5050 Order 생성 |
| 대규모 텔레메트리 쿼리 | L (30K frames) | TimescaleDB 시간 범위 쿼리 성능 |
| Potree 타일 서빙 성능 | M (1M pts) | Redis 캐시 적중률, 응답 시간 |
