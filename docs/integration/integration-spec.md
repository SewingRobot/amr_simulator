# 팀 간 통합 명세서

> **버전**: 1.0.0
> **최종 수정일**: 2026-03-21
> **상태**: Draft
> **대상**: 전 팀 (Frontend, Backend, Simulation Engine, Asset Manager, Map Manager)

---

## 목차

1. [개요](#1-개요)
2. [공유 프로토콜 정의 (Proto Schema)](#2-공유-프로토콜-정의-proto-schema)
3. [팀 간 의존성 매트릭스](#3-팀-간-의존성-매트릭스)
4. [개발 타이밍 & 동기화 포인트](#4-개발-타이밍--동기화-포인트)
5. [통합 테스트 방법](#5-통합-테스트-방법)
6. [문서 스키마 & 커뮤니케이션](#6-문서-스키마--커뮤니케이션)
7. [좌표계 및 단위 규약](#7-좌표계-및-단위-규약)
8. [데이터 포맷 규약](#8-데이터-포맷-규약)

---

## 1. 개요

### 1.1 문서 목적

이 문서는 AMR(Autonomous Mobile Robot) 통합 프레임워크 프로젝트에서 5개 팀이 AI agent swarm 기반으로 병렬 개발을 수행할 때 필요한 **팀 간 소통 프로토콜**, **데이터 스키마**, **통합 타이밍**, **테스트 방법**을 정의하는 마스터 계약(Master Contract) 문서이다.

각 팀은 독립적으로 개발하되, 이 문서에 정의된 계약(contract)을 반드시 준수해야 한다. 이 문서에 정의되지 않은 인터페이스를 사용하거나, 정의된 인터페이스를 변경하려면 반드시 RFC 프로세스를 거쳐야 한다.

### 1.2 팀 구성

| 팀 번호 | 팀 이름 | 기술 스택 | 주요 역할 |
|---------|---------|----------|----------|
| Team 1 | Frontend | React / TypeScript / Three.js | 3D 시각화, 사용자 인터페이스, 대시보드 |
| Team 2 | Backend | Rust / Axum / SQLx | REST API, WebSocket 릴레이, 미션 관리, 인증/인가, MQTT 브릿지 |
| Team 3 | Simulation Engine | C++ / gRPC / Eigen / Bullet Physics | 물리 시뮬레이션, 로봇 동역학, 센서 시뮬레이션 |
| Team 4 | Asset Manager | Rust / Tonic / SQLx | URDF/SDF 파싱, 3D 모델 변환(glTF), 에셋 저장소 관리 |
| Team 5 | Map Manager | Rust / Tonic / SQLx | 포인트 클라우드 처리, 맵 타일링, 로드맵 그래프 관리, 의미론적 영역 관리 |

### 1.3 통신 프로토콜 개요

| 통신 경로 | 프로토콜 | 데이터 포맷 | 용도 |
|----------|---------|-----------|-----|
| Frontend ↔ Backend | REST (HTTP/1.1, HTTPS) | JSON | CRUD 연산, 파일 업로드/다운로드 |
| Frontend ↔ Backend | WebSocket (WS/WSS) | JSON | 실시간 텔레메트리, 알림, 원격 제어 |
| Backend ↔ Sim Engine | gRPC (HTTP/2) | Protobuf | 시뮬레이션 제어, 텔레메트리 스트리밍 |
| Backend ↔ Asset Manager | gRPC (HTTP/2) | Protobuf | 에셋 조회/변환 요청 |
| Backend ↔ Map Manager | gRPC (HTTP/2) | Protobuf | 맵 관리, 로드맵 CRUD, 타일 조회 |
| Backend ↔ Real Robot | MQTT (v5) | JSON (VDA5050) | 실제 로봇 제어 및 상태 수신 |
| Sim Engine → Asset Manager | gRPC (HTTP/2) | Protobuf | 로봇 모델(URDF) 로딩 |
| Sim Engine → Map Manager | gRPC (HTTP/2) | Protobuf | 맵 데이터 로딩 |

### 1.4 용어 정의

| 용어 | 정의 |
|-----|------|
| AMR | Autonomous Mobile Robot — 자율이동 로봇 |
| URDF | Unified Robot Description Format — 로봇 기술 형식 |
| SDF | Simulation Description Format — 시뮬레이션 기술 형식 |
| glTF | GL Transmission Format — 3D 모델 전송 형식 |
| VDA5050 | VDA(독일자동차산업협회) 표준 AGV 통신 프로토콜 v2.0 |
| Roadmap | 로봇이 이동할 수 있는 경로를 나타내는 그래프 (노드 + 엣지) |
| Telemetry | 로봇의 위치, 속도, 배터리 등 실시간 상태 정보 |
| Potree | 대용량 포인트 클라우드 렌더링을 위한 옥트리 기반 타일링 형식 |
| ENU | East-North-Up 좌표계 |

---

## 2. 공유 프로토콜 정의 (Proto Schema)

### 2.1 Protobuf 디렉토리 구조

```
proto/
├── common/
│   ├── geometry.proto          # Pose2D, Pose3D, Vector3, Quaternion, BoundingBox
│   ├── telemetry.proto         # RobotState, TelemetryMessage, SensorReading
│   ├── identifiers.proto       # RobotId, MapId, AssetId, MissionId (all UUID strings)
│   └── errors.proto            # ErrorCode enum, ErrorDetail message
├── simulation/
│   ├── sim_service.proto       # SimulationService definition
│   └── sim_types.proto         # SimConfig, ScenarioDefinition, SimEvent
├── asset/
│   ├── asset_service.proto     # AssetService definition
│   └── asset_types.proto       # AssetDescriptor, AssetType enum, ConversionJob
├── map/
│   ├── map_service.proto       # MapService definition
│   └── map_types.proto         # MapDescriptor, RoadmapNode, RoadmapEdge, SemanticRegion, TileData
├── mission/
│   └── mission_types.proto     # Mission, MissionStep, MissionStatus enum
└── buf.yaml                    # buf.build configuration for linting and breaking change detection
```

모든 `.proto` 파일은 이 문서의 2.2절에 정의된 내용을 **단일 진실 원천(single source of truth)**으로 사용한다. 코드 생성은 `buf generate` 명령어를 사용하며, 각 팀은 자신의 빌드 시스템에 buf 코드 생성을 통합해야 한다.

### 2.2 공유 메시지 정의

아래는 모든 protobuf 파일의 **완전한 정의(complete definition)**이다. 각 팀은 이 정의를 기반으로 코드를 생성한다.

#### 2.2.1 buf.yaml

```yaml
# proto/buf.yaml
version: v1
name: buf.build/amr/proto
breaking:
  use:
    - FILE
lint:
  use:
    - DEFAULT
  enum_zero_value_suffix: _UNSPECIFIED
  rpc_allow_same_request_response: false
  rpc_allow_google_protobuf_empty_requests: true
  rpc_allow_google_protobuf_empty_responses: true
```

#### 2.2.2 proto/common/geometry.proto

```protobuf
// proto/common/geometry.proto
//
// 공통 기하학 타입 정의.
// 모든 좌표는 ENU(East-North-Up) 좌표계, 미터 단위, 라디안 단위를 사용한다.
// 쿼터니언은 Hamilton 규약 (x, y, z, w)을 따른다.

syntax = "proto3";
package amr.v1.common;

option java_multiple_files = true;
option java_package = "com.amr.v1.common";

// 3차원 벡터. 위치, 속도, 가속도 등 다양한 용도로 사용.
message Vector3 {
  double x = 1;  // East 방향 (미터 또는 미터/초)
  double y = 2;  // North 방향
  double z = 3;  // Up 방향
}

// 쿼터니언 회전 표현. Hamilton 규약 (x, y, z, w).
// 단위 쿼터니언이어야 한다: x² + y² + z² + w² = 1.
message Quaternion {
  double x = 1;
  double y = 2;
  double z = 3;
  double w = 4;  // 스칼라 부분
}

// 3D 자세 (위치 + 방향).
message Pose3D {
  Vector3 position = 1;       // 위치 (미터)
  Quaternion orientation = 2; // 방향 (단위 쿼터니언)
}

// 2D 자세 (평면 위치 + 방향각).
// 바닥 평면(XY 평면)에서의 로봇 자세를 간단히 표현할 때 사용.
message Pose2D {
  double x = 1;      // East 방향 위치 (미터)
  double y = 2;      // North 방향 위치 (미터)
  double theta = 3;  // 방향각 (라디안, 반시계 방향이 양수, East축 기준)
}

// 선형 속도와 각속도를 함께 표현하는 트위스트.
message Twist {
  Vector3 linear = 1;   // 선형 속도 (m/s)
  Vector3 angular = 2;  // 각속도 (rad/s)
}

// 축 정렬 바운딩 박스 (AABB).
message BoundingBox {
  Vector3 min = 1;  // 최소 꼭짓점 (x_min, y_min, z_min)
  Vector3 max = 2;  // 최대 꼭짓점 (x_max, y_max, z_max)
}

// 강체 변환 (Translation + Rotation).
message Transform {
  Vector3 translation = 1;    // 이동 (미터)
  Quaternion rotation = 2;    // 회전 (단위 쿼터니언)
}

// 2D 다각형. 의미론적 영역(Semantic Region) 경계 등에 사용.
message Polygon2D {
  repeated Point2D vertices = 1;  // 꼭짓점 리스트 (최소 3개, 시계 방향)
}

// 2D 점.
message Point2D {
  double x = 1;  // East (미터)
  double y = 2;  // North (미터)
}

// 색상 (시각화 용도).
message Color {
  float r = 1;  // Red [0.0, 1.0]
  float g = 2;  // Green [0.0, 1.0]
  float b = 3;  // Blue [0.0, 1.0]
  float a = 4;  // Alpha [0.0, 1.0]
}
```

#### 2.2.3 proto/common/identifiers.proto

```protobuf
// proto/common/identifiers.proto
//
// UUID 기반 식별자 래퍼 타입.
// 모든 ID는 UUID v4 형식의 소문자 hex 문자열이다.
// 예: "550e8400-e29b-41d4-a716-446655440000"

syntax = "proto3";
package amr.v1.common;

option java_multiple_files = true;
option java_package = "com.amr.v1.common";

// 로봇 식별자.
message RobotId {
  string value = 1;  // UUID v4 문자열
}

// 맵 식별자.
message MapId {
  string value = 1;  // UUID v4 문자열
}

// 에셋 식별자.
message AssetId {
  string value = 1;  // UUID v4 문자열
}

// 미션 식별자.
message MissionId {
  string value = 1;  // UUID v4 문자열
}

// 시나리오 식별자.
message ScenarioId {
  string value = 1;  // UUID v4 문자열
}

// 시뮬레이션 세션 식별자.
message SimSessionId {
  string value = 1;  // UUID v4 문자열
}

// 로드맵 식별자.
message RoadmapId {
  string value = 1;  // UUID v4 문자열
}

// 로드맵 노드 식별자.
message NodeId {
  string value = 1;  // UUID v4 문자열
}

// 로드맵 엣지 식별자.
message EdgeId {
  string value = 1;  // UUID v4 문자열
}

// 의미론적 영역 식별자.
message RegionId {
  string value = 1;  // UUID v4 문자열
}

// 사용자 식별자.
message UserId {
  string value = 1;  // UUID v4 문자열
}

// 플러그인 식별자.
message PluginId {
  string value = 1;  // UUID v4 문자열
}

// 변환 작업 식별자.
message ConversionJobId {
  string value = 1;  // UUID v4 문자열
}
```

#### 2.2.4 proto/common/errors.proto

```protobuf
// proto/common/errors.proto
//
// 공유 에러 코드 및 에러 상세 메시지.
// 에러 코드 범위는 팀별로 할당된다 (6.3절 참조).

syntax = "proto3";
package amr.v1.common;

option java_multiple_files = true;
option java_package = "com.amr.v1.common";

// 에러 코드 열거형.
// 범위: 1000-1999 Auth, 2000-2999 Robot/Mission, 3000-3999 Simulation,
//       4000-4999 Asset, 5000-5999 Map, 6000-6999 Plugin
enum ErrorCode {
  ERROR_CODE_UNSPECIFIED = 0;

  // 공통 에러 (0-999)
  ERROR_CODE_INTERNAL = 1;
  ERROR_CODE_INVALID_ARGUMENT = 2;
  ERROR_CODE_NOT_FOUND = 3;
  ERROR_CODE_ALREADY_EXISTS = 4;
  ERROR_CODE_PERMISSION_DENIED = 5;
  ERROR_CODE_RESOURCE_EXHAUSTED = 6;
  ERROR_CODE_UNAVAILABLE = 7;
  ERROR_CODE_DEADLINE_EXCEEDED = 8;
  ERROR_CODE_CANCELLED = 9;
  ERROR_CODE_DATA_LOSS = 10;
  ERROR_CODE_UNIMPLEMENTED = 11;

  // 인증/인가 에러 (1000-1999)
  ERROR_CODE_AUTH_TOKEN_EXPIRED = 1000;
  ERROR_CODE_AUTH_TOKEN_INVALID = 1001;
  ERROR_CODE_AUTH_INSUFFICIENT_SCOPE = 1002;
  ERROR_CODE_AUTH_USER_DISABLED = 1003;
  ERROR_CODE_AUTH_RATE_LIMITED = 1004;

  // 로봇/미션 에러 (2000-2999)
  ERROR_CODE_ROBOT_NOT_FOUND = 2000;
  ERROR_CODE_ROBOT_OFFLINE = 2001;
  ERROR_CODE_ROBOT_BUSY = 2002;
  ERROR_CODE_MISSION_NOT_FOUND = 2003;
  ERROR_CODE_MISSION_INVALID_STATE = 2004;
  ERROR_CODE_MISSION_PATH_NOT_FOUND = 2005;
  ERROR_CODE_MISSION_ZONE_LOCKED = 2006;
  ERROR_CODE_MISSION_DEADLOCK = 2007;

  // 시뮬레이션 에러 (3000-3999)
  ERROR_CODE_SIM_SESSION_NOT_FOUND = 3000;
  ERROR_CODE_SIM_ALREADY_RUNNING = 3001;
  ERROR_CODE_SIM_PHYSICS_ERROR = 3002;
  ERROR_CODE_SIM_SPAWN_FAILED = 3003;
  ERROR_CODE_SIM_ASSET_LOAD_FAILED = 3004;
  ERROR_CODE_SIM_MAP_LOAD_FAILED = 3005;
  ERROR_CODE_SIM_MAX_ROBOTS_EXCEEDED = 3006;
  ERROR_CODE_SIM_INVALID_CONFIG = 3007;

  // 에셋 에러 (4000-4999)
  ERROR_CODE_ASSET_NOT_FOUND = 4000;
  ERROR_CODE_ASSET_FORMAT_UNSUPPORTED = 4001;
  ERROR_CODE_ASSET_CONVERSION_FAILED = 4002;
  ERROR_CODE_ASSET_STORAGE_FULL = 4003;
  ERROR_CODE_ASSET_CORRUPTED = 4004;
  ERROR_CODE_ASSET_TOO_LARGE = 4005;
  ERROR_CODE_ASSET_VALIDATION_FAILED = 4006;

  // 맵 에러 (5000-5999)
  ERROR_CODE_MAP_NOT_FOUND = 5000;
  ERROR_CODE_MAP_PROCESSING_FAILED = 5001;
  ERROR_CODE_MAP_TILE_NOT_FOUND = 5002;
  ERROR_CODE_MAP_ROADMAP_INVALID = 5003;
  ERROR_CODE_MAP_REGION_OVERLAP = 5004;
  ERROR_CODE_MAP_TOO_LARGE = 5005;
  ERROR_CODE_MAP_FORMAT_UNSUPPORTED = 5006;
  ERROR_CODE_MAP_NODE_NOT_FOUND = 5007;
  ERROR_CODE_MAP_EDGE_NOT_FOUND = 5008;

  // 플러그인 에러 (6000-6999)
  ERROR_CODE_PLUGIN_NOT_FOUND = 6000;
  ERROR_CODE_PLUGIN_LOAD_FAILED = 6001;
  ERROR_CODE_PLUGIN_EXECUTION_FAILED = 6002;
  ERROR_CODE_PLUGIN_TIMEOUT = 6003;
  ERROR_CODE_PLUGIN_MEMORY_EXCEEDED = 6004;
  ERROR_CODE_PLUGIN_API_DENIED = 6005;
}

// 에러 상세 메시지.
message ErrorDetail {
  ErrorCode code = 1;            // 에러 코드
  string message = 2;            // 사람이 읽을 수 있는 에러 메시지
  string target = 3;             // 에러가 발생한 대상 (필드명, 리소스 ID 등)
  map<string, string> metadata = 4;  // 추가 메타데이터 (디버그 정보 등)
}

// 에러 응답 (다수 에러를 반환할 때).
message ErrorResponse {
  repeated ErrorDetail errors = 1;
  string trace_id = 2;  // OpenTelemetry trace ID
}
```

#### 2.2.5 proto/common/telemetry.proto

```protobuf
// proto/common/telemetry.proto
//
// 로봇 텔레메트리 및 센서 데이터 정의.
// Sim Engine이 텔레메트리를 생성하고, Backend가 중계하여 Frontend에 전달한다.

syntax = "proto3";
package amr.v1.common;

option java_multiple_files = true;
option java_package = "com.amr.v1.common";

import "common/geometry.proto";
import "common/identifiers.proto";

// 로봇 동작 모드.
enum OperationMode {
  OPERATION_MODE_UNSPECIFIED = 0;
  OPERATION_MODE_AUTOMATIC = 1;    // 자율 주행 모드
  OPERATION_MODE_SEMI_AUTOMATIC = 2; // 반자동 모드
  OPERATION_MODE_MANUAL = 3;       // 수동(원격) 조작 모드
  OPERATION_MODE_SERVICE = 4;      // 서비스/정비 모드
  OPERATION_MODE_TEACH_IN = 5;     // 경로 학습 모드
}

// 로봇 연결 상태.
enum ConnectionStatus {
  CONNECTION_STATUS_UNSPECIFIED = 0;
  CONNECTION_STATUS_ONLINE = 1;
  CONNECTION_STATUS_OFFLINE = 2;
  CONNECTION_STATUS_CONNECTION_BROKEN = 3;
}

// 배터리 상태.
message BatteryStatus {
  double charge_percentage = 1;    // 충전 비율 (0.0 ~ 100.0)
  double voltage = 2;              // 전압 (V)
  double current = 3;              // 전류 (A)
  double temperature = 4;          // 온도 (°C)
  bool is_charging = 5;            // 충전 중 여부
  uint32 estimated_minutes_remaining = 6; // 예상 잔여 시간 (분)
}

// 안전 상태.
message SafetyState {
  EStopStatus e_stop = 1;
  bool field_violation = 2;           // 안전 영역 침범 여부
  repeated string active_safety_zones = 3; // 현재 활성화된 안전 영역 ID
}

// 비상 정지 상태.
enum EStopStatus {
  E_STOP_STATUS_UNSPECIFIED = 0;
  E_STOP_STATUS_AUTO_ACK = 1;        // 자동 해제 가능
  E_STOP_STATUS_MANUAL = 2;          // 수동 비상 정지
  E_STOP_STATUS_REMOTE = 3;          // 원격 비상 정지
  E_STOP_STATUS_NONE = 4;            // 비상 정지 아님
}

// 개별 센서 읽기 값.
message SensorReading {
  string sensor_id = 1;              // 센서 식별자 (예: "lidar_front", "imu_0")
  string sensor_type = 2;            // 센서 유형 (예: "lidar", "imu", "camera", "ultrasonic")
  int64 timestamp_ms = 3;            // 측정 시각 (Unix milliseconds)
  oneof data {
    LidarScan lidar_scan = 10;
    ImuReading imu_reading = 11;
    UltrasonicReading ultrasonic_reading = 12;
    EncoderReading encoder_reading = 13;
  }
}

// 2D LiDAR 스캔 데이터.
message LidarScan {
  float angle_min = 1;               // 최소 각도 (라디안)
  float angle_max = 2;               // 최대 각도 (라디안)
  float angle_increment = 3;         // 각도 증분 (라디안)
  float range_min = 4;               // 최소 거리 (미터)
  float range_max = 5;               // 최대 거리 (미터)
  repeated float ranges = 6;         // 거리 배열 (미터)
  repeated float intensities = 7;    // 강도 배열
}

// IMU 센서 데이터.
message ImuReading {
  Vector3 linear_acceleration = 1;   // 선형 가속도 (m/s²)
  Vector3 angular_velocity = 2;      // 각속도 (rad/s)
  Quaternion orientation = 3;        // 방향 (쿼터니언)
}

// 초음파 센서 데이터.
message UltrasonicReading {
  float range = 1;                   // 거리 (미터)
  float field_of_view = 2;          // 시야각 (라디안)
}

// 엔코더 데이터.
message EncoderReading {
  double position = 1;               // 위치 (라디안 또는 미터)
  double velocity = 2;               // 속도 (rad/s 또는 m/s)
}

// 로봇 상태 메시지 (전체 상태 스냅샷).
message RobotState {
  RobotId robot_id = 1;
  int64 timestamp_ms = 2;            // Unix milliseconds
  uint64 sequence_number = 3;        // 시퀀스 번호 (순서 보장 및 중복 감지)

  // 위치 및 동작
  Pose3D pose = 4;                   // 현재 3D 자세
  Twist velocity = 5;               // 현재 속도
  Twist acceleration = 6;           // 현재 가속도

  // 상태 정보
  OperationMode operation_mode = 7;
  ConnectionStatus connection_status = 8;
  BatteryStatus battery = 9;
  SafetyState safety = 10;

  // 미션 관련
  MissionId current_mission_id = 11; // 현재 수행 중인 미션 (없으면 비어있음)
  string current_node_id = 12;       // 현재 위치한 노드 (VDA5050 호환)
  bool driving = 13;                 // 주행 중 여부

  // 에러
  repeated ErrorInfo errors = 14;
  repeated ErrorInfo warnings = 15;
}

// 에러/경고 정보 (VDA5050 호환).
message ErrorInfo {
  string error_type = 1;             // 에러 유형
  repeated ErrorReference references = 2; // 에러 참조
  string description = 3;            // 에러 설명
  ErrorLevel level = 4;
}

enum ErrorLevel {
  ERROR_LEVEL_UNSPECIFIED = 0;
  ERROR_LEVEL_WARNING = 1;
  ERROR_LEVEL_FATAL = 2;
}

message ErrorReference {
  string reference_key = 1;          // 참조 키 (예: "node_id", "edge_id")
  string reference_value = 2;        // 참조 값
}

// 텔레메트리 메시지 (스트리밍용, 경량화).
// RobotState의 자주 변하는 필드만 포함.
message TelemetryMessage {
  RobotId robot_id = 1;
  int64 timestamp_ms = 2;
  uint64 sequence_number = 3;

  Pose3D pose = 4;
  Twist velocity = 5;

  double battery_percentage = 6;
  bool driving = 7;

  repeated ErrorInfo errors = 8;
}

// 텔레메트리 구독 요청.
message TelemetrySubscription {
  repeated RobotId robot_ids = 1;    // 구독할 로봇 ID (빈 배열 = 모든 로봇)
  uint32 rate_hz = 2;                // 원하는 수신 빈도 (Hz, 기본값 10)
  repeated string fields = 3;        // 필터링할 필드 (빈 배열 = 전체 필드)
}
```

#### 2.2.6 proto/simulation/sim_types.proto

```protobuf
// proto/simulation/sim_types.proto
//
// 시뮬레이션 엔진 관련 타입 정의.

syntax = "proto3";
package amr.v1.simulation;

option java_multiple_files = true;
option java_package = "com.amr.v1.simulation";

import "common/geometry.proto";
import "common/identifiers.proto";
import "common/telemetry.proto";

// 시뮬레이션 설정.
message SimConfig {
  double time_step = 1;               // 물리 시뮬레이션 시간 단계 (초, 기본값 0.001)
  double real_time_factor = 2;        // 실시간 대비 속도 배율 (1.0 = 실시간, 2.0 = 2배속)
  amr.v1.common.Vector3 gravity = 3;  // 중력 벡터 (기본값: {0, 0, -9.81})
  PhysicsEngine physics_engine = 4;
  uint32 max_robots = 5;              // 최대 로봇 수 (기본값 100)
  bool enable_collision = 6;          // 충돌 감지 활성화 (기본값 true)
  bool enable_sensors = 7;            // 센서 시뮬레이션 활성화 (기본값 true)
  uint32 telemetry_rate_hz = 8;       // 텔레메트리 전송 빈도 (Hz, 기본값 10)
  RenderConfig render_config = 9;     // 렌더링 설정 (선택)
}

enum PhysicsEngine {
  PHYSICS_ENGINE_UNSPECIFIED = 0;
  PHYSICS_ENGINE_BULLET = 1;          // Bullet Physics
  PHYSICS_ENGINE_ODE = 2;             // Open Dynamics Engine
  PHYSICS_ENGINE_SIMPLE_2D = 3;       // 간단한 2D 물리 (성능 우선)
}

// 렌더링 설정 (서버 사이드 렌더링 시 사용).
message RenderConfig {
  uint32 width = 1;                    // 렌더 해상도 너비
  uint32 height = 2;                   // 렌더 해상도 높이
  bool headless = 3;                   // 헤드리스 모드 (기본값 true)
}

// 시뮬레이션 세션 상태.
enum SimSessionState {
  SIM_SESSION_STATE_UNSPECIFIED = 0;
  SIM_SESSION_STATE_INITIALIZING = 1;
  SIM_SESSION_STATE_READY = 2;
  SIM_SESSION_STATE_RUNNING = 3;
  SIM_SESSION_STATE_PAUSED = 4;
  SIM_SESSION_STATE_STOPPED = 5;
  SIM_SESSION_STATE_ERROR = 6;
}

// 시뮬레이션 세션 정보.
message SimSession {
  amr.v1.common.SimSessionId id = 1;
  SimSessionState state = 2;
  SimConfig config = 3;
  int64 created_at_ms = 4;           // 생성 시각 (Unix milliseconds)
  double sim_time = 5;               // 현재 시뮬레이션 시간 (초)
  double wall_time = 6;              // 경과된 실제 시간 (초)
  uint32 robot_count = 7;            // 현재 스폰된 로봇 수
  amr.v1.common.MapId loaded_map_id = 8; // 로드된 맵 ID
}

// 시나리오 정의 (사전 정의된 시뮬레이션 시나리오).
message ScenarioDefinition {
  amr.v1.common.ScenarioId id = 1;
  string name = 2;
  string description = 3;
  SimConfig config = 4;
  amr.v1.common.MapId map_id = 5;
  repeated SpawnRobotRequest initial_robots = 6;  // 초기 로봇 배치
  repeated ScenarioEvent scripted_events = 7;     // 스크립트 이벤트
}

// 스크립트 이벤트 (시나리오 재생 시 특정 시점에 발생).
message ScenarioEvent {
  double trigger_time = 1;            // 발동 시각 (시뮬레이션 시간, 초)
  oneof event {
    SpawnRobotRequest spawn_robot = 10;
    RemoveRobotRequest remove_robot = 11;
    SetRobotPoseRequest set_pose = 12;
    InjectFaultRequest inject_fault = 13;
  }
}

// 로봇 스폰 요청.
message SpawnRobotRequest {
  amr.v1.common.RobotId robot_id = 1;  // 지정하지 않으면 자동 생성
  amr.v1.common.AssetId asset_id = 2;  // 로봇 에셋(URDF) ID
  amr.v1.common.Pose3D initial_pose = 3;
  string robot_name = 4;               // 표시 이름 (선택)
  map<string, string> parameters = 5;  // 추가 파라미터
}

message SpawnRobotResponse {
  amr.v1.common.RobotId robot_id = 1;
  bool success = 2;
  string error_message = 3;
}

// 로봇 제거 요청.
message RemoveRobotRequest {
  amr.v1.common.RobotId robot_id = 1;
}

message RemoveRobotResponse {
  bool success = 1;
  string error_message = 2;
}

// 로봇 자세 설정 (텔레포트).
message SetRobotPoseRequest {
  amr.v1.common.RobotId robot_id = 1;
  amr.v1.common.Pose3D pose = 2;
}

message SetRobotPoseResponse {
  bool success = 1;
}

// 장애 주입 요청 (테스트용).
message InjectFaultRequest {
  amr.v1.common.RobotId robot_id = 1;
  FaultType fault_type = 2;
  double duration_seconds = 3;        // 0이면 영구적
  map<string, string> parameters = 4;
}

enum FaultType {
  FAULT_TYPE_UNSPECIFIED = 0;
  FAULT_TYPE_MOTOR_FAILURE = 1;
  FAULT_TYPE_SENSOR_NOISE = 2;
  FAULT_TYPE_BATTERY_DRAIN = 3;
  FAULT_TYPE_COMMUNICATION_LOSS = 4;
  FAULT_TYPE_WHEEL_SLIP = 5;
  FAULT_TYPE_OBSTACLE_SPAWN = 6;
}

message InjectFaultResponse {
  bool success = 1;
  string fault_id = 2;
}

// 속도 명령 (로봇 제어).
message VelocityCommand {
  amr.v1.common.RobotId robot_id = 1;
  amr.v1.common.Twist velocity = 2;
}

// 내비게이션 목표 (미션 기반 이동).
message NavigationGoal {
  amr.v1.common.RobotId robot_id = 1;
  amr.v1.common.Pose2D target_pose = 2;
  repeated amr.v1.common.Pose2D waypoints = 3;  // 경유지 (선택)
  double max_speed = 4;                // 최대 속도 (m/s)
  double goal_tolerance = 5;           // 도착 허용 오차 (미터)
}

message NavigationResult {
  amr.v1.common.RobotId robot_id = 1;
  NavigationStatus status = 2;
  amr.v1.common.Pose3D final_pose = 3;
  double distance_traveled = 4;        // 이동 거리 (미터)
  double time_elapsed = 5;             // 소요 시간 (초)
}

enum NavigationStatus {
  NAVIGATION_STATUS_UNSPECIFIED = 0;
  NAVIGATION_STATUS_SUCCEEDED = 1;
  NAVIGATION_STATUS_FAILED = 2;
  NAVIGATION_STATUS_CANCELLED = 3;
  NAVIGATION_STATUS_IN_PROGRESS = 4;
}

// 시뮬레이션 이벤트 (관찰자에게 스트리밍되는 이벤트).
message SimEvent {
  int64 timestamp_ms = 1;
  double sim_time = 2;
  oneof event {
    RobotSpawned robot_spawned = 10;
    RobotRemoved robot_removed = 11;
    CollisionEvent collision = 12;
    FaultEvent fault = 13;
    SimStateChanged state_changed = 14;
    NavigationResult navigation_completed = 15;
  }
}

message RobotSpawned {
  amr.v1.common.RobotId robot_id = 1;
  amr.v1.common.AssetId asset_id = 2;
  amr.v1.common.Pose3D pose = 3;
}

message RobotRemoved {
  amr.v1.common.RobotId robot_id = 1;
  string reason = 2;
}

message CollisionEvent {
  amr.v1.common.RobotId robot_a = 1;
  string object_b = 2;               // robot ID 또는 obstacle 이름
  amr.v1.common.Vector3 contact_point = 3;
  double impact_force = 4;           // 충돌 힘 (N)
}

message FaultEvent {
  amr.v1.common.RobotId robot_id = 1;
  FaultType fault_type = 2;
  string fault_id = 3;
  bool active = 4;                    // true=발생, false=해제
}

message SimStateChanged {
  SimSessionState previous_state = 1;
  SimSessionState new_state = 2;
  string reason = 3;
}

// 맵 로드 요청/응답.
message LoadMapRequest {
  amr.v1.common.MapId map_id = 1;
}

message LoadMapResponse {
  bool success = 1;
  string error_message = 2;
  amr.v1.common.BoundingBox world_bounds = 3;
}
```

#### 2.2.7 proto/simulation/sim_service.proto

```protobuf
// proto/simulation/sim_service.proto
//
// 시뮬레이션 엔진 gRPC 서비스 정의.
// Backend(Team 2)가 이 서비스의 클라이언트이며, Sim Engine(Team 3)이 서버이다.

syntax = "proto3";
package amr.v1.simulation;

option java_multiple_files = true;
option java_package = "com.amr.v1.simulation";

import "common/identifiers.proto";
import "common/telemetry.proto";
import "simulation/sim_types.proto";

service SimulationService {
  // === 세션 관리 ===

  // 새 시뮬레이션 세션을 생성한다.
  rpc CreateSession(CreateSessionRequest) returns (CreateSessionResponse);

  // 기존 세션 정보를 조회한다.
  rpc GetSession(GetSessionRequest) returns (SimSession);

  // 활성 세션 목록을 조회한다.
  rpc ListSessions(ListSessionsRequest) returns (ListSessionsResponse);

  // 세션을 삭제(종료)한다.
  rpc DeleteSession(DeleteSessionRequest) returns (DeleteSessionResponse);

  // === 시뮬레이션 제어 ===

  // 시뮬레이션을 시작한다.
  rpc Start(StartRequest) returns (StartResponse);

  // 시뮬레이션을 일시 정지한다.
  rpc Pause(PauseRequest) returns (PauseResponse);

  // 시뮬레이션을 재개한다.
  rpc Resume(ResumeRequest) returns (ResumeResponse);

  // 시뮬레이션을 정지한다.
  rpc Stop(StopRequest) returns (StopResponse);

  // 시뮬레이션을 리셋한다 (초기 상태로 되돌림).
  rpc Reset(ResetRequest) returns (ResetResponse);

  // 지정된 시간만큼 단일 스텝 진행 (디버깅용).
  rpc Step(StepRequest) returns (StepResponse);

  // 실시간 배율을 변경한다.
  rpc SetTimeScale(SetTimeScaleRequest) returns (SetTimeScaleResponse);

  // === 맵 관리 ===

  // 시뮬레이션 월드에 맵을 로드한다.
  rpc LoadMap(LoadMapRequest) returns (LoadMapResponse);

  // === 로봇 관리 ===

  // 로봇을 스폰한다.
  rpc SpawnRobot(SpawnRobotRequest) returns (SpawnRobotResponse);

  // 로봇을 제거한다.
  rpc RemoveRobot(RemoveRobotRequest) returns (RemoveRobotResponse);

  // 로봇 자세를 설정한다 (텔레포트).
  rpc SetRobotPose(SetRobotPoseRequest) returns (SetRobotPoseResponse);

  // 로봇에 속도 명령을 전송한다 (원격 제어용).
  rpc SendVelocityCommand(VelocityCommand) returns (VelocityCommandResponse);

  // 로봇에 내비게이션 목표를 설정한다 (미션 기반 이동).
  rpc SendNavigationGoal(NavigationGoal) returns (NavigationResult);

  // 장애를 주입한다 (테스트용).
  rpc InjectFault(InjectFaultRequest) returns (InjectFaultResponse);

  // 주입된 장애를 해제한다.
  rpc ClearFault(ClearFaultRequest) returns (ClearFaultResponse);

  // === 스트리밍 ===

  // 텔레메트리 스트림을 구독한다 (Server-streaming).
  rpc StreamTelemetry(amr.v1.common.TelemetrySubscription) returns (stream amr.v1.common.TelemetryMessage);

  // 모든 로봇의 전체 상태를 스트림으로 수신한다 (Server-streaming).
  rpc StreamRobotStates(StreamRobotStatesRequest) returns (stream amr.v1.common.RobotState);

  // 시뮬레이션 이벤트를 스트림으로 수신한다 (Server-streaming).
  rpc StreamEvents(StreamEventsRequest) returns (stream SimEvent);

  // === 시나리오 ===

  // 시나리오를 로드하여 실행한다.
  rpc RunScenario(RunScenarioRequest) returns (RunScenarioResponse);
}

// --- Request/Response 메시지 ---

message CreateSessionRequest {
  SimConfig config = 1;
  string name = 2;                    // 세션 이름 (선택)
}

message CreateSessionResponse {
  amr.v1.common.SimSessionId session_id = 1;
  SimSession session = 2;
}

message GetSessionRequest {
  amr.v1.common.SimSessionId session_id = 1;
}

message ListSessionsRequest {
  uint32 page_size = 1;
  string page_token = 2;
}

message ListSessionsResponse {
  repeated SimSession sessions = 1;
  string next_page_token = 2;
}

message DeleteSessionRequest {
  amr.v1.common.SimSessionId session_id = 1;
}

message DeleteSessionResponse {
  bool success = 1;
}

message StartRequest {
  amr.v1.common.SimSessionId session_id = 1;
}

message StartResponse {
  bool success = 1;
  string error_message = 2;
}

message PauseRequest {
  amr.v1.common.SimSessionId session_id = 1;
}

message PauseResponse {
  bool success = 1;
}

message ResumeRequest {
  amr.v1.common.SimSessionId session_id = 1;
}

message ResumeResponse {
  bool success = 1;
}

message StopRequest {
  amr.v1.common.SimSessionId session_id = 1;
}

message StopResponse {
  bool success = 1;
}

message ResetRequest {
  amr.v1.common.SimSessionId session_id = 1;
}

message ResetResponse {
  bool success = 1;
  SimSession session = 2;
}

message StepRequest {
  amr.v1.common.SimSessionId session_id = 1;
  double duration = 2;                // 진행할 시간 (초, 기본값: time_step)
}

message StepResponse {
  double new_sim_time = 1;
}

message SetTimeScaleRequest {
  amr.v1.common.SimSessionId session_id = 1;
  double real_time_factor = 2;
}

message SetTimeScaleResponse {
  bool success = 1;
}

message VelocityCommandResponse {
  bool success = 1;
}

message ClearFaultRequest {
  string fault_id = 1;
}

message ClearFaultResponse {
  bool success = 1;
}

message StreamRobotStatesRequest {
  amr.v1.common.SimSessionId session_id = 1;
  uint32 rate_hz = 2;                // 수신 빈도 (기본값 1)
}

message StreamEventsRequest {
  amr.v1.common.SimSessionId session_id = 1;
  repeated string event_types = 2;   // 필터링할 이벤트 타입 (빈 배열 = 전체)
}

message RunScenarioRequest {
  amr.v1.common.SimSessionId session_id = 1;
  ScenarioDefinition scenario = 2;
}

message RunScenarioResponse {
  bool success = 1;
  string error_message = 2;
}
```

#### 2.2.8 proto/asset/asset_types.proto

```protobuf
// proto/asset/asset_types.proto
//
// 에셋 관리 관련 타입 정의.
// 에셋은 로봇 모델(URDF/SDF), 3D 모델(glTF), 환경 모델 등을 포함한다.

syntax = "proto3";
package amr.v1.asset;

option java_multiple_files = true;
option java_package = "com.amr.v1.asset";

import "common/identifiers.proto";
import "common/geometry.proto";

// 에셋 유형.
enum AssetType {
  ASSET_TYPE_UNSPECIFIED = 0;
  ASSET_TYPE_ROBOT_MODEL = 1;         // 로봇 모델 (URDF/SDF)
  ASSET_TYPE_ENVIRONMENT_MODEL = 2;   // 환경 모델 (glTF, OBJ 등)
  ASSET_TYPE_TEXTURE = 3;             // 텍스처 (PNG, JPG 등)
  ASSET_TYPE_MATERIAL = 4;            // 재질 정의
  ASSET_TYPE_MESH = 5;                // 메시 데이터 (STL, DAE 등)
  ASSET_TYPE_PLUGIN = 6;              // WASM 플러그인
  ASSET_TYPE_ICON = 7;                // 아이콘 (SVG, PNG)
  ASSET_TYPE_THUMBNAIL = 8;           // 썸네일
}

// 에셋 파일 형식.
enum AssetFormat {
  ASSET_FORMAT_UNSPECIFIED = 0;
  ASSET_FORMAT_URDF = 1;              // Unified Robot Description Format
  ASSET_FORMAT_SDF = 2;               // Simulation Description Format
  ASSET_FORMAT_GLTF = 3;              // GL Transmission Format (JSON)
  ASSET_FORMAT_GLB = 4;               // GL Transmission Format (Binary)
  ASSET_FORMAT_OBJ = 5;              // Wavefront OBJ
  ASSET_FORMAT_STL = 6;              // STereoLithography
  ASSET_FORMAT_DAE = 7;              // COLLADA
  ASSET_FORMAT_FBX = 8;              // Filmbox
  ASSET_FORMAT_PNG = 9;
  ASSET_FORMAT_JPG = 10;
  ASSET_FORMAT_SVG = 11;
  ASSET_FORMAT_WASM = 12;             // WebAssembly
}

// 에셋 상태.
enum AssetStatus {
  ASSET_STATUS_UNSPECIFIED = 0;
  ASSET_STATUS_UPLOADING = 1;         // 업로드 중
  ASSET_STATUS_PROCESSING = 2;        // 처리 중 (변환, 검증 등)
  ASSET_STATUS_READY = 3;             // 사용 가능
  ASSET_STATUS_ERROR = 4;             // 오류 발생
  ASSET_STATUS_ARCHIVED = 5;          // 보관됨
}

// 에셋 디스크립터 (메타데이터).
message AssetDescriptor {
  amr.v1.common.AssetId id = 1;
  string name = 2;                     // 표시 이름
  string description = 3;             // 설명 (선택)
  AssetType asset_type = 4;
  AssetFormat original_format = 5;    // 원본 파일 형식
  AssetStatus status = 6;

  // 파일 정보
  string original_filename = 7;       // 원본 파일명
  uint64 file_size_bytes = 8;         // 파일 크기 (바이트)
  string checksum_sha256 = 9;         // SHA-256 체크섬

  // 변환된 포맷 목록
  repeated ConvertedAsset converted_assets = 10;

  // 로봇 모델 전용 메타데이터
  RobotModelMetadata robot_metadata = 11;

  // 바운딩 박스
  amr.v1.common.BoundingBox bounding_box = 12;

  // 타임스탬프
  int64 created_at_ms = 13;
  int64 updated_at_ms = 14;

  // 태그 (검색/분류용)
  repeated string tags = 15;

  // 소유자
  amr.v1.common.UserId owner_id = 16;

  // 썸네일 URL (presigned)
  string thumbnail_url = 17;

  // 버전
  uint32 version = 18;
}

// 변환된 에셋 정보.
message ConvertedAsset {
  AssetFormat format = 1;
  string storage_path = 2;            // MinIO 내 저장 경로
  uint64 file_size_bytes = 3;
  int64 converted_at_ms = 4;
  string download_url = 5;            // presigned URL (유효기간 제한)
}

// 로봇 모델 전용 메타데이터.
message RobotModelMetadata {
  string manufacturer = 1;
  string model_name = 2;
  repeated JointInfo joints = 3;
  repeated LinkInfo links = 4;
  repeated SensorInfo sensors = 5;
  KinematicsType kinematics = 6;
  double max_speed = 7;               // 최대 속도 (m/s)
  double max_payload = 8;             // 최대 적재 중량 (kg)
  double weight = 9;                  // 자체 중량 (kg)
}

enum KinematicsType {
  KINEMATICS_TYPE_UNSPECIFIED = 0;
  KINEMATICS_TYPE_DIFFERENTIAL = 1;   // 차동 구동
  KINEMATICS_TYPE_OMNIDIRECTIONAL = 2; // 전방향
  KINEMATICS_TYPE_ACKERMANN = 3;      // 애커만 조향
  KINEMATICS_TYPE_MECANUM = 4;        // 메카넘 휠
}

message JointInfo {
  string name = 1;
  string type = 2;                    // revolute, prismatic, fixed, continuous, floating, planar
  string parent_link = 3;
  string child_link = 4;
  amr.v1.common.Vector3 axis = 5;
  double lower_limit = 6;
  double upper_limit = 7;
  double max_effort = 8;
  double max_velocity = 9;
}

message LinkInfo {
  string name = 1;
  double mass = 2;                    // 질량 (kg)
  amr.v1.common.Vector3 inertia = 3; // 관성 모멘트 (Ixx, Iyy, Izz)
  bool has_visual = 4;
  bool has_collision = 5;
}

message SensorInfo {
  string name = 1;
  string type = 2;                    // lidar, camera, imu, ultrasonic, encoder
  string parent_link = 3;
  amr.v1.common.Transform pose_offset = 4; // 부모 링크 기준 센서 위치
  map<string, string> parameters = 5; // 센서별 파라미터
}

// 에셋 변환 작업.
message ConversionJob {
  amr.v1.common.ConversionJobId id = 1;
  amr.v1.common.AssetId source_asset_id = 2;
  AssetFormat source_format = 3;
  AssetFormat target_format = 4;
  ConversionStatus status = 5;
  double progress = 6;                // 진행률 (0.0 ~ 1.0)
  string error_message = 7;
  int64 created_at_ms = 8;
  int64 completed_at_ms = 9;
  ConversionOptions options = 10;
}

enum ConversionStatus {
  CONVERSION_STATUS_UNSPECIFIED = 0;
  CONVERSION_STATUS_QUEUED = 1;
  CONVERSION_STATUS_IN_PROGRESS = 2;
  CONVERSION_STATUS_COMPLETED = 3;
  CONVERSION_STATUS_FAILED = 4;
  CONVERSION_STATUS_CANCELLED = 5;
}

// 변환 옵션.
message ConversionOptions {
  bool generate_thumbnail = 1;        // 썸네일 생성 여부 (기본값 true)
  bool simplify_mesh = 2;             // 메시 단순화 (기본값 false)
  double target_triangle_count = 3;   // 목표 삼각형 수 (simplify_mesh가 true일 때)
  bool embed_textures = 4;            // 텍스처 임베딩 (glTF/GLB, 기본값 true)
  TextureResolution max_texture_resolution = 5;
}

enum TextureResolution {
  TEXTURE_RESOLUTION_UNSPECIFIED = 0;
  TEXTURE_RESOLUTION_256 = 1;
  TEXTURE_RESOLUTION_512 = 2;
  TEXTURE_RESOLUTION_1024 = 3;
  TEXTURE_RESOLUTION_2048 = 4;
  TEXTURE_RESOLUTION_4096 = 5;
  TEXTURE_RESOLUTION_ORIGINAL = 6;
}
```

#### 2.2.9 proto/asset/asset_service.proto

```protobuf
// proto/asset/asset_service.proto
//
// 에셋 매니저 gRPC 서비스 정의.
// Backend(Team 2)와 Sim Engine(Team 3)이 이 서비스의 클라이언트이다.

syntax = "proto3";
package amr.v1.asset;

option java_multiple_files = true;
option java_package = "com.amr.v1.asset";

import "common/identifiers.proto";
import "asset/asset_types.proto";

service AssetService {
  // === 에셋 CRUD ===

  // 에셋 메타데이터를 등록한다 (파일 업로드 전 단계).
  rpc CreateAsset(CreateAssetRequest) returns (CreateAssetResponse);

  // 에셋 정보를 조회한다.
  rpc GetAsset(GetAssetRequest) returns (AssetDescriptor);

  // 에셋 목록을 조회한다 (필터링/페이징 지원).
  rpc ListAssets(ListAssetsRequest) returns (ListAssetsResponse);

  // 에셋 메타데이터를 업데이트한다.
  rpc UpdateAsset(UpdateAssetRequest) returns (AssetDescriptor);

  // 에셋을 삭제(보관)한다.
  rpc DeleteAsset(DeleteAssetRequest) returns (DeleteAssetResponse);

  // === 파일 업로드/다운로드 ===

  // 에셋 파일 업로드를 위한 presigned URL을 발급한다.
  rpc GetUploadUrl(GetUploadUrlRequest) returns (GetUploadUrlResponse);

  // 에셋 파일 다운로드를 위한 presigned URL을 발급한다.
  rpc GetDownloadUrl(GetDownloadUrlRequest) returns (GetDownloadUrlResponse);

  // 업로드 완료를 통보한다 (이후 자동 변환 시작).
  rpc CompleteUpload(CompleteUploadRequest) returns (CompleteUploadResponse);

  // 에셋 파일을 gRPC 스트림으로 다운로드한다 (Sim Engine용).
  rpc DownloadAssetStream(DownloadAssetStreamRequest) returns (stream AssetChunk);

  // === 변환 ===

  // 명시적 형식 변환을 요청한다.
  rpc ConvertAsset(ConvertAssetRequest) returns (ConversionJob);

  // 변환 작업 상태를 조회한다.
  rpc GetConversionJob(GetConversionJobRequest) returns (ConversionJob);

  // 변환 작업 목록을 조회한다.
  rpc ListConversionJobs(ListConversionJobsRequest) returns (ListConversionJobsResponse);

  // === 검증 ===

  // URDF/SDF 파일의 유효성을 검증한다.
  rpc ValidateAsset(ValidateAssetRequest) returns (ValidateAssetResponse);
}

// --- Request/Response 메시지 ---

message CreateAssetRequest {
  string name = 1;
  string description = 2;
  AssetType asset_type = 3;
  AssetFormat format = 4;
  string original_filename = 5;
  uint64 file_size_bytes = 6;
  repeated string tags = 7;
}

message CreateAssetResponse {
  AssetDescriptor asset = 1;
  string upload_url = 2;             // presigned upload URL
}

message GetAssetRequest {
  amr.v1.common.AssetId asset_id = 1;
}

message ListAssetsRequest {
  uint32 page_size = 1;              // 페이지 크기 (기본값 20, 최대 100)
  string page_token = 2;
  AssetType type_filter = 3;         // 유형 필터 (UNSPECIFIED = 전체)
  AssetStatus status_filter = 4;     // 상태 필터
  string search_query = 5;           // 이름/태그 검색어
  string sort_by = 6;                // 정렬 필드: "name", "created_at", "updated_at", "size"
  bool sort_descending = 7;
}

message ListAssetsResponse {
  repeated AssetDescriptor assets = 1;
  string next_page_token = 2;
  uint32 total_count = 3;
}

message UpdateAssetRequest {
  amr.v1.common.AssetId asset_id = 1;
  optional string name = 2;
  optional string description = 3;
  repeated string tags = 4;
  bool replace_tags = 5;             // true: 태그 전체 교체, false: 태그 추가
}

message DeleteAssetRequest {
  amr.v1.common.AssetId asset_id = 1;
  bool permanent = 2;                // true: 영구 삭제, false: 보관
}

message DeleteAssetResponse {
  bool success = 1;
}

message GetUploadUrlRequest {
  amr.v1.common.AssetId asset_id = 1;
  string content_type = 2;           // MIME type
}

message GetUploadUrlResponse {
  string upload_url = 1;
  int64 expires_at_ms = 2;
}

message GetDownloadUrlRequest {
  amr.v1.common.AssetId asset_id = 1;
  AssetFormat format = 2;            // 원하는 형식 (UNSPECIFIED = 원본)
}

message GetDownloadUrlResponse {
  string download_url = 1;
  int64 expires_at_ms = 2;
  AssetFormat format = 3;
  uint64 file_size_bytes = 4;
}

message CompleteUploadRequest {
  amr.v1.common.AssetId asset_id = 1;
  string checksum_sha256 = 2;        // 클라이언트 측 체크섬 (무결성 검증)
  bool auto_convert = 3;             // 자동 변환 시작 여부 (기본값 true)
}

message CompleteUploadResponse {
  AssetDescriptor asset = 1;
  ConversionJob conversion_job = 2;  // 자동 변환이 시작된 경우
}

message DownloadAssetStreamRequest {
  amr.v1.common.AssetId asset_id = 1;
  AssetFormat format = 2;
}

message AssetChunk {
  bytes data = 1;                    // 청크 데이터 (최대 64KB)
  uint64 offset = 2;                 // 오프셋 (바이트)
  uint64 total_size = 3;             // 전체 크기 (바이트)
}

message ConvertAssetRequest {
  amr.v1.common.AssetId asset_id = 1;
  AssetFormat target_format = 2;
  ConversionOptions options = 3;
}

message GetConversionJobRequest {
  amr.v1.common.ConversionJobId job_id = 1;
}

message ListConversionJobsRequest {
  amr.v1.common.AssetId asset_id = 1;  // 특정 에셋의 작업만 (선택)
  uint32 page_size = 2;
  string page_token = 3;
}

message ListConversionJobsResponse {
  repeated ConversionJob jobs = 1;
  string next_page_token = 2;
}

message ValidateAssetRequest {
  amr.v1.common.AssetId asset_id = 1;
}

message ValidateAssetResponse {
  bool valid = 1;
  repeated ValidationError errors = 2;
  repeated ValidationError warnings = 3;
}

message ValidationError {
  string code = 1;                   // 에러 코드
  string message = 2;               // 에러 메시지
  string location = 3;              // 에러 위치 (XPath 등)
  string severity = 4;              // "error" | "warning"
}
```

#### 2.2.10 proto/map/map_types.proto

```protobuf
// proto/map/map_types.proto
//
// 맵 관리 관련 타입 정의.
// 포인트 클라우드, Potree 타일, 로드맵 그래프, 의미론적 영역을 포함한다.

syntax = "proto3";
package amr.v1.map;

option java_multiple_files = true;
option java_package = "com.amr.v1.map";

import "common/identifiers.proto";
import "common/geometry.proto";

// 맵 상태.
enum MapStatus {
  MAP_STATUS_UNSPECIFIED = 0;
  MAP_STATUS_UPLOADING = 1;           // 포인트 클라우드 업로드 중
  MAP_STATUS_PROCESSING = 2;         // Potree 타일링 처리 중
  MAP_STATUS_READY = 3;              // 사용 가능
  MAP_STATUS_ERROR = 4;              // 처리 오류
  MAP_STATUS_ARCHIVED = 5;           // 보관됨
}

// 맵 소스 형식.
enum MapSourceFormat {
  MAP_SOURCE_FORMAT_UNSPECIFIED = 0;
  MAP_SOURCE_FORMAT_PCD = 1;         // Point Cloud Data
  MAP_SOURCE_FORMAT_LAS = 2;         // LASer file
  MAP_SOURCE_FORMAT_LAZ = 3;         // Compressed LAS
  MAP_SOURCE_FORMAT_PLY = 4;         // Polygon File Format
  MAP_SOURCE_FORMAT_E57 = 5;         // ASTM E57
  MAP_SOURCE_FORMAT_XYZ = 6;         // ASCII XYZ
}

// 맵 디스크립터 (메타데이터).
message MapDescriptor {
  amr.v1.common.MapId id = 1;
  string name = 2;
  string description = 3;
  MapStatus status = 4;

  // 소스 정보
  MapSourceFormat source_format = 5;
  string original_filename = 6;
  uint64 file_size_bytes = 7;

  // 공간 정보
  amr.v1.common.BoundingBox bounding_box = 8;  // 맵 범위
  uint64 point_count = 9;             // 포인트 수
  double resolution = 10;             // 평균 포인트 해상도 (미터)

  // Potree 타일 정보
  PotreeMetadata potree = 11;

  // 연결된 로드맵
  repeated amr.v1.common.RoadmapId roadmap_ids = 12;

  // 의미론적 영역
  repeated amr.v1.common.RegionId region_ids = 13;

  // 타임스탬프
  int64 created_at_ms = 14;
  int64 updated_at_ms = 15;

  // 좌표계 정보
  string coordinate_system = 16;       // 예: "ENU", "UTM-52N"
  amr.v1.common.Vector3 origin_offset = 17; // 원점 오프셋

  // 소유자
  amr.v1.common.UserId owner_id = 18;

  // 태그
  repeated string tags = 19;

  // 처리 진행률 (PROCESSING 상태일 때)
  double processing_progress = 20;    // 0.0 ~ 1.0

  // 바닥면 높이 (2D 내비게이션용)
  double ground_plane_z = 21;
}

// Potree 메타데이터.
message PotreeMetadata {
  string base_url = 1;                // Potree 타일 베이스 URL
  uint32 hierarchy_depth = 2;         // 옥트리 깊이
  uint32 tile_count = 3;              // 타일 수
  double spacing = 4;                 // 루트 노드 포인트 간격 (미터)
  amr.v1.common.BoundingBox tight_bounding_box = 5;
  repeated string available_attributes = 6; // 사용 가능한 속성 (position, color, intensity 등)
}

// 타일 데이터 (개별 Potree 노드).
message TileData {
  string node_name = 1;               // 노드 이름 (예: "r", "r0", "r01")
  bytes point_data = 2;               // 포인트 데이터 (바이너리)
  uint32 point_count = 3;
  amr.v1.common.BoundingBox bounding_box = 4;
  uint32 level = 5;                    // LOD 레벨
  string encoding = 6;                // 인코딩 방식 (예: "LAZ")
}

// 로드맵 그래프 (로봇 내비게이션 경로).
message Roadmap {
  amr.v1.common.RoadmapId id = 1;
  amr.v1.common.MapId map_id = 2;     // 소속 맵
  string name = 3;
  string description = 4;

  repeated RoadmapNode nodes = 5;
  repeated RoadmapEdge edges = 6;

  int64 created_at_ms = 7;
  int64 updated_at_ms = 8;

  // 그래프 통계
  uint32 node_count = 9;
  uint32 edge_count = 10;
}

// 로드맵 노드.
message RoadmapNode {
  amr.v1.common.NodeId id = 1;
  amr.v1.common.Pose2D pose = 2;      // 노드 위치

  NodeType node_type = 3;
  string name = 4;                     // 표시 이름 (선택)
  string description = 5;

  // VDA5050 호환 속성
  map<string, string> actions = 6;    // 노드에서 수행 가능한 액션
  bool allowed_deviation = 7;          // 위치 편차 허용 여부

  // 메타데이터
  map<string, string> properties = 8;
}

enum NodeType {
  NODE_TYPE_UNSPECIFIED = 0;
  NODE_TYPE_WAYPOINT = 1;             // 일반 경유점
  NODE_TYPE_CHARGING_STATION = 2;     // 충전 스테이션
  NODE_TYPE_LOADING_POINT = 3;        // 적재 지점
  NODE_TYPE_UNLOADING_POINT = 4;      // 하역 지점
  NODE_TYPE_WAITING_POINT = 5;        // 대기 지점
  NODE_TYPE_PARKING = 6;             // 주차 지점
  NODE_TYPE_INTERSECTION = 7;        // 교차점
  NODE_TYPE_ELEVATOR = 8;            // 엘리베이터
  NODE_TYPE_DOOR = 9;                // 출입문
}

// 로드맵 엣지.
message RoadmapEdge {
  amr.v1.common.EdgeId id = 1;
  amr.v1.common.NodeId start_node_id = 2;
  amr.v1.common.NodeId end_node_id = 3;

  EdgeType edge_type = 4;
  bool bidirectional = 5;              // 양방향 여부 (기본값 true)

  double length = 6;                   // 엣지 길이 (미터)
  double max_speed = 7;               // 최대 속도 (m/s)
  double max_height = 8;              // 최대 높이 제한 (미터)
  double max_weight = 9;              // 최대 하중 제한 (kg)

  // 경로 보간 (직선이 아닌 경우)
  repeated amr.v1.common.Point2D control_points = 10; // 베지어 제어점

  // VDA5050 호환 속성
  string orientation_type = 11;       // "GLOBAL", "TANGENTIAL"
  double orientation = 12;            // 엣지 방향 (라디안)
  double rotation_allowed = 13;       // 허용 회전 각도 (라디안)
  map<string, string> actions = 14;   // 엣지에서 수행 가능한 액션

  // 교통 제어
  uint32 max_robots = 15;             // 동시 진입 가능한 최대 로봇 수 (0 = 무제한)

  // 메타데이터
  map<string, string> properties = 16;
}

enum EdgeType {
  EDGE_TYPE_UNSPECIFIED = 0;
  EDGE_TYPE_LINE = 1;                 // 직선
  EDGE_TYPE_CURVE = 2;               // 곡선 (베지어)
  EDGE_TYPE_RAMP = 3;               // 경사로
  EDGE_TYPE_ELEVATOR = 4;           // 엘리베이터 (수직 이동)
}

// 의미론적 영역 (구역 정의).
message SemanticRegion {
  amr.v1.common.RegionId id = 1;
  amr.v1.common.MapId map_id = 2;
  string name = 3;
  string description = 4;

  RegionType region_type = 5;
  amr.v1.common.Polygon2D boundary = 6; // 영역 경계 다각형
  amr.v1.common.Color color = 7;        // 시각화 색상

  // 규칙
  double speed_limit = 8;             // 속도 제한 (m/s, 0 = 제한 없음)
  bool entry_restricted = 9;          // 진입 제한 여부
  repeated string allowed_robot_ids = 10; // 진입 허용된 로봇 ID (빈 배열 = 전체 허용)
  uint32 max_robots = 11;             // 최대 동시 진입 로봇 수 (0 = 무제한)

  // 스케줄 (시간대별 규칙 적용)
  repeated Schedule schedules = 12;

  // 메타데이터
  map<string, string> properties = 13;
  int64 created_at_ms = 14;
  int64 updated_at_ms = 15;
}

enum RegionType {
  REGION_TYPE_UNSPECIFIED = 0;
  REGION_TYPE_ZONE = 1;              // 일반 구역
  REGION_TYPE_RESTRICTED = 2;        // 제한 구역
  REGION_TYPE_CHARGING = 3;          // 충전 구역
  REGION_TYPE_LOADING = 4;           // 적하 구역
  REGION_TYPE_PARKING = 5;           // 주차 구역
  REGION_TYPE_SLOW = 6;             // 서행 구역
  REGION_TYPE_ONE_WAY = 7;          // 일방통행 구역
  REGION_TYPE_BUFFER = 8;           // 버퍼 (대기) 구역
  REGION_TYPE_DANGER = 9;           // 위험 구역
  REGION_TYPE_CLEAN_ROOM = 10;      // 클린룸
}

// 시간 스케줄.
message Schedule {
  string cron_expression = 1;        // 크론 표현식 (적용 시간)
  bool active = 2;                    // true: 규칙 활성화, false: 비활성화
}

// 경로 탐색 요청.
message PathfindingRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
  amr.v1.common.NodeId start_node_id = 2;
  amr.v1.common.NodeId goal_node_id = 3;
  PathfindingAlgorithm algorithm = 4;
  amr.v1.common.RobotId robot_id = 5;  // 로봇별 제약 조건 적용을 위해
  PathConstraints constraints = 6;
}

enum PathfindingAlgorithm {
  PATHFINDING_ALGORITHM_UNSPECIFIED = 0;
  PATHFINDING_ALGORITHM_DIJKSTRA = 1;
  PATHFINDING_ALGORITHM_A_STAR = 2;
  PATHFINDING_ALGORITHM_THETA_STAR = 3;
}

message PathConstraints {
  double max_path_length = 1;         // 최대 경로 길이 (미터, 0 = 무제한)
  double max_height = 2;              // 로봇 높이 (미터)
  double max_weight = 3;              // 로봇 하중 (kg)
  repeated string avoid_region_ids = 4; // 회피할 영역 ID
}

// 경로 탐색 결과.
message PathfindingResult {
  bool found = 1;
  repeated amr.v1.common.NodeId path_node_ids = 2;  // 경유 노드 ID 리스트
  repeated amr.v1.common.EdgeId path_edge_ids = 3;  // 경유 엣지 ID 리스트
  double total_distance = 4;          // 총 거리 (미터)
  double estimated_time = 5;          // 예상 소요 시간 (초)
}
```

#### 2.2.11 proto/map/map_service.proto

```protobuf
// proto/map/map_service.proto
//
// 맵 매니저 gRPC 서비스 정의.
// Backend(Team 2)와 Sim Engine(Team 3)이 이 서비스의 클라이언트이다.

syntax = "proto3";
package amr.v1.map;

option java_multiple_files = true;
option java_package = "com.amr.v1.map";

import "common/identifiers.proto";
import "map/map_types.proto";

service MapService {
  // === 맵 CRUD ===

  // 맵 메타데이터를 등록한다 (포인트 클라우드 업로드 전 단계).
  rpc CreateMap(CreateMapRequest) returns (CreateMapResponse);

  // 맵 정보를 조회한다.
  rpc GetMap(GetMapRequest) returns (MapDescriptor);

  // 맵 목록을 조회한다.
  rpc ListMaps(ListMapsRequest) returns (ListMapsResponse);

  // 맵 메타데이터를 업데이트한다.
  rpc UpdateMap(UpdateMapRequest) returns (MapDescriptor);

  // 맵을 삭제한다.
  rpc DeleteMap(DeleteMapRequest) returns (DeleteMapResponse);

  // === 포인트 클라우드 업로드/처리 ===

  // 포인트 클라우드 업로드를 위한 presigned URL을 발급한다.
  rpc GetUploadUrl(GetMapUploadUrlRequest) returns (GetMapUploadUrlResponse);

  // 업로드 완료를 통보한다 (Potree 타일링 시작).
  rpc CompleteUpload(CompleteMapUploadRequest) returns (CompleteMapUploadResponse);

  // 처리 상태를 조회한다.
  rpc GetProcessingStatus(GetProcessingStatusRequest) returns (GetProcessingStatusResponse);

  // === 타일 서빙 ===

  // Potree 타일을 조회한다.
  rpc GetTile(GetTileRequest) returns (TileData);

  // 다수 타일을 스트림으로 조회한다 (Sim Engine에서 맵 로딩 시 사용).
  rpc StreamTiles(StreamTilesRequest) returns (stream TileData);

  // Potree 메타데이터를 조회한다.
  rpc GetPotreeMetadata(GetPotreeMetadataRequest) returns (PotreeMetadata);

  // === 로드맵 관리 ===

  // 로드맵을 생성한다.
  rpc CreateRoadmap(CreateRoadmapRequest) returns (Roadmap);

  // 로드맵을 조회한다.
  rpc GetRoadmap(GetRoadmapRequest) returns (Roadmap);

  // 맵에 연결된 로드맵 목록을 조회한다.
  rpc ListRoadmaps(ListRoadmapsRequest) returns (ListRoadmapsResponse);

  // 로드맵을 업데이트한다 (전체 교체).
  rpc UpdateRoadmap(UpdateRoadmapRequest) returns (Roadmap);

  // 로드맵을 삭제한다.
  rpc DeleteRoadmap(DeleteRoadmapRequest) returns (DeleteRoadmapResponse);

  // === 로드맵 노드/엣지 개별 관리 ===

  // 노드를 추가한다.
  rpc AddNode(AddNodeRequest) returns (RoadmapNode);

  // 노드를 업데이트한다.
  rpc UpdateNode(UpdateNodeRequest) returns (RoadmapNode);

  // 노드를 삭제한다 (연결된 엣지도 삭제).
  rpc RemoveNode(RemoveNodeRequest) returns (RemoveNodeResponse);

  // 엣지를 추가한다.
  rpc AddEdge(AddEdgeRequest) returns (RoadmapEdge);

  // 엣지를 업데이트한다.
  rpc UpdateEdge(UpdateEdgeRequest) returns (RoadmapEdge);

  // 엣지를 삭제한다.
  rpc RemoveEdge(RemoveEdgeRequest) returns (RemoveEdgeResponse);

  // === 의미론적 영역 관리 ===

  // 영역을 생성한다.
  rpc CreateRegion(CreateRegionRequest) returns (SemanticRegion);

  // 영역을 조회한다.
  rpc GetRegion(GetRegionRequest) returns (SemanticRegion);

  // 맵의 영역 목록을 조회한다.
  rpc ListRegions(ListRegionsRequest) returns (ListRegionsResponse);

  // 영역을 업데이트한다.
  rpc UpdateRegion(UpdateRegionRequest) returns (SemanticRegion);

  // 영역을 삭제한다.
  rpc DeleteRegion(DeleteRegionRequest) returns (DeleteRegionResponse);

  // === 경로 탐색 ===

  // 두 노드 간 최단 경로를 탐색한다.
  rpc FindPath(PathfindingRequest) returns (PathfindingResult);

  // === 공간 쿼리 ===

  // 좌표 기반으로 가장 가까운 노드를 찾는다.
  rpc FindNearestNode(FindNearestNodeRequest) returns (FindNearestNodeResponse);

  // 좌표가 어떤 영역에 속하는지 조회한다.
  rpc GetRegionsAtPoint(GetRegionsAtPointRequest) returns (GetRegionsAtPointResponse);
}

// --- Request/Response 메시지 ---

message CreateMapRequest {
  string name = 1;
  string description = 2;
  MapSourceFormat source_format = 3;
  string original_filename = 4;
  uint64 file_size_bytes = 5;
  string coordinate_system = 6;
  amr.v1.common.Vector3 origin_offset = 7;
  repeated string tags = 8;
}

message CreateMapResponse {
  MapDescriptor map = 1;
  string upload_url = 2;
}

message GetMapRequest {
  amr.v1.common.MapId map_id = 1;
}

message ListMapsRequest {
  uint32 page_size = 1;
  string page_token = 2;
  MapStatus status_filter = 3;
  string search_query = 4;
  string sort_by = 5;
  bool sort_descending = 6;
}

message ListMapsResponse {
  repeated MapDescriptor maps = 1;
  string next_page_token = 2;
  uint32 total_count = 3;
}

message UpdateMapRequest {
  amr.v1.common.MapId map_id = 1;
  optional string name = 2;
  optional string description = 3;
  repeated string tags = 4;
  bool replace_tags = 5;
  optional double ground_plane_z = 6;
}

message DeleteMapRequest {
  amr.v1.common.MapId map_id = 1;
  bool permanent = 2;
}

message DeleteMapResponse {
  bool success = 1;
}

message GetMapUploadUrlRequest {
  amr.v1.common.MapId map_id = 1;
  string content_type = 2;
}

message GetMapUploadUrlResponse {
  string upload_url = 1;
  int64 expires_at_ms = 2;
}

message CompleteMapUploadRequest {
  amr.v1.common.MapId map_id = 1;
  string checksum_sha256 = 2;
}

message CompleteMapUploadResponse {
  MapDescriptor map = 1;
}

message GetProcessingStatusRequest {
  amr.v1.common.MapId map_id = 1;
}

message GetProcessingStatusResponse {
  MapStatus status = 1;
  double progress = 2;
  string error_message = 3;
  ProcessingStats stats = 4;
}

message ProcessingStats {
  uint64 points_processed = 1;
  uint64 total_points = 2;
  uint32 tiles_generated = 3;
  double elapsed_seconds = 4;
}

message GetTileRequest {
  amr.v1.common.MapId map_id = 1;
  string node_name = 2;
}

message StreamTilesRequest {
  amr.v1.common.MapId map_id = 1;
  uint32 max_level = 2;               // 최대 LOD 레벨 (0 = 전체)
  amr.v1.common.BoundingBox bounds = 3; // 범위 필터 (비어있으면 전체)
}

message GetPotreeMetadataRequest {
  amr.v1.common.MapId map_id = 1;
}

message CreateRoadmapRequest {
  amr.v1.common.MapId map_id = 1;
  string name = 2;
  string description = 3;
  repeated RoadmapNode nodes = 4;      // 초기 노드 (선택)
  repeated RoadmapEdge edges = 5;      // 초기 엣지 (선택)
}

message GetRoadmapRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
}

message ListRoadmapsRequest {
  amr.v1.common.MapId map_id = 1;
  uint32 page_size = 2;
  string page_token = 3;
}

message ListRoadmapsResponse {
  repeated Roadmap roadmaps = 1;
  string next_page_token = 2;
}

message UpdateRoadmapRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
  optional string name = 2;
  optional string description = 3;
  repeated RoadmapNode nodes = 4;
  repeated RoadmapEdge edges = 5;
}

message DeleteRoadmapRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
}

message DeleteRoadmapResponse {
  bool success = 1;
}

message AddNodeRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
  RoadmapNode node = 2;
}

message UpdateNodeRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
  RoadmapNode node = 2;
}

message RemoveNodeRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
  amr.v1.common.NodeId node_id = 2;
}

message RemoveNodeResponse {
  bool success = 1;
  repeated amr.v1.common.EdgeId removed_edge_ids = 2; // 함께 삭제된 엣지 ID
}

message AddEdgeRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
  RoadmapEdge edge = 2;
}

message UpdateEdgeRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
  RoadmapEdge edge = 2;
}

message RemoveEdgeRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
  amr.v1.common.EdgeId edge_id = 2;
}

message RemoveEdgeResponse {
  bool success = 1;
}

message CreateRegionRequest {
  amr.v1.common.MapId map_id = 1;
  SemanticRegion region = 2;
}

message GetRegionRequest {
  amr.v1.common.RegionId region_id = 1;
}

message ListRegionsRequest {
  amr.v1.common.MapId map_id = 1;
  RegionType type_filter = 2;
  uint32 page_size = 3;
  string page_token = 4;
}

message ListRegionsResponse {
  repeated SemanticRegion regions = 1;
  string next_page_token = 2;
  uint32 total_count = 3;
}

message UpdateRegionRequest {
  SemanticRegion region = 1;
}

message DeleteRegionRequest {
  amr.v1.common.RegionId region_id = 1;
}

message DeleteRegionResponse {
  bool success = 1;
}

message FindNearestNodeRequest {
  amr.v1.common.RoadmapId roadmap_id = 1;
  amr.v1.common.Point2D point = 2;
  double max_distance = 3;           // 최대 검색 반경 (미터, 0 = 무제한)
  uint32 limit = 4;                   // 결과 수 (기본값 1)
}

message FindNearestNodeResponse {
  repeated NearestNodeResult results = 1;
}

message NearestNodeResult {
  RoadmapNode node = 1;
  double distance = 2;               // 거리 (미터)
}

message GetRegionsAtPointRequest {
  amr.v1.common.MapId map_id = 1;
  amr.v1.common.Point2D point = 2;
}

message GetRegionsAtPointResponse {
  repeated SemanticRegion regions = 1;
}
```

#### 2.2.12 proto/mission/mission_types.proto

```protobuf
// proto/mission/mission_types.proto
//
// 미션 관련 타입 정의.
// 미션은 Backend에서 관리하며, Sim Engine과 실제 로봇에 전달된다.
// VDA5050 Order 메시지와 호환 가능하도록 설계되었다.

syntax = "proto3";
package amr.v1.mission;

option java_multiple_files = true;
option java_package = "com.amr.v1.mission";

import "common/identifiers.proto";
import "common/geometry.proto";

// 미션 상태.
enum MissionStatus {
  MISSION_STATUS_UNSPECIFIED = 0;
  MISSION_STATUS_DRAFT = 1;          // 초안 (아직 배정되지 않음)
  MISSION_STATUS_QUEUED = 2;         // 대기열에 추가됨
  MISSION_STATUS_ASSIGNED = 3;       // 로봇에 배정됨
  MISSION_STATUS_IN_PROGRESS = 4;    // 수행 중
  MISSION_STATUS_PAUSED = 5;         // 일시 정지
  MISSION_STATUS_COMPLETED = 6;      // 완료
  MISSION_STATUS_FAILED = 7;         // 실패
  MISSION_STATUS_CANCELLED = 8;      // 취소됨
  MISSION_STATUS_ABORTING = 9;       // 중단 중
}

// 미션 우선순위.
enum MissionPriority {
  MISSION_PRIORITY_UNSPECIFIED = 0;
  MISSION_PRIORITY_LOW = 1;
  MISSION_PRIORITY_NORMAL = 2;
  MISSION_PRIORITY_HIGH = 3;
  MISSION_PRIORITY_URGENT = 4;
  MISSION_PRIORITY_CRITICAL = 5;
}

// 미션 정의.
message Mission {
  amr.v1.common.MissionId id = 1;
  string name = 2;
  string description = 3;

  MissionStatus status = 4;
  MissionPriority priority = 5;

  // 배정된 로봇
  amr.v1.common.RobotId assigned_robot_id = 6;

  // 맵/로드맵 참조
  amr.v1.common.MapId map_id = 7;
  amr.v1.common.RoadmapId roadmap_id = 8;

  // 미션 단계
  repeated MissionStep steps = 9;

  // 현재 진행 상태
  uint32 current_step_index = 10;     // 현재 수행 중인 단계 인덱스
  double progress = 11;               // 전체 진행률 (0.0 ~ 1.0)

  // 경로 정보
  repeated amr.v1.common.NodeId planned_path = 12;

  // 타임스탬프
  int64 created_at_ms = 13;
  int64 started_at_ms = 14;
  int64 completed_at_ms = 15;
  int64 updated_at_ms = 16;

  // 소유자
  amr.v1.common.UserId created_by = 17;

  // 오류 정보 (FAILED 상태일 때)
  string failure_reason = 18;

  // 예상 소요 시간 (초)
  double estimated_duration = 19;
  // 실제 소요 시간 (초)
  double actual_duration = 20;

  // 이동 거리 (미터)
  double total_distance = 21;

  // 재시도 정보
  uint32 retry_count = 22;
  uint32 max_retries = 23;

  // 메타데이터
  map<string, string> metadata = 24;
}

// 미션 단계.
message MissionStep {
  string step_id = 1;                  // 단계 ID (미션 내 고유)
  uint32 sequence = 2;                // 실행 순서

  StepType step_type = 3;
  MissionStepStatus status = 4;

  // 목적지 (NAVIGATE 타입인 경우)
  amr.v1.common.NodeId target_node_id = 5;
  amr.v1.common.Pose2D target_pose = 6;

  // 액션 (ACTION 타입인 경우)
  repeated MissionAction actions = 7;

  // 대기 (WAIT 타입인 경우)
  double wait_duration_seconds = 8;   // 대기 시간 (초, 0 = 조건 대기)
  string wait_condition = 9;          // 대기 조건 표현식

  // 타임스탬프
  int64 started_at_ms = 10;
  int64 completed_at_ms = 11;

  // 오류 정보
  string failure_reason = 12;

  // 설명
  string description = 13;
}

enum StepType {
  STEP_TYPE_UNSPECIFIED = 0;
  STEP_TYPE_NAVIGATE = 1;            // 목적지로 이동
  STEP_TYPE_ACTION = 2;              // 액션 수행 (적재, 하역 등)
  STEP_TYPE_WAIT = 3;                // 대기
  STEP_TYPE_CHARGE = 4;             // 충전
}

enum MissionStepStatus {
  MISSION_STEP_STATUS_UNSPECIFIED = 0;
  MISSION_STEP_STATUS_PENDING = 1;
  MISSION_STEP_STATUS_IN_PROGRESS = 2;
  MISSION_STEP_STATUS_COMPLETED = 3;
  MISSION_STEP_STATUS_FAILED = 4;
  MISSION_STEP_STATUS_SKIPPED = 5;
}

// 미션 액션 (노드에서 수행하는 작업).
message MissionAction {
  string action_id = 1;
  string action_type = 2;            // VDA5050 actionType (예: "pick", "drop", "scan", "wait")
  ActionStatus status = 3;
  map<string, string> parameters = 4; // 액션 파라미터
  BlockingType blocking_type = 5;
}

enum ActionStatus {
  ACTION_STATUS_UNSPECIFIED = 0;
  ACTION_STATUS_WAITING = 1;
  ACTION_STATUS_INITIALIZING = 2;
  ACTION_STATUS_RUNNING = 3;
  ACTION_STATUS_PAUSED = 4;
  ACTION_STATUS_FINISHED = 5;
  ACTION_STATUS_FAILED = 6;
}

enum BlockingType {
  BLOCKING_TYPE_UNSPECIFIED = 0;
  BLOCKING_TYPE_NONE = 1;            // 비차단 (이동 중 실행 가능)
  BLOCKING_TYPE_SOFT = 2;            // 소프트 차단 (이동 중 시작하되 노드에서 완료)
  BLOCKING_TYPE_HARD = 3;            // 하드 차단 (노드 도착 후 실행)
}

// 교통 규칙.
message TrafficRule {
  string rule_id = 1;
  string name = 2;
  TrafficRuleType rule_type = 3;

  // 적용 대상
  repeated amr.v1.common.EdgeId edge_ids = 4;
  repeated amr.v1.common.RegionId region_ids = 5;

  // 규칙 파라미터
  uint32 max_robots = 6;              // 최대 동시 로봇 수
  double speed_limit = 7;             // 속도 제한 (m/s)
  DirectionRule direction = 8;

  // 스케줄
  repeated Schedule schedules = 9;

  bool enabled = 10;
}

enum TrafficRuleType {
  TRAFFIC_RULE_TYPE_UNSPECIFIED = 0;
  TRAFFIC_RULE_TYPE_MUTEX_ZONE = 1;   // 상호 배제 구역
  TRAFFIC_RULE_TYPE_CAPACITY_LIMIT = 2; // 용량 제한
  TRAFFIC_RULE_TYPE_SPEED_LIMIT = 3;  // 속도 제한
  TRAFFIC_RULE_TYPE_DIRECTION = 4;    // 방향 제한
  TRAFFIC_RULE_TYPE_PRIORITY = 5;     // 우선 순위
}

enum DirectionRule {
  DIRECTION_RULE_UNSPECIFIED = 0;
  DIRECTION_RULE_FORWARD_ONLY = 1;
  DIRECTION_RULE_REVERSE_ONLY = 2;
  DIRECTION_RULE_BIDIRECTIONAL = 3;
}

// 시간 스케줄 (map_types.proto의 Schedule과 동일, 미션 패키지 내 독립 정의).
message Schedule {
  string cron_expression = 1;
  bool active = 2;
}

// 미션 업데이트 이벤트 (WebSocket 스트리밍용).
message MissionUpdateEvent {
  amr.v1.common.MissionId mission_id = 1;
  MissionStatus status = 2;
  uint32 current_step_index = 3;
  double progress = 4;
  string event_description = 5;
  int64 timestamp_ms = 6;
}
```

---

### 2.3 REST API Schema (OpenAPI 3.1)

Backend(Team 2)가 제공하는 REST API이며, Frontend(Team 1)가 소비한다. 모든 엔드포인트는 `/api/v1` 접두사를 사용한다.

```yaml
# openapi-spec.yaml (요약)
openapi: "3.1.0"
info:
  title: AMR Integrated Framework API
  version: "1.0.0"
  description: |
    AMR 통합 프레임워크 Backend REST API.
    Frontend(Team 1)가 이 API를 통해 모든 기능에 접근한다.
    인증은 JWT Bearer 토큰을 사용한다.

servers:
  - url: http://localhost:8080/api/v1
    description: 로컬 개발 서버
  - url: https://api.amr.example.com/api/v1
    description: 프로덕션 서버

security:
  - BearerAuth: []

components:
  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
      description: |
        JWT 토큰. /api/v1/auth/login 에서 발급.
        Access Token 유효기간: 15분.
        Refresh Token 유효기간: 7일.

  schemas:
    # --- 공통 ---
    Pagination:
      type: object
      properties:
        page:
          type: integer
          minimum: 1
          default: 1
        pageSize:
          type: integer
          minimum: 1
          maximum: 100
          default: 20
        totalCount:
          type: integer
        totalPages:
          type: integer

    ErrorResponse:
      type: object
      required: [code, message]
      properties:
        code:
          type: integer
          description: "에러 코드 (proto ErrorCode와 동일)"
        message:
          type: string
        target:
          type: string
        traceId:
          type: string
          format: uuid
        details:
          type: array
          items:
            $ref: "#/components/schemas/ErrorDetail"

    ErrorDetail:
      type: object
      properties:
        code:
          type: integer
        message:
          type: string
        target:
          type: string

    # --- 인증 ---
    LoginRequest:
      type: object
      required: [email, password]
      properties:
        email:
          type: string
          format: email
        password:
          type: string
          minLength: 8

    LoginResponse:
      type: object
      properties:
        accessToken:
          type: string
        refreshToken:
          type: string
        expiresIn:
          type: integer
          description: "Access Token 만료 시간 (초)"
        user:
          $ref: "#/components/schemas/User"

    RefreshRequest:
      type: object
      required: [refreshToken]
      properties:
        refreshToken:
          type: string

    User:
      type: object
      properties:
        id:
          type: string
          format: uuid
        email:
          type: string
          format: email
        name:
          type: string
        role:
          type: string
          enum: [admin, operator, viewer]
        createdAt:
          type: string
          format: date-time

    # --- 로봇 ---
    Robot:
      type: object
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string
        assetId:
          type: string
          format: uuid
        mapId:
          type: string
          format: uuid
        connectionStatus:
          type: string
          enum: [online, offline, connection_broken]
        operationMode:
          type: string
          enum: [automatic, semi_automatic, manual, service, teach_in]
        pose:
          $ref: "#/components/schemas/Pose3D"
        velocity:
          $ref: "#/components/schemas/Twist"
        battery:
          $ref: "#/components/schemas/BatteryStatus"
        currentMissionId:
          type: string
          format: uuid
          nullable: true
        driving:
          type: boolean
        errors:
          type: array
          items:
            $ref: "#/components/schemas/RobotError"
        createdAt:
          type: string
          format: date-time
        updatedAt:
          type: string
          format: date-time

    Pose3D:
      type: object
      properties:
        position:
          $ref: "#/components/schemas/Vector3"
        orientation:
          $ref: "#/components/schemas/Quaternion"

    Vector3:
      type: object
      properties:
        x: { type: number, format: double }
        y: { type: number, format: double }
        z: { type: number, format: double }

    Quaternion:
      type: object
      properties:
        x: { type: number, format: double }
        y: { type: number, format: double }
        z: { type: number, format: double }
        w: { type: number, format: double }

    Twist:
      type: object
      properties:
        linear:
          $ref: "#/components/schemas/Vector3"
        angular:
          $ref: "#/components/schemas/Vector3"

    BatteryStatus:
      type: object
      properties:
        chargePercentage: { type: number }
        voltage: { type: number }
        current: { type: number }
        temperature: { type: number }
        isCharging: { type: boolean }
        estimatedMinutesRemaining: { type: integer }

    RobotError:
      type: object
      properties:
        errorType: { type: string }
        description: { type: string }
        level: { type: string, enum: [warning, fatal] }

    CreateRobotRequest:
      type: object
      required: [name, assetId]
      properties:
        name: { type: string, minLength: 1, maxLength: 100 }
        assetId: { type: string, format: uuid }
        mapId: { type: string, format: uuid }
        initialPose:
          $ref: "#/components/schemas/Pose3D"

    # --- 미션 ---
    MissionCreate:
      type: object
      required: [name, mapId, roadmapId, steps]
      properties:
        name: { type: string }
        description: { type: string }
        priority:
          type: string
          enum: [low, normal, high, urgent, critical]
          default: normal
        robotId:
          type: string
          format: uuid
          nullable: true
          description: "배정할 로봇 ID (null이면 자동 배정)"
        mapId: { type: string, format: uuid }
        roadmapId: { type: string, format: uuid }
        steps:
          type: array
          items:
            $ref: "#/components/schemas/MissionStepCreate"
        maxRetries: { type: integer, default: 0 }
        metadata:
          type: object
          additionalProperties: { type: string }

    MissionStepCreate:
      type: object
      required: [stepType]
      properties:
        stepType:
          type: string
          enum: [navigate, action, wait, charge]
        targetNodeId:
          type: string
          format: uuid
        actions:
          type: array
          items:
            $ref: "#/components/schemas/MissionActionCreate"
        waitDurationSeconds: { type: number }
        waitCondition: { type: string }
        description: { type: string }

    MissionActionCreate:
      type: object
      required: [actionType, blockingType]
      properties:
        actionType: { type: string }
        parameters:
          type: object
          additionalProperties: { type: string }
        blockingType:
          type: string
          enum: [none, soft, hard]

    MissionResponse:
      type: object
      properties:
        id: { type: string, format: uuid }
        name: { type: string }
        description: { type: string }
        status:
          type: string
          enum: [draft, queued, assigned, in_progress, paused, completed, failed, cancelled, aborting]
        priority: { type: string }
        assignedRobotId: { type: string, format: uuid, nullable: true }
        mapId: { type: string, format: uuid }
        roadmapId: { type: string, format: uuid }
        steps:
          type: array
          items:
            $ref: "#/components/schemas/MissionStepResponse"
        currentStepIndex: { type: integer }
        progress: { type: number }
        plannedPath:
          type: array
          items: { type: string, format: uuid }
        createdAt: { type: string, format: date-time }
        startedAt: { type: string, format: date-time, nullable: true }
        completedAt: { type: string, format: date-time, nullable: true }
        estimatedDuration: { type: number }
        actualDuration: { type: number }
        totalDistance: { type: number }
        failureReason: { type: string, nullable: true }
        retryCount: { type: integer }
        maxRetries: { type: integer }

    MissionStepResponse:
      type: object
      properties:
        stepId: { type: string }
        sequence: { type: integer }
        stepType: { type: string }
        status:
          type: string
          enum: [pending, in_progress, completed, failed, skipped]
        targetNodeId: { type: string, format: uuid, nullable: true }
        actions:
          type: array
          items:
            type: object
            properties:
              actionId: { type: string }
              actionType: { type: string }
              status: { type: string }
              parameters: { type: object }
              blockingType: { type: string }
        waitDurationSeconds: { type: number, nullable: true }
        description: { type: string }
        startedAt: { type: string, format: date-time, nullable: true }
        completedAt: { type: string, format: date-time, nullable: true }
        failureReason: { type: string, nullable: true }

    # --- 시뮬레이션 ---
    SimSessionCreate:
      type: object
      properties:
        name: { type: string }
        config:
          $ref: "#/components/schemas/SimConfig"

    SimConfig:
      type: object
      properties:
        timeStep: { type: number, default: 0.001 }
        realTimeFactor: { type: number, default: 1.0 }
        gravity:
          $ref: "#/components/schemas/Vector3"
        physicsEngine:
          type: string
          enum: [bullet, ode, simple_2d]
          default: bullet
        maxRobots: { type: integer, default: 100 }
        enableCollision: { type: boolean, default: true }
        enableSensors: { type: boolean, default: true }
        telemetryRateHz: { type: integer, default: 10 }

    SimSessionResponse:
      type: object
      properties:
        id: { type: string, format: uuid }
        name: { type: string }
        state:
          type: string
          enum: [initializing, ready, running, paused, stopped, error]
        config:
          $ref: "#/components/schemas/SimConfig"
        simTime: { type: number }
        wallTime: { type: number }
        robotCount: { type: integer }
        loadedMapId: { type: string, format: uuid, nullable: true }
        createdAt: { type: string, format: date-time }

    # --- 에셋 ---
    AssetResponse:
      type: object
      properties:
        id: { type: string, format: uuid }
        name: { type: string }
        description: { type: string }
        assetType:
          type: string
          enum: [robot_model, environment_model, texture, material, mesh, plugin, icon, thumbnail]
        originalFormat: { type: string }
        status:
          type: string
          enum: [uploading, processing, ready, error, archived]
        originalFilename: { type: string }
        fileSizeBytes: { type: integer }
        convertedAssets:
          type: array
          items:
            type: object
            properties:
              format: { type: string }
              fileSizeBytes: { type: integer }
              downloadUrl: { type: string }
        robotMetadata:
          type: object
          nullable: true
          properties:
            manufacturer: { type: string }
            modelName: { type: string }
            kinematicsType: { type: string }
            maxSpeed: { type: number }
            maxPayload: { type: number }
            weight: { type: number }
        tags: { type: array, items: { type: string } }
        thumbnailUrl: { type: string, nullable: true }
        createdAt: { type: string, format: date-time }
        updatedAt: { type: string, format: date-time }

    # --- 맵 ---
    MapResponse:
      type: object
      properties:
        id: { type: string, format: uuid }
        name: { type: string }
        description: { type: string }
        status:
          type: string
          enum: [uploading, processing, ready, error, archived]
        sourceFormat: { type: string }
        boundingBox:
          type: object
          properties:
            min: { $ref: "#/components/schemas/Vector3" }
            max: { $ref: "#/components/schemas/Vector3" }
        pointCount: { type: integer }
        resolution: { type: number }
        potree:
          type: object
          nullable: true
          properties:
            baseUrl: { type: string }
            hierarchyDepth: { type: integer }
            tileCount: { type: integer }
        roadmapIds:
          type: array
          items: { type: string, format: uuid }
        regionIds:
          type: array
          items: { type: string, format: uuid }
        groundPlaneZ: { type: number }
        processingProgress: { type: number }
        tags: { type: array, items: { type: string } }
        createdAt: { type: string, format: date-time }
        updatedAt: { type: string, format: date-time }

    # --- 플러그인 ---
    PluginResponse:
      type: object
      properties:
        id: { type: string, format: uuid }
        name: { type: string }
        description: { type: string }
        version: { type: string }
        status:
          type: string
          enum: [active, inactive, error]
        events:
          type: array
          items: { type: string }
        permissions:
          type: array
          items: { type: string }
        createdAt: { type: string, format: date-time }

paths:
  # ==================== 인증 ====================
  /auth/login:
    post:
      tags: [Auth]
      summary: 로그인
      security: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/LoginRequest"
      responses:
        "200":
          description: 로그인 성공
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/LoginResponse"
        "401":
          description: 인증 실패
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ErrorResponse"

  /auth/refresh:
    post:
      tags: [Auth]
      summary: 토큰 갱신
      security: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/RefreshRequest"
      responses:
        "200":
          description: 갱신 성공
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/LoginResponse"

  /auth/me:
    get:
      tags: [Auth]
      summary: 현재 사용자 정보
      responses:
        "200":
          description: 사용자 정보
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/User"

  # ==================== 로봇 ====================
  /robots:
    get:
      tags: [Robots]
      summary: 로봇 목록 조회
      parameters:
        - name: page
          in: query
          schema: { type: integer, default: 1 }
        - name: pageSize
          in: query
          schema: { type: integer, default: 20 }
        - name: status
          in: query
          schema: { type: string, enum: [online, offline] }
        - name: mapId
          in: query
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 로봇 목록
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: "#/components/schemas/Robot"
                  pagination:
                    $ref: "#/components/schemas/Pagination"

    post:
      tags: [Robots]
      summary: 로봇 등록
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/CreateRobotRequest"
      responses:
        "201":
          description: 로봇 생성 성공
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Robot"

  /robots/{robotId}:
    get:
      tags: [Robots]
      summary: 로봇 상세 조회
      parameters:
        - name: robotId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 로봇 정보
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Robot"
        "404":
          description: 로봇을 찾을 수 없음

    patch:
      tags: [Robots]
      summary: 로봇 정보 수정
      parameters:
        - name: robotId
          in: path
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                name: { type: string }
                mapId: { type: string, format: uuid }
      responses:
        "200":
          description: 수정 성공
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/Robot"

    delete:
      tags: [Robots]
      summary: 로봇 삭제
      parameters:
        - name: robotId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 삭제 성공

  /robots/{robotId}/remote-control:
    post:
      tags: [Robots]
      summary: 원격 제어 명령 전송
      parameters:
        - name: robotId
          in: path
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [command]
              properties:
                command:
                  type: string
                  enum: [velocity, stop, emergency_stop, release_emergency_stop]
                velocity:
                  $ref: "#/components/schemas/Twist"
      responses:
        "200":
          description: 명령 전송 성공

  # ==================== 미션 ====================
  /missions:
    get:
      tags: [Missions]
      summary: 미션 목록 조회
      parameters:
        - name: page
          in: query
          schema: { type: integer, default: 1 }
        - name: pageSize
          in: query
          schema: { type: integer, default: 20 }
        - name: status
          in: query
          schema: { type: string }
        - name: robotId
          in: query
          schema: { type: string, format: uuid }
        - name: priority
          in: query
          schema: { type: string }
      responses:
        "200":
          description: 미션 목록
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: "#/components/schemas/MissionResponse"
                  pagination:
                    $ref: "#/components/schemas/Pagination"

    post:
      tags: [Missions]
      summary: 미션 생성
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/MissionCreate"
      responses:
        "201":
          description: 미션 생성 성공
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/MissionResponse"

  /missions/{missionId}:
    get:
      tags: [Missions]
      summary: 미션 상세 조회
      parameters:
        - name: missionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 미션 정보
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/MissionResponse"

    delete:
      tags: [Missions]
      summary: 미션 삭제
      parameters:
        - name: missionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 삭제 성공

  /missions/{missionId}/start:
    post:
      tags: [Missions]
      summary: 미션 시작
      parameters:
        - name: missionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 미션 시작 성공
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/MissionResponse"

  /missions/{missionId}/pause:
    post:
      tags: [Missions]
      summary: 미션 일시 정지
      parameters:
        - name: missionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 일시 정지 성공

  /missions/{missionId}/resume:
    post:
      tags: [Missions]
      summary: 미션 재개
      parameters:
        - name: missionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 재개 성공

  /missions/{missionId}/cancel:
    post:
      tags: [Missions]
      summary: 미션 취소
      parameters:
        - name: missionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 취소 성공

  # ==================== 시뮬레이션 ====================
  /simulations:
    get:
      tags: [Simulation]
      summary: 시뮬레이션 세션 목록
      responses:
        "200":
          description: 세션 목록
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: "#/components/schemas/SimSessionResponse"

    post:
      tags: [Simulation]
      summary: 시뮬레이션 세션 생성
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/SimSessionCreate"
      responses:
        "201":
          description: 세션 생성 성공
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/SimSessionResponse"

  /simulations/{sessionId}:
    get:
      tags: [Simulation]
      summary: 세션 상세 조회
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 세션 정보
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/SimSessionResponse"

    delete:
      tags: [Simulation]
      summary: 세션 삭제
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 삭제 성공

  /simulations/{sessionId}/start:
    post:
      tags: [Simulation]
      summary: 시뮬레이션 시작
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 시작 성공

  /simulations/{sessionId}/pause:
    post:
      tags: [Simulation]
      summary: 시뮬레이션 일시 정지
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 일시 정지 성공

  /simulations/{sessionId}/resume:
    post:
      tags: [Simulation]
      summary: 시뮬레이션 재개
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 재개 성공

  /simulations/{sessionId}/stop:
    post:
      tags: [Simulation]
      summary: 시뮬레이션 정지
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 정지 성공

  /simulations/{sessionId}/reset:
    post:
      tags: [Simulation]
      summary: 시뮬레이션 리셋
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 리셋 성공

  /simulations/{sessionId}/time-scale:
    put:
      tags: [Simulation]
      summary: 시간 배율 변경
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [realTimeFactor]
              properties:
                realTimeFactor: { type: number, minimum: 0.1, maximum: 100.0 }
      responses:
        "200":
          description: 변경 성공

  /simulations/{sessionId}/load-map:
    post:
      tags: [Simulation]
      summary: 시뮬레이션에 맵 로드
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [mapId]
              properties:
                mapId: { type: string, format: uuid }
      responses:
        "200":
          description: 맵 로드 성공

  /simulations/{sessionId}/robots:
    post:
      tags: [Simulation]
      summary: 시뮬레이션에 로봇 스폰
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [assetId, initialPose]
              properties:
                robotId: { type: string, format: uuid }
                assetId: { type: string, format: uuid }
                initialPose:
                  $ref: "#/components/schemas/Pose3D"
                robotName: { type: string }
      responses:
        "201":
          description: 스폰 성공

  /simulations/{sessionId}/robots/{robotId}:
    delete:
      tags: [Simulation]
      summary: 시뮬레이션에서 로봇 제거
      parameters:
        - name: sessionId
          in: path
          required: true
          schema: { type: string, format: uuid }
        - name: robotId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 제거 성공

  # ==================== 에셋 ====================
  /assets:
    get:
      tags: [Assets]
      summary: 에셋 목록 조회
      parameters:
        - name: page
          in: query
          schema: { type: integer, default: 1 }
        - name: pageSize
          in: query
          schema: { type: integer, default: 20 }
        - name: type
          in: query
          schema: { type: string, enum: [robot_model, environment_model, texture, material, mesh, plugin] }
        - name: status
          in: query
          schema: { type: string, enum: [uploading, processing, ready, error, archived] }
        - name: search
          in: query
          schema: { type: string }
      responses:
        "200":
          description: 에셋 목록
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: "#/components/schemas/AssetResponse"
                  pagination:
                    $ref: "#/components/schemas/Pagination"

    post:
      tags: [Assets]
      summary: 에셋 업로드 시작
      requestBody:
        required: true
        content:
          multipart/form-data:
            schema:
              type: object
              required: [file, name, assetType]
              properties:
                file:
                  type: string
                  format: binary
                name: { type: string }
                description: { type: string }
                assetType: { type: string }
                tags: { type: string, description: "쉼표 구분 태그" }
      responses:
        "201":
          description: 업로드 및 처리 시작
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/AssetResponse"

  /assets/{assetId}:
    get:
      tags: [Assets]
      summary: 에셋 상세 조회
      parameters:
        - name: assetId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 에셋 정보
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/AssetResponse"

    patch:
      tags: [Assets]
      summary: 에셋 정보 수정
      parameters:
        - name: assetId
          in: path
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                name: { type: string }
                description: { type: string }
                tags: { type: array, items: { type: string } }
      responses:
        "200":
          description: 수정 성공

    delete:
      tags: [Assets]
      summary: 에셋 삭제
      parameters:
        - name: assetId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 삭제 성공

  /assets/{assetId}/download:
    get:
      tags: [Assets]
      summary: 에셋 다운로드 URL 발급
      parameters:
        - name: assetId
          in: path
          required: true
          schema: { type: string, format: uuid }
        - name: format
          in: query
          schema: { type: string, enum: [urdf, sdf, gltf, glb, obj, stl] }
          description: "원하는 형식 (미지정 시 원본)"
      responses:
        "200":
          description: 다운로드 URL
          content:
            application/json:
              schema:
                type: object
                properties:
                  downloadUrl: { type: string, format: uri }
                  expiresAt: { type: string, format: date-time }
                  format: { type: string }
                  fileSizeBytes: { type: integer }

  # ==================== 맵 ====================
  /maps:
    get:
      tags: [Maps]
      summary: 맵 목록 조회
      parameters:
        - name: page
          in: query
          schema: { type: integer, default: 1 }
        - name: pageSize
          in: query
          schema: { type: integer, default: 20 }
        - name: status
          in: query
          schema: { type: string }
        - name: search
          in: query
          schema: { type: string }
      responses:
        "200":
          description: 맵 목록
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: "#/components/schemas/MapResponse"
                  pagination:
                    $ref: "#/components/schemas/Pagination"

    post:
      tags: [Maps]
      summary: 맵 업로드 시작
      requestBody:
        required: true
        content:
          multipart/form-data:
            schema:
              type: object
              required: [file, name]
              properties:
                file:
                  type: string
                  format: binary
                name: { type: string }
                description: { type: string }
                coordinateSystem: { type: string, default: "ENU" }
                tags: { type: string }
      responses:
        "201":
          description: 맵 업로드 및 처리 시작
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/MapResponse"

  /maps/{mapId}:
    get:
      tags: [Maps]
      summary: 맵 상세 조회
      parameters:
        - name: mapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 맵 정보
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/MapResponse"

    patch:
      tags: [Maps]
      summary: 맵 정보 수정
      parameters:
        - name: mapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                name: { type: string }
                description: { type: string }
                tags: { type: array, items: { type: string } }
                groundPlaneZ: { type: number }
      responses:
        "200":
          description: 수정 성공

    delete:
      tags: [Maps]
      summary: 맵 삭제
      parameters:
        - name: mapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 삭제 성공

  /maps/{mapId}/tiles/{nodeName}:
    get:
      tags: [Maps]
      summary: Potree 타일 조회
      parameters:
        - name: mapId
          in: path
          required: true
          schema: { type: string, format: uuid }
        - name: nodeName
          in: path
          required: true
          schema: { type: string }
      responses:
        "200":
          description: 타일 데이터
          content:
            application/octet-stream:
              schema:
                type: string
                format: binary

  /maps/{mapId}/potree-metadata:
    get:
      tags: [Maps]
      summary: Potree 메타데이터 조회
      parameters:
        - name: mapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: Potree 메타데이터
          content:
            application/json:
              schema:
                type: object
                properties:
                  baseUrl: { type: string }
                  hierarchyDepth: { type: integer }
                  tileCount: { type: integer }
                  spacing: { type: number }
                  boundingBox:
                    type: object
                    properties:
                      min: { $ref: "#/components/schemas/Vector3" }
                      max: { $ref: "#/components/schemas/Vector3" }
                  availableAttributes:
                    type: array
                    items: { type: string }

  /maps/{mapId}/roadmaps:
    get:
      tags: [Maps, Roadmaps]
      summary: 맵의 로드맵 목록 조회
      parameters:
        - name: mapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 로드맵 목록

    post:
      tags: [Maps, Roadmaps]
      summary: 로드맵 생성
      parameters:
        - name: mapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [name]
              properties:
                name: { type: string }
                description: { type: string }
                nodes:
                  type: array
                  items:
                    type: object
                    properties:
                      id: { type: string, format: uuid }
                      pose:
                        type: object
                        properties:
                          x: { type: number }
                          y: { type: number }
                          theta: { type: number }
                      nodeType: { type: string }
                      name: { type: string }
                edges:
                  type: array
                  items:
                    type: object
                    properties:
                      id: { type: string, format: uuid }
                      startNodeId: { type: string, format: uuid }
                      endNodeId: { type: string, format: uuid }
                      edgeType: { type: string }
                      bidirectional: { type: boolean, default: true }
                      maxSpeed: { type: number }
      responses:
        "201":
          description: 로드맵 생성 성공

  /roadmaps/{roadmapId}:
    get:
      tags: [Roadmaps]
      summary: 로드맵 상세 조회
      parameters:
        - name: roadmapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 로드맵 정보

    put:
      tags: [Roadmaps]
      summary: 로드맵 전체 업데이트
      parameters:
        - name: roadmapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 업데이트 성공

    delete:
      tags: [Roadmaps]
      summary: 로드맵 삭제
      parameters:
        - name: roadmapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 삭제 성공

  /roadmaps/{roadmapId}/nodes:
    post:
      tags: [Roadmaps]
      summary: 노드 추가
      parameters:
        - name: roadmapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "201":
          description: 노드 추가 성공

  /roadmaps/{roadmapId}/nodes/{nodeId}:
    put:
      tags: [Roadmaps]
      summary: 노드 업데이트
      parameters:
        - name: roadmapId
          in: path
          required: true
          schema: { type: string, format: uuid }
        - name: nodeId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 업데이트 성공

    delete:
      tags: [Roadmaps]
      summary: 노드 삭제
      parameters:
        - name: roadmapId
          in: path
          required: true
          schema: { type: string, format: uuid }
        - name: nodeId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 삭제 성공

  /roadmaps/{roadmapId}/edges:
    post:
      tags: [Roadmaps]
      summary: 엣지 추가
      parameters:
        - name: roadmapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "201":
          description: 엣지 추가 성공

  /roadmaps/{roadmapId}/edges/{edgeId}:
    delete:
      tags: [Roadmaps]
      summary: 엣지 삭제
      parameters:
        - name: roadmapId
          in: path
          required: true
          schema: { type: string, format: uuid }
        - name: edgeId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 삭제 성공

  /roadmaps/{roadmapId}/pathfinding:
    post:
      tags: [Roadmaps]
      summary: 경로 탐색
      parameters:
        - name: roadmapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [startNodeId, goalNodeId]
              properties:
                startNodeId: { type: string, format: uuid }
                goalNodeId: { type: string, format: uuid }
                algorithm:
                  type: string
                  enum: [dijkstra, a_star, theta_star]
                  default: a_star
                robotId: { type: string, format: uuid }
      responses:
        "200":
          description: 경로 탐색 결과
          content:
            application/json:
              schema:
                type: object
                properties:
                  found: { type: boolean }
                  pathNodeIds:
                    type: array
                    items: { type: string, format: uuid }
                  pathEdgeIds:
                    type: array
                    items: { type: string, format: uuid }
                  totalDistance: { type: number }
                  estimatedTime: { type: number }

  /maps/{mapId}/regions:
    get:
      tags: [Maps, Regions]
      summary: 영역 목록 조회
      parameters:
        - name: mapId
          in: path
          required: true
          schema: { type: string, format: uuid }
        - name: type
          in: query
          schema: { type: string }
      responses:
        "200":
          description: 영역 목록

    post:
      tags: [Maps, Regions]
      summary: 영역 생성
      parameters:
        - name: mapId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "201":
          description: 영역 생성 성공

  /regions/{regionId}:
    get:
      tags: [Regions]
      summary: 영역 상세 조회
      parameters:
        - name: regionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 영역 정보

    put:
      tags: [Regions]
      summary: 영역 업데이트
      parameters:
        - name: regionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 업데이트 성공

    delete:
      tags: [Regions]
      summary: 영역 삭제
      parameters:
        - name: regionId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 삭제 성공

  # ==================== 플러그인 ====================
  /plugins:
    get:
      tags: [Plugins]
      summary: 플러그인 목록 조회
      responses:
        "200":
          description: 플러그인 목록

    post:
      tags: [Plugins]
      summary: 플러그인 업로드
      requestBody:
        required: true
        content:
          multipart/form-data:
            schema:
              type: object
              required: [file, name]
              properties:
                file:
                  type: string
                  format: binary
                name: { type: string }
                description: { type: string }
                events:
                  type: string
                  description: "구독할 이벤트 (쉼표 구분)"
      responses:
        "201":
          description: 플러그인 업로드 성공

  /plugins/{pluginId}:
    get:
      tags: [Plugins]
      summary: 플러그인 상세 조회
      parameters:
        - name: pluginId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 플러그인 정보

    delete:
      tags: [Plugins]
      summary: 플러그인 삭제
      parameters:
        - name: pluginId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "204":
          description: 삭제 성공

  /plugins/{pluginId}/activate:
    post:
      tags: [Plugins]
      summary: 플러그인 활성화
      parameters:
        - name: pluginId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 활성화 성공

  /plugins/{pluginId}/deactivate:
    post:
      tags: [Plugins]
      summary: 플러그인 비활성화
      parameters:
        - name: pluginId
          in: path
          required: true
          schema: { type: string, format: uuid }
      responses:
        "200":
          description: 비활성화 성공

  # ==================== 대시보드 ====================
  /dashboard/stats:
    get:
      tags: [Dashboard]
      summary: 대시보드 통계
      responses:
        "200":
          description: 통계 정보
          content:
            application/json:
              schema:
                type: object
                properties:
                  totalRobots: { type: integer }
                  onlineRobots: { type: integer }
                  activeMissions: { type: integer }
                  completedMissionsToday: { type: integer }
                  failedMissionsToday: { type: integer }
                  averageBattery: { type: number }
                  totalDistanceToday: { type: number }
                  activeSimulations: { type: integer }
```

---

### 2.4 WebSocket 메시지 스키마

WebSocket 연결 URL: `ws(s)://{host}:{port}/api/v1/ws`

모든 WebSocket 메시지는 다음 엔벨로프 형식을 따른다:

```json
{
  "version": 1,
  "type": "<message_type>",
  "topic": "<topic_pattern>",
  "timestamp": 1711000000000,
  "payload": { ... }
}
```

**연결 및 구독 프로토콜:**

```json
// 클라이언트 → 서버: 구독 요청
{
  "version": 1,
  "type": "subscribe",
  "topic": "telemetry:*",
  "timestamp": 1711000000000,
  "payload": {
    "robotIds": ["uuid1", "uuid2"],
    "rateHz": 10
  }
}

// 서버 → 클라이언트: 구독 확인
{
  "version": 1,
  "type": "subscribe_ack",
  "topic": "telemetry:*",
  "timestamp": 1711000000001,
  "payload": {
    "subscriptionId": "uuid",
    "status": "ok"
  }
}

// 클라이언트 → 서버: 구독 해제
{
  "version": 1,
  "type": "unsubscribe",
  "topic": "telemetry:*",
  "timestamp": 1711000000002,
  "payload": {
    "subscriptionId": "uuid"
  }
}
```

#### 2.4.1 텔레메트리 메시지 (`telemetry`)

토픽 패턴: `telemetry:{robot_id}` 또는 `telemetry:*` (전체)

```json
{
  "version": 1,
  "type": "telemetry",
  "topic": "telemetry:550e8400-e29b-41d4-a716-446655440000",
  "timestamp": 1711000000000,
  "payload": {
    "robotId": "550e8400-e29b-41d4-a716-446655440000",
    "sequenceNumber": 42,
    "pose": {
      "position": { "x": 10.5, "y": 20.3, "z": 0.15 },
      "orientation": { "x": 0.0, "y": 0.0, "z": 0.383, "w": 0.924 }
    },
    "velocity": {
      "linear": { "x": 0.5, "y": 0.0, "z": 0.0 },
      "angular": { "x": 0.0, "y": 0.0, "z": 0.1 }
    },
    "battery": 85.5,
    "driving": true,
    "errors": []
  }
}
```

#### 2.4.2 미션 업데이트 메시지 (`mission_update`)

토픽 패턴: `mission:{mission_id}` 또는 `mission:*`

```json
{
  "version": 1,
  "type": "mission_update",
  "topic": "mission:660e8400-e29b-41d4-a716-446655440000",
  "timestamp": 1711000000000,
  "payload": {
    "missionId": "660e8400-e29b-41d4-a716-446655440000",
    "status": "in_progress",
    "currentStepIndex": 2,
    "progress": 0.45,
    "assignedRobotId": "550e8400-e29b-41d4-a716-446655440000",
    "eventDescription": "Step 2: 노드 N-005로 이동 중"
  }
}
```

#### 2.4.3 알림 메시지 (`alert`)

토픽 패턴: `alert:*` 또는 `alert:{severity}` (severity: info, warning, error, critical)

```json
{
  "version": 1,
  "type": "alert",
  "topic": "alert:error",
  "timestamp": 1711000000000,
  "payload": {
    "alertId": "770e8400-e29b-41d4-a716-446655440000",
    "severity": "error",
    "source": "sim-engine",
    "title": "로봇 충돌 감지",
    "message": "로봇 R-001과 장애물 간 충돌 발생 (충돌 힘: 15.3N)",
    "robotId": "550e8400-e29b-41d4-a716-446655440000",
    "relatedEntityId": "obstacle_01",
    "relatedEntityType": "obstacle",
    "acknowledged": false,
    "data": {
      "contactPoint": { "x": 12.5, "y": 8.3, "z": 0.2 },
      "impactForce": 15.3
    }
  }
}
```

#### 2.4.4 맵 업데이트 메시지 (`map_update`)

토픽 패턴: `map:{map_id}`

```json
{
  "version": 1,
  "type": "map_update",
  "topic": "map:880e8400-e29b-41d4-a716-446655440000",
  "timestamp": 1711000000000,
  "payload": {
    "mapId": "880e8400-e29b-41d4-a716-446655440000",
    "updateType": "processing_progress",
    "data": {
      "status": "processing",
      "progress": 0.75,
      "pointsProcessed": 7500000,
      "totalPoints": 10000000,
      "tilesGenerated": 150
    }
  }
}
```

`updateType` 종류:
- `processing_progress`: Potree 타일링 진행 상황
- `status_changed`: 맵 상태 변경
- `roadmap_updated`: 로드맵 변경
- `region_updated`: 영역 변경

#### 2.4.5 원격 제어 명령 (`remote_control_command`)

토픽 패턴: `control:{robot_id}` (클라이언트 → 서버)

```json
{
  "version": 1,
  "type": "remote_control_command",
  "topic": "control:550e8400-e29b-41d4-a716-446655440000",
  "timestamp": 1711000000000,
  "payload": {
    "robotId": "550e8400-e29b-41d4-a716-446655440000",
    "command": "velocity",
    "velocity": {
      "linear": { "x": 0.3, "y": 0.0, "z": 0.0 },
      "angular": { "x": 0.0, "y": 0.0, "z": 0.2 }
    }
  }
}
```

`command` 종류:
- `velocity`: 속도 명령 (velocity 필드 필수)
- `stop`: 즉시 정지
- `emergency_stop`: 비상 정지
- `release_emergency_stop`: 비상 정지 해제

#### 2.4.6 시뮬레이션 이벤트 (`sim_event`)

토픽 패턴: `sim:{session_id}`

```json
{
  "version": 1,
  "type": "sim_event",
  "topic": "sim:990e8400-e29b-41d4-a716-446655440000",
  "timestamp": 1711000000000,
  "payload": {
    "sessionId": "990e8400-e29b-41d4-a716-446655440000",
    "simTime": 45.678,
    "eventType": "collision",
    "data": {
      "robotA": "550e8400-e29b-41d4-a716-446655440000",
      "objectB": "wall_segment_12",
      "contactPoint": { "x": 5.2, "y": 3.1, "z": 0.1 },
      "impactForce": 8.7
    }
  }
}
```

`eventType` 종류: `robot_spawned`, `robot_removed`, `collision`, `fault`, `state_changed`, `navigation_completed`

---

### 2.5 MQTT / VDA5050 메시지 스키마

Backend가 MQTT 브로커를 통해 실제 로봇과 통신할 때 사용하는 VDA5050 v2.0 메시지 형식이다.

**MQTT 토픽 규약:**

```
{manufacturer}/{serialNumber}/v2/{topic}
```

예: `amr_co/robot_001/v2/order`, `amr_co/robot_001/v2/state`

**토픽 방향:**

| 토픽 | 방향 | 설명 |
|------|------|------|
| `order` | Backend → Robot | 주행 명령 |
| `instantActions` | Backend → Robot | 즉시 실행 액션 |
| `state` | Robot → Backend | 로봇 상태 |
| `visualization` | Robot → Backend | 시각화 정보 |
| `connection` | Robot → Backend | 연결 상태 |

#### 2.5.1 Order 메시지 (Backend → Robot)

```json
{
  "headerId": 1,
  "timestamp": "2026-03-21T10:00:00.000Z",
  "version": "2.0.0",
  "manufacturer": "amr_co",
  "serialNumber": "robot_001",
  "orderId": "order-uuid-001",
  "orderUpdateId": 0,
  "zoneSetId": "zone-set-uuid-001",
  "nodes": [
    {
      "nodeId": "node-uuid-001",
      "sequenceId": 0,
      "released": true,
      "nodePosition": {
        "x": 10.5,
        "y": 20.3,
        "theta": 1.57,
        "allowedDeviationXY": 0.5,
        "allowedDeviationTheta": 0.1,
        "mapId": "map-uuid-001",
        "mapDescription": "warehouse-floor-1"
      },
      "actions": [
        {
          "actionId": "action-uuid-001",
          "actionType": "pick",
          "actionDescription": "픽업 액션",
          "blockingType": "HARD",
          "actionParameters": [
            {
              "key": "stationType",
              "value": "conveyor"
            },
            {
              "key": "loadType",
              "value": "EPAL"
            }
          ]
        }
      ]
    },
    {
      "nodeId": "node-uuid-002",
      "sequenceId": 2,
      "released": true,
      "nodePosition": {
        "x": 15.0,
        "y": 25.0,
        "theta": 0.0,
        "allowedDeviationXY": 0.5,
        "allowedDeviationTheta": 0.1,
        "mapId": "map-uuid-001",
        "mapDescription": "warehouse-floor-1"
      },
      "actions": []
    }
  ],
  "edges": [
    {
      "edgeId": "edge-uuid-001",
      "sequenceId": 1,
      "released": true,
      "startNodeId": "node-uuid-001",
      "endNodeId": "node-uuid-002",
      "maxSpeed": 1.5,
      "maxHeight": 2.0,
      "minHeight": 0.0,
      "orientation": 0.785,
      "orientationType": "GLOBAL",
      "direction": "straight",
      "rotationAllowed": true,
      "maxRotationSpeed": 1.0,
      "length": 7.07,
      "actions": []
    }
  ]
}
```

#### 2.5.2 InstantAction 메시지 (Backend → Robot)

```json
{
  "headerId": 2,
  "timestamp": "2026-03-21T10:00:01.000Z",
  "version": "2.0.0",
  "manufacturer": "amr_co",
  "serialNumber": "robot_001",
  "instantActions": [
    {
      "actionId": "instant-action-uuid-001",
      "actionType": "cancelOrder",
      "actionDescription": "현재 주행 명령 취소",
      "blockingType": "HARD",
      "actionParameters": []
    }
  ]
}
```

일반적인 InstantAction 유형:
- `cancelOrder`: 현재 주행 명령 취소
- `stopPause`: 일시 정지
- `startPause`: 일시 정지 해제
- `stateRequest`: 상태 즉시 전송 요청
- `factsheetRequest`: 스펙 시트 요청

#### 2.5.3 State 메시지 (Robot → Backend)

```json
{
  "headerId": 100,
  "timestamp": "2026-03-21T10:00:00.500Z",
  "version": "2.0.0",
  "manufacturer": "amr_co",
  "serialNumber": "robot_001",
  "orderId": "order-uuid-001",
  "orderUpdateId": 0,
  "zoneSetId": "zone-set-uuid-001",
  "lastNodeId": "node-uuid-001",
  "lastNodeSequenceId": 0,
  "driving": true,
  "paused": false,
  "newBaseRequest": false,
  "distanceSinceLastNode": 3.5,
  "operatingMode": "AUTOMATIC",
  "nodeStates": [
    {
      "nodeId": "node-uuid-002",
      "sequenceId": 2,
      "released": true,
      "nodePosition": {
        "x": 15.0,
        "y": 25.0,
        "theta": 0.0,
        "mapId": "map-uuid-001"
      }
    }
  ],
  "edgeStates": [
    {
      "edgeId": "edge-uuid-001",
      "sequenceId": 1,
      "released": true
    }
  ],
  "agvPosition": {
    "x": 12.5,
    "y": 22.1,
    "theta": 0.45,
    "mapId": "map-uuid-001",
    "mapDescription": "warehouse-floor-1",
    "positionInitialized": true,
    "localizationScore": 0.95,
    "deviationRange": 0.1
  },
  "velocity": {
    "vx": 0.5,
    "vy": 0.0,
    "omega": 0.1
  },
  "loads": [],
  "actionStates": [
    {
      "actionId": "action-uuid-001",
      "actionType": "pick",
      "actionDescription": "픽업 액션",
      "actionStatus": "FINISHED",
      "resultDescription": "성공적으로 픽업 완료"
    }
  ],
  "batteryState": {
    "batteryCharge": 85.5,
    "batteryVoltage": 48.2,
    "batteryHealth": 95.0,
    "charging": false,
    "reach": 12000
  },
  "errors": [],
  "information": [
    {
      "infoType": "temperature",
      "infoReferences": [],
      "infoDescription": "모터 온도 정상",
      "infoLevel": "INFO"
    }
  ],
  "safetyState": {
    "eStop": "NONE",
    "fieldViolation": false
  }
}
```

#### 2.5.4 Visualization 메시지 (Robot → Backend)

```json
{
  "headerId": 101,
  "timestamp": "2026-03-21T10:00:00.500Z",
  "version": "2.0.0",
  "manufacturer": "amr_co",
  "serialNumber": "robot_001",
  "agvPosition": {
    "x": 12.5,
    "y": 22.1,
    "theta": 0.45,
    "mapId": "map-uuid-001",
    "positionInitialized": true,
    "localizationScore": 0.95,
    "deviationRange": 0.1
  },
  "velocity": {
    "vx": 0.5,
    "vy": 0.0,
    "omega": 0.1
  }
}
```

> **참고**: Visualization 메시지는 State 메시지의 경량화 버전으로, 높은 빈도(10Hz 이상)로 전송된다. Frontend의 실시간 위치 표시에 사용된다.

#### 2.5.5 Connection 메시지 (Robot → Backend)

```json
{
  "headerId": 1,
  "timestamp": "2026-03-21T10:00:00.000Z",
  "version": "2.0.0",
  "manufacturer": "amr_co",
  "serialNumber": "robot_001",
  "connectionState": "ONLINE"
}
```

`connectionState` 값: `ONLINE`, `OFFLINE`, `CONNECTIONBROKEN`

> **참고**: MQTT Last Will and Testament(LWT)를 사용하여 연결 끊김 시 자동으로 `CONNECTIONBROKEN` 메시지가 발행된다.

---

## 3. 팀 간 의존성 매트릭스

### 3.1 통신 방식 매트릭스

아래 표는 각 팀 간 통신 방식과 역할(클라이언트/서버)을 나타낸다.

```
              │ Frontend (T1) │ Backend (T2)  │ Sim Engine (T3) │ Asset Mgr (T4) │ Map Mgr (T5) │
──────────────┼───────────────┼───────────────┼─────────────────┼────────────────┼──────────────│
Frontend (T1) │       -       │ REST/WS 클라이언트│        -        │       -        │      -       │
Backend (T2)  │ REST/WS 서버  │       -       │ gRPC 클라이언트   │ gRPC 클라이언트  │ gRPC 클라이언트 │
Sim Engine(T3)│       -       │ gRPC 서버     │        -        │ gRPC 클라이언트  │ gRPC 클라이언트 │
Asset Mgr(T4) │       -       │ gRPC 서버     │ gRPC 서버       │       -        │      -       │
Map Mgr (T5)  │       -       │ gRPC 서버     │ gRPC 서버       │       -        │      -       │
```

### 3.2 데이터 흐름 의존성

#### 의존성 방향도

```
Frontend ──REST/WS──→ Backend ──gRPC──→ Sim Engine
                         │                  │
                         │ gRPC             │ gRPC (에셋 로딩)
                         ▼                  ▼
                    Asset Manager ←──gRPC── Sim Engine

                    Backend ──gRPC──→ Map Manager
                                          ▲
                                          │ gRPC (맵 로딩)
                                     Sim Engine
```

#### 팀별 의존성 상세

| 팀 | 의존 대상 | 의존 내용 | 차단 위험도 |
|----|----------|----------|-----------|
| T1 Frontend | T2 Backend | REST API, WebSocket 스트림 | **높음** - Backend API 없이 UI 개발 불가 |
| T2 Backend | T3 Sim Engine | gRPC SimulationService | 중간 - Mock으로 대체 가능 |
| T2 Backend | T4 Asset Manager | gRPC AssetService | 중간 - Mock으로 대체 가능 |
| T2 Backend | T5 Map Manager | gRPC MapService | 중간 - Mock으로 대체 가능 |
| T3 Sim Engine | T4 Asset Manager | gRPC DownloadAssetStream (URDF 로딩) | **높음** - 에셋 없이 시뮬레이션 불가 |
| T3 Sim Engine | T5 Map Manager | gRPC StreamTiles (맵 로딩) | **높음** - 맵 없이 시뮬레이션 불가 |

### 3.3 Mock 서버 요구사항

차단 위험도를 낮추기 위해 각 팀은 Week 3까지 Mock 서버를 제공해야 한다.

| 팀 | Mock 제공 범위 | 포트 | 비고 |
|----|-------------|------|------|
| T2 Backend | REST API + WebSocket (정적 데이터) | 8080 | OpenAPI에서 자동 생성 (Prism 등) |
| T3 Sim Engine | gRPC SimulationService (에코 응답 + 가상 텔레메트리) | 50051 | 10Hz 가상 텔레메트리 스트림 |
| T4 Asset Manager | gRPC AssetService (샘플 URDF/glTF 제공) | 50052 | 기본 TurtleBot URDF 포함 |
| T5 Map Manager | gRPC MapService (샘플 맵/로드맵 제공) | 50053 | 10m x 10m 샘플 맵 포함 |

---

## 4. 개발 타이밍 & 동기화 포인트

### 4.1 Phase별 팀 간 동기화

전체 프로젝트는 4개 Phase로 나뉘며, 각 Phase에 명확한 동기화 포인트가 있다.

```
Week  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28
      ├──────────────┤├──────────────────────────────┤├──────────────────────┤├──────────────┤
       Phase 1 (기반)   Phase 2 (핵심 기능)              Phase 3 (고급 기능)    Phase 4 (안정화)
      P  F  M         ▲                    ▲                         ▲                  ▲
      r  r  o         │                    │                         │                  │
      o  e  c         Phase 1              Phase 2                   Phase 3            Final
      t  e  k         Integration          Integration               Integration       Integration
      o  z  s         Checkpoint           Checkpoint                Checkpoint         Review
         e
```

#### Proto Freeze (Week 2)

**목적**: 모든 팀이 proto 파일 정의에 합의한다.

**준비물**:
- 각 팀은 Week 1 동안 자신이 필요한 proto 메시지/서비스를 제안한다
- Week 2 시작 시 PR 리뷰 → 전 팀 승인 → merge

**완료 조건**:
- `proto/` 디렉토리의 모든 파일이 main 브랜치에 merge됨
- `buf lint` 통과
- 모든 팀 리드 승인

**이후 규칙**:
- Proto 파일 변경은 반드시 RFC 프로세스를 거쳐야 한다
- 하위 호환성을 깨는 변경(breaking change)은 전 팀 승인 필요
- `buf breaking` 체크 통과 필수

#### Mock Server Availability (Week 3)

**목적**: 각 팀이 Mock 서버를 제공하여 병렬 개발을 가능하게 한다.

**준비물**:
- 각 팀의 Mock 서버가 Docker 이미지로 제공됨
- Mock 데이터 시나리오 문서

**완료 조건**:
- 모든 Mock 서버가 `docker compose -f docker-compose.mocks.yml up`으로 기동 가능
- 기본 요청/응답 시나리오 테스트 통과

#### Phase 1 Integration Checkpoint (Week 6)

**목적**: 첫 번째 전체 스택 통합 데모.

**데모 시나리오**:
1. 시뮬레이션 세션 생성
2. 샘플 맵 로드
3. 로봇 스폰 (샘플 URDF)
4. 시뮬레이션 시작
5. Frontend에서 로봇 위치가 실시간 업데이트되는 것을 확인

**완료 조건**:
- 위 시나리오가 Docker Compose 환경에서 정상 동작
- 텔레메트리 지연 시간 < 500ms

#### Phase 2 Integration Checkpoint (Week 14)

**목적**: 핵심 기능 전체 통합 데모.

**데모 시나리오**:
1. 포인트 클라우드 업로드 → Potree 타일링 완료 → Frontend 렌더링
2. 로드맵 편집 (노드/엣지 추가) → 저장
3. 미션 생성 (경로 자동 탐색) → 로봇 배정
4. 시뮬레이션 실행 → 로봇이 경로를 따라 이동
5. Frontend 대시보드에서 미션 진행 상황 모니터링

**완료 조건**:
- 위 시나리오 전체 동작
- 텔레메트리 지연 시간 < 200ms
- 포인트 클라우드 10M 포인트 이하 정상 처리

#### Phase 3 Integration Checkpoint (Week 20)

**목적**: 고급 기능 및 실제 로봇 통합 데모.

**데모 시나리오**:
1. VDA5050 시뮬레이터를 통한 가상 실제 로봇 연결
2. 하이브리드 모드: 시뮬레이션 로봇 + 가상 실제 로봇 동시 운용
3. 다중 로봇 교통 관리 (5대 로봇, 교차 경로)
4. WASM 플러그인 로드 및 이벤트 처리

**완료 조건**:
- VDA5050 메시지 교환 정상 동작
- 교통 관리: 충돌 0건, 데드락 0건
- 플러그인 이벤트 수신 및 API 호출 정상

#### Phase 4 Final Integration (Week 28)

**목적**: 프로덕션 준비 완료 확인.

**체크리스트**:
- [ ] 전체 E2E 테스트 스위트 통과
- [ ] 성능 벤치마크 달성 (아래 5.2절 참조)
- [ ] 보안 감사 통과
- [ ] 문서화 완료
- [ ] 장애 복구 시나리오 테스트 통과
- [ ] 부하 테스트 (50대 로봇 동시 운용) 통과

### 4.2 Breaking Change Protocol

Proto 파일 또는 REST API에 하위 호환성을 깨는 변경이 필요한 경우 다음 프로세스를 따른다.

**1단계: RFC 작성**
- `docs/rfcs/RFC-{번호}-{제목}.md` 파일 생성
- 템플릿은 6.1절 참조

**2단계: 리뷰 요청**
- PR 생성, 모든 팀 리드를 리뷰어로 지정
- Slack `#integration` 채널에 공지

**3단계: 승인**
- 영향받는 모든 팀 리드가 48시간 이내에 승인
- 48시간 내 미응답 시 자동 승인으로 간주하지 **않음** (반드시 명시적 승인 필요)

**4단계: 적용**
- PR merge 후, 각 팀은 1주일 이내에 마이그레이션 완료
- CI에서 `buf breaking` 체크 통과 확인

**Phase별 제한**:
- Phase 1-2: Breaking change 허용 (RFC 프로세스 준수 필요)
- Phase 3: Breaking change 원칙적 금지 (긴급한 경우 전 팀 만장일치 필요)
- Phase 4: Breaking change 절대 금지

### 4.3 API 버전관리

#### Protobuf 버전관리

- 패키지 이름에 버전 포함: `amr.v1.common`, `amr.v1.simulation` 등
- 메이저 버전 변경 시 새 패키지 생성: `amr.v2.common`
- 필드 추가는 하위 호환 (기존 필드 번호 유지)
- 필드 삭제 시: `reserved` 키워드 사용, 필드 번호 재사용 금지

```protobuf
message Example {
  reserved 3, 5;
  reserved "old_field_name";
  string new_field = 1;
}
```

#### REST API 버전관리

- URL 접두사: `/api/v1/...`, `/api/v2/...`
- v1 엔드포인트는 v2 출시 후 최소 6개월 유지
- 응답 헤더에 deprecation 정보 포함:
  ```
  Deprecation: true
  Sunset: Sat, 21 Sep 2026 00:00:00 GMT
  Link: </api/v2/robots>; rel="successor-version"
  ```

#### WebSocket 메시지 버전관리

- 메시지 엔벨로프의 `version` 필드로 관리
- 서버는 클라이언트가 요청한 버전의 메시지를 전송
- 버전 미지정 시 최신 버전 사용

---

## 5. 통합 테스트 방법

### 5.1 로컬 통합 테스트 (Docker Compose)

```yaml
# docker-compose.integration.yml
version: "3.9"

services:
  postgres:
    image: postgis/postgis:16-3.4
    environment:
      POSTGRES_DB: amr
      POSTGRES_USER: amr
      POSTGRES_PASSWORD: amr
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U amr"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - minio_data:/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 5s
      timeout: 5s
      retries: 5

  mqtt:
    image: eclipse-mosquitto:2
    ports:
      - "1883:1883"
      - "9883:9883"
    volumes:
      - ./config/mosquitto.conf:/mosquitto/config/mosquitto.conf
    healthcheck:
      test: ["CMD", "mosquitto_sub", "-t", "$$SYS/#", "-C", "1", "-i", "healthcheck", "-W", "3"]
      interval: 10s
      timeout: 10s
      retries: 3

  sim-engine:
    build:
      context: ./sim-engine
      dockerfile: Dockerfile
    ports:
      - "50051:50051"
    environment:
      ASSET_MANAGER_URL: "asset-manager:50052"
      MAP_MANAGER_URL: "map-manager:50053"
      LOG_LEVEL: info
    depends_on:
      asset-manager:
        condition: service_healthy
      map-manager:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "/bin/grpc_health_probe", "-addr=:50051"]
      interval: 10s
      timeout: 5s
      retries: 5

  asset-manager:
    build:
      context: ./asset-manager
      dockerfile: Dockerfile
    ports:
      - "50052:50052"
    environment:
      DATABASE_URL: "postgresql://amr:amr@postgres:5432/amr"
      MINIO_ENDPOINT: "http://minio:9000"
      MINIO_ACCESS_KEY: minioadmin
      MINIO_SECRET_KEY: minioadmin
      MINIO_BUCKET: assets
      LOG_LEVEL: info
      RUST_LOG: asset_manager=debug
    depends_on:
      postgres:
        condition: service_healthy
      minio:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "/bin/grpc_health_probe", "-addr=:50052"]
      interval: 10s
      timeout: 5s
      retries: 5

  map-manager:
    build:
      context: ./map-manager
      dockerfile: Dockerfile
    ports:
      - "50053:50053"
    environment:
      DATABASE_URL: "postgresql://amr:amr@postgres:5432/amr"
      MINIO_ENDPOINT: "http://minio:9000"
      MINIO_ACCESS_KEY: minioadmin
      MINIO_SECRET_KEY: minioadmin
      MINIO_BUCKET: maps
      REDIS_URL: "redis://redis:6379"
      LOG_LEVEL: info
      RUST_LOG: map_manager=debug
    depends_on:
      postgres:
        condition: service_healthy
      minio:
        condition: service_healthy
      redis:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "/bin/grpc_health_probe", "-addr=:50053"]
      interval: 10s
      timeout: 5s
      retries: 5

  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: "postgresql://amr:amr@postgres:5432/amr"
      REDIS_URL: "redis://redis:6379"
      MINIO_ENDPOINT: "http://minio:9000"
      MINIO_ACCESS_KEY: minioadmin
      MINIO_SECRET_KEY: minioadmin
      MQTT_BROKER_URL: "mqtt://mqtt:1883"
      GRPC_SIM_ENGINE_URL: "http://sim-engine:50051"
      GRPC_ASSET_MANAGER_URL: "http://asset-manager:50052"
      GRPC_MAP_MANAGER_URL: "http://map-manager:50053"
      JWT_SECRET: "integration-test-secret-do-not-use-in-production"
      LOG_LEVEL: info
      RUST_LOG: backend=debug
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
      minio:
        condition: service_healthy
      mqtt:
        condition: service_healthy
      sim-engine:
        condition: service_healthy
      asset-manager:
        condition: service_healthy
      map-manager:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 5

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      VITE_API_URL: "http://localhost:8080"
      VITE_WS_URL: "ws://localhost:8080/api/v1/ws"
    depends_on:
      backend:
        condition: service_healthy

volumes:
  postgres_data:
  minio_data:
```

**실행 방법:**

```bash
# 전체 서비스 기동
docker compose -f docker-compose.integration.yml up -d

# 서비스 상태 확인
docker compose -f docker-compose.integration.yml ps

# 통합 테스트 실행
./scripts/run-integration-tests.sh

# 서비스 종료 및 정리
docker compose -f docker-compose.integration.yml down -v
```

### 5.2 통합 테스트 시나리오

#### Scenario 1: Asset Upload → Sim → Render

**목적**: 에셋 파이프라인의 전체 흐름을 검증한다.

**전제 조건**: 모든 서비스가 정상 기동됨.

**단계**:

| 단계 | 행위자 | 동작 | 검증 |
|------|--------|------|------|
| 1 | Tester | `POST /api/v1/assets` 로 URDF 파일 업로드 | HTTP 201, `status: uploading` |
| 2 | Asset Manager | 파일 수신, MinIO 저장, URDF 파싱, glTF 변환 시작 | DB에 에셋 레코드 생성됨 |
| 3 | Tester | `GET /api/v1/assets/{id}` 폴링 | `status: ready`, `convertedAssets`에 glTF 포함 |
| 4 | Tester | `POST /api/v1/simulations` 세션 생성 | HTTP 201, `state: ready` |
| 5 | Tester | `POST /api/v1/simulations/{id}/robots` 로봇 스폰 | HTTP 201, 에셋 ID 참조 |
| 6 | Sim Engine | AssetService.DownloadAssetStream으로 URDF 다운로드 | gRPC 스트림 정상 완료 |
| 7 | Sim Engine | URDF 파싱, 물리 모델 생성 | 로봇 객체 생성됨 |
| 8 | Frontend | `GET /api/v1/assets/{id}/download?format=glb` 로 glTF 다운로드 | glTF 파일 다운로드 성공 |
| 9 | Frontend | Three.js로 glTF 렌더링 | 3D 모델이 화면에 표시됨 |
| 10 | 검증 | 세 시스템의 로봇 바운딩 박스 비교 | 오차 < 1cm |

**성능 기준**: URDF → glTF 변환 < 30초 (10MB 이하 파일 기준)

#### Scenario 2: Map → Roadmap → Mission → Sim

**목적**: 맵 업로드부터 미션 실행까지의 전체 워크플로우를 검증한다.

**단계**:

| 단계 | 행위자 | 동작 | 검증 |
|------|--------|------|------|
| 1 | Tester | `POST /api/v1/maps` 로 PCD 파일 업로드 (1M 포인트) | HTTP 201 |
| 2 | Map Manager | 포인트 클라우드 수신, Potree 타일링 시작 | WebSocket `map_update` 이벤트 수신 |
| 3 | Tester | WebSocket으로 처리 진행률 모니터링 | `progress` 값이 0.0 → 1.0 증가 |
| 4 | Frontend | Potree 타일 로딩 및 렌더링 | 포인트 클라우드가 3D 뷰에 표시 |
| 5 | Tester | `POST /api/v1/maps/{id}/roadmaps` 로드맵 생성 (10 노드, 12 엣지) | HTTP 201, 노드/엣지 수 일치 |
| 6 | Frontend | 로드맵 그래프 오버레이 렌더링 | 노드와 엣지가 맵 위에 표시 |
| 7 | Tester | `POST /api/v1/missions` 미션 생성 (출발/도착 노드 지정) | HTTP 201, `plannedPath` 반환 |
| 8 | Backend | MapService.FindPath로 경로 탐색 | 경로가 존재하고 유효함 |
| 9 | Tester | `POST /api/v1/missions/{id}/start` 미션 시작 | HTTP 200, `status: in_progress` |
| 10 | Sim Engine | NavigationGoal 수신, 경로 추종 시작 | 텔레메트리에서 위치 변화 확인 |
| 11 | Frontend | WebSocket으로 텔레메트리 수신, 로봇 위치 업데이트 | 로봇이 경로를 따라 이동하는 것이 표시 |
| 12 | 검증 | 미션 완료 시 `status: completed` | 전체 경로 오차 < 0.5m |

**성능 기준**: 1M 포인트 Potree 타일링 < 60초

#### Scenario 3: Real-time Telemetry Flow

**목적**: 텔레메트리 파이프라인의 실시간 성능을 검증한다.

**단계**:

1. Sim Engine에서 1대 로봇이 10Hz로 텔레메트리 생성
2. Backend가 gRPC 스트림으로 수신
3. Backend가 Redis pub/sub으로 발행
4. Backend WebSocket 릴레이가 구독하여 Frontend에 전달
5. Frontend가 수신하여 3D 뷰어 업데이트

**검증 항목**:

| 항목 | 기준 |
|------|------|
| 종단 간 지연시간 (E2E Latency) | p50 < 50ms, p99 < 100ms |
| 메시지 손실률 | 60초 동안 0% |
| 시퀀스 번호 연속성 | 누락 없음 |
| Frontend 프레임률 영향 | 60fps 유지 (10Hz 텔레메트리 수신 중) |

**측정 방법**:
- Sim Engine이 발송하는 `timestamp_ms`와 Frontend 수신 시 `Date.now()` 차이로 E2E 지연시간 측정
- `sequence_number`를 추적하여 손실 감지

#### Scenario 4: Multi-Robot Traffic

**목적**: 다중 로봇 환경에서의 교통 관리 기능을 검증한다.

**단계**:

1. 시뮬레이션 세션 생성, 맵 및 로드맵 로드
2. 5대 로봇 스폰 (서로 다른 초기 위치)
3. 5개 미션 생성 (경로가 교차하도록 설계)
4. 전 미션 동시 시작
5. Backend Traffic Service가 zone lock 관리
6. 시뮬레이션 실행 및 모니터링

**검증 항목**:

| 항목 | 기준 |
|------|------|
| 충돌 | 0건 |
| 데드락 | 0건 |
| 전체 미션 완료 | 5/5 완료 |
| 총 소요 시간 | 단일 로봇 대비 2배 이내 |
| 대기 시간 합계 | 로봇당 평균 30초 이내 |

#### Scenario 5: Plugin Integration

**목적**: WASM 플러그인 시스템의 통합을 검증한다.

**단계**:

1. 샘플 WASM 플러그인 업로드 (`POST /api/v1/plugins`)
   - 플러그인 동작: `mission_completed` 이벤트 수신 시, 동일 로봇에 새 미션(복귀 미션) 생성
2. 플러그인 활성화 (`POST /api/v1/plugins/{id}/activate`)
3. 로봇 미션 생성 및 실행
4. 미션 완료 시 플러그인 트리거
5. 플러그인이 Host API를 통해 새 미션 생성

**검증 항목**:

| 항목 | 기준 |
|------|------|
| 플러그인 로드 | 성공, 에러 없음 |
| 이벤트 수신 | `mission_completed` 이벤트 1회 수신 |
| 새 미션 생성 | Host API를 통해 성공 |
| Frontend 반영 | 대시보드에 새 미션 표시 |
| 플러그인 실행 시간 | < 1초 |
| 메모리 사용량 | < 10MB |

### 5.3 CI/CD 통합 테스트 파이프라인

```yaml
# .github/workflows/integration.yml
name: Integration Tests

on:
  push:
    branches: [main]
  pull_request:
    paths:
      - 'proto/**'
      - 'backend/**'
      - 'frontend/**'
      - 'sim-engine/**'
      - 'asset-manager/**'
      - 'map-manager/**'
      - 'docker-compose.integration.yml'

concurrency:
  group: integration-${{ github.ref }}
  cancel-in-progress: true

jobs:
  proto-lint:
    name: Proto Lint & Breaking Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: bufbuild/buf-setup-action@v1
        with:
          version: '1.30.0'
      - name: Lint proto files
        run: buf lint proto/
      - name: Check for breaking changes
        if: github.event_name == 'pull_request'
        run: buf breaking proto/ --against '.git#branch=main'

  build:
    name: Build All Services
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: [backend, sim-engine, asset-manager, map-manager, frontend]
    steps:
      - uses: actions/checkout@v4
      - name: Build Docker image
        run: |
          docker build -t amr-${{ matrix.service }}:test ./${{ matrix.service }}

  integration-test:
    name: Integration Tests
    needs: [proto-lint, build]
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4

      - name: Start services
        run: |
          docker compose -f docker-compose.integration.yml up -d
        env:
          COMPOSE_PROJECT_NAME: amr-integration

      - name: Wait for services to be healthy
        run: |
          ./scripts/wait-for-services.sh
        timeout-minutes: 5

      - name: Run integration tests
        run: |
          ./scripts/run-integration-tests.sh
        timeout-minutes: 15

      - name: Collect logs on failure
        if: failure()
        run: |
          docker compose -f docker-compose.integration.yml logs > integration-logs.txt

      - name: Upload logs on failure
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: integration-logs
          path: integration-logs.txt

      - name: Cleanup
        if: always()
        run: |
          docker compose -f docker-compose.integration.yml down -v

  performance-test:
    name: Performance Baseline
    needs: integration-test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4

      - name: Start services
        run: |
          docker compose -f docker-compose.integration.yml up -d

      - name: Wait for services
        run: ./scripts/wait-for-services.sh

      - name: Run performance tests
        run: |
          ./scripts/run-performance-tests.sh

      - name: Upload performance report
        uses: actions/upload-artifact@v4
        with:
          name: performance-report
          path: performance-report.json

      - name: Cleanup
        if: always()
        run: |
          docker compose -f docker-compose.integration.yml down -v
```

### 5.4 Contract Testing

각 팀은 자신이 소비하는 인터페이스에 대해 계약 테스트를 작성하고 유지한다.

#### Proto 호환성 테스트

```bash
# buf breaking change detection
# CI에서 main 브랜치 대비 변경 감지
buf breaking proto/ --against '.git#branch=main'
```

#### OpenAPI 스키마 검증

```bash
# OpenAPI spec diff (oasdiff 사용)
oasdiff diff openapi-spec-main.yaml openapi-spec-pr.yaml --format text

# Breaking change 감지
oasdiff breaking openapi-spec-main.yaml openapi-spec-pr.yaml
```

#### WebSocket 메시지 스키마 검증

Frontend(Team 1)는 수신하는 WebSocket 메시지를 JSON Schema로 검증한다.

```typescript
// Frontend contract test example
import Ajv from 'ajv';

const telemetrySchema = {
  type: 'object',
  required: ['version', 'type', 'topic', 'timestamp', 'payload'],
  properties: {
    version: { type: 'integer', const: 1 },
    type: { type: 'string', const: 'telemetry' },
    topic: { type: 'string', pattern: '^telemetry:[0-9a-f-]{36}$' },
    timestamp: { type: 'integer' },
    payload: {
      type: 'object',
      required: ['robotId', 'sequenceNumber', 'pose', 'velocity', 'battery', 'driving'],
      properties: {
        robotId: { type: 'string', format: 'uuid' },
        sequenceNumber: { type: 'integer', minimum: 0 },
        pose: {
          type: 'object',
          required: ['position', 'orientation'],
          properties: {
            position: { $ref: '#/$defs/vector3' },
            orientation: { $ref: '#/$defs/quaternion' }
          }
        },
        velocity: {
          type: 'object',
          required: ['linear', 'angular'],
          properties: {
            linear: { $ref: '#/$defs/vector3' },
            angular: { $ref: '#/$defs/vector3' }
          }
        },
        battery: { type: 'number', minimum: 0, maximum: 100 },
        driving: { type: 'boolean' },
        errors: { type: 'array' }
      }
    }
  },
  $defs: {
    vector3: {
      type: 'object',
      required: ['x', 'y', 'z'],
      properties: {
        x: { type: 'number' },
        y: { type: 'number' },
        z: { type: 'number' }
      }
    },
    quaternion: {
      type: 'object',
      required: ['x', 'y', 'z', 'w'],
      properties: {
        x: { type: 'number' },
        y: { type: 'number' },
        z: { type: 'number' },
        w: { type: 'number' }
      }
    }
  }
};

const ajv = new Ajv();
const validate = ajv.compile(telemetrySchema);

// 테스트에서 Mock 서버 메시지 검증
test('telemetry message conforms to schema', () => {
  const message = receiveTelemetryFromMockServer();
  const valid = validate(message);
  expect(valid).toBe(true);
});
```

#### gRPC 계약 테스트

각 팀은 Mock 서버를 상대로 자신의 gRPC 클라이언트를 테스트한다.

```rust
// Backend(Team 2) gRPC client contract test example
#[tokio::test]
async fn test_sim_engine_create_session_contract() {
    let channel = Channel::from_static("http://localhost:50051")
        .connect()
        .await
        .unwrap();

    let mut client = SimulationServiceClient::new(channel);

    let request = CreateSessionRequest {
        config: Some(SimConfig {
            time_step: 0.001,
            real_time_factor: 1.0,
            ..Default::default()
        }),
        name: "contract-test".to_string(),
    };

    let response = client.create_session(request).await.unwrap();
    let session = response.into_inner();

    assert!(session.session_id.is_some());
    assert!(session.session.is_some());
    let session_info = session.session.unwrap();
    assert_eq!(session_info.state(), SimSessionState::Ready);
}
```

---

## 6. 문서 스키마 & 커뮤니케이션

### 6.1 인터페이스 변경 요청 (RFC) 템플릿

모든 인터페이스 변경은 RFC 문서를 통해 제안하고 승인을 받아야 한다. RFC 파일은 `docs/rfcs/` 디렉토리에 저장한다.

```markdown
# RFC-{number}: {title}

- **작성자**: {이름} (Team N)
- **작성일**: YYYY-MM-DD
- **상태**: Draft | In Review | Approved | Rejected | Withdrawn
- **영향 범위**: proto | rest | websocket | mqtt

## 요약

한 줄 요약.

## 동기

이 변경이 필요한 이유를 설명한다. 현재 제약 사항, 사용 사례, 기술적 부채 등을 포함한다.

## 변경 내용

### Before (현재)

```protobuf
// 현재 정의
message Example {
  string id = 1;
  string name = 2;
}
```

### After (제안)

```protobuf
// 제안 정의
message Example {
  string id = 1;
  string name = 2;
  string description = 3;  // 새로 추가
  int64 created_at_ms = 4; // 새로 추가
}
```

### 하위 호환성

- [ ] 하위 호환 (기존 클라이언트에 영향 없음)
- [ ] 하위 호환 깨짐 (마이그레이션 필요)

## 영향받는 팀

- [ ] Team 1 (Frontend) — {영향 설명}
- [ ] Team 2 (Backend) — {영향 설명}
- [ ] Team 3 (Sim Engine) — {영향 설명}
- [ ] Team 4 (Asset Manager) — {영향 설명}
- [ ] Team 5 (Map Manager) — {영향 설명}

## 마이그레이션 계획

각 영향받는 팀이 수행해야 할 작업을 단계별로 설명한다.

1. Team X: ...
2. Team Y: ...

## 대안 검토

검토한 다른 방안과 채택하지 않은 이유를 설명한다.

## 승인

- [ ] Team 1 Lead — @name (YYYY-MM-DD)
- [ ] Team 2 Lead — @name (YYYY-MM-DD)
- [ ] Team 3 Lead — @name (YYYY-MM-DD)
- [ ] Team 4 Lead — @name (YYYY-MM-DD)
- [ ] Team 5 Lead — @name (YYYY-MM-DD)
```

### 6.2 팀 간 상태 리포트 스키마

각 팀은 매주 월요일까지 상태 리포트를 `docs/status/week-{N}/team-{M}.json` 파일로 제출한다.

```json
{
  "team": "Team 3: Simulation Engine",
  "week": 7,
  "phase": "Phase 2",
  "reportDate": "2026-05-04",
  "status": "on_track",
  "statusDescription": "Phase 2 일정에 따라 정상 진행 중",
  "completedThisWeek": [
    "Bullet Physics 엔진 통합 완료",
    "gRPC StreamTelemetry 구현 (10Hz)",
    "로봇 충돌 감지 구현"
  ],
  "plannedNextWeek": [
    "LiDAR 센서 시뮬레이션 구현",
    "NavigationGoal RPC 구현",
    "다중 로봇 동시 시뮬레이션 테스트"
  ],
  "blockers": [
    {
      "description": "Asset Manager에서 URDF 다운로드 시 간헐적 타임아웃 발생",
      "blockedBy": "Team 4",
      "impact": "로봇 스폰 테스트 간헐적 실패",
      "requestedResolutionDate": "2026-05-08",
      "severity": "medium"
    }
  ],
  "interfaceChangesNeeded": [
    {
      "type": "proto_change",
      "file": "proto/common/telemetry.proto",
      "description": "TelemetryMessage에 sensor_readings 필드 추가 필요",
      "rfcId": "RFC-003",
      "rfcStatus": "in_review"
    }
  ],
  "mockServerStatus": {
    "available": true,
    "endpoint": "localhost:50051",
    "lastUpdated": "2026-05-02",
    "supportedScenarios": [
      "basic_telemetry",
      "spawn_robot",
      "navigation_goal"
    ]
  },
  "metrics": {
    "testsTotal": 145,
    "testsPassing": 138,
    "testsFailing": 5,
    "testsSkipped": 2,
    "codeCoverage": 72.5
  }
}
```

**`status` 값 정의:**

| 값 | 의미 | 에스컬레이션 |
|----|------|------------|
| `on_track` | 일정대로 진행 중 | 없음 |
| `at_risk` | 일정 지연 가능성 있음 | 팀 리드 간 논의 |
| `blocked` | 외부 의존성으로 인해 진행 불가 | 즉시 에스컬레이션, 24시간 내 해결 방안 수립 |

### 6.3 에러 코드 규약

팀 간 에러 코드 충돌을 방지하기 위해 다음과 같이 범위를 할당한다.

| 코드 범위 | 소유 팀 | 용도 |
|----------|---------|------|
| 0-999 | 공통 | 공통 에러 (proto `ErrorCode`에 정의) |
| 1000-1999 | Team 2 (Backend) | 인증/인가 에러 |
| 2000-2999 | Team 2 (Backend) | 로봇/미션 에러 |
| 3000-3999 | Team 3 (Sim Engine) | 시뮬레이션 에러 |
| 4000-4999 | Team 4 (Asset Manager) | 에셋 관리 에러 |
| 5000-5999 | Team 5 (Map Manager) | 맵 관리 에러 |
| 6000-6999 | Team 2 (Backend) | 플러그인 에러 |
| 7000-9999 | 예약 | 향후 확장용 |

**에러 코드 사용 규칙:**
- 새 에러 코드를 추가할 때는 자신의 범위 내에서만 할당한다
- `proto/common/errors.proto`의 `ErrorCode` enum에 추가하고, RFC 프로세스를 거친다
- 에러 코드는 한번 할당되면 의미를 변경할 수 없다 (삭제만 가능)
- REST API에서는 HTTP 상태 코드와 함께 `code` 필드에 에러 코드를 반환한다

**HTTP 상태 코드 매핑:**

| gRPC Status | HTTP Status | 사용 상황 |
|------------|------------|----------|
| OK | 200 | 성공 |
| INVALID_ARGUMENT | 400 | 잘못된 요청 |
| UNAUTHENTICATED | 401 | 인증 실패 |
| PERMISSION_DENIED | 403 | 권한 없음 |
| NOT_FOUND | 404 | 리소스 없음 |
| ALREADY_EXISTS | 409 | 리소스 충돌 |
| RESOURCE_EXHAUSTED | 429 | 요청 제한 초과 |
| INTERNAL | 500 | 서버 내부 에러 |
| UNAVAILABLE | 503 | 서비스 일시 불가 |

### 6.4 로깅 규약

모든 서비스는 구조화된 JSON 로깅을 사용하며, OpenTelemetry 추적을 지원한다.

#### 로그 형식

```json
{
  "timestamp": "2026-03-21T10:00:00.123Z",
  "level": "info",
  "service": "backend",
  "version": "1.2.3",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "parent_span_id": "a1b2c3d4e5f60718",
  "message": "Mission created successfully",
  "fields": {
    "mission_id": "660e8400-e29b-41d4-a716-446655440000",
    "robot_id": "550e8400-e29b-41d4-a716-446655440000",
    "step_count": 5,
    "duration_ms": 45
  }
}
```

#### 서비스별 로거 이름

| 서비스 | `service` 값 | 로그 레벨 (프로덕션) |
|-------|-------------|-------------------|
| Frontend | `frontend` | `warn` |
| Backend | `backend` | `info` |
| Sim Engine | `sim-engine` | `info` |
| Asset Manager | `asset-manager` | `info` |
| Map Manager | `map-manager` | `info` |

#### 로그 레벨 가이드

| 레벨 | 용도 | 예시 |
|------|------|------|
| `debug` | 개발/디버깅용 상세 정보 | "Received telemetry message, seq=42" |
| `info` | 정상 비즈니스 이벤트 | "Mission started", "Robot spawned" |
| `warn` | 비정상이지만 복구 가능한 상황 | "Retry attempt 2/3 for gRPC call" |
| `error` | 복구 불가능한 오류 | "Database connection failed", "URDF parse error" |

#### OpenTelemetry Trace 전파

gRPC 호출 시:
- `traceparent` 메타데이터 키로 W3C Trace Context 전파
- 예: `traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`

HTTP 호출 시:
- `traceparent` 헤더로 W3C Trace Context 전파
- `tracestate` 헤더로 벤더별 추가 정보 전파

WebSocket 메시지:
- 메시지 엔벨로프에 `traceId` 필드로 전파 (선택적)

---

## 7. 좌표계 및 단위 규약

모든 팀은 다음 좌표계 및 단위 규약을 **반드시** 준수해야 한다. 이 규약은 ROS REP 103 (Standard Units of Measure and Coordinate Conventions) 및 REP 105 (Coordinate Frames for Mobile Platforms)를 기반으로 한다.

### 7.1 좌표계

| 항목 | 규약 | 비고 |
|------|------|------|
| 좌표계 | ENU (East-North-Up) | 오른손 법칙 |
| X축 | East (동쪽) | 양의 방향 = 동쪽 |
| Y축 | North (북쪽) | 양의 방향 = 북쪽 |
| Z축 | Up (위쪽) | 양의 방향 = 위쪽 (중력 반대) |

### 7.2 단위

| 물리량 | 단위 | 비고 |
|--------|------|------|
| 거리/위치 | 미터 (m) | |
| 각도 | 라디안 (rad) | 반시계 방향이 양수 |
| 시간 | 초 (s) | 물리 시뮬레이션, 소요 시간 등 |
| 속도 | m/s | |
| 각속도 | rad/s | |
| 가속도 | m/s² | |
| 힘 | 뉴턴 (N) | |
| 질량 | 킬로그램 (kg) | |
| 온도 | 섭씨 (°C) | |
| 전압 | 볼트 (V) | |
| 전류 | 암페어 (A) | |

### 7.3 회전 표현

| 항목 | 규약 |
|------|------|
| 3D 회전 | 쿼터니언 (x, y, z, w) — Hamilton 규약 |
| 2D 회전 | theta (라디안) — East축 기준 반시계 방향 |
| 단위 쿼터니언 | x² + y² + z² + w² = 1 |
| 항등 쿼터니언 | (0, 0, 0, 1) |

### 7.4 타임스탬프

| 용도 | 형식 | 예시 |
|------|------|------|
| REST API 응답 | ISO 8601 (UTC) | `"2026-03-21T10:00:00.000Z"` |
| WebSocket 메시지 | Unix milliseconds (int64) | `1711015200000` |
| Proto 메시지 | Unix milliseconds (int64) | `1711015200000` |
| MQTT/VDA5050 | ISO 8601 (UTC) | `"2026-03-21T10:00:00.000Z"` |
| 로그 | ISO 8601 (UTC) | `"2026-03-21T10:00:00.123Z"` |

### 7.5 좌표 변환 참조

Frontend(Three.js)는 기본적으로 Y-up 좌표계를 사용하므로, ENU → Three.js 변환이 필요하다:

```
ENU (서버)     →  Three.js (Frontend)
X (East)       →  X
Y (North)      →  Z (부호 반전: -Z)
Z (Up)         →  Y
```

**이 변환은 Frontend(Team 1)의 책임**이며, 서버 측에서는 항상 ENU 좌표계만 사용한다.

---

## 8. 데이터 포맷 규약

### 8.1 식별자 (UUID)

| 항목 | 규약 | 예시 |
|------|------|------|
| 형식 | UUID v4 | `"550e8400-e29b-41d4-a716-446655440000"` |
| 표기 | 소문자 hex, 하이픈 포함 | |
| 길이 | 36자 (하이픈 포함) | |
| 생성 | 각 서비스가 자체 생성 | |

### 8.2 JSON 필드 명명

| 컨텍스트 | 규약 | 예시 |
|----------|------|------|
| REST API (JSON) | camelCase | `robotId`, `createdAt`, `batteryPercentage` |
| Proto 정의 | snake_case | `robot_id`, `created_at`, `battery_percentage` |
| Proto-generated JSON | snake_case | Proto의 기본 JSON 매핑 사용 |
| WebSocket 메시지 | camelCase | REST API와 동일 |
| MQTT/VDA5050 | camelCase | VDA5050 표준 준수 |

> **주의**: Backend(Team 2)는 REST/WebSocket 응답에서 Proto 메시지를 JSON으로 변환할 때 camelCase로 매핑해야 한다.

### 8.3 바이너리 데이터

| 항목 | 규약 |
|------|------|
| 바이트 순서 | Little-endian |
| 포인트 클라우드 좌표 | float64 (double precision, 8바이트) |
| 센서 읽기 값 | float32 (single precision, 4바이트) |
| 이미지 데이터 | 원본 형식 유지 (PNG, JPG 등) |
| gRPC 스트리밍 청크 크기 | 최대 64KB (65,536바이트) |

### 8.4 페이지네이션

REST API의 목록 조회 엔드포인트는 다음 페이지네이션 규약을 따른다.

**요청 파라미터:**

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|--------|------|
| `page` | integer | 1 | 페이지 번호 (1-based) |
| `pageSize` | integer | 20 | 페이지 크기 (최소 1, 최대 100) |
| `sortBy` | string | `createdAt` | 정렬 필드 |
| `sortDesc` | boolean | `true` | 내림차순 여부 |

**응답 구조:**

```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "pageSize": 20,
    "totalCount": 150,
    "totalPages": 8
  }
}
```

gRPC 목록 조회는 커서 기반 페이지네이션을 사용한다:

| 필드 | 설명 |
|------|------|
| `page_size` | 페이지 크기 |
| `page_token` | 커서 토큰 (이전 응답의 `next_page_token`) |
| `next_page_token` | 다음 페이지 커서 (빈 문자열이면 마지막 페이지) |

### 8.5 파일 업로드

| 항목 | 규약 |
|------|------|
| 방식 | Presigned URL (MinIO) |
| 최대 파일 크기 | 5GB (multipart upload) |
| 무결성 검증 | SHA-256 체크섬 |
| 지원 형식 (에셋) | URDF, SDF, glTF, GLB, OBJ, STL, DAE, FBX |
| 지원 형식 (맵) | PCD, LAS, LAZ, PLY, E57, XYZ |
| 지원 형식 (플러그인) | WASM |

**업로드 흐름:**

```
1. Client → Backend:  POST /api/v1/assets (메타데이터)
2. Backend → Client:  201 {asset, uploadUrl}
3. Client → MinIO:    PUT {uploadUrl} (파일 바이너리)
4. Client → Backend:  POST /api/v1/assets/{id}/complete-upload {checksumSha256}
5. Backend → Service: 처리 시작 (변환, 타일링 등)
6. Service → Backend: 처리 완료 통보
7. Backend → Client:  WebSocket map_update / asset status 변경
```

### 8.6 날짜/시간 포맷

| 형식 | 사용처 | 예시 |
|------|--------|------|
| ISO 8601 UTC | REST API, VDA5050, 로그 | `"2026-03-21T10:00:00.123Z"` |
| Unix milliseconds (int64) | Proto, WebSocket payload | `1711015200123` |
| Unix seconds (double) | 시뮬레이션 시간 | `45.678` |

> **중요**: 모든 시간은 UTC 기준이다. 로컬 시간대 변환은 Frontend의 책임이다.

---

## 부록 A: 서비스 포트 할당

| 서비스 | 프로토콜 | 포트 | 비고 |
|--------|---------|------|------|
| Frontend (dev server) | HTTP | 3000 | Vite dev server |
| Backend REST/WS | HTTP | 8080 | Axum 서버 |
| Sim Engine gRPC | HTTP/2 | 50051 | |
| Asset Manager gRPC | HTTP/2 | 50052 | |
| Map Manager gRPC | HTTP/2 | 50053 | |
| PostgreSQL | TCP | 5432 | |
| Redis | TCP | 6379 | |
| MinIO API | HTTP | 9000 | |
| MinIO Console | HTTP | 9001 | |
| MQTT Broker | TCP | 1883 | Mosquitto |
| MQTT WebSocket | WS | 9883 | 선택적 |

## 부록 B: 환경 변수 목록

모든 서비스에서 사용하는 환경 변수 목록이다.

| 환경 변수 | 서비스 | 설명 | 예시 |
|----------|--------|------|------|
| `DATABASE_URL` | Backend, Asset Mgr, Map Mgr | PostgreSQL 연결 문자열 | `postgresql://amr:amr@localhost:5432/amr` |
| `REDIS_URL` | Backend, Map Mgr | Redis 연결 문자열 | `redis://localhost:6379` |
| `MINIO_ENDPOINT` | Backend, Asset Mgr, Map Mgr | MinIO 엔드포인트 | `http://localhost:9000` |
| `MINIO_ACCESS_KEY` | Backend, Asset Mgr, Map Mgr | MinIO 접근 키 | `minioadmin` |
| `MINIO_SECRET_KEY` | Backend, Asset Mgr, Map Mgr | MinIO 비밀 키 | `minioadmin` |
| `MINIO_BUCKET` | Asset Mgr, Map Mgr | MinIO 버킷 이름 | `assets`, `maps` |
| `MQTT_BROKER_URL` | Backend | MQTT 브로커 URL | `mqtt://localhost:1883` |
| `GRPC_SIM_ENGINE_URL` | Backend | Sim Engine gRPC URL | `http://localhost:50051` |
| `GRPC_ASSET_MANAGER_URL` | Backend, Sim Engine | Asset Manager gRPC URL | `http://localhost:50052` |
| `GRPC_MAP_MANAGER_URL` | Backend, Sim Engine | Map Manager gRPC URL | `http://localhost:50053` |
| `JWT_SECRET` | Backend | JWT 서명 키 | (임의 문자열, 최소 32자) |
| `LOG_LEVEL` | 전체 | 로그 레벨 | `debug`, `info`, `warn`, `error` |
| `RUST_LOG` | Rust 서비스 | Rust 로그 필터 | `backend=debug,tower_http=debug` |
| `VITE_API_URL` | Frontend | Backend API URL | `http://localhost:8080` |
| `VITE_WS_URL` | Frontend | WebSocket URL | `ws://localhost:8080/api/v1/ws` |

## 부록 C: 성능 벤치마크 기준

Phase 4 Final Integration에서 달성해야 할 성능 기준이다.

| 항목 | 기준 | 측정 조건 |
|------|------|----------|
| REST API 응답 시간 (p99) | < 200ms | 목록 조회, 50개 로봇 |
| WebSocket 텔레메트리 지연 (p99) | < 100ms | 10대 로봇, 10Hz |
| gRPC 호출 지연 (p99) | < 50ms | Unary RPC |
| Potree 타일 로딩 시간 | < 500ms | 단일 타일 조회 |
| URDF → glTF 변환 시간 | < 30s | 10MB 이하 파일 |
| PCD → Potree 타일링 시간 | < 120s | 10M 포인트 |
| 맵 경로 탐색 시간 | < 100ms | 1000 노드 그래프, A* |
| 시뮬레이션 1 스텝 시간 | < 1ms | 10대 로봇, 충돌 감지 포함 |
| 동시 WebSocket 연결 | 100+ | |
| 동시 시뮬레이션 로봇 | 50+ | 10Hz 텔레메트리 |
| Frontend 초기 로딩 시간 | < 3s | gzip 압축, 프로덕션 빌드 |
| Frontend 3D 뷰어 FPS | 60fps | 10대 로봇 + 1M 포인트 클라우드 |

---

> **이 문서는 살아있는 문서(living document)**이다. 변경이 필요한 경우 RFC 프로세스를 통해 수정하고, 변경 이력은 Git 히스토리에서 관리한다.
