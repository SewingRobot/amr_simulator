# A-05: 고급 시뮬레이션 (Advanced Simulation)

## 1. 개요

### 1.1 목표
MuJoCo 물리 백엔드 및 Isaac Sim gRPC 브릿지를 통합하고,
고급 센서(3D LiDAR, RGB/Depth Camera, IMU, Encoder)를 시뮬레이션하여
실제 로봇 센서와 동일한 데이터를 생성한다.

### 1.2 담당팀

| 팀 | 역할 |
|----|------|
| **Team 3 (Sim Engine)** | 물리 백엔드 플러그인, 센서 시뮬레이션, 드라이브 모델 전체 |

### 1.3 Phase: **3**

### 1.4 선행조건
- C-01: 기본 시뮬레이션 엔진 (경량 물리 백엔드, 플러그인 아키텍처)

---

## 2. 스코프

### In-Scope
- MuJoCo 3.x 공유 라이브러리 플러그인 (고정밀 접촉/마찰)
- Isaac Sim gRPC 브릿지 (포토리얼리스틱 렌더링 위임)
- 3D LiDAR 멀티빔 시뮬레이션 (레이캐스팅)
- RGB/Depth 카메라 (Vulkan 헤드리스 렌더링)
- IMU 노이즈 모델 (가우시안 + 바이어스 드리프트)
- 엔코더 틱 시뮬레이션
- 메카넘 드라이브 운동학 모델

### Out-of-Scope
- 날씨/조명 동적 변화 (→ Phase 4)
- GPU 클러스터 분산 시뮬레이션 (→ Phase 4)
- 강화학습 환경 인터페이스 (→ Phase 4)

---

## 3. 상세 스펙

### 3.1 MuJoCo 플러그인

```cpp
class MujocoBackend : public PhysicsBackend {
public:
    void initialize(const PhysicsConfig& config) override;
    void step(double dt) override;
    void add_body(const RigidBodyDesc& desc) override;
    ContactResult get_contacts() const override;
    // MuJoCo-specific: mjModel*, mjData* 관리
private:
    mjModel* m_model = nullptr;
    mjData* m_data = nullptr;
};
```

- `dlopen`으로 MuJoCo .so/.dylib 동적 로딩
- MJCF ↔ USD 매핑: `UsdPhysics` 스키마 → MuJoCo geom/body 변환
- 스텝 주기: 200Hz (5ms), 하위 스텝: 4 (1.25ms 물리 정밀도)

### 3.2 Isaac Sim 브릿지

```protobuf
// Isaac Sim gRPC 브릿지
service IsaacSimBridge {
    rpc CreateScene(SceneDescription) returns (SceneHandle);
    rpc Step(StepRequest) returns (StepResponse);
    rpc GetSensorData(SensorQuery) returns (SensorData);
    rpc Shutdown(Empty) returns (Empty);
}
```

- Isaac Sim이 별도 프로세스/원격 서버에서 실행
- 시뮬 엔진이 gRPC 클라이언트로 Isaac Sim 제어
- 포토리얼리스틱 카메라 데이터만 Isaac Sim에서 수신, 물리는 내부 엔진 사용 가능

### 3.3 3D LiDAR 멀티빔

```cpp
struct LidarConfig {
    int num_beams = 16;           // 빔 수 (16, 32, 64, 128)
    double fov_vertical = 30.0;   // 수직 FOV (deg)
    double fov_horizontal = 360.0;// 수평 FOV (deg)
    double range_min = 0.1;       // 최소 거리 (m)
    double range_max = 100.0;     // 최대 거리 (m)
    double angular_resolution = 0.2; // 수평 각도 분해능 (deg)
    double noise_stddev = 0.01;   // 거리 노이즈 표준편차 (m)
    double update_rate = 10.0;    // Hz
};
```

- 레이캐스팅: 물리 백엔드의 collision geometry 활용
- 출력 포맷: `PointCloud2` 호환 (x, y, z, intensity, ring)

### 3.4 카메라 (RGB/Depth)

- Vulkan Compute Pipeline 헤드리스 렌더링
- RGB: RGBA8 640x480 기본, 최대 1920x1080
- Depth: Float32 동일 해상도, range [0.1, 50.0]m
- 렌즈 모델: Pinhole (fx, fy, cx, cy) + radial distortion (k1, k2, k3)

### 3.5 IMU 노이즈 모델

```cpp
struct ImuNoiseModel {
    double accel_noise_density;     // m/s²/√Hz
    double gyro_noise_density;      // rad/s/√Hz
    double accel_bias_instability;  // m/s²
    double gyro_bias_instability;   // rad/s
    double accel_random_walk;       // m/s³/√Hz
    double gyro_random_walk;        // rad/s²/√Hz
};
// 출력: 6-DOF (ax, ay, az, gx, gy, gz) + timestamp
```

### 3.6 메카넘 드라이브

```
v_x = (w1 + w2 + w3 + w4) * r / 4
v_y = (-w1 + w2 + w3 - w4) * r / 4
ω_z = (-w1 + w2 - w3 + w4) * r / (4 * (l_x + l_y))
```

- 4바퀴 메카넘 역운동학/순운동학 모델
- 바퀴 반지름(r), 축간 거리(l_x, l_y) 파라미터화

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#simulation-sensors) 참조

---

## 4. 구현 모듈

| 모듈 | 팀 | 경로 |
|------|-----|------|
| `mujoco_backend.cpp/h` | Team 3 | `sim-engine/src/physics/mujoco_backend.cpp` |
| `isaac_sim_bridge.cpp/h` | Team 3 | `sim-engine/src/physics/isaac_sim_bridge.cpp` |
| `lidar_sensor.cpp/h` | Team 3 | `sim-engine/src/sensors/lidar_sensor.cpp` |
| `camera_sensor.cpp/h` | Team 3 | `sim-engine/src/sensors/camera_sensor.cpp` |
| `imu_sensor.cpp/h` | Team 3 | `sim-engine/src/sensors/imu_sensor.cpp` |
| `encoder_sensor.cpp/h` | Team 3 | `sim-engine/src/sensors/encoder_sensor.cpp` |
| `mecanum_drive.cpp/h` | Team 3 | `sim-engine/src/kinematics/mecanum_drive.cpp` |

---

## 5. 의존성

| 항목 | 종류 | 비고 |
|------|------|------|
| MuJoCo 3.x | 라이브러리 | 동적 로딩 (선택적) |
| Isaac Sim | 외부 서비스 | gRPC 연결 (선택적) |
| Vulkan SDK 1.3+ | 시스템 | 헤드리스 카메라 렌더링 |
| PhysicsBackend 인터페이스 | 내부 | C-01에서 정의된 플러그인 API |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#sensor-data) 참조

---

## 6. 테스트 기준

| 테스트 유형 | 기준 |
|------------|------|
| 단위 테스트 | 각 센서 출력 포맷 정확성, 노이즈 통계 검증 |
| 물리 테스트 | MuJoCo 충돌/마찰 시나리오 기대값 비교 |
| 성능 테스트 | 10대 로봇, 전 센서 활성화 시 200Hz 유지 |
| 운동학 테스트 | 메카넘 드라이브 궤적이 해석적 해와 ε < 1% |
| 통합 테스트 | LiDAR + Camera 데이터가 Backend까지 정상 전달 |

---

## 7. 완료 조건

- [ ] MuJoCo 백엔드 플러그인 로딩 및 시뮬레이션 동작
- [ ] Isaac Sim 브릿지 gRPC 연결 및 센서 데이터 수신
- [ ] 3D LiDAR 멀티빔 포인트 클라우드 생성 확인
- [ ] RGB/Depth 카메라 Vulkan 헤드리스 렌더링 동작
- [ ] IMU 노이즈 모델 통계적 검증 통과
- [ ] 메카넘 드라이브 운동학 정확성 확인
- [ ] 10대 로봇 동시 시뮬레이션 200Hz 성능 달성
