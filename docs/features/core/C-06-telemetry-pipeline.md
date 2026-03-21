# C-06: Telemetry Pipeline (E2E)

> **Phase**: 1
> **담당 팀**: Team 2 (Backend, 주도) + Team 3 (Sim Engine) + Team 1 (Frontend)
> **상태**: Draft
> **최종 수정일**: 2026-03-22
> **특성**: 크로스팀 통합 기능 — C-01/C-02/C-03의 연동을 검증한다

---

## 1. 개요

Sim Engine에서 생성된 가상 로봇의 텔레메트리 데이터가
Backend를 거쳐 Frontend 3D 뷰어에 실시간으로 반영되는 E2E 파이프라인을 구현한다.
이 기능은 개별 팀(C-01 Frontend, C-02 Sim Engine, C-03 Backend)에서
구현한 컴포넌트의 통합 동작을 검증하는 핵심 연동 기능이다.

**데이터 흐름:**
```
Sim Engine ──gRPC stream──► Backend ──WebSocket──► Frontend (3D Viewer)
  (10Hz)      (정규화)         (팬아웃)       (Zustand → Three.js)
```

**목표 지연시간:** E2E < 100ms (p95)

---

## 2. 스코프

### 2.1 In-Scope

| # | 항목 | 담당 팀 | 설명 |
|---|------|---------|------|
| 1 | gRPC StreamTelemetry | Team 3 | Sim Engine이 텔레메트리 서버 스트림 제공 |
| 2 | gRPC 클라이언트 수신 | Team 2 | Backend가 Sim Engine의 gRPC stream 수신 |
| 3 | TelemetryMessage 정규화 | Team 2 | gRPC 메시지 → 내부 통일 포맷 변환 |
| 4 | WebSocket 팬아웃 | Team 2 | 정규화된 텔레메트리 → 구독 중인 WS 클라이언트에 전달 |
| 5 | WebSocket 수신 | Team 1 | WS 메시지 → Zustand store 업데이트 |
| 6 | 3D 뷰어 포즈 반영 | Team 1 | store 변경 → Three.js 로봇 모델 position/rotation 업데이트 |
| 7 | E2E 지연시간 측정 | 전체 | 각 구간별 latency 측정 및 검증 |
| 8 | 다중 로봇 스트리밍 | 전체 | 10대 로봇 x 10Hz 동시 스트리밍 안정성 검증 |
| 9 | 연결 끊김/재연결 | Team 1+2 | WebSocket 재연결 처리 (exponential backoff) |

### 2.2 Out-of-Scope

| 항목 | 구현 시점 | 비고 |
|------|----------|------|
| TimescaleDB 텔레메트리 저장 | A-08 | 실시간 전달만 구현, DB 저장 없음 |
| Redis pub/sub 팬아웃 | A-01 | 인메모리 broadcast 채널 사용 |
| MQTT 실로봇 텔레메트리 | A-04 | 시뮬레이션 텔레메트리만 대상 |
| 텔레메트리 히스토리 조회 API | A-08 | 저장 기능과 함께 구현 |
| 대시보드 차트/그래프 | A-10 | 3D 포즈 표시만 구현 |

---

## 3. 상세 스펙

### 3.1 데이터 흐름 및 지연시간 목표

```
┌──────────┐    gRPC stream     ┌──────────┐    WebSocket      ┌──────────┐
│          │ ← TelemetryMsg → │          │ ← JSON msg →    │          │
│ Sim      │    < 10ms         │ Backend  │    < 20ms        │ Frontend │
│ Engine   │                   │          │                  │ Browser  │
└──────────┘                   └──────────┘                  └──────────┘
  생성 ─────── 수신/정규화 ─────── WS 전송 ─────── 렌더링
   t0          t0 + 10ms        t0 + 30ms      t0 + 50ms

총 E2E 목표: t0 → 렌더링 완료 < 100ms (p95)
```

| 구간 | 지연시간 목표 | 측정 방법 |
|------|-------------|----------|
| Sim Engine → Backend (gRPC) | < 10ms | gRPC 메시지 내 timestamp 비교 |
| Backend 정규화 처리 | < 5ms | 내부 span 계측 |
| Backend → Frontend (WebSocket) | < 20ms | WS 메시지 timestamp 비교 |
| Frontend store 업데이트 + 렌더링 | < 30ms | requestAnimationFrame 기준 |
| **총 E2E** | **< 100ms (p95)** | Sim timestamp → 렌더링 완료 |

### 3.2 TelemetryMessage 통일 포맷

시뮬레이션과 실로봇 모두 동일한 구조를 사용한다.

```rust
pub struct TelemetryMessage {
    pub robot_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
    pub orientation_theta: f64,       // 라디안
    pub velocity_linear: f64,         // m/s
    pub velocity_angular: f64,        // rad/s
    pub battery_level: f64,           // 0.0 ~ 100.0
    pub battery_charging: bool,
    pub operating_mode: String,       // "automatic", "manual", "charging"
    pub map_id: Option<Uuid>,
    pub errors: Vec<String>,
    pub payload: Option<serde_json::Value>,  // 추가 센서 데이터
}
```

**WebSocket JSON 포맷:** `WsOutgoingMessage { msg_type: "data", topic: "telemetry:{robot_id}", payload: TelemetryMessage, timestamp: unix_ms }`

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#telemetry-message) 참조

### 3.3 WebSocket 팬아웃 메커니즘

**구독 토픽:**
- `telemetry:{robot_id}` — 특정 로봇 텔레메트리
- `telemetry:*` — 모든 로봇 텔레메트리 (robot_id 포함)

**팬아웃 구현 (Phase 1 — 인메모리):**
```
gRPC stream 수신 → tokio::broadcast::channel → WsSession 필터링 → WS 전송
```

- `broadcast::channel` 용량: 1,000 메시지
- 각 WsSession이 자신의 구독 토픽에 매칭되는 메시지만 전송

**Backpressure 처리:**
- WS 클라이언트 전송 버퍼가 가득 차면 가장 오래된 메시지 드롭
- 드롭된 메시지 수를 메트릭으로 기록
- 드롭 발생 시 클라이언트에 `msg_type: "warning"` 알림 (선택적)

### 3.4 연결 관리

**WebSocket 재연결 (Frontend):**
```
연결 끊김 감지 → 1초 대기 → 재연결 시도
실패 → 2초 대기 → 재연결 시도
실패 → 4초 대기 → 재연결 시도
실패 → 8초 대기 → 재연결 시도
...
최대 대기: 30초
성공 시: 이전 구독 토픽 자동 재구독
```

**gRPC 스트림 재연결 (Backend → Sim Engine):**
- 시뮬레이션 세션 시작 시 StreamTelemetry 호출
- 스트림 종료 감지 시 시뮬레이션 상태 확인 후 재연결
- 시뮬레이션 중지 상태면 재연결하지 않음

**하트비트:**
- Backend → Frontend: 30초 간격 WS ping
- 클라이언트 pong 미응답 3회 시 세션 종료

### 3.5 관측성 (Observability)

**수집 메트릭:**

| 메트릭 | 타입 | 설명 |
|--------|------|------|
| `telemetry_messages_received_total` | Counter | gRPC로 수신한 메시지 수 |
| `telemetry_messages_sent_total` | Counter | WS로 전송한 메시지 수 |
| `telemetry_messages_dropped_total` | Counter | backpressure로 드롭된 메시지 수 |
| `telemetry_e2e_latency_ms` | Histogram | E2E 지연시간 (p50, p95, p99) |
| `telemetry_grpc_latency_ms` | Histogram | gRPC 구간 지연시간 |
| `telemetry_ws_latency_ms` | Histogram | WebSocket 구간 지연시간 |
| `ws_active_sessions` | Gauge | 활성 WebSocket 세션 수 |
| `ws_subscriptions_count` | Gauge | 전체 구독 수 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#telemetry-data) 참조

---

## 4. 구현 모듈

### 4.1 Sim Engine (Team 3)

| 모듈 | 설명 |
|------|------|
| `StreamTelemetry` RPC 서버 | 시뮬레이션 중인 로봇의 텔레메트리를 10Hz로 스트리밍 |
| 텔레메트리 생성기 | 물리 엔진 상태 → TelemetryMessage 변환 |

### 4.2 Backend (Team 2)

| 모듈 | 파일 경로 | 설명 |
|------|----------|------|
| gRPC 텔레메트리 클라이언트 | `src/grpc/sim_client.rs` | StreamTelemetry 호출, 스트림 수신 |
| 텔레메트리 서비스 | `src/services/telemetry_service.rs` | 정규화, broadcast 채널 팬아웃 |
| WebSocket 핸들러 | `src/api/ws/handler.rs` | WS 업그레이드, 구독 관리 |
| WS 세션 매니저 | `src/api/ws/session.rs` | 세션 생명주기, 구독 매칭 |
| WS 메시지 타입 | `src/api/ws/messages.rs` | WsOutgoingMessage, WsIncomingMessage |

### 4.3 Frontend (Team 1)

| 모듈 | 설명 |
|------|------|
| WebSocket 매니저 | 연결/재연결/구독 관리 (exponential backoff) |
| Telemetry Zustand Store | 로봇별 최신 텔레메트리 상태 관리 |
| 3D 포즈 업데이트 | Three.js 로봇 모델의 position/quaternion 업데이트 |

---

## 5. 의존성

### 5.1 인프라 의존성

| 구성 요소 | 용도 | Phase 1 필수 |
|-----------|------|-------------|
| PostgreSQL | Backend 기본 DB (로봇 등록 등) | 예 |
| Docker Compose | 전체 스택 통합 환경 | 예 |

### 5.2 팀 간 의존성

| 의존 대상 | 의존 내용 | 선행 조건 |
|-----------|----------|----------|
| Team 3 (Sim Engine) | StreamTelemetry gRPC 서버 | C-02 기본 시뮬레이션 루프 |
| Team 2 (Backend) | WebSocket relay, gRPC client | C-03 기본 서버 부트스트랩 |
| Team 1 (Frontend) | WebSocket 수신, 3D 렌더링 | C-01 기본 3D 뷰어 |
| Team 6 (Proto) | TelemetryMessage proto 정의 | proto 파일 공유 |

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#telemetry-pipeline) 참조

---

## 6. 테스트 기준

### 6.1 E2E 테스트 시나리오

| # | 시나리오 | 절차 | 성공 기준 |
|---|---------|------|----------|
| 1 | 단일 로봇 텔레메트리 | Sim에서 로봇 1대 spawn → 브라우저에서 포즈 확인 | 3D 뷰어에서 로봇 위치가 실시간 업데이트 |
| 2 | 다중 로봇 텔레메트리 | 10대 로봇 spawn → 브라우저에서 모든 로봇 업데이트 확인 | 10대 모두 3D 뷰어에 표시, 위치 변경 반영 |
| 3 | E2E 지연시간 | Sim timestamp → 렌더링 완료 시간 측정 | p95 < 100ms |
| 4 | WebSocket 재연결 | WS 연결 강제 종료 → 자동 재연결 확인 | 30초 이내 재연결, 이전 구독 복원 |
| 5 | 다중 클라이언트 | 3개 브라우저 탭에서 동시 수신 | 모든 탭이 동일한 텔레메트리 수신 |
| 6 | 장시간 안정성 | 10대 로봇 x 10Hz, 60초 이상 연속 스트리밍 | 메시지 손실 없음, 메모리 누수 없음 |

### 6.2 부하 테스트

| 항목 | 조건 | 목표 |
|------|------|------|
| 처리량 | 10 로봇 x 10Hz = 100 msg/s | 모든 메시지 WS로 전달 |
| 동시 WS 연결 | 10 클라이언트 동시 구독 | 모든 클라이언트 수신 성공 |
| backpressure | 느린 WS 클라이언트 시뮬레이션 | 다른 클라이언트 영향 없음, 느린 클라이언트만 드롭 |

### 6.3 단위 테스트

- TelemetryMessage 정규화: gRPC proto → 내부 구조체 변환 시 모든 필드 매핑 정확
- WS 구독 매칭: `telemetry:*` → 모든 로봇, `telemetry:{id}` → 해당 로봇만
- Backpressure: 버퍼 가득 참 → 오래된 메시지 드롭, 카운터 증가
- 재연결 backoff: 1s → 2s → 4s → 8s → max 30s

---

## 7. 완료 조건

- [ ] Sim Engine에서 생성한 로봇의 포즈가 브라우저 3D 뷰어에 실시간 표시
- [ ] 10대 로봇 x 10Hz 스트리밍 안정적 동작 (60초 이상)
- [ ] E2E 지연시간 < 100ms (p95)
- [ ] WebSocket 재연결 동작 확인 (exponential backoff, 구독 복원)
- [ ] 3개 브라우저 탭 동시 수신 확인
- [ ] TelemetryMessage 포맷이 proto 정의와 일치
- [ ] backpressure 발생 시 느린 클라이언트만 영향 받음
- [ ] 메트릭 수집 동작 (message count, latency percentiles)
- [ ] Docker Compose로 전체 스택 (Sim Engine + Backend + Frontend) 통합 테스트 통과
