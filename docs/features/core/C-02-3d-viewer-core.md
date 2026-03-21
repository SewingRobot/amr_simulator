# C-02: 3D Viewer Core

> **문서 버전**: 1.0
> **최종 수정일**: 2026-03-22
> **담당 팀**: Team 1 (Frontend)
> **Phase**: 1 (Prototype)

---

## 1. 개요

### 1.1 목표
브라우저에서 AMR 로봇과 환경을 실시간 3D로 시각화하는 기본 뷰어를 구축한다. WebSocket을 통해 수신한 텔레메트리로 로봇 포즈를 실시간 업데이트하고, 카메라 컨트롤과 로봇 선택 인터랙션을 제공한다.

### 1.2 배경
3D Viewer는 운영자가 AMR 시스템의 현재 상태를 직관적으로 파악하는 핵심 UI 컴포넌트이다. Phase 1에서는 Three.js / React Three Fiber 기반으로 최소 기능 뷰어를 구현하며, 10대 로봇의 실시간 시각화를 30fps 이상으로 제공한다.

---

## 2. 스코프

### 2.1 In-Scope (이 기능에서 구현)
- Three.js / React Three Fiber 씬 설정 (lighting, grid, axes)
- 카메라 컨트롤 (orbit, top-down, follow 모드 전환)
- glTF 로봇 모델 로드 및 렌더링 (fallback box 모델 포함)
- WebSocket 텔레메트리로 로봇 포즈 실시간 업데이트 (10Hz)
- 로봇 선택 (클릭) 및 정보 패널 표시
- 기본 환경 렌더링 (그리드, 벽, 장애물을 Three.js 기본 도형으로)
- 포인트 클라우드 기본 표시 (BufferGeometry로 소규모)

### 2.2 Out-of-Scope (후속 기능에서 구현)
- 포인트 클라우드 LOD / Potree 프로그레시브 로딩 (A-06)
- 맵 에디터 (로드맵 편집, 시맨틱 영역) (A-08)
- 센서 데이터 시각화 (LiDAR 팬, 카메라 프러스텀) (A-10)
- 하이브리드 모드 (실제 + 시뮬 오버레이)
- WebGPU 최적화
- 거리/각도 측정 도구

---

## 3. 상세 스펙

### 3.1 씬 구성

**조명:**
- `AmbientLight`: intensity 0.4, color white
- `DirectionalLight`: intensity 0.8, position (10, 20, 10), castShadow enabled

**기본 요소:**
- `GridHelper`: 20m x 15m (월드 크기에 맞춤), 1m spacing, color #cccccc
- `AxesHelper`: size 3m (X=red, Y=green, Z=blue)
- Background: solid color `#f0f0f0` (light mode), `#1a1a2e` (dark mode)

**R3F Canvas 설정:**
```tsx
<Canvas
  shadows
  camera={{ position: [10, 15, 10], fov: 50, near: 0.1, far: 1000 }}
  gl={{ antialias: true, alpha: false }}
>
  <ambientLight intensity={0.4} />
  <directionalLight position={[10, 20, 10]} intensity={0.8} castShadow />
  <EnvironmentRenderer world={worldConfig} />
  <RobotManager robots={robots} selectedId={selectedId} />
  <CameraController mode={cameraMode} targetRobot={followTarget} />
  <GridHelper />
</Canvas>
```

### 3.2 로봇 렌더링

**glTF 모델 로딩:**
- `useGLTF` (Drei) 또는 `GLTFLoader`로 `.glb` 파일 로드
- glTF 미제공 시 fallback: colored `BoxGeometry` (0.5m x 0.3m x 0.2m) + 방향 화살표 (`ConeGeometry`)
- 로봇별 고유 색상 할당 (HSL 기반, hue = robot_index * 137.5 % 360)

**포즈 업데이트:**
- WebSocket으로 10Hz 텔레메트리 수신
- 60fps 렌더링을 위한 보간: `THREE.Vector3.lerp`, `THREE.Quaternion.slerp`
- 보간 계수: `alpha = min(deltaTime / interpolation_duration, 1.0)`, `interpolation_duration = 100ms`

**좌표계 변환:**
> 📋 좌표계 변환 (ENU → Three.js): [integration-spec.md](../../integration/integration-spec.md#coordinate-system) 참조

시뮬레이터 ENU 좌표 (x=East, y=North, z=Up) → Three.js 좌표 (x=Right, y=Up, z=Back):
```typescript
function enuToThreeJs(enu: { x: number; y: number; theta: number }) {
  return {
    x: enu.x,
    y: 0,        // ground plane
    z: -enu.y,   // North → -Z
    rotationY: -enu.theta  // heading → Y-axis rotation
  };
}
```

**로봇 라벨:**
- Drei `<Html>` 컴포넌트로 로봇 이름/상태 오버레이
- 표시 내용: robot ID, 상태 아이콘 (idle/moving/error)
- 카메라 거리 15m 이상일 때 자동 숨김 (성능 최적화)

**이동 궤적 (Trail):**
- 옵션으로 마지막 N개 위치를 점선으로 표시 (`THREE.Line`)
- 기본 N=50, 설정에서 on/off 가능

### 3.3 카메라 컨트롤

| 모드 | 단축키 | 동작 |
|------|--------|------|
| Orbit | `1` | OrbitControls: 마우스 드래그 회전, 휠 줌, 우클릭 팬 |
| Top-down | `2` | 카메라를 Y축 상단에 고정 (position.y=30), 2D 뷰 내비게이션 |
| Follow | `3` | 선택된 로봇을 추적, 로봇 뒤 상방에서 바라봄 (offset: [0, 5, -8]) |

**모드 전환:**
- `CameraController` 컴포넌트에서 mode state에 따라 카메라 위치/타겟 애니메이션
- 전환 시 0.5초 ease-in-out 보간
- Follow 모드: 로봇 미선택 시 자동으로 Orbit 모드로 전환

### 3.4 로봇 선택

**Raycasting:**
- Canvas 클릭 이벤트에서 `THREE.Raycaster`로 로봇 메시 교차 검사
- R3F의 `onPointerDown` 이벤트 활용
- 선택된 로봇: Drei `<Outlines>` 컴포넌트로 외곽선 강조 (color: #00ff88, thickness: 3)
- 빈 공간 클릭 시 선택 해제

**정보 패널 (InfoPanel):**
- 선택된 로봇의 상세 정보를 사이드 패널로 표시
- 표시 항목:
  - Robot ID / Name
  - Status (Idle, Moving, Error, Battery Dead)
  - Position (x, y) - 소수점 2자리
  - Heading (degrees)
  - Linear / Angular Velocity
  - Battery Level (% + progress bar)
  - Collision 상태
- 10Hz로 실시간 업데이트

### 3.5 환경 렌더링 (Phase 1 간소화)

월드 정의 JSON을 받아 Three.js 기본 도형으로 렌더링한다.

**벽:**
- `BoxGeometry`로 렌더링 (두께 0.1m, 높이 2m)
- wall segment의 시작점/끝점에서 위치, 길이, 회전 계산
- Material: `MeshStandardMaterial`, color `#888888`

**장애물:**
- `box` 타입: `BoxGeometry(size.x, 1.5, size.y)`, color `#ff8800`
- `circle` 타입: `CylinderGeometry(radius, radius, 1.5)`, color `#ff8800`
- Material: `MeshStandardMaterial`, opacity 0.9

**바닥:**
- `PlaneGeometry`로 월드 크기만큼 바닥 렌더링
- Material: `MeshStandardMaterial`, color `#e0e0e0`

### 3.6 포인트 클라우드 기본 표시

- Phase 1에서는 Potree 없이 `THREE.BufferGeometry` + `THREE.Points`로 렌더링
- 최대 100만 포인트 지원 (소규모 맵용)
- `Float32Array`로 position 버퍼, `Uint8Array`로 color 버퍼
- `PointsMaterial`: size 0.05, sizeAttenuation true
- 데이터 소스: Backend REST API에서 다운로드

---

## 4. 구현 모듈 / 컴포넌트

| 파일 경로 | 컴포넌트명 | 설명 |
|-----------|-----------|------|
| `src/components/viewer3d/ViewerPage.tsx` | `ViewerPage` | 3D 뷰어 페이지 최상위, Canvas + 패널 레이아웃 |
| `src/components/viewer3d/SceneCanvas.tsx` | `SceneCanvas` | R3F Canvas 래퍼, 조명/그리드/축 설정 |
| `src/components/viewer3d/EnvironmentRenderer.tsx` | `EnvironmentRenderer` | 벽, 장애물, 바닥 렌더링 |
| `src/components/viewer3d/RobotModel.tsx` | `RobotModel` | glTF 로드 또는 fallback box + 방향 화살표 |
| `src/components/viewer3d/RobotManager.tsx` | `RobotManager` | 복수 로봇 메시 관리, 포즈 보간 업데이트 |
| `src/components/viewer3d/CameraController.tsx` | `CameraController` | orbit/top-down/follow 모드 전환 |
| `src/components/viewer3d/SelectionManager.tsx` | `SelectionManager` | raycaster 기반 로봇 선택 |
| `src/components/viewer3d/InfoPanel.tsx` | `InfoPanel` | 선택 로봇 상세 정보 사이드 패널 |
| `src/components/viewer3d/RobotTrail.tsx` | `RobotTrail` | 이동 궤적 라인 렌더링 |
| `src/components/viewer3d/ViewerToolbar.tsx` | `ViewerToolbar` | 카메라 모드 전환 버튼, 설정 |
| `src/components/viewer3d/PointCloudBasic.tsx` | `PointCloudBasic` | BufferGeometry 기반 포인트 클라우드 표시 |
| `src/hooks/useTelemetry.ts` | `useTelemetry` | WebSocket 텔레메트리 구독 훅 |
| `src/hooks/useRobotPose.ts` | `useRobotPose` | 특정 로봇 포즈 실시간 추적 훅 |

---

## 5. 의존성

| 의존 대상 | 종류 | 상세 |
|-----------|------|------|
| C-03 (Backend Core) | 런타임 | WebSocket 서버에서 텔레메트리 수신 |
| C-01 (Sim Engine Core) | 간접 | 텔레메트리 데이터 원본 생성 (Backend를 통해 수신) |
| Three.js / R3F / Drei | 라이브러리 | 3D 렌더링 핵심 의존 |
| Zustand | 상태관리 | 선택 상태, 카메라 모드, 로봇 목록 관리 |
| TanStack Query | 데이터 | REST API 호출 (맵 데이터, 로봇 목록) |

> 📋 WebSocket 메시지 포맷: [integration-spec.md](../../integration/integration-spec.md#websocket-messages) 참조

**개발 시 독립성:**
Phase 1 초기에는 Mock WebSocket 서버 (MSW)로 텔레메트리를 시뮬레이션하여 Backend 없이 단독 개발 가능. glTF 모델 미제공 시 fallback box 모델로 동작.

---

## 6. 테스트 기준

### 6.1 기능 테스트

| 테스트 케이스 | 검증 조건 |
|-------------|----------|
| 씬 초기화 | Canvas 렌더링, 그리드/축/조명 표시 확인 |
| glTF 모델 로드 | `.glb` 파일 로드 후 3초 이내 렌더링 완료 |
| Fallback 모델 | glTF 미제공 시 colored box + 방향 화살표 표시 |
| 포즈 업데이트 | Mock 10Hz 텔레메트리 → 로봇 위치 실시간 반영 |
| 보간 부드러움 | 10Hz 데이터에서 60fps 보간, 시각적 끊김 없음 |
| 카메라 Orbit | 마우스 드래그 회전, 휠 줌, 우클릭 팬 동작 |
| 카메라 Top-down | 키 `2` → 상단 뷰 전환 확인 |
| 카메라 Follow | 로봇 선택 후 키 `3` → 로봇 추적 확인 |
| 로봇 선택 | 로봇 클릭 → 외곽선 강조 + InfoPanel 표시 |
| 선택 해제 | 빈 공간 클릭 → 선택 해제, InfoPanel 닫힘 |
| 환경 렌더링 | 월드 JSON → 벽/장애물 올바른 위치에 표시 |

### 6.2 성능 테스트

| 테스트 케이스 | 검증 조건 |
|-------------|----------|
| 10대 로봇 렌더링 | 30fps 이상 유지 (Chrome DevTools Performance 측정) |
| 포인트 클라우드 50만 포인트 | 렌더링 시 30fps 이상 유지 |
| 메모리 사용량 | 10대 로봇 + 환경 → GPU 메모리 512MB 이하 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#viewer-test-data) 참조

---

## 7. 완료 조건 (Definition of Done)

- [ ] R3F Canvas에 환경(벽, 장애물, 그리드) 렌더링
- [ ] glTF 로봇 모델 로드 또는 fallback box 모델 렌더링
- [ ] WebSocket 텔레메트리로 로봇 포즈 실시간 업데이트 (10Hz → 60fps 보간)
- [ ] 좌표계 변환 (ENU → Three.js) 정확히 적용
- [ ] 카메라 컨트롤 3종 (orbit / top-down / follow) 동작
- [ ] 로봇 클릭 선택 + 외곽선 강조 + 정보 패널 표시
- [ ] 10대 로봇 동시 렌더링 시 30fps 이상
- [ ] Storybook에 뷰어 컴포넌트 등록 (SceneCanvas, RobotModel, InfoPanel)
- [ ] Vitest 컴포넌트 테스트 통과
