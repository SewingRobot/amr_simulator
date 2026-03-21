# 테스트 데이터 명세서

이 문서는 각 기능의 테스트에 필요한 더미 데이터의 구성, 생성 방법, 의미적 요구사항을 정의한다. 추후 이 명세를 기반으로 테스트 데이터를 자동 생성하거나 실 데이터와 교체할 수 있어야 한다.

> **관련 문서**
> - [아키텍처 개요](../architecture.md) — 전체 데이터 흐름
> - [데이터 관리 파이프라인](data-pipeline.md) — 데이터 생명주기
> - [통합 인터페이스 명세](integration-spec.md) — 크로스팀 인터페이스 계약

---

## 1. 테스트 데이터 설계 원칙

- **포맷 준수 (Format Compliance):** 모든 더미 데이터는 proto/JSON/파일 포맷 스펙을 정확히 준수해야 한다
- **의미적 유효성 (Semantic Validity):** 데이터 간 참조 무결성이 유지되어야 한다 (예: mission.robot_id는 실제 존재하는 robot의 id)
- **물리적 타당성 (Physical Plausibility):** 시뮬레이션/렌더링에 사용되는 데이터는 물리적으로 타당해야 한다 (예: 로봇 속도가 0~2m/s 범위, 좌표가 맵 범위 내)
- **재현성 (Reproducibility):** 시드 기반 생성으로 동일 데이터를 반복 생성 가능해야 한다
- **확장성 (Scalability):** 소규모(단위 테스트) → 중규모(통합 테스트) → 대규모(부하 테스트) 데이터셋을 생성할 수 있어야 한다

### 1.1 포맷 준수 vs 의미 필수 구분

| 수준 | 설명 | 예시 |
|------|------|------|
| **포맷만 준수** | 필드명, 타입, 구조가 올바르면 충분. 값의 의미는 무관 | WebSocket 파싱 테스트, CRUD API 테스트 |
| **의미 필수** | 값이 물리적/논리적으로 유효해야 함. 참조 무결성 필요 | 물리 시뮬레이션, 경로 탐색, 렌더링 정합성 테스트 |

---

## 2. 데이터 유형별 명세

### 2.1 로봇 모델 데이터

**용도:** C-01 (Sim Engine Core), C-02 (3D Viewer Core), C-05 (Asset Core)

| 항목 | 포맷만 준수 | 의미 필수 | 설명 |
|------|:----------:|:--------:|------|
| glTF 로봇 모델 | ✅ | ❌ | 렌더링 테스트에는 단순 박스 모델도 가능. 메시 구조만 올바르면 됨 |
| URDF 로봇 모델 | ❌ | ✅ | 물리 시뮬에 사용하므로 link/joint 구조, 질량, 관성 모멘트가 물리적으로 유효해야 함 |
| USD 로봇 모델 | ❌ | ✅ | Sim Engine의 씬 포맷. UsdPhysics schema 포함, URDF→USD 변환 결과와 호환 |
| 로봇 설정 JSON | ❌ | ✅ | wheel_radius, wheel_separation 등이 물리적으로 유효한 값이어야 함 |

**데이터셋:** `test_robot_simple.urdf` (2-wheel diff drive, 0.5x0.3x0.2m 박스, 2D LiDAR 1개), `test_robot_simple.usda` (USD 변환), `test_robot_simple.glb` (glTF 변환)

**로봇 설정 JSON:**
```json
{
  "id": "test-robot-001",
  "name": "TestBot",
  "model_id": "simple_diff_drive",
  "drive_type": "differential",
  "wheel_radius": 0.05,
  "wheel_separation": 0.3,
  "max_linear_speed": 2.0,
  "max_angular_speed": 3.14,
  "collision_radius": 0.25,
  "mass_kg": 50.0,
  "initial_battery": 100.0,
  "sensors": [
    {
      "type": "lidar_2d",
      "name": "front_lidar",
      "update_rate": 10.0,
      "mount_pose": {"x": 0.2, "y": 0.0, "z": 0.15, "roll": 0, "pitch": 0, "yaw": 0},
      "params": {"angle_min": -3.14159, "angle_max": 3.14159, "range_min": 0.1, "range_max": 30.0, "ray_count": 720}
    }
  ]
}
```

**URDF 요구사항:** `base_link` (박스, mass=50kg, 유효한 inertia), `left_wheel`/`right_wheel` (cylinder, radius=0.05m, continuous joint), `lidar_link` (fixed joint, offset +0.2m X, +0.15m Z). 모든 link에 collision/inertial 필수.

### 2.2 포인트 클라우드 데이터

**용도:** C-04 (Map Core), C-02 (3D Viewer Core)

| 항목 | 포맷만 준수 | 의미 필수 | 설명 |
|------|:----------:|:--------:|------|
| 소규모 PLY (10K pts) | ✅ | ❌ | 파이프라인 테스트용. 바운딩 박스 내 랜덤 분포 |
| 중규모 PLY (1M pts) | ❌ | ✅ | 통합 테스트용. 직사각형 방 형상의 구조화된 포인트 |
| 대규모 LAS (100M pts) | ❌ | ✅ | 성능 테스트용. 실제 창고 스캔과 유사한 밀도/분포 |

**좌표계:** 오른손 좌표계 (ROS REP-103), X-전방, Y-좌측, Z-상방. 단위: 미터.

**생성 스크립트 (중규모 방 형태):**
```python
import numpy as np
import open3d as o3d

def create_room_pointcloud(
    width: float = 20.0, depth: float = 15.0, height: float = 3.0,
    density: int = 2000, rng: np.random.Generator = None,
) -> o3d.geometry.PointCloud:
    """6개 면(바닥, 천장, 벽 4개)에 균등 분포 포인트를 배치한다."""
    if rng is None:
        rng = np.random.default_rng(42)
    pcd = o3d.geometry.PointCloud()
    floor = rng.uniform([0, 0, 0], [width, depth, 0.001], size=(density, 3))
    ceiling = rng.uniform([0, 0, height - 0.001], [width, depth, height], size=(density, 3))
    wall_n = rng.uniform([0, 0, 0], [width, 0.001, height], size=(density, 3))
    wall_s = rng.uniform([0, depth - 0.001, 0], [width, depth, height], size=(density, 3))
    wall_e = rng.uniform([width - 0.001, 0, 0], [width, depth, height], size=(density, 3))
    wall_w = rng.uniform([0, 0, 0], [0.001, depth, height], size=(density, 3))
    all_points = np.concatenate([floor, ceiling, wall_n, wall_s, wall_e, wall_w])
    pcd.points = o3d.utility.Vector3dVector(all_points)
    return pcd
```

**검증 기준:** 좌표 범위 절대값 10,000m 이내, PLY 헤더 포인트 수 일치, 파일 크기 10GB 미만.

### 2.3 로드맵 그래프 데이터

**용도:** C-04 (Map Core), A-01 (Mission System), A-02 (Traffic Management)

| 항목 | 포맷만 준수 | 의미 필수 | 설명 |
|------|:----------:|:--------:|------|
| 단순 그래프 (5 nodes) | 부분 | ✅ | 에지 거리가 노드 간 유클리드 거리와 일치 |
| 창고 그래프 (20 nodes) | ❌ | ✅ | 격자형 경유점, 충전소, 적재독, 대기점 포함 |
| 대규모 그래프 (200+ nodes) | ❌ | ✅ | A* 성능 테스트용 |

**Protobuf 매핑:** `proto/map/map_service.proto`의 `RoadmapNode`, `RoadmapEdge`, `EdgeDirection` enum에 대응.

**단순 그래프 (5 nodes):**
```json
{
  "map_id": "test-map-001",
  "version": 1,
  "nodes": [
    {"id": "n1", "name": "Entry", "node_type": "waypoint", "position": {"x": 1.0, "y": 1.0, "z": 0.0}},
    {"id": "n2", "name": "Aisle1-Start", "node_type": "waypoint", "position": {"x": 5.0, "y": 1.0, "z": 0.0}},
    {"id": "n3", "name": "Aisle1-End", "node_type": "waypoint", "position": {"x": 5.0, "y": 10.0, "z": 0.0}},
    {"id": "n4", "name": "Charger-1", "node_type": "charging_station", "position": {"x": 10.0, "y": 1.0, "z": 0.0}},
    {"id": "n5", "name": "Dock-A", "node_type": "loading_dock", "position": {"x": 10.0, "y": 10.0, "z": 0.0}}
  ],
  "edges": [
    {"id": "e1", "source_node_id": "n1", "target_node_id": "n2", "distance": 4.0, "max_speed": 1.5, "direction": "bidirectional", "cost_factor": 1.0},
    {"id": "e2", "source_node_id": "n2", "target_node_id": "n3", "distance": 9.0, "max_speed": 2.0, "direction": "bidirectional", "cost_factor": 1.0},
    {"id": "e3", "source_node_id": "n2", "target_node_id": "n4", "distance": 5.0, "max_speed": 1.0, "direction": "bidirectional", "cost_factor": 1.0},
    {"id": "e4", "source_node_id": "n3", "target_node_id": "n5", "distance": 5.0, "max_speed": 1.0, "direction": "unidirectional", "cost_factor": 1.0}
  ]
}
```

**검증:** `distance` = `euclidean(source.pos, target.pos)` (오차 < 0.01m), source/target 노드 존재, `max_speed` > 0 이며 2.0m/s 이하, 노드 좌표가 포인트 클라우드 범위 내.

### 2.4 시맨틱 레이어 데이터

**용도:** C-04 (Map Core), A-02 (Traffic Management), A-06 (Map Editor)

**Protobuf 매핑:** `SemanticRegion`, `RegionType` enum (no_go_zone, speed_limit_zone, charging_area, loading_dock, waiting_area, one_way_zone, restricted_area).

**데이터셋 (5개 영역):**
```json
[
  {"id": "sr-001", "map_id": "test-map-001", "name": "Charging Zone A", "region_type": "charging_area",
   "geometry": {"vertices": [{"x":0,"y":0,"z":0},{"x":1.5,"y":0,"z":0},{"x":1.5,"y":1.5,"z":0},{"x":0,"y":1.5,"z":0},{"x":0,"y":0,"z":0}]},
   "properties": {"charger_count": 2, "charger_type": "contact"}},
  {"id": "sr-002", "map_id": "test-map-001", "name": "No-Go Zone", "region_type": "no_go_zone",
   "geometry": {"vertices": [{"x":2,"y":2,"z":0},{"x":3,"y":2,"z":0},{"x":3,"y":3,"z":0},{"x":2,"y":3,"z":0},{"x":2,"y":2,"z":0}]},
   "properties": {}},
  {"id": "sr-003", "map_id": "test-map-001", "name": "Speed Limit Corridor", "region_type": "speed_limit_zone",
   "geometry": {"vertices": [{"x":0,"y":6,"z":0},{"x":20,"y":6,"z":0},{"x":20,"y":7,"z":0},{"x":0,"y":7,"z":0},{"x":0,"y":6,"z":0}]},
   "properties": {"max_speed_mps": 0.3}},
  {"id": "sr-004", "map_id": "test-map-001", "name": "Loading Dock B", "region_type": "loading_dock",
   "geometry": {"vertices": [{"x":18,"y":13,"z":0},{"x":20,"y":13,"z":0},{"x":20,"y":15,"z":0},{"x":18,"y":15,"z":0},{"x":18,"y":13,"z":0}]},
   "properties": {"dock_type": "pallet", "capacity": 1}},
  {"id": "sr-005", "map_id": "test-map-001", "name": "One-Way East", "region_type": "one_way_zone",
   "geometry": {"vertices": [{"x":8,"y":0,"z":0},{"x":20,"y":0,"z":0},{"x":20,"y":1,"z":0},{"x":8,"y":1,"z":0},{"x":8,"y":0,"z":0}]},
   "properties": {"allowed_direction_deg": 0.0}}
]
```

**검증:** 닫힌 폴리곤 (첫/끝 꼭짓점 동일), 꼭짓점이 맵 범위 내, `speed_limit_zone`은 `max_speed_mps` 필수, `charging_area`는 `charger_count` 필수.

### 2.5 텔레메트리 데이터

**용도:** C-06 (Telemetry Pipeline), C-02 (3D Viewer Core)

| 항목 | 포맷만 준수 | 의미 필수 | 설명 |
|------|:----------:|:--------:|------|
| 단일 프레임 | ✅ | ❌ | WebSocket 파싱/렌더링 테스트 |
| 시계열 (60초) | ❌ | ✅ | 직선 이동 궤적. position이 시간에 따라 연속 변화 |
| 다중 로봇 시계열 | ❌ | ✅ | 10대 로봇, 충돌 없는 개별 경로 |

**Protobuf 매핑:** `proto/simulation/sim_service.proto`의 `TelemetryMessage`. 실제 로봇과 시뮬 로봇이 동일 포맷 (mode-agnostic).

**단일 프레임:**
```json
{
  "header": {"timestamp": "2026-03-21T09:30:00.000Z", "frame_id": "map", "sequence": 1},
  "robot_id": "test-robot-001",
  "robot_state": {
    "pose": {
      "position": {"x": 5.0, "y": 3.2, "z": 0.0},
      "orientation": {"x": 0.0, "y": 0.0, "z": 0.707, "w": 0.707}
    },
    "velocity": {"linear": {"x": 0.5, "y": 0.0, "z": 0.0}, "angular": {"x": 0.0, "y": 0.0, "z": 0.1}},
    "battery": {"percentage": 85.5, "voltage": 24.2, "charging": false},
    "odometry_distance": 127.3
  },
  "sensor_data": []
}
```

**시계열 검증:** `timestamp` 단조 증가, `position` 변화량 ≈ `velocity.linear * dt`, `battery.percentage` 단조 감소 (비충전 시), 쿼터니언 노름 = 1.0 (오차 1e-6), `odometry_distance` 단조 증가.

### 2.6 시뮬레이션 월드 데이터

**용도:** C-01 (Sim Engine Core)

| 항목 | 포맷만 준수 | 의미 필수 | 설명 |
|------|:----------:|:--------:|------|
| 빈 월드 | ✅ | ❌ | 장애물 없는 평면. 기본 이동 테스트 |
| 간단한 방 (20m x 15m) | ❌ | ✅ | 4면 벽 + 선반 2개. 충돌 테스트 |
| 창고 월드 (40m x 30m) | ❌ | ✅ | 선반 8개, 충전소, 적재독. 시나리오 테스트 |

**Sim Engine 연동:** OpenUSD가 기본 씬 포맷. 아래 JSON은 월드 구성 정의이며, `world_generator.py`가 USD로 변환한다.

**간단한 방:**
```json
{
  "world": {
    "name": "test_room", "size": [20.0, 15.0], "ground_plane": true, "gravity": [0.0, 0.0, -9.81],
    "walls": [
      {"from": [0, 0], "to": [20, 0], "height": 3.0, "thickness": 0.1},
      {"from": [20, 0], "to": [20, 15], "height": 3.0, "thickness": 0.1},
      {"from": [20, 15], "to": [0, 15], "height": 3.0, "thickness": 0.1},
      {"from": [0, 15], "to": [0, 0], "height": 3.0, "thickness": 0.1}
    ],
    "obstacles": [
      {"type": "box", "position": [5, 7], "size": [4.0, 0.2], "height": 2.5, "name": "shelf_1", "is_static": true},
      {"type": "box", "position": [10, 7], "size": [4.0, 0.2], "height": 2.5, "name": "shelf_2", "is_static": true}
    ]
  }
}
```

### 2.7 에셋 메타데이터, 미션, VDA5050

**에셋 메타데이터** (C-05): CRUD 테스트는 포맷만, URDF 변환 테스트는 의미 필수. MinIO 경로 `assets/{type}/{id}/{format}/`.

**미션 데이터** (A-01, A-02): 로드맵 그래프 위의 유효한 start/end 노드 참조 필수. 경로가 존재해야 함.

```json
{
  "id": "mission-001", "name": "Pickup from Dock-A",
  "robot_id": "test-robot-001", "map_id": "test-map-001",
  "status": "pending", "priority": 5,
  "start_node_id": "n1", "goal_node_id": "n5",
  "waypoints": ["n1", "n2", "n3", "n5"],
  "created_at": "2026-03-21T09:30:00.000Z"
}
```

**VDA5050 메시지** (A-04): Order는 VDA5050 v2.0 스펙 준수, nodes/edges가 로드맵과 일치 (의미 필수). State는 파싱 테스트용 (포맷만).

```json
{
  "headerId": 1, "timestamp": "2026-03-21T09:30:00.000Z",
  "version": "2.0.0", "manufacturer": "TestManufacturer", "serialNumber": "test-robot-001",
  "orderId": "order-001", "orderUpdateId": 0,
  "nodes": [
    {"nodeId": "n1", "sequenceId": 0, "released": true, "nodePosition": {"x": 1.0, "y": 1.0, "theta": 0.0, "mapId": "test-map-001"}, "actions": []},
    {"nodeId": "n2", "sequenceId": 2, "released": true, "nodePosition": {"x": 5.0, "y": 1.0, "theta": 0.0, "mapId": "test-map-001"}, "actions": []}
  ],
  "edges": [
    {"edgeId": "e1", "sequenceId": 1, "released": true, "startNodeId": "n1", "endNodeId": "n2", "maxSpeed": 1.5, "actions": []}
  ]
}
```

---

## 3. 데이터셋 크기 등급

| 등급 | 용도 | 로봇 수 | 포인트 수 | 그래프 노드 | 미션 수 | 텔레메트리 프레임 |
|------|------|:-------:|:---------:|:----------:|:-------:|:----------------:|
| **XS** | 단위 테스트 | 1 | 1K | 3 | 1 | 10 |
| **S** | 컴포넌트 테스트 | 3 | 10K | 10 | 5 | 600 |
| **M** | 통합 테스트 | 10 | 1M | 50 | 20 | 6,000 |
| **L** | 성능 테스트 | 50 | 10M | 200 | 100 | 30,000 |
| **XL** | 부하 테스트 | 200 | 100M | 1,000 | 1,000 | 120,000 |

**등급별 권장 사용처:**

| 테스트 유형 | 권장 등급 |
|------------|----------|
| proto 파싱/직렬화, gRPC 핸들러 단위 테스트 | XS~S |
| WebSocket 릴레이, Potree 변환 파이프라인 | S~M |
| A* 경로 탐색 (10K 노드에서 5ms 이내 목표) | M~L |
| E2E 시뮬레이션 시나리오, TimescaleDB 쿼리 성능 | M~L |
| 시스템 부하 테스트, 동시 접속 한계 | XL |

---

## 4. 데이터 생성 도구

### 4.1 디렉토리 구조

```
test-data/
├── generators/
│   ├── config.py                # 공통 설정 (시드, 크기 등급, 좌표계)
│   ├── robot_generator.py       # 로봇 모델/설정 생성
│   ├── pointcloud_generator.py  # 포인트 클라우드 생성
│   ├── roadmap_generator.py     # 로드맵 그래프 생성
│   ├── semantic_generator.py    # 시맨틱 영역 생성
│   ├── telemetry_generator.py   # 텔레메트리 시계열 생성
│   ├── world_generator.py       # 시뮬레이션 월드 생성
│   ├── mission_generator.py     # 미션 데이터 생성
│   └── vda5050_generator.py     # VDA5050 메시지 생성
├── fixtures/
│   ├── xs/                      # Extra Small 데이터셋 (git 추적)
│   ├── s/                       # Small 데이터셋 (git 추적)
│   ├── m/                       # Medium (생성 스크립트로 재생성, .gitignore)
│   └── shared/                  # 공유 파일 (URDF, glTF, USD 등)
├── validators/
│   ├── schema_validator.py      # JSON Schema 검증
│   ├── reference_validator.py   # 참조 무결성 검증
│   └── physics_validator.py     # 물리적 타당성 검증
├── generate_all.py              # 전체 데이터셋 생성 진입점
└── validate_all.py              # 전체 데이터셋 검증 스크립트
```

### 4.2 공통 설정

```python
"""test-data/generators/config.py"""
import numpy as np
from dataclasses import dataclass
from enum import Enum

SEED = 42

class DatasetSize(Enum):
    XS = "xs"; S = "s"; M = "m"; L = "l"; XL = "xl"

@dataclass
class SizeConfig:
    robot_count: int
    point_count: int
    graph_nodes: int
    mission_count: int
    telemetry_frames: int

SIZE_CONFIGS = {
    DatasetSize.XS: SizeConfig(1, 1_000, 3, 1, 10),
    DatasetSize.S:  SizeConfig(3, 10_000, 10, 5, 600),
    DatasetSize.M:  SizeConfig(10, 1_000_000, 50, 20, 6_000),
    DatasetSize.L:  SizeConfig(50, 10_000_000, 200, 100, 30_000),
    DatasetSize.XL: SizeConfig(200, 100_000_000, 1_000, 1_000, 120_000),
}

def get_rng(seed: int = SEED) -> np.random.Generator:
    return np.random.default_rng(seed)
```

### 4.3 실행 방법

```bash
# 전체 데이터셋 생성 (기본: S 등급)
python test-data/generate_all.py

# 특정 등급/유형 지정
python test-data/generate_all.py --size m --type roadmap

# 시드 변경
python test-data/generate_all.py --seed 123

# 검증
python test-data/validate_all.py --fixtures test-data/fixtures/s/
python test-data/validate_all.py --check references --fixtures test-data/fixtures/m/
```

---

## 5. 크로스 데이터 참조 무결성

### 5.1 참조 그래프

```
Robot Config
  ├── robot_id ──→ TelemetryMessage.robot_id
  ├── robot_id ──→ Mission.robot_id
  ├── robot_id ──→ VDA5050 Order.serialNumber
  └── model_id ──→ Asset Metadata.model

Map (ID)
  ├── map_id ────→ RoadmapGraph.map_id
  ├── map_id ────→ SemanticLayer.map_id
  ├── map_id ────→ PointCloudMetadata.map_id
  └── map_id ────→ Mission.map_id

RoadmapNode
  ├── node_id ───→ RoadmapEdge.source_node_id / target_node_id
  ├── node_id ───→ Mission.start_node_id / goal_node_id
  ├── node_id ───→ VDA5050 Order.nodes[].nodeId
  └── position ──→ PointCloud bounding box 범위 내

World.size ──────→ PointCloud bounding box와 일치
```

### 5.2 참조 무결성 검증 규칙

| 규칙 ID | 검증 대상 | 조건 |
|---------|----------|------|
| REF-01 | Mission → Robot | `mission.robot_id` ∈ `{robot.id}` |
| REF-02 | Mission → Map | `mission.map_id` ∈ `{map.id}` |
| REF-03 | Mission → Roadmap | `start_node_id`, `goal_node_id` ∈ `{node.id}` |
| REF-04 | Mission → Path | 연속 waypoints 사이에 edge 존재 |
| REF-05 | Edge → Node | `source_node_id`, `target_node_id` ∈ `{node.id}` |
| REF-06 | Edge distance | `|distance - euclidean(src, tgt)| < 0.01` |
| REF-07 | Node → PointCloud | 노드 `position`이 포인트 클라우드 바운딩 박스 내 |
| REF-08 | Semantic → Map | 모든 꼭짓점이 맵 범위 내 |
| REF-09 | VDA5050 → Roadmap | Order `nodeId`가 로드맵 노드 ID와 일치 |
| REF-10 | Telemetry → Robot | `telemetry.robot_id` ∈ `{robot.id}` |

---

## 6. 실 데이터 관리 전략

### 6.1 디렉토리 분리

- `test-data/fixtures/xs/`, `s/`: git에 포함 (소규모)
- `test-data/fixtures/m/` 이상: `.gitignore` (생성 스크립트로 재생성)
- `data/`: 실 데이터 (.gitignore, 별도 스토리지)

### 6.2 데이터 전환

환경변수 `DATA_SOURCE=test|real`로 전환. 실 데이터와 더미 데이터는 동일한 스키마/포맷을 사용한다.

### 6.3 실 데이터 온보딩 체크리스트

- [ ] 포인트 클라우드 좌표계 확인 (ENU 변환 필요 여부)
- [ ] URDF/SDF 모델 단위 확인 (미터 vs 밀리미터)
- [ ] 로드맵 노드 좌표가 포인트 클라우드 범위 내에 있는지 확인
- [ ] 에셋 파일 크기 500MB 이하, 포인트 클라우드 10GB 이하
- [ ] 검증 API 호출: `POST /api/validate/{type}`
- [ ] 검증 스크립트: `python test-data/validate_all.py --fixtures data/`

### 6.4 스키마 호환성 유지

1. proto 파일 수정 → 테스트 데이터 생성기 업데이트 → 기존 fixture 재생성
2. 새 필드 추가 시 마이그레이션 스크립트로 기존 fixture에 기본값 삽입
3. CI에서 PR마다 `validate_all.py` 실행하여 fixture ↔ 스키마 호환성 확인

---

## 7. 데이터 유형 ↔ 기능 매핑 매트릭스

| 데이터 유형 | C-01 | C-02 | C-03 | C-04 | C-05 | C-06 | A-01 | A-02 | A-03 | A-04 |
|------------|:----:|:----:|:----:|:----:|:----:|:----:|:----:|:----:|:----:|:----:|
| 로봇 모델 (URDF/USD) | ✅ | | | | ✅ | | | | | |
| 로봇 모델 (glTF) | | ✅ | | | ✅ | | | | | |
| 로봇 설정 JSON | ✅ | | ✅ | | | | | | | |
| 포인트 클라우드 | | ✅ | | ✅ | | | | | | |
| 로드맵 그래프 | | | | ✅ | | | ✅ | ✅ | | |
| 시맨틱 레이어 | | | | ✅ | | | | ✅ | | |
| 장애물 | ✅ | | | ✅ | | | | ✅ | | |
| 텔레메트리 | | ✅ | | | | ✅ | | | | |
| 시뮬레이션 월드 | ✅ | | | | | | | | | |
| 에셋 메타데이터 | | | | | ✅ | | | | | |
| 미션 데이터 | | | ✅ | | | | ✅ | ✅ | | |
| VDA5050 메시지 | | | | | | | | | | ✅ |
