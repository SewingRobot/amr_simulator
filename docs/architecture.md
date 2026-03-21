# AMR 통합 프레임워크 — 아키텍처 개요

> 이 문서는 최상위 통합 아키텍처 문서입니다. 각 기능의 상세 설계는 `docs/features/` 하위 문서를, 팀별 기술스택과 구현 가이드는 `docs/teams/` 문서를 참조하세요.

---

## 1. 프로젝트 개요

AMR(Autonomous Mobile Robot) 통합 프레임워크는 자율 이동 로봇의 셋업, 원격 조작, 관제, 시뮬레이션을 하나의 플랫폼에서 수행하는 통합 시스템이다. 5개 서브 프로젝트(Frontend, Backend, Simulation Engine, Asset Manager, Map Manager)를 병렬 개발하며, VDA5050 표준 기반 통신과 OpenUSD 씬 포맷, 플러거블 물리 엔진 아키텍처를 통해 경량 배포부터 고충실도 시뮬레이션까지 유연하게 지원한다.

---

## 2. 전체 아키텍처 다이어그램

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Web App (Frontend)                              │
│              React + TypeScript + Three.js/R3F + WebGPU                │
│         Dashboard │ 3D Viewer │ Mission Control │ Map Editor           │
└──────────┬──────────────────┬─────────────────────┬─────────────────────┘
           │ WebSocket / REST │ WebRTC (video)      │ HTTP Range (Potree)
┌──────────▼──────────────────▼─────────────────────▼─────────────────────┐
│                        Backend (Rust / Axum)                            │
│       API Gateway │ Auth │ WebSocket Relay │ Plugin Host (WASM)         │
│       Mission Service │ Traffic Service │ ROS 2 Bridge Plugin           │
│       PostgreSQL + TimescaleDB │ Redis │ MinIO (S3)                     │
└──┬──────────┬──────────────┬──────────────────┬─────────────────────────┘
   │ gRPC     │ gRPC         │ gRPC             │ MQTT (VDA5050 v2.0)
┌──▼───────┐ ┌▼────────────┐ ┌▼─────────────┐  ┌▼──────────────────────┐
│  Sim     │ │   Asset     │ │    Map       │  │   Real Robots         │
│  Engine  │ │  Manager    │ │  Manager     │  │   (VDA5050 native)    │
│  C++/Py  │ │  Rust/Py    │ │  Rust/Py     │  │   [ROS 2 Bridge       │
│          │ │             │ │              │  │    Plugin — optional]  │
│ ┌──────────────────────────────┐           │  └───────────────────────┘
│ │  PhysicsBackend Interface    │           │
│ │  ┌─────────┬────────┬──────┐ │           │
│ │  │Custom   │MuJoCo  │Isaac │ │           │
│ │  │Lightweight│Plugin │Sim   │ │           │
│ │  │(default)│        │Plugin│ │           │
│ │  └─────────┴────────┴──────┘ │           │
│ └──────────────────────────────┘           │
└──────────┘ └─────────────┘ └──────────────┘

Scene Format: OpenUSD (Universal Scene Description)
  ├── URDF/SDF → USD 변환 (Asset Manager)
  ├── USD → glTF 변환 (Frontend 웹 렌더링용)
  └── UsdPhysics schema (Simulation용)
```

---

## 3. 기능 분류 — Core vs Advanced

### 핵심 기능 (Core — Phase 1 프로토타입에 포함)

| ID | 기능명 | 설명 |
|----|--------|------|
| **C-01** | Sim Engine Core | 커스텀 경량 물리 엔진, 기본 로봇(차동구동), 2D LiDAR 시뮬, gRPC 텔레메트리 스트리밍 |
| **C-02** | 3D Viewer Core | Three.js 씬 렌더링, glTF 로봇 모델 로드, 카메라 컨트롤, 실시간 포즈 업데이트 |
| **C-03** | Backend Core | REST API (auth, robot CRUD), WebSocket 릴레이, PostgreSQL, 기본 gRPC 클라이언트 |
| **C-04** | Map Core | 포인트 클라우드 업로드/처리/Potree 변환, 기본 로드맵 그래프 CRUD, 타일 서빙 |
| **C-05** | Asset Core | 에셋 업로드/다운로드, 메타데이터 CRUD, 기본 URDF→glTF 변환 |
| **C-06** | Telemetry Pipeline | Sim→Backend→Frontend E2E 실시간 텔레메트리 흐름 |

### 고도화 기능 (Advanced — Phase 2 이후)

| ID | 기능명 | 설명 |
|----|--------|------|
| **A-01** | Mission System | 미션 CRUD, 상태머신, VDA5050 오더 생성, 경로 계획 |
| **A-02** | Traffic Management | 존 잠금, 충돌 감지, 우선순위 큐, 데드락 방지 |
| **A-03** | Remote Control | 조이스틱/키보드 원격 조종, WebRTC 카메라 피드 |
| **A-04** | VDA5050 Real Robot | MQTT 실로봇 통신, VDA5050 v2.0 브릿지 |
| **A-05** | Advanced Simulation | MuJoCo 백엔드, Isaac Sim 브릿지, 3D LiDAR, 카메라 센서, IMU |
| **A-06** | Map Editor | 시맨틱 영역 편집, 장애물 배치, 언두/리두 |
| **A-07** | Plugin System | WASM 런타임, HTTP 웹훅, ROS 2 브릿지 플러그인 |
| **A-08** | Dashboard & Analytics | 플릿 KPI, 미션 분석, 히트맵 |
| **A-09** | OpenUSD Pipeline | 전체 USD 워크플로우, USD↔SDF/URDF 양방향 변환, UsdPhysics |
| **A-10** | Digital Twin | 하이브리드 모드(실제+시뮬 오버레이), 디지털 트윈 예측 |

> 각 기능의 상세 스펙은 `docs/features/core/` 및 `docs/features/advanced/` 참조

---

## 4. 통신 아키텍처

### 기본 원칙

- **네이티브 VDA5050 통신이 기본**이며, ROS 2는 선택적 플러그인으로 제공
- ROS 2 의존성을 제거하여 경량 배포 가능 (ROS 인프라 불필요)
- ROS 2 기반 로봇이 있는 팀은 ROS 2 Bridge Plugin을 로드하여 DDS ↔ MQTT/VDA5050 변환

### 프로토콜 매트릭스

| 구간 | 프로토콜 | 포맷 | 용도 |
|------|----------|------|------|
| Frontend ↔ Backend | REST (HTTPS) | JSON | CRUD, 인증, 설정 |
| Frontend ↔ Backend | WebSocket (WSS) | JSON | 실시간 텔레메트리, 미션 상태 |
| Frontend ↔ Backend | WebRTC | H.264/VP9 | 로봇/시뮬 카메라 스트림 |
| Frontend ↔ Backend | HTTP Range | Binary | Potree 포인트 클라우드 타일 |
| Backend ↔ Sim Engine | gRPC (양방향 스트리밍) | Protobuf | 시뮬 제어, 텔레메트리 수신 |
| Backend ↔ Asset Manager | gRPC | Protobuf | 에셋 CRUD, 파일 전송 |
| Backend ↔ Map Manager | gRPC | Protobuf | 맵/로드맵 CRUD, 타일 서빙 |
| Backend ↔ Real Robots | MQTT (VDA5050 v2.0) | JSON | 오더, 상태, 시각화 메시지 |
| Backend ↔ ROS 2 Robots | ROS 2 Bridge Plugin | DDS ↔ MQTT 변환 | ROS 2 토픽/서비스 브릿지 |
| Backend ↔ Plugins | WASM / HTTP Webhook | JSON | 외부 시스템 통합 (WMS 등) |

### 공유 인터페이스 정의

모든 gRPC 서비스 인터페이스는 `proto/` 디렉토리에서 단일 진실 소스(Single Source of Truth)로 관리한다.

```
proto/
  common/       geometry.proto, telemetry.proto, identifiers.proto
  simulation/   sim_service.proto, sim_types.proto
  asset/        asset_service.proto, asset_types.proto
  map/          map_service.proto, map_types.proto
  mission/      mission_types.proto
```

---

## 5. 데이터 흐름 개요

### 텔레메트리 흐름 (Robot → User)

```
Robot/Sim Engine
  → MQTT(VDA5050) 또는 gRPC Stream
    → Backend (WebSocket Relay)
      → Frontend (실시간 3D 렌더링)
      → TimescaleDB (이력 저장)
```

### 명령 흐름 (User → Robot)

```
Frontend (미션 생성 / 원격 조종)
  → REST 또는 WebSocket
    → Backend (Mission Service / Traffic Service)
      → MQTT(VDA5050 Order) → Real Robot
      → gRPC (SetRobotCommand) → Sim Engine
```

### 에셋 흐름

```
URDF/SDF 업로드 → Asset Manager
  → OpenUSD 변환 (시뮬레이션용)
  → glTF 변환 (웹 렌더링용)
  → MinIO 저장 + PostgreSQL 메타데이터
  → Frontend/Sim Engine에서 필요 시 다운로드
```

### 맵 데이터 흐름

```
LAS/PLY 포인트 클라우드 업로드 → Map Manager
  → Open3D 처리 (필터링, 다운샘플링)
  → Potree 옥트리 변환 → MinIO 저장
  → Frontend에서 HTTP Range로 LOD 프로그레시브 로딩
  → 로드맵 그래프 / 시맨틱 레이어 → PostGIS
```

---

## 6. 공유 규약

| 항목 | 규약 |
|------|------|
| 좌표계 | 오른손 좌표계 (ROS REP-103 기준): X-전방, Y-좌측, Z-상방 |
| 단위 | SI 단위: 미터(m), 라디안(rad), 초(s), 킬로그램(kg) |
| ID 포맷 | UUIDv7 (시간순 정렬 가능) |
| Timestamp | ISO 8601 (UTC): `2026-03-21T09:30:00.000Z` |
| 각도 표현 | 쿼터니언 (내부), 오일러 각도 (UI 표시용) |
| 에셋 네이밍 | `{type}_{vendor}_{model}_{version}` (예: `robot_mir_250_v2`) |

> 상세 규약은 [`docs/integration/integration-spec.md`](integration/integration-spec.md) 참조

---

## 7. 개발 Phase 개요

| Phase | 기간 | 내용 | 마일스톤 |
|-------|------|------|----------|
| **Phase 1: Prototype** | Week 1-8 | 핵심 기능(C-01~C-06) 전체 구현. 시뮬레이션 필수 포함. E2E 데이터 플로우 동작 | 시뮬 로봇이 맵 위에서 주행하고, 브라우저에서 실시간 3D 시각화 |
| **Phase 2: Mission & Control** | Week 9-16 | A-01 미션 시스템, A-02 트래픽, A-03 원격 조종, A-06 맵 에디터 | 미션 생성→로봇 할당→시뮬 실행→모니터링 전체 흐름 |
| **Phase 3: Real Robot & Advanced Sim** | Week 17-22 | A-04 VDA5050 실로봇, A-05 고급 시뮬(MuJoCo/Isaac Sim), A-09 OpenUSD | 실로봇+시뮬 동일 인터페이스 운용 |
| **Phase 4: Production** | Week 23-30 | A-07 플러그인, A-08 대시보드, A-10 디지털 트윈, 배포 자동화, 보안 강화 | 프로덕션 레디 |

> 단계별 상세 개발 계획은 [`docs/development-plan.md`](development-plan.md) 참조

---

## 8. 문서 구조 안내

```
docs/
├── architecture.md                              # 본 문서
├── development-plan.md                          # 단계별 개발 계획
├── features/
│   ├── core/
│   │   ├── C-01-sim-engine-core.md
│   │   ├── C-02-3d-viewer-core.md
│   │   ├── C-03-backend-core.md
│   │   ├── C-04-map-core.md
│   │   ├── C-05-asset-core.md
│   │   └── C-06-telemetry-pipeline.md
│   └── advanced/
│       ├── A-01-mission-system.md
│       ├── A-02-traffic-management.md
│       ├── A-03-remote-control.md
│       ├── A-04-vda5050-real-robot.md
│       ├── A-05-advanced-simulation.md
│       ├── A-06-map-editor.md
│       ├── A-07-plugin-system.md
│       ├── A-08-dashboard-analytics.md
│       ├── A-09-openusd-pipeline.md
│       └── A-10-digital-twin.md
├── integration/
│   ├── integration-spec.md                      # 크로스팀 인터페이스 통합 관리
│   ├── test-data-spec.md                        # 테스트 데이터 명세서
│   └── data-pipeline.md                         # 데이터 관리 파이프라인
└── teams/                                       # compact 팀 문서 (기술스택 + 기능 참조)
    ├── team1-frontend.md
    ├── team2-backend.md
    ├── team3-sim-engine.md
    ├── team4-asset-manager.md
    └── team5-map-manager.md
```

| 문서 유형 | 설명 |
|-----------|------|
| `features/core/C-*.md` | Phase 1 핵심 기능 상세 스펙 (구현 범위, API, 테스트 기준) |
| `features/advanced/A-*.md` | Phase 2+ 고도화 기능 상세 스펙 |
| `teams/team*.md` | 팀별 기술스택, 모듈 구조, 담당 기능 참조 |
| `integration/` | 크로스팀 인터페이스 계약, 테스트 데이터, 파이프라인 |
