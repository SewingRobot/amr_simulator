# C-03: Backend Core

> **문서 버전**: 1.0
> **최종 수정일**: 2026-03-22
> **담당 팀**: Team 2 (Backend)
> **Phase**: 1 (Prototype)

---

## 1. 개요

### 1.1 목표
REST API, WebSocket 텔레메트리 릴레이, JWT 인증, PostgreSQL 기본 스키마를 포함한 백엔드 코어를 구축한다. Axum 기반 HTTP 서버로 프론트엔드에 API를 제공하고, Sim Engine gRPC 텔레메트리를 WebSocket으로 브릿지하여 브라우저 클라이언트에 실시간 데이터를 전달한다.

### 1.2 배경
Backend는 AMR 프레임워크의 중앙 허브로서, 프론트엔드(REST/WebSocket 소비자)와 Sim Engine(gRPC 서버) 사이를 중계한다. Phase 1에서는 인증, 기본 CRUD, 텔레메트리 브릿지에 집중하며, Mission/Traffic 서비스는 후속 기능에서 구현한다.

---

## 2. 스코프

### 2.1 In-Scope (이 기능에서 구현)
- Axum HTTP 서버 설정 (CORS, 에러 핸들링, health check)
- JWT 인증 (login, refresh, me)
- Robot CRUD REST API
- Map CRUD REST API (기본)
- WebSocket 서버 (연결, 토픽 구독, 텔레메트리 팬아웃)
- PostgreSQL 연결 풀 + 기본 테이블 (users, robots, maps)
- gRPC 클라이언트 스텁 (Sim Engine 연결)
- Sim Engine gRPC → WebSocket 텔레메트리 브릿지
- 기본 설정/환경변수 관리 (`AppConfig`)
- DB 마이그레이션 (sqlx migrate)

### 2.2 Out-of-Scope (후속 기능에서 구현)
- Mission Service (CRUD, 상태 머신, VDA5050) → A-01
- Traffic Service (존 잠금, 충돌 감지) → A-02
- MQTT / VDA5050 실로봇 통신 → A-04
- Plugin System (WASM, Webhook) → A-07
- TimescaleDB 텔레메트리 저장 → A-03
- ROS 2 Bridge
- Redis pub/sub 이벤트 버스
- Asset Manager / Map Manager gRPC 클라이언트 → Phase 2
- Rate limiting 미들웨어

---

## 3. 상세 스펙

### 3.1 서버 부트스트랩

```rust
// main.rs 의사 코드
#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();       // 환경변수 로딩
    let db_pool = create_pg_pool(&config).await;  // PostgreSQL 풀
    sqlx::migrate!().run(&db_pool).await;     // 마이그레이션 자동 적용
    let sim_client = SimGrpcClient::connect(&config.sim_engine_url).await;
    let app_state = AppState { db_pool, config, sim_client };
    let router = create_router(app_state);
    axum::serve(listener, router).await;
}
```

### 3.2 REST API (Phase 1)

| Method | Path | Auth | 설명 |
|--------|------|------|------|
| GET | `/health` | No | 서버 상태 확인 (DB 연결 포함) |
| POST | `/api/auth/login` | No | 이메일/비밀번호 → JWT access + refresh token |
| POST | `/api/auth/refresh` | No | refresh token → 새 access token |
| GET | `/api/auth/me` | Yes | 현재 사용자 정보 반환 |
| GET | `/api/robots` | Yes | 로봇 목록 (pagination, filter by status) |
| POST | `/api/robots` | Yes(admin) | 로봇 등록 |
| GET | `/api/robots/:id` | Yes | 로봇 상세 정보 |
| PUT | `/api/robots/:id` | Yes(admin) | 로봇 정보 수정 |
| DELETE | `/api/robots/:id` | Yes(admin) | 로봇 삭제 |
| GET | `/api/maps` | Yes | 맵 목록 |
| POST | `/api/maps` | Yes | 맵 생성 |
| GET | `/api/maps/:id` | Yes | 맵 상세 정보 |
| PUT | `/api/maps/:id` | Yes | 맵 수정 |
| DELETE | `/api/maps/:id` | Yes(admin) | 맵 삭제 |

> 📋 REST API 전체 스키마 (request/response body): [integration-spec.md](../../integration/integration-spec.md#rest-api) 참조

### 3.3 인증 (JWT)

**토큰 구조:**
```rust
struct Claims {
    sub: Uuid,           // user_id
    email: String,
    role: UserRole,      // Admin, Operator, Viewer
    exp: u64,            // expiration (Unix timestamp)
    iat: u64,            // issued at
}
```

**토큰 설정:**
- Access token: 유효기간 15분, `Authorization: Bearer <token>` 헤더
- Refresh token: 유효기간 7일, HTTP-only cookie 또는 body
- 서명 알고리즘: HS256, secret은 환경변수 `JWT_SECRET`

**비밀번호 해싱:** Argon2id (argon2 crate)

**역할 기반 접근 제어 (RBAC):**

| Role | 권한 |
|------|------|
| Admin | 모든 API 접근, 사용자/로봇/맵 CRUD |
| Operator | 로봇/맵 조회, 미션 관리 (Phase 2) |
| Viewer | 조회 전용 |

### 3.4 WebSocket

**연결:**
- Endpoint: `GET /api/ws?token={jwt}`
- JWT 검증 후 WebSocket 업그레이드
- 연결 실패 시 `401 Unauthorized` 반환

**메시지 프로토콜:**
```json
// Client → Server: 토픽 구독
{"type": "subscribe", "topics": ["telemetry:*"]}
{"type": "subscribe", "topics": ["telemetry:robot-001"]}
{"type": "unsubscribe", "topics": ["telemetry:*"]}

// Server → Client: 텔레메트리
{
  "type": "telemetry",
  "payload": {
    "robot_id": "robot-001",
    "timestamp": 1679500000000,
    "position": {"x": 5.23, "y": 3.14},
    "heading": 1.57,
    "velocity": {"linear": 0.5, "angular": 0.0},
    "battery": 85.3,
    "status": "MOVING"
  }
}

// Server → Client: 이벤트 알림
{"type": "event", "payload": {"kind": "robot_spawned", "robot_id": "robot-002"}}
```

> 📋 WebSocket 메시지 전체 스키마: [integration-spec.md](../../integration/integration-spec.md#websocket-messages) 참조

**Heartbeat:**
- 서버가 30초마다 ping 전송
- 클라이언트가 10초 내 pong 미응답 시 연결 해제
- 클라이언트 재연결은 프론트엔드에서 처리

**팬아웃 아키텍처:**
- `tokio::sync::broadcast` 채널로 텔레메트리 배포
- 각 WebSocket 세션이 broadcast receiver를 구독
- 토픽 필터링: 세션별 구독 토픽 목록과 매칭하여 필요한 메시지만 전송

### 3.5 Sim Engine gRPC 브릿지

Backend는 Sim Engine에 대해 gRPC **클라이언트** 역할을 한다.

**텔레메트리 수신 흐름:**
```
Sim Engine (gRPC Server)
    │ StreamTelemetry (server streaming)
    ▼
Backend (gRPC Client)
    │ TelemetryMessage → JSON 변환
    │ broadcast 채널에 publish
    ▼
WebSocket Sessions
    │ broadcast 수신 → 토픽 필터링
    ▼
Browser Clients
```

**명령 전달 흐름:**
```
Browser Client
    │ WebSocket: {"type":"command","payload":{...}}
    ▼
Backend WebSocket Handler
    │ JSON → SendCommandRequest 변환
    ▼
Sim Engine (gRPC SendCommand)
```

**연결 관리:**
- Sim Engine 연결 실패 시 5초 간격 재시도 (최대 10회)
- 연결 상태를 health check endpoint에 포함
- 연결 끊김 시 WebSocket 클라이언트에 `{"type":"event","payload":{"kind":"sim_disconnected"}}` 전송

### 3.6 DB Schema (Phase 1 최소)

```sql
-- 001_create_users.sql
CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email       VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role        VARCHAR(20) NOT NULL DEFAULT 'viewer',  -- admin, operator, viewer
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 002_create_robots.sql
CREATE TABLE robots (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(100) UNIQUE NOT NULL,
    model_id    VARCHAR(100),            -- 로봇 모델 식별자 (glTF 참조)
    status      VARCHAR(20) NOT NULL DEFAULT 'offline',  -- online, offline, error
    config_json JSONB NOT NULL DEFAULT '{}',  -- RobotConfig (wheel_radius, etc.)
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 003_create_maps.sql
CREATE TABLE maps (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          VARCHAR(100) NOT NULL,
    version       INTEGER NOT NULL DEFAULT 1,
    metadata_json JSONB NOT NULL DEFAULT '{}',  -- 월드 크기, 설명 등
    world_json    JSONB,                         -- Phase 1: 월드 정의 JSON 저장
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**시드 데이터:**
- admin 사용자 1명 (`admin@amr.local` / 환경변수 `ADMIN_PASSWORD`)
- 기본 맵 1개 (simple warehouse)

### 3.7 설정 / 환경변수

| 변수명 | 기본값 | 설명 |
|--------|--------|------|
| `DATABASE_URL` | - | PostgreSQL 연결 문자열 (필수) |
| `JWT_SECRET` | - | JWT 서명 키 (필수, 32자 이상) |
| `ADMIN_PASSWORD` | - | 초기 admin 비밀번호 (필수) |
| `SIM_ENGINE_URL` | `http://localhost:50051` | Sim Engine gRPC 주소 |
| `SERVER_HOST` | `0.0.0.0` | HTTP 서버 바인드 주소 |
| `SERVER_PORT` | `8080` | HTTP 서버 포트 |
| `CORS_ORIGINS` | `http://localhost:5173` | 허용 CORS origin (쉼표 구분) |
| `LOG_LEVEL` | `info` | 로깅 레벨 (trace/debug/info/warn/error) |

---

## 4. 구현 모듈

| 파일 경로 | 모듈/구조체 | 설명 |
|-----------|-----------|------|
| `src/main.rs` | - | 서버 부트스트랩, 설정 로딩, 마이그레이션 실행 |
| `src/config.rs` | `AppConfig` | 환경변수 파싱, 설정 검증 |
| `src/state.rs` | `AppState` | 공유 상태 (DB pool, gRPC client, broadcast channel) |
| `src/error.rs` | `AppError` | 전역 에러 타입, Axum IntoResponse 구현 |
| `src/api/router.rs` | `create_router()` | 라우트 정의, 미들웨어 등록 |
| `src/api/middleware/auth.rs` | `AuthMiddleware` | JWT 검증, Claims 추출, 역할 검사 |
| `src/api/middleware/cors.rs` | `cors_layer()` | CORS 설정 |
| `src/api/handlers/auth.rs` | `login`, `refresh`, `me` | 인증 핸들러 |
| `src/api/handlers/robots.rs` | `list`, `create`, `get`, `update`, `delete` | 로봇 CRUD 핸들러 |
| `src/api/handlers/maps.rs` | `list`, `create`, `get`, `update`, `delete` | 맵 CRUD 핸들러 |
| `src/api/ws/handler.rs` | `ws_upgrade` | WebSocket 업그레이드 핸들러 |
| `src/api/ws/session.rs` | `WsSession` | 클라이언트별 세션, 토픽 구독/필터링 |
| `src/api/ws/messages.rs` | `WsMessage` | WebSocket 메시지 타입 정의 (serde) |
| `src/api/dto/*.rs` | Request/Response 타입 | API 요청/응답 DTO |
| `src/services/auth_service.rs` | `AuthService` | 비밀번호 해싱, JWT 생성/검증 |
| `src/services/telemetry_service.rs` | `TelemetryService` | 텔레메트리 broadcast 채널 관리 |
| `src/grpc/sim_client.rs` | `SimGrpcClient` | Sim Engine gRPC 클라이언트, 재연결 로직 |
| `src/db/pool.rs` | `create_pg_pool()` | PostgreSQL 연결 풀 생성 |
| `src/db/queries/users.rs` | 사용자 쿼리 | `find_by_email`, `create_user` |
| `src/db/queries/robots.rs` | 로봇 쿼리 | `list`, `find_by_id`, `create`, `update`, `delete` |
| `src/db/queries/maps.rs` | 맵 쿼리 | `list`, `find_by_id`, `create`, `update`, `delete` |
| `migrations/*.sql` | SQL 파일 | DB 스키마 마이그레이션 |

---

## 5. 의존성

| 의존 대상 | 종류 | 상세 |
|-----------|------|------|
| Proto 스키마 | 빌드 의존 | `proto/simulation.proto` — gRPC 클라이언트 코드 생성 |
| C-01 (Sim Engine) | 런타임 | gRPC 서버 필요 (목 서버로 대체 가능) |
| PostgreSQL | 인프라 | 15+ 버전, `docker-compose.yml`로 로컬 실행 |
| Axum | 크레이트 | HTTP 서버 프레임워크 |
| Tonic | 크레이트 | gRPC 클라이언트 |
| SQLx | 크레이트 | PostgreSQL 비동기 드라이버 |
| jsonwebtoken | 크레이트 | JWT 생성/검증 |
| argon2 | 크레이트 | 비밀번호 해싱 |
| serde / serde_json | 크레이트 | JSON 직렬화 |
| tokio | 크레이트 | 비동기 런타임 |
| tracing | 크레이트 | 구조화 로깅 |

**개발 시 독립성:**
Sim Engine 미가동 시 목 gRPC 서버(`tests/mocks/mock_sim_engine.rs`)로 텔레메트리 브릿지 테스트 가능. `docker-compose.yml`에 PostgreSQL 포함.

---

## 6. 테스트 기준

### 6.1 인증 테스트

| 테스트 케이스 | 검증 조건 |
|-------------|----------|
| 로그인 성공 | 올바른 email/password → 200 + access_token + refresh_token |
| 로그인 실패 | 잘못된 password → 401 Unauthorized |
| 토큰 갱신 | 유효한 refresh_token → 새 access_token 발급 |
| 만료 토큰 | 만료된 access_token → 401, refresh 필요 |
| 보호 엔드포인트 | 토큰 없이 `/api/robots` 접근 → 401 |
| 역할 검사 | Viewer 역할로 POST `/api/robots` → 403 Forbidden |

### 6.2 CRUD 테스트

| 테스트 케이스 | 검증 조건 |
|-------------|----------|
| Robot 생성 | POST `/api/robots` → 201, 응답에 id 포함 |
| Robot 목록 | GET `/api/robots` → 200, 배열 반환 |
| Robot 상세 | GET `/api/robots/:id` → 200, 생성 데이터 일치 |
| Robot 수정 | PUT `/api/robots/:id` → 200, 수정 내용 반영 |
| Robot 삭제 | DELETE `/api/robots/:id` → 204, 이후 GET → 404 |
| Map CRUD | 동일 패턴으로 maps 엔드포인트 테스트 |
| 존재하지 않는 리소스 | GET `/api/robots/{random_uuid}` → 404 |

### 6.3 WebSocket 테스트

| 테스트 케이스 | 검증 조건 |
|-------------|----------|
| 연결 | 유효한 JWT로 `/api/ws` 연결 → 101 Switching Protocols |
| 인증 실패 | 잘못된 JWT → 연결 거부 (401) |
| 구독 | subscribe 메시지 → 이후 해당 토픽 메시지 수신 |
| 텔레메트리 수신 | 목 데이터 broadcast → 구독 클라이언트에 전달 확인 |
| Heartbeat | 30초 ping → pong 응답 확인 |

### 6.4 gRPC 브릿지 테스트

| 테스트 케이스 | 검증 조건 |
|-------------|----------|
| 텔레메트리 전달 | Mock Sim Engine → gRPC stream → WebSocket 클라이언트 수신 |
| 명령 전달 | WebSocket command → gRPC SendCommand 호출 확인 |
| 연결 끊김 | Sim Engine 종료 → 재연결 시도 + 클라이언트 알림 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#backend-test-data) 참조

---

## 7. 완료 조건 (Definition of Done)

- [ ] Axum 서버 기동, `GET /health` → 200 OK (DB 연결 상태 포함)
- [ ] JWT 인증 동작 (login, refresh, me, 역할 기반 접근 제어)
- [ ] Robot CRUD API 동작 (5개 엔드포인트)
- [ ] Map CRUD API 동작 (5개 엔드포인트)
- [ ] WebSocket 연결 + 토픽 구독 + 텔레메트리 팬아웃
- [ ] Sim Engine gRPC 클라이언트 연동 (목 서버 기준)
- [ ] gRPC 텔레메트리 → WebSocket 브릿지 동작
- [ ] DB 마이그레이션 자동 적용 (3개 테이블)
- [ ] 환경변수 기반 설정 (`AppConfig`)
- [ ] CORS 설정 동작
- [ ] 단위/통합 테스트 통과 (service layer coverage 80% 이상)
- [ ] `docker-compose.yml`로 로컬 개발 환경 일괄 기동
