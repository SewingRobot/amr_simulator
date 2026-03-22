# AMR 통합 프레임워크 — 개발 계획

> 각 기능의 상세 스펙은 `docs/features/` 하위 문서를 참조. 본 문서는 전체 일정, 의존성, 팀 할당을 관리한다.

---

## 1. 전체 기능 목록

### 핵심 기능 (Core)

| ID | 기능명 | 주 담당팀 | 설명 |
|----|--------|-----------|------|
| C-01 | Sim Engine Core | T3-SimEngine | 커스텀 경량 물리 엔진, 차동구동 로봇, 2D LiDAR, gRPC 스트리밍 |
| C-02 | 3D Viewer Core | T1-Frontend | Three.js 씬, glTF 모델 로드, 카메라 컨트롤, 실시간 포즈 업데이트 |
| C-03 | Backend Core | T2-Backend | REST API (auth, robot CRUD), WebSocket 릴레이, PostgreSQL, gRPC 클라이언트 |
| C-04 | Map Core | T5-MapManager | 포인트 클라우드 업로드/처리/Potree 변환, 로드맵 그래프 CRUD, 타일 서빙 |
| C-05 | Asset Core | T4-AssetManager | 에셋 업로드/다운로드, 메타데이터 CRUD, URDF→glTF 변환 |
| C-06 | Telemetry Pipeline | T2+T1+T3 (크로스팀) | Sim→Backend→Frontend E2E 실시간 텔레메트리 흐름 |

### 고도화 기능 (Advanced)

| ID | 기능명 | 주 담당팀 | 설명 |
|----|--------|-----------|------|
| A-01 | Mission System | T2-Backend | 미션 CRUD, 상태머신, VDA5050 오더 생성, 경로 계획 |
| A-02 | Traffic Management | T2-Backend | 존 잠금, 충돌 감지, 우선순위 큐, 데드락 방지 |
| A-03 | Remote Control | T1-Frontend + T3-SimEngine | 조이스틱/키보드 원격 조종, WebRTC 카메라 피드 |
| A-04 | VDA5050 Real Robot | T2-Backend | MQTT 실로봇 통신, VDA5050 v2.0 브릿지 |
| A-05 | Advanced Simulation | T3-SimEngine | MuJoCo 백엔드, Isaac Sim 브릿지, 3D LiDAR, 카메라 센서, IMU |
| A-06 | Map Editor | T1-Frontend + T5-MapManager | 시맨틱 영역 편집, 장애물 배치, 언두/리두 |
| A-07 | Plugin System | T2-Backend | WASM 런타임, HTTP 웹훅, ROS 2 브릿지 플러그인 |
| A-08 | Dashboard & Analytics | T1-Frontend + T2-Backend | 플릿 KPI, 미션 분석, 히트맵 |
| A-09 | OpenUSD Pipeline | T4-AssetManager + T3-SimEngine | 전체 USD 워크플로우, USD↔SDF/URDF 양방향 변환, UsdPhysics |
| A-10 | Digital Twin | T1+T2+T3 (크로스팀) | 하이브리드 모드(실제+시뮬 오버레이), 디지털 트윈 예측 |

---

## 2. 기능 의존성 그래프

```
C-03 Backend Core ─────┐
                       ├──→ C-06 Telemetry Pipeline ──→ A-01 Mission System
C-01 Sim Engine Core ──┘         │                           │
                                 │                           ├──→ A-02 Traffic Management
C-02 3D Viewer Core ─────────────┘                           │
                                                             ├──→ A-03 Remote Control
C-04 Map Core ──→ A-06 Map Editor                           │
                                                             └──→ A-04 VDA5050 Real Robot
C-05 Asset Core ──→ A-09 OpenUSD Pipeline
                                                  A-01 + A-04 ──→ A-10 Digital Twin
C-01 Sim Engine Core ──→ A-05 Advanced Simulation
                                                  A-01 + A-02 ──→ A-08 Dashboard & Analytics
C-03 Backend Core ──→ A-07 Plugin System
```

### 핵심 의존성 규칙

- **C-06** (Telemetry Pipeline)은 C-01, C-02, C-03이 모두 필요 (E2E 통합)
- **A-01** (Mission)은 C-06 완료 후 착수 (텔레메트리 기반 미션 상태 추적)
- **A-02** (Traffic)는 A-01 의존 (미션 기반 트래픽 제어)
- **A-04** (VDA5050 Real Robot)는 A-01 의존 (미션 오더 발행 필요)
- **A-10** (Digital Twin)은 A-01 + A-04 완료 후 착수 (실+시뮬 통합)
- **A-05**, **A-06**, **A-07**은 각각의 Core 기능만 의존하므로 독립 착수 가능

---

## 3. Phase별 개발 계획

### Phase 1: Prototype (Week 1-8)

**목표:** 핵심 기능(C-01~C-06) 전체 구현. 시뮬 로봇이 맵 위에서 주행하고 브라우저에서 실시간 3D 시각화.

| 주차 | T1-Frontend | T2-Backend | T3-SimEngine | T4-AssetManager | T5-MapManager |
|------|-------------|------------|--------------|-----------------|---------------|
| W1-2 | 프로젝트 셋업, React 스캐폴딩, Three.js 기본 씬 | 프로젝트 셋업, Axum 스캐폴딩, DB 마이그레이션 | 프로젝트 셋업, CMake, 물리 루프 기본 구조 | 프로젝트 셋업, gRPC 서비스 스캐폴딩 | 프로젝트 셋업, gRPC 서비스 스캐폴딩 |
| W3-4 | **C-02** glTF 로더, 카메라 컨트롤, 포즈 렌더링 | **C-03** REST API (auth, robot CRUD), gRPC 클라이언트 | **C-01** 차동구동 모델, 2D LiDAR 시뮬 | **C-05** 업로드/다운로드 API, 메타데이터 CRUD | **C-04** 포인트 클라우드 업로드, Open3D 처리 |
| W5-6 | **C-02** WebSocket 클라이언트, 실시간 포즈 업데이트 | **C-03** WebSocket 릴레이, Sim gRPC 연결 | **C-01** gRPC 텔레메트리 스트리밍 | **C-05** URDF→glTF 변환 파이프라인 | **C-04** Potree 변환, 로드맵 그래프 CRUD |
| W7-8 | **C-06** E2E 통합, 맵 타일 로딩, 통합 테스트 | **C-06** E2E 텔레메트리 릴레이 통합 | **C-06** 시뮬→백엔드 스트리밍 통합 | 통합 테스트, 에셋 서빙 최적화 | 타일 서빙 최적화, 통합 테스트 |

**크로스팀 싱크 포인트:**
- **W2 끝:** Protobuf 스키마 확정 (`proto/` 리뷰)
- **W4 끝:** 각 팀 gRPC 서비스 스텁 연동 확인
- **W6 끝:** Backend↔SimEngine 텔레메트리 스트리밍 검증
- **W8 끝:** E2E 데모 — 시뮬 로봇 주행 + 브라우저 3D 시각화

### Phase 2: Mission & Control (Week 9-16)

**목표:** 미션 생성→로봇 할당→시뮬 실행→모니터링 전체 흐름 완성.

#### Phase 1 Carryover (Week 9-10)

The following items from Phase 1 were completed as scaffolds and need MinIO integration during Phase 2:

| ID | Item | Integrate With | Owner |
|----|------|---------------|-------|
| C-04 | Potree tile serving with MinIO (actual point cloud upload/process/serve) | A-06 Map Editor | T5-MapManager |
| C-05 | Asset upload/download with MinIO (actual file storage flow) | A-09 OpenUSD Pipeline | T4-AssetManager |

These should be completed in Week 9-10 alongside A-01/A-02 setup work.

| 주차 | T1-Frontend | T2-Backend | T3-SimEngine | T4-AssetManager | T5-MapManager |
|------|-------------|------------|--------------|-----------------|---------------|
| W9-10 | 미션 UI 스캐폴딩, 맵 에디터 기본 UI | **A-01** 미션 CRUD API, 상태머신 + MinIO proxy wiring (C-04/C-05 carryover) | 다중 로봇 시뮬 지원 | 에셋 버전 관리 + **C-05 MinIO integration** | **A-06** 시맨틱 영역 API + **C-04 Potree MinIO integration** |
| W11-12 | **A-03** 원격 조종 UI (키보드/조이스틱) | **A-01** VDA5050 오더 생성, 경로 계획 | **A-03** 원격 명령 수신, 카메라 시뮬 | 에셋 검색/필터링 | **A-06** 장애물 배치, 편집 API |
| W13-14 | **A-06** 맵 에디터 프론트 (편집 도구, 언두/리두) | **A-02** 트래픽 관리 (존 잠금, 충돌 감지) | WebRTC 카메라 피드 스트리밍 | 에셋 프리뷰 생성 | 맵 에디터 백엔드 통합 |
| W15-16 | 미션 모니터링 대시보드, 통합 테스트 | **A-02** 우선순위 큐, 데드락 방지 | 트래픽 시나리오 테스트 | 통합 테스트 | 통합 테스트 |

**크로스팀 싱크 포인트:**
- **W10 끝:** 미션 API 스키마 확정, Frontend↔Backend 미션 흐름 검증
- **W12 끝:** 원격 조종 E2E (Frontend→Backend→SimEngine) 동작 확인
- **W14 끝:** 맵 에디터 Frontend↔MapManager 통합 검증
- **W16 끝:** E2E 데모 — 미션 생성→할당→시뮬 실행→모니터링

### Phase 3: Real Robot & Advanced Sim (Week 17-22)

**목표:** 실로봇+시뮬 동일 인터페이스 운용. 고급 시뮬레이션 백엔드 통합.

| 주차 | T1-Frontend | T2-Backend | T3-SimEngine | T4-AssetManager | T5-MapManager |
|------|-------------|------------|--------------|-----------------|---------------|
| W17-18 | 실로봇/시뮬 통합 뷰 UI | **A-04** MQTT VDA5050 브릿지 | **A-05** MuJoCo 백엔드 통합 | **A-09** USD↔URDF/SDF 양방향 변환 | 실환경 맵 연동 |
| W19-20 | 실로봇 텔레메트리 표시 | **A-04** 실로봇 상태 관리 | **A-05** 3D LiDAR, 카메라 센서, IMU | **A-09** UsdPhysics 스키마 지원 | 실로봇 경로 맵 오버레이 |
| W21-22 | 시뮬+실 오버레이 뷰 | 실로봇 통합 테스트 | Isaac Sim 브릿지 (실험적) | USD 워크플로우 통합 테스트 | 통합 테스트 |

**크로스팀 싱크 포인트:**
- **W18 끝:** VDA5050 MQTT 통신 검증 (Backend↔실로봇 또는 모의 클라이언트)
- **W20 끝:** MuJoCo 백엔드 E2E 동작 확인
- **W22 끝:** E2E 데모 — 실로봇+시뮬 동일 인터페이스 운용

### Phase 4: Production (Week 23-30)

**목표:** 프로덕션 레디 시스템. 플러그인, 분석, 디지털 트윈, 배포 자동화, 보안 강화.

| 주차 | T1-Frontend | T2-Backend | T3-SimEngine | T4-AssetManager | T5-MapManager |
|------|-------------|------------|--------------|-----------------|---------------|
| W23-24 | **A-08** 플릿 KPI 대시보드 | **A-07** WASM 플러그인 런타임 | 시뮬 성능 최적화 | 에셋 CDN 캐싱 | 맵 데이터 최적화 |
| W25-26 | **A-08** 미션 분석, 히트맵 | **A-07** HTTP 웹훅, ROS 2 브릿지 | **A-10** 디지털 트윈 시뮬 | 에셋 마켓플레이스 UI | 대규모 맵 성능 테스트 |
| W27-28 | **A-10** 디지털 트윈 오버레이 UI | **A-10** 하이브리드 모드 API | 예측 시뮬레이션 | 배포 자동화 | 배포 자동화 |
| W29-30 | E2E 테스트, 보안 감사, 문서화 | RBAC 강화, 배포 자동화, 보안 감사 | 부하 테스트, 문서화 | 문서화 | 문서화 |

**크로스팀 싱크 포인트:**
- **W24 끝:** 플러그인 아키텍처 통합 검증
- **W26 끝:** 대시보드 데이터 파이프라인 E2E 검증
- **W28 끝:** 디지털 트윈 E2E 데모
- **W30 끝:** 프로덕션 배포 리허설, 전체 시스템 부하 테스트

---

## 4. 타임라인 (Gantt Chart)

```
Feature              W1  W2  W3  W4  W5  W6  W7  W8 │ W9  W10 W11 W12 W13 W14 W15 W16 │ W17 W18 W19 W20 W21 W22 │ W23 W24 W25 W26 W27 W28 W29 W30
                     ── Phase 1: Prototype ──────────│── Phase 2: Mission & Control ───│── Phase 3: Real+AdvSim ─│── Phase 4: Production ──────────
C-01 SimEngine Core  ████████████████████████         │                                 │                         │
C-02 3D Viewer Core      ████████████████████         │                                 │                         │
C-03 Backend Core        ████████████████████         │                                 │                         │
C-04 Map Core            ████████████████████         │                                 │                         │
C-05 Asset Core          ████████████████████         │                                 │                         │
C-06 Telemetry           ············████████         │                                 │                         │
                                             ◆ Demo1 │                                 │                         │
A-01 Mission System                          │       ████████████████████              │                         │
A-02 Traffic Mgmt                            │                   ████████████          │                         │
A-03 Remote Control                          │           ████████████                  │                         │
A-06 Map Editor                              │       ████████████████████              │                         │
                                             │                           ◆ Demo2      │                         │
A-04 VDA5050 Real                            │                                        ████████████              │
A-05 Advanced Sim                            │                                        ████████████              │
A-09 OpenUSD Pipeline                        │                                        ████████████              │
                                             │                                                    ◆ Demo3      │
A-07 Plugin System                           │                                                    │ ████████████
A-08 Dashboard                               │                                                    │ ████████████████
A-10 Digital Twin                            │                                                    │     ████████████████
                                             │                                                    │                   ◆ Release

◆ = 마일스톤    ████ = 개발 기간    ···· = 의존성 대기 (선행 작업 병행)
```

---

## 5. Phase별 Definition of Done

### Phase 1: Prototype (W8)

- [ ] Sim Engine이 차동구동 로봇 1대를 물리 시뮬레이션하고 gRPC로 텔레메트리 스트리밍
- [ ] Backend가 Sim Engine 텔레메트리를 수신하여 WebSocket으로 Frontend에 릴레이
- [ ] Frontend 3D Viewer에서 glTF 로봇 모델이 실시간 포즈 업데이트로 움직임
- [ ] Map Manager가 포인트 클라우드를 Potree로 변환하고 Frontend에서 LOD 로딩
- [ ] Asset Manager가 URDF를 glTF로 변환하여 Frontend에서 렌더링
- [ ] E2E 데모 영상 / 라이브 데모 수행

### Phase 2: Mission & Control (W16)

- [ ] 미션 CRUD → 로봇 할당 → 시뮬 실행 → 상태 모니터링 전체 흐름 동작
- [ ] 트래픽 관리: 다중 로봇 존 잠금, 기본 충돌 회피 동작
- [ ] 원격 조종: 키보드/조이스틱으로 시뮬 로봇 실시간 제어, 카메라 피드 확인
- [ ] 맵 에디터: 시맨틱 영역/장애물 편집, 언두/리두 동작
- [ ] E2E 데모: 미션 생성→할당→시뮬 실행→모니터링 시나리오

### Phase 3: Real Robot & Advanced Sim (W22)

- [ ] VDA5050 MQTT를 통해 실로봇(또는 VDA5050 모의 클라이언트)과 통신
- [ ] 실로봇과 시뮬 로봇을 동일 인터페이스로 관제 (UI 구분 없이 운용 가능)
- [ ] MuJoCo 물리 백엔드로 시뮬레이션 실행 가능
- [ ] OpenUSD 파이프라인: URDF↔USD 양방향 변환 동작
- [ ] E2E 데모: 실로봇+시뮬 동시 운용

### Phase 4: Production (W30)

- [ ] WASM 플러그인 로드/실행 동작, HTTP 웹훅 연동 동작
- [ ] 대시보드: 플릿 KPI, 미션 분석, 히트맵 시각화
- [ ] 디지털 트윈: 실+시뮬 오버레이 뷰, 예측 시뮬레이션 동작
- [ ] Docker Compose / K8s 배포 스크립트 완비
- [ ] RBAC, API rate limiting, 감사 로그 등 보안 기본 요소 적용
- [ ] 전체 시스템 부하 테스트 통과 (로봇 50대 동시 시뮬 기준)
- [ ] 사용자 가이드 및 API 문서 완비

---

## 6. 참조 문서

| 문서 | 설명 |
|------|------|
| [`docs/architecture.md`](architecture.md) | 전체 아키텍처 개요 |
| [`docs/features/core/C-*.md`](features/core/) | 핵심 기능 상세 스펙 |
| [`docs/features/advanced/A-*.md`](features/advanced/) | 고도화 기능 상세 스펙 |
| [`docs/integration/integration-spec.md`](integration/integration-spec.md) | 크로스팀 인터페이스 계약 |
| [`docs/teams/team*.md`](teams/) | 팀별 기술스택 및 구현 가이드 |
