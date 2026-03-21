# A-04: VDA5050 실로봇 통신 (VDA5050 Real Robot Communication)

## 1. 개요

### 1.1 목표
MQTT를 통한 VDA5050 v2.0 표준 프로토콜로 실제 로봇과 통신한다.
Order, State, Connection, Visualization 메시지를 처리하며,
로봇 온/오프라인 감지 및 텔레메트리 수집을 구현한다.

### 1.2 담당팀

| 팀 | 역할 |
|----|------|
| **Team 2 (Backend)** | MQTT 클라이언트, VDA5050 메시지 처리, 로봇 상태 관리 전체 |

### 1.3 Phase: **3**

### 1.4 선행조건
- A-01: 미션 시스템 (VDA5050 Order 생성)

---

## 2. 스코프

### In-Scope
- MQTT 클라이언트 (rumqttc, TLS 지원)
- VDA5050 v2.0 토픽 구조 구현
- Order 메시지 발행 (Backend → Robot)
- State 메시지 구독 및 파싱 (Robot → Backend)
- Connection 메시지 기반 온/오프라인 감지
- Visualization 메시지 수신 (로봇 경로 예측 등)
- 실로봇 텔레메트리를 시뮬레이션과 동일한 TelemetryMessage 포맷으로 변환

### Out-of-Scope
- VDA5050 v3.0 지원 (→ 향후)
- 로봇 펌웨어 업데이트 (→ Phase 4)
- 다중 MQTT 브로커 클러스터링 (→ Phase 4)

---

## 3. 상세 스펙

### 3.1 VDA5050 토픽 구조

```
{interface_name}/{major_version}/{manufacturer}/{serial_number}/{topic}

예시:
uagv/v2/amr-corp/robot-001/order         ← Backend 발행 (로봇 구독)
uagv/v2/amr-corp/robot-001/instantActions ← Backend 발행
uagv/v2/amr-corp/robot-001/state         ← 로봇 발행 (Backend 구독)
uagv/v2/amr-corp/robot-001/connection    ← 로봇 발행 (LWT)
uagv/v2/amr-corp/robot-001/visualization ← 로봇 발행
```

### 3.2 MQTT 클라이언트 설정

```rust
use rumqttc::{AsyncClient, MqttOptions, QoS, Transport};

let mut mqtt_opts = MqttOptions::new("amr-backend", &config.mqtt.host, config.mqtt.port);
mqtt_opts.set_keep_alive(Duration::from_secs(30));
mqtt_opts.set_clean_session(true);
mqtt_opts.set_transport(Transport::tls_with_config(tls_config)); // TLS 필수

// QoS 설정
// Order, instantActions: QoS 1 (at least once)
// State, Connection: QoS 1
// Visualization: QoS 0 (best effort)
```

### 3.3 State 메시지 처리

```rust
pub struct Vda5050StateHandler {
    pub async fn handle_state(&self, state: Vda5050State) -> Result<()> {
        // 1. 로봇 위치(agvPosition) → DB 업데이트
        // 2. 배터리 상태(batteryState) → DB 업데이트
        // 3. 주문 상태(orderState) → 미션 상태머신 전이 트리거
        // 4. 에러(errors) → 알림 생성
        // 5. TelemetryMessage로 변환 → WebSocket 브로드캐스트
    }
}
```

State에서 추출한 데이터를 `TelemetryMessage` 포맷으로 변환하여
시뮬레이션 로봇과 동일한 파이프라인으로 처리한다.

### 3.4 Connection 메시지 (온/오프라인)

```json
{
  "headerId": 1,
  "timestamp": "2026-03-22T10:00:00.000Z",
  "version": "2.0.0",
  "manufacturer": "amr-corp",
  "serialNumber": "robot-001",
  "connectionState": "ONLINE"  // ONLINE | OFFLINE | CONNECTIONBROKEN
}
```

- MQTT Last Will and Testament (LWT) 활용: 로봇 연결 끊김 시 자동 OFFLINE 발행
- OFFLINE 감지 시: 해당 로봇의 존 잠금 해제, 진행 중 미션 FAILED 처리, Frontend 알림

### 3.5 텔레메트리 통합

```rust
impl From<Vda5050State> for TelemetryMessage {
    fn from(state: Vda5050State) -> Self {
        TelemetryMessage {
            robot_id: format!("{}:{}", state.manufacturer, state.serial_number),
            timestamp: state.timestamp,
            pose: Pose {
                x: state.agv_position.x,
                y: state.agv_position.y,
                theta: state.agv_position.theta,
            },
            battery_percent: state.battery_state.battery_charge,
            velocity: state.agv_velocity.into(),
            // ... 동일 포맷
        }
    }
}
```

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#vda5050-mqtt) 참조

---

## 4. 구현 모듈

| 모듈 | 팀 | 경로 |
|------|-----|------|
| `mqtt_client.rs` | Team 2 | `backend/src/infrastructure/mqtt_client.rs` |
| `vda5050_handler.rs` | Team 2 | `backend/src/handlers/vda5050_handler.rs` |
| `vda5050_types.rs` | Team 2 | `backend/src/models/vda5050.rs` |
| `connection_monitor.rs` | Team 2 | `backend/src/services/connection_monitor.rs` |
| `telemetry_converter.rs` | Team 2 | `backend/src/services/telemetry_converter.rs` |

---

## 5. 의존성

| 항목 | 종류 | 비고 |
|------|------|------|
| rumqttc 0.24+ | 크레이트 | Async MQTT 클라이언트 |
| Eclipse Mosquitto 2.0+ | 인프라 | MQTT 브로커 |
| TLS 인증서 | 보안 | 클라이언트/서버 인증서 |
| Mission Service | 서비스 | State → 미션 상태 전이 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#vda5050-messages) 참조

---

## 6. 테스트 기준

| 테스트 유형 | 기준 |
|------------|------|
| 단위 테스트 | VDA5050 JSON 파싱/직렬화 정확성 (모든 메시지 타입) |
| 통합 테스트 | Order 발행 → State 수신 → 미션 상태 전이 E2E |
| 연결 테스트 | LWT 기반 오프라인 감지 → 잠금 해제 → 알림 생성 |
| 호환성 테스트 | VDA5050 v2.0 공식 JSON 스키마 검증 통과 |
| 부하 테스트 | 100대 로봇 State 10Hz 동시 수신 (1,000 msg/s) |

---

## 7. 완료 조건

- [ ] MQTT TLS 연결 및 VDA5050 토픽 구독/발행 동작
- [ ] Order 메시지가 VDA5050 v2.0 스키마 준수
- [ ] State 메시지 파싱 및 TelemetryMessage 변환 정상
- [ ] Connection LWT 기반 온/오프라인 감지 정확
- [ ] 실로봇과 시뮬레이션 로봇이 동일한 텔레메트리 파이프라인 사용
- [ ] 100대 로봇 동시 통신 부하 테스트 통과
