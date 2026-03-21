# A-01: 미션 관리 시스템 (Mission Management System)

## 1. 개요

### 1.1 목표
미션 CRUD 및 상태머신(CREATED → ASSIGNED → EXECUTING → COMPLETED/FAILED/CANCELLED)을 구현하고,
VDA5050 Order 메시지를 생성하여 로봇에 전송하며, Map Manager의 A* 경로 계획과 연동한다.

### 1.2 담당팀

| 팀 | 역할 |
|----|------|
| **Team 2 (Backend)** | 미션 서비스, 상태머신, VDA5050 Order 생성, REST API |
| **Team 1 (Frontend)** | 미션 리스트/상세/생성 UI, 실시간 상태 업데이트 |
| **Team 5 (Map Manager)** | A* 경로 탐색 API 제공 |

### 1.3 Phase: **2**

### 1.4 선행조건
- C-03: WebSocket 텔레메트리 연동 완료
- C-04: 로드맵 그래프 데이터 구조 완료

---

## 2. 스코프

### In-Scope
- 미션 CRUD REST API (`POST/GET/PUT/DELETE /api/v1/missions`)
- 미션 상태머신 (6개 상태, 7개 전이)
- VDA5050 v2.0 Order 메시지 생성 및 MQTT 전송
- Map Manager gRPC를 통한 A* 경로 요청
- 미션 리스트/상세/생성 UI (React)
- WebSocket을 통한 실시간 미션 상태 업데이트

### Out-of-Scope
- 멀티로봇 동시 미션 최적화 (→ A-02)
- 미션 스케줄링/자동 할당 (→ Phase 3)
- 미션 이력 분석/대시보드 (→ A-08)

---

## 3. 상세 스펙

### 3.1 미션 상태머신

```
CREATED ──assign──► ASSIGNED ──robot_accept──► EXECUTING
   │                   │                          │
   │ cancel            │ cancel                   ├── complete ──► COMPLETED
   ▼                   ▼                          ├── error ────► FAILED
CANCELLED          CANCELLED                      └── cancel ───► CANCELLED
```

| 현재 상태 | 다음 상태 | 트리거 |
|-----------|----------|--------|
| CREATED | ASSIGNED | `POST /api/v1/missions/{id}/assign` |
| CREATED | CANCELLED | `POST /api/v1/missions/{id}/cancel` |
| ASSIGNED | EXECUTING | VDA5050 State에서 로봇 수락 확인 |
| ASSIGNED | CANCELLED | `POST /api/v1/missions/{id}/cancel` |
| EXECUTING | COMPLETED | VDA5050 State: 모든 노드/엣지 완료 |
| EXECUTING | FAILED | VDA5050 State: 에러 발생 |
| EXECUTING | CANCELLED | `POST /api/v1/missions/{id}/cancel` → instantAction(cancelOrder) |

### 3.2 VDA5050 Order 생성

```rust
pub struct Vda5050OrderBuilder {
    pub fn from_mission(mission: &Mission, path: &[PathNode]) -> Order {
        // 경로의 각 PathNode → VDA5050 Node + Edge 변환
        // orderId = mission.id, orderUpdateId = 0 (초기)
        // nodes[].actions: pick, drop, charge 등 미션 타입별 액션 매핑
    }
}
```

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#vda5050-order) 참조

### 3.3 A* 경로 요청

```protobuf
// Map Manager gRPC
rpc FindPath(FindPathRequest) returns (FindPathResponse);
message FindPathRequest {
  string map_id = 1;
  string start_node_id = 2;
  string goal_node_id = 3;
}
message FindPathResponse {
  repeated PathNode nodes = 1;
  double total_distance = 2;
  double estimated_time = 3;
}
```

### 3.4 Frontend UI

- **미션 리스트**: 테이블 뷰 (상태 필터, 로봇 필터, 날짜 정렬)
- **미션 생성 폼**: 출발지/도착지 노드 선택(3D 맵 클릭), 미션 타입(transport, patrol, charge), 우선순위
- **미션 상세**: 상태 타임라인, 경로 시각화(3D 맵 오버레이), 소요시간, 할당 로봇 정보
- **실시간 업데이트**: WebSocket `mission.state_changed` 이벤트 구독

---

## 4. 구현 모듈

| 모듈 | 팀 | 경로 |
|------|-----|------|
| `mission_service.rs` | Team 2 | `backend/src/services/mission_service.rs` |
| `vda5050_order_builder.rs` | Team 2 | `backend/src/services/vda5050.rs` |
| `MissionListPage.tsx` | Team 1 | `frontend/src/pages/MissionListPage.tsx` |
| `MissionCreateForm.tsx` | Team 1 | `frontend/src/components/mission/MissionCreateForm.tsx` |
| `MissionDetailPanel.tsx` | Team 1 | `frontend/src/components/mission/MissionDetailPanel.tsx` |
| `useMissionWebSocket.ts` | Team 1 | `frontend/src/hooks/useMissionWebSocket.ts` |

---

## 5. 의존성

| 항목 | 종류 | 비고 |
|------|------|------|
| PostgreSQL missions 테이블 | DB | C-03에서 스키마 정의 |
| Map Manager FindPath gRPC | 서비스 | Team 5 제공 |
| MQTT 브로커 (Mosquitto) | 인프라 | VDA5050 Order 발행 |
| WebSocket relay | 서비스 | 미션 상태 이벤트 브로드캐스트 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#mission-data) 참조

---

## 6. 테스트 기준

| 테스트 유형 | 기준 |
|------------|------|
| 단위 테스트 | 상태머신 전이 100% 커버리지 (유효/무효 전이 모두) |
| 통합 테스트 | 미션 생성 → 할당 → Order 전송 → 완료 E2E 흐름 |
| API 테스트 | 모든 REST 엔드포인트 정상/에러 케이스 |
| Frontend 테스트 | 미션 생성 폼 유효성 검증, 상태 표시 정확성 |
| 성능 테스트 | 동시 100개 미션 생성 시 p95 < 200ms |

---

## 7. 완료 조건

- [ ] 미션 CRUD API 전체 구현 및 OpenAPI 문서화
- [ ] 상태머신 모든 전이가 정상 작동
- [ ] VDA5050 Order 메시지가 MQTT로 정상 발행
- [ ] A* 경로 요청/응답 연동 확인
- [ ] Frontend 미션 리스트/상세/생성 UI 완성
- [ ] WebSocket을 통한 실시간 상태 반영 확인
- [ ] 통합 테스트 통과율 95% 이상
