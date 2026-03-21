# AI 에이전트 할당 전략

이 문서는 AMR 통합 프레임워크 프로젝트에서 AI 에이전트 스웜의 구성, 역할 분담, Phase별 할당 전략을 정의한다. 추후 각 에이전트별 구체적인 task 할당 문서 작성의 기반이 된다.

## 1. 에이전트 유형 정의

### 1.1 팀 에이전트 (Team Agent)
각 서브 프로젝트를 담당하는 핵심 에이전트. 해당 서브 프로젝트의 코드를 작성하고 테스트한다.

| 에이전트 ID | 담당 팀 | 주요 언어 | 필요 역량 |
|------------|---------|----------|----------|
| `frontend-agent` | Team 1 | TypeScript, React, Three.js | 웹 프론트엔드, 3D 렌더링, WebSocket |
| `backend-agent` | Team 2 | Rust, SQL | Axum, gRPC, PostgreSQL, MQTT |
| `sim-agent` | Team 3 | C++, Python | 물리 시뮬레이션, gRPC, CMake |
| `asset-agent` | Team 4 | Rust, Python | 파일 변환, gRPC, MinIO |
| `map-agent` | Team 5 | Rust, Python | PostGIS, Open3D, Potree |

### 1.2 통합 에이전트 (Integration Agent)
크로스팀 작업을 전담하는 에이전트.

| 에이전트 ID | 역할 | 담당 범위 |
|------------|------|----------|
| `proto-agent` | Proto/인터페이스 관리 | proto/ 파일 관리, 코드 생성 검증, RFC 처리 |
| `infra-agent` | 인프라/배포 | Docker Compose, CI/CD, Dockerfile, 환경 설정 |
| `test-agent` | 통합 테스트 | E2E 테스트 작성/실행, 테스트 데이터 생성, 체크포인트 검증 |
| `docs-agent` | 문서 관리 | CHANGELOG, 상태 리포트 종합, 문서 일관성 검증 |

### 1.3 리뷰 에이전트 (Review Agent)
코드 품질과 인터페이스 준수를 검증하는 에이전트.

| 에이전트 ID | 역할 |
|------------|------|
| `review-agent` | 머지 전 코드 리뷰, 인터페이스 계약 준수 확인, 테스트 커버리지 체크 |

## 2. Phase별 에이전트 할당

### 2.1 Phase 1: Prototype (Week 1-8)

**목표:** 핵심 기능(C-01~C-06) E2E 동작

#### Week 1-2: Foundation
| 에이전트 | Task | 의존성 |
|----------|------|--------|
| `proto-agent` | proto/ 스키마 초안 작성 + 팀 리뷰 | 없음 (최우선) |
| `infra-agent` | 모노레포 구조 설정, Docker Compose 기본, CI 파이프라인 | 없음 |
| `sim-agent` | C-01: CMake 프로젝트 설정, 빌드 시스템 | 없음 |
| `backend-agent` | C-03: Cargo 프로젝트 설정, DB 마이그레이션 | 없음 |
| `frontend-agent` | C-02: Vite+React 프로젝트 설정, Three.js 기본 씬 | 없음 |
| `asset-agent` | C-05: Cargo 프로젝트 설정, MinIO 연동 | 없음 |
| `map-agent` | C-04: Cargo 프로젝트 설정, PostGIS 스키마 | 없음 |

#### Week 3-4: Core Implementation
| 에이전트 | Task | 의존성 |
|----------|------|--------|
| `proto-agent` | proto/ 스키마 확정 (Proto Freeze), 코드 생성 | Week 2 리뷰 완료 |
| `sim-agent` | C-01: 커스텀 물리 엔진, 차동 구동, gRPC 서버 | proto freeze |
| `backend-agent` | C-03: REST API (auth, robots, maps), WebSocket 서버 | proto freeze |
| `frontend-agent` | C-02: 로봇 렌더링, 카메라 컨트롤, WS 클라이언트 | proto freeze |
| `asset-agent` | C-05: 에셋 업/다운로드, 기본 메타데이터 CRUD | proto freeze |
| `map-agent` | C-04: 포인트 클라우드 업로드, Python 처리 파이프라인 | proto freeze |
| `test-agent` | 테스트 데이터 생성 (XS, S 데이터셋), mock 서버 | proto freeze |

#### Week 5-6: Sensors & Streaming
| 에이전트 | Task | 의존성 |
|----------|------|--------|
| `sim-agent` | C-01: 2D LiDAR, 배터리 모델, 다중 로봇 | C-01 물리엔진 |
| `backend-agent` | C-03: gRPC→WS 텔레메트리 브릿지, 로봇 상태 관리 | C-03 WS 서버 |
| `frontend-agent` | C-02: WS 텔레메트리 수신, 포즈 보간, 로봇 선택 | C-02 기본 씬 |
| `asset-agent` | C-05: URDF→glTF 변환 파이프라인 | C-05 업로드 |
| `map-agent` | C-04: Potree 변환, 타일 서빙, 로드맵 그래프 CRUD | C-04 업로드 |

#### Week 7-8: Integration & Polish
| 에이전트 | Task | 의존성 |
|----------|------|--------|
| `test-agent` | C-06: E2E 텔레메트리 파이프라인 통합 테스트 | 전체 C-01~C-05 |
| `infra-agent` | Docker Compose 전체 스택, 통합 테스트 환경 | 전체 |
| `frontend-agent` | C-02: 포인트 클라우드 기본 표시, glTF 로드 통합 | C-04 타일, C-05 에셋 |
| `backend-agent` | C-03: Asset/Map 프록시 엔드포인트 | C-04, C-05 gRPC |
| `sim-agent` | C-01: 성능 최적화, 시나리오 스크립트 | C-01 완성 |
| `review-agent` | 전체 코드 리뷰, 인터페이스 계약 준수 확인 | 전체 |
| `docs-agent` | Phase 1 완료 보고서, CHANGELOG 정리 | 전체 |

### 2.2 Phase 2: Mission & Control (Week 9-16)

| 에이전트 | 주요 Task | 기능 참조 |
|----------|----------|----------|
| `backend-agent` | 미션 서비스, 트래픽 서비스 | A-01, A-02 |
| `frontend-agent` | 미션 UI, 맵 에디터, 원격 조종 | A-01 UI, A-03, A-06 |
| `map-agent` | 시맨틱 레이어, A* 경로 계획, 장애물 관리 | A-06, A-01 pathfinding |
| `sim-agent` | 시나리오 고도화 (다중 미션 시뮬) | A-01 시뮬 검증 |
| `test-agent` | 미션 E2E 테스트, 트래픽 시나리오 테스트 | A-01, A-02 |
| `proto-agent` | mission_types.proto 확장, RFC 처리 | A-01 |

### 2.3 Phase 3: Real Robot & Advanced Sim (Week 17-22)

| 에이전트 | 주요 Task | 기능 참조 |
|----------|----------|----------|
| `backend-agent` | VDA5050 MQTT 브릿지, 실로봇 텔레메트리 | A-04 |
| `sim-agent` | MuJoCo 백엔드, 고급 센서 (3D LiDAR, Camera) | A-05 |
| `asset-agent` | OpenUSD 변환 파이프라인 (URDF→USD, USD→glTF) | A-09 |
| `frontend-agent` | 하이브리드 뷰 모드, WebRTC 카메라 | A-03 camera, A-10 기본 |
| `infra-agent` | MQTT 브로커 설정, Isaac Sim 연동 환경 | A-04, A-05 |

### 2.4 Phase 4: Production (Week 23-30)

| 에이전트 | 주요 Task | 기능 참조 |
|----------|----------|----------|
| `backend-agent` | WASM 플러그인 시스템, ROS 2 브릿지 플러그인 | A-07 |
| `frontend-agent` | 대시보드, 분석, 디지털 트윈 | A-08, A-10 |
| `sim-agent` | Isaac Sim 브릿지, OpenUSD 씬 로딩 | A-05, A-09 |
| `infra-agent` | K8s Helm, 프로덕션 배포 자동화 | 배포 |
| `test-agent` | 전체 E2E, 부하 테스트 (L/XL 데이터셋) | 전체 |
| `docs-agent` | 최종 문서 정리, 사용자 가이드 | 전체 |

## 3. 에이전트 간 동시성 규칙

### 3.1 파일 잠금
- 각 에이전트는 자기 서브 프로젝트 디렉토리만 수정
- proto/ 변경은 `proto-agent`만 가능
- docker-compose 변경은 `infra-agent`만 가능
- docs/rfcs/, docs/issues/ 는 모든 에이전트 쓰기 가능

### 3.2 동시 작업 가능 매트릭스
```
                sim  backend  frontend  asset  map  proto  infra  test
sim-agent        O      X        X       X     X      X      X      X
backend-agent    X      O        X       X     X      X      X      X
frontend-agent   X      X        O       X     X      X      X      X
asset-agent      X      X        X       O     X      X      X      X
map-agent        X      X        X       X     O      X      X      X
proto-agent      X      X        X       X     X      O      X      X
infra-agent      X      X        X       X     X      X      O      X
test-agent       (read-only access to all, writes to tests/ only)
```

### 3.3 병렬화 최대화 전략
- Phase 1 Week 1-2: 모든 팀 에이전트 병렬 (프로젝트 스캐폴딩)
- Proto Freeze 후: 모든 팀 에이전트 독립 병렬 개발
- 통합 체크포인트: 직렬화 (test-agent가 순차 검증)

## 4. 에이전트 스케일링 전략

### 4.1 팀 내 멀티 에이전트
복잡한 기능은 팀 내에서 서브 에이전트를 추가 할당:

| 상황 | 전략 | 예시 |
|------|------|------|
| 독립적 서브 모듈 존재 | 모듈별 에이전트 분할 | `sim-agent-physics` + `sim-agent-sensors` |
| 백엔드 서비스 분리 | 서비스별 에이전트 | `backend-agent-mission` + `backend-agent-traffic` |
| 프론트엔드 복잡 UI | 페이지별 에이전트 | `frontend-agent-viewer` + `frontend-agent-editor` |
| 변환 파이프라인 독립 | 파이프라인별 에이전트 | `asset-agent-urdf` + `asset-agent-gltf` |

### 4.2 추천 에이전트 수 (Phase별)
| Phase | 팀 에이전트 | 통합 에이전트 | 총 에이전트 | 근거 |
|-------|-----------|-------------|-----------|------|
| Phase 1 | 5 | 4 (proto, infra, test, docs) | 9 | 기본 스캐폴딩, 병렬화 효과 높음 |
| Phase 2 | 7 (backend x2, frontend x2) | 3 (proto, test, docs) | 10 | 미션+트래픽, 에디터+원격조종 분할 |
| Phase 3 | 6 (sim x2) | 3 | 9 | MuJoCo+Isaac Sim 병렬 |
| Phase 4 | 7 | 4 (infra K8s 추가) | 11 | 최종 통합, 배포 준비 |

## 5. 에이전트 컨텍스트 관리

### 5.1 에이전트 시작 시 로드할 문서
각 에이전트 유형별로 세션 시작 시 반드시 읽어야 할 문서:

**팀 에이전트 공통:**
1. `CLAUDE.md` (프로젝트 루트)
2. `.claude/rules/*.md` (모든 규칙 문서)
3. `docs/architecture.md` (전체 아키텍처)
4. 자기 팀 문서: `docs/teams/team{N}-*.md`
5. 현재 작업 중인 기능 문서: `docs/features/{core|advanced}/{feature}.md`
6. `docs/rfcs/` — 미처리 RFC 확인
7. `docs/issues/` — 자기 팀 담당 이슈 확인
8. 최근 `docs/worklog/` — 이전 세션 작업 일지

**통합 에이전트 추가:**
- `docs/integration/integration-spec.md`
- `docs/integration/test-data-spec.md`
- `docs/integration/data-pipeline.md`

### 5.2 에이전트 종료 시 반드시 수행
1. 작업 일지 작성 (`docs/worklog/`)
2. CHANGELOG.md 업데이트 (기능 변경 시)
3. 미해결 이슈 기록 (`docs/issues/`)
4. 다른 팀 요청 사항 RFC 작성 (필요 시)
5. Git commit + push (커밋 규칙 준수)

## 6. Task 할당 문서 템플릿

추후 각 에이전트에게 전달할 task 할당 문서의 표준 포맷:

```markdown
# Task 할당: {agent-id}

**기간:** Week 3-4
**Phase:** 1
**기능:** C-01 Sim Engine Core

## 목표
이번 작업 세션에서 달성할 구체적 목표

## 구현할 항목
- [ ] 항목 1: 설명 + 참조 문서
- [ ] 항목 2: 설명 + 참조 문서

## 참조 문서
- 기능 스펙: `docs/features/core/C-01-sim-engine-core.md`
- 팀 기술스택: `docs/teams/team3-sim-engine.md`
- 인터페이스: `docs/integration/integration-spec.md#simulation-service`
- 테스트 데이터: `docs/integration/test-data-spec.md#sim-engine-test-data`

## 의존성
- Proto 스키마 확정 완료 (확인)
- Backend gRPC 클라이언트 mock 사용 가능 (확인)

## 완료 기준
- [ ] 빌드 성공
- [ ] 단위 테스트 통과
- [ ] 작업 일지 작성
- [ ] CHANGELOG 업데이트

## 제약 사항
- proto/ 파일 직접 수정 금지 (RFC 제출)
- 다른 서브 프로젝트 디렉토리 수정 금지
```
