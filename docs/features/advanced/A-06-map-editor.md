# A-06: 맵 에디터 (Map Editor)

## 1. 개요

### 1.1 목표
시맨틱 영역 편집, 장애물 배치, 포인트 클라우드 LOD 뷰어, 로드맵 그래프 편집을
지원하는 브라우저 기반 3D 맵 에디터를 구현한다. 언두/리두 및 맵 버전 관리를 포함한다.

### 1.2 담당팀

| 팀 | 역할 |
|----|------|
| **Team 1 (Frontend)** | 3D 에디터 UI, 폴리곤 드로잉, 언두/리두, Potree 뷰어 |
| **Team 5 (Map Manager)** | 시맨틱/장애물 저장 API, 맵 버전 관리, Potree 타일 서빙 |

### 1.3 Phase: **2**

### 1.4 선행조건
- C-02: 3D 뷰어 기본 구현 (Three.js 기반)
- C-04: 로드맵 그래프 데이터 구조

---

## 2. 스코프

### In-Scope
- Potree LOD 포인트 클라우드 스트리밍 뷰어
- 로드맵 노드/엣지 편집 (추가, 이동, 삭제)
- 폴리곤 기반 시맨틱 영역 드로잉
- 영역 타입: no-go, speed-limit, charging, parking, loading
- 장애물 배치 (박스, 실린더, 커스텀 메시)
- 언두/리두 (Command Pattern)
- 맵 버전 저장/로드

### Out-of-Scope
- SLAM 기반 자동 맵 생성 (→ Phase 3)
- 멀티플로어 맵 편집 (→ Phase 4)
- 협업 동시 편집 (→ Phase 4)

---

## 3. 상세 스펙

### 3.1 Potree LOD 스트리밍

```typescript
// Potree 로더 설정
const potreeLoader = new PotreeLoader();
potreeLoader.load(metadataUrl, (pointCloud) => {
  pointCloud.material.pointSizeType = PointSizeType.ADAPTIVE;
  pointCloud.material.size = 1.0;
  pointCloud.material.pointColorType = PointColorType.RGB;
  scene.add(pointCloud);
});
```

- Map Manager가 Potree octree 포맷 타일 제공 (`/api/v1/maps/{id}/potree/`)
- LOD: 카메라 거리 기반 자동 디테일 조절
- 포인트 버짓: 최대 500만 포인트 동시 렌더링

### 3.2 로드맵 그래프 편집

```typescript
interface RoadmapEditor {
  addNode(position: Vector3, type: NodeType): string;
  removeNode(nodeId: string): void;
  moveNode(nodeId: string, newPosition: Vector3): void;
  addEdge(fromId: string, toId: string, bidirectional: boolean): string;
  removeEdge(edgeId: string): void;
}

type NodeType = 'waypoint' | 'station' | 'charger' | 'elevator';
```

- 3D 뷰에서 클릭하여 노드 추가 (포인트 클라우드 표면 스냅)
- 노드 간 드래그로 엣지 생성
- 노드 속성 패널: 타입, 이름, 허용 방향, 속도 제한

### 3.3 시맨틱 영역 드로잉

```typescript
interface SemanticRegion {
  id: string;
  name: string;
  type: RegionType;
  polygon: Vector3[];          // 3D 폴리곤 꼭짓점 (바닥면)
  properties: RegionProperties;
}

type RegionType = 'no_go' | 'speed_limit' | 'charging' | 'parking' | 'loading';

interface RegionProperties {
  max_speed?: number;          // speed_limit 영역 전용 (m/s)
  charger_count?: number;      // charging 영역 전용
  capacity?: number;           // parking 영역: 최대 로봇 수
}
```

- 마우스 클릭으로 폴리곤 꼭짓점 지정, 더블클릭으로 완료
- 영역 타입별 색상: no-go(빨강), speed-limit(노랑), charging(파랑), parking(초록), loading(보라)
- 영역 내 로봇 진입 시 규칙 자동 적용 (no-go: 경로 차단, speed-limit: 속도 제한)

### 3.4 장애물 배치

- 기본 형상: Box(가로/세로/높이), Cylinder(반지름/높이)
- 커스텀 메시: glTF 파일 업로드 → 충돌 메시로 등록
- 드래그로 위치/회전 조정 (TransformControls)
- 시뮬레이션 엔진에 장애물 데이터 전달 → 충돌 환경 구성

### 3.5 언두/리두 (Command Pattern)

```typescript
interface EditorCommand {
  execute(): void;
  undo(): void;
  description: string;
}

class EditorHistory {
  private undoStack: EditorCommand[] = [];
  private redoStack: EditorCommand[] = [];
  execute(cmd: EditorCommand): void;
  undo(): void;  // Ctrl+Z
  redo(): void;  // Ctrl+Shift+Z
}
```

- 모든 편집 작업을 Command 객체로 래핑
- 최대 100개 히스토리 유지
- 키보드 단축키: Ctrl+Z (언두), Ctrl+Shift+Z (리두)

### 3.6 맵 버전 관리

- `PUT /api/v1/maps/{id}` → 새 버전 생성 (이전 버전 보존)
- 버전 목록 조회, 특정 버전 로드, 버전 비교 (diff 시각화)
- 활성 맵 버전 지정 (운영 중 사용 버전)

> 📋 인터페이스 상세: [integration-spec.md](../../integration/integration-spec.md#map-editor-api) 참조

---

## 4. 구현 모듈

| 모듈 | 팀 | 경로 |
|------|-----|------|
| `MapEditorPage.tsx` | Team 1 | `frontend/src/pages/MapEditorPage.tsx` |
| `RoadmapEditor.tsx` | Team 1 | `frontend/src/components/editor/RoadmapEditor.tsx` |
| `RegionDrawer.tsx` | Team 1 | `frontend/src/components/editor/RegionDrawer.tsx` |
| `ObstaclePlacer.tsx` | Team 1 | `frontend/src/components/editor/ObstaclePlacer.tsx` |
| `EditorHistory.ts` | Team 1 | `frontend/src/utils/EditorHistory.ts` |
| `PotreeViewer.tsx` | Team 1 | `frontend/src/components/viewer/PotreeViewer.tsx` |

---

## 5. 의존성

| 항목 | 종류 | 비고 |
|------|------|------|
| Potree | 라이브러리 | 포인트 클라우드 LOD 렌더링 |
| Three.js TransformControls | 라이브러리 | 객체 위치/회전 조작 |
| Map Manager REST API | 서비스 | 맵 데이터 CRUD, Potree 타일 |

> 📋 테스트 데이터: [test-data-spec.md](../../integration/test-data-spec.md#map-data) 참조

---

## 6. 테스트 기준

| 테스트 유형 | 기준 |
|------------|------|
| 단위 테스트 | Command Pattern 실행/언두/리두 정확성 |
| UI 테스트 | 폴리곤 드로잉 → 저장 → 재로드 데이터 일치 |
| 성능 테스트 | 1억 포인트 클라우드 LOD 렌더링 30fps 이상 |
| 통합 테스트 | 맵 편집 → 저장 → 시뮬레이션 환경 반영 E2E |
| 버전 테스트 | 맵 버전 생성/로드/비교 정상 동작 |

---

## 7. 완료 조건

- [ ] Potree LOD 스트리밍 뷰어 30fps 이상 동작
- [ ] 로드맵 노드/엣지 CRUD 편집 및 저장
- [ ] 시맨틱 영역 폴리곤 드로잉 및 타입별 속성 설정
- [ ] 장애물 배치 및 시뮬레이션 연동
- [ ] 언두/리두 100개 히스토리 동작
- [ ] 맵 버전 저장/로드 기능 완성
