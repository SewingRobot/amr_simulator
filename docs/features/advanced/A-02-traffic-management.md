# A-02: 교통 관리 시스템 (Traffic Management)

## 1. 개요

### 1.1 목표
다수의 AMR이 동일 환경에서 운행 시 충돌을 방지하기 위해 존 잠금(zone locking),
충돌 감지(conflict detection), 우선순위 큐(priority queue), 데드락 방지를 구현한다.

### 1.2 담당팀

| 팀 | 역할 |
|----|------|
| **Team 2 (Backend)** | 교통 관리 서비스 전체 (존 잠금, 충돌 감지, 우선순위, 데드락 방지) |
| **Team 1 (Frontend)** | 존 잠금 시각화 (3D 맵 오버레이) |

### 1.3 Phase: **2**

### 1.4 선행조건
- A-01: 미션 시스템 (미션 할당 전 교통 검증 필요)
- C-04: 로드맵 그래프 (존/노드/엣지 정의)

---

## 2. 스코프

### In-Scope
- Redis 기반 존 잠금/해제 메커니즘
- 미션 할당 전 경로 충돌 감지
- 우선순위 기반 미션 큐 관리
- 타임아웃 기반 데드락 방지
- Frontend 존 잠금 상태 시각화

### Out-of-Scope
- 동적 경로 재계획 (→ Phase 3)
- 교차로 신호등 시뮬레이션 (→ Phase 4)
- 멀티플로어 교통 관리 (→ Phase 4)

---

## 3. 상세 스펙

### 3.1 존 잠금 (Zone Locking)

```rust
// Redis key: zone_lock:{zone_id}
// Value: robot_id
// TTL: zone_lock_timeout_secs (default 60s)

impl TrafficService {
    pub async fn acquire_zone_lock(&self, zone_id: &str, robot_id: &str) -> Result<bool>;
    pub async fn release_zone_lock(&self, zone_id: &str, robot_id: &str) -> Result<()>;
    pub async fn get_zone_lock(&self, zone_id: &str) -> Result<Option<String>>;
    pub async fn release_all_locks_for_robot(&self, robot_id: &str) -> Result<u32>;
}
```

- Redis `SET NX EX` 원자적 잠금 획득
- 로봇이 존을 벗어나면 자동 해제
- 로봇 연결 끊김 시 해당 로봇의 모든 잠금 해제

### 3.2 충돌 감지 (Conflict Detection)

```rust
pub async fn check_path_conflicts(
    &self,
    robot_id: &str,
    path_zones: &[String],
) -> Result<Vec<Conflict>>;

pub struct Conflict {
    pub zone_id: String,
    pub locked_by: String,       // 충돌 로봇 ID
    pub conflict_type: ConflictType, // HeadOn, CrossPath, SameZone
}
```

미션 할당 시 경로의 모든 존을 검사하여 잠긴 존이 있으면 충돌 리포트를 반환한다.
충돌 시 전략: 대기(wait), 재경로(reroute), 우선순위 비교(priority).

### 3.3 우선순위 큐

| 우선순위 | 레벨 | 설명 |
|---------|------|------|
| EMERGENCY | 0 | 긴급 정지/복구 (다른 모든 미션 일시정지) |
| HIGH | 1 | 충전 필요, 안전 관련 |
| NORMAL | 2 | 일반 운송 미션 |
| LOW | 3 | 순찰, 비긴급 작업 |

동일 존 요청 시 높은 우선순위 로봇이 선점. 동일 우선순위면 FIFO.

### 3.4 데드락 방지

```
데드락 시나리오: Robot A가 Zone 1 점유 → Zone 2 요청
                Robot B가 Zone 2 점유 → Zone 1 요청
```

- **Wait-For 그래프**: 대기 관계를 DAG로 추적, 사이클 발생 시 데드락 감지
- **타임아웃**: 존 잠금 요청 대기가 `deadlock_timeout_secs` (default 30s) 초과 시 낮은 우선순위 미션 취소
- **순서 규칙**: 존 ID 오름차순으로만 잠금 획득 허용 (예방적 방지)

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#traffic-management) 참조

### 3.5 Frontend 시각화

- 3D 맵에 존 영역 반투명 오버레이 (잠금 상태별 색상)
  - 🟢 미잠금: 초록 (10% opacity)
  - 🔴 잠금: 빨강 (30% opacity) + 점유 로봇 ID 라벨
  - 🟡 요청 대기: 노랑 (20% opacity)
- WebSocket `zone.lock_changed` 이벤트로 실시간 갱신

---

## 4. 구현 모듈

| 모듈 | 팀 | 경로 |
|------|-----|------|
| `traffic_service.rs` | Team 2 | `backend/src/services/traffic_service.rs` |
| `deadlock_detector.rs` | Team 2 | `backend/src/services/deadlock_detector.rs` |
| `priority_queue.rs` | Team 2 | `backend/src/services/priority_queue.rs` |
| `ZoneLockOverlay.tsx` | Team 1 | `frontend/src/components/map/ZoneLockOverlay.tsx` |

---

## 5. 의존성

| 항목 | 종류 | 비고 |
|------|------|------|
| Redis 7.x | 인프라 | 존 잠금 저장소 (SET NX EX) |
| Map Manager 존 정의 | 데이터 | 존 polygon, 존 ID 목록 |
| Mission Service | 서비스 | 미션 할당 전 충돌 검사 호출 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#traffic-zones) 참조

---

## 6. 테스트 기준

| 테스트 유형 | 기준 |
|------------|------|
| 단위 테스트 | 존 잠금 CRUD, 우선순위 비교, 데드락 감지 로직 |
| 통합 테스트 | 2대 로봇 동시 미션 할당 시 충돌 감지 및 우선순위 처리 |
| 데드락 테스트 | 순환 대기 시나리오 재현 및 자동 해소 확인 |
| 성능 테스트 | 50개 존, 20대 로봇 동시 잠금 요청 시 p95 < 50ms |
| Frontend 테스트 | 존 잠금 상태 색상 변경 정확성 |

---

## 7. 완료 조건

- [ ] Redis 기반 존 잠금/해제 원자적 동작 확인
- [ ] 미션 할당 시 경로 충돌 자동 감지
- [ ] 우선순위 큐에 따른 존 선점 동작 확인
- [ ] 데드락 감지 및 타임아웃 자동 해소
- [ ] Frontend 존 잠금 시각화 실시간 반영
- [ ] 동시 20대 로봇 교통 관리 안정성 확인
