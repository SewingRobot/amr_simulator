# Team 2: Backend (중앙 서버)

> **문서 버전**: 1.0.0
> **최종 수정일**: 2026-03-21
> **대상 독자**: AI 에이전트 스웜 (개발 팀)
> **목적**: 이 문서만으로 백엔드 전체를 구현할 수 있도록 모든 세부 사항을 기술한다.

---

## 1. 팀 개요

### 1.1 팀 명칭
**Team 2 – Backend (Central Server)**

### 1.2 담당 범위
AMR 통합 프레임워크의 **중앙 서버**를 구축한다. 모든 클라이언트(웹 프론트엔드, 모바일, 외부 시스템)가 이 서버를 통해 데이터를 조회·조작하며, 실제 로봇 및 시뮬레이션 엔진과의 통신을 중계한다.

담당 범위는 다음과 같다:
- **REST API Gateway**: 프론트엔드 및 외부 시스템에 HTTP API를 제공
- **WebSocket Relay**: 실시간 텔레메트리·이벤트를 브라우저 클라이언트로 전달
- **MQTT / VDA5050 Bridge**: 실제 로봇과의 통신 (명령 전송, 상태 수신)
- **gRPC Client Layer**: Simulation Engine, Asset Manager, Map Manager와의 통신
- **Mission Management**: 미션 생성·할당·실행·완료 상태 머신
- **Traffic Management**: 구역 잠금, 충돌 감지, 우선순위 기반 교통 관리
- **Telemetry Pipeline**: 로봇 텔레메트리 수집·저장·배포
- **Plugin System**: WASM 기반 플러그인 런타임 및 웹훅 디스패처
- **인증/인가**: JWT 기반 인증, 역할 기반 접근 제어(RBAC)

### 1.3 핵심 목표
| 목표 | 측정 기준 |
|------|----------|
| API 응답 시간 | p95 < 100ms (CRUD), p95 < 500ms (복합 쿼리) |
| WebSocket 동시 연결 | 1,000개 이상 |
| 텔레메트리 처리량 | 100대 로봇 × 10Hz = 1,000 msg/s |
| 가용성 | 99.9% uptime |
| 테스트 커버리지 | 서비스 레이어 80% 이상 |

### 1.4 의존 관계

```
┌─────────────────────────────────────────────────────┐
│                   Frontend (Team 1)                 │
│              REST API + WebSocket 소비자              │
└──────────────────────┬──────────────────────────────┘
                       │ HTTP/WS
                       ▼
┌─────────────────────────────────────────────────────┐
│              ★ Backend (Team 2) ★                   │
│         Axum REST · WebSocket · MQTT · gRPC         │
└──┬──────────┬──────────┬──────────┬─────────────────┘
   │ gRPC     │ gRPC     │ gRPC    │ MQTT/VDA5050
   ▼          ▼          ▼         ▼
┌──────┐ ┌────────┐ ┌────────┐ ┌──────────┐
│ Sim  │ │ Asset  │ │  Map   │ │  Real    │
│Engine│ │Manager │ │Manager │ │ Robots   │
│(T3)  │ │ (T4)   │ │ (T5)   │ │          │
└──────┘ └────────┘ └────────┘ └──────────┘
```

| 의존 대상 | 통신 방식 | 역할 |
|-----------|----------|------|
| Frontend (Team 1) | REST API / WebSocket | API 소비자. Backend가 API를 제공한다. |
| Simulation Engine (Team 3) | gRPC (Backend = client) | 시뮬레이션 시작/중지, 가상 로봇 생성, 텔레메트리 스트림 수신 |
| Asset Manager (Team 4) | gRPC (Backend = client) | 3D 모델·텍스처 등 에셋 관리 위임 |
| Map Manager (Team 5) | gRPC (Backend = client) | 포인트클라우드 업로드, 타일 조회, 로드맵 경로 탐색 |
| Real Robots | MQTT (VDA5050 v2.0) | 주문(Order) 전송, 상태(State) 수신, 연결(Connection) 모니터링 |
| Proto/Integration (Team 6) | 공유 proto 파일 | `.proto` 정의를 공유하여 gRPC 코드 생성 |

---

## 2. 기술 스택 상세

### 2.1 언어 및 런타임
| 항목 | 버전/설정 |
|------|----------|
| **Rust** | Edition 2024, MSRV 1.82+ |
| **Tokio** | 1.43+ (multi-threaded runtime, `rt-multi-thread`, `macros`, `signal` features) |

### 2.2 핵심 크레이트

```toml
[package]
name = "amr-backend"
version = "0.1.0"
edition = "2024"
rust-version = "1.82"

[dependencies]
# Web Framework
axum = { version = "0.8", features = ["ws", "multipart", "macros"] }
axum-extra = { version = "0.10", features = ["typed-header", "cookie"] }
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace", "compression-gzip", "limit", "request-id"] }
hyper = { version = "1.6", features = ["full"] }

# gRPC
tonic = { version = "0.13", features = ["tls", "gzip"] }
tonic-build = "0.13"
prost = "0.13"
prost-types = "0.13"

# Database
sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls",
    "postgres",
    "uuid",
    "chrono",
    "json",
    "migrate"
] }

# Redis
fred = { version = "10", features = ["subscriber-client", "tokio-runtime"] }

# S3 / MinIO
aws-sdk-s3 = "1.72"
aws-config = { version = "1.6", features = ["behavior-version-latest"] }

# MQTT
rumqttc = { version = "0.24", features = ["use-rustls"] }

# Auth
jsonwebtoken = "9.3"
argon2 = "0.5"

# WASM Plugin Runtime
wasmtime = { version = "28", features = ["component-model"] }
wasmtime-wasi = "28"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Validation
validator = { version = "0.19", features = ["derive"] }

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.28"
opentelemetry = "0.27"
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }

# Utilities
uuid = { version = "1.12", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1.43", features = ["full"] }
dotenvy = "0.15"
thiserror = "2.0"
anyhow = "1.0"
futures = "0.3"
bytes = "1.9"

# OpenAPI Documentation
utoipa = { version = "5", features = ["axum_extras", "uuid", "chrono"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }

[build-dependencies]
tonic-build = "0.13"

[dev-dependencies]
testcontainers = "0.23"
testcontainers-modules = { version = "0.11", features = ["postgres", "redis", "mosquitto"] }
reqwest = { version = "0.12", features = ["json", "websocket"] }
tokio-tungstenite = "0.26"
wiremock = "0.6"
fake = { version = "3.1", features = ["derive", "uuid", "chrono"] }
criterion = { version = "0.5", features = ["async_tokio"] }
```

### 2.3 인프라 구성 요소

| 구성 요소 | 버전 | 용도 |
|-----------|------|------|
| **PostgreSQL** | 16+ | 주 데이터 저장소 |
| **TimescaleDB** | 2.17+ (PG16 extension) | 텔레메트리 시계열 데이터 저장 및 압축 |
| **PostGIS** | 3.5+ (PG16 extension) | 지리공간 데이터 (맵 영역, 로봇 위치) |
| **Redis** | 7.4+ | 캐시, pub/sub, 교통 구역 잠금 |
| **MinIO** | latest | S3 호환 오브젝트 스토리지 (에셋, 포인트클라우드) |
| **Eclipse Mosquitto** | 2.0+ | MQTT 브로커 (VDA5050) |
| **Docker** | 27+ | 컨테이너화 배포 |
| **Docker Compose** | 2.32+ | 로컬 개발 환경 |

---

## 3. 디렉토리 구조 상세

```
backend/
├── src/
│   ├── main.rs                 # 진입점, 서버 부트스트랩
│   ├── config.rs               # 환경 변수 파싱, AppConfig 구조체
│   ├── error.rs                # 전역 에러 타입, Axum IntoResponse 구현
│   ├── state.rs                # AppState: 공유 상태 (DB pool, Redis, clients)
│   ├── api/
│   │   ├── mod.rs              # API 모듈 re-export
│   │   ├── router.rs           # Axum Router 조립, 라우트 등록
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs         # JWT 검증 미들웨어, Claims 추출
│   │   │   ├── cors.rs         # CORS 설정
│   │   │   └── rate_limit.rs   # Token bucket 기반 요청 제한
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs         # 로그인, 토큰 갱신, 사용자 정보 조회
│   │   │   ├── robots.rs       # 로봇 CRUD + 상태 조회
│   │   │   ├── missions.rs     # 미션 CRUD + 할당 + 상태 전이
│   │   │   ├── maps.rs         # 맵 CRUD
│   │   │   ├── roadmap.rs      # 로드맵 그래프 조회/수정
│   │   │   ├── semantic.rs     # 시맨틱 레이어 (구역 정의)
│   │   │   ├── assets.rs       # 에셋 프록시 (Asset Manager gRPC 위임)
│   │   │   ├── users.rs        # 사용자 관리 (관리자 전용)
│   │   │   ├── plugins.rs      # 플러그인 CRUD, 활성화/비활성화
│   │   │   └── tiles.rs        # 포인트클라우드 타일 프록시
│   │   ├── ws/
│   │   │   ├── mod.rs
│   │   │   ├── handler.rs      # WebSocket 업그레이드 핸들러
│   │   │   ├── session.rs      # 클라이언트별 세션, 토픽 구독 관리
│   │   │   └── messages.rs     # WebSocket 메시지 타입 정의
│   │   └── dto/
│   │       ├── mod.rs
│   │       ├── auth.rs         # LoginRequest, LoginResponse, TokenResponse
│   │       ├── robots.rs       # RobotCreateRequest, RobotResponse, etc.
│   │       ├── missions.rs     # MissionCreateRequest, MissionResponse, etc.
│   │       ├── maps.rs         # MapCreateRequest, MapResponse, etc.
│   │       ├── assets.rs       # AssetResponse, etc.
│   │       ├── users.rs        # UserCreateRequest, UserResponse, etc.
│   │       ├── plugins.rs      # PluginResponse, PluginConfigRequest, etc.
│   │       ├── pagination.rs   # PaginationParams, PaginatedResponse<T>
│   │       └── common.rs       # ErrorResponse, SuccessResponse
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth_service.rs     # 비밀번호 해싱(Argon2), JWT 생성/검증
│   │   ├── mission_service.rs  # 미션 상태 머신, VDA5050 주문 생성
│   │   ├── traffic_service.rs  # 구역 잠금, 충돌 감지, 우선순위
│   │   ├── robot_service.rs    # 로봇 레지스트리, 상태 관리
│   │   └── telemetry_service.rs # 텔레메트리 수집, 팬아웃, 저장
│   ├── mqtt/
│   │   ├── mod.rs
│   │   ├── client.rs           # MQTT 연결 관리, 자동 재연결
│   │   ├── vda5050.rs          # VDA5050 메시지 타입 (Order, State, Connection 등)
│   │   ├── topics.rs           # 토픽 네이밍 규칙
│   │   └── bridge.rs           # MQTT ↔ 내부 이벤트 버스 브릿지
│   ├── grpc/
│   │   ├── mod.rs
│   │   ├── sim_client.rs       # Simulation Engine gRPC 클라이언트
│   │   ├── asset_client.rs     # Asset Manager gRPC 클라이언트
│   │   └── map_client.rs       # Map Manager gRPC 클라이언트
│   ├── plugins/
│   │   ├── mod.rs
│   │   ├── wasm_host.rs        # Wasmtime 런타임, 플러그인 라이프사이클
│   │   ├── webhook.rs          # HTTP 웹훅 디스패처
│   │   └── plugin_api.rs       # 플러그인에 노출되는 호스트 API
│   ├── db/
│   │   ├── mod.rs
│   │   ├── pool.rs             # PostgreSQL 커넥션 풀 설정
│   │   ├── queries/
│   │   │   ├── mod.rs
│   │   │   ├── robots.rs       # 로봇 테이블 쿼리
│   │   │   ├── missions.rs     # 미션/미션스텝 테이블 쿼리
│   │   │   ├── maps.rs         # 맵 테이블 쿼리
│   │   │   ├── users.rs        # 사용자 테이블 쿼리
│   │   │   ├── assets.rs       # 에셋 메타데이터 쿼리
│   │   │   ├── telemetry.rs    # 텔레메트리 hypertable 쿼리
│   │   │   ├── roadmap.rs      # 로드맵 그래프 쿼리
│   │   │   ├── semantic.rs     # 시맨틱 영역 쿼리
│   │   │   └── plugins.rs      # 플러그인 테이블 쿼리
│   │   └── models.rs           # 데이터베이스 모델 구조체 (sqlx::FromRow)
│   └── events/
│       ├── mod.rs
│       ├── bus.rs              # 내부 이벤트 버스 (tokio::sync::broadcast)
│       └── types.rs            # 이벤트 타입 열거형
├── proto/                      # 공유 proto 파일 (git submodule 또는 복사)
│   ├── simulation.proto
│   ├── asset.proto
│   └── map.proto
├── migrations/
│   ├── 001_create_users.sql
│   ├── 002_create_robots.sql
│   ├── 003_create_maps.sql
│   ├── 004_create_roadmap.sql
│   ├── 005_create_semantic_regions.sql
│   ├── 006_create_missions.sql
│   ├── 007_create_mission_steps.sql
│   ├── 008_create_assets.sql
│   ├── 009_create_telemetry.sql
│   └── 010_create_plugins.sql
├── tests/
│   ├── common/
│   │   └── mod.rs              # 테스트 헬퍼, 테스트 DB 설정
│   ├── api/
│   │   ├── test_auth.rs
│   │   ├── test_robots.rs
│   │   ├── test_missions.rs
│   │   ├── test_maps.rs
│   │   └── test_users.rs
│   ├── services/
│   │   ├── test_mission_service.rs
│   │   ├── test_traffic_service.rs
│   │   └── test_auth_service.rs
│   ├── integration/
│   │   ├── test_mqtt_bridge.rs
│   │   ├── test_websocket.rs
│   │   └── test_telemetry_pipeline.rs
│   └── mocks/
│       ├── mock_sim_engine.rs
│       ├── mock_asset_manager.rs
│       └── mock_map_manager.rs
├── build.rs                    # tonic-build proto 코드 생성
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── docker-compose.yml          # 로컬 개발용 인프라
├── .env.example
└── sqlx-data.json              # SQLx 오프라인 모드용 쿼리 캐시
```

---

## 4. 구성 모듈 상세 스펙

### 4.0 진입점 및 공유 상태

#### `src/main.rs` – 서버 부트스트랩

서버 초기화 순서:
1. `dotenvy::dotenv()` 로드
2. `tracing_subscriber` 초기화 (OpenTelemetry 연동)
3. `AppConfig` 파싱
4. PostgreSQL 커넥션 풀 생성 + 마이그레이션 실행
5. Redis 클라이언트 연결
6. MQTT 클라이언트 연결 + VDA5050 토픽 구독
7. gRPC 클라이언트 연결 (Sim, Asset, Map)
8. 이벤트 버스 생성
9. WASM 플러그인 런타임 초기화
10. `AppState` 조립
11. Axum Router 빌드 + 미들웨어 적용
12. `axum::serve()` 바인딩 (graceful shutdown 포함)

```rust
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tracing::info;

mod api;
mod config;
mod db;
mod error;
mod events;
mod grpc;
mod mqtt;
mod plugins;
mod services;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // 트레이싱 초기화
    let _guard = observability::init_tracing()?;

    let config = config::AppConfig::from_env()?;
    info!("Starting AMR Backend v{}", env!("CARGO_PKG_VERSION"));

    // 데이터베이스
    let db_pool = db::pool::create_pool(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    // Redis
    let redis_client = fred::clients::Client::default();
    redis_client.init().await?;

    // MQTT
    let mqtt_client = mqtt::client::create_mqtt_client(&config).await?;

    // gRPC 클라이언트
    let sim_client = grpc::sim_client::SimClient::connect(&config.grpc_sim_engine_url).await?;
    let asset_client = grpc::asset_client::AssetClient::connect(&config.grpc_asset_manager_url).await?;
    let map_client = grpc::map_client::MapClient::connect(&config.grpc_map_manager_url).await?;

    // 이벤트 버스
    let event_bus = events::bus::EventBus::new(10_000);

    // 플러그인 런타임
    let plugin_runtime = plugins::wasm_host::WasmHost::new()?;

    // AppState 조립
    let state = state::AppState {
        config: Arc::new(config.clone()),
        db: db_pool,
        redis: redis_client,
        mqtt: mqtt_client,
        sim_client: Arc::new(tokio::sync::Mutex::new(sim_client)),
        asset_client: Arc::new(tokio::sync::Mutex::new(asset_client)),
        map_client: Arc::new(tokio::sync::Mutex::new(map_client)),
        event_bus: Arc::new(event_bus),
        plugin_runtime: Arc::new(tokio::sync::Mutex::new(plugin_runtime)),
    };

    // MQTT 브릿지 시작
    mqtt::bridge::start_bridge(state.clone()).await?;

    // 텔레메트리 서비스 시작
    services::telemetry_service::start_ingestion(state.clone()).await?;

    // 라우터 빌드
    let app = api::router::build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async { signal::ctrl_c().await.unwrap() };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("Shutdown signal received");
}
```

#### `src/config.rs` – 설정

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_port")]
    pub port: u16,

    pub database_url: String,
    pub redis_url: String,

    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub minio_bucket: String,

    pub mqtt_broker_url: String,
    #[serde(default = "default_mqtt_client_id")]
    pub mqtt_client_id: String,

    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration_secs: u64,
    #[serde(default = "default_jwt_refresh_expiration")]
    pub jwt_refresh_expiration_secs: u64,

    pub grpc_sim_engine_url: String,
    pub grpc_asset_manager_url: String,
    pub grpc_map_manager_url: String,

    #[serde(default = "default_otel_endpoint")]
    pub otel_exporter_otlp_endpoint: String,

    #[serde(default = "default_zone_lock_timeout")]
    pub zone_lock_timeout_secs: u64,

    #[serde(default = "default_ws_heartbeat_interval")]
    pub ws_heartbeat_interval_secs: u64,

    #[serde(default = "default_ws_max_missed_pongs")]
    pub ws_max_missed_pongs: u32,

    #[serde(default = "default_telemetry_retention_days")]
    pub telemetry_retention_days: i32,

    #[serde(default = "default_telemetry_compression_days")]
    pub telemetry_compression_after_days: i32,
}

fn default_port() -> u16 { 8080 }
fn default_mqtt_client_id() -> String { "amr-backend".to_string() }
fn default_jwt_expiration() -> u64 { 3600 }
fn default_jwt_refresh_expiration() -> u64 { 604800 }
fn default_otel_endpoint() -> String { "http://localhost:4317".to_string() }
fn default_zone_lock_timeout() -> u64 { 60 }
fn default_ws_heartbeat_interval() -> u64 { 30 }
fn default_ws_max_missed_pongs() -> u32 { 3 }
fn default_telemetry_retention_days() -> i32 { 30 }
fn default_telemetry_compression_days() -> i32 { 7 }

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = envy::from_env::<AppConfig>()?;
        Ok(config)
    }
}
```

#### `src/state.rs` – 공유 상태

```rust
use std::sync::Arc;
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::config::AppConfig;
use crate::events::bus::EventBus;
use crate::grpc::{sim_client::SimClient, asset_client::AssetClient, map_client::MapClient};
use crate::plugins::wasm_host::WasmHost;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: PgPool,
    pub redis: fred::clients::Client,
    pub mqtt: rumqttc::AsyncClient,
    pub sim_client: Arc<Mutex<SimClient>>,
    pub asset_client: Arc<Mutex<AssetClient>>,
    pub map_client: Arc<Mutex<MapClient>>,
    pub event_bus: Arc<EventBus>,
    pub plugin_runtime: Arc<Mutex<WasmHost>>,
}
```

#### `src/error.rs` – 전역 에러 타입

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("리소스를 찾을 수 없습니다: {0}")]
    NotFound(String),

    #[error("인증이 필요합니다: {0}")]
    Unauthorized(String),

    #[error("접근 권한이 없습니다: {0}")]
    Forbidden(String),

    #[error("유효성 검사 실패: {0}")]
    Validation(String),

    #[error("충돌: {0}")]
    Conflict(String),

    #[error("내부 서버 오류: {0}")]
    Internal(String),

    #[error("서비스를 사용할 수 없습니다: {0}")]
    ServiceUnavailable(String),

    #[error("요청이 너무 많습니다")]
    TooManyRequests,

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "내부 서버 오류가 발생했습니다".to_string())
            }
            AppError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE", msg.clone()),
            AppError::TooManyRequests => (StatusCode::TOO_MANY_REQUESTS, "TOO_MANY_REQUESTS", "요청이 너무 많습니다".to_string()),
            AppError::Sqlx(e) => {
                tracing::error!("Database error: {:?}", e);
                match e {
                    sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", "리소스를 찾을 수 없습니다".to_string()),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "데이터베이스 오류가 발생했습니다".to_string()),
                }
            }
            AppError::Anyhow(e) => {
                tracing::error!("Unexpected error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "내부 서버 오류가 발생했습니다".to_string())
            }
        };

        let body = ErrorResponse {
            error: error_type.to_string(),
            message,
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

---

### 4.1 API Gateway (REST)

#### `src/api/router.rs` – 라우터 조립

```rust
use axum::{
    middleware,
    Router,
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer, PropagateRequestIdLayer},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::state::AppState;
use super::middleware as app_middleware;
use super::handlers;

pub fn build_router(state: AppState) -> Router {
    let public_routes = Router::new()
        .nest("/api/auth", handlers::auth::routes())
        .nest("/api/ws", super::ws::handler::routes());

    let protected_routes = Router::new()
        .nest("/api/robots", handlers::robots::routes())
        .nest("/api/missions", handlers::missions::routes())
        .nest("/api/maps", handlers::maps::routes())
        .nest("/api/assets", handlers::assets::routes())
        .nest("/api/users", handlers::users::routes())
        .nest("/api/plugins", handlers::plugins::routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            app_middleware::auth::jwt_auth,
        ));

    let x_request_id = tower::ServiceBuilder::new()
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id());

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(app_middleware::cors::cors_layer())
        .layer(x_request_id)
        .with_state(state)
}

#[derive(OpenApi)]
#[openapi(
    paths(/* 모든 handler 함수 등록 */),
    components(schemas(/* 모든 DTO 등록 */)),
    tags(
        (name = "auth", description = "인증 API"),
        (name = "robots", description = "로봇 관리 API"),
        (name = "missions", description = "미션 관리 API"),
        (name = "maps", description = "맵 관리 API"),
        (name = "assets", description = "에셋 관리 API"),
        (name = "users", description = "사용자 관리 API"),
        (name = "plugins", description = "플러그인 관리 API"),
    )
)]
struct ApiDoc;
```

#### `src/api/middleware/auth.rs` – JWT 인증 미들웨어

```rust
use axum::{
    extract::{State, Request},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,        // user_id
    pub email: String,
    pub role: String,     // "admin" | "operator" | "viewer"
    pub exp: usize,       // expiration timestamp
    pub iat: usize,       // issued at
    pub token_type: String, // "access" | "refresh"
}

pub async fn jwt_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Authorization 헤더가 필요합니다".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Bearer 토큰 형식이 아닙니다".into()))?;

    let decoding_key = DecodingKey::from_secret(state.config.jwt_secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("토큰 검증 실패: {}", e)))?;

    if token_data.claims.token_type != "access" {
        return Err(AppError::Unauthorized("access 토큰이 아닙니다".into()));
    }

    // Claims를 request extensions에 삽입하여 핸들러에서 접근 가능하게 함
    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}

/// 역할 기반 접근 제어를 위한 헬퍼
pub fn require_role(claims: &Claims, required: &[&str]) -> Result<(), AppError> {
    if required.contains(&claims.role.as_str()) {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "이 작업에는 {:?} 역할이 필요합니다",
            required
        )))
    }
}
```

#### `src/api/middleware/cors.rs`

```rust
use tower_http::cors::{CorsLayer, Any};
use axum::http::{header, Method};

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any) // 프로덕션에서는 특정 도메인으로 제한
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .expose_headers([header::CONTENT_DISPOSITION])
        .max_age(std::time::Duration::from_secs(3600))
}
```

#### `src/api/middleware/rate_limit.rs`

```rust
use axum::{
    extract::{ConnectInfo, State, Request},
    middleware::Next,
    response::Response,
};
use fred::prelude::*;
use std::net::SocketAddr;
use std::time::Duration;

use crate::error::AppError;
use crate::state::AppState;

/// Token bucket 기반 요청 제한
/// - 기본: IP당 100 요청/분
/// - 인증 사용자: 300 요청/분
/// - 관리자: 1000 요청/분
pub async fn rate_limit(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let key = format!("rate_limit:{}", addr.ip());
    let limit: u64 = 100; // 기본 제한
    let window_secs: i64 = 60;

    let current: u64 = state.redis.incr(&key).await.unwrap_or(1);

    if current == 1 {
        let _: () = state.redis.expire(&key, window_secs).await.unwrap_or(());
    }

    if current > limit {
        return Err(AppError::TooManyRequests);
    }

    Ok(next.run(req).await)
}
```

---

#### 4.1.1 Auth Endpoints 상세

**파일**: `src/api/handlers/auth.rs`

##### `POST /api/auth/login`

| 항목 | 값 |
|------|---|
| **메서드** | POST |
| **경로** | `/api/auth/login` |
| **인증 필요** | 아니오 |
| **역할 제한** | 없음 |
| **요청 Content-Type** | `application/json` |

**요청 스키마**:
```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "유효한 이메일 주소를 입력하세요"))]
    pub email: String,
    #[validate(length(min = 8, message = "비밀번호는 최소 8자 이상이어야 합니다"))]
    pub password: String,
}
```

**응답 스키마**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,      // 항상 "Bearer"
    pub expires_in: u64,          // 초 단위
    pub user: UserInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}
```

**상태 코드**:
| 코드 | 설명 |
|------|------|
| 200 | 로그인 성공 |
| 400 | 유효성 검사 실패 (이메일 형식, 비밀번호 길이) |
| 401 | 이메일 또는 비밀번호 불일치 |
| 429 | 요청 제한 초과 |
| 500 | 내부 서버 오류 |

**비즈니스 로직**:
1. 요청 본문 유효성 검사 (validator)
2. 이메일로 사용자 조회 (`db::queries::users::find_by_email`)
3. Argon2로 비밀번호 해시 검증
4. access_token 생성 (JWT, exp = 설정값, token_type = "access")
5. refresh_token 생성 (JWT, exp = 7일, token_type = "refresh")
6. `LoginResponse` 반환

```rust
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let user = db::queries::users::find_by_email(&state.db, &req.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("이메일 또는 비밀번호가 일치하지 않습니다".into()))?;

    let is_valid = services::auth_service::verify_password(&req.password, &user.password_hash)?;
    if !is_valid {
        return Err(AppError::Unauthorized("이메일 또는 비밀번호가 일치하지 않습니다".into()));
    }

    let access_token = services::auth_service::generate_token(
        &state.config, &user, "access", state.config.jwt_expiration_secs,
    )?;
    let refresh_token = services::auth_service::generate_token(
        &state.config, &user, "refresh", state.config.jwt_refresh_expiration_secs,
    )?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiration_secs,
        user: UserInfo {
            id: user.id,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
        },
    }))
}
```

##### `POST /api/auth/refresh`

| 항목 | 값 |
|------|---|
| **메서드** | POST |
| **경로** | `/api/auth/refresh` |
| **인증 필요** | 아니오 (refresh_token 자체가 인증 수단) |
| **역할 제한** | 없음 |

**요청 스키마**:
```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
```

**응답 스키마**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}
```

**상태 코드**:
| 코드 | 설명 |
|------|------|
| 200 | 토큰 갱신 성공 |
| 401 | refresh_token이 만료되었거나 유효하지 않음 |

**비즈니스 로직**:
1. refresh_token을 JWT로 디코딩/검증
2. `token_type`이 "refresh"인지 확인
3. `sub`(user_id)로 사용자 존재 확인
4. 새 access_token 생성 후 반환

##### `GET /api/auth/me`

| 항목 | 값 |
|------|---|
| **메서드** | GET |
| **경로** | `/api/auth/me` |
| **인증 필요** | 예 (JWT) |
| **역할 제한** | 없음 (인증된 모든 사용자) |

**응답 스키마**: `UserInfo` (위와 동일)

**상태 코드**:
| 코드 | 설명 |
|------|------|
| 200 | 성공 |
| 401 | 인증 실패 |

**비즈니스 로직**:
1. JWT Claims에서 `sub`(user_id) 추출
2. 사용자 정보 조회
3. `UserInfo` 반환

---

#### 4.1.2 Robot Endpoints 상세

**파일**: `src/api/handlers/robots.rs`

##### `GET /api/robots`

| 항목 | 값 |
|------|---|
| **메서드** | GET |
| **경로** | `/api/robots` |
| **인증 필요** | 예 |
| **역할 제한** | viewer, operator, admin |

**쿼리 파라미터**:
```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct RobotListParams {
    pub status: Option<String>,    // IDLE, BUSY, OFFLINE, ERROR
    pub page: Option<u32>,         // 기본값 1
    pub limit: Option<u32>,        // 기본값 20, 최대 100
    pub search: Option<String>,    // 이름으로 검색
    pub sort_by: Option<String>,   // name, status, created_at
    pub sort_order: Option<String>,// asc, desc
}
```

**응답 스키마**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RobotResponse {
    pub id: Uuid,
    pub name: String,
    pub model_id: String,
    pub serial_number: String,
    pub manufacturer: String,
    pub status: String,               // IDLE, BUSY, OFFLINE, ERROR
    pub battery_level: Option<f64>,    // 0.0 ~ 100.0
    pub position: Option<RobotPosition>,
    pub current_mission_id: Option<Uuid>,
    pub config: serde_json::Value,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RobotPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub theta: f64,  // 방향 (라디안)
    pub map_id: Uuid,
}
```

**상태 코드**:
| 코드 | 설명 |
|------|------|
| 200 | 성공 |
| 400 | 잘못된 쿼리 파라미터 |
| 401 | 인증 실패 |

##### `POST /api/robots`

| 항목 | 값 |
|------|---|
| **메서드** | POST |
| **경로** | `/api/robots` |
| **인증 필요** | 예 |
| **역할 제한** | admin만 |

**요청 스키마**:
```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RobotCreateRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub model_id: String,
    #[validate(length(min = 1, max = 100))]
    pub serial_number: String,
    #[validate(length(min = 1, max = 100))]
    pub manufacturer: String,
    pub config: Option<serde_json::Value>,
}
```

**상태 코드**:
| 코드 | 설명 |
|------|------|
| 201 | 생성 성공 |
| 400 | 유효성 검사 실패 |
| 401 | 인증 실패 |
| 403 | admin 권한 필요 |
| 409 | serial_number 중복 |

**비즈니스 로직**:
1. Claims에서 역할 확인 (`require_role(claims, &["admin"])`)
2. 유효성 검사
3. serial_number 중복 확인
4. DB에 로봇 레코드 삽입 (초기 status = "OFFLINE")
5. 이벤트 버스에 `RobotCreated` 이벤트 발행
6. 생성된 로봇 반환 (201 Created)

##### `GET /api/robots/:id`

| 항목 | 값 |
|------|---|
| **메서드** | GET |
| **경로** | `/api/robots/:id` |
| **인증 필요** | 예 |
| **역할 제한** | viewer, operator, admin |

**상태 코드**: 200 성공, 404 로봇 없음

##### `PUT /api/robots/:id`

| 항목 | 값 |
|------|---|
| **메서드** | PUT |
| **경로** | `/api/robots/:id` |
| **인증 필요** | 예 |
| **역할 제한** | admin만 |

**요청 스키마**:
```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RobotUpdateRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
}
```

**상태 코드**: 200 성공, 400 유효성 오류, 403 권한 없음, 404 로봇 없음

##### `DELETE /api/robots/:id`

| 항목 | 값 |
|------|---|
| **메서드** | DELETE |
| **경로** | `/api/robots/:id` |
| **인증 필요** | 예 |
| **역할 제한** | admin만 |

**상태 코드**: 204 삭제 성공, 403 권한 없음, 404 로봇 없음, 409 활성 미션이 있어 삭제 불가

**비즈니스 로직**: 활성 미션(ASSIGNED, EXECUTING)이 있는 로봇은 삭제할 수 없다. 확인 후 소프트 삭제 또는 하드 삭제.

##### `GET /api/robots/:id/telemetry`

| 항목 | 값 |
|------|---|
| **메서드** | GET |
| **경로** | `/api/robots/:id/telemetry` |
| **인증 필요** | 예 |
| **역할 제한** | viewer, operator, admin |

**쿼리 파라미터**:
```rust
#[derive(Debug, Deserialize)]
pub struct TelemetryQueryParams {
    pub from: DateTime<Utc>,        // 시작 시간 (필수)
    pub to: DateTime<Utc>,          // 종료 시간 (필수)
    pub interval: Option<String>,   // 다운샘플링 간격: "1s", "10s", "1m", "5m", "1h"
    pub limit: Option<u32>,         // 최대 반환 개수, 기본 1000
}
```

**응답 스키마**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct TelemetryPoint {
    pub timestamp: DateTime<Utc>,
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
    pub orientation_theta: f64,
    pub velocity_linear: f64,
    pub velocity_angular: f64,
    pub battery_level: f64,
    pub battery_charging: bool,
    pub errors: Vec<RobotError>,
    pub operating_mode: String,
}
```

---

#### 4.1.3 Mission Endpoints 상세

**파일**: `src/api/handlers/missions.rs`

##### `GET /api/missions`

| 항목 | 값 |
|------|---|
| **메서드** | GET |
| **경로** | `/api/missions` |
| **인증 필요** | 예 |
| **역할 제한** | viewer, operator, admin |

**쿼리 파라미터**:
```rust
#[derive(Debug, Deserialize)]
pub struct MissionListParams {
    pub status: Option<String>,    // CREATED, ASSIGNED, EXECUTING, COMPLETED, FAILED, CANCELLED
    pub robot_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,   // created_at, priority, status
    pub sort_order: Option<String>,
}
```

**응답**: `PaginatedResponse<MissionResponse>`

```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct MissionResponse {
    pub id: Uuid,
    pub robot_id: Option<Uuid>,
    pub mission_type: String,       // TRANSPORT, PATROL, CHARGE, CUSTOM
    pub status: String,             // CREATED, ASSIGNED, EXECUTING, COMPLETED, FAILED, CANCELLED
    pub priority: i32,              // 1 (최저) ~ 10 (최고), 기본 5
    pub steps: Vec<MissionStepResponse>,
    pub progress: f64,              // 0.0 ~ 1.0
    pub error_message: Option<String>,
    pub created_by: Uuid,
    pub assigned_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MissionStepResponse {
    pub id: Uuid,
    pub sequence: i32,
    pub step_type: String,          // NAVIGATE, WAIT, ACTION, CHARGE
    pub target_node_id: Option<String>,
    pub parameters: serde_json::Value,
    pub status: String,             // PENDING, EXECUTING, COMPLETED, FAILED, SKIPPED
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

##### `POST /api/missions`

| 항목 | 값 |
|------|---|
| **메서드** | POST |
| **경로** | `/api/missions` |
| **인증 필요** | 예 |
| **역할 제한** | operator, admin |

**요청 스키마**:
```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MissionCreateRequest {
    pub robot_id: Option<Uuid>,     // 선택적 – 나중에 assign 가능
    #[validate(length(min = 1, max = 50))]
    pub mission_type: String,       // TRANSPORT, PATROL, CHARGE, CUSTOM
    #[validate(range(min = 1, max = 10))]
    pub priority: Option<i32>,      // 기본값 5
    #[validate(length(min = 1))]
    pub steps: Vec<MissionStepCreate>,
    pub map_id: Uuid,               // 미션이 실행될 맵
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MissionStepCreate {
    pub sequence: i32,
    pub step_type: String,          // NAVIGATE, WAIT, ACTION, CHARGE
    pub target_node_id: Option<String>, // 로드맵 노드 ID
    pub parameters: Option<serde_json::Value>,
}
```

**상태 코드**:
| 코드 | 설명 |
|------|------|
| 201 | 미션 생성 성공 |
| 400 | 유효성 검사 실패 (빈 steps, 잘못된 step_type 등) |
| 403 | operator/admin 권한 필요 |
| 404 | robot_id가 지정되었으나 해당 로봇 없음 |

**비즈니스 로직**:
1. 유효성 검사
2. robot_id가 지정된 경우 로봇 존재 확인
3. map_id로 맵 존재 확인
4. steps의 target_node_id가 로드맵에 존재하는지 확인 (MapClient gRPC 호출)
5. DB에 mission + mission_steps 삽입 (트랜잭션)
6. 초기 status = "CREATED", robot_id가 있으면 "ASSIGNED"
7. 이벤트 버스에 `MissionCreated` 이벤트 발행
8. 플러그인 `on_mission_created` 호출

##### `POST /api/missions/:id/assign`

| 항목 | 값 |
|------|---|
| **메서드** | POST |
| **경로** | `/api/missions/:id/assign` |
| **인증 필요** | 예 |
| **역할 제한** | operator, admin |

**요청 스키마**:
```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct MissionAssignRequest {
    pub robot_id: Uuid,
}
```

**상태 코드**:
| 코드 | 설명 |
|------|------|
| 200 | 할당 성공 |
| 400 | 미션이 CREATED 상태가 아님 |
| 404 | 미션 또는 로봇 없음 |
| 409 | 로봇이 이미 다른 미션을 실행 중 |

**비즈니스 로직**:
1. 미션 상태가 CREATED인지 확인
2. 로봇이 IDLE 상태인지 확인
3. TrafficService를 통해 경로 충돌 검사
4. 미션 상태를 ASSIGNED로 변경, robot_id 설정
5. VDA5050 Order 메시지 생성
6. MQTT를 통해 로봇에 Order 전송
7. 로봇 상태를 BUSY로 변경
8. 미션 상태를 EXECUTING으로 변경
9. 이벤트 발행

##### `POST /api/missions/:id/cancel`

| 항목 | 값 |
|------|---|
| **메서드** | POST |
| **경로** | `/api/missions/:id/cancel` |
| **인증 필요** | 예 |
| **역할 제한** | operator, admin |

**상태 코드**: 200 취소 성공, 400 이미 완료/실패/취소됨

**비즈니스 로직**:
1. 미션 상태가 CREATED, ASSIGNED, EXECUTING 중 하나인지 확인
2. EXECUTING 상태라면 VDA5050 instantAction(cancelOrder) 전송
3. 미션 상태를 CANCELLED로 변경
4. 로봇이 할당되어 있으면 상태를 IDLE로 복원
5. TrafficService에서 관련 구역 잠금 해제
6. 이벤트 발행

##### `PUT /api/missions/:id`

CREATED 상태에서만 수정 가능. priority, steps 변경.

##### `DELETE /api/missions/:id`

CREATED 상태에서만 삭제 가능.

---

#### 4.1.4 Map Endpoints 상세

**파일**: `src/api/handlers/maps.rs`

##### `GET /api/maps`

| 항목 | 값 |
|------|---|
| **메서드** | GET |
| **경로** | `/api/maps` |
| **인증** | 예 |
| **역할** | 전체 |

**응답**: `Vec<MapResponse>`

```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct MapResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,             // EMPTY, PROCESSING, READY, ERROR
    pub bounds: Option<MapBounds>,
    pub point_cloud_size_bytes: Option<i64>,
    pub tile_count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MapBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}
```

##### `POST /api/maps`

**요청**:
```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct MapCreateRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
}
```

**상태 코드**: 201 생성, 400 유효성, 409 이름 중복

##### `GET /api/maps/:id/roadmap`

**응답**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct RoadmapResponse {
    pub map_id: Uuid,
    pub nodes: Vec<RoadmapNode>,
    pub edges: Vec<RoadmapEdge>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RoadmapNode {
    pub node_id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub name: Option<String>,
    pub node_type: String,          // WAYPOINT, STATION, CHARGER, ELEVATOR
    pub properties: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RoadmapEdge {
    pub edge_id: String,
    pub start_node_id: String,
    pub end_node_id: String,
    pub bidirectional: bool,
    pub weight: f64,                // 가중치 (보통 거리)
    pub max_speed: Option<f64>,
    pub properties: serde_json::Value,
}
```

로드맵 데이터는 Map Manager gRPC를 통해 조회하고, 로컬 DB에도 캐시 저장.

##### `PUT /api/maps/:id/roadmap`

**요청**: `RoadmapUpdateRequest { nodes: Vec<RoadmapNode>, edges: Vec<RoadmapEdge> }`

Map Manager gRPC로 업데이트 위임 후 로컬 캐시 갱신.

##### `GET /api/maps/:id/semantic`

**응답**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct SemanticLayerResponse {
    pub map_id: Uuid,
    pub regions: Vec<SemanticRegion>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SemanticRegion {
    pub region_id: Uuid,
    pub name: String,
    pub region_type: String,        // ZONE, RESTRICTED, CHARGING, LOADING, STORAGE
    pub polygon: Vec<[f64; 2]>,     // [[x,y], [x,y], ...] 다각형 꼭짓점
    pub properties: serde_json::Value,
    pub max_robots: Option<i32>,    // 동시 진입 가능 로봇 수
    pub speed_limit: Option<f64>,   // m/s
}
```

##### `POST /api/maps/:id/pointcloud/upload`

| 항목 | 값 |
|------|---|
| **메서드** | POST |
| **경로** | `/api/maps/:id/pointcloud/upload` |
| **Content-Type** | `multipart/form-data` |
| **인증** | 예, admin/operator |

**비즈니스 로직**:
1. multipart 스트림 수신
2. Map Manager gRPC의 `UploadPointCloud` 스트리밍 RPC로 청크 전달
3. processing_job_id 반환
4. 맵 상태를 PROCESSING으로 변경

**응답**: `{ "processing_job_id": "uuid" }`

##### `GET /api/maps/:id/pointcloud/status`

Map Manager gRPC의 `GetProcessingStatus` 호출하여 진행률 반환.

**응답**: `{ "status": "PROCESSING", "progress": 0.45, "message": "Building octree..." }`

##### `GET /api/maps/:id/tiles/:nodeId`

| 항목 | 값 |
|------|---|
| **쿼리 파라미터** | `lod` (Level of Detail, 0-10, 기본 0) |
| **응답 Content-Type** | `application/octet-stream` |

Map Manager gRPC의 `GetTile` 호출하여 바이너리 타일 데이터를 프록시 반환. 캐시 헤더 포함 (`Cache-Control: public, max-age=3600`).

---

#### 4.1.5 Asset Endpoints 상세

**파일**: `src/api/handlers/assets.rs`

모든 에셋 요청은 Asset Manager gRPC로 프록시한다.

##### `GET /api/assets`

**쿼리**: `{ type?: "robot_model" | "texture" | "scene", format?: "glb" | "gltf" | "fbx", page?, limit? }`

gRPC `ListAssets` 호출 → 응답 변환.

##### `POST /api/assets/upload`

multipart 수신 → gRPC `UploadAsset` 스트리밍 RPC로 전달.

##### `GET /api/assets/:id/download`

gRPC `GetAsset` → 바이너리 스트림 → HTTP 응답 (`Content-Disposition: attachment`).

##### `DELETE /api/assets/:id`

gRPC `DeleteAsset` 호출. admin만 가능.

---

#### 4.1.6 User Endpoints 상세

**파일**: `src/api/handlers/users.rs`

모든 엔드포인트는 **admin 전용**.

##### `GET /api/users`

**응답**: `Vec<UserResponse>` (password_hash 제외)

##### `POST /api/users`

```rust
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UserCreateRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    pub role: String,  // admin, operator, viewer
}
```

비밀번호를 Argon2로 해싱한 후 DB 삽입.

##### `PUT /api/users/:id`

역할 변경, 이메일 변경. 비밀번호 변경은 별도 엔드포인트 고려.

##### `DELETE /api/users/:id`

자기 자신은 삭제 불가. 다른 admin 사용자 삭제 시 최소 1명의 admin이 남아야 함.

---

#### 4.1.7 Plugin Endpoints 상세

**파일**: `src/api/handlers/plugins.rs`

##### `GET /api/plugins`

**응답**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct PluginResponse {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub plugin_type: String,        // WASM, WEBHOOK
    pub enabled: bool,
    pub config: serde_json::Value,
    pub hooks: Vec<String>,         // ["on_mission_created", "on_robot_state_changed", ...]
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

##### `POST /api/plugins`

multipart로 WASM 바이너리 업로드. Wasmtime으로 검증 (필수 인터페이스 구현 확인) 후 MinIO에 저장, DB에 메타데이터 등록.

##### `PUT /api/plugins/:id/config`

```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct PluginConfigRequest {
    pub config: serde_json::Value,
}
```

##### `POST /api/plugins/:id/enable` / `POST /api/plugins/:id/disable`

플러그인 활성화/비활성화. 활성화 시 WASM 모듈을 Wasmtime에 로드.

---

### 4.2 WebSocket Relay 상세

#### 연결 흐름

1. 클라이언트가 `ws://host/api/ws?token={JWT}` 로 WebSocket 업그레이드 요청
2. 서버가 JWT 토큰 검증
3. 검증 성공 시 WebSocket 연결 수립, `WsSession` 생성
4. 클라이언트가 subscribe 메시지를 보내 원하는 토픽 구독
5. 서버가 해당 토픽의 이벤트를 클라이언트로 팬아웃

#### 메시지 포맷

```rust
/// 서버 → 클라이언트 메시지
#[derive(Debug, Serialize, Deserialize)]
pub struct WsOutgoingMessage {
    pub msg_type: String,           // "data", "error", "subscribed", "unsubscribed", "pong"
    pub topic: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub timestamp: i64,             // Unix timestamp (milliseconds)
}

/// 클라이언트 → 서버 메시지
#[derive(Debug, Serialize, Deserialize)]
pub struct WsIncomingMessage {
    pub action: String,             // "subscribe", "unsubscribe", "ping"
    pub topic: Option<String>,
}
```

#### 토픽 체계

| 토픽 패턴 | 설명 | 페이로드 |
|-----------|------|---------|
| `telemetry:{robot_id}` | 특정 로봇의 실시간 텔레메트리 | `TelemetryPoint` |
| `telemetry:*` | 모든 로봇의 텔레메트리 | `{ robot_id, ...TelemetryPoint }` |
| `missions` | 모든 미션 상태 변경 | `MissionResponse` |
| `missions:{mission_id}` | 특정 미션 상태 변경 | `MissionResponse` |
| `alerts` | 시스템 알림 (오류, 경고) | `Alert` |
| `robots` | 로봇 상태 변경 (연결/해제) | `RobotResponse` |
| `map:{map_id}` | 맵 관련 이벤트 (처리 완료 등) | `MapResponse` |

#### `src/api/ws/session.rs`

```rust
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

pub struct WsSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub subscriptions: Arc<Mutex<HashSet<String>>>,
    pub tx: mpsc::UnboundedSender<String>,       // JSON 문자열을 WS로 전송
    pub last_pong: Arc<Mutex<std::time::Instant>>,
}

impl WsSession {
    pub fn new(user_id: Uuid, tx: mpsc::UnboundedSender<String>) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            user_id,
            subscriptions: Arc::new(Mutex::new(HashSet::new())),
            tx,
            last_pong: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }

    pub async fn subscribe(&self, topic: &str) {
        self.subscriptions.lock().await.insert(topic.to_string());
    }

    pub async fn unsubscribe(&self, topic: &str) {
        self.subscriptions.lock().await.remove(topic);
    }

    pub async fn is_subscribed(&self, topic: &str) -> bool {
        let subs = self.subscriptions.lock().await;
        // 정확 일치 또는 와일드카드 매칭
        subs.contains(topic) || subs.contains(&topic.split(':').next().map(|p| format!("{}:*", p)).unwrap_or_default())
    }
}
```

#### 하트비트 관리

```rust
// 서버 측 핑 전송 루프
async fn heartbeat_loop(session: Arc<WsSession>, config: &AppConfig) {
    let interval = Duration::from_secs(config.ws_heartbeat_interval_secs);
    let max_missed = config.ws_max_missed_pongs;

    loop {
        tokio::time::sleep(interval).await;

        let last = *session.last_pong.lock().await;
        let elapsed = last.elapsed();

        if elapsed > interval * max_missed {
            tracing::info!("Session {} timed out, disconnecting", session.session_id);
            // 연결 종료 트리거
            break;
        }

        // ping 전송
        let ping_msg = serde_json::json!({
            "msg_type": "ping",
            "timestamp": chrono::Utc::now().timestamp_millis()
        });
        let _ = session.tx.send(ping_msg.to_string());
    }
}
```

#### 팬아웃 메커니즘

이벤트 버스(`tokio::sync::broadcast`)에서 이벤트 수신 → 각 연결된 `WsSession`의 구독 목록 확인 → 매칭되는 세션에만 메시지 전송.

```rust
// WebSocket 관리자 – 모든 활성 세션 관리
pub struct WsManager {
    sessions: Arc<RwLock<HashMap<Uuid, Arc<WsSession>>>>,
}

impl WsManager {
    pub async fn broadcast(&self, topic: &str, payload: serde_json::Value) {
        let sessions = self.sessions.read().await;
        for (_, session) in sessions.iter() {
            if session.is_subscribed(topic).await {
                let msg = WsOutgoingMessage {
                    msg_type: "data".to_string(),
                    topic: Some(topic.to_string()),
                    payload: Some(payload.clone()),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };
                let _ = session.tx.send(serde_json::to_string(&msg).unwrap());
            }
        }
    }
}
```

---

### 4.3 Mission Service 상세

**파일**: `src/services/mission_service.rs`

#### 상태 머신

```
                    ┌──────────┐
                    │ CREATED  │
                    └────┬─────┘
                         │
              ┌──────────┼──────────┐
              │          │          │
              ▼          ▼          │
        ┌──────────┐  ┌──────────┐ │
        │CANCELLED │  │ ASSIGNED │ │
        └──────────┘  └────┬─────┘ │
                           │       │
                           ▼       │
                     ┌──────────┐  │
                     │EXECUTING │  │
                     └────┬─────┘  │
                          │        │
              ┌───────────┼────────┤
              │           │        │
              ▼           ▼        ▼
        ┌──────────┐ ┌────────┐ ┌──────────┐
        │COMPLETED │ │ FAILED │ │CANCELLED │
        └──────────┘ └────────┘ └──────────┘
```

유효한 전이:
| 현재 상태 | 가능한 전이 | 트리거 |
|-----------|-----------|--------|
| CREATED | ASSIGNED | `assign` API 호출 |
| CREATED | CANCELLED | `cancel` API 호출 |
| ASSIGNED | EXECUTING | VDA5050 Order 전송 후 로봇이 수락 |
| ASSIGNED | CANCELLED | `cancel` API 호출 |
| EXECUTING | COMPLETED | 로봇이 모든 스텝 완료 (VDA5050 State) |
| EXECUTING | FAILED | 로봇 오류 (VDA5050 State에 에러) |
| EXECUTING | CANCELLED | `cancel` API 호출 (instantAction 전송) |

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum MissionStatus {
    Created,
    Assigned,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

impl MissionStatus {
    pub fn can_transition_to(&self, target: &MissionStatus) -> bool {
        matches!(
            (self, target),
            (MissionStatus::Created, MissionStatus::Assigned)
            | (MissionStatus::Created, MissionStatus::Cancelled)
            | (MissionStatus::Assigned, MissionStatus::Executing)
            | (MissionStatus::Assigned, MissionStatus::Cancelled)
            | (MissionStatus::Executing, MissionStatus::Completed)
            | (MissionStatus::Executing, MissionStatus::Failed)
            | (MissionStatus::Executing, MissionStatus::Cancelled)
        )
    }
}
```

#### VDA5050 Order 생성

미션의 스텝을 VDA5050 Order 메시지의 `nodes[]`와 `edges[]`로 변환한다.

```rust
pub fn mission_to_vda5050_order(
    mission: &Mission,
    steps: &[MissionStep],
    roadmap_nodes: &HashMap<String, RoadmapNode>,
    roadmap_edges: &HashMap<String, RoadmapEdge>,
) -> Vda5050Order {
    let order_id = mission.id.to_string();
    let order_update_id = 0_u32;

    let mut vda_nodes = Vec::new();
    let mut vda_edges = Vec::new();
    let mut sequence_id = 0_u32;

    for step in steps {
        if let Some(node_id) = &step.target_node_id {
            let roadmap_node = &roadmap_nodes[node_id];
            vda_nodes.push(Vda5050Node {
                node_id: node_id.clone(),
                sequence_id,
                released: true,
                node_position: Some(NodePosition {
                    x: roadmap_node.x,
                    y: roadmap_node.y,
                    theta: 0.0,
                    map_id: mission.map_id.to_string(),
                    allowed_deviation_xy: Some(0.5),
                    allowed_deviation_theta: Some(0.1),
                }),
                actions: step_to_actions(step),
            });

            // 이전 노드와의 edge 추가
            if sequence_id > 0 {
                let prev_node_id = &steps[(sequence_id as usize - 1) / 2].target_node_id;
                if let Some(prev) = prev_node_id {
                    let edge_key = format!("{}->{}", prev, node_id);
                    if let Some(edge) = roadmap_edges.get(&edge_key) {
                        vda_edges.push(Vda5050Edge {
                            edge_id: edge.edge_id.clone(),
                            sequence_id: sequence_id - 1,
                            released: true,
                            start_node_id: prev.clone(),
                            end_node_id: node_id.clone(),
                            max_speed: edge.max_speed,
                            actions: vec![],
                        });
                    }
                }
            }

            sequence_id += 2; // VDA5050: 노드는 짝수, 엣지는 홀수
        }
    }

    Vda5050Order {
        header_id: 0,
        timestamp: Utc::now(),
        version: "2.0.0".to_string(),
        manufacturer: String::new(), // 로봇에서 채움
        serial_number: String::new(),
        order_id,
        order_update_id,
        zone_set_id: None,
        nodes: vda_nodes,
        edges: vda_edges,
    }
}
```

---

### 4.4 Traffic Service 상세

**파일**: `src/services/traffic_service.rs`

#### 구역 잠금 메커니즘

Redis를 사용하여 구역별 잠금을 관리한다.

```rust
use fred::prelude::*;
use uuid::Uuid;
use std::time::Duration;

pub struct TrafficService {
    redis: fred::clients::Client,
    lock_timeout: Duration,
}

impl TrafficService {
    pub fn new(redis: fred::clients::Client, lock_timeout_secs: u64) -> Self {
        Self {
            redis,
            lock_timeout: Duration::from_secs(lock_timeout_secs),
        }
    }

    /// 구역 잠금 획득
    /// 성공: true, 이미 잠겨 있음: false
    pub async fn acquire_zone_lock(
        &self,
        zone_id: &str,
        robot_id: Uuid,
    ) -> anyhow::Result<bool> {
        let key = format!("zone_lock:{}", zone_id);
        let value = robot_id.to_string();
        let ttl_ms = self.lock_timeout.as_millis() as i64;

        // SET NX EX – 이미 존재하면 실패
        let result: Option<String> = self.redis
            .set(&key, &value, Some(Expiration::PX(ttl_ms)), Some(SetPolicy::NX), false)
            .await?;

        Ok(result.is_some())
    }

    /// 구역 잠금 해제 (소유자만)
    pub async fn release_zone_lock(
        &self,
        zone_id: &str,
        robot_id: Uuid,
    ) -> anyhow::Result<bool> {
        let key = format!("zone_lock:{}", zone_id);
        let current: Option<String> = self.redis.get(&key).await?;

        if current.as_deref() == Some(&robot_id.to_string()) {
            let _: () = self.redis.del(&key).await?;
            Ok(true)
        } else {
            Ok(false) // 소유자가 아님
        }
    }

    /// 구역의 현재 잠금 상태 확인
    pub async fn get_zone_lock(
        &self,
        zone_id: &str,
    ) -> anyhow::Result<Option<Uuid>> {
        let key = format!("zone_lock:{}", zone_id);
        let value: Option<String> = self.redis.get(&key).await?;
        Ok(value.and_then(|v| Uuid::parse_str(&v).ok()))
    }

    /// 경로 충돌 검사
    /// 미션의 모든 스텝이 지나는 구역을 확인하고,
    /// 현재 잠긴 구역과 겹치는지 판단
    pub async fn check_path_conflicts(
        &self,
        path_zones: &[String],
        requesting_robot_id: Uuid,
    ) -> anyhow::Result<Vec<ZoneConflict>> {
        let mut conflicts = Vec::new();

        for zone_id in path_zones {
            if let Some(locked_by) = self.get_zone_lock(zone_id).await? {
                if locked_by != requesting_robot_id {
                    conflicts.push(ZoneConflict {
                        zone_id: zone_id.clone(),
                        locked_by,
                    });
                }
            }
        }

        Ok(conflicts)
    }

    /// 로봇의 모든 잠금 해제 (로봇 비상 정지 또는 오프라인 시)
    pub async fn release_all_locks_for_robot(
        &self,
        robot_id: Uuid,
    ) -> anyhow::Result<u32> {
        // SCAN으로 zone_lock:* 키 순회
        let pattern = "zone_lock:*";
        let mut released = 0u32;
        let mut cursor = "0".to_string();

        loop {
            let (next_cursor, keys): (String, Vec<String>) =
                self.redis.scan(cursor, Some(pattern), Some(100)).await?;

            for key in keys {
                let value: Option<String> = self.redis.get(&key).await?;
                if value.as_deref() == Some(&robot_id.to_string()) {
                    let _: () = self.redis.del(&key).await?;
                    released += 1;
                }
            }

            if next_cursor == "0" {
                break;
            }
            cursor = next_cursor;
        }

        Ok(released)
    }
}

#[derive(Debug)]
pub struct ZoneConflict {
    pub zone_id: String,
    pub locked_by: Uuid,
}
```

#### 우선순위 큐

```rust
/// 구역 접근 대기열 (Redis Sorted Set 사용)
/// Score = priority (높을수록 우선) + timestamp (같은 우선순위면 먼저 요청한 것 우선)
pub async fn enqueue_zone_request(
    &self,
    zone_id: &str,
    robot_id: Uuid,
    priority: i32,
) -> anyhow::Result<()> {
    let key = format!("zone_queue:{}", zone_id);
    // score = priority * 1_000_000_000_000 - timestamp_ms (높은 priority, 낮은 timestamp가 우선)
    let score = (priority as f64) * 1_000_000_000_000.0
        - (chrono::Utc::now().timestamp_millis() as f64);
    let _: () = self.redis
        .zadd(&key, None, None, false, false, (score, robot_id.to_string()))
        .await?;
    Ok(())
}

/// 대기열에서 가장 높은 우선순위 로봇 확인
pub async fn peek_zone_queue(
    &self,
    zone_id: &str,
) -> anyhow::Result<Option<Uuid>> {
    let key = format!("zone_queue:{}", zone_id);
    let result: Vec<String> = self.redis
        .zrevrange(&key, 0, 0, false)
        .await?;
    Ok(result.first().and_then(|v| Uuid::parse_str(v).ok()))
}
```

---

### 4.5 MQTT / VDA5050 Bridge 상세

**파일**: `src/mqtt/`

#### `src/mqtt/client.rs` – MQTT 연결 관리

```rust
use rumqttc::{AsyncClient, MqttOptions, QoS, EventLoop};
use std::time::Duration;

pub async fn create_mqtt_client(config: &AppConfig) -> anyhow::Result<(AsyncClient, EventLoop)> {
    let mut mqtt_options = MqttOptions::new(
        &config.mqtt_client_id,
        &config.mqtt_broker_host,
        config.mqtt_broker_port,
    );
    mqtt_options
        .set_keep_alive(Duration::from_secs(30))
        .set_clean_session(true)
        .set_max_packet_size(256 * 1024, 256 * 1024); // 256KB

    let (client, event_loop) = AsyncClient::new(mqtt_options, 100);

    // VDA5050 토픽 구독
    // +는 single-level wildcard
    client.subscribe("/v2/+/+/state", QoS::AtLeastOnce).await?;
    client.subscribe("/v2/+/+/visualization", QoS::AtLeastOnce).await?;
    client.subscribe("/v2/+/+/connection", QoS::AtLeastOnce).await?;

    Ok((client, event_loop))
}
```

#### `src/mqtt/vda5050.rs` – VDA5050 메시지 타입

VDA5050 v2.0 표준을 완전히 준수하는 Rust 구조체 정의:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================
// VDA5050 Order Message
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vda5050Order {
    pub header_id: u32,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub manufacturer: String,
    pub serial_number: String,
    pub order_id: String,
    pub order_update_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_set_id: Option<String>,
    pub nodes: Vec<Vda5050Node>,
    pub edges: Vec<Vda5050Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vda5050Node {
    pub node_id: String,
    pub sequence_id: u32,
    pub released: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_position: Option<NodePosition>,
    pub actions: Vec<Vda5050Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
    pub map_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_deviation_xy: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_deviation_theta: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vda5050Edge {
    pub edge_id: String,
    pub sequence_id: u32,
    pub released: bool,
    pub start_node_id: String,
    pub end_node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<f64>,
    pub actions: Vec<Vda5050Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vda5050Action {
    pub action_type: String,
    pub action_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking_type: Option<String>, // NONE, SOFT, HARD
    pub action_parameters: Vec<ActionParameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionParameter {
    pub key: String,
    pub value: serde_json::Value,
}

// ============================================================
// VDA5050 Instant Actions
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vda5050InstantActions {
    pub header_id: u32,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub manufacturer: String,
    pub serial_number: String,
    pub instant_actions: Vec<Vda5050Action>,
}

// ============================================================
// VDA5050 State Message (로봇 → 서버)
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vda5050State {
    pub header_id: u32,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub manufacturer: String,
    pub serial_number: String,
    pub order_id: String,
    pub order_update_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_set_id: Option<String>,
    pub last_node_id: String,
    pub last_node_sequence_id: u32,
    pub node_states: Vec<NodeState>,
    pub edge_states: Vec<EdgeState>,
    pub agv_position: Option<AgvPosition>,
    pub velocity: Option<Velocity>,
    pub loads: Vec<Load>,
    pub driving: bool,
    pub paused: bool,
    pub new_base_request: bool,
    pub distance_since_last_node: f64,
    pub action_states: Vec<ActionState>,
    pub battery_state: BatteryState,
    pub operating_mode: String, // AUTOMATIC, SEMIAUTOMATIC, MANUAL, SERVICE, TEACHIN
    pub errors: Vec<Vda5050Error>,
    pub information: Vec<Vda5050Info>,
    pub safety_state: SafetyState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgvPosition {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
    pub map_id: String,
    pub position_initialized: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub localization_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deviation_range: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
    pub omega: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryState {
    pub battery_charge: f64,        // 0.0 ~ 100.0
    pub battery_voltage: Option<f64>,
    pub battery_health: Option<f64>,
    pub charging: bool,
    pub reach: Option<f64>,         // 남은 주행 가능 거리 (m)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionState {
    pub action_id: String,
    pub action_type: Option<String>,
    pub action_status: String,      // WAITING, INITIALIZING, RUNNING, PAUSED, FINISHED, FAILED
    pub action_description: Option<String>,
    pub result_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeState {
    pub node_id: String,
    pub sequence_id: u32,
    pub released: bool,
    pub node_description: Option<String>,
    pub node_position: Option<NodePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeState {
    pub edge_id: String,
    pub sequence_id: u32,
    pub released: bool,
    pub edge_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vda5050Error {
    pub error_type: String,
    pub error_level: String,        // WARNING, FATAL
    pub error_description: Option<String>,
    pub error_references: Vec<ErrorReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorReference {
    pub reference_key: String,
    pub reference_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vda5050Info {
    pub info_type: String,
    pub info_level: String,         // DEBUG, INFO
    pub info_description: Option<String>,
    pub info_references: Vec<ErrorReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyState {
    pub e_stop: String,             // AUTOACK, MANUAL, REMOTE, NONE
    pub field_violation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Load {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

// ============================================================
// VDA5050 Connection Message
// ============================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vda5050Connection {
    pub header_id: u32,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub manufacturer: String,
    pub serial_number: String,
    pub connection_state: String,   // ONLINE, OFFLINE, CONNECTIONBROKEN
}
```

#### `src/mqtt/topics.rs` – 토픽 네이밍

```rust
pub struct Vda5050Topics;

impl Vda5050Topics {
    pub fn order(manufacturer: &str, serial_number: &str) -> String {
        format!("/v2/{}/{}/order", manufacturer, serial_number)
    }

    pub fn instant_actions(manufacturer: &str, serial_number: &str) -> String {
        format!("/v2/{}/{}/instantActions", manufacturer, serial_number)
    }

    pub fn state(manufacturer: &str, serial_number: &str) -> String {
        format!("/v2/{}/{}/state", manufacturer, serial_number)
    }

    pub fn visualization(manufacturer: &str, serial_number: &str) -> String {
        format!("/v2/{}/{}/visualization", manufacturer, serial_number)
    }

    pub fn connection(manufacturer: &str, serial_number: &str) -> String {
        format!("/v2/{}/{}/connection", manufacturer, serial_number)
    }

    /// 토픽 문자열에서 manufacturer와 serial_number 추출
    pub fn parse_topic(topic: &str) -> Option<(String, String, String)> {
        let parts: Vec<&str> = topic.split('/').collect();
        // /v2/{manufacturer}/{serialNumber}/{topic_type}
        if parts.len() == 5 && parts[1] == "v2" {
            Some((
                parts[2].to_string(),  // manufacturer
                parts[3].to_string(),  // serial_number
                parts[4].to_string(),  // topic_type
            ))
        } else {
            None
        }
    }
}
```

#### `src/mqtt/bridge.rs` – MQTT ↔ 이벤트 버스 브릿지

```rust
use rumqttc::{Event, Packet, QoS};
use crate::events::types::AppEvent;
use crate::state::AppState;

pub async fn start_bridge(state: AppState) -> anyhow::Result<()> {
    let mut event_loop = state.mqtt_event_loop.clone();

    tokio::spawn(async move {
        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    let topic = publish.topic.clone();
                    let payload = publish.payload.to_vec();

                    if let Some((manufacturer, serial_number, topic_type)) =
                        super::topics::Vda5050Topics::parse_topic(&topic)
                    {
                        match topic_type.as_str() {
                            "state" => {
                                handle_robot_state(
                                    &state, &manufacturer, &serial_number, &payload
                                ).await;
                            }
                            "visualization" => {
                                handle_visualization(
                                    &state, &manufacturer, &serial_number, &payload
                                ).await;
                            }
                            "connection" => {
                                handle_connection(
                                    &state, &manufacturer, &serial_number, &payload
                                ).await;
                            }
                            _ => {
                                tracing::debug!("Unknown VDA5050 topic type: {}", topic_type);
                            }
                        }
                    }
                }
                Ok(_) => {} // 다른 이벤트 무시
                Err(e) => {
                    tracing::error!("MQTT event loop error: {:?}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    });

    Ok(())
}

async fn handle_robot_state(
    state: &AppState,
    manufacturer: &str,
    serial_number: &str,
    payload: &[u8],
) {
    let vda_state: super::vda5050::Vda5050State = match serde_json::from_slice(payload) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to parse VDA5050 state: {:?}", e);
            return;
        }
    };

    // 1. 로봇 상태 업데이트 (DB)
    if let Err(e) = state.db_update_robot_from_vda_state(manufacturer, serial_number, &vda_state).await {
        tracing::error!("Failed to update robot state: {:?}", e);
    }

    // 2. 미션 상태 업데이트
    if let Err(e) = services::mission_service::update_from_robot_state(
        &state.db, &vda_state
    ).await {
        tracing::error!("Failed to update mission from state: {:?}", e);
    }

    // 3. 텔레메트리 변환 및 이벤트 발행
    let robot = db::queries::robots::find_by_serial(
        &state.db, manufacturer, serial_number
    ).await;

    if let Ok(Some(robot)) = robot {
        let telemetry = services::telemetry_service::vda_state_to_telemetry(
            robot.id, &vda_state
        );

        // 이벤트 버스로 팬아웃 (WebSocket 클라이언트에게 전달됨)
        let _ = state.event_bus.send(AppEvent::TelemetryReceived {
            robot_id: robot.id,
            telemetry: telemetry.clone(),
        });

        // TimescaleDB에 저장
        let _ = db::queries::telemetry::insert(&state.db, &telemetry).await;

        // 플러그인 알림
        let _ = state.plugin_runtime.lock().await
            .on_robot_state_changed(robot.id, &vda_state);
    }
}

async fn handle_connection(
    state: &AppState,
    manufacturer: &str,
    serial_number: &str,
    payload: &[u8],
) {
    let conn: super::vda5050::Vda5050Connection = match serde_json::from_slice(payload) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to parse VDA5050 connection: {:?}", e);
            return;
        }
    };

    let new_status = match conn.connection_state.as_str() {
        "ONLINE" => "IDLE",
        "OFFLINE" | "CONNECTIONBROKEN" => "OFFLINE",
        _ => "OFFLINE",
    };

    let _ = db::queries::robots::update_status_by_serial(
        &state.db, manufacturer, serial_number, new_status
    ).await;

    // OFFLINE된 로봇의 구역 잠금 모두 해제
    if new_status == "OFFLINE" {
        if let Ok(Some(robot)) = db::queries::robots::find_by_serial(
            &state.db, manufacturer, serial_number
        ).await {
            let _ = state.traffic_service.release_all_locks_for_robot(robot.id).await;
        }
    }

    let _ = state.event_bus.send(AppEvent::RobotConnectionChanged {
        manufacturer: manufacturer.to_string(),
        serial_number: serial_number.to_string(),
        connection_state: conn.connection_state,
    });
}
```

---

### 4.6 gRPC Clients 상세

#### `build.rs` – Proto 코드 생성

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)  // 클라이언트만 생성
        .build_client(true)
        .out_dir("src/grpc/generated")
        .compile_protos(
            &[
                "proto/simulation.proto",
                "proto/asset.proto",
                "proto/map.proto",
            ],
            &["proto/"],
        )?;
    Ok(())
}
```

#### `src/grpc/sim_client.rs` – Simulation Engine 클라이언트

```rust
use tonic::transport::Channel;
use crate::grpc::generated::simulation::*;
use crate::grpc::generated::simulation::simulation_service_client::SimulationServiceClient;

pub struct SimClient {
    client: SimulationServiceClient<Channel>,
}

impl SimClient {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(url.to_string())?
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(30))
            .connect()
            .await?;

        Ok(Self {
            client: SimulationServiceClient::new(channel),
        })
    }

    /// 시뮬레이션 시작, SimState 스트림 반환
    pub async fn start_simulation(
        &mut self,
        config: SimulationConfig,
    ) -> anyhow::Result<tonic::Streaming<SimState>> {
        let request = tonic::Request::new(StartSimulationRequest {
            config: Some(config),
        });
        let response = self.client.start_simulation(request).await?;
        Ok(response.into_inner())
    }

    /// 시뮬레이션 중지
    pub async fn stop_simulation(&mut self) -> anyhow::Result<()> {
        let request = tonic::Request::new(StopSimulationRequest {});
        self.client.stop_simulation(request).await?;
        Ok(())
    }

    /// 가상 로봇 생성
    pub async fn spawn_robot(
        &mut self,
        robot_config: RobotConfig,
    ) -> anyhow::Result<String> {
        let request = tonic::Request::new(SpawnRobotRequest {
            config: Some(robot_config),
        });
        let response = self.client.spawn_robot(request).await?;
        Ok(response.into_inner().robot_id)
    }

    /// 로봇에 속도 명령 전송
    pub async fn send_command(
        &mut self,
        robot_id: String,
        velocity: VelocityCommand,
    ) -> anyhow::Result<()> {
        let request = tonic::Request::new(SendCommandRequest {
            robot_id,
            command: Some(velocity),
        });
        self.client.send_command(request).await?;
        Ok(())
    }

    /// 텔레메트리 스트림 수신
    pub async fn stream_telemetry(
        &mut self,
    ) -> anyhow::Result<tonic::Streaming<TelemetryMessage>> {
        let request = tonic::Request::new(StreamTelemetryRequest {});
        let response = self.client.stream_telemetry(request).await?;
        Ok(response.into_inner())
    }
}
```

#### `src/grpc/asset_client.rs` – Asset Manager 클라이언트

```rust
pub struct AssetClient {
    client: AssetServiceClient<Channel>,
}

impl AssetClient {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(url.to_string())?
            .connect_timeout(std::time::Duration::from_secs(10))
            .connect()
            .await?;
        Ok(Self {
            client: AssetServiceClient::new(channel),
        })
    }

    pub async fn list_assets(&mut self, filter: AssetFilter) -> anyhow::Result<Vec<Asset>> {
        let request = tonic::Request::new(ListAssetsRequest { filter: Some(filter) });
        let response = self.client.list_assets(request).await?;
        Ok(response.into_inner().assets)
    }

    pub async fn get_asset(&mut self, id: &str) -> anyhow::Result<Asset> {
        let request = tonic::Request::new(GetAssetRequest { id: id.to_string() });
        let response = self.client.get_asset(request).await?;
        Ok(response.into_inner().asset.unwrap())
    }

    /// 스트리밍 업로드
    pub async fn upload_asset(
        &mut self,
        stream: impl futures::Stream<Item = UploadChunk> + Send + 'static,
    ) -> anyhow::Result<Asset> {
        let request = tonic::Request::new(stream);
        let response = self.client.upload_asset(request).await?;
        Ok(response.into_inner().asset.unwrap())
    }

    pub async fn delete_asset(&mut self, id: &str) -> anyhow::Result<()> {
        let request = tonic::Request::new(DeleteAssetRequest { id: id.to_string() });
        self.client.delete_asset(request).await?;
        Ok(())
    }

    pub async fn convert_asset(
        &mut self,
        id: &str,
        target_format: &str,
    ) -> anyhow::Result<Asset> {
        let request = tonic::Request::new(ConvertAssetRequest {
            id: id.to_string(),
            target_format: target_format.to_string(),
        });
        let response = self.client.convert_asset(request).await?;
        Ok(response.into_inner().asset.unwrap())
    }
}
```

#### `src/grpc/map_client.rs` – Map Manager 클라이언트

```rust
pub struct MapClient {
    client: MapServiceClient<Channel>,
}

impl MapClient {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(url.to_string())?
            .connect_timeout(std::time::Duration::from_secs(10))
            .connect()
            .await?;
        Ok(Self {
            client: MapServiceClient::new(channel),
        })
    }

    pub async fn create_map(&mut self, metadata: MapMetadata) -> anyhow::Result<MapInfo> {
        let request = tonic::Request::new(CreateMapRequest { metadata: Some(metadata) });
        let response = self.client.create_map(request).await?;
        Ok(response.into_inner().map.unwrap())
    }

    /// 포인트클라우드 스트리밍 업로드
    pub async fn upload_point_cloud(
        &mut self,
        stream: impl futures::Stream<Item = PointCloudChunk> + Send + 'static,
    ) -> anyhow::Result<ProcessingJob> {
        let request = tonic::Request::new(stream);
        let response = self.client.upload_point_cloud(request).await?;
        Ok(response.into_inner().job.unwrap())
    }

    pub async fn get_processing_status(
        &mut self,
        job_id: &str,
    ) -> anyhow::Result<ProcessingStatus> {
        let request = tonic::Request::new(GetProcessingStatusRequest {
            job_id: job_id.to_string(),
        });
        let response = self.client.get_processing_status(request).await?;
        Ok(response.into_inner().status.unwrap())
    }

    /// 바이너리 타일 데이터 조회
    pub async fn get_tile(
        &mut self,
        map_id: &str,
        node_id: &str,
        lod: u32,
    ) -> anyhow::Result<Vec<u8>> {
        let request = tonic::Request::new(GetTileRequest {
            map_id: map_id.to_string(),
            node_id: node_id.to_string(),
            lod,
        });
        let response = self.client.get_tile(request).await?;
        Ok(response.into_inner().data)
    }

    pub async fn get_roadmap(
        &mut self,
        map_id: &str,
    ) -> anyhow::Result<(Vec<GraphNode>, Vec<GraphEdge>)> {
        let request = tonic::Request::new(GetRoadmapRequest {
            map_id: map_id.to_string(),
        });
        let response = self.client.get_roadmap(request).await?;
        let inner = response.into_inner();
        Ok((inner.nodes, inner.edges))
    }

    pub async fn update_roadmap(
        &mut self,
        map_id: &str,
        nodes: Vec<GraphNode>,
        edges: Vec<GraphEdge>,
    ) -> anyhow::Result<()> {
        let request = tonic::Request::new(UpdateRoadmapRequest {
            map_id: map_id.to_string(),
            nodes,
            edges,
        });
        self.client.update_roadmap(request).await?;
        Ok(())
    }

    /// 경로 탐색
    pub async fn find_path(
        &mut self,
        map_id: &str,
        start_node: &str,
        end_node: &str,
    ) -> anyhow::Result<Vec<String>> {
        let request = tonic::Request::new(FindPathRequest {
            map_id: map_id.to_string(),
            start_node_id: start_node.to_string(),
            end_node_id: end_node.to_string(),
        });
        let response = self.client.find_path(request).await?;
        Ok(response.into_inner().path_node_ids)
    }
}
```

---

### 4.7 Plugin System 상세

**파일**: `src/plugins/`

#### WASM 플러그인 인터페이스

플러그인은 다음 함수를 export해야 한다 (Component Model 인터페이스):

```wit
// plugin.wit – WASM Component Model Interface
package amr:plugin@0.1.0;

interface plugin {
    /// 플러그인 메타데이터 반환
    get-info: func() -> plugin-info;

    /// 미션 생성 시 호출 – 수정된 미션을 반환할 수 있음
    on-mission-created: func(mission: mission-data) -> option<mission-data>;

    /// 미션 완료 시 호출
    on-mission-completed: func(mission: mission-data);

    /// 로봇 상태 변경 시 호출
    on-robot-state-changed: func(robot-id: string, state: robot-state-data);

    /// 알림 발생 시 호출
    on-alert: func(alert: alert-data);
}

record plugin-info {
    name: string,
    version: string,
    description: string,
    hooks: list<string>,
}

record mission-data {
    id: string,
    robot-id: option<string>,
    mission-type: string,
    status: string,
    priority: s32,
    steps-json: string,  // JSON 직렬화된 스텝 배열
}

record robot-state-data {
    position-x: float64,
    position-y: float64,
    position-z: float64,
    battery-level: float64,
    operating-mode: string,
    errors-json: string,  // JSON 직렬화된 에러 배열
}

record alert-data {
    alert-type: string,
    severity: string,     // INFO, WARNING, ERROR, CRITICAL
    message: string,
    source: string,
    metadata-json: string,
}
```

#### `src/plugins/wasm_host.rs`

```rust
use wasmtime::*;
use wasmtime_wasi::WasiCtxBuilder;
use std::collections::HashMap;
use uuid::Uuid;

pub struct WasmHost {
    engine: Engine,
    plugins: HashMap<Uuid, LoadedPlugin>,
}

struct LoadedPlugin {
    instance: Instance,
    store: Store<WasiState>,
    info: PluginInfo,
    enabled: bool,
}

struct WasiState {
    wasi_ctx: wasmtime_wasi::WasiCtx,
}

impl WasmHost {
    pub fn new() -> anyhow::Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);

        let engine = Engine::new(&config)?;

        Ok(Self {
            engine,
            plugins: HashMap::new(),
        })
    }

    /// WASM 바이너리 검증 및 로드
    pub async fn load_plugin(
        &mut self,
        plugin_id: Uuid,
        wasm_bytes: &[u8],
    ) -> anyhow::Result<PluginInfo> {
        let module = Module::new(&self.engine, wasm_bytes)?;

        // 제한된 WASI 환경 (파일시스템 없음, 네트워크 없음)
        let wasi_ctx = WasiCtxBuilder::new()
            .build();

        let mut store = Store::new(&self.engine, WasiState { wasi_ctx });

        // 연료 제한 (무한 루프 방지)
        store.set_fuel(1_000_000)?;

        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker_sync(&mut linker)?;

        // 호스트 콜백 등록
        self.register_host_functions(&mut linker)?;

        let instance = linker.instantiate(&mut store, &module)?;

        // get_info 호출하여 메타데이터 확인
        let get_info = instance
            .get_typed_func::<(), (i32,)>(&mut store, "get-info")?;
        // ... 메타데이터 파싱 ...

        let info = PluginInfo {
            name: "plugin_name".to_string(),
            version: "0.1.0".to_string(),
            hooks: vec![],
        };

        self.plugins.insert(plugin_id, LoadedPlugin {
            instance,
            store,
            info: info.clone(),
            enabled: false,
        });

        Ok(info)
    }

    /// 호스트 함수 등록 (플러그인이 호출할 수 있는 API)
    fn register_host_functions(&self, linker: &mut Linker<WasiState>) -> anyhow::Result<()> {
        // create_mission: 플러그인이 새 미션 생성 요청
        linker.func_wrap("amr:host", "create-mission", |_caller: Caller<'_, WasiState>, _json_ptr: i32, _json_len: i32| -> i32 {
            // 호스트에서 미션 생성 처리
            0 // 성공
        })?;

        // get_robot_state: 플러그인이 로봇 상태 조회
        linker.func_wrap("amr:host", "get-robot-state", |_caller: Caller<'_, WasiState>, _robot_id_ptr: i32, _len: i32| -> i32 {
            0
        })?;

        // send_alert: 플러그인이 알림 전송
        linker.func_wrap("amr:host", "send-alert", |_caller: Caller<'_, WasiState>, _json_ptr: i32, _json_len: i32| -> i32 {
            0
        })?;

        Ok(())
    }

    /// 활성화된 모든 플러그인의 on_mission_created 호출
    pub async fn on_mission_created(&mut self, mission: &MissionData) -> Option<MissionData> {
        let mut modified = None;
        for (_, plugin) in self.plugins.iter_mut() {
            if plugin.enabled && plugin.info.hooks.contains(&"on_mission_created".to_string()) {
                // WASM 함수 호출
                // 결과가 Some이면 mission을 수정
                // ... (직렬화/역직렬화 처리)
            }
        }
        modified
    }

    pub fn enable_plugin(&mut self, plugin_id: Uuid) -> anyhow::Result<()> {
        let plugin = self.plugins.get_mut(&plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found"))?;
        plugin.enabled = true;
        Ok(())
    }

    pub fn disable_plugin(&mut self, plugin_id: Uuid) -> anyhow::Result<()> {
        let plugin = self.plugins.get_mut(&plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found"))?;
        plugin.enabled = false;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub hooks: Vec<String>,
}
```

#### `src/plugins/webhook.rs` – HTTP 웹훅 디스패처

```rust
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

pub struct WebhookDispatcher {
    client: Client,
}

impl WebhookDispatcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        Self { client }
    }

    /// 웹훅 URL로 이벤트 전송 (fire-and-forget, 실패 시 재시도 3회)
    pub async fn dispatch<T: Serialize>(
        &self,
        url: &str,
        event_type: &str,
        payload: &T,
    ) -> anyhow::Result<()> {
        let body = serde_json::json!({
            "event_type": event_type,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "payload": payload,
        });

        let mut attempts = 0;
        loop {
            attempts += 1;
            match self.client.post(url).json(&body).send().await {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                Ok(resp) => {
                    tracing::warn!("Webhook {} returned status {}", url, resp.status());
                }
                Err(e) => {
                    tracing::warn!("Webhook {} failed: {:?}", url, e);
                }
            }
            if attempts >= 3 {
                tracing::error!("Webhook {} failed after 3 attempts", url);
                return Err(anyhow::anyhow!("Webhook delivery failed"));
            }
            tokio::time::sleep(Duration::from_secs(2u64.pow(attempts))).await;
        }
    }
}
```

---

### 4.8 Telemetry Service 상세

**파일**: `src/services/telemetry_service.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 정규화된 텔레메트리 메시지 (MQTT/gRPC 소스에 관계없이 동일 형식)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryMessage {
    pub robot_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
    pub orientation_theta: f64,
    pub velocity_linear: f64,
    pub velocity_angular: f64,
    pub battery_level: f64,
    pub battery_charging: bool,
    pub operating_mode: String,
    pub map_id: Option<Uuid>,
    pub errors: Vec<String>,
    pub payload: Option<serde_json::Value>, // 추가 센서 데이터
}

/// VDA5050 State → TelemetryMessage 변환
pub fn vda_state_to_telemetry(robot_id: Uuid, state: &Vda5050State) -> TelemetryMessage {
    let position = state.agv_position.as_ref();
    let velocity = state.velocity.as_ref();

    TelemetryMessage {
        robot_id,
        timestamp: state.timestamp,
        position_x: position.map(|p| p.x).unwrap_or(0.0),
        position_y: position.map(|p| p.y).unwrap_or(0.0),
        position_z: 0.0,
        orientation_theta: position.map(|p| p.theta).unwrap_or(0.0),
        velocity_linear: velocity.map(|v| (v.vx.powi(2) + v.vy.powi(2)).sqrt()).unwrap_or(0.0),
        velocity_angular: velocity.map(|v| v.omega).unwrap_or(0.0),
        battery_level: state.battery_state.battery_charge,
        battery_charging: state.battery_state.charging,
        operating_mode: state.operating_mode.clone(),
        map_id: position.and_then(|p| Uuid::parse_str(&p.map_id).ok()),
        errors: state.errors.iter()
            .map(|e| format!("{}: {}", e.error_type, e.error_description.clone().unwrap_or_default()))
            .collect(),
        payload: None,
    }
}

/// 텔레메트리 수집 시작 (gRPC 시뮬레이션 스트림 + MQTT 실제 로봇)
pub async fn start_ingestion(state: AppState) -> anyhow::Result<()> {
    // gRPC 시뮬레이션 텔레메트리 수집은 시뮬레이션 시작 시 별도 태스크로 실행됨
    // MQTT 텔레메트리는 bridge.rs에서 처리됨

    // Redis pub/sub 팬아웃 태스크
    tokio::spawn(fanout_to_redis(state.clone()));

    // TimescaleDB 배치 삽입 태스크
    tokio::spawn(batch_insert_to_db(state.clone()));

    Ok(())
}

/// 이벤트 버스에서 텔레메트리 수신 → Redis pub/sub로 전달
async fn fanout_to_redis(state: AppState) {
    let mut rx = state.event_bus.subscribe();
    let redis = state.redis.clone();

    loop {
        match rx.recv().await {
            Ok(AppEvent::TelemetryReceived { robot_id, telemetry }) => {
                let channel = format!("telemetry:{}", robot_id);
                let payload = serde_json::to_string(&telemetry).unwrap();
                let _: Result<(), _> = redis.publish(&channel, &payload).await;
            }
            _ => {}
        }
    }
}

/// 텔레메트리를 배치로 TimescaleDB에 삽입 (성능 최적화)
async fn batch_insert_to_db(state: AppState) {
    let mut rx = state.event_bus.subscribe();
    let mut buffer: Vec<TelemetryMessage> = Vec::with_capacity(100);
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

    loop {
        tokio::select! {
            event = rx.recv() => {
                if let Ok(AppEvent::TelemetryReceived { telemetry, .. }) = event {
                    buffer.push(telemetry);
                    if buffer.len() >= 100 {
                        flush_buffer(&state.db, &mut buffer).await;
                    }
                }
            }
            _ = interval.tick() => {
                if !buffer.is_empty() {
                    flush_buffer(&state.db, &mut buffer).await;
                }
            }
        }
    }
}

async fn flush_buffer(db: &PgPool, buffer: &mut Vec<TelemetryMessage>) {
    if buffer.is_empty() { return; }

    // COPY 또는 배치 INSERT로 성능 최적화
    if let Err(e) = db::queries::telemetry::batch_insert(db, buffer).await {
        tracing::error!("Failed to batch insert telemetry: {:?}", e);
    }
    buffer.clear();
}
```

---

### 4.9 이벤트 버스

**파일**: `src/events/`

#### `src/events/types.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    // 텔레메트리
    TelemetryReceived {
        robot_id: Uuid,
        telemetry: TelemetryMessage,
    },

    // 로봇
    RobotCreated { robot_id: Uuid },
    RobotUpdated { robot_id: Uuid },
    RobotDeleted { robot_id: Uuid },
    RobotStatusChanged {
        robot_id: Uuid,
        old_status: String,
        new_status: String,
    },
    RobotConnectionChanged {
        manufacturer: String,
        serial_number: String,
        connection_state: String,
    },

    // 미션
    MissionCreated { mission_id: Uuid },
    MissionAssigned { mission_id: Uuid, robot_id: Uuid },
    MissionStarted { mission_id: Uuid },
    MissionCompleted { mission_id: Uuid },
    MissionFailed { mission_id: Uuid, error: String },
    MissionCancelled { mission_id: Uuid },
    MissionStepCompleted { mission_id: Uuid, step_id: Uuid, sequence: i32 },

    // 맵
    MapCreated { map_id: Uuid },
    MapProcessingCompleted { map_id: Uuid },
    MapProcessingFailed { map_id: Uuid, error: String },

    // 알림
    Alert {
        alert_type: String,
        severity: String,
        message: String,
        source: String,
        metadata: serde_json::Value,
        timestamp: DateTime<Utc>,
    },

    // 교통
    ZoneLocked { zone_id: String, robot_id: Uuid },
    ZoneReleased { zone_id: String, robot_id: Uuid },
    TrafficConflict { zone_id: String, robots: Vec<Uuid> },
}
```

#### `src/events/bus.rs`

```rust
use tokio::sync::broadcast;
use super::types::AppEvent;

pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn send(&self, event: AppEvent) -> Result<usize, broadcast::error::SendError<AppEvent>> {
        self.sender.send(event)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }
}
```

---

## 5. 데이터베이스 스키마 상세

### 5.1 users 테이블

**파일**: `migrations/001_create_users.sql`

```sql
-- 001_create_users.sql
-- 사용자 테이블: 인증 및 RBAC

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'viewer'
        CHECK (role IN ('admin', 'operator', 'viewer')),
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 이메일 검색 인덱스
CREATE INDEX idx_users_email ON users (email);
-- 역할별 필터링 인덱스
CREATE INDEX idx_users_role ON users (role);

-- updated_at 자동 갱신 트리거
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 기본 관리자 계정 (비밀번호: admin123 – Argon2 해시)
-- 주의: 프로덕션에서 반드시 변경할 것
INSERT INTO users (email, password_hash, role) VALUES
    ('admin@amr.local', '$argon2id$v=19$m=19456,t=2,p=1$placeholder_salt$placeholder_hash', 'admin');
```

### 5.2 robots 테이블

**파일**: `migrations/002_create_robots.sql`

```sql
-- 002_create_robots.sql
-- 로봇 레지스트리

CREATE TABLE robots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    model_id VARCHAR(100) NOT NULL,
    serial_number VARCHAR(100) UNIQUE NOT NULL,
    manufacturer VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'OFFLINE'
        CHECK (status IN ('IDLE', 'BUSY', 'OFFLINE', 'ERROR', 'CHARGING', 'MAINTENANCE')),
    battery_level DOUBLE PRECISION CHECK (battery_level >= 0 AND battery_level <= 100),
    position_x DOUBLE PRECISION,
    position_y DOUBLE PRECISION,
    position_z DOUBLE PRECISION,
    orientation_theta DOUBLE PRECISION,
    current_map_id UUID,
    current_mission_id UUID,
    operating_mode VARCHAR(50) DEFAULT 'AUTOMATIC'
        CHECK (operating_mode IN ('AUTOMATIC', 'SEMIAUTOMATIC', 'MANUAL', 'SERVICE', 'TEACHIN')),
    config JSONB NOT NULL DEFAULT '{}',
    firmware_version VARCHAR(50),
    last_seen_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 상태별 필터링
CREATE INDEX idx_robots_status ON robots (status);
-- 제조사+시리얼번호 조합 검색
CREATE INDEX idx_robots_manufacturer_serial ON robots (manufacturer, serial_number);
-- 이름 검색 (LIKE 지원)
CREATE INDEX idx_robots_name ON robots USING gin (name gin_trgm_ops);
-- 현재 맵 기준 필터링
CREATE INDEX idx_robots_current_map ON robots (current_map_id);

-- pg_trgm 확장 (텍스트 유사도 검색)
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TRIGGER trigger_robots_updated_at
    BEFORE UPDATE ON robots
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### 5.3 maps 테이블 (PostGIS 포함)

**파일**: `migrations/003_create_maps.sql`

```sql
-- 003_create_maps.sql
-- 맵 메타데이터 (PostGIS 확장 필요)

CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE maps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'EMPTY'
        CHECK (status IN ('EMPTY', 'PROCESSING', 'READY', 'ERROR')),
    -- 맵 경계 (Bounding Box)
    bounds_min_x DOUBLE PRECISION,
    bounds_min_y DOUBLE PRECISION,
    bounds_min_z DOUBLE PRECISION,
    bounds_max_x DOUBLE PRECISION,
    bounds_max_y DOUBLE PRECISION,
    bounds_max_z DOUBLE PRECISION,
    -- PostGIS 경계 다각형 (2D)
    bounds_geom GEOMETRY(POLYGON, 4326),
    -- 포인트클라우드 메타데이터
    point_cloud_size_bytes BIGINT,
    point_count BIGINT,
    tile_count INTEGER,
    resolution DOUBLE PRECISION,    -- m/pixel
    -- 처리 상태
    processing_job_id VARCHAR(255),
    processing_progress DOUBLE PRECISION DEFAULT 0.0,
    processing_error TEXT,
    -- 메타데이터
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_maps_status ON maps (status);
CREATE INDEX idx_maps_bounds_geom ON maps USING GIST (bounds_geom);

CREATE TRIGGER trigger_maps_updated_at
    BEFORE UPDATE ON maps
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### 5.4 roadmap_graphs 테이블

**파일**: `migrations/004_create_roadmap.sql`

```sql
-- 004_create_roadmap.sql
-- 로드맵 그래프 (노드 + 엣지)

CREATE TABLE roadmap_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    node_id VARCHAR(100) NOT NULL,
    x DOUBLE PRECISION NOT NULL,
    y DOUBLE PRECISION NOT NULL,
    z DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    name VARCHAR(255),
    node_type VARCHAR(50) NOT NULL DEFAULT 'WAYPOINT'
        CHECK (node_type IN ('WAYPOINT', 'STATION', 'CHARGER', 'ELEVATOR', 'DOCK', 'INTERSECTION')),
    properties JSONB NOT NULL DEFAULT '{}',
    -- PostGIS point
    geom GEOMETRY(POINT, 4326),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (map_id, node_id)
);

CREATE INDEX idx_roadmap_nodes_map ON roadmap_nodes (map_id);
CREATE INDEX idx_roadmap_nodes_type ON roadmap_nodes (map_id, node_type);
CREATE INDEX idx_roadmap_nodes_geom ON roadmap_nodes USING GIST (geom);

CREATE TABLE roadmap_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    edge_id VARCHAR(100) NOT NULL,
    start_node_id VARCHAR(100) NOT NULL,
    end_node_id VARCHAR(100) NOT NULL,
    bidirectional BOOLEAN NOT NULL DEFAULT false,
    weight DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    max_speed DOUBLE PRECISION,     -- m/s
    properties JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (map_id, edge_id),
    -- 외래 키는 map_id + node_id 조합으로 참조
    FOREIGN KEY (map_id, start_node_id) REFERENCES roadmap_nodes(map_id, node_id) ON DELETE CASCADE,
    FOREIGN KEY (map_id, end_node_id) REFERENCES roadmap_nodes(map_id, node_id) ON DELETE CASCADE
);

CREATE INDEX idx_roadmap_edges_map ON roadmap_edges (map_id);
CREATE INDEX idx_roadmap_edges_nodes ON roadmap_edges (map_id, start_node_id, end_node_id);

CREATE TRIGGER trigger_roadmap_nodes_updated_at
    BEFORE UPDATE ON roadmap_nodes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_roadmap_edges_updated_at
    BEFORE UPDATE ON roadmap_edges
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### 5.5 semantic_regions 테이블 (PostGIS geometry)

**파일**: `migrations/005_create_semantic_regions.sql`

```sql
-- 005_create_semantic_regions.sql
-- 시맨틱 레이어: 구역 정의 (교통 관리, 제한 구역 등)

CREATE TABLE semantic_regions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id UUID NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    region_type VARCHAR(50) NOT NULL
        CHECK (region_type IN ('ZONE', 'RESTRICTED', 'CHARGING', 'LOADING', 'STORAGE', 'ELEVATOR', 'ONEWAY', 'SLOWDOWN')),
    -- PostGIS 다각형
    polygon GEOMETRY(POLYGON, 4326) NOT NULL,
    -- 교통 관리 속성
    max_robots INTEGER DEFAULT 1,           -- 동시 진입 가능 로봇 수
    speed_limit DOUBLE PRECISION,           -- m/s 제한 속도
    -- 접근 제어
    allowed_robot_types VARCHAR(255)[],     -- 허용 로봇 유형
    allowed_directions VARCHAR(50),         -- BOTH, FORWARD_ONLY (일방통행)
    -- 기타 속성
    properties JSONB NOT NULL DEFAULT '{}',
    priority INTEGER NOT NULL DEFAULT 0,     -- 겹치는 구역 시 우선순위
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_semantic_regions_map ON semantic_regions (map_id);
CREATE INDEX idx_semantic_regions_type ON semantic_regions (map_id, region_type);
CREATE INDEX idx_semantic_regions_polygon ON semantic_regions USING GIST (polygon);

CREATE TRIGGER trigger_semantic_regions_updated_at
    BEFORE UPDATE ON semantic_regions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### 5.6 missions 테이블

**파일**: `migrations/006_create_missions.sql`

```sql
-- 006_create_missions.sql
-- 미션 관리

CREATE TABLE missions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    robot_id UUID REFERENCES robots(id) ON DELETE SET NULL,
    map_id UUID NOT NULL REFERENCES maps(id),
    mission_type VARCHAR(50) NOT NULL
        CHECK (mission_type IN ('TRANSPORT', 'PATROL', 'CHARGE', 'CUSTOM')),
    status VARCHAR(50) NOT NULL DEFAULT 'CREATED'
        CHECK (status IN ('CREATED', 'ASSIGNED', 'EXECUTING', 'COMPLETED', 'FAILED', 'CANCELLED')),
    priority INTEGER NOT NULL DEFAULT 5
        CHECK (priority >= 1 AND priority <= 10),
    progress DOUBLE PRECISION NOT NULL DEFAULT 0.0
        CHECK (progress >= 0.0 AND progress <= 1.0),
    error_message TEXT,
    -- VDA5050 관련
    vda5050_order_id VARCHAR(255),
    vda5050_order_update_id INTEGER DEFAULT 0,
    -- 타임스탬프
    created_by UUID NOT NULL REFERENCES users(id),
    assigned_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_missions_status ON missions (status);
CREATE INDEX idx_missions_robot ON missions (robot_id);
CREATE INDEX idx_missions_map ON missions (map_id);
CREATE INDEX idx_missions_priority ON missions (priority DESC, created_at ASC);
CREATE INDEX idx_missions_created_by ON missions (created_by);
CREATE INDEX idx_missions_created_at ON missions (created_at DESC);

-- 복합 인덱스: 활성 미션 조회 (상태 + 로봇)
CREATE INDEX idx_missions_active ON missions (robot_id, status)
    WHERE status IN ('ASSIGNED', 'EXECUTING');

CREATE TRIGGER trigger_missions_updated_at
    BEFORE UPDATE ON missions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### 5.7 mission_steps 테이블

**파일**: `migrations/007_create_mission_steps.sql`

```sql
-- 007_create_mission_steps.sql
-- 미션 스텝 (순서가 있는 작업 단계)

CREATE TABLE mission_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mission_id UUID NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
    sequence INTEGER NOT NULL,
    step_type VARCHAR(50) NOT NULL
        CHECK (step_type IN ('NAVIGATE', 'WAIT', 'ACTION', 'CHARGE', 'PICKUP', 'DROPOFF')),
    target_node_id VARCHAR(100),    -- 로드맵 노드 ID
    parameters JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING'
        CHECK (status IN ('PENDING', 'EXECUTING', 'COMPLETED', 'FAILED', 'SKIPPED')),
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (mission_id, sequence)
);

CREATE INDEX idx_mission_steps_mission ON mission_steps (mission_id, sequence);
CREATE INDEX idx_mission_steps_status ON mission_steps (mission_id, status);

CREATE TRIGGER trigger_mission_steps_updated_at
    BEFORE UPDATE ON mission_steps
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### 5.8 assets 테이블

**파일**: `migrations/008_create_assets.sql`

```sql
-- 008_create_assets.sql
-- 에셋 메타데이터 (실제 파일은 MinIO/S3에 저장)

CREATE TABLE assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    asset_type VARCHAR(50) NOT NULL
        CHECK (asset_type IN ('robot_model', 'texture', 'scene', 'environment', 'material', 'animation', 'other')),
    format VARCHAR(20) NOT NULL,    -- glb, gltf, fbx, png, jpg, etc.
    file_size BIGINT NOT NULL,
    s3_bucket VARCHAR(255) NOT NULL,
    s3_key VARCHAR(1024) NOT NULL,
    content_type VARCHAR(255),
    checksum VARCHAR(64),           -- SHA-256
    -- 3D 모델 메타데이터
    metadata JSONB NOT NULL DEFAULT '{}',
    -- 변환 상태 (포맷 변환 요청 시)
    conversion_status VARCHAR(50) DEFAULT 'NONE'
        CHECK (conversion_status IN ('NONE', 'PENDING', 'PROCESSING', 'COMPLETED', 'FAILED')),
    original_asset_id UUID REFERENCES assets(id),
    -- 기타
    uploaded_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_assets_type ON assets (asset_type);
CREATE INDEX idx_assets_format ON assets (format);
CREATE INDEX idx_assets_uploaded_by ON assets (uploaded_by);

CREATE TRIGGER trigger_assets_updated_at
    BEFORE UPDATE ON assets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

### 5.9 telemetry 하이퍼테이블 (TimescaleDB)

**파일**: `migrations/009_create_telemetry.sql`

```sql
-- 009_create_telemetry.sql
-- TimescaleDB 하이퍼테이블: 로봇 텔레메트리 시계열 데이터

CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE telemetry (
    time TIMESTAMPTZ NOT NULL,
    robot_id UUID NOT NULL,
    position_x DOUBLE PRECISION NOT NULL,
    position_y DOUBLE PRECISION NOT NULL,
    position_z DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    orientation_theta DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    velocity_linear DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    velocity_angular DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    battery_level DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    battery_charging BOOLEAN NOT NULL DEFAULT false,
    operating_mode VARCHAR(50) NOT NULL DEFAULT 'AUTOMATIC',
    map_id UUID,
    errors JSONB NOT NULL DEFAULT '[]',
    payload JSONB  -- 추가 센서 데이터
);

-- TimescaleDB 하이퍼테이블 변환 (1시간 단위 청크)
SELECT create_hypertable('telemetry', by_range('time', INTERVAL '1 hour'));

-- 인덱스
CREATE INDEX idx_telemetry_robot_time ON telemetry (robot_id, time DESC);
CREATE INDEX idx_telemetry_map ON telemetry (map_id, time DESC)
    WHERE map_id IS NOT NULL;

-- 7일 후 자동 압축
ALTER TABLE telemetry SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'robot_id',
    timescaledb.compress_orderby = 'time DESC'
);

SELECT add_compression_policy('telemetry', INTERVAL '7 days');

-- 30일 후 원시 데이터 삭제 (연속 집계는 유지)
SELECT add_retention_policy('telemetry', INTERVAL '30 days');

-- 연속 집계: 1분 평균 (영구 보관)
CREATE MATERIALIZED VIEW telemetry_1min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 minute', time) AS bucket,
    robot_id,
    AVG(position_x) AS avg_position_x,
    AVG(position_y) AS avg_position_y,
    AVG(position_z) AS avg_position_z,
    AVG(orientation_theta) AS avg_orientation_theta,
    AVG(velocity_linear) AS avg_velocity_linear,
    AVG(velocity_angular) AS avg_velocity_angular,
    AVG(battery_level) AS avg_battery_level,
    MAX(battery_level) AS max_battery_level,
    MIN(battery_level) AS min_battery_level,
    COUNT(*) AS sample_count
FROM telemetry
GROUP BY bucket, robot_id
WITH NO DATA;

-- 연속 집계 정책: 2분 지연으로 실시간에 가깝게 갱신
SELECT add_continuous_aggregate_policy('telemetry_1min',
    start_offset => INTERVAL '1 hour',
    end_offset => INTERVAL '2 minutes',
    schedule_interval => INTERVAL '1 minute'
);
```

### 5.10 plugins 테이블

**파일**: `migrations/010_create_plugins.sql`

```sql
-- 010_create_plugins.sql
-- 플러그인 레지스트리

CREATE TABLE plugins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    plugin_type VARCHAR(50) NOT NULL
        CHECK (plugin_type IN ('WASM', 'WEBHOOK')),
    enabled BOOLEAN NOT NULL DEFAULT false,
    -- WASM 플러그인
    wasm_s3_bucket VARCHAR(255),
    wasm_s3_key VARCHAR(1024),
    wasm_size_bytes BIGINT,
    wasm_checksum VARCHAR(64),       -- SHA-256
    -- WEBHOOK 플러그인
    webhook_url VARCHAR(2048),
    webhook_secret VARCHAR(255),     -- 서명 검증용 시크릿
    -- 공통
    hooks VARCHAR(100)[] NOT NULL DEFAULT '{}',  -- 구독하는 이벤트 목록
    config JSONB NOT NULL DEFAULT '{}',          -- 플러그인별 설정
    -- 메타
    uploaded_by UUID REFERENCES users(id),
    last_invoked_at TIMESTAMPTZ,
    invocation_count BIGINT NOT NULL DEFAULT 0,
    error_count BIGINT NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_plugins_type ON plugins (plugin_type);
CREATE INDEX idx_plugins_enabled ON plugins (enabled);
CREATE INDEX idx_plugins_hooks ON plugins USING GIN (hooks);

CREATE TRIGGER trigger_plugins_updated_at
    BEFORE UPDATE ON plugins
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

---

## 6. 설정 및 환경 변수

### `.env.example`

```env
# ============================================================
# AMR Backend 환경 변수
# ============================================================

# 서버
PORT=8080
RUST_LOG=info,amr_backend=debug,tower_http=debug,sqlx=warn

# 데이터베이스 (PostgreSQL 16 + TimescaleDB + PostGIS)
DATABASE_URL=postgresql://amr:amr_password@localhost:5432/amr_db
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5

# Redis
REDIS_URL=redis://localhost:6379

# MinIO / S3
MINIO_ENDPOINT=http://localhost:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=minioadmin
MINIO_BUCKET=amr-assets

# MQTT (VDA5050)
MQTT_BROKER_URL=mqtt://localhost:1883
MQTT_CLIENT_ID=amr-backend

# JWT
JWT_SECRET=your-256-bit-secret-key-change-this-in-production
JWT_EXPIRATION_SECS=3600
JWT_REFRESH_EXPIRATION_SECS=604800

# gRPC 서비스 연결
GRPC_SIM_ENGINE_URL=http://localhost:50051
GRPC_ASSET_MANAGER_URL=http://localhost:50052
GRPC_MAP_MANAGER_URL=http://localhost:50053

# OpenTelemetry
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=amr-backend

# 교통 관리
ZONE_LOCK_TIMEOUT_SECS=60

# WebSocket
WS_HEARTBEAT_INTERVAL_SECS=30
WS_MAX_MISSED_PONGS=3

# 텔레메트리
TELEMETRY_RETENTION_DAYS=30
TELEMETRY_COMPRESSION_AFTER_DAYS=7
```

### `docker-compose.yml` (로컬 개발용)

```yaml
version: "3.9"

services:
  postgres:
    image: timescale/timescaledb-ha:pg16
    ports:
      - "5432:5432"
    environment:
      POSTGRES_DB: amr_db
      POSTGRES_USER: amr
      POSTGRES_PASSWORD: amr_password
    volumes:
      - pgdata:/home/postgres/pgdata/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U amr -d amr_db"]
      interval: 5s
      timeout: 5s
      retries: 5

  # PostGIS 활성화를 위한 init 스크립트
  postgres-init:
    image: timescale/timescaledb-ha:pg16
    depends_on:
      postgres:
        condition: service_healthy
    entrypoint: >
      psql -h postgres -U amr -d amr_db -c "CREATE EXTENSION IF NOT EXISTS postgis;"
    environment:
      PGPASSWORD: amr_password

  redis:
    image: redis:7.4-alpine
    ports:
      - "6379:6379"
    command: redis-server --maxmemory 256mb --maxmemory-policy allkeys-lru

  minio:
    image: minio/minio:latest
    ports:
      - "9000:9000"
      - "9001:9001"   # MinIO Console
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    command: server /data --console-address ":9001"
    volumes:
      - minio_data:/data

  mosquitto:
    image: eclipse-mosquitto:2
    ports:
      - "1883:1883"
      - "9883:9883"   # WebSocket (옵션)
    volumes:
      - ./mosquitto.conf:/mosquitto/config/mosquitto.conf

  # OpenTelemetry Collector (옵션)
  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
    volumes:
      - ./otel-config.yaml:/etc/otelcol-contrib/config.yaml

  # Jaeger (트레이싱 UI, 옵션)
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686" # Jaeger UI
      - "14268:14268"

volumes:
  pgdata:
  minio_data:
```

### `Dockerfile`

```dockerfile
# Stage 1: Build
FROM rust:1.82-bookworm AS builder

WORKDIR /app

# 의존성 캐싱을 위해 Cargo.toml/lock 먼저 복사
COPY Cargo.toml Cargo.lock ./
COPY proto/ proto/
COPY build.rs ./

# 빈 src 생성하여 의존성만 빌드
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# 실제 소스 복사 후 빌드
COPY src/ src/
COPY migrations/ migrations/
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/amr-backend /app/amr-backend
COPY migrations/ /app/migrations/

ENV RUST_LOG=info,amr_backend=debug

EXPOSE 8080

ENTRYPOINT ["/app/amr-backend"]
```

---

## 7. 에러 처리 전략

### 7.1 에러 분류 체계

| AppError 변형 | HTTP 상태 코드 | 사용 시나리오 |
|--------------|---------------|-------------|
| `NotFound` | 404 | 리소스 ID로 조회 실패 |
| `Unauthorized` | 401 | JWT 누락/만료/무효 |
| `Forbidden` | 403 | 역할 권한 부족 |
| `Validation` | 400 | 요청 본문 유효성 검사 실패 |
| `Conflict` | 409 | 중복 키, 상태 충돌 |
| `Internal` | 500 | 예기치 않은 서버 오류 |
| `ServiceUnavailable` | 503 | gRPC 서비스 연결 실패, MQTT 연결 끊김 |
| `TooManyRequests` | 429 | 요청 제한 초과 |

### 7.2 에러 응답 형식

모든 에러 응답은 일관된 JSON 형식을 따른다:

```json
{
    "error": "VALIDATION_ERROR",
    "message": "이메일 형식이 올바르지 않습니다",
    "details": {
        "field": "email",
        "value": "not-an-email"
    }
}
```

### 7.3 gRPC 에러 매핑

gRPC 서비스 호출 시 반환되는 `tonic::Status` 에러를 HTTP 에러로 매핑:

```rust
impl From<tonic::Status> for AppError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => AppError::NotFound(status.message().to_string()),
            tonic::Code::InvalidArgument => AppError::Validation(status.message().to_string()),
            tonic::Code::PermissionDenied => AppError::Forbidden(status.message().to_string()),
            tonic::Code::Unavailable => AppError::ServiceUnavailable(
                format!("gRPC 서비스 연결 실패: {}", status.message())
            ),
            tonic::Code::AlreadyExists => AppError::Conflict(status.message().to_string()),
            _ => AppError::Internal(format!("gRPC 오류: {}", status.message())),
        }
    }
}
```

### 7.4 트레이싱 통합

모든 에러에 `request_id` 컨텍스트를 포함하여 로그 추적을 가능하게 한다:

```rust
// 미들웨어에서 request_id 설정
// tower_http::request_id 레이어가 자동으로 X-Request-Id 헤더를 생성

// 에러 로깅 시 request_id 포함
tracing::error!(
    request_id = %request_id,
    error = %error,
    "요청 처리 실패"
);
```

---

## 8. 개발 계획

> 📋 Phase별 상세 일정 및 팀별 할당: [`development-plan.md`](../development-plan.md) 참조
>
> 본 팀의 기능별 상세 스펙은 아래 기능 문서 참조:
> - C-XX: `docs/features/core/C-XX-*.md`
> - A-XX: `docs/features/advanced/A-XX-*.md`

---

## 9. 테스트 계획

### 9.1 단위 테스트

#### 서비스 레이어 테스트

```rust
// tests/services/test_mission_service.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_state_transitions() {
        let status = MissionStatus::Created;
        assert!(status.can_transition_to(&MissionStatus::Assigned));
        assert!(status.can_transition_to(&MissionStatus::Cancelled));
        assert!(!status.can_transition_to(&MissionStatus::Executing));
        assert!(!status.can_transition_to(&MissionStatus::Completed));
    }

    #[test]
    fn test_invalid_state_transitions() {
        let status = MissionStatus::Completed;
        assert!(!status.can_transition_to(&MissionStatus::Created));
        assert!(!status.can_transition_to(&MissionStatus::Assigned));
        assert!(!status.can_transition_to(&MissionStatus::Cancelled));
    }

    #[test]
    fn test_mission_to_vda5050_order() {
        // 미션 스텝 → VDA5050 Order 변환 검증
        // 노드 sequence_id는 짝수, 엣지 sequence_id는 홀수
        // ...
    }
}
```

#### 인증 테스트

```rust
#[cfg(test)]
mod tests {
    use crate::services::auth_service;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "secure_password_123";
        let hash = auth_service::hash_password(password).unwrap();
        assert!(auth_service::verify_password(password, &hash).unwrap());
        assert!(!auth_service::verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_jwt_generation_and_validation() {
        let config = test_config();
        let user = test_user();
        let token = auth_service::generate_token(&config, &user, "access", 3600).unwrap();
        let claims = auth_service::validate_token(&config, &token).unwrap();
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.role, user.role);
    }

    #[test]
    fn test_expired_jwt() {
        let config = test_config();
        let user = test_user();
        let token = auth_service::generate_token(&config, &user, "access", 0).unwrap();
        // 즉시 만료
        std::thread::sleep(std::time::Duration::from_secs(1));
        let result = auth_service::validate_token(&config, &token);
        assert!(result.is_err());
    }
}
```

#### VDA5050 직렬화 테스트

```rust
#[cfg(test)]
mod tests {
    use crate::mqtt::vda5050::*;

    #[test]
    fn test_vda5050_order_serialization() {
        let order = Vda5050Order {
            header_id: 1,
            timestamp: chrono::Utc::now(),
            version: "2.0.0".to_string(),
            manufacturer: "TestMfg".to_string(),
            serial_number: "SN001".to_string(),
            order_id: "order-001".to_string(),
            order_update_id: 0,
            zone_set_id: None,
            nodes: vec![],
            edges: vec![],
        };

        let json = serde_json::to_string(&order).unwrap();
        let deserialized: Vda5050Order = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.order_id, "order-001");
        assert_eq!(deserialized.version, "2.0.0");

        // camelCase 확인
        assert!(json.contains("\"orderId\""));
        assert!(json.contains("\"headerId\""));
        assert!(json.contains("\"serialNumber\""));
    }

    #[test]
    fn test_vda5050_state_deserialization() {
        let json = r#"{
            "headerId": 42,
            "timestamp": "2026-03-21T10:00:00Z",
            "version": "2.0.0",
            "manufacturer": "TestMfg",
            "serialNumber": "SN001",
            "orderId": "order-001",
            "orderUpdateId": 0,
            "lastNodeId": "node-5",
            "lastNodeSequenceId": 8,
            "nodeStates": [],
            "edgeStates": [],
            "driving": true,
            "paused": false,
            "newBaseRequest": false,
            "distanceSinceLastNode": 1.5,
            "actionStates": [],
            "batteryState": {
                "batteryCharge": 85.5,
                "charging": false
            },
            "operatingMode": "AUTOMATIC",
            "errors": [],
            "information": [],
            "safetyState": {
                "eStop": "NONE",
                "fieldViolation": false
            }
        }"#;

        let state: Vda5050State = serde_json::from_str(json).unwrap();
        assert_eq!(state.last_node_id, "node-5");
        assert_eq!(state.battery_state.battery_charge, 85.5);
        assert!(state.driving);
    }
}
```

### 9.2 통합 테스트

#### API 통합 테스트 (testcontainers 사용)

```rust
// tests/common/mod.rs
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;

pub struct TestApp {
    pub client: reqwest::Client,
    pub base_url: String,
    pub admin_token: String,
    // containers는 Drop 시 자동 정리
    _pg_container: ContainerAsync<Postgres>,
    _redis_container: ContainerAsync<Redis>,
}

impl TestApp {
    pub async fn new() -> Self {
        // PostgreSQL 컨테이너 시작
        let pg = Postgres::default()
            .start()
            .await
            .expect("Failed to start PostgreSQL");

        // Redis 컨테이너 시작
        let redis = Redis::default()
            .start()
            .await
            .expect("Failed to start Redis");

        // 테스트 설정
        let config = AppConfig {
            database_url: format!(
                "postgresql://postgres:postgres@localhost:{}/postgres",
                pg.get_host_port_ipv4(5432).await.unwrap()
            ),
            redis_url: format!(
                "redis://localhost:{}",
                redis.get_host_port_ipv4(6379).await.unwrap()
            ),
            // ... 기타 설정
        };

        // 마이그레이션 실행
        let pool = db::pool::create_pool(&config.database_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // 앱 시작
        let app = api::router::build_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // 관리자 토큰 생성
        // ...

        Self {
            client: reqwest::Client::new(),
            base_url: format!("http://{}", addr),
            admin_token: String::new(),
            _pg_container: pg,
            _redis_container: redis,
        }
    }

    pub fn auth_header(&self) -> (&str, String) {
        ("Authorization", format!("Bearer {}", self.admin_token))
    }
}

// tests/api/test_robots.rs
#[tokio::test]
async fn test_robot_crud() {
    let app = TestApp::new().await;

    // 로봇 생성
    let res = app.client
        .post(&format!("{}/api/robots", app.base_url))
        .header(app.auth_header().0, &app.auth_header().1)
        .json(&serde_json::json!({
            "name": "Test Robot",
            "model_id": "model-a",
            "serial_number": "SN-TEST-001",
            "manufacturer": "TestMfg"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 201);
    let robot: serde_json::Value = res.json().await.unwrap();
    let robot_id = robot["id"].as_str().unwrap();

    // 로봇 조회
    let res = app.client
        .get(&format!("{}/api/robots/{}", app.base_url, robot_id))
        .header(app.auth_header().0, &app.auth_header().1)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let fetched: serde_json::Value = res.json().await.unwrap();
    assert_eq!(fetched["name"], "Test Robot");
    assert_eq!(fetched["status"], "OFFLINE");

    // 로봇 수정
    let res = app.client
        .put(&format!("{}/api/robots/{}", app.base_url, robot_id))
        .header(app.auth_header().0, &app.auth_header().1)
        .json(&serde_json::json!({"name": "Updated Robot"}))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);

    // 로봇 삭제
    let res = app.client
        .delete(&format!("{}/api/robots/{}", app.base_url, robot_id))
        .header(app.auth_header().0, &app.auth_header().1)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 204);
}
```

#### WebSocket 통합 테스트

```rust
// tests/integration/test_websocket.rs
#[tokio::test]
async fn test_websocket_telemetry_fanout() {
    let app = TestApp::new().await;

    // WebSocket 연결
    let ws_url = format!(
        "ws://{}/api/ws?token={}",
        app.addr, app.admin_token
    );
    let (mut ws_stream, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("WebSocket 연결 실패");

    // 텔레메트리 토픽 구독
    let subscribe_msg = serde_json::json!({
        "action": "subscribe",
        "topic": "telemetry:*"
    });
    ws_stream.send(Message::Text(subscribe_msg.to_string())).await.unwrap();

    // 구독 확인 응답 수신
    let msg = ws_stream.next().await.unwrap().unwrap();
    let response: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
    assert_eq!(response["msg_type"], "subscribed");

    // 이벤트 버스에 텔레메트리 이벤트 발행
    app.event_bus.send(AppEvent::TelemetryReceived {
        robot_id: Uuid::new_v4(),
        telemetry: test_telemetry(),
    }).unwrap();

    // WebSocket으로 텔레메트리 수신 확인
    let msg = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next(),
    ).await.unwrap().unwrap().unwrap();

    let data: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
    assert_eq!(data["msg_type"], "data");
    assert!(data["payload"]["position_x"].is_number());
}
```

### 9.3 부하 테스트

#### k6 스크립트 예시

```javascript
// tests/load/api_load_test.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import ws from 'k6/ws';

export const options = {
    stages: [
        { duration: '1m', target: 50 },    // ramp up
        { duration: '5m', target: 100 },   // sustained load
        { duration: '1m', target: 0 },     // ramp down
    ],
    thresholds: {
        http_req_duration: ['p(95)<100'],   // 95th percentile < 100ms
        http_req_failed: ['rate<0.01'],      // 에러율 < 1%
    },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

export function setup() {
    const loginRes = http.post(`${BASE_URL}/api/auth/login`, JSON.stringify({
        email: 'admin@amr.local',
        password: 'admin123',
    }), { headers: { 'Content-Type': 'application/json' } });

    return { token: loginRes.json('access_token') };
}

export default function (data) {
    const headers = {
        Authorization: `Bearer ${data.token}`,
        'Content-Type': 'application/json',
    };

    // 로봇 목록 조회
    const robotsRes = http.get(`${BASE_URL}/api/robots?page=1&limit=20`, { headers });
    check(robotsRes, {
        'robots status 200': (r) => r.status === 200,
        'robots response time < 100ms': (r) => r.timings.duration < 100,
    });

    // 미션 목록 조회
    const missionsRes = http.get(`${BASE_URL}/api/missions?page=1&limit=20`, { headers });
    check(missionsRes, {
        'missions status 200': (r) => r.status === 200,
    });

    sleep(0.5);
}
```

#### WebSocket 동시 연결 테스트

```javascript
// tests/load/ws_load_test.js
import ws from 'k6/ws';
import { check } from 'k6';

export const options = {
    vus: 1000,           // 1000개 동시 WebSocket 연결
    duration: '5m',
};

export default function () {
    const url = `ws://localhost:8080/api/ws?token=${__ENV.TOKEN}`;

    const res = ws.connect(url, {}, function (socket) {
        socket.on('open', () => {
            socket.send(JSON.stringify({
                action: 'subscribe',
                topic: 'telemetry:*',
            }));
        });

        socket.on('message', (data) => {
            const msg = JSON.parse(data);
            check(msg, {
                'has msg_type': (m) => m.msg_type !== undefined,
            });
        });

        socket.setTimeout(() => {
            socket.close();
        }, 300000); // 5분 후 닫기
    });

    check(res, { 'ws status 101': (r) => r && r.status === 101 });
}
```

---

## 10. 목 서버 전략

### 10.1 Simulation Engine 목 서버

**파일**: `tests/mocks/mock_sim_engine.rs`

목 gRPC 서버를 tonic으로 구현하여 통합 테스트에서 사용한다.

```rust
use tonic::{Request, Response, Status};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct MockSimEngine {
    telemetry_data: Vec<TelemetryMessage>,
}

#[tonic::async_trait]
impl SimulationService for MockSimEngine {
    type StartSimulationStream = ReceiverStream<Result<SimState, Status>>;

    async fn start_simulation(
        &self,
        _request: Request<StartSimulationRequest>,
    ) -> Result<Response<Self::StartSimulationStream>, Status> {
        let (tx, rx) = mpsc::channel(100);

        // 미리 정의된 시뮬레이션 상태를 주기적으로 전송
        tokio::spawn(async move {
            for i in 0..100 {
                let state = SimState {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    robots: vec![/* 가상 로봇 상태 */],
                };
                if tx.send(Ok(state)).await.is_err() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn stop_simulation(
        &self,
        _request: Request<StopSimulationRequest>,
    ) -> Result<Response<StopSimulationResponse>, Status> {
        Ok(Response::new(StopSimulationResponse {}))
    }

    async fn spawn_robot(
        &self,
        _request: Request<SpawnRobotRequest>,
    ) -> Result<Response<SpawnRobotResponse>, Status> {
        Ok(Response::new(SpawnRobotResponse {
            robot_id: uuid::Uuid::new_v4().to_string(),
        }))
    }

    // ... 기타 메서드
}

/// 테스트용 목 서버 시작
pub async fn start_mock_sim_engine(port: u16) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let service = MockSimEngine::default();

    tonic::transport::Server::builder()
        .add_service(SimulationServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
```

### 10.2 Asset Manager 목 서버

인메모리 `HashMap`으로 에셋을 저장하는 목 구현. 업로드 시 바이트를 메모리에 보관하고, 다운로드 시 반환.

### 10.3 Map Manager 목 서버

샘플 포인트클라우드 메타데이터와 미리 정의된 로드맵 그래프를 반환하는 목 구현. `FindPath`는 간단한 BFS로 구현.

### 10.4 MQTT 브로커

로컬 개발 및 테스트에서는 docker-compose의 Eclipse Mosquitto를 사용한다. testcontainers에서도 Mosquitto 컨테이너를 자동 시작.

```rust
// testcontainers로 Mosquitto 시작
use testcontainers_modules::mosquitto::Mosquitto;

let mosquitto = Mosquitto::default()
    .start()
    .await
    .expect("Failed to start Mosquitto");

let mqtt_port = mosquitto.get_host_port_ipv4(1883).await.unwrap();
```

---

## 11. 다른 팀과의 인터페이스 계약

### 11.1 Frontend Team (Team 1) 인터페이스

#### REST API 계약

- **문서 형식**: OpenAPI 3.1 스펙 (자동 생성)
- **생성 도구**: `utoipa` 크레이트
- **엔드포인트**: `GET /api-docs/openapi.json` → JSON 스펙 반환
- **UI**: `GET /swagger-ui/` → Swagger UI

프론트엔드 팀은 이 OpenAPI 스펙을 기반으로:
1. TypeScript 타입을 자동 생성 (`openapi-typescript` 도구 사용)
2. API 클라이언트 코드를 자동 생성
3. 목 서버를 생성하여 독립 개발

#### WebSocket 메시지 JSON Schema

프론트엔드에 제공할 WebSocket 메시지 스키마:

```json
{
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "WebSocket Message",
    "oneOf": [
        {
            "title": "Outgoing (Server → Client)",
            "type": "object",
            "properties": {
                "msg_type": { "enum": ["data", "error", "subscribed", "unsubscribed", "ping"] },
                "topic": { "type": "string" },
                "payload": { "type": "object" },
                "timestamp": { "type": "integer", "description": "Unix timestamp (ms)" }
            },
            "required": ["msg_type", "timestamp"]
        },
        {
            "title": "Incoming (Client → Server)",
            "type": "object",
            "properties": {
                "action": { "enum": ["subscribe", "unsubscribe", "pong"] },
                "topic": { "type": "string" }
            },
            "required": ["action"]
        }
    ]
}
```

#### 인증 흐름 계약

1. 프론트엔드는 `POST /api/auth/login`으로 access_token과 refresh_token을 받는다
2. 모든 API 요청에 `Authorization: Bearer {access_token}` 헤더를 포함한다
3. 401 응답 시 `POST /api/auth/refresh`로 토큰을 갱신한다
4. refresh_token도 만료되면 로그인 페이지로 리다이렉트한다
5. WebSocket 연결 시 쿼리 파라미터로 토큰을 전달한다: `ws://host/api/ws?token={access_token}`

### 11.2 Simulation Engine Team (Team 3) 인터페이스

> 📋 Proto 서비스 정의 상세: [`integration-spec.md`](../integration/integration-spec.md#shared-proto) 참조

### 11.3 Asset Manager Team (Team 4) 인터페이스

> 📋 Proto 서비스 정의 상세: [`integration-spec.md`](../integration/integration-spec.md#shared-proto) 참조

### 11.4 Map Manager Team (Team 5) 인터페이스

> 📋 Proto 서비스 정의 상세: [`integration-spec.md`](../integration/integration-spec.md#shared-proto) 참조

### 11.5 Proto/Integration Team (Team 6) 인터페이스

- Backend 팀은 `proto/` 디렉토리의 `.proto` 파일을 **소비자** 역할로 사용한다
- Proto 파일의 원본 관리는 Team 6 또는 각 서비스 팀이 담당한다
- Backend 팀은 `build.rs`에서 `tonic-build`로 Rust 코드를 자동 생성한다
- Proto 파일 변경 시 Backend 팀에 사전 통보가 필요하다 (breaking change 방지)
- 버전 관리: proto 파일에 `package` 버전을 명시하고, 하위 호환성을 유지한다

---

## 부록 A: 데이터베이스 모델 구조체

**파일**: `src/db/models.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Robot {
    pub id: Uuid,
    pub name: String,
    pub model_id: String,
    pub serial_number: String,
    pub manufacturer: String,
    pub status: String,
    pub battery_level: Option<f64>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
    pub position_z: Option<f64>,
    pub orientation_theta: Option<f64>,
    pub current_map_id: Option<Uuid>,
    pub current_mission_id: Option<Uuid>,
    pub operating_mode: Option<String>,
    pub config: serde_json::Value,
    pub firmware_version: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Map {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub bounds_min_x: Option<f64>,
    pub bounds_min_y: Option<f64>,
    pub bounds_min_z: Option<f64>,
    pub bounds_max_x: Option<f64>,
    pub bounds_max_y: Option<f64>,
    pub bounds_max_z: Option<f64>,
    pub point_cloud_size_bytes: Option<i64>,
    pub point_count: Option<i64>,
    pub tile_count: Option<i32>,
    pub resolution: Option<f64>,
    pub processing_job_id: Option<String>,
    pub processing_progress: Option<f64>,
    pub processing_error: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Mission {
    pub id: Uuid,
    pub robot_id: Option<Uuid>,
    pub map_id: Uuid,
    pub mission_type: String,
    pub status: String,
    pub priority: i32,
    pub progress: f64,
    pub error_message: Option<String>,
    pub vda5050_order_id: Option<String>,
    pub vda5050_order_update_id: Option<i32>,
    pub created_by: Uuid,
    pub assigned_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MissionStep {
    pub id: Uuid,
    pub mission_id: Uuid,
    pub sequence: i32,
    pub step_type: String,
    pub target_node_id: Option<String>,
    pub parameters: serde_json::Value,
    pub status: String,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RoadmapNodeModel {
    pub id: Uuid,
    pub map_id: Uuid,
    pub node_id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub name: Option<String>,
    pub node_type: String,
    pub properties: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RoadmapEdgeModel {
    pub id: Uuid,
    pub map_id: Uuid,
    pub edge_id: String,
    pub start_node_id: String,
    pub end_node_id: String,
    pub bidirectional: bool,
    pub weight: f64,
    pub max_speed: Option<f64>,
    pub properties: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SemanticRegionModel {
    pub id: Uuid,
    pub map_id: Uuid,
    pub name: String,
    pub region_type: String,
    // polygon은 PostGIS geometry이므로 별도 처리 필요
    pub max_robots: Option<i32>,
    pub speed_limit: Option<f64>,
    pub allowed_robot_types: Option<Vec<String>>,
    pub allowed_directions: Option<String>,
    pub properties: serde_json::Value,
    pub priority: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AssetModel {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub format: String,
    pub file_size: i64,
    pub s3_bucket: String,
    pub s3_key: String,
    pub content_type: Option<String>,
    pub checksum: Option<String>,
    pub metadata: serde_json::Value,
    pub conversion_status: Option<String>,
    pub original_asset_id: Option<Uuid>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TelemetryRow {
    pub time: DateTime<Utc>,
    pub robot_id: Uuid,
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
    pub orientation_theta: f64,
    pub velocity_linear: f64,
    pub velocity_angular: f64,
    pub battery_level: f64,
    pub battery_charging: bool,
    pub operating_mode: String,
    pub map_id: Option<Uuid>,
    pub errors: serde_json::Value,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PluginModel {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub plugin_type: String,
    pub enabled: bool,
    pub wasm_s3_bucket: Option<String>,
    pub wasm_s3_key: Option<String>,
    pub wasm_size_bytes: Option<i64>,
    pub wasm_checksum: Option<String>,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub hooks: Vec<String>,
    pub config: serde_json::Value,
    pub uploaded_by: Option<Uuid>,
    pub last_invoked_at: Option<DateTime<Utc>>,
    pub invocation_count: i64,
    pub error_count: i64,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## 부록 B: 서비스 레이어 헬퍼

#### `src/services/auth_service.rs`

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};

use crate::api::middleware::auth::Claims;
use crate::config::AppConfig;
use crate::db::models::User;

/// Argon2id로 비밀번호 해싱
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?;
    Ok(hash.to_string())
}

/// 비밀번호 검증
pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// JWT 토큰 생성
pub fn generate_token(
    config: &AppConfig,
    user: &User,
    token_type: &str,
    expiration_secs: u64,
) -> anyhow::Result<String> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        role: user.role.clone(),
        exp: now + expiration_secs as usize,
        iat: now,
        token_type: token_type.to_string(),
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;

    Ok(token)
}

/// JWT 토큰 검증
pub fn validate_token(config: &AppConfig, token: &str) -> anyhow::Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}
```

---

## 부록 C: 핵심 DB 쿼리 예시

#### `src/db/queries/robots.rs`

```rust
use sqlx::PgPool;
use uuid::Uuid;
use crate::db::models::Robot;

pub async fn find_all(
    pool: &PgPool,
    status: Option<&str>,
    search: Option<&str>,
    page: u32,
    limit: u32,
) -> sqlx::Result<(Vec<Robot>, i64)> {
    let offset = ((page - 1) * limit) as i64;
    let limit = limit as i64;

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM robots
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::text IS NULL OR name ILIKE '%' || $2 || '%')
        "#,
    )
    .bind(status)
    .bind(search)
    .fetch_one(pool)
    .await?;

    let robots = sqlx::query_as::<_, Robot>(
        r#"
        SELECT * FROM robots
        WHERE ($1::text IS NULL OR status = $1)
          AND ($2::text IS NULL OR name ILIKE '%' || $2 || '%')
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(status)
    .bind(search)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((robots, total.0))
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Robot>> {
    sqlx::query_as::<_, Robot>("SELECT * FROM robots WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_serial(
    pool: &PgPool,
    manufacturer: &str,
    serial_number: &str,
) -> sqlx::Result<Option<Robot>> {
    sqlx::query_as::<_, Robot>(
        "SELECT * FROM robots WHERE manufacturer = $1 AND serial_number = $2",
    )
    .bind(manufacturer)
    .bind(serial_number)
    .fetch_optional(pool)
    .await
}

pub async fn insert(pool: &PgPool, robot: &Robot) -> sqlx::Result<Robot> {
    sqlx::query_as::<_, Robot>(
        r#"
        INSERT INTO robots (name, model_id, serial_number, manufacturer, config)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(&robot.name)
    .bind(&robot.model_id)
    .bind(&robot.serial_number)
    .bind(&robot.manufacturer)
    .bind(&robot.config)
    .fetch_one(pool)
    .await
}

pub async fn update(pool: &PgPool, id: Uuid, name: Option<&str>, config: Option<&serde_json::Value>) -> sqlx::Result<Robot> {
    sqlx::query_as::<_, Robot>(
        r#"
        UPDATE robots
        SET name = COALESCE($2, name),
            config = COALESCE($3, config)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(config)
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &PgPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM robots WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> sqlx::Result<()> {
    sqlx::query("UPDATE robots SET status = $2, last_seen_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(status)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_status_by_serial(
    pool: &PgPool,
    manufacturer: &str,
    serial_number: &str,
    status: &str,
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE robots SET status = $3, last_seen_at = NOW() WHERE manufacturer = $1 AND serial_number = $2",
    )
    .bind(manufacturer)
    .bind(serial_number)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn has_active_missions(pool: &PgPool, robot_id: Uuid) -> sqlx::Result<bool> {
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM missions WHERE robot_id = $1 AND status IN ('ASSIGNED', 'EXECUTING')",
    )
    .bind(robot_id)
    .fetch_one(pool)
    .await?;

    Ok(result.0 > 0)
}
```

#### `src/db/queries/telemetry.rs`

```rust
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::services::telemetry_service::TelemetryMessage;
use crate::db::models::TelemetryRow;

/// 배치 삽입 (COPY 프로토콜 사용이 이상적이나, sqlx에서는 multi-row INSERT 사용)
pub async fn batch_insert(pool: &PgPool, messages: &[TelemetryMessage]) -> sqlx::Result<()> {
    if messages.is_empty() {
        return Ok(());
    }

    // 동적 multi-row INSERT 구성
    let mut query = String::from(
        "INSERT INTO telemetry (time, robot_id, position_x, position_y, position_z, \
         orientation_theta, velocity_linear, velocity_angular, battery_level, \
         battery_charging, operating_mode, map_id, errors) VALUES "
    );

    let mut params: Vec<String> = Vec::new();
    for (i, _) in messages.iter().enumerate() {
        let base = i * 13 + 1;
        params.push(format!(
            "(${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${})",
            base, base+1, base+2, base+3, base+4, base+5, base+6,
            base+7, base+8, base+9, base+10, base+11, base+12
        ));
    }
    query.push_str(&params.join(", "));

    let mut q = sqlx::query(&query);
    for msg in messages {
        q = q
            .bind(msg.timestamp)
            .bind(msg.robot_id)
            .bind(msg.position_x)
            .bind(msg.position_y)
            .bind(msg.position_z)
            .bind(msg.orientation_theta)
            .bind(msg.velocity_linear)
            .bind(msg.velocity_angular)
            .bind(msg.battery_level)
            .bind(msg.battery_charging)
            .bind(&msg.operating_mode)
            .bind(msg.map_id)
            .bind(serde_json::to_value(&msg.errors).unwrap());
    }

    q.execute(pool).await?;
    Ok(())
}

/// 시간 범위로 텔레메트리 조회 (원시 데이터)
pub async fn find_by_robot_and_time_range(
    pool: &PgPool,
    robot_id: Uuid,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    limit: i64,
) -> sqlx::Result<Vec<TelemetryRow>> {
    sqlx::query_as::<_, TelemetryRow>(
        r#"
        SELECT * FROM telemetry
        WHERE robot_id = $1 AND time >= $2 AND time <= $3
        ORDER BY time DESC
        LIMIT $4
        "#,
    )
    .bind(robot_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// 다운샘플링된 텔레메트리 조회 (1분 평균)
pub async fn find_downsampled(
    pool: &PgPool,
    robot_id: Uuid,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    bucket_interval: &str,  // '1 minute', '5 minutes', '1 hour'
    limit: i64,
) -> sqlx::Result<Vec<TelemetryRow>> {
    let query = format!(
        r#"
        SELECT
            time_bucket('{interval}', time) AS time,
            robot_id,
            AVG(position_x) AS position_x,
            AVG(position_y) AS position_y,
            AVG(position_z) AS position_z,
            AVG(orientation_theta) AS orientation_theta,
            AVG(velocity_linear) AS velocity_linear,
            AVG(velocity_angular) AS velocity_angular,
            AVG(battery_level) AS battery_level,
            bool_or(battery_charging) AS battery_charging,
            MODE() WITHIN GROUP (ORDER BY operating_mode) AS operating_mode,
            NULL::uuid AS map_id,
            '[]'::jsonb AS errors,
            NULL::jsonb AS payload
        FROM telemetry
        WHERE robot_id = $1 AND time >= $2 AND time <= $3
        GROUP BY time_bucket('{interval}', time), robot_id
        ORDER BY time DESC
        LIMIT $4
        "#,
        interval = bucket_interval
    );

    sqlx::query_as::<_, TelemetryRow>(&query)
        .bind(robot_id)
        .bind(from)
        .bind(to)
        .bind(limit)
        .fetch_all(pool)
        .await
}
```

---

*문서 끝*
