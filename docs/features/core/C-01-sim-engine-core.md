# C-01: Simulation Engine Core

> **문서 버전**: 1.0
> **최종 수정일**: 2026-03-22
> **담당 팀**: Team 3 (Sim Engine)
> **Phase**: 1 (Prototype)

---

## 1. 개요

### 1.1 목표
커스텀 경량 물리 엔진을 사용하여 AMR 주행 시뮬레이션의 기본 골격을 구축한다. 차동 구동 로봇 모델, 2D LiDAR 센서, 고정 타임스텝 루프, gRPC 텔레메트리 스트리밍을 포함하며, 백엔드 및 프론트엔드와 연동 가능한 최소 기능 시뮬레이터를 제공한다.

### 1.2 배경
Sim Engine은 실제 로봇이 생성하는 것과 동일한 형식의 텔레메트리를 생산하여, 상위 시스템(Backend, Frontend)이 실제 로봇과 시뮬레이션 로봇을 구분하지 않고 동작할 수 있도록 한다. Phase 1에서는 2D 평면 kinematics와 기본 충돌 감지에 집중한다.

---

## 2. 스코프

### 2.1 In-Scope (이 기능에서 구현)
- 커스텀 경량 물리 백엔드 (2D kinematics, 간단한 충돌)
- 차동 구동 로봇 모델 (2-wheel differential drive)
- 2D LiDAR 센서 시뮬레이션 (ray casting)
- 고정 타임스텝 시뮬레이션 루프 (실시간 비율 조절)
- gRPC 서버 (CreateSim, StartSim, StopSim, SpawnRobot, SendCommand, StreamTelemetry)
- 기본 씬 로딩 (JSON 월드 정의)
- 배터리 모델 (단순 선형 소모)

### 2.2 Out-of-Scope (A-05에서 구현)
- MuJoCo, Isaac Sim 백엔드
- 3D LiDAR, Camera, IMU, Encoder 센서
- OpenUSD 씬 로딩
- 메카넘 드라이브
- 동적 객체, 보행자 시뮬레이션
- Python 바인딩 / 시나리오 스크립팅
- Vulkan 렌더링 파이프라인

---

## 3. 상세 스펙

### 3.1 커스텀 경량 물리 엔진

2D 평면 상에서의 로봇 이동을 시뮬레이션한다 (x, y, theta).

**차동 구동 kinematics:**
```
v_left  = v - omega * L/2
v_right = v + omega * L/2
delta_x     = v * cos(theta) * dt
delta_y     = v * sin(theta) * dt
delta_theta = omega * dt
```

**제약 조건:**
- 가속도 제한: `max_acceleration`, `max_angular_acceleration` 적용
- 속도 클램핑: 명령 속도가 `max_linear_speed`, `max_angular_speed` 초과 시 클램핑
- 타임스텝: 기본 10ms (100Hz), 설정 가능 (`sim_config.timestep_ms`)

**충돌 감지/응답:**
- 로봇: 원형 충돌체 (`collision_radius`)
- 벽/장애물: AABB (Axis-Aligned Bounding Box)
- 감지: 원-AABB 교차 테스트, 원-원 교차 테스트
- 응답: 겹침(penetration) 발생 시 최소 분리 벡터(MTV) 방향으로 위치 보정, 속도 0으로 리셋

### 3.2 로봇 모델

```cpp
struct RobotConfig {
    std::string id;
    double wheel_radius = 0.05;        // m
    double wheel_separation = 0.3;     // m
    double max_linear_speed = 2.0;     // m/s
    double max_angular_speed = 3.14;   // rad/s
    double max_acceleration = 1.0;     // m/s^2
    double max_angular_accel = 2.0;    // rad/s^2
    double battery_capacity = 100.0;   // %
    double battery_drain_rate = 0.01;  // %/s (이동 중)
    double battery_idle_rate = 0.001;  // %/s (대기 중)
    double collision_radius = 0.25;    // m
};

struct RobotState {
    std::string id;
    double x = 0.0, y = 0.0;          // position (m)
    double theta = 0.0;                // heading (rad)
    double v_linear = 0.0;            // current linear velocity (m/s)
    double v_angular = 0.0;           // current angular velocity (rad/s)
    double battery = 100.0;           // battery level (%)
    bool is_colliding = false;
    uint64_t timestamp_ns = 0;        // simulation time (ns)
};
```

### 3.3 2D LiDAR 센서

| 파라미터 | 기본값 | 설명 |
|---------|--------|------|
| `min_angle` | -pi | 스캔 시작 각도 (rad) |
| `max_angle` | pi | 스캔 종료 각도 (rad) |
| `angle_increment` | pi/180 | 각도 해상도 (rad) → 360 rays |
| `max_range` | 30.0 | 최대 감지 거리 (m) |
| `noise_stddev` | 0.01 | Gaussian 노이즈 표준편차 (m) |
| `update_rate` | 10 | 업데이트 주기 (Hz) |

**구현 방식:**
- 로봇 위치에서 각 방향으로 ray를 발사
- 월드 지오메트리(벽, 장애물, 다른 로봇)와 교차 테스트
- 가장 가까운 교차점까지의 거리를 반환 (교차 없으면 `max_range`)
- 출력: `std::vector<float> ranges` (기본 360개 값)
- Gaussian 노이즈 (sigma = 0.01m) 추가

### 3.4 시뮬레이션 루프

```
initialize_world(json_config)

while running:
    t_start = now()

    // 1. 명령 처리
    process_pending_commands()       // gRPC에서 수신한 속도 명령 큐 처리

    // 2. 로봇 업데이트
    for each robot in robots:
        robot.apply_acceleration(dt) // 가속도 제한 적용하여 속도 업데이트
        robot.integrate_pose(dt)     // kinematics로 위치 적분
        robot.update_battery(dt)     // 배터리 소모 계산

    // 3. 충돌 처리
    check_robot_wall_collisions()
    check_robot_robot_collisions()

    // 4. 센서 업데이트 (10Hz이므로 매 10번째 스텝)
    if sim_tick % sensor_interval == 0:
        for each robot:
            robot.update_lidar(world)

    // 5. 텔레메트리 전송 (10Hz)
    if sim_tick % telemetry_interval == 0:
        stream_telemetry_to_grpc()

    // 6. 실시간 동기화
    elapsed = now() - t_start
    if elapsed < dt:
        sleep(dt - elapsed)

    sim_tick++
```

### 3.5 gRPC Service (Phase 1 구현 RPC)

| RPC | 타입 | 설명 |
|-----|------|------|
| `CreateSimulation` | Unary | 월드 JSON 설정으로 시뮬 인스턴스 생성, sim_id 반환 |
| `StartSimulation` | Unary | 시뮬 루프 시작, 텔레메트리 스트림 활성화 |
| `StopSimulation` | Unary | 시뮬 루프 정지, 리소스 정리 |
| `SpawnRobot` | Unary | 시뮬에 로봇 추가 (위치, 설정 지정) |
| `RemoveRobot` | Unary | 시뮬에서 로봇 제거 |
| `SendCommand` | Unary | 특정 로봇에 속도 명령 전송 (linear_vel, angular_vel) |
| `StreamTelemetry` | Server streaming | 전체 로봇의 텔레메트리를 10Hz로 스트리밍 |

> 📋 gRPC 서비스 정의 상세: [integration-spec.md](../../integration/integration-spec.md#simulation-service) 참조
> 📋 TelemetryMessage 포맷: [integration-spec.md](../../integration/integration-spec.md#telemetry-proto) 참조

### 3.6 월드 정의 (Phase 1 JSON 포맷)

```json
{
  "world": {
    "name": "warehouse_simple",
    "size": [20.0, 15.0],
    "timestep_ms": 10,
    "walls": [
      {"from": [0, 0], "to": [20, 0]},
      {"from": [20, 0], "to": [20, 15]},
      {"from": [20, 15], "to": [0, 15]},
      {"from": [0, 15], "to": [0, 0]}
    ],
    "obstacles": [
      {"type": "box", "position": [5, 3], "size": [1, 2]},
      {"type": "circle", "position": [12, 8], "radius": 0.5}
    ],
    "spawn_points": [
      {"id": "sp1", "position": [2, 2], "heading": 0},
      {"id": "sp2", "position": [4, 2], "heading": 0}
    ]
  }
}
```

### 3.7 배터리 모델

- 선형 소모: 이동 중 `battery -= drain_rate * dt`, 대기 중 `battery -= idle_rate * dt`
- `battery <= 0` 시 로봇 정지 (속도 명령 무시), 상태를 `BATTERY_DEAD`로 변경
- Phase 1에서 충전 로직은 미구현 (Out-of-Scope)

---

## 4. 구현 모듈

| 파일 경로 | 클래스/함수 | 설명 |
|-----------|------------|------|
| `src/core/simulation.h/cpp` | `Simulation` | 시뮬 인스턴스 관리, 월드 초기화, 로봇 컬렉션 |
| `src/core/sim_loop.h/cpp` | `SimLoop` | 고정 타임스텝 루프 실행, 실시간 동기화 |
| `src/core/world.h/cpp` | `World` | 월드 상태 컨테이너 (벽, 장애물, 로봇 목록) |
| `src/core/clock.h/cpp` | `SimClock` | 시뮬레이션 시간 관리, 일시정지/배속 지원 |
| `src/core/config.h/cpp` | `SimConfig` | JSON 설정 파싱 및 검증 |
| `src/physics/custom_backend.h/cpp` | `CustomPhysicsBackend` | 2D kinematics, 가속도 제한 |
| `src/physics/collision.h/cpp` | `CollisionSystem` | AABB/원형 충돌 감지, MTV 응답 |
| `src/robots/robot.h/cpp` | `Robot` | 로봇 엔티티 (state + config + sensors) |
| `src/robots/differential_drive.h/cpp` | `DifferentialDrive` | 차동 구동 kinematics 계산 |
| `src/robots/robot_controller.h/cpp` | `RobotController` | 속도 명령 수신, 가속도 프로파일 적용 |
| `src/sensors/lidar_2d.h/cpp` | `Lidar2D` | 2D ray casting, 노이즈 모델 적용 |
| `src/sensors/noise_model.h/cpp` | `GaussianNoise` | Gaussian 노이즈 생성기 |
| `src/environment/json_loader.h/cpp` | `JsonWorldLoader` | JSON 월드 파일 파싱 → World 객체 생성 |
| `src/grpc/sim_server.h/cpp` | `SimGrpcServer` | gRPC 서비스 구현, RPC 핸들러 등록 |
| `src/grpc/telemetry_streamer.h/cpp` | `TelemetryStreamer` | 텔레메트리 수집 및 gRPC stream 전송 |
| `src/grpc/command_handler.h/cpp` | `CommandHandler` | gRPC 명령 수신 → 로봇 명령 큐 전달 |

---

## 5. 의존성

| 의존 대상 | 종류 | 상세 |
|-----------|------|------|
| Proto 스키마 | 빌드 의존 | `proto/simulation.proto`, `proto/telemetry.proto` 확정 필요 |
| gRPC C++ | 라이브러리 | `grpc/1.60+`, `protobuf/25.x+` |
| Eigen | 라이브러리 | 선형 대수 연산 (벡터, 행렬) |
| nlohmann/json | 라이브러리 | JSON 월드 파일 파싱 |
| spdlog | 라이브러리 | 구조화 로깅 |
| Google Test | 테스트 | 단위/통합 테스트 프레임워크 |

**참고:** Phase 1에서는 가장 독립적인 모듈이다. Asset Manager 로봇 모델 로드는 하드코딩으로 대체하며, 외부 팀 의존 없이 단독 개발/테스트가 가능하다.

---

## 6. 테스트 기준

### 6.1 단위 테스트

| 테스트 케이스 | 검증 조건 |
|-------------|----------|
| 차동 구동 직진 | v=1.0, omega=0 → 10초 후 위치 (10, 0), 오차 < 1cm |
| 차동 구동 회전 | v=0, omega=pi/2 → 2초 후 theta=pi, 오차 < 0.01rad |
| 가속도 제한 | 순간 속도 변경 시 가속도가 max_acceleration 이내 |
| 벽 충돌 | 로봇이 벽 방향으로 이동 시 벽을 통과하지 않음 |
| 로봇 간 충돌 | 두 로봇이 접근 시 겹치지 않음 |
| LiDAR 직선 벽 | 알려진 거리의 벽에 대해 측정 오차 < 1cm |
| LiDAR 코너 | L자 벽에서 레이캐스트 결과가 해석적 해와 일치 |
| 배터리 소모 | 이동 10초 → battery 감소량 = drain_rate * 10 |
| 배터리 방전 | battery=0 시 속도 명령이 무시됨 |

### 6.2 통합 테스트

| 테스트 케이스 | 검증 조건 |
|-------------|----------|
| E2E gRPC 흐름 | CreateSim → SpawnRobot → SendCommand → StreamTelemetry 정상 동작 |
| 10대 로봇 성능 | 10대 동시 시뮬, 실시간 비율 1.0 유지 (step time < 10ms) |
| JSON 월드 로딩 | 유효한 JSON → 벽/장애물 올바르게 생성 |
| 잘못된 JSON | 유효하지 않은 JSON → 명확한 에러 메시지 반환 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#sim-engine-test-data) 참조

---

## 7. 완료 조건 (Definition of Done)

- [ ] 커스텀 물리 엔진으로 차동 구동 로봇 이동 가능
- [ ] 가속도 제한이 적용된 속도 제어 동작
- [ ] AABB/원형 충돌 감지 및 응답 동작
- [ ] 2D LiDAR 레이캐스팅 동작 (360 rays, Gaussian 노이즈)
- [ ] JSON 월드 파일 로딩 및 환경 구성
- [ ] 고정 타임스텝 루프 (100Hz) 실시간 동기화
- [ ] gRPC 서버 기동 및 7개 RPC 동작
- [ ] gRPC 텔레메트리 스트리밍 (10Hz)
- [ ] 배터리 모델 (선형 소모, 방전 시 정지)
- [ ] 10대 로봇 동시 시뮬레이션 (실시간)
- [ ] 단위 테스트 통과 (coverage 80% 이상)
- [ ] Backend gRPC 클라이언트와 연동 확인
