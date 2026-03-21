# A-03: 원격 제어 (Remote Control)

## 1. 개요

### 1.1 목표
브라우저에서 가상 조이스틱 또는 키보드(WASD)로 로봇을 원격 조종하고,
WebRTC를 통해 로봇 카메라 피드를 실시간 스트리밍한다.

### 1.2 담당팀

| 팀 | 역할 |
|----|------|
| **Team 1 (Frontend)** | 조이스틱 UI, 키보드 입력, WebRTC 비디오 뷰어, 긴급정지 버튼 |
| **Team 2 (Backend)** | WebSocket 속도 명령 중계, WebRTC 시그널링 서버 |

### 1.3 Phase: **2**

### 1.4 선행조건
- C-02: 3D 뷰어 기본 구현
- C-03: WebSocket 텔레메트리 연동
- C-06: 로봇 상태 모니터링 UI

---

## 2. 스코프

### In-Scope
- 가상 조이스틱 (nipplejs 라이브러리)
- WASD 키보드 제어
- 속도 명령(linear/angular velocity) WebSocket 전송
- WebRTC 시그널링 서버 (Backend)
- 카메라 피드 비디오 표시
- 긴급정지(E-Stop) 버튼
- 연결 상태 표시 (connected/reconnecting/disconnected)

### Out-of-Scope
- 자율주행 모드 전환 (→ Phase 3)
- 다중 카메라 전환 (→ Phase 3)
- 모바일 앱 원격 제어 (→ Phase 4)

---

## 3. 상세 스펙

### 3.1 속도 명령 프로토콜

```typescript
// WebSocket 메시지 (Frontend → Backend → Robot)
interface VelocityCommand {
  type: "velocity_command";
  robot_id: string;
  linear_x: number;   // m/s, range: [-1.0, 1.0]
  linear_y: number;    // m/s, range: [-1.0, 1.0] (omni-directional)
  angular_z: number;   // rad/s, range: [-1.5, 1.5]
  timestamp: number;   // Unix ms
}
```

- 전송 주기: 10Hz (100ms interval)
- 명령 미수신 500ms 초과 시 자동 정지 (Backend watchdog)
- Backend는 수신한 명령을 VDA5050 instantAction 또는 시뮬레이션 gRPC로 전달

### 3.2 키보드 매핑

| 키 | 동작 | 값 |
|----|------|-----|
| W | 전진 | linear_x = +0.5 |
| S | 후진 | linear_x = -0.5 |
| A | 좌회전 | angular_z = +0.8 |
| D | 우회전 | angular_z = -0.8 |
| Shift + W/S | 고속 전진/후진 | linear_x = ±1.0 |
| Space | 긴급정지 | linear_x/y = 0, angular_z = 0 |

### 3.3 가상 조이스틱

```typescript
// nipplejs 설정
const joystick = nipplejs.create({
  zone: joystickContainer,
  mode: 'static',
  position: { left: '80px', bottom: '80px' },
  size: 120,
  threshold: 0.1,
});

// distance → linear velocity, angle → angular velocity 변환
joystick.on('move', (evt, data) => {
  const linear_x = Math.cos(data.angle.radian) * data.force * maxLinearVel;
  const angular_z = Math.sin(data.angle.radian) * data.force * maxAngularVel;
  sendVelocityCommand({ linear_x, linear_y: 0, angular_z });
});
```

### 3.4 WebRTC 카메라 피드

```
[Robot/Sim Camera] → STUN/TURN → [Browser <video>]
                         ↕
              [Backend Signaling Server]
              (WebSocket /ws/webrtc/signal)
```

- 시그널링: SDP offer/answer + ICE candidate 교환 (WebSocket 경유)
- 코덱: H.264 Baseline (호환성) 또는 VP8
- 해상도: 640x480 @ 15fps (기본), 1280x720 @ 30fps (고화질)
- TURN 서버: coturn (NAT 환경 대응)

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#remote-control) 참조

### 3.5 긴급정지 (E-Stop)

- UI: 빨간 원형 버튼, 항상 화면 우상단 고정
- 동작: 즉시 `linear_x/y = 0, angular_z = 0` 전송 + VDA5050 instantAction `stopMovement`
- 키보드: Space 키 바인딩
- E-Stop 후 해제(resume) 버튼 표시 → 확인 클릭 시 제어 재개

### 3.6 연결 상태

| 상태 | UI 표시 | 조건 |
|------|---------|------|
| Connected | 🟢 초록 인디케이터 | WebSocket 연결 정상 |
| Reconnecting | 🟡 노랑 + 스피너 | WebSocket 재연결 시도 중 (최대 5회) |
| Disconnected | 🔴 빨강 + 경고 | 연결 실패, 제어 비활성화 |

---

## 4. 구현 모듈

| 모듈 | 팀 | 경로 |
|------|-----|------|
| `RemoteControlPanel.tsx` | Team 1 | `frontend/src/components/control/RemoteControlPanel.tsx` |
| `VirtualJoystick.tsx` | Team 1 | `frontend/src/components/control/VirtualJoystick.tsx` |
| `useKeyboardControl.ts` | Team 1 | `frontend/src/hooks/useKeyboardControl.ts` |
| `WebRTCViewer.tsx` | Team 1 | `frontend/src/components/control/WebRTCViewer.tsx` |
| `webrtc_signaling.rs` | Team 2 | `backend/src/handlers/webrtc_signaling.rs` |
| `velocity_relay.rs` | Team 2 | `backend/src/handlers/velocity_relay.rs` |

---

## 5. 의존성

| 항목 | 종류 | 비고 |
|------|------|------|
| nipplejs | npm 패키지 | 가상 조이스틱 라이브러리 |
| coturn | 인프라 | TURN 서버 (WebRTC NAT traversal) |
| WebSocket relay | 서비스 | 속도 명령 중계 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#remote-control) 참조

---

## 6. 테스트 기준

| 테스트 유형 | 기준 |
|------------|------|
| 단위 테스트 | 키보드→속도 변환, 조이스틱→속도 변환 정확성 |
| 통합 테스트 | 명령 전송 → Backend 중계 → 로봇/시뮬 수신 E2E |
| WebRTC 테스트 | 시그널링 → 연결 수립 → 비디오 수신 확인 |
| Watchdog 테스트 | 명령 중단 시 500ms 내 자동 정지 |
| UX 테스트 | 조이스틱/키보드 응답 지연 < 100ms (체감) |

---

## 7. 완료 조건

- [ ] 가상 조이스틱으로 시뮬레이션 로봇 조종 가능
- [ ] WASD 키보드로 속도 제어 동작 확인
- [ ] WebRTC 카메라 피드 640x480@15fps 이상 표시
- [ ] 긴급정지 즉시 동작 및 해제 확인
- [ ] 연결 상태 UI 정확한 반영
- [ ] 명령 미수신 시 자동 정지(watchdog) 동작
