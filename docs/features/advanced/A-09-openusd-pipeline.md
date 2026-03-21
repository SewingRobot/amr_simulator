# A-09: OpenUSD 파이프라인 (OpenUSD Pipeline)

## 1. 개요

### 1.1 목표
전체 OpenUSD 워크플로우를 구축한다. URDF/SDF ↔ USD 양방향 변환,
UsdPhysics 스키마를 활용한 물리 속성 매핑, USD 씬 로딩을 통해
시뮬레이션 엔진과 웹 뷰어 간 에셋 파이프라인을 통합한다.

### 1.2 담당팀

| 팀 | 역할 |
|----|------|
| **Team 4 (Asset Manager)** | URDF/SDF ↔ USD 변환기, USD → glTF 변환, 에셋 파이프라인 |
| **Team 3 (Sim Engine)** | USD 씬 로딩, UsdPhysics 스키마 → 물리 백엔드 매핑 |

### 1.3 Phase: **3**

### 1.4 선행조건
- C-01: 기본 시뮬레이션 엔진 (PhysicsBackend 인터페이스)
- C-05: 에셋 관리 기본 구조 (glTF 로드, 메타데이터 DB)

---

## 2. 스코프

### In-Scope
- OpenUSD SDK 통합 (C++, pxr 라이브러리)
- URDF → USD 변환기 (Python, `urdf2usd`)
- SDF → USD 변환기 (Python, `sdf2usd`)
- USD → glTF 변환기 (웹 뷰어용)
- UsdPhysics 스키마 매핑 (mass, inertia, collision, joint)
- Sim Engine에서 USD 씬 직접 로딩
- 에셋 파이프라인 업데이트 (업로드 → 변환 → 저장 → 서빙)

### Out-of-Scope
- USD Composition Arc 전체 지원 (reference, payload만 지원) (→ 향후)
- USD 실시간 협업 편집 (→ 향후)
- Omniverse Nucleus 연동 (→ 향후)

---

## 3. 상세 스펙

### 3.1 OpenUSD SDK 통합

```cpp
// Sim Engine에서의 USD 씬 로딩
#include <pxr/usd/usd/stage.h>
#include <pxr/usd/usdGeom/mesh.h>
#include <pxr/usd/usdPhysics/rigidBodyAPI.h>

class UsdSceneLoader {
public:
    bool load(const std::string& usd_path);
    std::vector<RigidBodyDesc> extract_physics_bodies();
    std::vector<MeshData> extract_visual_meshes();
    std::vector<JointDesc> extract_joints();
private:
    pxr::UsdStageRefPtr m_stage;
};
```

- USD 포맷: `.usda` (ASCII, 디버깅), `.usdc` (Crate, 프로덕션), `.usdz` (패키징)
- Layer 합성: sublayer, reference, payload 지원
- 버전: OpenUSD 24.x+

### 3.2 UsdPhysics 스키마 매핑

| USD 스키마 | 물리 백엔드 속성 |
|-----------|----------------|
| `UsdPhysicsRigidBodyAPI` | mass, center_of_mass, velocity |
| `UsdPhysicsMassAPI` | mass, inertia_tensor (diagonalInertia) |
| `UsdPhysicsCollisionAPI` | collision geometry (mesh, box, sphere, capsule) |
| `UsdPhysicsJoint` | joint type, limits, drives |
| `UsdPhysicsDriveAPI` | motor target position/velocity, stiffness, damping |
| `UsdPhysicsMaterialAPI` | static_friction, dynamic_friction, restitution |

```cpp
RigidBodyDesc UsdSceneLoader::convert_rigid_body(const pxr::UsdPrim& prim) {
    auto rigidBodyAPI = pxr::UsdPhysicsRigidBodyAPI(prim);
    auto massAPI = pxr::UsdPhysicsMassAPI(prim);
    // mass, inertia, collision shape 추출 → RigidBodyDesc 생성
}
```

### 3.3 URDF → USD 변환기

```python
# Asset Manager Python 변환 스크립트
class UrdfToUsdConverter:
    def convert(self, urdf_path: str, output_usd_path: str) -> None:
        """URDF 파일을 USD로 변환한다."""
        # 1. urdf_parser_py로 URDF 파싱
        # 2. 각 link → UsdGeom.Xform + UsdPhysics.RigidBodyAPI
        # 3. 각 joint → UsdPhysics.Joint (revolute, prismatic, fixed)
        # 4. 메시 파일 (.stl/.dae) → USD reference
        # 5. mass, inertia → UsdPhysics.MassAPI
        # 6. collision geometry → UsdPhysics.CollisionAPI
```

### 3.4 SDF → USD 변환기

```python
class SdfToUsdConverter:
    def convert(self, sdf_path: str, output_usd_path: str) -> None:
        """SDF/World 파일을 USD로 변환한다."""
        # 1. libsdformat으로 SDF 파싱
        # 2. 각 model → USD sublayer
        # 3. link, joint, sensor 매핑 (URDF 변환과 유사)
        # 4. world 요소 (light, ground_plane) → USD 프림
```

### 3.5 USD → glTF 변환기

```python
class UsdToGltfConverter:
    def convert(self, usd_path: str, output_gltf_path: str) -> None:
        """USD를 웹 뷰어용 glTF 2.0으로 변환한다."""
        # 1. UsdGeom 메시 추출
        # 2. 머티리얼 → PBR (UsdPreviewSurface → glTF PBR)
        # 3. 텍스처 복사/변환 (PNG/JPEG)
        # 4. Draco 압축 적용 (선택적)
        # 5. .glb (바이너리) 출력
```

### 3.6 에셋 파이프라인 업데이트

```
[URDF/SDF 업로드] → [변환기] → [USD 저장] → [USD → glTF] → [glTF 서빙]
                                    ↓
                         [Sim Engine USD 로딩]
```

- 업로드 시 소스 포맷 자동 감지 (URDF, SDF, USD, glTF)
- 비동기 변환 작업 큐 (Tokio task 또는 Python subprocess)
- 변환 결과 캐싱 (소스 해시 기반)

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#asset-pipeline) 참조

---

## 4. 구현 모듈

| 모듈 | 팀 | 경로 |
|------|-----|------|
| `usd_scene_loader.cpp/h` | Team 3 | `sim-engine/src/scene/usd_scene_loader.cpp` |
| `usd_physics_mapper.cpp/h` | Team 3 | `sim-engine/src/scene/usd_physics_mapper.cpp` |
| `urdf2usd.py` | Team 4 | `asset-manager/converters/urdf2usd.py` |
| `sdf2usd.py` | Team 4 | `asset-manager/converters/sdf2usd.py` |
| `usd2gltf.py` | Team 4 | `asset-manager/converters/usd2gltf.py` |
| `conversion_pipeline.rs` | Team 4 | `asset-manager/src/services/conversion_pipeline.rs` |

---

## 5. 의존성

| 항목 | 종류 | 비고 |
|------|------|------|
| OpenUSD SDK 24.x+ | 라이브러리 | C++ (pxr), Python 바인딩 |
| urdf_parser_py | Python 패키지 | URDF 파싱 |
| libsdformat 14.x | 라이브러리 | SDF 파싱 |
| trimesh / pygltflib | Python 패키지 | glTF 생성 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#usd-assets) 참조

---

## 6. 테스트 기준

| 테스트 유형 | 기준 |
|------------|------|
| 변환 테스트 | 5종 URDF, 3종 SDF → USD 변환 후 원본 대비 메시/조인트 수 일치 |
| 물리 매핑 테스트 | UsdPhysics 속성 → PhysicsBackend 속성 정확히 매핑 |
| 라운드트립 테스트 | URDF → USD → 시뮬레이션 로드 → 동작 결과가 원본 URDF 직접 로드와 일치 |
| glTF 테스트 | USD → glTF 변환 후 Three.js 로드 성공, 시각적 차이 없음 |
| 성능 테스트 | 100MB USD 씬 로딩 < 10초 |

---

## 7. 완료 조건

- [ ] OpenUSD SDK C++ 통합 및 Sim Engine에서 USD 씬 로딩
- [ ] UsdPhysics 스키마 → PhysicsBackend 속성 매핑 동작
- [ ] URDF → USD 변환기 5종 로봇 모델 변환 성공
- [ ] SDF → USD 변환기 3종 환경 변환 성공
- [ ] USD → glTF 변환 후 웹 뷰어 정상 표시
- [ ] 에셋 파이프라인 자동 변환 및 캐싱 동작
