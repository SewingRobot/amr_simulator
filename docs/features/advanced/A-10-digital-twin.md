# A-10: 디지털 트윈 (Digital Twin)

## 1. 개요

### 1.1 목표
실제 로봇과 시뮬레이션 로봇을 동일 뷰에서 오버레이하는 하이브리드 모드를 구현하고,
시뮬레이션이 실제 환경보다 약간 앞서 실행되어 근미래 예측을 제공하며,
신규 로봇 커미셔닝 워크플로우(시뮬 → 실제 전환)를 지원한다.

### 1.2 담당팀

| 팀 | 역할 |
|----|------|
| **Team 1 (Frontend)** | 하이브리드 뷰 UI, 뷰 모드 토글, 예측 궤적 시각화 |
| **Team 2 (Backend)** | 듀얼 데이터 소스 관리, 예측 요청 API, 커미셔닝 워크플로우 |
| **Team 3 (Sim Engine)** | 실시간 환경 미러링, 근미래 시뮬레이션 실행 |

### 1.3 Phase: **4**

### 1.4 선행조건
- A-04: VDA5050 실로봇 통신 (실제 로봇 데이터 소스)
- A-05: 고급 시뮬레이션 (정밀 시뮬레이션 능력)
- C-02: 3D 뷰어 (시각화 기반)

---

## 2. 스코프

### In-Scope
- 듀얼 데이터 소스 아키텍처 (실제 + 시뮬레이션 텔레메트리)
- 뷰 모드 토글 (Real-Only, Sim-Only, Hybrid)
- 시뮬레이션 인스턴스가 실제 환경을 미러링
- 근미래 예측 (시뮬레이션이 5-30초 앞서 실행)
- 신규 로봇 커미셔닝 워크플로우 (시뮬 검증 → 실제 전환)
- 실제 vs 시뮬 위치 차이 시각화

### Out-of-Scope
- 과거 시나리오 리플레이 기반 what-if 분석 (→ 향후)
- 멀티사이트 디지털 트윈 (→ 향후)
- AR/VR 디지털 트윈 뷰 (→ 향후)

---

## 3. 상세 스펙

### 3.1 듀얼 데이터 소스

```rust
pub struct DualTelemetryManager {
    real_source: RealRobotTelemetry,    // VDA5050 MQTT → TelemetryMessage
    sim_source: SimTelemetry,            // Sim Engine gRPC → TelemetryMessage
    mode: TwinMode,
}

pub enum TwinMode {
    RealOnly,     // 실제 로봇 데이터만 표시
    SimOnly,      // 시뮬레이션 데이터만 표시
    Hybrid,       // 양쪽 모두 표시 (오버레이)
}
```

- 동일한 `robot_id`에 대해 실제/시뮬 데이터를 별도 채널로 관리
- WebSocket을 통해 Frontend에 모드에 따른 데이터 전달
- Hybrid 모드: 두 데이터 스트림을 동시 전송, Frontend에서 오버레이

### 3.2 뷰 모드 토글

```typescript
// Frontend 뷰 모드 UI
interface DigitalTwinControls {
  viewMode: 'real' | 'sim' | 'hybrid';
  showPrediction: boolean;
  predictionHorizon: number;  // seconds (5, 10, 15, 30)
  showDeviation: boolean;     // 실제 vs 시뮬 위치 차이 표시
}
```

- 3D 뷰어 툴바에 모드 토글 버튼
- Hybrid 모드 시각적 구분:
  - 실제 로봇: 불투명 렌더링 (100% opacity)
  - 시뮬 로봇: 반투명 렌더링 (40% opacity, 파란색 틴트)
  - 편차 라인: 실제↔시뮬 위치 간 점선 표시 (빨간색, 편차 > 0.5m 시)

### 3.3 환경 미러링

```protobuf
// Sim Engine gRPC - 디지털 트윈 전용
rpc SyncRobotState(SyncRequest) returns (SyncResponse);
message SyncRequest {
    string robot_id = 1;
    Pose real_pose = 2;           // 실제 로봇 현재 위치
    Velocity real_velocity = 3;   // 실제 로봇 현재 속도
    double timestamp = 4;
}
```

- Backend가 실제 로봇 State 수신 시 동일 상태를 Sim Engine에 동기화
- Sim Engine은 동기화된 상태에서 시뮬레이션 계속 실행
- 동기화 주기: 실제 로봇 State 수신 주기와 동일 (10Hz)

### 3.4 근미래 예측

```
[현재 시점]──────[+5s]──────[+10s]──────[+15s]──────[+30s]
   실제 로봇 위치     시뮬 예측 궤적 (점선 표시)
```

```rust
pub struct PredictionRequest {
    pub robot_id: String,
    pub horizon_secs: f64,        // 예측 시간 범위
    pub current_mission: Option<String>, // 현재 미션 ID
}

pub struct PredictionResponse {
    pub trajectory: Vec<PredictedPose>,  // 예측 궤적
    pub confidence: f64,                  // 신뢰도 (0.0-1.0)
    pub predicted_events: Vec<PredictedEvent>, // 예측 이벤트 (충돌, 배터리 부족 등)
}
```

- Sim Engine이 현재 로봇 상태 + 현재 미션 경로 기반으로 미래 궤적 계산
- 예측 궤적을 3D 뷰어에 점선으로 시각화
- 예측 충돌/이벤트 발생 시 사전 경고 알림

### 3.5 신규 로봇 커미셔닝 워크플로우

```
Step 1: [시뮬레이션 생성]
  └─ 신규 로봇 모델을 시뮬레이션에 추가
  └─ 기본 동작 테스트 (이동, 회전, 센서)

Step 2: [미션 검증]
  └─ 시뮬레이션 환경에서 미션 실행
  └─ 경로 계획, 충돌 회피 검증

Step 3: [하이브리드 전환]
  └─ 실제 로봇 연결 (VDA5050)
  └─ Hybrid 모드: 시뮬 + 실제 동시 표시
  └─ 편차 모니터링

Step 4: [실제 전환]
  └─ 시뮬레이션 인스턴스 유지 (디지털 트윈)
  └─ 운영 모드로 전환
```

```typescript
interface CommissioningWorkflow {
  step: 'sim_test' | 'mission_verify' | 'hybrid_validate' | 'production';
  robotId: string;
  simInstanceId: string;
  realRobotConnected: boolean;
  validationChecklist: ValidationItem[];
}
```

### 3.6 편차 모니터링

| 지표 | 경고 임계값 | 위험 임계값 |
|------|-----------|-----------|
| 위치 편차 | > 0.5m | > 2.0m |
| 방향 편차 | > 15° | > 45° |
| 속도 편차 | > 0.3 m/s | > 1.0 m/s |
| 미션 진행률 차이 | > 10% | > 30% |

편차가 위험 임계값 초과 시 시뮬레이션 재동기화 트리거.

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#digital-twin) 참조

---

## 4. 구현 모듈

| 모듈 | 팀 | 경로 |
|------|-----|------|
| `DigitalTwinView.tsx` | Team 1 | `frontend/src/components/twin/DigitalTwinView.tsx` |
| `TwinModeToggle.tsx` | Team 1 | `frontend/src/components/twin/TwinModeToggle.tsx` |
| `PredictionOverlay.tsx` | Team 1 | `frontend/src/components/twin/PredictionOverlay.tsx` |
| `CommissioningWizard.tsx` | Team 1 | `frontend/src/components/twin/CommissioningWizard.tsx` |
| `twin_service.rs` | Team 2 | `backend/src/services/twin_service.rs` |
| `dual_telemetry.rs` | Team 2 | `backend/src/services/dual_telemetry.rs` |
| `prediction_engine.cpp` | Team 3 | `sim-engine/src/twin/prediction_engine.cpp` |
| `state_synchronizer.cpp` | Team 3 | `sim-engine/src/twin/state_synchronizer.cpp` |

---

## 5. 의존성

| 항목 | 종류 | 비고 |
|------|------|------|
| VDA5050 MQTT 통신 | 서비스 | A-04 실로봇 데이터 소스 |
| Sim Engine gRPC | 서비스 | 미러링, 예측 실행 |
| 3D 뷰어 (Three.js) | Frontend | 오버레이 렌더링 |
| WebSocket relay | 서비스 | 듀얼 텔레메트리 전달 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#digital-twin) 참조

---

## 6. 테스트 기준

| 테스트 유형 | 기준 |
|------------|------|
| 동기화 테스트 | 실제 상태 → Sim 동기화 지연 < 200ms |
| 예측 테스트 | 10초 예측 궤적 vs 실제 궤적 편차 < 0.5m (직선 주행) |
| 모드 전환 테스트 | Real/Sim/Hybrid 모드 전환 시 렌더링 끊김 없음 |
| 커미셔닝 테스트 | 4단계 워크플로우 완주 E2E |
| 성능 테스트 | 10대 로봇 Hybrid 모드 30fps 유지 |

---

## 7. 완료 조건

- [ ] 듀얼 데이터 소스로 실제/시뮬 텔레메트리 동시 관리
- [ ] Real/Sim/Hybrid 뷰 모드 토글 동작
- [ ] 시뮬레이션 인스턴스가 실제 환경을 실시간 미러링
- [ ] 근미래 예측 궤적이 3D 뷰어에 시각화
- [ ] 편차 모니터링 및 경고 알림 동작
- [ ] 신규 로봇 커미셔닝 4단계 워크플로우 완성
- [ ] 10대 로봇 Hybrid 모드 안정적 운영
