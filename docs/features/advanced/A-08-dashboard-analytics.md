# A-08: 대시보드 및 분석 (Dashboard & Analytics)

## 1. 개요

### 1.1 목표
플릿 KPI 대시보드, 미션 분석, 로봇 이동 히트맵, 텔레메트리 이력 저장/재생을
구현하여 운영자에게 데이터 기반 인사이트를 제공한다.

### 1.2 담당팀

| 팀 | 역할 |
|----|------|
| **Team 1 (Frontend)** | KPI 카드, 차트(Recharts), 히트맵(D3/Deck.gl), 텔레메트리 재생 UI |
| **Team 2 (Backend)** | TimescaleDB 연속 집계, 분석 쿼리 API, 텔레메트리 이력 저장 |

### 1.3 Phase: **4**

### 1.4 선행조건
- A-01: 미션 시스템 (미션 데이터 소스)
- C-06: 로봇 상태 모니터링 (텔레메트리 파이프라인)

---

## 2. 스코프

### In-Scope
- TimescaleDB 하이퍼테이블 및 연속 집계(continuous aggregates)
- KPI 카드 (일일 미션 수, 가동률, 평균 미션 시간, 오류율)
- 로봇 상태 분포 차트 (Recharts pie/bar chart)
- 이동 경로 히트맵 (D3 또는 Deck.gl HeatmapLayer)
- 텔레메트리 이력 쿼리 및 재생 UI
- 실시간 알림 피드 (미션 실패, 배터리 부족, 연결 끊김)

### Out-of-Scope
- 예측 분석 / ML 기반 이상 탐지 (→ 향후)
- 커스텀 대시보드 빌더 (→ 향후)
- 외부 BI 도구 연동 (Grafana, Tableau) (→ 향후)

---

## 3. 상세 스펙

### 3.1 TimescaleDB 설정

```sql
-- 텔레메트리 하이퍼테이블
CREATE TABLE telemetry_history (
    time        TIMESTAMPTZ NOT NULL,
    robot_id    TEXT NOT NULL,
    x           DOUBLE PRECISION,
    y           DOUBLE PRECISION,
    theta       DOUBLE PRECISION,
    linear_vel  DOUBLE PRECISION,
    angular_vel DOUBLE PRECISION,
    battery_pct DOUBLE PRECISION,
    status      TEXT
);
SELECT create_hypertable('telemetry_history', 'time');

-- 연속 집계: 1분 단위 로봇별 평균
CREATE MATERIALIZED VIEW telemetry_1min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 minute', time) AS bucket,
    robot_id,
    AVG(linear_vel) AS avg_vel,
    AVG(battery_pct) AS avg_battery,
    COUNT(*) AS sample_count
FROM telemetry_history
GROUP BY bucket, robot_id;

-- 보존 정책: 원본 30일, 1분 집계 1년
SELECT add_retention_policy('telemetry_history', INTERVAL '30 days');
```

### 3.2 KPI 카드

| KPI | 계산 | 갱신 주기 |
|-----|------|----------|
| 일일 미션 완료 수 | `COUNT(missions WHERE status='COMPLETED' AND date=today)` | 1분 |
| 플릿 가동률 | `SUM(active_time) / SUM(total_time) * 100` | 5분 |
| 평균 미션 소요시간 | `AVG(completed_at - started_at)` | 5분 |
| 미션 성공률 | `COMPLETED / (COMPLETED + FAILED) * 100` | 5분 |
| 평균 배터리 잔량 | `AVG(battery_pct) across fleet` | 실시간 |

### 3.3 로봇 상태 분포 차트

```typescript
// Recharts 도넛 차트
<PieChart>
  <Pie data={[
    { name: 'Idle', value: idleCount, fill: '#82ca9d' },
    { name: 'Executing', value: execCount, fill: '#8884d8' },
    { name: 'Charging', value: chargeCount, fill: '#ffc658' },
    { name: 'Error', value: errorCount, fill: '#ff7300' },
    { name: 'Offline', value: offlineCount, fill: '#999' },
  ]} />
</PieChart>
```

- 실시간 WebSocket 이벤트로 갱신
- 시간대별 상태 변화 스택 바 차트 (24시간)

### 3.4 이동 히트맵

```typescript
// Deck.gl HeatmapLayer
new HeatmapLayer({
  data: positionHistory, // [{x, y, weight}]
  getPosition: d => [d.x, d.y],
  getWeight: d => d.visit_count,
  radiusPixels: 30,
  intensity: 1,
  threshold: 0.05,
});
```

- 시간 범위 필터 (최근 1시간, 24시간, 7일, 커스텀)
- 로봇 필터 (개별/전체)
- 3D 맵 위에 오버레이

### 3.5 텔레메트리 재생

```typescript
interface PlaybackControls {
  timeRange: { start: Date; end: Date };
  playbackSpeed: number;  // 0.5x, 1x, 2x, 5x, 10x
  robotIds: string[];     // 선택한 로봇들
  isPlaying: boolean;
}
```

- Backend API: `GET /api/v1/telemetry/history?robot_id=...&start=...&end=...`
- 재생 UI: 타임라인 슬라이더, 재생/일시정지/속도 조절
- 3D 뷰어에서 과거 로봇 위치를 고스트로 표시
- 재생 중 실시간 데이터 표시 일시 중단

### 3.6 알림 피드

| 알림 유형 | 조건 | 심각도 |
|----------|------|--------|
| 미션 실패 | mission.status = FAILED | ERROR |
| 배터리 부족 | battery_pct < 20% | WARNING |
| 로봇 오프라인 | connection = OFFLINE | ERROR |
| 미션 지연 | 소요시간 > 예상시간 × 1.5 | WARNING |
| 존 데드락 | 데드락 감지 | ERROR |

- 우측 사이드 패널 알림 목록 (최신 50개)
- 브라우저 Notification API 연동 (사용자 허용 시)

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#analytics-api) 참조

---

## 4. 구현 모듈

| 모듈 | 팀 | 경로 |
|------|-----|------|
| `DashboardPage.tsx` | Team 1 | `frontend/src/pages/DashboardPage.tsx` |
| `KpiCards.tsx` | Team 1 | `frontend/src/components/dashboard/KpiCards.tsx` |
| `RobotStatusChart.tsx` | Team 1 | `frontend/src/components/dashboard/RobotStatusChart.tsx` |
| `HeatmapOverlay.tsx` | Team 1 | `frontend/src/components/dashboard/HeatmapOverlay.tsx` |
| `TelemetryReplay.tsx` | Team 1 | `frontend/src/components/dashboard/TelemetryReplay.tsx` |
| `AlertFeed.tsx` | Team 1 | `frontend/src/components/dashboard/AlertFeed.tsx` |
| `analytics_service.rs` | Team 2 | `backend/src/services/analytics_service.rs` |
| `telemetry_history.rs` | Team 2 | `backend/src/services/telemetry_history.rs` |

---

## 5. 의존성

| 항목 | 종류 | 비고 |
|------|------|------|
| TimescaleDB | DB | PostgreSQL 확장 (하이퍼테이블, 연속 집계) |
| Recharts | npm | 차트 라이브러리 |
| Deck.gl | npm | 히트맵 렌더링 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#analytics-data) 참조

---

## 6. 테스트 기준

| 테스트 유형 | 기준 |
|------------|------|
| 쿼리 테스트 | 연속 집계 결과와 원본 데이터 집계 일치 |
| 성능 테스트 | 30일 데이터 KPI 쿼리 p95 < 500ms |
| UI 테스트 | KPI 카드 실시간 갱신, 차트 렌더링 정확성 |
| 재생 테스트 | 1시간 분량 텔레메트리 재생 → 프레임 드롭 없이 동작 |
| 알림 테스트 | 조건 충족 시 1초 이내 알림 표시 |

---

## 7. 완료 조건

- [ ] TimescaleDB 하이퍼테이블 및 연속 집계 설정 완료
- [ ] KPI 카드 5종 실시간 표시
- [ ] 로봇 상태 분포 차트 실시간 갱신
- [ ] 히트맵 시간 범위/로봇 필터 적용 동작
- [ ] 텔레메트리 재생 UI (타임라인, 속도 조절, 고스트 표시)
- [ ] 알림 피드 실시간 동작 및 브라우저 알림 연동
