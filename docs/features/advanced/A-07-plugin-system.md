# A-07: 플러그인 시스템 (Plugin System)

## 1. 개요

### 1.1 목표
WASM 기반 플러그인 런타임으로 안전하게 서드파티 코드를 실행하고,
HTTP 웹훅 디스패처로 외부 시스템 연동을 지원하며,
ROS 2 브릿지를 플러그인으로 구현하여 DDS↔MQTT 변환을 제공한다.

### 1.2 담당팀

| 팀 | 역할 |
|----|------|
| **Team 2 (Backend)** | WASM 런타임, 플러그인 라이프사이클, 웹훅 디스패처, ROS 2 브릿지 |

### 1.3 Phase: **4**

### 1.4 선행조건
- A-01: 미션 시스템 (플러그인 이벤트 소스)
- A-04: VDA5050 실로봇 통신 (ROS 2 브릿지 전제)

---

## 2. 스코프

### In-Scope
- Wasmtime WASM 런타임 통합
- 플러그인 라이프사이클 관리 (업로드/검증/등록/활성화/비활성화)
- 플러그인 API (이벤트 핸들러 + 시스템 호출)
- HTTP 웹훅 디스패처 (외부 URL에 이벤트 전송)
- ROS 2 Bridge 플러그인 (DDS ↔ MQTT 토픽 변환)

### Out-of-Scope
- 플러그인 마켓플레이스 (→ 향후)
- 플러그인 GUI 커스텀 위젯 (→ 향후)
- 다국어 플러그인 SDK (→ 향후, 현재 Rust/AssemblyScript만)

---

## 3. 상세 스펙

### 3.1 WASM 런타임

```rust
use wasmtime::{Engine, Store, Module, Linker, Instance};

pub struct PluginRuntime {
    engine: Engine,
    plugins: HashMap<String, PluginInstance>,
}

pub struct PluginInstance {
    module: Module,
    store: Store<PluginState>,
    instance: Instance,
    metadata: PluginMetadata,
    status: PluginStatus, // Uploaded, Validated, Registered, Enabled, Disabled
}
```

- 메모리 제한: 플러그인당 최대 64MB
- 실행 시간 제한: 호출당 최대 5초 (fuel-based)
- WASI 서브셋: 파일시스템 접근 불가, 네트워크 불가 (Host function으로만 외부 접근)

### 3.2 플러그인 라이프사이클

```
UPLOADED → (검증) → VALIDATED → (등록) → REGISTERED → (활성화) → ENABLED
                                                          ↕
                                                      DISABLED
```

| 단계 | 동작 |
|------|------|
| Upload | `.wasm` 파일 업로드 (`POST /api/v1/plugins`, multipart) |
| Validate | WASM 바이너리 검증, 필수 export 함수 존재 확인, 메모리 제한 검사 |
| Register | 메타데이터 DB 저장, 이벤트 구독 목록 등록 |
| Enable | 런타임에 인스턴스 생성, 이벤트 디스패처에 연결 |
| Disable | 인스턴스 정지, 이벤트 구독 해제 (데이터 보존) |

### 3.3 플러그인 API

```rust
// 플러그인이 구현해야 하는 export 함수 (WASM)
extern "C" {
    fn plugin_init() -> i32;
    fn on_mission_created(mission_json: *const u8, len: u32) -> i32;
    fn on_mission_completed(mission_json: *const u8, len: u32) -> i32;
    fn on_robot_state_changed(state_json: *const u8, len: u32) -> i32;
}

// 플러그인이 호출할 수 있는 host 함수
extern "C" {
    fn host_create_mission(json: *const u8, len: u32) -> i32;
    fn host_get_robot_state(robot_id: *const u8, len: u32, out: *mut u8, out_len: *mut u32) -> i32;
    fn host_log(level: i32, msg: *const u8, len: u32);
    fn host_kv_get(key: *const u8, key_len: u32, out: *mut u8, out_len: *mut u32) -> i32;
    fn host_kv_set(key: *const u8, key_len: u32, val: *const u8, val_len: u32) -> i32;
}
```

### 3.4 HTTP 웹훅 디스패처

```rust
pub struct WebhookConfig {
    pub url: String,                    // 대상 URL
    pub events: Vec<String>,            // 구독 이벤트 목록
    pub headers: HashMap<String, String>, // 커스텀 헤더 (인증 등)
    pub retry_count: u32,               // 재시도 횟수 (default 3)
    pub timeout_ms: u64,                // 요청 타임아웃 (default 5000)
}
```

- 이벤트 발생 시 등록된 URL에 POST 요청 (JSON payload)
- 실패 시 지수 백오프 재시도 (1s, 2s, 4s)
- 웹훅 실행 로그 저장 (성공/실패, 응답 코드, 소요 시간)

### 3.5 ROS 2 Bridge 플러그인

```
[ROS 2 Node] ←DDS→ [ROS 2 Bridge Plugin] ←MQTT→ [Backend] ←WS→ [Frontend]
```

- DDS (CycloneDDS/FastDDS) ↔ MQTT 토픽 양방향 변환
- 매핑 설정: ROS 2 토픽 → MQTT 토픽 (JSON 구성 파일)
- 지원 메시지: `geometry_msgs/Twist`, `nav_msgs/Odometry`, `sensor_msgs/LaserScan`
- 네이티브 플러그인 (WASM이 아닌 별도 프로세스, gRPC로 Backend와 통신)

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#plugin-api) 참조

---

## 4. 구현 모듈

| 모듈 | 팀 | 경로 |
|------|-----|------|
| `plugin_runtime.rs` | Team 2 | `backend/src/plugins/plugin_runtime.rs` |
| `plugin_service.rs` | Team 2 | `backend/src/services/plugin_service.rs` |
| `webhook_dispatcher.rs` | Team 2 | `backend/src/plugins/webhook_dispatcher.rs` |
| `plugin_api_host.rs` | Team 2 | `backend/src/plugins/plugin_api_host.rs` |
| `ros2_bridge/` | Team 2 | `backend/src/plugins/ros2_bridge/` (별도 바이너리) |

---

## 5. 의존성

| 항목 | 종류 | 비고 |
|------|------|------|
| wasmtime 19+ | 크레이트 | WASM 런타임 |
| reqwest | 크레이트 | 웹훅 HTTP 클라이언트 |
| CycloneDDS / rclrs | 라이브러리 | ROS 2 브릿지 (선택적) |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#plugin-data) 참조

---

## 6. 테스트 기준

| 테스트 유형 | 기준 |
|------------|------|
| 단위 테스트 | 플러그인 로드/언로드, API 호출 정확성 |
| 샌드박스 테스트 | 메모리 초과, 시간 초과 시 안전한 종료 확인 |
| 웹훅 테스트 | 이벤트 발생 → HTTP POST 전달 → 재시도 로직 |
| ROS 2 테스트 | DDS ↔ MQTT 메시지 양방향 변환 정확성 |
| 보안 테스트 | 악성 WASM 모듈 거부 (파일시스템/네트워크 접근 차단) |

---

## 7. 완료 조건

- [ ] WASM 플러그인 업로드 → 검증 → 활성화 라이프사이클 동작
- [ ] 플러그인 API (이벤트 수신 + Host 함수 호출) 정상 동작
- [ ] 메모리/시간 제한 샌드박스 안전성 확인
- [ ] HTTP 웹훅 디스패처 이벤트 전달 및 재시도 동작
- [ ] ROS 2 Bridge 기본 토픽 변환 동작 확인
- [ ] 예제 플러그인 3종 제공 (Rust, AssemblyScript)
