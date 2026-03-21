# Team 3: Simulation Engine - 상세 개발 명세서

> **문서 버전**: 1.0
> **최종 수정일**: 2026-03-21
> **대상 독자**: AI 에이전트 개발 스웜 (본 문서만으로 전체 시뮬레이션 엔진 구축 가능)

---

## 1. 팀 개요

### 1.1 팀 명칭 및 담당 범위

- **팀 명칭**: Team 3 - Simulation Engine (Sim Engine)
- **담당 범위**: AMR(Autonomous Mobile Robot) 통합 프레임워크의 물리 시뮬레이션 엔진 전체를 설계, 구현, 테스트, 배포한다. 시뮬레이션 엔진은 물리 기반 로봇 동작, 센서 시뮬레이션, 환경 구성, 텔레메트리 생성을 포함하며, gRPC를 통해 백엔드 시스템과 통신한다.

### 1.2 핵심 목표

1. **Mode-Agnostic 텔레메트리**: 실제 로봇이 생성하는 텔레메트리와 **완전히 동일한 형식과 의미**의 데이터를 생성한다. 백엔드, 프론트엔드, 네비게이션 스택 등 모든 상위 시스템이 실제 로봇과 시뮬레이션 로봇을 구분할 수 없어야 한다.
2. **물리적 정확성**: 충돌, 마찰, 관성, 중력 등 물리 법칙을 준수하며, 로봇의 kinematic/dynamic 모델이 실제 하드웨어 사양과 일치해야 한다.
3. **고성능**: 10대의 로봇을 200Hz 이상으로 시뮬레이션하며, 센서 데이터(LiDAR, 카메라, IMU 등)를 실시간으로 생성한다.
4. **확장성**: 새로운 로봇 모델, 센서 타입, 환경 요소를 플러그인 방식으로 추가 가능해야 한다.
5. **스크립팅 지원**: Python 바인딩을 통해 시나리오 작성, 자동 테스트, 학습 환경 구축이 가능해야 한다.

### 1.3 의존 관계

| 의존 대상 | 방향 | 설명 |
|-----------|------|------|
| **Backend Team** | Sim Engine → Backend | Sim Engine이 gRPC **서버**를 제공하고, Backend가 **클라이언트**로 연결하여 시뮬레이션을 제어하고 텔레메트리를 수신한다. |
| **Asset Manager Team** | Sim Engine → Asset Manager | 로봇 모델(OpenUSD), 환경 객체 메시를 Asset Manager의 gRPC API 또는 로컬 파일 경로를 통해 로드한다. URDF/SDF는 Asset Manager에서 USD로 변환하여 제공한다. |
| **Map Manager Team** | Sim Engine → Map Manager | 맵 데이터(포인트 클라우드, 2D 그리드맵, 로드맵)를 로드하여 시뮬레이션 환경을 구성한다. |
| **Proto Team** | 공유 | `proto/` 디렉토리의 공유 protobuf 정의를 사용한다. Sim Engine 전용 proto도 이 디렉토리에 추가한다. |

### 1.4 아키텍처 위치

```
┌──────────────┐     gRPC      ┌──────────────┐
│   Backend    │◄──────────────│  Sim Engine   │
│   (Client)   │  Telemetry    │   (Server)    │
└──────────────┘  + Commands   └───────┬───────┘
                                       │
                    ┌──────────────────┼──────────────────┐
                    │                  │                  │
              ┌─────▼─────┐    ┌──────▼──────┐    ┌─────▼─────┐
              │  Physics   │    │   Sensors   │    │Environment│
              │  Backend   │    │ Simulation  │    │  Loader   │
              │(Pluggable) │    │(Vulkan+Ray) │    │(OpenUSD)  │
              └────────────┘    └─────────────┘    └────────────┘
```

---

## 2. 기술 스택 상세

### 2.1 언어 및 컴파일러

| 항목 | 사양 | 비고 |
|------|------|------|
| **언어** | C++20 | C++17 호환 유지, C++20 기능(concepts, ranges, coroutines) 선택적 사용 |
| **컴파일러** | GCC 13+ 또는 Clang 17+ | `-std=c++20 -Wall -Wextra -Wpedantic -O2` (릴리스), `-O0 -g -fsanitize=address,undefined` (디버그) |
| **Python** | 3.11+ | pybind11 바인딩 전용 |

### 2.2 빌드 시스템

| 항목 | 버전 | 용도 |
|------|------|------|
| **CMake** | 3.28+ | 빌드 구성, 타겟 정의, 의존성 연결 |
| **Conan** | 2.x | 외부 라이브러리 패키지 관리 |
| **Ninja** | 최신 | 빌드 백엔드 (CMake -G Ninja) |

### 2.3 핵심 라이브러리

| 라이브러리 | 버전 | 용도 | Conan 패키지 |
|-----------|------|------|-------------|
| **물리 엔진** | 플러그인 아키텍처 | 기본: 커스텀 경량 엔진(주행 특화), 플러그인: MuJoCo 3.x(고정밀), Isaac Sim(포토리얼리스틱) | 커스텀(내장), `mujoco/3.x`(선택) |
| **OpenUSD SDK** | 24.x+ | 씬 포맷 (.usd/.usda/.usdc), 계층적 씬 합성, 물리 스키마 내장 | `usd-core` (시스템 설치 또는 소스 빌드) |
| **Vulkan SDK** | 1.3+ | 헤드리스 렌더링 (카메라 센서) | 시스템 설치 |
| **gRPC C++** | 1.60+ | 서비스 인터페이스 | `grpc/1.60.0` |
| **Protobuf** | 25.x+ | 메시지 직렬화 | `protobuf/25.0` (gRPC 종속) |
| **pybind11** | 2.12+ | Python 바인딩 | `pybind11/2.12.0` |
| **libsdformat** | 14.x | SDF/URDF 임포트 변환 (USD로 변환) | 소스 빌드 또는 시스템 설치 |
| **spdlog** | 1.13+ | 구조화 로깅 | `spdlog/1.13.0` |
| **Google Test** | 1.14+ | 단위/통합 테스트 | `gtest/1.14.0` |
| **Google Benchmark** | 1.8+ | 성능 벤치마크 | `benchmark/1.8.3` |
| **Eigen** | 3.4+ | 선형 대수 연산 | `eigen/3.4.0` |
| **fmt** | 10.x | 문자열 포맷 (spdlog 종속) | `fmt/10.2.0` |
| **GLFW** | 3.3+ | 디버그 뷰포트 (선택) | `glfw/3.3.9` |

### 2.4 개발 도구

| 도구 | 용도 |
|------|------|
| **clang-format** | 코드 포맷 (`.clang-format` 파일 제공) |
| **clang-tidy** | 정적 분석 |
| **cppcheck** | 추가 정적 분석 |
| **valgrind / ASan** | 메모리 검사 |
| **perf / Tracy** | 성능 프로파일링 |
| **Docker** | 빌드 및 실행 컨테이너화 |

---

## 3. 디렉토리 구조 상세

```
sim-engine/
├── src/
│   ├── main.cpp                        # Entry point - gRPC 서버 시작 및 시뮬레이션 인스턴스 관리
│   ├── core/
│   │   ├── simulation.h                # Main simulation class 선언
│   │   ├── simulation.cpp              # Main simulation class 구현
│   │   ├── sim_loop.h                  # Fixed-timestep simulation loop 선언
│   │   ├── sim_loop.cpp                # Fixed-timestep simulation loop 구현
│   │   ├── world.h                     # World state container 선언
│   │   ├── world.cpp                   # World state container 구현
│   │   ├── entity.h                    # Entity base class 선언
│   │   ├── entity.cpp                  # Entity base class 구현
│   │   ├── entity_manager.h            # Entity lifecycle management 선언
│   │   ├── entity_manager.cpp          # Entity lifecycle management 구현
│   │   ├── clock.h                     # Simulation clock 선언
│   │   ├── clock.cpp                   # Simulation clock 구현
│   │   ├── config.h                    # Configuration structs 선언
│   │   └── config.cpp                  # Configuration 로딩/검증
│   ├── physics/
│   │   ├── physics_backend.h           # PhysicsBackend 추상 인터페이스 (플러그인)
│   │   ├── custom_backend.h            # 커스텀 경량 엔진 선언 (2D 주행 특화)
│   │   ├── custom_backend.cpp          # 커스텀 경량 엔진 구현
│   │   ├── mujoco_backend.h            # MuJoCo 백엔드 선언 (고정밀 3D)
│   │   ├── mujoco_backend.cpp          # MuJoCo 백엔드 구현
│   │   ├── isaac_bridge.h              # Isaac Sim 브릿지 선언 (GPU 가속)
│   │   ├── isaac_bridge.cpp            # Isaac Sim 브릿지 구현 (gRPC 원격)
│   │   ├── backend_factory.h           # 백엔드 팩토리 (설정 기반 생성)
│   │   ├── backend_factory.cpp         # 백엔드 팩토리 구현
│   │   ├── collision.h                 # 충돌 감지/응답 선언
│   │   ├── collision.cpp               # 충돌 감지/응답 구현
│   │   ├── rigid_body.h                # Rigid body wrapper 선언
│   │   ├── rigid_body.cpp              # Rigid body wrapper 구현
│   │   ├── constraints.h               # Joint constraints 선언
│   │   └── constraints.cpp             # Joint constraints 구현
│   ├── robots/
│   │   ├── robot.h                     # Robot entity 선언
│   │   ├── robot.cpp                   # Robot entity 구현
│   │   ├── differential_drive.h        # Differential drive 선언
│   │   ├── differential_drive.cpp      # Differential drive 구현
│   │   ├── mecanum_drive.h             # Mecanum drive 선언
│   │   ├── mecanum_drive.cpp           # Mecanum drive 구현
│   │   ├── robot_controller.h          # 속도 명령 처리 선언
│   │   ├── robot_controller.cpp        # 속도 명령 처리 구현
│   │   ├── robot_state.h               # Robot state 선언
│   │   └── robot_state.cpp             # Robot state 구현
│   ├── sensors/
│   │   ├── sensor.h                    # Sensor base class
│   │   ├── sensor.cpp                  # Sensor base class 구현
│   │   ├── lidar_2d.h                  # 2D LiDAR 선언
│   │   ├── lidar_2d.cpp                # 2D LiDAR 구현 (ray casting)
│   │   ├── lidar_3d.h                  # 3D LiDAR 선언
│   │   ├── lidar_3d.cpp                # 3D LiDAR 구현
│   │   ├── camera_rgb.h                # RGB 카메라 선언
│   │   ├── camera_rgb.cpp              # RGB 카메라 구현 (Vulkan)
│   │   ├── camera_depth.h              # Depth 카메라 선언
│   │   ├── camera_depth.cpp            # Depth 카메라 구현
│   │   ├── imu.h                       # IMU 선언
│   │   ├── imu.cpp                     # IMU 구현
│   │   ├── encoder.h                   # Wheel encoder 선언
│   │   ├── encoder.cpp                 # Wheel encoder 구현
│   │   ├── noise_model.h               # 노이즈 모델 선언
│   │   └── noise_model.cpp             # 노이즈 모델 구현
│   ├── environment/
│   │   ├── scene_loader.h              # USD 씬 로더 선언
│   │   ├── scene_loader.cpp            # USD 씬 로더 구현
│   │   ├── usd_loader.h               # OpenUSD 파싱 선언 (.usd/.usda/.usdc)
│   │   ├── usd_loader.cpp             # OpenUSD 파싱 구현
│   │   ├── sdf_importer.h             # SDF/URDF → USD 변환 임포터 선언
│   │   ├── sdf_importer.cpp           # SDF/URDF → USD 변환 임포터 구현
│   │   ├── static_object.h             # 정적 객체 선언
│   │   ├── static_object.cpp           # 정적 객체 구현
│   │   ├── dynamic_object.h            # 동적 객체 선언
│   │   ├── dynamic_object.cpp          # 동적 객체 구현
│   │   ├── ground_plane.h              # 지면 선언
│   │   └── ground_plane.cpp            # 지면 구현
│   ├── rendering/
│   │   ├── vulkan_context.h            # Vulkan 초기화 선언
│   │   ├── vulkan_context.cpp          # Vulkan 초기화 구현
│   │   ├── render_pipeline.h           # 렌더 파이프라인 선언
│   │   ├── render_pipeline.cpp         # 렌더 파이프라인 구현
│   │   ├── framebuffer.h               # Offscreen framebuffer 선언
│   │   ├── framebuffer.cpp             # Offscreen framebuffer 구현
│   │   ├── viewport.h                  # 디버그 뷰포트 선언
│   │   └── viewport.cpp               # 디버그 뷰포트 구현
│   ├── grpc/
│   │   ├── sim_server.h                # gRPC 서버 선언
│   │   ├── sim_server.cpp              # gRPC 서버 구현
│   │   ├── telemetry_streamer.h        # 텔레메트리 스트림 선언
│   │   ├── telemetry_streamer.cpp      # 텔레메트리 스트림 구현
│   │   ├── command_handler.h           # 명령 처리 선언
│   │   └── command_handler.cpp         # 명령 처리 구현
│   └── python/
│       ├── bindings.cpp                # pybind11 모듈 정의
│       ├── py_simulation.cpp           # Python Simulation 래퍼
│       └── py_scenario.cpp             # 시나리오 헬퍼
├── include/
│   └── sim_engine/                     # Public headers (외부 노출용)
│       ├── simulation.h                # → src/core/simulation.h 복사 또는 전달 헤더
│       ├── types.h                     # 공용 타입 정의
│       └── version.h                   # 버전 매크로
├── proto/                              # Protobuf 정의 (루트 proto/ 심링크 또는 복사)
│   ├── simulation.proto                # SimulationService 정의
│   ├── telemetry.proto                 # 텔레메트리 메시지 정의
│   ├── robot.proto                     # 로봇 관련 메시지
│   ├── sensor.proto                    # 센서 데이터 메시지
│   └── common.proto                    # 공용 메시지 (Vector3, Quaternion 등)
├── tests/
│   ├── unit/
│   │   ├── test_sim_loop.cpp           # 시뮬레이션 루프 테스트
│   │   ├── test_differential_drive.cpp # 차동 구동 운동학 테스트
│   │   ├── test_mecanum_drive.cpp      # 메카넘 구동 운동학 테스트
│   │   ├── test_lidar_2d.cpp           # 2D LiDAR 테스트
│   │   ├── test_lidar_3d.cpp           # 3D LiDAR 테스트
│   │   ├── test_imu.cpp               # IMU 테스트
│   │   ├── test_encoder.cpp            # 엔코더 테스트
│   │   ├── test_collision.cpp          # 충돌 감지 테스트
│   │   ├── test_noise_model.cpp        # 노이즈 모델 테스트
│   │   ├── test_clock.cpp              # 시뮬레이션 시계 테스트
│   │   └── test_entity_manager.cpp     # 엔티티 매니저 테스트
│   ├── integration/
│   │   ├── test_grpc_server.cpp        # gRPC 서버 통합 테스트
│   │   ├── test_full_scenario.cpp      # 전체 시나리오 통합 테스트
│   │   └── test_python_bindings.py     # Python 바인딩 테스트
│   └── benchmarks/
│       ├── bench_physics.cpp           # 물리 엔진 벤치마크
│       ├── bench_lidar.cpp             # LiDAR 성능 벤치마크
│       └── bench_full_loop.cpp         # 전체 루프 벤치마크
├── scenarios/                          # Python 시나리오 스크립트
│   ├── warehouse_basic.py              # 기본 창고 시나리오
│   ├── multi_robot_traffic.py          # 다중 로봇 교통 시나리오
│   └── sensor_validation.py            # 센서 검증 시나리오
├── models/                             # 내장 테스트 모델
│   ├── simple_robot.usda               # 단순 로봇 모델 (OpenUSD)
│   ├── warehouse.usda                  # 창고 환경 모델 (OpenUSD)
│   └── box_obstacle.usda              # 박스 장애물 모델 (OpenUSD)
├── shaders/                            # Vulkan 셰이더
│   ├── mesh.vert                       # 메시 버텍스 셰이더
│   ├── mesh.frag                       # 메시 프래그먼트 셰이더
│   ├── depth.vert                      # 뎁스 버텍스 셰이더
│   └── depth.frag                      # 뎁스 프래그먼트 셰이더
├── config/
│   └── default_config.yaml             # 기본 설정 파일
├── CMakeLists.txt                      # 루트 CMake 설정
├── conanfile.py                        # Conan 패키지 설정
├── Dockerfile                          # Docker 빌드/실행 설정
├── .clang-format                       # 코드 포맷 설정
├── .clang-tidy                         # 정적 분석 설정
└── README.md                           # 프로젝트 개요
```

---

## 4. 구성 모듈 상세 스펙

### 4.1 Core - Simulation Loop

#### 4.1.1 Fixed Timestep 설계

시뮬레이션은 **고정 타임스텝(fixed timestep)** 방식으로 동작한다. 기본 물리 타임스텝은 **1ms(1000Hz)**이며, 설정을 통해 변경 가능하다. 고정 타임스텝은 물리 시뮬레이션의 결정론적 재현성을 보장한다.

```cpp
// src/core/config.h
#pragma once

#include <cstdint>
#include <string>
#include <vector>

namespace amr::sim {

struct PhysicsConfig {
    double fixed_timestep = 0.001;      // 1ms = 1000Hz
    std::string backend = "custom";     // "custom" | "mujoco" | "isaac_sim"
    double gravity_x = 0.0;
    double gravity_y = 0.0;
    double gravity_z = -9.81;
    double default_friction = 0.8;
    double default_restitution = 0.1;
    // 커스텀 백엔드 설정
    std::string collision_mode = "2d";  // "2d" | "2.5d"
    int max_robots = 200;
    // MuJoCo 백엔드 설정
    int mujoco_solver_iterations = 50;
    // Isaac Sim 백엔드 설정
    std::string isaac_endpoint = "localhost:50055";
    bool isaac_use_gpu_physics = true;
    bool isaac_use_rtx_lidar = true;
};

struct SimulationConfig {
    PhysicsConfig physics;
    double realtime_factor = 1.0;       // 1.0 = 실시간, 0.0 = 최대 속도, 2.0 = 2배속
    uint16_t grpc_port = 50051;
    std::string log_level = "info";     // trace, debug, info, warn, error, critical
    bool enable_rendering = false;      // Vulkan 헤드리스 렌더링 활성화
    bool enable_debug_viewport = false; // GLFW 디버그 뷰포트
    uint32_t sensor_thread_count = 2;   // 센서 업데이트 스레드 수
};

struct RobotConfig {
    std::string model_path;             // USD 파일 경로 (.usd/.usda/.usdc)
    std::string robot_id;               // 고유 ID (비어있으면 자동 생성)
    double initial_x = 0.0;
    double initial_y = 0.0;
    double initial_z = 0.0;
    double initial_yaw = 0.0;           // Z축 회전 (라디안)
    double initial_battery = 100.0;     // 배터리 퍼센트
    std::string drive_type = "differential"; // "differential" 또는 "mecanum"
};

} // namespace amr::sim
```

#### 4.1.2 시뮬레이션 루프 구조

매 스텝은 다음 6단계를 순차적으로 실행한다:

```
┌─────────────────────────────────────────────────────┐
│              Simulation Loop (1 Step)                │
│                                                     │
│  1. Process Incoming Commands (gRPC 큐에서 읽기)    │
│     └─ VelocityCommand, SpawnRequest 등             │
│  2. Update Robot Controllers                         │
│     └─ 명령 → 휠 속도 → 물리 엔진에 힘/속도 적용   │
│  3. Step Physics Backend                             │
│     └─ backend->step(dt) (Custom/MuJoCo/Isaac)      │
│  4. Update Sensors                                   │
│     └─ LiDAR ray cast, IMU 계산, 카메라 렌더링      │
│  5. Collect Telemetry                                │
│     └─ 로봇 상태, 센서 데이터를 메시지로 변환       │
│  6. Stream Telemetry (gRPC)                          │
│     └─ 구독 중인 클라이언트에 스트리밍               │
│  7. Sleep to Maintain Realtime Factor                │
│     └─ wall_clock 기준으로 대기 시간 계산           │
└─────────────────────────────────────────────────────┘
```

```cpp
// src/core/sim_loop.h
#pragma once

#include "core/config.h"
#include "core/world.h"
#include "core/clock.h"
#include "physics/physics_engine.h"
#include "grpc/command_handler.h"
#include "grpc/telemetry_streamer.h"

#include <atomic>
#include <functional>
#include <thread>

namespace amr::sim {

class SimLoop {
public:
    explicit SimLoop(const SimulationConfig& config);
    ~SimLoop();

    // 시뮬레이션 시작 (별도 스레드에서 루프 실행)
    void start();

    // 시뮬레이션 정지
    void stop();

    // 단일 스텝 실행 (Python 바인딩용)
    void stepOnce();

    // 리얼타임 팩터 설정
    void setRealtimeFactor(double factor);

    // 콜백 등록
    using PreStepCallback = std::function<void(double dt)>;
    using PostStepCallback = std::function<void(double dt, const WorldState& state)>;
    void setPreStepCallback(PreStepCallback cb);
    void setPostStepCallback(PostStepCallback cb);

    // 상태 조회
    bool isRunning() const { return running_.load(); }
    uint64_t stepCount() const { return step_count_.load(); }
    double simTime() const;

private:
    void loopThread();
    void executeStep(double dt);

    SimulationConfig config_;
    std::unique_ptr<World> world_;
    std::unique_ptr<SimClock> clock_;
    std::unique_ptr<PhysicsEngine> physics_;
    CommandHandler* command_handler_ = nullptr;      // 외부 주입
    TelemetryStreamer* telemetry_streamer_ = nullptr; // 외부 주입

    std::thread loop_thread_;
    std::atomic<bool> running_{false};
    std::atomic<uint64_t> step_count_{0};
    std::atomic<double> realtime_factor_{1.0};

    PreStepCallback pre_step_cb_;
    PostStepCallback post_step_cb_;
};

} // namespace amr::sim
```

```cpp
// src/core/sim_loop.cpp
#include "core/sim_loop.h"
#include <spdlog/spdlog.h>
#include <chrono>

namespace amr::sim {

SimLoop::SimLoop(const SimulationConfig& config)
    : config_(config)
    , clock_(std::make_unique<SimClock>())
    , realtime_factor_(config.realtime_factor) {
    // 물리 백엔드 초기화 (설정에 따라 custom/mujoco/isaac_sim 선택)
    physics_ = PhysicsEngine::create(config.physics.backend, config.physics);
    world_ = std::make_unique<World>();
}

SimLoop::~SimLoop() {
    stop();
}

void SimLoop::start() {
    if (running_.load()) return;
    running_ = true;
    clock_->reset();
    loop_thread_ = std::thread(&SimLoop::loopThread, this);
    spdlog::info("Simulation loop started (dt={:.4f}s, rtf={:.1f})",
                 config_.physics.fixed_timestep, realtime_factor_.load());
}

void SimLoop::stop() {
    running_ = false;
    if (loop_thread_.joinable()) {
        loop_thread_.join();
    }
    spdlog::info("Simulation loop stopped after {} steps", step_count_.load());
}

void SimLoop::stepOnce() {
    executeStep(config_.physics.fixed_timestep);
}

void SimLoop::setRealtimeFactor(double factor) {
    realtime_factor_ = factor;
    spdlog::info("Realtime factor set to {:.2f}", factor);
}

double SimLoop::simTime() const {
    return clock_->simTime();
}

void SimLoop::loopThread() {
    const double dt = config_.physics.fixed_timestep;

    while (running_.load()) {
        auto wall_start = std::chrono::steady_clock::now();

        executeStep(dt);

        // 리얼타임 팩터에 따른 대기
        double rtf = realtime_factor_.load();
        if (rtf > 0.0) {
            auto wall_end = std::chrono::steady_clock::now();
            double elapsed_wall = std::chrono::duration<double>(wall_end - wall_start).count();
            double target_wall = dt / rtf;
            double sleep_time = target_wall - elapsed_wall;

            if (sleep_time > 0.0) {
                std::this_thread::sleep_for(
                    std::chrono::duration<double>(sleep_time));
            } else {
                // 리얼타임을 유지하지 못하는 경우 경고
                spdlog::warn("Simulation running slower than realtime: "
                            "step took {:.4f}s, target {:.4f}s",
                            elapsed_wall, target_wall);
            }
        }
        // rtf == 0.0: 대기 없이 최대 속도로 실행
    }
}

void SimLoop::executeStep(double dt) {
    // Step 1: 수신된 명령 처리
    if (command_handler_) {
        command_handler_->processQueuedCommands(*world_);
    }

    // Pre-step 콜백
    if (pre_step_cb_) pre_step_cb_(dt);

    // Step 2: 로봇 컨트롤러 업데이트
    for (auto& [id, robot] : world_->robots()) {
        robot->updateController(dt);
    }

    // Step 3: 물리 엔진 스텝
    physics_->step(dt);

    // Step 4: 센서 업데이트
    for (auto& [id, robot] : world_->robots()) {
        robot->updateSensors(dt, *physics_);
    }

    // Step 5: 텔레메트리 수집
    auto telemetry = world_->collectTelemetry();

    // Step 6: 텔레메트리 스트리밍
    if (telemetry_streamer_) {
        telemetry_streamer_->broadcast(telemetry);
    }

    // 시뮬레이션 시계 및 카운터 업데이트
    clock_->advance(dt);
    step_count_.fetch_add(1);

    // Post-step 콜백
    if (post_step_cb_) {
        post_step_cb_(dt, world_->getState());
    }
}

} // namespace amr::sim
```

#### 4.1.3 Simulation Clock

```cpp
// src/core/clock.h
#pragma once

#include <chrono>
#include <cstdint>

namespace amr::sim {

class SimClock {
public:
    SimClock() = default;

    void reset();
    void advance(double dt);

    // 시뮬레이션 시간 (초 단위)
    double simTime() const { return sim_time_; }

    // 시뮬레이션 시간 (나노초 단위, protobuf timestamp용)
    uint64_t simTimeNs() const {
        return static_cast<uint64_t>(sim_time_ * 1e9);
    }

    // 스텝 카운트
    uint64_t stepCount() const { return step_count_; }

    // 실제 벽시계 경과 시간
    double wallTime() const;

private:
    double sim_time_ = 0.0;
    uint64_t step_count_ = 0;
    std::chrono::steady_clock::time_point wall_start_;
};

} // namespace amr::sim
```

```cpp
// src/core/clock.cpp
#include "core/clock.h"

namespace amr::sim {

void SimClock::reset() {
    sim_time_ = 0.0;
    step_count_ = 0;
    wall_start_ = std::chrono::steady_clock::now();
}

void SimClock::advance(double dt) {
    sim_time_ += dt;
    step_count_++;
}

double SimClock::wallTime() const {
    auto now = std::chrono::steady_clock::now();
    return std::chrono::duration<double>(now - wall_start_).count();
}

} // namespace amr::sim
```

#### 4.1.4 Entity 시스템

Entity-Component 패턴을 사용하여 월드 내 모든 객체를 관리한다.

```cpp
// src/core/entity.h
#pragma once

#include <string>
#include <memory>
#include <unordered_map>
#include <typeindex>
#include <Eigen/Geometry>

namespace amr::sim {

// 모든 컴포넌트의 기반 클래스
struct Component {
    virtual ~Component() = default;
};

// 3D 위치 및 방향
struct TransformComponent : public Component {
    Eigen::Vector3d position{0, 0, 0};
    Eigen::Quaterniond orientation{1, 0, 0, 0}; // w, x, y, z
    Eigen::Vector3d linear_velocity{0, 0, 0};
    Eigen::Vector3d angular_velocity{0, 0, 0};

    // 편의 함수
    double yaw() const;
    void setYaw(double yaw);
    Eigen::Isometry3d pose() const;
};

// 엔티티: 고유 ID와 컴포넌트 집합
class Entity {
public:
    explicit Entity(std::string id);
    virtual ~Entity() = default;

    const std::string& id() const { return id_; }

    template<typename T, typename... Args>
    T& addComponent(Args&&... args) {
        auto comp = std::make_unique<T>(std::forward<Args>(args)...);
        T& ref = *comp;
        components_[std::type_index(typeid(T))] = std::move(comp);
        return ref;
    }

    template<typename T>
    T* getComponent() {
        auto it = components_.find(std::type_index(typeid(T)));
        if (it == components_.end()) return nullptr;
        return static_cast<T*>(it->second.get());
    }

    template<typename T>
    const T* getComponent() const {
        auto it = components_.find(std::type_index(typeid(T)));
        if (it == components_.end()) return nullptr;
        return static_cast<const T*>(it->second.get());
    }

    template<typename T>
    bool hasComponent() const {
        return components_.count(std::type_index(typeid(T))) > 0;
    }

private:
    std::string id_;
    std::unordered_map<std::type_index, std::unique_ptr<Component>> components_;
};

} // namespace amr::sim
```

```cpp
// src/core/entity_manager.h
#pragma once

#include "core/entity.h"
#include <functional>
#include <mutex>
#include <vector>

namespace amr::sim {

class EntityManager {
public:
    EntityManager() = default;

    // 엔티티 생성 및 등록
    Entity& createEntity(const std::string& id);

    // 엔티티 제거
    bool removeEntity(const std::string& id);

    // 엔티티 조회
    Entity* getEntity(const std::string& id);
    const Entity* getEntity(const std::string& id) const;

    // 특정 컴포넌트를 가진 엔티티 순회
    template<typename T>
    void forEach(std::function<void(Entity&, T&)> fn) {
        std::lock_guard<std::mutex> lock(mutex_);
        for (auto& [id, entity] : entities_) {
            if (auto* comp = entity->getComponent<T>()) {
                fn(*entity, *comp);
            }
        }
    }

    // 전체 엔티티 수
    size_t count() const;

    // 모든 엔티티 제거
    void clear();

private:
    std::unordered_map<std::string, std::unique_ptr<Entity>> entities_;
    mutable std::mutex mutex_;
};

} // namespace amr::sim
```

#### 4.1.5 World State Container

```cpp
// src/core/world.h
#pragma once

#include "core/entity_manager.h"
#include "robots/robot.h"
#include "environment/static_object.h"
#include "environment/dynamic_object.h"

#include <unordered_map>
#include <memory>
#include <vector>

namespace amr::sim {

// 월드 전체 상태 스냅샷
struct WorldState {
    double sim_time;
    uint64_t step_count;
    size_t robot_count;
    size_t object_count;
    std::vector<RobotState> robot_states;
};

// 텔레메트리 패킷 (스트리밍용)
struct TelemetryPacket {
    double timestamp;
    std::string robot_id;
    RobotState robot_state;
    std::vector<SensorData> sensor_data;
};

class World {
public:
    World() = default;

    // 로봇 관리
    Robot& addRobot(std::unique_ptr<Robot> robot);
    bool removeRobot(const std::string& robot_id);
    Robot* getRobot(const std::string& robot_id);
    const std::unordered_map<std::string, std::unique_ptr<Robot>>& robots() const {
        return robots_;
    }
    std::unordered_map<std::string, std::unique_ptr<Robot>>& robots() {
        return robots_;
    }

    // 환경 객체 관리
    void addStaticObject(std::unique_ptr<StaticObject> obj);
    void addDynamicObject(std::unique_ptr<DynamicObject> obj);
    bool removeObject(const std::string& object_id);

    // 상태 조회
    WorldState getState() const;

    // 텔레메트리 수집
    std::vector<TelemetryPacket> collectTelemetry() const;

    // 전체 리셋
    void reset();

private:
    EntityManager entity_manager_;
    std::unordered_map<std::string, std::unique_ptr<Robot>> robots_;
    std::unordered_map<std::string, std::unique_ptr<StaticObject>> static_objects_;
    std::unordered_map<std::string, std::unique_ptr<DynamicObject>> dynamic_objects_;
};

} // namespace amr::sim
```

#### 4.1.6 스레드 모델

시뮬레이션 엔진은 다음 3개의 주요 스레드를 사용한다:

| 스레드 | 역할 | 우선순위 |
|--------|------|----------|
| **Main Sim Thread** | 시뮬레이션 루프 실행 (물리 + 센서 + 텔레메트리) | 높음 (SCHED_FIFO 권장) |
| **gRPC Server Thread** | gRPC 요청 수신, 명령 큐에 삽입, 텔레메트리 스트림 관리 | 보통 |
| **Render Thread** (선택) | Vulkan 헤드리스 렌더링 (카메라 센서) | 보통 |

스레드 간 통신은 **lock-free 큐** (명령 전달) 및 **mutex** (상태 조회)를 사용한다.

```cpp
// 스레드 간 통신을 위한 lock-free 명령 큐
// src/grpc/command_handler.h
#pragma once

#include "core/world.h"
#include <queue>
#include <mutex>
#include <variant>

namespace amr::sim {

struct VelocityCommand {
    std::string robot_id;
    double linear_x = 0.0;   // 전진/후진 속도 (m/s)
    double linear_y = 0.0;   // 좌/우 속도 (m/s, mecanum만 해당)
    double angular_z = 0.0;  // 회전 속도 (rad/s)
};

struct SpawnRobotCommand {
    RobotConfig config;
};

struct RemoveRobotCommand {
    std::string robot_id;
};

struct SpawnObjectCommand {
    std::string model_path;
    std::string object_id;
    double x, y, z;
    double roll, pitch, yaw;
    bool is_static;
};

struct RemoveObjectCommand {
    std::string object_id;
};

using SimCommand = std::variant<
    VelocityCommand,
    SpawnRobotCommand,
    RemoveRobotCommand,
    SpawnObjectCommand,
    RemoveObjectCommand
>;

class CommandHandler {
public:
    // gRPC 스레드에서 호출: 명령을 큐에 삽입
    void enqueue(SimCommand cmd);

    // Sim 스레드에서 호출: 큐의 모든 명령을 처리
    void processQueuedCommands(World& world);

private:
    std::queue<SimCommand> queue_;
    std::mutex mutex_;
};

} // namespace amr::sim
```

---

### 4.2 Physics Engine

#### 4.2.1 플러그인 백엔드 아키텍처

물리 엔진은 **플러그인 백엔드 아키텍처**로 설계한다. 기본은 AMR 주행/네비게이션에 특화된 커스텀 경량 엔진이며, MuJoCo(고정밀 3D)와 Isaac Sim(포토리얼리스틱)을 플러그인으로 지원한다.

- **기본: 커스텀 경량 엔진** — AMR 주행/네비게이션에 특화:
  - 2D 운동학 (differential drive, mecanum drive)
  - 단순 2D/2.5D 충돌 감지 (AABB, circle-circle, polygon)
  - 바퀴-지면 접촉 모델 (단순화된 마찰)
  - 배터리 소모 모델
  - 전체 3D 강체 역학 없음 — 속도와 단순함에 집중
  - 목표: 단일 코어에서 100+ 로봇을 실시간 속도로 시뮬레이션
  - 구현: 커스텀 C++ 코드, 외부 물리 라이브러리 의존성 없음

- **플러그인: MuJoCo 백엔드** — 고정밀 물리:
  - 전체 3D 강체 역학, 접촉 물리
  - 매니퓰레이션 작업, 정밀 충돌 응답에 유용
  - 공유 라이브러리(.so/.dylib)를 통해 동적 로드

- **플러그인: NVIDIA Isaac Sim 백엔드** — 포토리얼리스틱 시뮬레이션:
  - GPU 가속 물리 (PhysX)
  - 포토리얼리스틱 렌더링 (RTX)
  - 고급 센서 시뮬레이션 (ray-traced lidar, synthetic camera)
  - Isaac Sim의 Python API 또는 gRPC로 연결
  - 별도 프로세스로 실행, gRPC 통신

**PhysicsBackend 인터페이스**:
```cpp
class PhysicsBackend {
public:
    virtual ~PhysicsBackend() = default;
    virtual void init(const PhysicsConfig& config) = 0;
    virtual void shutdown() = 0;
    virtual void step(double dt) = 0;
    virtual BodyHandle addBody(const BodyDesc& desc) = 0;
    virtual void removeBody(BodyHandle handle) = 0;
    virtual void setBodyPose(BodyHandle handle, const Pose3D& pose) = 0;
    virtual Pose3D getBodyPose(BodyHandle handle) const = 0;
    virtual void setBodyVelocity(BodyHandle handle, const Twist& vel) = 0;
    virtual Twist getBodyVelocity(BodyHandle handle) const = 0;
    virtual std::vector<RaycastResult> batchRaycast(const std::vector<Ray>& rays) = 0;
    virtual std::vector<ContactInfo> getContacts() const = 0;
    virtual std::string getName() const = 0;
};

class CustomLightweightBackend : public PhysicsBackend { /* 2D 중심, 고속 */ };
class MuJoCoBackend : public PhysicsBackend { /* 전체 3D, 고정밀 */ };
class IsaacSimBridge : public PhysicsBackend { /* 원격, GPU 가속 */ };
```

```cpp
// src/physics/physics_engine.h
#pragma once

#include "core/config.h"
#include "physics/rigid_body.h"
#include "physics/constraints.h"

#include <Eigen/Core>
#include <memory>
#include <string>
#include <vector>

namespace amr::sim {

struct Ray {
    Eigen::Vector3d from;
    Eigen::Vector3d to;
};

struct RaycastResult {
    bool hit = false;
    Eigen::Vector3d hit_point{0, 0, 0};
    Eigen::Vector3d hit_normal{0, 0, 0};
    double hit_fraction = 1.0;  // 0.0 = from, 1.0 = to
    double distance = 0.0;
    std::string hit_entity_id;
};

struct ContactPoint {
    Eigen::Vector3d position;
    Eigen::Vector3d normal;
    double depth;
    std::string body_a_id;
    std::string body_b_id;
};

class PhysicsEngine {
public:
    virtual ~PhysicsEngine() = default;

    // 팩토리 메서드
    static std::unique_ptr<PhysicsEngine> create(
        const std::string& backend,
        const PhysicsConfig& config);

    // 초기화 및 정리
    virtual void init(const PhysicsConfig& config) = 0;
    virtual void shutdown() = 0;

    // 강체 관리
    virtual void addBody(RigidBody* body) = 0;
    virtual void removeBody(RigidBody* body) = 0;

    // 제약 조건 관리
    virtual void addConstraint(Constraint* constraint) = 0;
    virtual void removeConstraint(Constraint* constraint) = 0;

    // 시뮬레이션 스텝
    virtual void step(double dt) = 0;

    // 레이캐스트 (단일)
    virtual RaycastResult raycast(
        const Eigen::Vector3d& from,
        const Eigen::Vector3d& to) = 0;

    // 배치 레이캐스트 (센서 시뮬레이션 최적화)
    virtual std::vector<RaycastResult> batchRaycast(
        const std::vector<Ray>& rays) = 0;

    // 충돌 접점 조회
    virtual std::vector<ContactPoint> getContactPoints() const = 0;

    // 중력 설정
    virtual void setGravity(const Eigen::Vector3d& gravity) = 0;
};

} // namespace amr::sim
```

#### 4.2.2 커스텀 경량 백엔드 (기본)

AMR 주행/네비게이션에 특화된 2D 중심 경량 물리 엔진이다. 외부 물리 라이브러리 의존성 없이 커스텀 C++로 구현한다.

```cpp
// src/physics/custom_backend.h
#pragma once

#include "physics/physics_engine.h"

#include <memory>
#include <unordered_map>
#include <vector>

namespace amr::sim {

class CustomLightweightBackend : public PhysicsEngine {
public:
    CustomLightweightBackend();
    ~CustomLightweightBackend() override;

    void init(const PhysicsConfig& config) override;
    void shutdown() override;

    void addBody(RigidBody* body) override;
    void removeBody(RigidBody* body) override;

    void addConstraint(Constraint* constraint) override;
    void removeConstraint(Constraint* constraint) override;

    void step(double dt) override;

    RaycastResult raycast(
        const Eigen::Vector3d& from,
        const Eigen::Vector3d& to) override;

    std::vector<RaycastResult> batchRaycast(
        const std::vector<Ray>& rays) override;

    std::vector<ContactPoint> getContactPoints() const override;

    void setGravity(const Eigen::Vector3d& gravity) override;

    std::string getName() const { return "custom"; }

private:
    // 2D 운동학 업데이트
    void updateKinematics(double dt);

    // 2D/2.5D 충돌 감지 (AABB, circle-circle, polygon)
    void detectCollisions();
    void resolveCollisions();

    // 바퀴-지면 접촉 모델
    void applyWheelGroundFriction(double dt);

    // 배터리 소모 모델
    void updateBatteryDepletion(double dt);

    struct Body2D {
        std::string id;
        double x, y, theta;          // 2D 포즈
        double vx, vy, omega;        // 2D 속도
        double radius;               // 충돌 반경 (원형 근사)
        double half_width, half_length; // AABB 충돌
        bool is_static;
    };

    std::vector<Body2D> bodies_;
    std::vector<ContactPoint> contacts_;
    PhysicsConfig config_;
    std::string collision_mode_ = "2d"; // "2d" | "2.5d"
};

} // namespace amr::sim
```

```cpp
// src/physics/custom_backend.cpp
#include "physics/custom_backend.h"
#include <spdlog/spdlog.h>
#include <cmath>
#include <algorithm>

namespace amr::sim {

CustomLightweightBackend::CustomLightweightBackend() = default;

CustomLightweightBackend::~CustomLightweightBackend() {
    shutdown();
}

void CustomLightweightBackend::init(const PhysicsConfig& config) {
    config_ = config;
    collision_mode_ = config.collision_mode;

    spdlog::info("Custom Lightweight Backend initialized: "
                 "collision_mode={}, max_robots={}, friction={}",
                 collision_mode_, config.max_robots, config.default_friction);
}

void CustomLightweightBackend::shutdown() {
    bodies_.clear();
    contacts_.clear();
    spdlog::info("Custom Lightweight Backend shutdown");
}

void CustomLightweightBackend::step(double dt) {
    // Step 1: 2D 운동학 업데이트
    updateKinematics(dt);

    // Step 2: 충돌 감지
    detectCollisions();

    // Step 3: 충돌 응답
    resolveCollisions();

    // Step 4: 바퀴-지면 마찰
    applyWheelGroundFriction(dt);

    // Step 5: 배터리 소모
    updateBatteryDepletion(dt);
}

void CustomLightweightBackend::updateKinematics(double dt) {
    for (auto& body : bodies_) {
        if (body.is_static) continue;
        body.x += body.vx * std::cos(body.theta) * dt
                - body.vy * std::sin(body.theta) * dt;
        body.y += body.vx * std::sin(body.theta) * dt
                + body.vy * std::cos(body.theta) * dt;
        body.theta += body.omega * dt;
    }
}

void CustomLightweightBackend::detectCollisions() {
    contacts_.clear();
    // Circle-circle 충돌 감지 (O(n^2), n < 200에서 충분)
    for (size_t i = 0; i < bodies_.size(); ++i) {
        for (size_t j = i + 1; j < bodies_.size(); ++j) {
            double dx = bodies_[j].x - bodies_[i].x;
            double dy = bodies_[j].y - bodies_[i].y;
            double dist = std::sqrt(dx * dx + dy * dy);
            double min_dist = bodies_[i].radius + bodies_[j].radius;

            if (dist < min_dist && dist > 0.0) {
                ContactPoint cp;
                cp.position = Eigen::Vector3d(
                    (bodies_[i].x + bodies_[j].x) * 0.5,
                    (bodies_[i].y + bodies_[j].y) * 0.5, 0.0);
                cp.normal = Eigen::Vector3d(dx / dist, dy / dist, 0.0);
                cp.depth = min_dist - dist;
                cp.body_a_id = bodies_[i].id;
                cp.body_b_id = bodies_[j].id;
                contacts_.push_back(cp);
            }
        }
    }
}

void CustomLightweightBackend::resolveCollisions() {
    for (const auto& contact : contacts_) {
        // 간단한 위치 보정 (penetration resolution)
        // 실제 구현에서는 body 인덱스를 룩업하여 위치 보정
    }
}

void CustomLightweightBackend::applyWheelGroundFriction(double /*dt*/) {
    // 단순화된 마찰 모델: 속도에 비례하는 감속
    for (auto& body : bodies_) {
        if (body.is_static) continue;
        double friction = config_.default_friction;
        body.vx *= (1.0 - friction * 0.01);
        body.vy *= (1.0 - friction * 0.01);
    }
}

void CustomLightweightBackend::updateBatteryDepletion(double /*dt*/) {
    // 배터리 소모는 Robot 레벨에서 처리 (이동 거리 기반)
}

RaycastResult CustomLightweightBackend::raycast(
    const Eigen::Vector3d& from, const Eigen::Vector3d& to) {
    // 2D 레이캐스트: 각 body의 원형 충돌체와 ray-circle 교차 검사
    RaycastResult result;
    Eigen::Vector3d dir = to - from;
    double max_dist = dir.norm();
    if (max_dist < 1e-9) return result;
    dir /= max_dist;

    double closest = max_dist;
    for (const auto& body : bodies_) {
        Eigen::Vector3d center(body.x, body.y, 0.0);
        Eigen::Vector3d oc = from - center;
        double a = dir.dot(dir);
        double b = 2.0 * oc.dot(dir);
        double c = oc.dot(oc) - body.radius * body.radius;
        double discriminant = b * b - 4 * a * c;

        if (discriminant >= 0) {
            double t = (-b - std::sqrt(discriminant)) / (2.0 * a);
            if (t >= 0 && t < closest) {
                closest = t;
                result.hit = true;
                result.hit_point = from + dir * t;
                result.hit_normal = (result.hit_point - center).normalized();
                result.hit_fraction = t / max_dist;
                result.distance = t;
                result.hit_entity_id = body.id;
            }
        }
    }
    return result;
}

std::vector<RaycastResult> CustomLightweightBackend::batchRaycast(
    const std::vector<Ray>& rays) {
    std::vector<RaycastResult> results;
    results.reserve(rays.size());
    for (const auto& ray : rays) {
        results.push_back(raycast(ray.from, ray.to));
    }
    return results;
}

std::vector<ContactPoint> CustomLightweightBackend::getContactPoints() const {
    return contacts_;
}

void CustomLightweightBackend::setGravity(const Eigen::Vector3d& /*gravity*/) {
    // 2D 엔진에서는 중력 무시 (바닥면 주행 전제)
}

void CustomLightweightBackend::addBody(RigidBody* body) {
    // RigidBody를 Body2D로 변환하여 추가
    if (!body) return;
    Body2D b2d;
    b2d.id = body->id();
    auto pos = body->position();
    b2d.x = pos.x(); b2d.y = pos.y(); b2d.theta = 0.0;
    b2d.vx = 0.0; b2d.vy = 0.0; b2d.omega = 0.0;
    b2d.radius = 0.25; // 기본 충돌 반경
    b2d.is_static = (body->mass() == 0.0);
    bodies_.push_back(b2d);
}

void CustomLightweightBackend::removeBody(RigidBody* body) {
    if (!body) return;
    bodies_.erase(
        std::remove_if(bodies_.begin(), bodies_.end(),
            [&](const Body2D& b) { return b.id == body->id(); }),
        bodies_.end());
}

void CustomLightweightBackend::addConstraint(Constraint* /*constraint*/) {
    // 2D 엔진에서는 제약 조건 미지원 (주행 로봇 전용)
}

void CustomLightweightBackend::removeConstraint(Constraint* /*constraint*/) {
    // 2D 엔진에서는 제약 조건 미지원
}

} // namespace amr::sim
```

#### 4.2.2.1 MuJoCo 백엔드 (플러그인)

고정밀 3D 물리 시뮬레이션이 필요할 때 사용하는 플러그인 백엔드이다. 공유 라이브러리로 동적 로드된다.

```cpp
// src/physics/mujoco_backend.h
#pragma once

#include "physics/physics_engine.h"
#include <mujoco/mujoco.h>
#include <memory>

namespace amr::sim {

class MuJoCoBackend : public PhysicsEngine {
public:
    MuJoCoBackend();
    ~MuJoCoBackend() override;

    void init(const PhysicsConfig& config) override;
    void shutdown() override;

    void addBody(RigidBody* body) override;
    void removeBody(RigidBody* body) override;
    void addConstraint(Constraint* constraint) override;
    void removeConstraint(Constraint* constraint) override;

    void step(double dt) override;

    RaycastResult raycast(
        const Eigen::Vector3d& from,
        const Eigen::Vector3d& to) override;
    std::vector<RaycastResult> batchRaycast(
        const std::vector<Ray>& rays) override;
    std::vector<ContactPoint> getContactPoints() const override;
    void setGravity(const Eigen::Vector3d& gravity) override;

    std::string getName() const { return "mujoco"; }

private:
    mjModel* model_ = nullptr;
    mjData* data_ = nullptr;
    PhysicsConfig config_;
    int solver_iterations_ = 50;
};

} // namespace amr::sim
```

#### 4.2.2.2 Isaac Sim 브릿지 (플러그인)

NVIDIA Isaac Sim과 gRPC로 통신하는 원격 백엔드이다. GPU 가속 물리 및 포토리얼리스틱 센서 시뮬레이션을 지원한다.

```cpp
// src/physics/isaac_bridge.h
#pragma once

#include "physics/physics_engine.h"
#include <grpcpp/grpcpp.h>
#include <memory>
#include <string>

namespace amr::sim {

class IsaacSimBridge : public PhysicsEngine {
public:
    IsaacSimBridge();
    ~IsaacSimBridge() override;

    void init(const PhysicsConfig& config) override;
    void shutdown() override;

    void addBody(RigidBody* body) override;
    void removeBody(RigidBody* body) override;
    void addConstraint(Constraint* constraint) override;
    void removeConstraint(Constraint* constraint) override;

    void step(double dt) override;

    RaycastResult raycast(
        const Eigen::Vector3d& from,
        const Eigen::Vector3d& to) override;
    std::vector<RaycastResult> batchRaycast(
        const std::vector<Ray>& rays) override;
    std::vector<ContactPoint> getContactPoints() const override;
    void setGravity(const Eigen::Vector3d& gravity) override;

    std::string getName() const { return "isaac_sim"; }

private:
    std::string endpoint_ = "localhost:50055";
    std::shared_ptr<grpc::Channel> channel_;
    bool use_gpu_physics_ = true;
    bool use_rtx_lidar_ = true;
    PhysicsConfig config_;
};

} // namespace amr::sim
```

#### 4.2.3 Rigid Body Wrapper

```cpp
// src/physics/rigid_body.h
#pragma once

#include <Eigen/Core>
#include <Eigen/Geometry>
#include <memory>
#include <string>

namespace amr::sim {

enum class CollisionShape {
    Box,        // 직육면체
    Cylinder,   // 원기둥
    Sphere,     // 구
    ConvexHull, // 볼록 껍질
    TriMesh,    // 삼각형 메시 (정적 객체 전용)
    Compound    // 복합 형상
};

struct RigidBodyConfig {
    std::string id;
    CollisionShape shape = CollisionShape::Box;
    Eigen::Vector3d dimensions{1.0, 1.0, 1.0}; // Box: half-extents, Cylinder: radius/height, Sphere: radius
    double mass = 1.0;                          // 0.0 = static
    double friction = 0.5;
    double restitution = 0.1;
    double linear_damping = 0.05;
    double angular_damping = 0.85;
    Eigen::Vector3d initial_position{0, 0, 0};
    Eigen::Quaterniond initial_orientation{1, 0, 0, 0};
};

class RigidBody {
public:
    explicit RigidBody(const RigidBodyConfig& config);
    ~RigidBody();

    // 위치/방향 조회
    Eigen::Vector3d position() const;
    Eigen::Quaterniond orientation() const;
    Eigen::Isometry3d pose() const;

    // 속도 조회
    Eigen::Vector3d linearVelocity() const;
    Eigen::Vector3d angularVelocity() const;

    // 속도 설정 (kinematic 제어)
    void setLinearVelocity(const Eigen::Vector3d& vel);
    void setAngularVelocity(const Eigen::Vector3d& vel);

    // 위치 직접 설정 (teleport)
    void setPose(const Eigen::Vector3d& pos, const Eigen::Quaterniond& orient);

    // 힘/토크 적용
    void applyForce(const Eigen::Vector3d& force);
    void applyTorque(const Eigen::Vector3d& torque);
    void applyCentralImpulse(const Eigen::Vector3d& impulse);

    // 활성화/비활성화
    void activate();
    void deactivate();

    const std::string& id() const { return config_.id; }
    double mass() const { return config_.mass; }

private:
    void createShape();
    void createBody();

    RigidBodyConfig config_;
    // 내부 물리 표현은 백엔드에 의해 관리됨
    Eigen::Vector3d position_;
    Eigen::Quaterniond orientation_;
    Eigen::Vector3d linear_velocity_{0, 0, 0};
    Eigen::Vector3d angular_velocity_{0, 0, 0};
};

} // namespace amr::sim
```

#### 4.2.4 Constraints (관절)

```cpp
// src/physics/constraints.h
#pragma once

#include <Eigen/Core>
#include <memory>
#include <string>

namespace amr::sim {

class RigidBody;

enum class ConstraintType {
    Hinge,      // 경첩 관절 (1축 회전) - 바퀴용
    Fixed,      // 고정 관절
    Slider,     // 슬라이더 (1축 이동)
    Generic6DOF // 6자유도 범용
};

class Constraint {
public:
    virtual ~Constraint() = default;
    virtual ConstraintType type() const = 0;
};

// 바퀴 관절: 경첩(hinge) 제약으로 구현
class HingeConstraint : public Constraint {
public:
    HingeConstraint(
        RigidBody& chassis,
        RigidBody& wheel,
        const Eigen::Vector3d& pivot_in_chassis,
        const Eigen::Vector3d& pivot_in_wheel,
        const Eigen::Vector3d& axis_in_chassis,
        const Eigen::Vector3d& axis_in_wheel);

    // 모터 제어
    void enableMotor(bool enable);
    void setMotorTargetVelocity(double velocity);
    void setMaxMotorImpulse(double impulse);

    ConstraintType type() const override { return ConstraintType::Hinge; }

    // 현재 회전 각도 조회
    double getAngle() const;

private:
    RigidBody& chassis_;
    RigidBody& wheel_;
    Eigen::Vector3d axis_;
    double target_velocity_ = 0.0;
    double max_impulse_ = 0.0;
    bool motor_enabled_ = false;
};

} // namespace amr::sim
```

---

### 4.3 Robot Models

#### 4.3.1 Robot State

```cpp
// src/robots/robot_state.h
#pragma once

#include <Eigen/Core>
#include <Eigen/Geometry>
#include <string>

namespace amr::sim {

struct BatteryState {
    double percentage = 100.0;    // 0.0 ~ 100.0
    double voltage = 24.0;        // 볼트
    bool charging = false;
    double discharge_rate = 0.01; // % per meter traveled
    double idle_rate = 0.001;     // % per second idle
};

struct WheelState {
    double velocity = 0.0;        // rad/s
    int64_t encoder_ticks = 0;    // 누적 엔코더 틱
    double angle = 0.0;           // 누적 회전 각도 (rad)
};

struct RobotState {
    // 식별
    std::string robot_id;

    // 위치 및 방향 (월드 좌표계)
    Eigen::Vector3d position{0, 0, 0};
    Eigen::Quaterniond orientation{1, 0, 0, 0};

    // 속도 (로봇 로컬 좌표계)
    Eigen::Vector3d linear_velocity{0, 0, 0};
    Eigen::Vector3d angular_velocity{0, 0, 0};

    // 바퀴 상태
    WheelState left_wheel;
    WheelState right_wheel;
    // mecanum인 경우: front_left, front_right, rear_left, rear_right
    WheelState wheels[4]; // 인덱스 0=FL, 1=FR, 2=RL, 3=RR

    // 배터리
    BatteryState battery;

    // 주행 누적 거리 (미터)
    double odometry_distance = 0.0;

    // 타임스탬프 (시뮬레이션 시간, 초)
    double timestamp = 0.0;

    // 편의 함수
    double yaw() const;
    double speed() const;
};

} // namespace amr::sim
```

#### 4.3.2 Robot Entity

```cpp
// src/robots/robot.h
#pragma once

#include "robots/robot_state.h"
#include "robots/robot_controller.h"
#include "sensors/sensor.h"
#include "physics/rigid_body.h"
#include "physics/constraints.h"
#include "physics/physics_engine.h"
#include "core/config.h"

#include <memory>
#include <string>
#include <vector>

namespace amr::sim {

class Robot {
public:
    explicit Robot(const RobotConfig& config);
    ~Robot();

    // 초기화: 물리 바디 + 센서 생성
    void init(PhysicsEngine& physics);

    // 속도 명령 수신
    void setCommand(const VelocityCommand& cmd);

    // 컨트롤러 업데이트 (물리 스텝 전)
    void updateController(double dt);

    // 센서 업데이트 (물리 스텝 후)
    void updateSensors(double dt, PhysicsEngine& physics);

    // 상태 조회
    RobotState getState() const;
    const std::string& id() const { return config_.robot_id; }

    // 센서 데이터 읽기
    std::vector<SensorData> readSensors() const;

    // 센서 추가
    void addSensor(std::unique_ptr<Sensor> sensor);

    // 물리 바디 접근
    RigidBody* chassisBody() { return chassis_.get(); }

    // 텔레포트 (위치 직접 설정)
    void teleport(const Eigen::Vector3d& position, double yaw);

    // 리셋
    void reset();

private:
    void createPhysicsBodies(PhysicsEngine& physics);
    void updateBattery(double dt);
    void updateOdometry(double dt);

    RobotConfig config_;
    std::unique_ptr<RobotController> controller_;
    std::unique_ptr<RigidBody> chassis_;
    std::vector<std::unique_ptr<RigidBody>> wheels_;
    std::vector<std::unique_ptr<HingeConstraint>> wheel_joints_;
    std::vector<std::unique_ptr<Sensor>> sensors_;

    RobotState state_;
    VelocityCommand current_command_;
    double prev_left_angle_ = 0.0;
    double prev_right_angle_ = 0.0;
};

} // namespace amr::sim
```

#### 4.3.3 Differential Drive

```cpp
// src/robots/differential_drive.h
#pragma once

#include "robots/robot_controller.h"

namespace amr::sim {

struct DifferentialDriveParams {
    double wheel_radius = 0.05;       // 바퀴 반지름 (m)
    double wheel_separation = 0.3;    // 바퀴 간 거리 (m), L
    double max_linear_speed = 1.0;    // 최대 전진 속도 (m/s)
    double max_angular_speed = 2.0;   // 최대 회전 속도 (rad/s)
    double max_linear_accel = 0.5;    // 최대 전진 가속도 (m/s²)
    double max_angular_accel = 1.0;   // 최대 회전 가속도 (rad/s²)
    double max_wheel_torque = 5.0;    // 최대 바퀴 토크 (Nm)
};

class DifferentialDrive : public RobotController {
public:
    explicit DifferentialDrive(const DifferentialDriveParams& params);

    // VelocityCommand (v, ω) → 좌/우 바퀴 속도
    WheelVelocities computeWheelVelocities(
        const VelocityCommand& cmd, double dt) override;

    // 좌/우 바퀴 속도 → 로봇 속도 (포워드 키네매틱스)
    // 엔코더 기반 오도메트리 계산에 사용
    VelocityEstimate forwardKinematics(
        double left_wheel_vel,
        double right_wheel_vel) const override;

    DriveType driveType() const override { return DriveType::Differential; }

private:
    DifferentialDriveParams params_;

    // 가속도 제한 적용
    double applyAccelLimit(double current, double target, double max_accel, double dt);

    double current_linear_ = 0.0;
    double current_angular_ = 0.0;
};

/*
 * 역운동학 (Inverse Kinematics):
 *   v_left  = (v - ω * L/2) / r
 *   v_right = (v + ω * L/2) / r
 *
 *   여기서:
 *     v = 전진 속도 (m/s)
 *     ω = 회전 속도 (rad/s)
 *     L = 바퀴 간 거리 (m)
 *     r = 바퀴 반지름 (m)
 *
 * 순운동학 (Forward Kinematics):
 *   v = r * (v_right + v_left) / 2
 *   ω = r * (v_right - v_left) / L
 */

} // namespace amr::sim
```

```cpp
// src/robots/differential_drive.cpp
#include "robots/differential_drive.h"
#include <algorithm>
#include <cmath>

namespace amr::sim {

DifferentialDrive::DifferentialDrive(const DifferentialDriveParams& params)
    : params_(params) {}

WheelVelocities DifferentialDrive::computeWheelVelocities(
    const VelocityCommand& cmd, double dt) {

    // 속도 제한 적용
    double target_linear = std::clamp(cmd.linear_x,
        -params_.max_linear_speed, params_.max_linear_speed);
    double target_angular = std::clamp(cmd.angular_z,
        -params_.max_angular_speed, params_.max_angular_speed);

    // 가속도 제한 적용
    current_linear_ = applyAccelLimit(
        current_linear_, target_linear, params_.max_linear_accel, dt);
    current_angular_ = applyAccelLimit(
        current_angular_, target_angular, params_.max_angular_accel, dt);

    // 역운동학: 선속도/각속도 → 바퀴 각속도 (rad/s)
    double v_left = (current_linear_ - current_angular_ * params_.wheel_separation / 2.0)
                    / params_.wheel_radius;
    double v_right = (current_linear_ + current_angular_ * params_.wheel_separation / 2.0)
                     / params_.wheel_radius;

    WheelVelocities result;
    result.left = v_left;
    result.right = v_right;
    return result;
}

VelocityEstimate DifferentialDrive::forwardKinematics(
    double left_wheel_vel, double right_wheel_vel) const {

    VelocityEstimate est;
    est.linear_x = params_.wheel_radius * (right_wheel_vel + left_wheel_vel) / 2.0;
    est.angular_z = params_.wheel_radius * (right_wheel_vel - left_wheel_vel)
                    / params_.wheel_separation;
    return est;
}

double DifferentialDrive::applyAccelLimit(
    double current, double target, double max_accel, double dt) {
    double diff = target - current;
    double max_change = max_accel * dt;
    if (std::abs(diff) > max_change) {
        return current + std::copysign(max_change, diff);
    }
    return target;
}

} // namespace amr::sim
```

#### 4.3.4 Mecanum Drive

```cpp
// src/robots/mecanum_drive.h
#pragma once

#include "robots/robot_controller.h"

namespace amr::sim {

struct MecanumDriveParams {
    double wheel_radius = 0.05;
    double wheel_separation_x = 0.3; // 전후 바퀴 간 거리
    double wheel_separation_y = 0.25; // 좌우 바퀴 간 거리
    double max_linear_speed = 1.0;
    double max_angular_speed = 2.0;
    double max_linear_accel = 0.5;
    double max_angular_accel = 1.0;
    double roller_angle = M_PI / 4.0; // 메카넘 롤러 각도 (45도)
};

class MecanumDrive : public RobotController {
public:
    explicit MecanumDrive(const MecanumDriveParams& params);

    WheelVelocities computeWheelVelocities(
        const VelocityCommand& cmd, double dt) override;

    VelocityEstimate forwardKinematics(
        double left_wheel_vel,
        double right_wheel_vel) const override;

    DriveType driveType() const override { return DriveType::Mecanum; }

    // 4바퀴 순운동학
    VelocityEstimate forwardKinematics4(
        double fl, double fr, double rl, double rr) const;

private:
    MecanumDriveParams params_;
    double current_vx_ = 0.0;
    double current_vy_ = 0.0;
    double current_wz_ = 0.0;
};

/*
 * 메카넘 역운동학 (Inverse Kinematics):
 *   Lx = wheel_separation_x / 2
 *   Ly = wheel_separation_y / 2
 *
 *   v_fl = (1/r) * (vx - vy - (Lx + Ly) * ω)
 *   v_fr = (1/r) * (vx + vy + (Lx + Ly) * ω)
 *   v_rl = (1/r) * (vx + vy - (Lx + Ly) * ω)
 *   v_rr = (1/r) * (vx - vy + (Lx + Ly) * ω)
 *
 * 순운동학 (Forward Kinematics):
 *   vx = r/4 * (v_fl + v_fr + v_rl + v_rr)
 *   vy = r/4 * (-v_fl + v_fr + v_rl - v_rr)
 *   ω  = r/(4*(Lx+Ly)) * (-v_fl + v_fr - v_rl + v_rr)
 */

} // namespace amr::sim
```

#### 4.3.5 Robot Controller 인터페이스

```cpp
// src/robots/robot_controller.h
#pragma once

#include <string>

namespace amr::sim {

struct VelocityCommand; // forward declaration (command_handler.h)

struct WheelVelocities {
    double left = 0.0;
    double right = 0.0;
    // Mecanum 전용
    double front_left = 0.0;
    double front_right = 0.0;
    double rear_left = 0.0;
    double rear_right = 0.0;
};

struct VelocityEstimate {
    double linear_x = 0.0;
    double linear_y = 0.0;  // mecanum만 해당
    double angular_z = 0.0;
};

enum class DriveType {
    Differential,
    Mecanum
};

class RobotController {
public:
    virtual ~RobotController() = default;

    virtual WheelVelocities computeWheelVelocities(
        const VelocityCommand& cmd, double dt) = 0;

    virtual VelocityEstimate forwardKinematics(
        double left_wheel_vel,
        double right_wheel_vel) const = 0;

    virtual DriveType driveType() const = 0;
};

} // namespace amr::sim
```

---

### 4.4 Sensors

#### 4.4.1 센서 기반 클래스

```cpp
// src/sensors/sensor.h
#pragma once

#include "physics/physics_engine.h"

#include <Eigen/Core>
#include <Eigen/Geometry>
#include <memory>
#include <string>
#include <variant>
#include <vector>

namespace amr::sim {

// 센서 데이터 타입
struct LidarScanData {
    std::vector<float> ranges;         // 거리 값 (m)
    std::vector<float> intensities;    // 반사 강도 (0.0 ~ 1.0)
    float angle_min;                   // 시작 각도 (rad)
    float angle_max;                   // 종료 각도 (rad)
    float angle_increment;             // 각도 증분 (rad)
    float range_min;                   // 최소 거리 (m)
    float range_max;                   // 최대 거리 (m)
};

struct PointCloudData {
    std::vector<float> points;         // [x0,y0,z0, x1,y1,z1, ...]
    std::vector<float> intensities;
    uint32_t width;
    uint32_t height;
};

struct ImageData {
    std::vector<uint8_t> data;         // RGB 또는 mono
    uint32_t width;
    uint32_t height;
    uint32_t channels;                 // 1=mono, 3=RGB, 4=RGBA
    std::string encoding;              // "rgb8", "mono8", "32FC1" (depth)
};

struct DepthImageData {
    std::vector<float> data;           // depth in meters
    uint32_t width;
    uint32_t height;
};

struct ImuData {
    Eigen::Vector3d linear_acceleration;
    Eigen::Vector3d angular_velocity;
    Eigen::Quaterniond orientation;    // 선택적
};

struct EncoderData {
    int64_t left_ticks;
    int64_t right_ticks;
    double left_velocity;              // rad/s
    double right_velocity;             // rad/s
};

using SensorData = std::variant<
    LidarScanData,
    PointCloudData,
    ImageData,
    DepthImageData,
    ImuData,
    EncoderData
>;

struct SensorConfig {
    std::string name;                  // 센서 이름 (예: "front_lidar")
    std::string type;                  // "lidar_2d", "lidar_3d", "camera_rgb", "camera_depth", "imu", "encoder"
    double update_rate = 10.0;         // Hz
    Eigen::Vector3d position_offset{0, 0, 0};    // 로봇 중심 기준 위치 오프셋
    Eigen::Quaterniond orientation_offset{1, 0, 0, 0}; // 로봇 중심 기준 방향 오프셋
};

class Sensor {
public:
    explicit Sensor(const SensorConfig& config);
    virtual ~Sensor() = default;

    // 센서 업데이트 (주기에 따라 자동 판단)
    // dt: 시뮬레이션 타임스텝
    // robot_pose: 로봇의 월드 좌표 포즈
    // physics: 물리 엔진 (레이캐스트 등)
    bool update(double dt,
                const Eigen::Isometry3d& robot_pose,
                PhysicsEngine& physics);

    // 최신 센서 데이터 반환
    virtual SensorData getData() const = 0;

    // 센서 설정 조회
    const SensorConfig& config() const { return config_; }
    const std::string& name() const { return config_.name; }

    // 월드 좌표에서 센서 포즈 계산
    Eigen::Isometry3d worldPose(const Eigen::Isometry3d& robot_pose) const;

protected:
    // 구현 클래스가 오버라이드
    virtual void doUpdate(const Eigen::Isometry3d& sensor_pose,
                          PhysicsEngine& physics) = 0;

    SensorConfig config_;
    double time_since_last_update_ = 0.0;
    double update_period_;  // 1.0 / update_rate
};

} // namespace amr::sim
```

```cpp
// src/sensors/sensor.cpp
#include "sensors/sensor.h"

namespace amr::sim {

Sensor::Sensor(const SensorConfig& config)
    : config_(config)
    , update_period_(1.0 / config.update_rate) {}

bool Sensor::update(double dt,
                    const Eigen::Isometry3d& robot_pose,
                    PhysicsEngine& physics) {
    time_since_last_update_ += dt;
    if (time_since_last_update_ >= update_period_) {
        time_since_last_update_ -= update_period_;
        auto sensor_pose = worldPose(robot_pose);
        doUpdate(sensor_pose, physics);
        return true;  // 데이터가 갱신됨
    }
    return false;  // 아직 갱신 주기가 아님
}

Eigen::Isometry3d Sensor::worldPose(const Eigen::Isometry3d& robot_pose) const {
    Eigen::Isometry3d sensor_offset = Eigen::Isometry3d::Identity();
    sensor_offset.translate(config_.position_offset);
    sensor_offset.rotate(config_.orientation_offset);
    return robot_pose * sensor_offset;
}

} // namespace amr::sim
```

#### 4.4.2 2D LiDAR

```cpp
// src/sensors/lidar_2d.h
#pragma once

#include "sensors/sensor.h"
#include "sensors/noise_model.h"
#include <vector>

namespace amr::sim {

struct Lidar2DConfig : public SensorConfig {
    float angle_min = -M_PI;          // -180도
    float angle_max = M_PI;           // +180도
    float angle_increment = 0.00436;  // ~0.25도 (360도 / 828빔)
    float range_min = 0.1;            // 최소 감지 거리 (m)
    float range_max = 12.0;           // 최대 감지 거리 (m)
    float noise_stddev = 0.005;       // 거리 노이즈 표준편차 (m)

    // 빔 수 계산
    int numBeams() const {
        return static_cast<int>((angle_max - angle_min) / angle_increment) + 1;
    }
};

class Lidar2D : public Sensor {
public:
    explicit Lidar2D(const Lidar2DConfig& config);

    SensorData getData() const override;

protected:
    void doUpdate(const Eigen::Isometry3d& sensor_pose,
                  PhysicsEngine& physics) override;

private:
    Lidar2DConfig lidar_config_;
    LidarScanData last_scan_;
    GaussianNoise noise_;
};

} // namespace amr::sim
```

```cpp
// src/sensors/lidar_2d.cpp
#include "sensors/lidar_2d.h"
#include <cmath>

namespace amr::sim {

Lidar2D::Lidar2D(const Lidar2DConfig& config)
    : Sensor(config)
    , lidar_config_(config)
    , noise_(0.0, config.noise_stddev) {

    // 결과 버퍼 사전 할당
    int num_beams = lidar_config_.numBeams();
    last_scan_.ranges.resize(num_beams, lidar_config_.range_max);
    last_scan_.intensities.resize(num_beams, 0.0f);
    last_scan_.angle_min = lidar_config_.angle_min;
    last_scan_.angle_max = lidar_config_.angle_max;
    last_scan_.angle_increment = lidar_config_.angle_increment;
    last_scan_.range_min = lidar_config_.range_min;
    last_scan_.range_max = lidar_config_.range_max;
}

void Lidar2D::doUpdate(const Eigen::Isometry3d& sensor_pose,
                        PhysicsEngine& physics) {
    int num_beams = lidar_config_.numBeams();
    Eigen::Vector3d origin = sensor_pose.translation();

    // 배치 레이캐스트를 위한 레이 생성
    std::vector<Ray> rays;
    rays.reserve(num_beams);

    for (int i = 0; i < num_beams; ++i) {
        float angle = lidar_config_.angle_min + i * lidar_config_.angle_increment;

        // 센서 로컬 좌표계에서 방향 벡터 계산
        Eigen::Vector3d local_dir(
            std::cos(angle),
            std::sin(angle),
            0.0);

        // 월드 좌표계로 변환
        Eigen::Vector3d world_dir = sensor_pose.rotation() * local_dir;
        Eigen::Vector3d end_point = origin + world_dir * lidar_config_.range_max;

        rays.push_back({origin, end_point});
    }

    // 배치 레이캐스트 실행
    auto results = physics.batchRaycast(rays);

    // 결과 처리
    for (int i = 0; i < num_beams; ++i) {
        if (results[i].hit) {
            float range = static_cast<float>(results[i].distance);

            // 노이즈 적용
            range += static_cast<float>(noise_.sample());

            // 범위 확인
            if (range >= lidar_config_.range_min && range <= lidar_config_.range_max) {
                last_scan_.ranges[i] = range;
                last_scan_.intensities[i] = 1.0f; // 단순 반사 모델
            } else {
                last_scan_.ranges[i] = lidar_config_.range_max;
                last_scan_.intensities[i] = 0.0f;
            }
        } else {
            last_scan_.ranges[i] = lidar_config_.range_max;
            last_scan_.intensities[i] = 0.0f;
        }
    }
}

SensorData Lidar2D::getData() const {
    return last_scan_;
}

} // namespace amr::sim
```

#### 4.4.3 3D LiDAR

```cpp
// src/sensors/lidar_3d.h
#pragma once

#include "sensors/sensor.h"
#include "sensors/noise_model.h"

namespace amr::sim {

struct Lidar3DConfig : public SensorConfig {
    // 수평 파라미터
    float h_angle_min = -M_PI;
    float h_angle_max = M_PI;
    float h_angle_increment = 0.003491; // ~0.2도

    // 수직 파라미터
    float v_angle_min = -0.2618;  // -15도
    float v_angle_max = 0.2618;   // +15도
    int v_num_beams = 16;         // 수직 빔 수

    float range_min = 0.3;
    float range_max = 100.0;
    float noise_stddev = 0.01;

    int hNumBeams() const {
        return static_cast<int>((h_angle_max - h_angle_min) / h_angle_increment) + 1;
    }
    int totalBeams() const { return hNumBeams() * v_num_beams; }
};

class Lidar3D : public Sensor {
public:
    explicit Lidar3D(const Lidar3DConfig& config);

    SensorData getData() const override;

protected:
    void doUpdate(const Eigen::Isometry3d& sensor_pose,
                  PhysicsEngine& physics) override;

private:
    Lidar3DConfig lidar_config_;
    PointCloudData last_cloud_;
    GaussianNoise noise_;
};

} // namespace amr::sim
```

#### 4.4.4 IMU

```cpp
// src/sensors/imu.h
#pragma once

#include "sensors/sensor.h"
#include "sensors/noise_model.h"

namespace amr::sim {

struct ImuConfig : public SensorConfig {
    // 가속도계 노이즈
    double accel_noise_density = 0.001;     // m/s²/√Hz
    double accel_random_walk = 0.0001;      // m/s³/√Hz (바이어스 드리프트)

    // 자이로스코프 노이즈
    double gyro_noise_density = 0.0001;     // rad/s/√Hz
    double gyro_random_walk = 0.00001;      // rad/s²/√Hz (바이어스 드리프트)

    // 초기 바이어스
    Eigen::Vector3d accel_bias{0, 0, 0};
    Eigen::Vector3d gyro_bias{0, 0, 0};

    // 중력 보상
    bool include_gravity = true;
    Eigen::Vector3d gravity{0, 0, -9.81};
};

class Imu : public Sensor {
public:
    explicit Imu(const ImuConfig& config);

    SensorData getData() const override;

protected:
    void doUpdate(const Eigen::Isometry3d& sensor_pose,
                  PhysicsEngine& physics) override;

private:
    ImuConfig imu_config_;
    ImuData last_data_;

    // 바이어스 드리프트 상태
    Eigen::Vector3d accel_bias_state_;
    Eigen::Vector3d gyro_bias_state_;

    // 이전 프레임 데이터 (미분 계산용)
    Eigen::Vector3d prev_linear_vel_{0, 0, 0};
    Eigen::Quaterniond prev_orientation_{1, 0, 0, 0};
    bool first_update_ = true;

    GaussianNoise accel_noise_;
    GaussianNoise gyro_noise_;
};

} // namespace amr::sim
```

```cpp
// src/sensors/imu.cpp
#include "sensors/imu.h"

namespace amr::sim {

Imu::Imu(const ImuConfig& config)
    : Sensor(config)
    , imu_config_(config)
    , accel_bias_state_(config.accel_bias)
    , gyro_bias_state_(config.gyro_bias)
    , accel_noise_(0.0, config.accel_noise_density)
    , gyro_noise_(0.0, config.gyro_noise_density) {}

void Imu::doUpdate(const Eigen::Isometry3d& sensor_pose,
                    PhysicsEngine& /*physics*/) {
    double dt = update_period_;

    Eigen::Quaterniond orientation(sensor_pose.rotation());
    Eigen::Vector3d position = sensor_pose.translation();

    // 각속도 계산 (방향 변화로부터)
    if (!first_update_) {
        Eigen::Quaterniond dq = prev_orientation_.conjugate() * orientation;
        // 쿼터니언 → 각속도 근사
        Eigen::Vector3d angular_vel;
        if (dq.w() < 0) {
            dq.coeffs() *= -1; // 최단 경로
        }
        angular_vel = 2.0 * dq.vec() / dt;

        // 센서 로컬 좌표계로 변환
        last_data_.angular_velocity = sensor_pose.rotation().transpose() * angular_vel;

        // 바이어스 및 노이즈 적용
        last_data_.angular_velocity += gyro_bias_state_;
        last_data_.angular_velocity.x() += gyro_noise_.sample();
        last_data_.angular_velocity.y() += gyro_noise_.sample();
        last_data_.angular_velocity.z() += gyro_noise_.sample();

        // 선가속도 계산 (속도 변화로부터)
        // 실제로는 물리 엔진에서 직접 가져오는 것이 더 정확
        // 여기서는 유한 차분 방식 사용
        Eigen::Vector3d accel_world = Eigen::Vector3d::Zero();

        // 중력 포함
        if (imu_config_.include_gravity) {
            accel_world -= imu_config_.gravity;
        }

        // 센서 로컬 좌표계로 변환
        last_data_.linear_acceleration =
            sensor_pose.rotation().transpose() * accel_world;

        // 바이어스 및 노이즈 적용
        last_data_.linear_acceleration += accel_bias_state_;
        last_data_.linear_acceleration.x() += accel_noise_.sample();
        last_data_.linear_acceleration.y() += accel_noise_.sample();
        last_data_.linear_acceleration.z() += accel_noise_.sample();
    }

    last_data_.orientation = orientation;
    prev_orientation_ = orientation;
    first_update_ = false;

    // 바이어스 드리프트 (랜덤 워크)
    GaussianNoise bias_drift_a(0.0, imu_config_.accel_random_walk * std::sqrt(dt));
    GaussianNoise bias_drift_g(0.0, imu_config_.gyro_random_walk * std::sqrt(dt));
    accel_bias_state_.x() += bias_drift_a.sample();
    accel_bias_state_.y() += bias_drift_a.sample();
    accel_bias_state_.z() += bias_drift_a.sample();
    gyro_bias_state_.x() += bias_drift_g.sample();
    gyro_bias_state_.y() += bias_drift_g.sample();
    gyro_bias_state_.z() += bias_drift_g.sample();
}

SensorData Imu::getData() const {
    return last_data_;
}

} // namespace amr::sim
```

#### 4.4.5 Wheel Encoder

```cpp
// src/sensors/encoder.h
#pragma once

#include "sensors/sensor.h"
#include "sensors/noise_model.h"

namespace amr::sim {

struct EncoderConfig : public SensorConfig {
    int ticks_per_revolution = 4096;  // 회전당 틱 수
    double wheel_radius = 0.05;       // 바퀴 반지름 (m)
    double missed_tick_probability = 0.001; // 틱 누락 확률
};

class Encoder : public Sensor {
public:
    explicit Encoder(const EncoderConfig& config);

    SensorData getData() const override;

    // 바퀴 각속도 설정 (Robot에서 호출)
    void setWheelVelocities(double left_vel, double right_vel);

protected:
    void doUpdate(const Eigen::Isometry3d& sensor_pose,
                  PhysicsEngine& physics) override;

private:
    EncoderConfig enc_config_;
    EncoderData last_data_{};

    double left_angle_accum_ = 0.0;   // 누적 좌측 바퀴 각도
    double right_angle_accum_ = 0.0;  // 누적 우측 바퀴 각도
    double current_left_vel_ = 0.0;
    double current_right_vel_ = 0.0;

    UniformNoise missed_tick_noise_;
};

} // namespace amr::sim
```

#### 4.4.6 RGB Camera (Vulkan)

```cpp
// src/sensors/camera_rgb.h
#pragma once

#include "sensors/sensor.h"
#include "rendering/vulkan_context.h"
#include "rendering/render_pipeline.h"
#include "rendering/framebuffer.h"
#include "sensors/noise_model.h"

namespace amr::sim {

struct CameraRGBConfig : public SensorConfig {
    uint32_t width = 640;
    uint32_t height = 480;
    double fov_horizontal = 1.0472;    // 60도 (rad)
    double near_clip = 0.1;
    double far_clip = 100.0;
    double noise_stddev = 2.0;         // 픽셀 값 노이즈 (0-255 범위)
};

class CameraRGB : public Sensor {
public:
    CameraRGB(const CameraRGBConfig& config,
              std::shared_ptr<VulkanContext> vulkan);

    SensorData getData() const override;

protected:
    void doUpdate(const Eigen::Isometry3d& sensor_pose,
                  PhysicsEngine& physics) override;

private:
    CameraRGBConfig cam_config_;
    std::shared_ptr<VulkanContext> vulkan_;
    std::unique_ptr<RenderPipeline> pipeline_;
    std::unique_ptr<Framebuffer> framebuffer_;
    ImageData last_image_;
    GaussianNoise noise_;

    Eigen::Matrix4d computeViewMatrix(const Eigen::Isometry3d& pose) const;
    Eigen::Matrix4d computeProjectionMatrix() const;
};

} // namespace amr::sim
```

#### 4.4.7 Depth Camera

```cpp
// src/sensors/camera_depth.h
#pragma once

#include "sensors/sensor.h"
#include "rendering/vulkan_context.h"
#include "rendering/render_pipeline.h"
#include "rendering/framebuffer.h"
#include "sensors/noise_model.h"

namespace amr::sim {

struct CameraDepthConfig : public SensorConfig {
    uint32_t width = 640;
    uint32_t height = 480;
    double fov_horizontal = 1.0472;
    double near_clip = 0.1;
    double far_clip = 10.0;
    double noise_stddev = 0.005;       // 뎁스 노이즈 (m)
};

class CameraDepth : public Sensor {
public:
    CameraDepth(const CameraDepthConfig& config,
                std::shared_ptr<VulkanContext> vulkan);

    SensorData getData() const override;

protected:
    void doUpdate(const Eigen::Isometry3d& sensor_pose,
                  PhysicsEngine& physics) override;

private:
    CameraDepthConfig cam_config_;
    std::shared_ptr<VulkanContext> vulkan_;
    std::unique_ptr<RenderPipeline> depth_pipeline_;
    std::unique_ptr<Framebuffer> depth_framebuffer_;
    DepthImageData last_depth_;
    GaussianNoise noise_;
};

} // namespace amr::sim
```

#### 4.4.8 Noise Model

```cpp
// src/sensors/noise_model.h
#pragma once

#include <random>

namespace amr::sim {

class GaussianNoise {
public:
    GaussianNoise(double mean = 0.0, double stddev = 1.0, uint64_t seed = 0);

    double sample();
    void setSeed(uint64_t seed);
    void setParameters(double mean, double stddev);

private:
    std::mt19937_64 rng_;
    std::normal_distribution<double> dist_;
};

class UniformNoise {
public:
    UniformNoise(double min = 0.0, double max = 1.0, uint64_t seed = 0);

    double sample();
    void setSeed(uint64_t seed);

private:
    std::mt19937_64 rng_;
    std::uniform_real_distribution<double> dist_;
};

} // namespace amr::sim
```

```cpp
// src/sensors/noise_model.cpp
#include "sensors/noise_model.h"
#include <chrono>

namespace amr::sim {

GaussianNoise::GaussianNoise(double mean, double stddev, uint64_t seed)
    : dist_(mean, stddev) {
    if (seed == 0) {
        seed = std::chrono::steady_clock::now().time_since_epoch().count();
    }
    rng_.seed(seed);
}

double GaussianNoise::sample() {
    return dist_(rng_);
}

void GaussianNoise::setSeed(uint64_t seed) {
    rng_.seed(seed);
}

void GaussianNoise::setParameters(double mean, double stddev) {
    dist_ = std::normal_distribution<double>(mean, stddev);
}

UniformNoise::UniformNoise(double min, double max, uint64_t seed)
    : dist_(min, max) {
    if (seed == 0) {
        seed = std::chrono::steady_clock::now().time_since_epoch().count();
    }
    rng_.seed(seed);
}

double UniformNoise::sample() {
    return dist_(rng_);
}

void UniformNoise::setSeed(uint64_t seed) {
    rng_.seed(seed);
}

} // namespace amr::sim
```

---

### 4.5 gRPC Service Interface

> 📋 Proto 정의 상세: [`integration-spec.md`](../integration/integration-spec.md#shared-proto) 참조

#### 4.5.5 gRPC 서버 구현

```cpp
// src/grpc/sim_server.h
#pragma once

#include "simulation.grpc.pb.h"
#include "core/simulation.h"
#include "grpc/command_handler.h"
#include "grpc/telemetry_streamer.h"

#include <grpcpp/grpcpp.h>
#include <memory>
#include <string>
#include <unordered_map>
#include <mutex>

namespace amr::sim {

class SimServer final : public amr::simulation::SimulationService::Service {
public:
    explicit SimServer(uint16_t port = 50051);
    ~SimServer();

    // 서버 시작 (블로킹)
    void run();

    // 서버 시작 (별도 스레드)
    void runAsync();

    // 서버 정지
    void shutdown();

    // gRPC 서비스 메서드 구현
    grpc::Status CreateSimulation(
        grpc::ServerContext* context,
        const amr::simulation::CreateSimRequest* request,
        amr::simulation::CreateSimResponse* response) override;

    grpc::Status StartSimulation(
        grpc::ServerContext* context,
        const amr::simulation::StartSimRequest* request,
        grpc::ServerWriter<amr::simulation::SimEvent>* writer) override;

    grpc::Status StopSimulation(
        grpc::ServerContext* context,
        const amr::simulation::StopSimRequest* request,
        amr::simulation::StopSimResponse* response) override;

    grpc::Status ResetSimulation(
        grpc::ServerContext* context,
        const amr::simulation::ResetSimRequest* request,
        amr::simulation::ResetSimResponse* response) override;

    grpc::Status SpawnRobot(
        grpc::ServerContext* context,
        const amr::simulation::SpawnRobotRequest* request,
        amr::simulation::SpawnRobotResponse* response) override;

    grpc::Status RemoveRobot(
        grpc::ServerContext* context,
        const amr::simulation::RemoveRobotRequest* request,
        amr::simulation::RemoveRobotResponse* response) override;

    grpc::Status SendCommand(
        grpc::ServerContext* context,
        const amr::simulation::RobotCommand* request,
        amr::simulation::CommandResponse* response) override;

    grpc::Status StreamTelemetry(
        grpc::ServerContext* context,
        const amr::simulation::TelemetryRequest* request,
        grpc::ServerWriter<amr::simulation::TelemetryMessage>* writer) override;

    grpc::Status SpawnObject(
        grpc::ServerContext* context,
        const amr::simulation::SpawnObjectRequest* request,
        amr::simulation::SpawnObjectResponse* response) override;

    grpc::Status RemoveObject(
        grpc::ServerContext* context,
        const amr::simulation::RemoveObjectRequest* request,
        amr::simulation::RemoveObjectResponse* response) override;

    grpc::Status GetSimState(
        grpc::ServerContext* context,
        const amr::simulation::GetSimStateRequest* request,
        amr::simulation::SimState* response) override;

    grpc::Status SetRealtimeFactor(
        grpc::ServerContext* context,
        const amr::simulation::RealtimeFactorRequest* request,
        amr::simulation::RealtimeFactorResponse* response) override;

private:
    uint16_t port_;
    std::unique_ptr<grpc::Server> server_;
    std::thread server_thread_;

    // 시뮬레이션 인스턴스 관리 (sim_id → Simulation)
    std::unordered_map<std::string, std::unique_ptr<Simulation>> simulations_;
    std::mutex sim_mutex_;

    // 텔레메트리 스트리머
    TelemetryStreamer telemetry_streamer_;

    // 헬퍼
    Simulation* getSimulation(const std::string& sim_id);
    std::string generateId();
};

} // namespace amr::sim
```

#### 4.5.6 텔레메트리 스트리머

```cpp
// src/grpc/telemetry_streamer.h
#pragma once

#include "simulation.grpc.pb.h"

#include <grpcpp/grpcpp.h>
#include <functional>
#include <mutex>
#include <string>
#include <unordered_map>
#include <vector>

namespace amr::sim {

struct TelemetryPacket; // forward declaration

// 텔레메트리 구독 정보
struct TelemetrySubscription {
    std::string sub_id;
    std::vector<std::string> robot_ids; // 비어있으면 모든 로봇
    std::vector<std::string> sensor_types; // 비어있으면 모든 센서
    double max_rate;
    grpc::ServerWriter<amr::simulation::TelemetryMessage>* writer;
    double last_send_time = 0.0;
};

class TelemetryStreamer {
public:
    TelemetryStreamer() = default;

    // 구독 추가
    std::string addSubscription(
        const std::vector<std::string>& robot_ids,
        const std::vector<std::string>& sensor_types,
        double max_rate,
        grpc::ServerWriter<amr::simulation::TelemetryMessage>* writer);

    // 구독 제거
    void removeSubscription(const std::string& sub_id);

    // 텔레메트리 브로드캐스트 (sim loop에서 호출)
    void broadcast(const std::vector<TelemetryPacket>& packets);

    // 활성 구독 수
    size_t activeSubscriptionCount() const;

private:
    amr::simulation::TelemetryMessage convertToProto(
        const TelemetryPacket& packet);

    bool matchesFilter(
        const TelemetrySubscription& sub,
        const TelemetryPacket& packet);

    std::unordered_map<std::string, TelemetrySubscription> subscriptions_;
    mutable std::mutex mutex_;
    uint64_t next_sub_id_ = 0;
};

} // namespace amr::sim
```

---

### 4.6 Python Bindings

#### 4.6.1 pybind11 모듈 정의

```cpp
// src/python/bindings.cpp
#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include <pybind11/eigen.h>
#include <pybind11/functional.h>

#include "core/simulation.h"
#include "core/config.h"
#include "robots/robot_state.h"
#include "sensors/sensor.h"

namespace py = pybind11;
using namespace amr::sim;

PYBIND11_MODULE(amr_sim, m) {
    m.doc() = "AMR Simulation Engine Python Bindings";

    // === Config 클래스 ===
    py::class_<PhysicsConfig>(m, "PhysicsConfig")
        .def(py::init<>())
        .def_readwrite("fixed_timestep", &PhysicsConfig::fixed_timestep)
        .def_readwrite("gravity_x", &PhysicsConfig::gravity_x)
        .def_readwrite("gravity_y", &PhysicsConfig::gravity_y)
        .def_readwrite("gravity_z", &PhysicsConfig::gravity_z)
        .def_readwrite("default_friction", &PhysicsConfig::default_friction)
        .def_readwrite("default_restitution", &PhysicsConfig::default_restitution);

    py::class_<SimulationConfig>(m, "SimulationConfig")
        .def(py::init<>())
        .def_readwrite("physics", &SimulationConfig::physics)
        .def_readwrite("realtime_factor", &SimulationConfig::realtime_factor)
        .def_readwrite("grpc_port", &SimulationConfig::grpc_port)
        .def_readwrite("log_level", &SimulationConfig::log_level)
        .def_readwrite("enable_rendering", &SimulationConfig::enable_rendering);

    py::class_<RobotConfig>(m, "RobotConfig")
        .def(py::init<>())
        .def_readwrite("model_path", &RobotConfig::model_path)
        .def_readwrite("robot_id", &RobotConfig::robot_id)
        .def_readwrite("initial_x", &RobotConfig::initial_x)
        .def_readwrite("initial_y", &RobotConfig::initial_y)
        .def_readwrite("initial_z", &RobotConfig::initial_z)
        .def_readwrite("initial_yaw", &RobotConfig::initial_yaw)
        .def_readwrite("initial_battery", &RobotConfig::initial_battery)
        .def_readwrite("drive_type", &RobotConfig::drive_type);

    // === RobotState ===
    py::class_<BatteryState>(m, "BatteryState")
        .def(py::init<>())
        .def_readwrite("percentage", &BatteryState::percentage)
        .def_readwrite("voltage", &BatteryState::voltage)
        .def_readwrite("charging", &BatteryState::charging);

    py::class_<RobotState>(m, "RobotState")
        .def(py::init<>())
        .def_readwrite("robot_id", &RobotState::robot_id)
        .def_readwrite("position", &RobotState::position)
        .def_readwrite("orientation", &RobotState::orientation)
        .def_readwrite("linear_velocity", &RobotState::linear_velocity)
        .def_readwrite("angular_velocity", &RobotState::angular_velocity)
        .def_readwrite("battery", &RobotState::battery)
        .def_readwrite("odometry_distance", &RobotState::odometry_distance)
        .def_readwrite("timestamp", &RobotState::timestamp)
        .def("yaw", &RobotState::yaw)
        .def("speed", &RobotState::speed);

    // === Simulation ===
    py::class_<Simulation>(m, "Simulation")
        .def(py::init<>())
        .def(py::init<const SimulationConfig&>())
        .def("load_world", &Simulation::loadWorld,
             py::arg("sdf_path"),
             "USD 월드 파일을 로드합니다 (SDF/URDF 임포트 지원)")
        .def("spawn_robot",
             [](Simulation& sim, const std::string& model_path,
                std::vector<double> pose, const std::string& drive_type) {
                 RobotConfig config;
                 config.model_path = model_path;
                 if (pose.size() >= 3) {
                     config.initial_x = pose[0];
                     config.initial_y = pose[1];
                     config.initial_z = pose[2];
                 }
                 if (pose.size() >= 6) {
                     config.initial_yaw = pose[5]; // [x, y, z, roll, pitch, yaw]
                 }
                 config.drive_type = drive_type;
                 return sim.spawnRobot(config);
             },
             py::arg("model_path"),
             py::arg("pose") = std::vector<double>{0, 0, 0, 0, 0, 0},
             py::arg("drive_type") = "differential",
             "로봇을 스폰합니다. pose=[x,y,z,roll,pitch,yaw]")
        .def("remove_robot", &Simulation::removeRobot,
             py::arg("robot_id"))
        .def("send_command",
             [](Simulation& sim, const std::string& robot_id,
                double linear, double angular, double linear_y) {
                 VelocityCommand cmd;
                 cmd.robot_id = robot_id;
                 cmd.linear_x = linear;
                 cmd.linear_y = linear_y;
                 cmd.angular_z = angular;
                 sim.sendCommand(robot_id, cmd);
             },
             py::arg("robot_id"),
             py::arg("linear") = 0.0,
             py::arg("angular") = 0.0,
             py::arg("linear_y") = 0.0,
             "로봇에 속도 명령을 전송합니다")
        .def("step", &Simulation::step,
             "시뮬레이션을 1 스텝 진행합니다")
        .def("start", &Simulation::start,
             py::arg("realtime_factor") = 1.0,
             "시뮬레이션을 시작합니다 (별도 스레드)")
        .def("stop", &Simulation::stop,
             "시뮬레이션을 정지합니다")
        .def("reset", &Simulation::reset,
             "시뮬레이션을 리셋합니다")
        .def("get_state", &Simulation::getState,
             "현재 시뮬레이션 상태를 반환합니다")
        .def("get_robot_state", &Simulation::getRobotState,
             py::arg("robot_id"),
             "특정 로봇의 상태를 반환합니다")
        .def("get_sensor_data", &Simulation::getSensorData,
             py::arg("robot_id"),
             "특정 로봇의 센서 데이터를 반환합니다")
        .def("set_realtime_factor",
             [](Simulation& sim, double factor) {
                 sim.start(factor);
             },
             py::arg("factor"),
             "리얼타임 팩터를 설정합니다")
        .def("sim_time", &Simulation::simTime,
             "현재 시뮬레이션 시간을 반환합니다 (초)")
        .def("step_count", &Simulation::stepCount,
             "현재 스텝 카운트를 반환합니다")
        .def("shutdown", &Simulation::stop,
             "시뮬레이션을 종료합니다")
        .def_property_readonly("is_running", &Simulation::isRunning);

    // === Sensor Data 접근 ===
    m.def("get_lidar_ranges",
          [](const SensorData& data) -> py::object {
              if (auto* scan = std::get_if<LidarScanData>(&data)) {
                  return py::cast(scan->ranges);
              }
              return py::none();
          },
          "LiDAR 스캔의 거리 데이터를 반환합니다");

    m.def("get_imu_data",
          [](const SensorData& data) -> py::object {
              if (auto* imu = std::get_if<ImuData>(&data)) {
                  return py::make_tuple(
                      imu->linear_acceleration,
                      imu->angular_velocity);
              }
              return py::none();
          },
          "IMU 데이터를 반환합니다 (linear_accel, angular_vel)");
}
```

#### 4.6.2 Python 사용 예시

```python
# scenarios/warehouse_basic.py
"""기본 창고 시나리오: 단일 로봇이 창고 내에서 주행"""

import amr_sim
import math
import time

def main():
    # 시뮬레이션 생성
    config = amr_sim.SimulationConfig()
    config.physics.fixed_timestep = 0.001
    config.realtime_factor = 0.0  # 최대 속도
    config.enable_rendering = False

    sim = amr_sim.Simulation(config)

    # 월드 로드
    sim.load_world("models/warehouse.usda")
    print("World loaded")

    # 로봇 스폰
    robot_id = sim.spawn_robot(
        model_path="models/simple_robot.usda",
        pose=[1.0, 1.0, 0.0, 0.0, 0.0, 0.0],  # x, y, z, roll, pitch, yaw
        drive_type="differential"
    )
    print(f"Robot spawned: {robot_id}")

    # 직선 주행 테스트 (5초간)
    steps_per_second = int(1.0 / config.physics.fixed_timestep)
    total_steps = steps_per_second * 5  # 5초

    print("Starting straight drive test...")
    for step in range(total_steps):
        sim.send_command(robot_id, linear=0.5, angular=0.0)
        sim.step()

        if step % steps_per_second == 0:
            state = sim.get_robot_state(robot_id)
            print(f"  t={state.timestamp:.1f}s: "
                  f"pos=({state.position[0]:.2f}, {state.position[1]:.2f}), "
                  f"yaw={state.yaw():.2f}rad, "
                  f"speed={state.speed():.2f}m/s, "
                  f"battery={state.battery.percentage:.1f}%")

    # 회전 주행 테스트 (원 그리기, 10초)
    print("\nStarting circular drive test...")
    total_steps = steps_per_second * 10

    for step in range(total_steps):
        sim.send_command(robot_id, linear=0.3, angular=0.5)
        sim.step()

        if step % (steps_per_second * 2) == 0:
            state = sim.get_robot_state(robot_id)
            print(f"  t={state.timestamp:.1f}s: "
                  f"pos=({state.position[0]:.2f}, {state.position[1]:.2f}), "
                  f"distance={state.odometry_distance:.2f}m")

    # 최종 상태
    final_state = sim.get_robot_state(robot_id)
    print(f"\nFinal state:")
    print(f"  Position: ({final_state.position[0]:.3f}, {final_state.position[1]:.3f})")
    print(f"  Total distance: {final_state.odometry_distance:.3f}m")
    print(f"  Battery: {final_state.battery.percentage:.1f}%")
    print(f"  Steps: {sim.step_count()}")

    sim.shutdown()
    print("Done")

if __name__ == "__main__":
    main()
```

```python
# scenarios/multi_robot_traffic.py
"""다중 로봇 교통 시나리오: 여러 로봇이 교차 경로를 주행"""

import amr_sim
import math

def main():
    sim = amr_sim.Simulation()
    sim.load_world("models/warehouse.usda")

    # 4대의 로봇 스폰 (각 모서리에서 출발)
    robots = []
    spawn_configs = [
        {"pose": [1, 1, 0, 0, 0, 0], "target": [9, 9]},
        {"pose": [9, 1, 0, 0, 0, math.pi/2], "target": [1, 9]},
        {"pose": [9, 9, 0, 0, 0, math.pi], "target": [1, 1]},
        {"pose": [1, 9, 0, 0, 0, -math.pi/2], "target": [9, 1]},
    ]

    for i, cfg in enumerate(spawn_configs):
        rid = sim.spawn_robot(
            "models/simple_robot.usda",
            pose=cfg["pose"]
        )
        robots.append({"id": rid, "target": cfg["target"]})
        print(f"Spawned robot {rid} at {cfg['pose'][:2]} → target {cfg['target']}")

    # 단순 P 제어로 목표점을 향해 주행
    dt = 0.001
    for step in range(30000):  # 30초
        for r in robots:
            state = sim.get_robot_state(r["id"])
            dx = r["target"][0] - state.position[0]
            dy = r["target"][1] - state.position[1]
            dist = math.sqrt(dx*dx + dy*dy)

            if dist < 0.1:
                sim.send_command(r["id"], linear=0.0, angular=0.0)
                continue

            target_yaw = math.atan2(dy, dx)
            yaw_error = target_yaw - state.yaw()
            # 각도 정규화
            while yaw_error > math.pi: yaw_error -= 2*math.pi
            while yaw_error < -math.pi: yaw_error += 2*math.pi

            linear = min(0.5, dist * 0.5)
            angular = max(-1.0, min(1.0, yaw_error * 2.0))

            sim.send_command(r["id"], linear=linear, angular=angular)

        sim.step()

        if step % 5000 == 0:
            print(f"\n--- Step {step} ---")
            for r in robots:
                s = sim.get_robot_state(r["id"])
                print(f"  {r['id']}: ({s.position[0]:.2f}, {s.position[1]:.2f})")

    sim.shutdown()

if __name__ == "__main__":
    main()
```

```python
# scenarios/sensor_validation.py
"""센서 검증 시나리오: 알려진 기하학으로 센서 출력 검증"""

import amr_sim
import math
import numpy as np

def validate_lidar():
    """정사각형 방 안에서 LiDAR 출력 검증"""
    sim = amr_sim.Simulation()

    # 10m x 10m 방 (빈 방)
    sim.load_world("models/warehouse.usda")

    # 방 중앙에 로봇 스폰
    robot_id = sim.spawn_robot(
        "models/simple_robot.usda",
        pose=[5.0, 5.0, 0.0, 0.0, 0.0, 0.0]
    )

    # 안정화를 위해 100 스텝 실행
    for _ in range(100):
        sim.step()

    # LiDAR 데이터 읽기
    sensor_data = sim.get_sensor_data(robot_id)
    for data in sensor_data:
        ranges = amr_sim.get_lidar_ranges(data)
        if ranges is not None:
            ranges = np.array(ranges)
            print(f"LiDAR scan: {len(ranges)} beams")
            print(f"  Min range: {np.min(ranges):.3f}m")
            print(f"  Max range: {np.max(ranges):.3f}m")
            print(f"  Mean range: {np.mean(ranges):.3f}m")

            # 중앙에서 벽까지 약 5m이므로, 대부분의 빔이 ~5m여야 함
            valid_ranges = ranges[ranges < 11.0]
            if len(valid_ranges) > 0:
                print(f"  Valid beams: {len(valid_ranges)}/{len(ranges)}")
                print(f"  Expected ~5.0m, got mean={np.mean(valid_ranges):.3f}m")

    sim.shutdown()

def validate_odometry():
    """직선 주행 후 오도메트리 검증"""
    sim = amr_sim.Simulation()
    sim.load_world("models/warehouse.usda")

    robot_id = sim.spawn_robot(
        "models/simple_robot.usda",
        pose=[1.0, 5.0, 0.0, 0.0, 0.0, 0.0]
    )

    # 0.5 m/s로 10초 = 5m 주행
    for _ in range(10000):
        sim.send_command(robot_id, linear=0.5, angular=0.0)
        sim.step()

    state = sim.get_robot_state(robot_id)
    expected_x = 1.0 + 0.5 * 10.0  # 6.0m
    actual_x = state.position[0]
    error = abs(actual_x - expected_x)

    print(f"Odometry validation:")
    print(f"  Expected x={expected_x:.3f}m, Actual x={actual_x:.3f}m")
    print(f"  Error: {error:.4f}m ({error/expected_x*100:.2f}%)")
    print(f"  Odometry distance: {state.odometry_distance:.3f}m (expected ~5.0m)")

    assert error < 0.1, f"Odometry error too large: {error}m"
    print("  PASSED")

    sim.shutdown()

if __name__ == "__main__":
    print("=== LiDAR Validation ===")
    validate_lidar()
    print("\n=== Odometry Validation ===")
    validate_odometry()
```

---

### 4.7 Environment

#### 4.7.1 USD Scene Loader

```cpp
// src/environment/scene_loader.h
#pragma once

#include "core/world.h"
#include "physics/physics_engine.h"

#include <string>
#include <memory>

namespace amr::sim {

// USD 씬 파싱 결과
struct SceneDescription {
    struct ModelDesc {
        std::string name;
        std::string usd_path;          // 외부 모델 참조 (.usd/.usda/.usdc)
        bool is_static = true;

        struct LinkDesc {
            std::string name;

            struct CollisionDesc {
                std::string shape_type; // "box", "cylinder", "sphere", "mesh"
                Eigen::Vector3d size{1, 1, 1};
                std::string mesh_uri;
                double friction = 0.5;
                double restitution = 0.1;
            };
            std::vector<CollisionDesc> collisions;

            Eigen::Vector3d position{0, 0, 0};
            Eigen::Quaterniond orientation{1, 0, 0, 0};
            double mass = 0.0;
        };
        std::vector<LinkDesc> links;

        Eigen::Vector3d position{0, 0, 0};
        Eigen::Quaterniond orientation{1, 0, 0, 0};
    };

    std::string name;
    std::vector<ModelDesc> models;
    Eigen::Vector3d gravity{0, 0, -9.81};
};

class SceneLoader {
public:
    SceneLoader() = default;

    // USD 파일을 파싱하여 SceneDescription 생성 (기본 포맷)
    SceneDescription loadUSD(const std::string& usd_path);

    // SDF/URDF 파일을 USD로 변환 후 로드 (레거시 임포트)
    SceneDescription loadSDF(const std::string& sdf_path);

    // SceneDescription을 World와 PhysicsEngine에 적용
    void applyToWorld(
        const SceneDescription& scene,
        World& world,
        PhysicsEngine& physics);

    // 단일 로봇 USD 파싱
    RobotConfig parseRobotUSD(const std::string& usd_path);

    // SDF/URDF → USD 변환
    std::string convertToUSD(const std::string& sdf_or_urdf_path,
                             const std::string& output_dir = "");

private:
    void parseUsdStage(/* pxr::UsdStageRefPtr */);
    void parseUsdPrim(/* pxr::UsdPrim */);
    void parseUsdPhysics(/* pxr::UsdPhysicsRigidBodyAPI */);
    void parseUsdCollision(/* pxr::UsdPhysicsCollisionAPI */);
};

} // namespace amr::sim
```

#### 4.7.2 Static Object

```cpp
// src/environment/static_object.h
#pragma once

#include "core/entity.h"
#include "physics/rigid_body.h"
#include <memory>
#include <string>

namespace amr::sim {

class StaticObject {
public:
    StaticObject(const std::string& id,
                 const RigidBodyConfig& body_config);

    const std::string& id() const { return id_; }
    RigidBody* body() { return body_.get(); }

    void addToPhysics(PhysicsEngine& physics);
    void removeFromPhysics(PhysicsEngine& physics);

private:
    std::string id_;
    std::unique_ptr<RigidBody> body_;
};

} // namespace amr::sim
```

#### 4.7.3 Dynamic Object

```cpp
// src/environment/dynamic_object.h
#pragma once

#include "core/entity.h"
#include "physics/rigid_body.h"
#include <Eigen/Core>
#include <memory>
#include <string>
#include <vector>

namespace amr::sim {

struct DynamicObjectWaypoint {
    Eigen::Vector3d position;
    double wait_time = 0.0;  // 초
};

class DynamicObject {
public:
    DynamicObject(const std::string& id,
                  const RigidBodyConfig& body_config);

    // 경로 설정
    void setPath(const std::vector<DynamicObjectWaypoint>& waypoints,
                 double speed, bool loop = true);

    // 매 스텝 업데이트 (경로 추적)
    void update(double dt);

    const std::string& id() const { return id_; }
    RigidBody* body() { return body_.get(); }

    void addToPhysics(PhysicsEngine& physics);
    void removeFromPhysics(PhysicsEngine& physics);

private:
    std::string id_;
    std::unique_ptr<RigidBody> body_;

    std::vector<DynamicObjectWaypoint> waypoints_;
    double speed_ = 1.0;
    bool loop_ = true;
    int current_waypoint_idx_ = 0;
    double wait_timer_ = 0.0;
    bool waiting_ = false;
};

} // namespace amr::sim
```

#### 4.7.4 Ground Plane

```cpp
// src/environment/ground_plane.h
#pragma once

#include <Eigen/Core>

namespace amr::sim {

struct GroundPlaneConfig {
    double friction = 0.8;
    double restitution = 0.1;
    Eigen::Vector3d normal{0, 0, 1};   // 위쪽 법선
    double offset = 0.0;               // 원점으로부터 오프셋
    std::string material = "concrete"; // 재질 이름 (로깅/디스플레이용)
};

// 지면은 물리 백엔드의 init()에서 자동 생성됨.
// 이 구조체는 지면 설정을 SDF에서 읽어올 때 사용됨.

} // namespace amr::sim
```

---

### 4.8 Rendering (Vulkan Headless)

#### 4.8.1 Vulkan Context

```cpp
// src/rendering/vulkan_context.h
#pragma once

#include <vulkan/vulkan.h>
#include <memory>
#include <string>
#include <vector>

namespace amr::sim {

class VulkanContext {
public:
    VulkanContext();
    ~VulkanContext();

    // 초기화 (헤드리스 모드)
    bool initHeadless();

    // 디버그 뷰포트 포함 초기화 (GLFW)
    bool initWithWindow(uint32_t width, uint32_t height, const std::string& title);

    // 정리
    void cleanup();

    // Vulkan 핸들 접근
    VkInstance instance() const { return instance_; }
    VkPhysicalDevice physicalDevice() const { return physical_device_; }
    VkDevice device() const { return device_; }
    VkQueue graphicsQueue() const { return graphics_queue_; }
    uint32_t graphicsQueueFamily() const { return graphics_queue_family_; }
    VkCommandPool commandPool() const { return command_pool_; }

    // 메모리 타입 검색
    uint32_t findMemoryType(uint32_t type_filter,
                            VkMemoryPropertyFlags properties) const;

    // 일회성 커맨드 버퍼
    VkCommandBuffer beginSingleTimeCommands();
    void endSingleTimeCommands(VkCommandBuffer cmd);

    bool isInitialized() const { return initialized_; }

private:
    bool createInstance();
    bool pickPhysicalDevice();
    bool createLogicalDevice();
    bool createCommandPool();

    VkInstance instance_ = VK_NULL_HANDLE;
    VkPhysicalDevice physical_device_ = VK_NULL_HANDLE;
    VkDevice device_ = VK_NULL_HANDLE;
    VkQueue graphics_queue_ = VK_NULL_HANDLE;
    uint32_t graphics_queue_family_ = 0;
    VkCommandPool command_pool_ = VK_NULL_HANDLE;

    bool initialized_ = false;
    bool headless_ = true;
};

} // namespace amr::sim
```

#### 4.8.2 Framebuffer (Offscreen Rendering)

```cpp
// src/rendering/framebuffer.h
#pragma once

#include "rendering/vulkan_context.h"
#include <cstdint>
#include <vector>

namespace amr::sim {

struct FramebufferConfig {
    uint32_t width = 640;
    uint32_t height = 480;
    VkFormat color_format = VK_FORMAT_R8G8B8A8_UNORM;
    VkFormat depth_format = VK_FORMAT_D32_SFLOAT;
    bool enable_depth = true;
};

class Framebuffer {
public:
    Framebuffer(std::shared_ptr<VulkanContext> context,
                const FramebufferConfig& config);
    ~Framebuffer();

    // 이미지 데이터 읽기 (CPU로 전송)
    std::vector<uint8_t> readColorImage();
    std::vector<float> readDepthImage();

    VkFramebuffer framebuffer() const { return framebuffer_; }
    VkRenderPass renderPass() const { return render_pass_; }
    uint32_t width() const { return config_.width; }
    uint32_t height() const { return config_.height; }

private:
    void create();
    void cleanup();

    std::shared_ptr<VulkanContext> context_;
    FramebufferConfig config_;

    VkImage color_image_ = VK_NULL_HANDLE;
    VkDeviceMemory color_memory_ = VK_NULL_HANDLE;
    VkImageView color_view_ = VK_NULL_HANDLE;

    VkImage depth_image_ = VK_NULL_HANDLE;
    VkDeviceMemory depth_memory_ = VK_NULL_HANDLE;
    VkImageView depth_view_ = VK_NULL_HANDLE;

    VkRenderPass render_pass_ = VK_NULL_HANDLE;
    VkFramebuffer framebuffer_ = VK_NULL_HANDLE;
};

} // namespace amr::sim
```

---

## 5. 핵심 C++ 클래스/인터페이스 종합

### 5.1 Simulation (최상위 클래스)

```cpp
// src/core/simulation.h
#pragma once

#include "core/config.h"
#include "core/sim_loop.h"
#include "core/world.h"
#include "physics/physics_engine.h"
#include "environment/scene_loader.h"
#include "grpc/sim_server.h"
#include "grpc/command_handler.h"
#include "grpc/telemetry_streamer.h"
#include "rendering/vulkan_context.h"

#include <memory>
#include <string>
#include <vector>

namespace amr::sim {

class Simulation {
public:
    Simulation();
    explicit Simulation(const SimulationConfig& config);
    ~Simulation();

    // 월드 로드 (USD 또는 SDF/URDF - SDF/URDF는 자동으로 USD 변환)
    void loadWorld(const std::string& scene_path);

    // 로봇 관리
    std::string spawnRobot(const RobotConfig& config);
    void removeRobot(const std::string& robot_id);

    // 명령 전송
    void sendCommand(const std::string& robot_id, const VelocityCommand& cmd);

    // 시뮬레이션 제어
    void step();                       // 단일 스텝 (Python 바인딩용)
    void start(double realtime_factor = 1.0); // 연속 실행
    void stop();                       // 정지
    void reset();                      // 리셋

    // 상태 조회
    SimState getState() const;
    RobotState getRobotState(const std::string& robot_id) const;
    std::vector<SensorData> getSensorData(const std::string& robot_id) const;

    // 환경 객체 관리
    std::string spawnObject(const std::string& model_path,
                            const Eigen::Vector3d& position,
                            double yaw = 0.0,
                            bool is_static = true);
    void removeObject(const std::string& object_id);

    // 속성 조회
    bool isRunning() const;
    double simTime() const;
    uint64_t stepCount() const;

private:
    SimulationConfig config_;
    std::unique_ptr<World> world_;
    std::unique_ptr<PhysicsEngine> physics_;
    std::unique_ptr<SimLoop> sim_loop_;
    std::unique_ptr<SceneLoader> scene_loader_;
    std::unique_ptr<CommandHandler> command_handler_;
    std::unique_ptr<TelemetryStreamer> telemetry_streamer_;
    std::shared_ptr<VulkanContext> vulkan_context_;

    bool initialized_ = false;

    void initialize();
};

} // namespace amr::sim
```

### 5.2 Entry Point

```cpp
// src/main.cpp
#include "core/simulation.h"
#include "grpc/sim_server.h"

#include <spdlog/spdlog.h>
#include <spdlog/sinks/stdout_color_sinks.h>

#include <csignal>
#include <iostream>
#include <string>

namespace {
    std::atomic<bool> g_shutdown_requested{false};

    void signalHandler(int signum) {
        spdlog::info("Received signal {}, shutting down...", signum);
        g_shutdown_requested = true;
    }
}

int main(int argc, char* argv[]) {
    // 로깅 초기화
    auto logger = spdlog::stdout_color_mt("sim_engine");
    spdlog::set_default_logger(logger);
    spdlog::set_level(spdlog::level::info);
    spdlog::set_pattern("[%Y-%m-%d %H:%M:%S.%e] [%^%l%$] [%t] %v");

    spdlog::info("AMR Simulation Engine starting...");

    // 시그널 핸들러 등록
    std::signal(SIGINT, signalHandler);
    std::signal(SIGTERM, signalHandler);

    // 설정
    amr::sim::SimulationConfig config;

    // 커맨드라인 인자 처리
    for (int i = 1; i < argc; ++i) {
        std::string arg = argv[i];
        if (arg == "--port" && i + 1 < argc) {
            config.grpc_port = static_cast<uint16_t>(std::stoi(argv[++i]));
        } else if (arg == "--log-level" && i + 1 < argc) {
            config.log_level = argv[++i];
        } else if (arg == "--enable-rendering") {
            config.enable_rendering = true;
        } else if (arg == "--help") {
            std::cout << "Usage: sim_engine [options]\n"
                      << "  --port <port>          gRPC port (default: 50051)\n"
                      << "  --log-level <level>    Log level (default: info)\n"
                      << "  --enable-rendering     Enable Vulkan rendering\n"
                      << "  --help                 Show this help\n";
            return 0;
        }
    }

    // 로그 레벨 설정
    spdlog::set_level(spdlog::level::from_str(config.log_level));

    // gRPC 서버 시작
    amr::sim::SimServer server(config.grpc_port);
    spdlog::info("Starting gRPC server on port {}...", config.grpc_port);
    server.runAsync();

    spdlog::info("Simulation Engine ready. Press Ctrl+C to shutdown.");

    // 종료 대기
    while (!g_shutdown_requested) {
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }

    server.shutdown();
    spdlog::info("Simulation Engine shutdown complete.");
    return 0;
}
```

---

## 6. 성능 요구사항

### 6.1 성능 목표표

| 항목 | 목표 | 측정 조건 |
|------|------|----------|
| **커스텀 경량 엔진 스텝** | < 0.1ms | 100개 로봇, 2D 운동학 전용 |
| **MuJoCo 백엔드 스텝** | < 0.5ms | 10개 로봇, 전체 3D 강체 역학 |
| **Isaac Sim 백엔드** | GPU-bound | 별도 프로세스, GPU 가속 물리 |
| **물리 스텝 (기본)** | < 0.5ms | 10개 로봇 + 단순 충돌 형상 (커스텀 백엔드) |
| **2D LiDAR** (360 빔) | < 0.2ms | 단일 스캔 (ray casting) |
| **2D LiDAR** (1440 빔) | < 0.8ms | 고해상도 스캔 |
| **3D LiDAR** (16채널 × 1800 빔) | < 5ms | 단일 프레임 |
| **RGB 카메라** (640×480) | < 5ms | Vulkan 오프스크린 렌더링 |
| **깊이 카메라** (640×480) | < 3ms | 뎁스 버퍼 읽기 |
| **IMU** | < 0.01ms | 단일 업데이트 |
| **엔코더** | < 0.01ms | 단일 업데이트 |
| **전체 루프** (10 로봇, 모든 센서) | < 5ms | → 200Hz 이상 지원 |
| **gRPC 텔레메트리 전송** | < 1ms | 메시지 직렬화 + 네트워크 전송 |
| **메모리** | < 500MB | 창고 씬 + 10개 로봇 |
| **시작 시간** | < 3초 | 월드 로드 + 물리 초기화 |

### 6.2 성능 최적화 전략

1. **배치 레이캐스트**: 개별 `rayTest()` 대신 빔 배열을 한 번에 처리. 향후 GPU 가속 레이캐스트(Vulkan compute shader) 도입 가능.
2. **센서 스레드 풀**: 복수 로봇의 센서 업데이트를 병렬 처리 (`std::async` 또는 전용 스레드 풀).
3. **메모리 사전 할당**: 센서 데이터 버퍼를 매 프레임 할당하지 않고 재사용.
4. **Protobuf Arena 할당**: gRPC 메시지 직렬화 시 `google::protobuf::Arena` 사용.
5. **충돌 형상 단순화**: 복잡한 메시 대신 convex decomposition 또는 단순 primitive 사용.
6. **프로파일링**: Google Benchmark + Tracy 기반으로 주기적 성능 측정 및 회귀 감지.

### 6.3 벤치마크 코드

```cpp
// tests/benchmarks/bench_physics.cpp
#include <benchmark/benchmark.h>
#include "physics/custom_backend.h"
#include "physics/rigid_body.h"
#include "core/config.h"

using namespace amr::sim;

static void BM_CustomBackendStep_Empty(benchmark::State& state) {
    CustomLightweightBackend physics;
    PhysicsConfig config;
    config.backend = "custom";
    physics.init(config);

    for (auto _ : state) {
        physics.step(0.001);
    }
}
BENCHMARK(BM_CustomBackendStep_Empty);

static void BM_CustomBackendStep_NRobots(benchmark::State& state) {
    int num_robots = state.range(0);
    CustomLightweightBackend physics;
    PhysicsConfig config;
    config.backend = "custom";
    config.max_robots = num_robots + 10;
    physics.init(config);

    std::vector<std::unique_ptr<RigidBody>> bodies;
    for (int i = 0; i < num_robots; ++i) {
        RigidBodyConfig bcfg;
        bcfg.id = "robot_" + std::to_string(i);
        bcfg.shape = CollisionShape::Box;
        bcfg.dimensions = Eigen::Vector3d(0.25, 0.15, 0.1);
        bcfg.mass = 10.0;
        bcfg.initial_position = Eigen::Vector3d(i * 2.0, 0, 0.1);
        auto body = std::make_unique<RigidBody>(bcfg);
        physics.addBody(body.get());
        bodies.push_back(std::move(body));
    }

    for (auto _ : state) {
        physics.step(0.001);
    }

    state.SetItemsProcessed(state.iterations());
}
BENCHMARK(BM_CustomBackendStep_NRobots)->Arg(1)->Arg(10)->Arg(50)->Arg(100)->Arg(200);

static void BM_Raycast_NRays(benchmark::State& state) {
    int num_rays = state.range(0);
    CustomLightweightBackend physics;
    PhysicsConfig config;
    config.backend = "custom";
    physics.init(config);

    // 바닥에 박스 추가
    RigidBodyConfig bcfg;
    bcfg.id = "wall";
    bcfg.shape = CollisionShape::Box;
    bcfg.dimensions = Eigen::Vector3d(5, 5, 1);
    bcfg.mass = 0.0; // static
    bcfg.initial_position = Eigen::Vector3d(5, 0, 1);
    auto wall = std::make_unique<RigidBody>(bcfg);
    physics.addBody(wall.get());

    // 레이 생성
    std::vector<Ray> rays;
    rays.reserve(num_rays);
    for (int i = 0; i < num_rays; ++i) {
        double angle = -M_PI + 2.0 * M_PI * i / num_rays;
        rays.push_back({
            Eigen::Vector3d(0, 0, 0.5),
            Eigen::Vector3d(std::cos(angle) * 12.0,
                           std::sin(angle) * 12.0, 0.5)
        });
    }

    for (auto _ : state) {
        auto results = physics.batchRaycast(rays);
        benchmark::DoNotOptimize(results);
    }

    state.SetItemsProcessed(state.iterations() * num_rays);
}
BENCHMARK(BM_Raycast_NRays)->Arg(360)->Arg(720)->Arg(1440)->Arg(2880);

BENCHMARK_MAIN();
```

---

## 7. 개발 계획

> 📋 Phase별 상세 일정 및 팀별 할당: [`development-plan.md`](../development-plan.md) 참조
>
> 본 팀의 기능별 상세 스펙은 아래 기능 문서 참조:
> - C-XX: `docs/features/core/C-XX-*.md`
> - A-XX: `docs/features/advanced/A-XX-*.md`

---

## 8. 테스트 계획

### 8.1 단위 테스트 (Google Test)

#### 8.1.1 운동학 테스트

```cpp
// tests/unit/test_differential_drive.cpp
#include <gtest/gtest.h>
#include "robots/differential_drive.h"

using namespace amr::sim;

class DifferentialDriveTest : public ::testing::Test {
protected:
    DifferentialDriveParams params;
    std::unique_ptr<DifferentialDrive> drive;

    void SetUp() override {
        params.wheel_radius = 0.05;
        params.wheel_separation = 0.3;
        params.max_linear_speed = 1.0;
        params.max_angular_speed = 2.0;
        params.max_linear_accel = 100.0; // 가속도 제한 무효화 (즉시 반응)
        params.max_angular_accel = 100.0;
        drive = std::make_unique<DifferentialDrive>(params);
    }
};

// 직진: v=1.0, ω=0.0 → 양쪽 바퀴 동일 속도
TEST_F(DifferentialDriveTest, StraightForward) {
    VelocityCommand cmd;
    cmd.linear_x = 1.0;
    cmd.angular_z = 0.0;

    auto wv = drive->computeWheelVelocities(cmd, 0.001);

    double expected = 1.0 / params.wheel_radius; // 20 rad/s
    EXPECT_NEAR(wv.left, expected, 1e-6);
    EXPECT_NEAR(wv.right, expected, 1e-6);
}

// 제자리 회전: v=0.0, ω=1.0 → 좌우 반대 방향
TEST_F(DifferentialDriveTest, PureSpin) {
    VelocityCommand cmd;
    cmd.linear_x = 0.0;
    cmd.angular_z = 1.0;

    auto wv = drive->computeWheelVelocities(cmd, 0.001);

    double half_sep = params.wheel_separation / 2.0;
    double expected_left = -1.0 * half_sep / params.wheel_radius;
    double expected_right = 1.0 * half_sep / params.wheel_radius;

    EXPECT_NEAR(wv.left, expected_left, 1e-6);
    EXPECT_NEAR(wv.right, expected_right, 1e-6);
}

// 순운동학 역검증: FK(IK(v, ω)) ≈ (v, ω)
TEST_F(DifferentialDriveTest, ForwardInverseConsistency) {
    VelocityCommand cmd;
    cmd.linear_x = 0.5;
    cmd.angular_z = 0.3;

    auto wv = drive->computeWheelVelocities(cmd, 0.001);
    auto est = drive->forwardKinematics(wv.left, wv.right);

    EXPECT_NEAR(est.linear_x, 0.5, 1e-6);
    EXPECT_NEAR(est.angular_z, 0.3, 1e-6);
}

// 속도 제한 테스트
TEST_F(DifferentialDriveTest, SpeedClamping) {
    VelocityCommand cmd;
    cmd.linear_x = 5.0;  // 최대 1.0
    cmd.angular_z = 0.0;

    auto wv = drive->computeWheelVelocities(cmd, 0.001);

    double expected = params.max_linear_speed / params.wheel_radius;
    EXPECT_NEAR(wv.left, expected, 1e-6);
    EXPECT_NEAR(wv.right, expected, 1e-6);
}

// 정지: v=0, ω=0 → 바퀴 속도 0
TEST_F(DifferentialDriveTest, Stop) {
    VelocityCommand cmd;
    cmd.linear_x = 0.0;
    cmd.angular_z = 0.0;

    auto wv = drive->computeWheelVelocities(cmd, 0.001);

    EXPECT_NEAR(wv.left, 0.0, 1e-6);
    EXPECT_NEAR(wv.right, 0.0, 1e-6);
}
```

#### 8.1.2 물리 엔진 테스트

```cpp
// tests/unit/test_collision.cpp
#include <gtest/gtest.h>
#include "physics/custom_backend.h"
#include "physics/rigid_body.h"

using namespace amr::sim;

class PhysicsTest : public ::testing::Test {
protected:
    CustomLightweightBackend physics;

    void SetUp() override {
        PhysicsConfig config;
        config.gravity_z = -9.81;
        physics.init(config);
    }

    void TearDown() override {
        physics.shutdown();
    }
};

// 중력 낙하 테스트
TEST_F(PhysicsTest, GravityFall) {
    RigidBodyConfig bcfg;
    bcfg.id = "falling_box";
    bcfg.shape = CollisionShape::Box;
    bcfg.dimensions = Eigen::Vector3d(0.1, 0.1, 0.1);
    bcfg.mass = 1.0;
    bcfg.initial_position = Eigen::Vector3d(0, 0, 5.0);

    RigidBody body(bcfg);
    physics.addBody(&body);

    // 1초간 시뮬레이션 (1000 스텝)
    for (int i = 0; i < 1000; ++i) {
        physics.step(0.001);
    }

    // 1초 후 z 위치: z = 5.0 - 0.5 * 9.81 * 1.0^2 ≈ 0.095
    // 지면 충돌로 인해 z ≈ 0.1 (박스 반높이)
    auto pos = body.position();
    EXPECT_NEAR(pos.z(), 0.1, 0.05); // 지면 위에 안착
    EXPECT_NEAR(pos.x(), 0.0, 1e-6); // x, y 변화 없음
    EXPECT_NEAR(pos.y(), 0.0, 1e-6);
}

// 레이캐스트 테스트
TEST_F(PhysicsTest, RaycastHit) {
    // 원점에서 x=5 위치에 벽 설치
    RigidBodyConfig bcfg;
    bcfg.id = "wall";
    bcfg.shape = CollisionShape::Box;
    bcfg.dimensions = Eigen::Vector3d(0.1, 5.0, 2.0);
    bcfg.mass = 0.0; // static
    bcfg.initial_position = Eigen::Vector3d(5.0, 0, 1.0);

    RigidBody wall(bcfg);
    physics.addBody(&wall);
    physics.step(0.001); // 한 번 스텝하여 내부 상태 갱신

    // 원점에서 x 방향으로 레이캐스트
    auto result = physics.raycast(
        Eigen::Vector3d(0, 0, 1.0),
        Eigen::Vector3d(10.0, 0, 1.0));

    EXPECT_TRUE(result.hit);
    EXPECT_NEAR(result.hit_point.x(), 4.9, 0.1); // 벽 표면
    EXPECT_NEAR(result.distance, 4.9, 0.1);
}

// 레이캐스트 미스 테스트
TEST_F(PhysicsTest, RaycastMiss) {
    auto result = physics.raycast(
        Eigen::Vector3d(0, 0, 5.0),
        Eigen::Vector3d(10.0, 0, 5.0));

    EXPECT_FALSE(result.hit);
}
```

#### 8.1.3 LiDAR 테스트

```cpp
// tests/unit/test_lidar_2d.cpp
#include <gtest/gtest.h>
#include "sensors/lidar_2d.h"
#include "physics/custom_backend.h"

using namespace amr::sim;

class Lidar2DTest : public ::testing::Test {
protected:
    CustomLightweightBackend physics;
    std::unique_ptr<Lidar2D> lidar;

    void SetUp() override {
        PhysicsConfig pcfg;
        physics.init(pcfg);

        // 원점 주위에 4면 벽 생성 (각 5m 거리)
        createWall("wall_px", Eigen::Vector3d(5.0, 0, 1), Eigen::Vector3d(0.1, 10, 2));
        createWall("wall_nx", Eigen::Vector3d(-5.0, 0, 1), Eigen::Vector3d(0.1, 10, 2));
        createWall("wall_py", Eigen::Vector3d(0, 5.0, 1), Eigen::Vector3d(10, 0.1, 2));
        createWall("wall_ny", Eigen::Vector3d(0, -5.0, 1), Eigen::Vector3d(10, 0.1, 2));

        physics.step(0.001);

        // LiDAR 설정
        Lidar2DConfig lcfg;
        lcfg.name = "test_lidar";
        lcfg.type = "lidar_2d";
        lcfg.update_rate = 1000.0; // 매 스텝 업데이트
        lcfg.angle_min = -M_PI;
        lcfg.angle_max = M_PI;
        lcfg.angle_increment = M_PI / 180.0; // 1도 간격 = 360 빔
        lcfg.range_min = 0.1;
        lcfg.range_max = 12.0;
        lcfg.noise_stddev = 0.0; // 노이즈 없음 (정확도 테스트)
        lidar = std::make_unique<Lidar2D>(lcfg);
    }

    std::vector<std::unique_ptr<RigidBody>> walls_;

    void createWall(const std::string& id, Eigen::Vector3d pos, Eigen::Vector3d size) {
        RigidBodyConfig bcfg;
        bcfg.id = id;
        bcfg.shape = CollisionShape::Box;
        bcfg.dimensions = size;
        bcfg.mass = 0.0;
        bcfg.initial_position = pos;
        auto body = std::make_unique<RigidBody>(bcfg);
        physics.addBody(body.get());
        walls_.push_back(std::move(body));
    }
};

// 원점에서 스캔: 모든 빔이 약 5m 거리의 벽에 맞아야 함
TEST_F(Lidar2DTest, SquareRoomScan) {
    Eigen::Isometry3d pose = Eigen::Isometry3d::Identity();
    pose.translate(Eigen::Vector3d(0, 0, 1.0));

    lidar->update(0.001, pose, physics);

    auto data = lidar->getData();
    auto& scan = std::get<LidarScanData>(data);

    // 모든 빔이 유효한 범위여야 함
    for (size_t i = 0; i < scan.ranges.size(); ++i) {
        EXPECT_GE(scan.ranges[i], 4.0f) << "Beam " << i << " too short";
        EXPECT_LE(scan.ranges[i], 8.0f) << "Beam " << i << " too long";
        // 대각선 최대: sqrt(5^2 + 5^2) ≈ 7.07m
    }

    // 0도 빔 (x 양방향) ≈ 5m
    int beam_0 = scan.ranges.size() / 2; // angle ≈ 0
    EXPECT_NEAR(scan.ranges[beam_0], 5.0f, 0.2f);
}
```

#### 8.1.4 노이즈 모델 테스트

```cpp
// tests/unit/test_noise_model.cpp
#include <gtest/gtest.h>
#include "sensors/noise_model.h"
#include <cmath>
#include <numeric>

using namespace amr::sim;

TEST(NoiseModelTest, GaussianMeanAndStddev) {
    double target_mean = 0.0;
    double target_stddev = 0.01;
    GaussianNoise noise(target_mean, target_stddev, 42);

    const int N = 100000;
    std::vector<double> samples(N);
    for (int i = 0; i < N; ++i) {
        samples[i] = noise.sample();
    }

    double mean = std::accumulate(samples.begin(), samples.end(), 0.0) / N;
    double sq_sum = 0.0;
    for (double s : samples) sq_sum += (s - mean) * (s - mean);
    double stddev = std::sqrt(sq_sum / N);

    EXPECT_NEAR(mean, target_mean, 0.001);
    EXPECT_NEAR(stddev, target_stddev, 0.001);
}

TEST(NoiseModelTest, DeterministicWithSeed) {
    GaussianNoise noise1(0, 1, 12345);
    GaussianNoise noise2(0, 1, 12345);

    for (int i = 0; i < 100; ++i) {
        EXPECT_DOUBLE_EQ(noise1.sample(), noise2.sample());
    }
}
```

#### 8.1.5 시뮬레이션 시계 테스트

```cpp
// tests/unit/test_clock.cpp
#include <gtest/gtest.h>
#include "core/clock.h"

using namespace amr::sim;

TEST(ClockTest, BasicAdvance) {
    SimClock clock;
    clock.reset();

    EXPECT_DOUBLE_EQ(clock.simTime(), 0.0);
    EXPECT_EQ(clock.stepCount(), 0);

    clock.advance(0.001);
    EXPECT_NEAR(clock.simTime(), 0.001, 1e-12);
    EXPECT_EQ(clock.stepCount(), 1);

    for (int i = 0; i < 999; ++i) clock.advance(0.001);
    EXPECT_NEAR(clock.simTime(), 1.0, 1e-9);
    EXPECT_EQ(clock.stepCount(), 1000);
}

TEST(ClockTest, NanosecondConversion) {
    SimClock clock;
    clock.reset();
    clock.advance(1.5);
    EXPECT_EQ(clock.simTimeNs(), 1500000000ULL);
}
```

### 8.2 통합 테스트

```cpp
// tests/integration/test_grpc_server.cpp
#include <gtest/gtest.h>
#include <grpcpp/grpcpp.h>
#include "simulation.grpc.pb.h"
#include "grpc/sim_server.h"
#include <thread>
#include <chrono>

using namespace amr::simulation;

class GrpcIntegrationTest : public ::testing::Test {
protected:
    std::unique_ptr<amr::sim::SimServer> server;
    std::unique_ptr<SimulationService::Stub> stub;

    void SetUp() override {
        server = std::make_unique<amr::sim::SimServer>(50099);
        server->runAsync();
        std::this_thread::sleep_for(std::chrono::milliseconds(500));

        auto channel = grpc::CreateChannel(
            "localhost:50099", grpc::InsecureChannelCredentials());
        stub = SimulationService::NewStub(channel);
    }

    void TearDown() override {
        server->shutdown();
    }
};

TEST_F(GrpcIntegrationTest, CreateAndSpawnRobot) {
    // 시뮬레이션 생성
    grpc::ClientContext ctx1;
    CreateSimRequest create_req;
    create_req.set_world_file("models/warehouse.usda");
    CreateSimResponse create_resp;
    auto status = stub->CreateSimulation(&ctx1, create_req, &create_resp);
    ASSERT_TRUE(status.ok());
    ASSERT_TRUE(create_resp.success());
    std::string sim_id = create_resp.sim_id();

    // 로봇 스폰
    grpc::ClientContext ctx2;
    SpawnRobotRequest spawn_req;
    spawn_req.set_sim_id(sim_id);
    spawn_req.set_model_path("models/simple_robot.usda");
    spawn_req.set_drive_type("differential");
    auto* pose = spawn_req.mutable_initial_pose();
    pose->mutable_position()->set_x(1.0);
    pose->mutable_position()->set_y(2.0);
    SpawnRobotResponse spawn_resp;
    status = stub->SpawnRobot(&ctx2, spawn_req, &spawn_resp);
    ASSERT_TRUE(status.ok());
    ASSERT_TRUE(spawn_resp.success());
    std::string robot_id = spawn_resp.robot_id();

    // 명령 전송
    grpc::ClientContext ctx3;
    RobotCommand cmd;
    cmd.set_sim_id(sim_id);
    cmd.set_robot_id(robot_id);
    auto* vel = cmd.mutable_velocity();
    vel->set_linear_x(0.5);
    vel->set_angular_z(0.0);
    CommandResponse cmd_resp;
    status = stub->SendCommand(&ctx3, cmd, &cmd_resp);
    ASSERT_TRUE(status.ok());
    ASSERT_TRUE(cmd_resp.success());

    // 상태 확인
    grpc::ClientContext ctx4;
    GetSimStateRequest state_req;
    state_req.set_sim_id(sim_id);
    SimState state;
    status = stub->GetSimState(&ctx4, state_req, &state);
    ASSERT_TRUE(status.ok());
    EXPECT_EQ(state.robot_count(), 1);
}
```

### 8.3 벤치마크 (Google Benchmark)

벤치마크 코드는 섹션 6.3에서 이미 정의됨. 추가로:

```cpp
// tests/benchmarks/bench_full_loop.cpp
#include <benchmark/benchmark.h>
#include "core/simulation.h"

using namespace amr::sim;

static void BM_FullLoop_1Robot(benchmark::State& state) {
    SimulationConfig config;
    config.realtime_factor = 0.0; // 최대 속도
    Simulation sim(config);
    sim.loadWorld("models/warehouse.usda");

    RobotConfig rcfg;
    rcfg.model_path = "models/simple_robot.usda";
    rcfg.initial_x = 5.0;
    rcfg.initial_y = 5.0;
    auto rid = sim.spawnRobot(rcfg);

    VelocityCommand cmd;
    cmd.robot_id = rid;
    cmd.linear_x = 0.5;

    for (auto _ : state) {
        sim.sendCommand(rid, cmd);
        sim.step();
    }

    state.SetItemsProcessed(state.iterations());
}
BENCHMARK(BM_FullLoop_1Robot);

static void BM_FullLoop_NRobots(benchmark::State& state) {
    int n = state.range(0);
    SimulationConfig config;
    config.realtime_factor = 0.0;
    Simulation sim(config);
    sim.loadWorld("models/warehouse.usda");

    std::vector<std::string> robot_ids;
    for (int i = 0; i < n; ++i) {
        RobotConfig rcfg;
        rcfg.model_path = "models/simple_robot.usda";
        rcfg.initial_x = 1.0 + i * 2.0;
        rcfg.initial_y = 5.0;
        robot_ids.push_back(sim.spawnRobot(rcfg));
    }

    for (auto _ : state) {
        for (auto& rid : robot_ids) {
            VelocityCommand cmd;
            cmd.robot_id = rid;
            cmd.linear_x = 0.3;
            sim.sendCommand(rid, cmd);
        }
        sim.step();
    }

    state.SetItemsProcessed(state.iterations() * n);
}
BENCHMARK(BM_FullLoop_NRobots)->Arg(1)->Arg(5)->Arg(10)->Arg(20);

BENCHMARK_MAIN();
```

### 8.4 검증 테스트

**분석적 검증 항목**:

| 테스트 | 방법 | 허용 오차 |
|--------|------|----------|
| 직선 주행 오도메트리 | v=0.5m/s × 10s = 5.0m 이동 → x 좌표 확인 | < 1% |
| 원형 주행 반경 | v=0.5, ω=0.5 → 반경 r = v/ω = 1.0m → 궤적 분석 | < 5% |
| LiDAR 사각형 방 | 중앙에서 사방 5m 벽 → 0°빔 = 5.0m, 45°빔 ≈ 7.07m | < 0.1m |
| 중력 낙하 | h=5m 낙하 → t ≈ 1.01s → z ≈ 0 (지면 충돌) | 지면에 안착 |
| IMU 정지 상태 | 정지 로봇 → accel ≈ (0, 0, 9.81), gyro ≈ (0, 0, 0) | 노이즈 범위 내 |

---

## 9. 목 데이터/테스트 에셋

### 9.1 좌표계 규약

**ROS REP 103 준수**:
- **X**: 전방 (Forward)
- **Y**: 좌측 (Left)
- **Z**: 상방 (Up)
- **회전**: 오른손 법칙 (Z축 기준 반시계 = 양수)
- **단위**: 미터 (m), 라디안 (rad), 초 (s)

### 9.2 테스트 모델 및 월드 데이터

> 📋 테스트 월드 데이터: [`test-data-spec.md`](../integration/test-data-spec.md#sim-world-data) 참조

Simple Robot SDF, Warehouse SDF, Box Obstacle SDF 등 레거시 SDF 모델 정의는 위 문서를 참고한다. 실제 사용 시 USD로 변환하여 사용.

---

## 10. 빌드 시스템 상세

### 10.1 CMakeLists.txt

```cmake
# CMakeLists.txt
cmake_minimum_required(VERSION 3.28)
project(sim_engine VERSION 0.1.0 LANGUAGES CXX)

# ===== C++ 표준 =====
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# ===== 빌드 옵션 =====
option(SIM_BUILD_TESTS "Build unit and integration tests" ON)
option(SIM_BUILD_BENCHMARKS "Build benchmarks" ON)
option(SIM_BUILD_PYTHON "Build Python bindings" ON)
option(SIM_ENABLE_RENDERING "Enable Vulkan rendering support" OFF)
option(SIM_ENABLE_SANITIZERS "Enable address/undefined sanitizers" OFF)
option(SIM_ENABLE_MUJOCO "Enable MuJoCo physics backend plugin" OFF)
option(SIM_ENABLE_ISAAC "Enable NVIDIA Isaac Sim bridge plugin" OFF)

# ===== 컴파일러 플래그 =====
add_compile_options(-Wall -Wextra -Wpedantic)
if(SIM_ENABLE_SANITIZERS)
    add_compile_options(-fsanitize=address,undefined)
    add_link_options(-fsanitize=address,undefined)
endif()

# ===== Conan 패키지 =====
find_package(gRPC REQUIRED)
find_package(Protobuf REQUIRED)
find_package(spdlog REQUIRED)
find_package(Eigen3 REQUIRED)
find_package(fmt REQUIRED)
find_package(pxr REQUIRED)  # OpenUSD SDK

if(SIM_ENABLE_MUJOCO)
    find_package(mujoco REQUIRED)
endif()

if(SIM_ENABLE_RENDERING)
    find_package(Vulkan REQUIRED)
    find_package(glfw3 QUIET)
endif()

if(SIM_BUILD_TESTS)
    find_package(GTest REQUIRED)
    enable_testing()
endif()

if(SIM_BUILD_BENCHMARKS)
    find_package(benchmark REQUIRED)
endif()

if(SIM_BUILD_PYTHON)
    find_package(pybind11 REQUIRED)
    find_package(Python3 COMPONENTS Interpreter Development REQUIRED)
endif()

# ===== Protobuf 코드 생성 =====
set(PROTO_DIR ${CMAKE_CURRENT_SOURCE_DIR}/proto)
set(PROTO_FILES
    ${PROTO_DIR}/common.proto
    ${PROTO_DIR}/sensor.proto
    ${PROTO_DIR}/robot.proto
    ${PROTO_DIR}/simulation.proto
)

set(PROTO_GEN_DIR ${CMAKE_CURRENT_BINARY_DIR}/proto_gen)
file(MAKE_DIRECTORY ${PROTO_GEN_DIR})

# protobuf + grpc 코드 생성
set(PROTO_SRCS)
set(PROTO_HDRS)
set(GRPC_SRCS)
set(GRPC_HDRS)

foreach(PROTO_FILE ${PROTO_FILES})
    get_filename_component(PROTO_NAME ${PROTO_FILE} NAME_WE)

    list(APPEND PROTO_SRCS ${PROTO_GEN_DIR}/${PROTO_NAME}.pb.cc)
    list(APPEND PROTO_HDRS ${PROTO_GEN_DIR}/${PROTO_NAME}.pb.h)
    list(APPEND GRPC_SRCS ${PROTO_GEN_DIR}/${PROTO_NAME}.grpc.pb.cc)
    list(APPEND GRPC_HDRS ${PROTO_GEN_DIR}/${PROTO_NAME}.grpc.pb.h)

    add_custom_command(
        OUTPUT
            ${PROTO_GEN_DIR}/${PROTO_NAME}.pb.cc
            ${PROTO_GEN_DIR}/${PROTO_NAME}.pb.h
            ${PROTO_GEN_DIR}/${PROTO_NAME}.grpc.pb.cc
            ${PROTO_GEN_DIR}/${PROTO_NAME}.grpc.pb.h
        COMMAND protobuf::protoc
            --proto_path=${PROTO_DIR}
            --cpp_out=${PROTO_GEN_DIR}
            --grpc_out=${PROTO_GEN_DIR}
            --plugin=protoc-gen-grpc=$<TARGET_FILE:gRPC::grpc_cpp_plugin>
            ${PROTO_FILE}
        DEPENDS ${PROTO_FILE}
        COMMENT "Generating protobuf/gRPC code for ${PROTO_NAME}"
    )
endforeach()

# Proto 라이브러리
add_library(sim_proto STATIC ${PROTO_SRCS} ${GRPC_SRCS})
target_include_directories(sim_proto PUBLIC ${PROTO_GEN_DIR})
target_link_libraries(sim_proto PUBLIC
    protobuf::protobuf
    gRPC::grpc++
)

# ===== 핵심 라이브러리 =====
set(CORE_SOURCES
    src/core/simulation.cpp
    src/core/sim_loop.cpp
    src/core/world.cpp
    src/core/entity.cpp
    src/core/entity_manager.cpp
    src/core/clock.cpp
    src/core/config.cpp
)

set(PHYSICS_SOURCES
    src/physics/custom_backend.cpp
    src/physics/backend_factory.cpp
    src/physics/rigid_body.cpp
    src/physics/collision.cpp
    src/physics/constraints.cpp
)

# MuJoCo 백엔드 (선택적 플러그인)
if(SIM_ENABLE_MUJOCO)
    list(APPEND PHYSICS_SOURCES src/physics/mujoco_backend.cpp)
endif()

# Isaac Sim 브릿지 (선택적 플러그인)
if(SIM_ENABLE_ISAAC)
    list(APPEND PHYSICS_SOURCES src/physics/isaac_bridge.cpp)
endif()

set(ROBOT_SOURCES
    src/robots/robot.cpp
    src/robots/differential_drive.cpp
    src/robots/mecanum_drive.cpp
    src/robots/robot_controller.cpp
    src/robots/robot_state.cpp
)

set(SENSOR_SOURCES
    src/sensors/sensor.cpp
    src/sensors/lidar_2d.cpp
    src/sensors/lidar_3d.cpp
    src/sensors/imu.cpp
    src/sensors/encoder.cpp
    src/sensors/noise_model.cpp
)

set(ENVIRONMENT_SOURCES
    src/environment/scene_loader.cpp
    src/environment/usd_loader.cpp
    src/environment/sdf_importer.cpp
    src/environment/static_object.cpp
    src/environment/dynamic_object.cpp
    src/environment/ground_plane.cpp
)

set(GRPC_SOURCES
    src/grpc/sim_server.cpp
    src/grpc/telemetry_streamer.cpp
    src/grpc/command_handler.cpp
)

# 렌더링 소스 (조건부)
set(RENDERING_SOURCES)
if(SIM_ENABLE_RENDERING)
    list(APPEND RENDERING_SOURCES
        src/rendering/vulkan_context.cpp
        src/rendering/render_pipeline.cpp
        src/rendering/framebuffer.cpp
        src/rendering/viewport.cpp
        src/sensors/camera_rgb.cpp
        src/sensors/camera_depth.cpp
    )
endif()

# sim_engine 정적 라이브러리
add_library(sim_engine_lib STATIC
    ${CORE_SOURCES}
    ${PHYSICS_SOURCES}
    ${ROBOT_SOURCES}
    ${SENSOR_SOURCES}
    ${ENVIRONMENT_SOURCES}
    ${GRPC_SOURCES}
    ${RENDERING_SOURCES}
)

target_include_directories(sim_engine_lib PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

target_link_libraries(sim_engine_lib PUBLIC
    sim_proto
    Eigen3::Eigen
    spdlog::spdlog
    fmt::fmt
    gRPC::grpc++
    protobuf::protobuf
    pxr::usd
    pxr::usdGeom
    pxr::usdPhysics
)

if(SIM_ENABLE_MUJOCO)
    target_compile_definitions(sim_engine_lib PUBLIC SIM_ENABLE_MUJOCO)
    target_link_libraries(sim_engine_lib PUBLIC mujoco::mujoco)
endif()

if(SIM_ENABLE_ISAAC)
    target_compile_definitions(sim_engine_lib PUBLIC SIM_ENABLE_ISAAC)
endif()

if(SIM_ENABLE_RENDERING)
    target_compile_definitions(sim_engine_lib PUBLIC SIM_ENABLE_RENDERING)
    target_link_libraries(sim_engine_lib PUBLIC Vulkan::Vulkan)
    if(TARGET glfw)
        target_link_libraries(sim_engine_lib PUBLIC glfw)
    endif()
endif()

# ===== 실행 파일 =====
add_executable(sim_engine src/main.cpp)
target_link_libraries(sim_engine PRIVATE sim_engine_lib)

# ===== Python 바인딩 =====
if(SIM_BUILD_PYTHON)
    pybind11_add_module(amr_sim
        src/python/bindings.cpp
        src/python/py_simulation.cpp
        src/python/py_scenario.cpp
    )
    target_link_libraries(amr_sim PRIVATE sim_engine_lib)
endif()

# ===== 테스트 =====
if(SIM_BUILD_TESTS)
    # 단위 테스트
    set(UNIT_TESTS
        tests/unit/test_sim_loop.cpp
        tests/unit/test_differential_drive.cpp
        tests/unit/test_mecanum_drive.cpp
        tests/unit/test_lidar_2d.cpp
        tests/unit/test_lidar_3d.cpp
        tests/unit/test_imu.cpp
        tests/unit/test_encoder.cpp
        tests/unit/test_collision.cpp
        tests/unit/test_noise_model.cpp
        tests/unit/test_clock.cpp
        tests/unit/test_entity_manager.cpp
    )

    foreach(TEST_FILE ${UNIT_TESTS})
        get_filename_component(TEST_NAME ${TEST_FILE} NAME_WE)
        add_executable(${TEST_NAME} ${TEST_FILE})
        target_link_libraries(${TEST_NAME} PRIVATE
            sim_engine_lib
            GTest::gtest_main
        )
        add_test(NAME ${TEST_NAME} COMMAND ${TEST_NAME})
    endforeach()

    # 통합 테스트
    add_executable(test_grpc_server tests/integration/test_grpc_server.cpp)
    target_link_libraries(test_grpc_server PRIVATE
        sim_engine_lib
        GTest::gtest_main
    )
    add_test(NAME test_grpc_server COMMAND test_grpc_server)

    add_executable(test_full_scenario tests/integration/test_full_scenario.cpp)
    target_link_libraries(test_full_scenario PRIVATE
        sim_engine_lib
        GTest::gtest_main
    )
    add_test(NAME test_full_scenario COMMAND test_full_scenario)
endif()

# ===== 벤치마크 =====
if(SIM_BUILD_BENCHMARKS)
    add_executable(bench_physics tests/benchmarks/bench_physics.cpp)
    target_link_libraries(bench_physics PRIVATE sim_engine_lib benchmark::benchmark)

    add_executable(bench_lidar tests/benchmarks/bench_lidar.cpp)
    target_link_libraries(bench_lidar PRIVATE sim_engine_lib benchmark::benchmark)

    add_executable(bench_full_loop tests/benchmarks/bench_full_loop.cpp)
    target_link_libraries(bench_full_loop PRIVATE sim_engine_lib benchmark::benchmark)
endif()

# ===== 설치 =====
install(TARGETS sim_engine RUNTIME DESTINATION bin)
install(DIRECTORY include/sim_engine DESTINATION include)
install(DIRECTORY models/ DESTINATION share/sim_engine/models)
install(DIRECTORY scenarios/ DESTINATION share/sim_engine/scenarios)

if(SIM_BUILD_PYTHON)
    install(TARGETS amr_sim LIBRARY DESTINATION ${Python3_SITEARCH})
endif()

# ===== 정보 출력 =====
message(STATUS "=== sim_engine build configuration ===")
message(STATUS "  C++ standard: ${CMAKE_CXX_STANDARD}")
message(STATUS "  Build tests: ${SIM_BUILD_TESTS}")
message(STATUS "  Build benchmarks: ${SIM_BUILD_BENCHMARKS}")
message(STATUS "  Build Python: ${SIM_BUILD_PYTHON}")
message(STATUS "  Enable rendering: ${SIM_ENABLE_RENDERING}")
message(STATUS "  Enable sanitizers: ${SIM_ENABLE_SANITIZERS}")
message(STATUS "  Enable MuJoCo: ${SIM_ENABLE_MUJOCO}")
message(STATUS "  Enable Isaac Sim: ${SIM_ENABLE_ISAAC}")
```

### 10.2 Conan 패키지 설정

```python
# conanfile.py
from conan import ConanFile
from conan.tools.cmake import CMake, CMakeDeps, CMakeToolchain, cmake_layout

class SimEngineConan(ConanFile):
    name = "sim_engine"
    version = "0.1.0"
    settings = "os", "compiler", "build_type", "arch"
    options = {
        "enable_rendering": [True, False],
        "enable_mujoco": [True, False],
        "enable_isaac": [True, False],
        "build_tests": [True, False],
        "build_benchmarks": [True, False],
        "build_python": [True, False],
    }
    default_options = {
        "enable_rendering": False,
        "enable_mujoco": False,
        "enable_isaac": False,
        "build_tests": True,
        "build_benchmarks": True,
        "build_python": True,
    }

    def requirements(self):
        self.requires("grpc/1.60.0")
        self.requires("protobuf/25.0")
        self.requires("spdlog/1.13.0")
        self.requires("eigen/3.4.0")
        self.requires("fmt/10.2.0")

        if self.options.build_tests:
            self.requires("gtest/1.14.0")

        if self.options.build_benchmarks:
            self.requires("benchmark/1.8.3")

        if self.options.build_python:
            self.requires("pybind11/2.12.0")

        if self.options.enable_rendering:
            self.requires("glfw/3.3.9")

        if self.options.enable_mujoco:
            self.requires("mujoco/3.1.0")

        # OpenUSD SDK는 시스템 설치 또는 소스 빌드 (Conan 외부)

    def layout(self):
        cmake_layout(self)

    def generate(self):
        deps = CMakeDeps(self)
        deps.generate()

        tc = CMakeToolchain(self)
        tc.variables["SIM_BUILD_TESTS"] = self.options.build_tests
        tc.variables["SIM_BUILD_BENCHMARKS"] = self.options.build_benchmarks
        tc.variables["SIM_BUILD_PYTHON"] = self.options.build_python
        tc.variables["SIM_ENABLE_RENDERING"] = self.options.enable_rendering
        tc.variables["SIM_ENABLE_MUJOCO"] = self.options.enable_mujoco
        tc.variables["SIM_ENABLE_ISAAC"] = self.options.enable_isaac
        tc.generate()

    def build(self):
        cmake = CMake(self)
        cmake.configure()
        cmake.build()

    def package(self):
        cmake = CMake(self)
        cmake.install()
```

### 10.3 Dockerfile

```dockerfile
# Dockerfile
# === 빌드 스테이지 ===
FROM ubuntu:24.04 AS builder

ENV DEBIAN_FRONTEND=noninteractive

# 시스템 의존성 설치
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    ninja-build \
    git \
    python3 \
    python3-pip \
    python3-dev \
    pkg-config \
    libvulkan-dev \
    vulkan-tools \
    && rm -rf /var/lib/apt/lists/*

# Conan 설치
RUN pip3 install --break-system-packages conan

# Conan 프로파일 설정
RUN conan profile detect

# 소스 복사
WORKDIR /app
COPY conanfile.py .
COPY CMakeLists.txt .
COPY proto/ proto/
COPY src/ src/
COPY include/ include/
COPY tests/ tests/
COPY models/ models/
COPY scenarios/ scenarios/
COPY shaders/ shaders/

# Conan 의존성 설치
RUN conan install . --build=missing -s build_type=Release \
    -o enable_rendering=False -o build_python=True

# CMake 빌드
RUN cmake --preset conan-release \
    -DSIM_ENABLE_RENDERING=OFF \
    -DSIM_BUILD_PYTHON=ON
RUN cmake --build --preset conan-release -j$(nproc)

# 테스트 실행
RUN cd build/Release && ctest --output-on-failure

# === 실행 스테이지 ===
FROM ubuntu:24.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    libstdc++6 \
    && rm -rf /var/lib/apt/lists/*

# 빌드 결과 복사
COPY --from=builder /app/build/Release/sim_engine /usr/local/bin/
COPY --from=builder /app/models/ /usr/local/share/sim_engine/models/
COPY --from=builder /app/scenarios/ /usr/local/share/sim_engine/scenarios/

# Python 모듈 복사 (있는 경우)
COPY --from=builder /app/build/Release/amr_sim*.so /usr/local/lib/python3/dist-packages/ 2>/dev/null || true

# gRPC 포트
EXPOSE 50051

# 기본 설정
ENV SIM_ENGINE_LOG_LEVEL=info
ENV SIM_ENGINE_PORT=50051

ENTRYPOINT ["sim_engine"]
CMD ["--port", "50051", "--log-level", "info"]
```

---

## 11. 다른 팀과의 인터페이스 계약

### 11.1 Backend Team 인터페이스

**통신 방식**: gRPC (Sim Engine = 서버, Backend = 클라이언트)

**계약 사항**:

1. **서비스 엔드포인트**: `SimulationService` (proto/simulation.proto)
2. **텔레메트리 형식**: `TelemetryMessage`는 실제 로봇의 텔레메트리와 **완전히 동일**해야 한다.
   - `robot_state.pose`: 월드 좌표계 (X-forward, Y-left, Z-up)
   - `robot_state.velocity`: 로봇 로컬 좌표계
   - `sensor_data`: 실제 센서 드라이버가 생성하는 것과 동일한 형식
3. **mode-agnostic 보장**: Backend는 `TelemetryMessage`의 출처가 실제 로봇인지 시뮬레이션인지 구분하지 않는다.
4. **포트**: 기본 50051, 환경 변수 `SIM_ENGINE_PORT`로 설정 가능

```
Backend                              Sim Engine
   │                                     │
   │──── CreateSimulation ──────────────►│
   │◄─── CreateSimResponse ─────────────│
   │                                     │
   │──── SpawnRobot ────────────────────►│
   │◄─── SpawnRobotResponse ────────────│
   │                                     │
   │──── StartSimulation ───────────────►│
   │◄─── stream SimEvent ───────────────│
   │                                     │
   │──── StreamTelemetry ───────────────►│
   │◄─── stream TelemetryMessage ───────│
   │                                     │
   │──── SendCommand (반복) ────────────►│
   │◄─── CommandResponse ───────────────│
   │                                     │
   │──── StopSimulation ────────────────►│
   │◄─── StopSimResponse ──────────────│
```

### 11.2 Asset Manager Team 인터페이스

**통신 방식**: gRPC 또는 로컬 파일 시스템

**계약 사항**:

1. **모델 로드**: Asset Manager가 USD 파일을 로컬 경로 또는 gRPC를 통해 제공 (SDF/URDF는 USD로 변환하여 제공)
2. **파일 형식**: OpenUSD (.usd/.usda/.usdc) — 업계 표준, NVIDIA Omniverse/Isaac Sim 네이티브
3. **좌표계**: X-forward, Y-left, Z-up
4. **단위**: 미터(m), 킬로그램(kg), 라디안(rad)
5. **메시 형식**: STL 또는 OBJ (충돌용), glTF 2.0 (렌더링용)

```
Asset Manager                        Sim Engine
   │                                     │
   │◄─── GetModel(model_id) ────────────│
   │──── ModelData(usd_content) ────────►│
   │                                     │
   │◄─── GetMesh(mesh_uri) ─────────────│
   │──── MeshData(vertices, faces) ─────►│
```

또는 로컬 파일 경로:
```
Sim Engine → 로컬 파일 시스템: /assets/models/{model_id}/{model_name}.usda
```

### 11.3 Map Manager Team 인터페이스

**통신 방식**: gRPC 또는 로컬 파일 시스템

**계약 사항**:

1. **맵 데이터 형식**:
   - 2D 그리드맵: `nav_msgs/OccupancyGrid` 형식 (width, height, resolution, origin, data[])
   - 포인트 클라우드: PCD 형식 또는 proto `PointCloud` 메시지
   - 로드맵: 노드 그래프 (JSON 또는 proto)
2. **맵 → 시뮬레이션 환경 변환**:
   - 그리드맵의 점유 셀(occupied cell) → 시뮬레이션의 정적 충돌 박스
   - 포인트 클라우드 → 삼각형 메시 재구성 → 충돌 메시
3. **좌표 정렬**: 맵의 원점과 시뮬레이션 월드의 원점이 일치해야 한다

```
Map Manager                          Sim Engine
   │                                     │
   │◄─── GetMap(map_id) ────────────────│
   │──── MapData(grid/pointcloud) ──────►│
   │                                     │
   │◄─── GetRoadmap(map_id) ────────────│
   │──── RoadmapData(nodes, edges) ─────►│
```

### 11.4 Proto Team 인터페이스

**계약 사항**:

1. **공유 디렉토리**: 루트 `proto/` 디렉토리의 `.proto` 파일을 사용
2. **네임스페이스**: `amr.common`, `amr.robot`, `amr.sensor`, `amr.simulation`
3. **호환성**: proto 메시지 변경 시 하위 호환성 유지 (필드 번호 변경 금지, 삭제 대신 deprecated)
4. **Sim Engine 전용 proto**: `simulation.proto`는 Sim Engine이 소유하되, `proto/` 디렉토리에 배치

---

## 12. 설정 파일

### 12.1 기본 설정 (YAML)

```yaml
# config/default_config.yaml
simulation:
  physics:
    backend: "custom"            # "custom" | "mujoco" | "isaac_sim"
    fixed_timestep: 0.001        # 1ms = 1000Hz
    gravity: [0.0, 0.0, -9.81]
    default_friction: 0.8
    default_restitution: 0.1

    # 커스텀 경량 백엔드 설정
    custom:
      collision_mode: "2d"       # "2d" | "2.5d"
      max_robots: 200

    # MuJoCo 백엔드 설정
    mujoco:
      timestep: 0.001
      solver_iterations: 50

    # Isaac Sim 백엔드 설정
    isaac_sim:
      endpoint: "localhost:50055"
      use_gpu_physics: true
      use_rtx_lidar: true

  realtime_factor: 1.0           # 1.0 = 실시간
  grpc_port: 50051
  log_level: "info"              # trace, debug, info, warn, error, critical
  enable_rendering: false
  enable_debug_viewport: false
  sensor_thread_count: 2

robot_defaults:
  differential:
    wheel_radius: 0.05
    wheel_separation: 0.3
    max_linear_speed: 1.0
    max_angular_speed: 2.0
    max_linear_accel: 0.5
    max_angular_accel: 1.0
    max_wheel_torque: 5.0

  mecanum:
    wheel_radius: 0.05
    wheel_separation_x: 0.3
    wheel_separation_y: 0.25
    max_linear_speed: 1.0
    max_angular_speed: 2.0

  battery:
    initial_percentage: 100.0
    voltage: 24.0
    discharge_rate_per_meter: 0.01
    idle_rate_per_second: 0.001

sensor_defaults:
  lidar_2d:
    update_rate: 10.0
    angle_min: -3.14159
    angle_max: 3.14159
    angle_increment: 0.00872    # ~0.5도
    range_min: 0.1
    range_max: 12.0
    noise_stddev: 0.005

  imu:
    update_rate: 100.0
    accel_noise_density: 0.001
    gyro_noise_density: 0.0001

  encoder:
    update_rate: 100.0
    ticks_per_revolution: 4096

  camera_rgb:
    update_rate: 30.0
    width: 640
    height: 480
    fov_horizontal: 1.0472       # 60도
    noise_stddev: 2.0
```

### 12.2 clang-format 설정

```yaml
# .clang-format
---
Language: Cpp
BasedOnStyle: Google
IndentWidth: 4
TabWidth: 4
UseTab: Never
ColumnLimit: 100
AccessModifierOffset: -4
AlignAfterOpenBracket: Align
AllowShortFunctionsOnASingleLine: Inline
AllowShortIfStatementsOnASingleLine: Never
AllowShortLoopsOnASingleLine: false
BreakBeforeBraces: Attach
NamespaceIndentation: None
SortIncludes: CaseInsensitive
IncludeBlocks: Regroup
IncludeCategories:
  - Regex: '^"'
    Priority: 1
  - Regex: '^<(gtest|gmock|benchmark)'
    Priority: 3
  - Regex: '^<'
    Priority: 2
```

---

## 13. 로깅 규약

### 13.1 로그 레벨 사용 기준

| 레벨 | 용도 | 예시 |
|------|------|------|
| `trace` | 매우 상세한 디버그 (매 스텝) | "Step 12345: physics dt=0.001, 10 contacts" |
| `debug` | 디버그 정보 | "Robot robot_01 command: v=0.5, ω=0.1" |
| `info` | 주요 이벤트 | "Simulation started", "Robot spawned: robot_01" |
| `warn` | 경고 (기능 영향 없음) | "Running slower than realtime: 120% of target" |
| `error` | 오류 (복구 가능) | "Failed to load model: file not found" |
| `critical` | 치명적 오류 (복구 불가) | "Physics backend init failed" |

### 13.2 로그 형식

```
[2026-03-21 14:30:00.123] [info] [12345] Simulation started (dt=0.001s, rtf=1.0)
│                          │       │       └─ 메시지
│                          │       └─ 스레드 ID
│                          └─ 로그 레벨
└─ 타임스탬프 (밀리초 정밀도)
```

---

## 14. 에러 처리 규약

### 14.1 에러 전파 방식

| 영역 | 방식 | 설명 |
|------|------|------|
| **C++ 내부** | `std::expected<T, Error>` 또는 예외 | 초기화 실패 → 예외, 런타임 → expected |
| **gRPC** | `grpc::Status` + 응답 메시지의 `error_message` | 클라이언트에게 에러 코드 + 메시지 전달 |
| **Python** | 예외 (`RuntimeError`, `ValueError`) | pybind11 자동 변환 |

### 14.2 에러 코드

```cpp
// src/core/error.h
#pragma once
#include <string>
#include <system_error>

namespace amr::sim {

enum class SimError {
    Ok = 0,
    SimNotFound,
    RobotNotFound,
    ObjectNotFound,
    ModelLoadFailed,
    WorldLoadFailed,
    PhysicsInitFailed,
    RenderInitFailed,
    InvalidConfig,
    AlreadyRunning,
    NotRunning,
    DuplicateId,
    InternalError
};

std::string errorMessage(SimError err);

} // namespace amr::sim
```

---

본 명세서는 AMR 시뮬레이션 엔진의 전체 설계, 구현, 테스트, 배포에 필요한 모든 정보를 포함한다. 개발 팀(AI 에이전트 스웜)은 이 문서만으로 Phase 1부터 Phase 4까지 순차적으로 전체 시스템을 구축할 수 있다.
