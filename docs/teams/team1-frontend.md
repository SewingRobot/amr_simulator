# Team 1: Frontend (Web App) — 상세 개발 명세서

> **문서 버전**: 1.0
> **최종 수정일**: 2026-03-21
> **대상 독자**: AI Agent Swarm (자율 개발 에이전트)
> **목적**: 이 문서만으로 AMR 통합 프레임워크의 전체 프론트엔드를 구축할 수 있도록 한다.

---

## 1. 팀 개요

### 1.1 팀 명칭 및 담당 범위

- **팀 명칭**: Team 1 — Frontend (Web App)
- **담당 범위**: AMR 통합 프레임워크의 웹 기반 사용자 인터페이스 전체. 3D 로봇 시각화, 포인트 클라우드 렌더링, 지도 편집, 미션 관리, 원격 제어, 대시보드를 포함한다.
- **핵심 목표**:
  1. 실시간 로봇 텔레메트리를 10-30Hz로 시각화하여 운영자에게 즉각적인 상황 인식 제공
  2. 1억 개 이상의 포인트 클라우드를 30fps 이상으로 렌더링
  3. 직관적인 지도 편집기(로드맵 그래프, 시맨틱 영역)로 비개발자도 사용 가능
  4. 시뮬레이션/실제 로봇 통합 운영을 단일 인터페이스로 지원
  5. 50대 이상의 로봇 동시 모니터링 시 프레임 드롭 없이 안정적 운영

### 1.2 다른 팀과의 의존 관계

| 의존 대상 팀 | 의존 내용 | 인터페이스 |
|---|---|---|
| **Backend Team** | REST API 제공 (로봇/미션/맵/사용자 CRUD), WebSocket 서버 (텔레메트리/알림), WebRTC 시그널링 서버 | OpenAPI 3.1 스키마, JSON Schema |
| **Proto Team** | Protobuf 메시지 정의 → TypeScript 타입 자동 생성 | `protoc-gen-ts` 출력물 (`src/types/generated/`) |
| **Asset Manager Team** | glTF 로봇 모델 파일, 에셋 메타데이터 | glTF 2.0 포맷, Asset JSON Schema |
| **Map Manager Team** | Potree 포인트 클라우드 타일 생성 및 제공 | Potree octree 포맷, 메타데이터 JSON |
| **Sim Engine Team** | 시뮬레이션 텔레메트리 데이터 (Proto 메시지와 동일 포맷) | WebSocket 텔레메트리 메시지 |

**의존 방향**: Frontend는 순수 소비자(consumer)이다. 다른 팀이 제공하는 API와 데이터를 소비하며, 직접 데이터를 생성하거나 서빙하지 않는다. 개발 초기에는 Mock 서버(MSW)를 사용하여 Backend 의존성 없이 독립적으로 개발한다.

---

## 2. 기술 스택 상세

### 2.1 언어 및 프레임워크

| 기술 | 버전 | 선정 사유 | 설정 |
|---|---|---|---|
| **TypeScript** | 5.7.x | 타입 안전성으로 대규모 코드베이스 유지보수성 확보, Proto 생성 타입과 자연스러운 통합 | `strict: true`, `noUncheckedIndexedAccess: true`, `exactOptionalPropertyTypes: true` |
| **React** | 19.x | Concurrent 기능(Suspense, Transitions)으로 대량 실시간 데이터 렌더링 최적화 | StrictMode 활성화, Automatic Batching 활용 |
| **Vite** | 6.x | 빠른 HMR, ESM 기반 번들링, 플러그인 생태계 | SWC 트랜스파일러 사용, chunk splitting 전략 적용 |

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "skipLibCheck": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"],
      "@components/*": ["src/components/*"],
      "@hooks/*": ["src/hooks/*"],
      "@services/*": ["src/services/*"],
      "@stores/*": ["src/stores/*"],
      "@types/*": ["src/types/*"],
      "@utils/*": ["src/utils/*"],
      "@constants/*": ["src/constants/*"]
    }
  },
  "include": ["src/**/*", "tests/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

### 2.2 3D 렌더링

| 기술 | 버전 | 선정 사유 |
|---|---|---|
| **Three.js** | 0.172.x | 성숙한 WebGL/WebGPU 3D 라이브러리, 대규모 생태계 |
| **React Three Fiber (R3F)** | 9.x | Three.js의 React 선언적 래퍼, 컴포넌트 기반 3D 씬 구성 |
| **Drei** | 10.x | R3F 헬퍼 라이브러리 (OrbitControls, Html, useGLTF 등) |
| **Potree** | 2.1.x (커스텀 빌드) | 포인트 클라우드 LOD(Level of Detail) 스트리밍, 1억+ 포인트 지원 |

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react-swc';
import tsconfigPaths from 'vite-tsconfig-paths';

export default defineConfig({
  plugins: [
    react(),
    tsconfigPaths(),
  ],
  resolve: {
    alias: {
      '@': '/src',
    },
  },
  build: {
    target: 'ES2022',
    rollupOptions: {
      output: {
        manualChunks: {
          'three-vendor': ['three', '@react-three/fiber', '@react-three/drei'],
          'potree': ['potree-core'],
          'ui-vendor': ['@radix-ui/react-dialog', '@radix-ui/react-dropdown-menu', '@radix-ui/react-tabs', '@radix-ui/react-tooltip'],
          'query': ['@tanstack/react-query'],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://localhost:8080',
        ws: true,
      },
    },
  },
  worker: {
    format: 'es',
  },
});
```

### 2.3 상태 관리

| 기술 | 버전 | 용도 |
|---|---|---|
| **Zustand** | 5.x | 로컬/3D 상태 관리 (로봇 포즈, 에디터 도구, 카메라 모드 등 고빈도 업데이트) |
| **TanStack Query** | 5.x | 서버 상태 관리 (REST API 캐싱, 자동 리페치, 옵티미스틱 업데이트) |

**선정 사유**: Zustand은 가볍고(~2KB) subscribe 기반이라 10-30Hz 텔레메트리 업데이트 시 불필요한 리렌더를 방지한다. TanStack Query는 서버 상태의 캐싱/무효화/재시도를 선언적으로 처리한다. 두 라이브러리는 관심사가 분리되어 충돌 없이 공존한다.

### 2.4 UI 컴포넌트 및 스타일링

| 기술 | 버전 | 용도 |
|---|---|---|
| **Radix UI** | 최신 | 접근성 준수(WAI-ARIA) headless 컴포넌트 (Dialog, DropdownMenu, Tabs, Tooltip, Select 등) |
| **Tailwind CSS** | 4.x | 유틸리티 퍼스트 CSS, 디자인 토큰 기반 일관된 스타일링 |
| **clsx + tailwind-merge** | 최신 | 조건부 클래스 병합, Tailwind 클래스 충돌 해소 |

```typescript
// tailwind.config.ts
import type { Config } from 'tailwindcss';

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        brand: {
          50: '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          800: '#1e40af',
          900: '#1e3a8a',
          950: '#172554',
        },
        status: {
          idle: '#22c55e',
          busy: '#3b82f6',
          error: '#ef4444',
          offline: '#6b7280',
        },
      },
      fontFamily: {
        sans: ['Inter', 'Pretendard', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'spin-slow': 'spin 3s linear infinite',
      },
    },
  },
  plugins: [],
} satisfies Config;
```

### 2.5 2D 맵 오버레이

| 기술 | 버전 | 용도 |
|---|---|---|
| **Deck.gl** | 9.x | 2D 맵 위 대량 데이터 시각화 (히트맵, 로봇 경로 궤적, 이동 밀도 분석) |
| **Canvas2D (네이티브)** | — | 경량 2D 오버레이 (미니맵, 간단한 경로 표시)에 사용 |

### 2.6 실시간 통신

| 기술 | 용도 |
|---|---|
| **WebSocket (native)** | 텔레메트리 스트리밍(10Hz), 미션 상태 업데이트, 알림, 원격 제어 명령 전송 |
| **WebRTC** | 로봇 카메라 영상 스트리밍 (저지연 < 200ms) |

### 2.7 테스팅

| 기술 | 버전 | 용도 |
|---|---|---|
| **Vitest** | 3.x | 단위/통합 테스트 (Vite 네이티브, Jest 호환 API) |
| **React Testing Library** | 16.x | 컴포넌트 테스트 (사용자 관점 테스트) |
| **Playwright** | 1.50.x | E2E 테스트 (크로스 브라우저) |
| **MSW (Mock Service Worker)** | 2.x | API 모킹 (REST + WebSocket) |
| **Storybook** | 8.x | 컴포넌트 카탈로그, 시각적 회귀 테스트 |
| **@storybook/test** | 8.x | Storybook 내 상호작용 테스트 |

### 2.8 코드 품질

| 기술 | 버전 | 설정 |
|---|---|---|
| **ESLint** | 9.x (Flat Config) | `@typescript-eslint`, `eslint-plugin-react-hooks`, `eslint-plugin-react-refresh` |
| **Prettier** | 3.x | `singleQuote: true`, `trailingComma: 'all'`, `printWidth: 100`, `semi: true` |
| **husky + lint-staged** | 최신 | pre-commit 시 lint + format 자동 실행 |

```javascript
// eslint.config.js
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  {
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'warn',
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
      '@typescript-eslint/strict-boolean-expressions': 'error',
    },
    languageOptions: {
      parserOptions: {
        project: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    ignores: ['dist/', 'node_modules/', '*.config.*'],
  },
);
```

### 2.9 package.json 핵심 의존성

```jsonc
{
  "name": "amr-frontend",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc -b && vite build",
    "preview": "vite preview",
    "lint": "eslint src/",
    "format": "prettier --write src/",
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest run --coverage",
    "test:e2e": "playwright test",
    "storybook": "storybook dev -p 6006",
    "build-storybook": "storybook build",
    "type-check": "tsc --noEmit",
    "generate-types": "buf generate ../proto --template buf.gen.yaml"
  },
  "dependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@react-three/fiber": "^9.0.0",
    "@react-three/drei": "^10.0.0",
    "three": "^0.172.0",
    "potree-core": "^2.1.0",
    "zustand": "^5.0.0",
    "@tanstack/react-query": "^5.60.0",
    "@radix-ui/react-dialog": "^1.1.0",
    "@radix-ui/react-dropdown-menu": "^2.1.0",
    "@radix-ui/react-tabs": "^1.1.0",
    "@radix-ui/react-tooltip": "^1.1.0",
    "@radix-ui/react-select": "^2.1.0",
    "@radix-ui/react-checkbox": "^1.1.0",
    "@radix-ui/react-switch": "^1.1.0",
    "@radix-ui/react-slider": "^1.2.0",
    "@radix-ui/react-toast": "^1.2.0",
    "@radix-ui/react-popover": "^1.1.0",
    "@deck.gl/core": "^9.0.0",
    "@deck.gl/layers": "^9.0.0",
    "@deck.gl/react": "^9.0.0",
    "nipplejs": "^0.10.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.6.0",
    "react-router-dom": "^7.1.0",
    "react-hook-form": "^7.54.0",
    "@hookform/resolvers": "^3.9.0",
    "zod": "^3.24.0",
    "recharts": "^2.15.0",
    "date-fns": "^4.1.0",
    "immer": "^10.1.0",
    "uuid": "^11.0.0"
  },
  "devDependencies": {
    "typescript": "^5.7.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@types/three": "^0.172.0",
    "@types/uuid": "^10.0.0",
    "vite": "^6.0.0",
    "@vitejs/plugin-react-swc": "^4.0.0",
    "vite-tsconfig-paths": "^5.1.0",
    "tailwindcss": "^4.0.0",
    "eslint": "^9.17.0",
    "@eslint/js": "^9.17.0",
    "typescript-eslint": "^8.18.0",
    "eslint-plugin-react-hooks": "^5.1.0",
    "eslint-plugin-react-refresh": "^0.4.0",
    "prettier": "^3.4.0",
    "prettier-plugin-tailwindcss": "^0.6.0",
    "vitest": "^3.0.0",
    "@vitest/coverage-v8": "^3.0.0",
    "@vitest/ui": "^3.0.0",
    "@testing-library/react": "^16.1.0",
    "@testing-library/jest-dom": "^6.6.0",
    "@testing-library/user-event": "^14.5.0",
    "jsdom": "^25.0.0",
    "playwright": "^1.50.0",
    "@playwright/test": "^1.50.0",
    "msw": "^2.7.0",
    "storybook": "^8.5.0",
    "@storybook/react-vite": "^8.5.0",
    "@storybook/test": "^8.5.0",
    "@faker-js/faker": "^9.3.0",
    "husky": "^9.1.0",
    "lint-staged": "^15.3.0"
  }
}
```

---

## 3. 디렉토리 구조 상세

```
frontend/
├── public/
│   ├── favicon.ico                        # 앱 파비콘
│   ├── robots.txt                         # 검색엔진 크롤링 설정
│   └── models/
│       └── robot-placeholder.glb          # 기본 로봇 3D 모델 (개발용)
│
├── src/
│   ├── main.tsx                           # React 앱 엔트리포인트 (ReactDOM.createRoot)
│   ├── index.css                          # Tailwind 디렉티브, 글로벌 스타일
│   ├── vite-env.d.ts                      # Vite 환경 타입 선언
│   │
│   ├── app/                               # App Shell, 라우팅, 글로벌 프로바이더
│   │   ├── App.tsx                        # 최상위 앱 컴포넌트 (프로바이더 래핑)
│   │   ├── Router.tsx                     # react-router-dom 라우트 정의
│   │   ├── Layout.tsx                     # 사이드바 + 탑바 + 콘텐츠 영역 레이아웃
│   │   ├── Sidebar.tsx                    # 네비게이션 사이드바 (축소/확장 가능)
│   │   ├── Topbar.tsx                     # 상단바 (유저 정보, 알림, 테마 토글)
│   │   ├── ErrorBoundary.tsx              # 글로벌 에러 바운더리 (fallback UI)
│   │   └── providers/
│   │       ├── QueryProvider.tsx          # TanStack Query 클라이언트 프로바이더
│   │       ├── AuthProvider.tsx           # 인증 컨텍스트 프로바이더
│   │       ├── ThemeProvider.tsx          # 라이트/다크 테마 프로바이더
│   │       ├── WebSocketProvider.tsx      # WebSocket 연결 관리 프로바이더
│   │       └── ToastProvider.tsx          # 전역 토스트 알림 프로바이더
│   │
│   ├── components/
│   │   ├── common/                        # 범용 재사용 컴포넌트
│   │   │   ├── Button.tsx                 # 기본 버튼 (variant: primary/secondary/danger/ghost, size: sm/md/lg)
│   │   │   ├── IconButton.tsx             # 아이콘 전용 버튼
│   │   │   ├── Input.tsx                  # 텍스트 입력 (label, error, helper text 포함)
│   │   │   ├── Select.tsx                 # Radix UI Select 래퍼
│   │   │   ├── Checkbox.tsx               # Radix UI Checkbox 래퍼
│   │   │   ├── Switch.tsx                 # Radix UI Switch 래퍼
│   │   │   ├── Slider.tsx                 # Radix UI Slider 래퍼
│   │   │   ├── Modal.tsx                  # Radix UI Dialog 기반 모달
│   │   │   ├── ConfirmDialog.tsx          # 확인/취소 다이얼로그
│   │   │   ├── Toast.tsx                  # Radix UI Toast 기반 알림
│   │   │   ├── Tooltip.tsx                # Radix UI Tooltip 래퍼
│   │   │   ├── Badge.tsx                  # 상태 표시 배지 (색상 variant)
│   │   │   ├── Card.tsx                   # 카드 컨테이너
│   │   │   ├── Tabs.tsx                   # Radix UI Tabs 래퍼
│   │   │   ├── Table.tsx                  # 데이터 테이블 (정렬, 필터, 페이지네이션)
│   │   │   ├── DataTable.tsx              # TanStack Table 기반 고급 테이블
│   │   │   ├── DropdownMenu.tsx           # Radix UI DropdownMenu 래퍼
│   │   │   ├── Popover.tsx                # Radix UI Popover 래퍼
│   │   │   ├── Spinner.tsx                # 로딩 스피너
│   │   │   ├── Skeleton.tsx               # 로딩 스켈레톤 UI
│   │   │   ├── EmptyState.tsx             # 데이터 없음 상태 표시
│   │   │   ├── ErrorState.tsx             # 에러 상태 표시 (재시도 버튼 포함)
│   │   │   ├── StatusIndicator.tsx        # 로봇 상태 인디케이터 (색상 dot + 라벨)
│   │   │   ├── SearchInput.tsx            # 검색 입력 (debounce 내장)
│   │   │   ├── ProgressBar.tsx            # 진행 막대
│   │   │   ├── Breadcrumb.tsx             # 경로 네비게이션
│   │   │   └── index.ts                   # barrel export
│   │   │
│   │   ├── dashboard/                     # 대시보드 모듈
│   │   │   ├── DashboardPage.tsx          # 대시보드 페이지 최상위 컴포넌트
│   │   │   ├── FleetOverview.tsx          # 로봇 현황 요약 (상태별 카운트, 파이차트)
│   │   │   ├── KPICards.tsx               # 핵심 지표 카드 (완료 미션, 평균 시간, 가동률)
│   │   │   ├── AlertFeed.tsx              # 실시간 알림 피드 (WebSocket 구독)
│   │   │   ├── MissionQueue.tsx           # 미션 대기열 (드래그 우선순위 변경)
│   │   │   ├── RobotList.tsx              # 로봇 목록 테이블 (정렬/필터)
│   │   │   ├── FleetMap.tsx               # 2D 미니맵 (로봇 위치 표시)
│   │   │   └── index.ts
│   │   │
│   │   ├── viewer3d/                      # 3D 뷰어 모듈
│   │   │   ├── ViewerPage.tsx             # 3D 뷰어 페이지 최상위 컴포넌트
│   │   │   ├── SceneManager.tsx           # Three.js 씬 오케스트레이션 (Canvas, 조명, 카메라)
│   │   │   ├── CameraController.tsx       # 카메라 모드 전환 (orbit/fly/top-down)
│   │   │   ├── RobotRenderer.tsx          # 로봇 glTF 모델 렌더링 + 포즈 업데이트
│   │   │   ├── RobotTrail.tsx             # 로봇 이동 궤적 라인 렌더링
│   │   │   ├── PointCloudRenderer.tsx     # Potree 포인트 클라우드 렌더링
│   │   │   ├── ObstacleLayer.tsx          # 장애물 렌더링 (정적 + 동적)
│   │   │   ├── SensorVisualization.tsx    # 센서 시각화 (LiDAR 팬, 카메라 프러스텀)
│   │   │   ├── GridHelper.tsx             # 그리드 + 축 헬퍼
│   │   │   ├── MeasurementTool.tsx        # 거리/각도 측정 도구
│   │   │   ├── SelectionManager.tsx       # 오브젝트 선택 (raycast 기반)
│   │   │   ├── InfoPanel.tsx              # 선택된 오브젝트 정보 패널 (Html overlay)
│   │   │   ├── ViewerToolbar.tsx          # 뷰어 상단 툴바 (모드 전환, 도구 선택)
│   │   │   ├── ViewModeSwitch.tsx         # 시뮬레이션/실제/하이브리드 모드 전환
│   │   │   ├── PerformanceMonitor.tsx     # FPS/드로우콜 모니터 (개발 모드)
│   │   │   └── index.ts
│   │   │
│   │   ├── map-editor/                    # 맵 에디터 모듈
│   │   │   ├── MapEditorPage.tsx          # 맵 에디터 페이지 최상위 컴포넌트
│   │   │   ├── PointCloudImporter.tsx     # 포인트 클라우드 파일 업로드 (LAS/PLY/PCD)
│   │   │   ├── EditorToolbar.tsx          # 에디터 도구 모음 (선택/노드추가/엣지추가/영역그리기)
│   │   │   ├── RoadmapGraphEditor.tsx     # 로드맵 그래프 편집 (노드/엣지 CRUD)
│   │   │   ├── WaypointNode.tsx           # 웨이포인트 노드 3D 렌더링 (sphere + label)
│   │   │   ├── EdgeRenderer.tsx           # 엣지 렌더링 (line + 방향 화살표)
│   │   │   ├── NodePropertiesPanel.tsx    # 노드 속성 편집 패널
│   │   │   ├── EdgePropertiesPanel.tsx    # 엣지 속성 편집 패널
│   │   │   ├── SemanticRegionEditor.tsx   # 시맨틱 영역 편집 (폴리곤 그리기)
│   │   │   ├── RegionRenderer.tsx         # 영역 폴리곤 3D 렌더링 (반투명 메시)
│   │   │   ├── RegionPropertiesPanel.tsx  # 영역 속성 편집 패널
│   │   │   ├── ObstacleEditor.tsx         # 정적 장애물 배치/이동/삭제
│   │   │   ├── UndoRedoManager.tsx        # 실행 취소/재실행 (Command Pattern)
│   │   │   ├── MapVersionHistory.tsx      # 맵 버전 이력 관리
│   │   │   └── index.ts
│   │   │
│   │   ├── mission-control/               # 미션 관리 모듈
│   │   │   ├── MissionControlPage.tsx     # 미션 관리 페이지 최상위 컴포넌트
│   │   │   ├── MissionCreator.tsx         # 미션 생성 위자드 (스텝 폼)
│   │   │   ├── MissionList.tsx            # 미션 목록 테이블 (필터/정렬/페이지네이션)
│   │   │   ├── MissionDetail.tsx          # 미션 상세 (단계별 진행, 타임라인)
│   │   │   ├── MissionTimeline.tsx        # 미션 실행 타임라인 시각화
│   │   │   ├── MissionMonitor.tsx         # 실시간 미션 모니터 (맵 오버레이)
│   │   │   ├── TrafficRuleEditor.tsx      # 교통 규칙 편집 (구역별 제한)
│   │   │   ├── ConflictResolutionPanel.tsx # 충돌 해소 설정 패널
│   │   │   └── index.ts
│   │   │
│   │   ├── remote-control/                # 원격 제어 모듈
│   │   │   ├── RemoteControlPage.tsx      # 원격 제어 페이지 최상위 컴포넌트
│   │   │   ├── JoystickController.tsx     # 가상 조이스틱 (nipplejs) + WASD 키보드
│   │   │   ├── CameraFeed.tsx             # WebRTC 카메라 영상 표시
│   │   │   ├── TelemetryPanel.tsx         # 실시간 텔레메트리 패널
│   │   │   ├── EmergencyStop.tsx          # 긴급 정지 버튼
│   │   │   ├── ConnectionStatus.tsx       # 연결 상태 표시
│   │   │   ├── SpeedControl.tsx           # 속도 제한 슬라이더
│   │   │   └── index.ts
│   │   │
│   │   └── settings/                      # 설정 모듈
│   │       ├── SettingsPage.tsx            # 설정 페이지 최상위 컴포넌트 (탭 기반)
│   │       ├── RobotConfig.tsx            # 로봇 등록/수정 설정
│   │       ├── UserManagement.tsx         # 사용자 관리 (CRUD + 역할)
│   │       ├── PluginManager.tsx          # 플러그인 관리 (설치/활성화/설정)
│   │       ├── SystemSettings.tsx         # 시스템 설정 (테마, 언어, 재접속)
│   │       └── index.ts
│   │
│   ├── hooks/                             # 커스텀 React Hooks
│   │   ├── useAuth.ts                     # 인증 관련 훅 (로그인/로그아웃/토큰 갱신)
│   │   ├── useWebSocket.ts               # WebSocket 연결/구독 관리 훅
│   │   ├── useWebRTC.ts                   # WebRTC 연결 관리 훅
│   │   ├── useTelemetry.ts               # 로봇 텔레메트리 구독 훅
│   │   ├── useRobotPose.ts               # 특정 로봇 포즈 실시간 추적 훅
│   │   ├── usePointCloud.ts              # 포인트 클라우드 로딩/LOD 관리 훅
│   │   ├── useMapEditor.ts               # 맵 에디터 상태/액션 통합 훅
│   │   ├── useUndoRedo.ts                # 실행 취소/재실행 훅 (Command Pattern)
│   │   ├── useMission.ts                 # 미션 CRUD 훅 (TanStack Query 래핑)
│   │   ├── useRemoteControl.ts           # 원격 제어 명령 전송 훅
│   │   ├── useKeyboard.ts                # 키보드 이벤트 핸들링 훅
│   │   ├── useDebounce.ts                # 디바운스 훅
│   │   ├── useThrottle.ts                # 스로틀 훅
│   │   ├── useMediaQuery.ts              # 미디어 쿼리 반응형 훅
│   │   ├── useLocalStorage.ts            # 로컬 스토리지 동기화 훅
│   │   ├── useInterval.ts                # 인터벌 관리 훅 (cleanup 보장)
│   │   └── index.ts
│   │
│   ├── services/                          # API 클라이언트 / 외부 서비스
│   │   ├── api.ts                         # REST API 클라이언트 (fetch 기반, 인터셉터, 재시도)
│   │   ├── websocket.ts                   # WebSocket 연결 관리자 (자동 재접속, 메시지 라우팅)
│   │   ├── webrtc.ts                      # WebRTC 피어 연결 관리자
│   │   ├── pointcloud.ts                  # Potree 타일 로더 (HTTP range requests)
│   │   ├── auth.service.ts                # 인증 API 호출 (login, refresh, me)
│   │   ├── robot.service.ts               # 로봇 API 호출 (CRUD)
│   │   ├── mission.service.ts             # 미션 API 호출 (CRUD + assign)
│   │   ├── map.service.ts                 # 맵 API 호출 (CRUD + roadmap + semantic)
│   │   ├── asset.service.ts               # 에셋 API 호출 (upload + CRUD)
│   │   ├── user.service.ts                # 사용자 API 호출 (CRUD)
│   │   ├── plugin.service.ts              # 플러그인 API 호출
│   │   └── index.ts
│   │
│   ├── stores/                            # Zustand 상태 저장소
│   │   ├── authStore.ts                   # 인증 상태 (user, token, login/logout)
│   │   ├── robotStore.ts                  # 로봇 상태 (Map<id, RobotState>, 선택, 포즈 업데이트)
│   │   ├── mapStore.ts                    # 맵 상태 (현재 맵, 로드맵 그래프, 시맨틱 영역)
│   │   ├── missionStore.ts               # 미션 상태 (미션 목록, 활성 미션)
│   │   ├── viewerStore.ts                # 3D 뷰어 상태 (카메라 모드, 뷰 모드, 선택)
│   │   ├── editorStore.ts                # 에디터 상태 (도구, undo/redo 스택)
│   │   ├── alertStore.ts                  # 알림 상태 (알림 목록, 읽음 상태)
│   │   ├── uiStore.ts                     # UI 상태 (사이드바 열림, 테마, 언어)
│   │   └── index.ts
│   │
│   ├── types/                             # TypeScript 타입 정의
│   │   ├── generated/                     # protoc-gen-ts 자동 생성 타입 (커밋하지 않음)
│   │   │   └── .gitkeep
│   │   ├── robot.types.ts                 # Robot, RobotStatus, RobotConfig 등
│   │   ├── map.types.ts                   # MapData, RoadmapNode, RoadmapEdge, SemanticRegion 등
│   │   ├── mission.types.ts               # Mission, MissionStatus, MissionStep 등
│   │   ├── telemetry.types.ts             # TelemetryMessage, Twist, Pose3D 등
│   │   ├── user.types.ts                  # User, UserRole 등
│   │   ├── api.types.ts                   # API 요청/응답 타입, PaginatedResponse 등
│   │   ├── websocket.types.ts             # WebSocket 메시지 타입
│   │   ├── viewer.types.ts                # 뷰어 관련 타입 (CameraMode, ViewMode 등)
│   │   ├── editor.types.ts                # 에디터 관련 타입 (EditorTool, Command 등)
│   │   ├── common.types.ts                # 공통 타입 (Vector3, Quaternion, BoundingBox 등)
│   │   └── index.ts
│   │
│   ├── utils/                             # 유틸리티 함수
│   │   ├── math.ts                        # 3D 수학 (쿼터니언 → 오일러 변환, 거리 계산 등)
│   │   ├── format.ts                      # 포맷팅 (날짜, 숫자, 단위 변환)
│   │   ├── validation.ts                  # Zod 스키마 기반 입력 검증
│   │   ├── color.ts                       # 상태별 색상 매핑, 히트맵 컬러 스케일
│   │   ├── geometry.ts                    # 2D/3D 기하 유틸 (폴리곤 포함 판정, 선분 교차)
│   │   ├── throttle.ts                    # 스로틀 유틸 함수
│   │   ├── debounce.ts                    # 디바운스 유틸 함수
│   │   ├── cn.ts                          # clsx + tailwind-merge 래퍼
│   │   └── index.ts
│   │
│   ├── constants/                         # 상수 값
│   │   ├── routes.ts                      # 라우트 경로 상수
│   │   ├── queryKeys.ts                   # TanStack Query 키 상수
│   │   ├── wsTopics.ts                    # WebSocket 토픽 상수
│   │   ├── editorDefaults.ts              # 에디터 기본값 (그리드 크기, 스냅 간격 등)
│   │   ├── viewerDefaults.ts              # 뷰어 기본값 (카메라 위치, FOV, near/far 등)
│   │   └── index.ts
│   │
│   └── __mocks__/                         # Mock 데이터 및 핸들러
│       ├── handlers/
│       │   ├── auth.handlers.ts           # 인증 API Mock 핸들러
│       │   ├── robot.handlers.ts          # 로봇 API Mock 핸들러
│       │   ├── mission.handlers.ts        # 미션 API Mock 핸들러
│       │   ├── map.handlers.ts            # 맵 API Mock 핸들러
│       │   └── index.ts                   # 모든 핸들러 통합 export
│       ├── data/
│       │   ├── robots.mock.ts             # Mock 로봇 데이터
│       │   ├── missions.mock.ts           # Mock 미션 데이터
│       │   ├── maps.mock.ts               # Mock 맵 데이터
│       │   └── users.mock.ts              # Mock 사용자 데이터
│       ├── factories/
│       │   ├── robot.factory.ts           # faker.js 기반 로봇 데이터 팩토리
│       │   ├── mission.factory.ts         # faker.js 기반 미션 데이터 팩토리
│       │   └── telemetry.factory.ts       # faker.js 기반 텔레메트리 팩토리
│       ├── server.ts                      # MSW 서버 설정 (테스트용)
│       ├── browser.ts                     # MSW 브라우저 워커 설정 (개발용)
│       └── ws-mock-server.ts              # WebSocket Mock 서버 (텔레메트리 생성)
│
├── tests/
│   ├── unit/                              # 단위 테스트
│   │   ├── stores/                        # Zustand 스토어 테스트
│   │   │   ├── authStore.test.ts
│   │   │   ├── robotStore.test.ts
│   │   │   └── ...
│   │   ├── hooks/                         # 커스텀 훅 테스트
│   │   │   ├── useWebSocket.test.ts
│   │   │   ├── useTelemetry.test.ts
│   │   │   └── ...
│   │   ├── services/                      # 서비스 클라이언트 테스트
│   │   │   ├── api.test.ts
│   │   │   ├── websocket.test.ts
│   │   │   └── ...
│   │   └── utils/                         # 유틸리티 함수 테스트
│   │       ├── math.test.ts
│   │       ├── geometry.test.ts
│   │       └── ...
│   │
│   ├── integration/                       # 통합 테스트
│   │   ├── dashboard.test.tsx             # 대시보드 페이지 통합 테스트
│   │   ├── map-editor.test.tsx            # 맵 에디터 통합 테스트
│   │   ├── mission-control.test.tsx       # 미션 관리 통합 테스트
│   │   └── setup.ts                       # MSW + 테스트 프로바이더 설정
│   │
│   └── e2e/                               # E2E 테스트 (Playwright)
│       ├── auth.spec.ts                   # 로그인/로그아웃 플로우
│       ├── dashboard.spec.ts              # 대시보드 데이터 로딩 검증
│       ├── mission.spec.ts                # 미션 생성 → 모니터링 플로우
│       ├── map-editor.spec.ts             # 맵 편집 플로우 (노드/엣지/영역)
│       ├── remote-control.spec.ts         # 원격 제어 세션
│       └── fixtures/                      # E2E 테스트 픽스처
│           └── test-data.ts
│
├── .storybook/
│   ├── main.ts                            # Storybook 설정 (Vite 빌더)
│   ├── preview.ts                         # Storybook 프리뷰 설정 (데코레이터, 파라미터)
│   └── preview-head.html                  # Storybook HTML head 커스텀
│
├── index.html                             # Vite HTML 엔트리
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json                     # Vite/Node 설정용 tsconfig
├── tailwind.config.ts
├── playwright.config.ts
├── .prettierrc.json
├── eslint.config.js
├── .env.example                           # 환경변수 예시
└── .gitignore
```

---

## 4. 구성 모듈 상세 스펙

### 4.1 App Shell & Routing

#### 목적
애플리케이션의 골격을 구성한다. 인증, 라우팅, 글로벌 프로바이더, 레이아웃을 관리한다.

#### App.tsx — 최상위 컴포넌트

```typescript
// src/app/App.tsx
import { QueryProvider } from './providers/QueryProvider';
import { AuthProvider } from './providers/AuthProvider';
import { ThemeProvider } from './providers/ThemeProvider';
import { WebSocketProvider } from './providers/WebSocketProvider';
import { ToastProvider } from './providers/ToastProvider';
import { ErrorBoundary } from './ErrorBoundary';
import { Router } from './Router';

export function App() {
  return (
    <ErrorBoundary>
      <ThemeProvider>
        <QueryProvider>
          <AuthProvider>
            <WebSocketProvider>
              <ToastProvider>
                <Router />
              </ToastProvider>
            </WebSocketProvider>
          </AuthProvider>
        </QueryProvider>
      </ThemeProvider>
    </ErrorBoundary>
  );
}
```

#### Router.tsx — 라우트 정의

```typescript
// src/app/Router.tsx
import { createBrowserRouter, RouterProvider, Navigate } from 'react-router-dom';
import { Layout } from './Layout';
import { AuthGuard } from './providers/AuthProvider';
import { lazy, Suspense } from 'react';
import { Spinner } from '@/components/common';
import { ROUTES } from '@/constants/routes';

// 코드 스플리팅: 각 페이지를 lazy 로딩
const DashboardPage = lazy(() => import('@/components/dashboard/DashboardPage'));
const ViewerPage = lazy(() => import('@/components/viewer3d/ViewerPage'));
const MapEditorPage = lazy(() => import('@/components/map-editor/MapEditorPage'));
const MissionControlPage = lazy(() => import('@/components/mission-control/MissionControlPage'));
const RemoteControlPage = lazy(() => import('@/components/remote-control/RemoteControlPage'));
const SettingsPage = lazy(() => import('@/components/settings/SettingsPage'));
const LoginPage = lazy(() => import('@/components/auth/LoginPage'));

const router = createBrowserRouter([
  {
    path: ROUTES.LOGIN,
    element: <Suspense fallback={<Spinner />}><LoginPage /></Suspense>,
  },
  {
    path: '/',
    element: (
      <AuthGuard>
        <Layout />
      </AuthGuard>
    ),
    children: [
      { index: true, element: <Navigate to={ROUTES.DASHBOARD} replace /> },
      {
        path: ROUTES.DASHBOARD,
        element: <Suspense fallback={<Spinner />}><DashboardPage /></Suspense>,
      },
      {
        path: ROUTES.VIEWER,
        element: <Suspense fallback={<Spinner />}><ViewerPage /></Suspense>,
      },
      {
        path: ROUTES.MAP_EDITOR,
        element: <Suspense fallback={<Spinner />}><MapEditorPage /></Suspense>,
      },
      {
        path: ROUTES.MISSIONS,
        element: <Suspense fallback={<Spinner />}><MissionControlPage /></Suspense>,
      },
      {
        path: ROUTES.REMOTE_CONTROL,  // '/remote/:robotId'
        element: <Suspense fallback={<Spinner />}><RemoteControlPage /></Suspense>,
      },
      {
        path: ROUTES.SETTINGS,
        element: <Suspense fallback={<Spinner />}><SettingsPage /></Suspense>,
        // 역할 기반 접근: admin만 접근 가능
      },
    ],
  },
]);

export function Router() {
  return <RouterProvider router={router} />;
}
```

#### 라우트 상수 정의

```typescript
// src/constants/routes.ts
export const ROUTES = {
  LOGIN: '/login',
  DASHBOARD: '/dashboard',
  VIEWER: '/viewer',
  MAP_EDITOR: '/map-editor',
  MISSIONS: '/missions',
  REMOTE_CONTROL: '/remote/:robotId',
  SETTINGS: '/settings',
} as const;

export function remoteControlRoute(robotId: string): string {
  return `/remote/${robotId}`;
}
```

#### Layout.tsx — 사이드바 + 탑바 + 콘텐츠

```typescript
// src/app/Layout.tsx
import { Outlet } from 'react-router-dom';
import { Sidebar } from './Sidebar';
import { Topbar } from './Topbar';
import { useUIStore } from '@/stores/uiStore';

export function Layout() {
  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);

  return (
    <div className="flex h-screen bg-gray-50 dark:bg-gray-900">
      <Sidebar collapsed={sidebarCollapsed} />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Topbar />
        <main className="flex-1 overflow-auto p-4">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
```

#### AuthGuard — 역할 기반 라우트 보호

```typescript
// src/app/providers/AuthProvider.tsx 내 AuthGuard 컴포넌트
import { Navigate, useLocation } from 'react-router-dom';
import { useAuthStore } from '@/stores/authStore';
import { ROUTES } from '@/constants/routes';

interface AuthGuardProps {
  children: React.ReactNode;
  requiredRole?: 'admin' | 'operator' | 'viewer';
}

export function AuthGuard({ children, requiredRole }: AuthGuardProps) {
  const { user, token } = useAuthStore();
  const location = useLocation();

  if (token == null) {
    return <Navigate to={ROUTES.LOGIN} state={{ from: location }} replace />;
  }

  if (requiredRole != null && user?.role !== requiredRole && user?.role !== 'admin') {
    return <Navigate to={ROUTES.DASHBOARD} replace />;
  }

  return <>{children}</>;
}
```

#### 테마 시스템 (라이트/다크)

```typescript
// src/app/providers/ThemeProvider.tsx
import { createContext, useContext, useEffect } from 'react';
import { useUIStore } from '@/stores/uiStore';

type Theme = 'light' | 'dark' | 'system';

const ThemeContext = createContext<{
  theme: Theme;
  setTheme: (theme: Theme) => void;
}>({ theme: 'system', setTheme: () => {} });

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const { theme, setTheme } = useUIStore();

  useEffect(() => {
    const root = document.documentElement;
    const resolvedTheme =
      theme === 'system'
        ? window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light'
        : theme;

    root.classList.toggle('dark', resolvedTheme === 'dark');
  }, [theme]);

  return (
    <ThemeContext.Provider value={{ theme, setTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}

export const useTheme = () => useContext(ThemeContext);
```

#### 글로벌 에러 바운더리

```typescript
// src/app/ErrorBoundary.tsx
import { Component, type ReactNode, type ErrorInfo } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('[ErrorBoundary]', error, errorInfo);
    // 프로덕션에서는 에러 리포팅 서비스로 전송
  }

  render() {
    if (this.state.hasError) {
      return (
        this.props.fallback ?? (
          <div className="flex h-screen items-center justify-center">
            <div className="text-center">
              <h1 className="text-2xl font-bold text-red-600">오류가 발생했습니다</h1>
              <p className="mt-2 text-gray-600">{this.state.error?.message}</p>
              <button
                className="mt-4 rounded bg-blue-600 px-4 py-2 text-white"
                onClick={() => { this.setState({ hasError: false, error: null }); }}
              >
                다시 시도
              </button>
            </div>
          </div>
        )
      );
    }
    return this.props.children;
  }
}
```

---

### 4.2 Dashboard Module

#### 목적
운영자가 한눈에 전체 로봇 플릿의 상태를 파악할 수 있는 대시보드를 제공한다. 실시간 업데이트 기반으로 운영 지표, 알림, 미션 큐를 표시한다.

#### DashboardPage.tsx

```typescript
// src/components/dashboard/DashboardPage.tsx
export default function DashboardPage() {
  return (
    <div className="grid grid-cols-12 gap-4">
      {/* 상단: KPI 카드 (4열) */}
      <div className="col-span-12">
        <KPICards />
      </div>

      {/* 중단 좌: 로봇 플릿 현황 */}
      <div className="col-span-8">
        <FleetOverview />
      </div>

      {/* 중단 우: 알림 피드 */}
      <div className="col-span-4">
        <AlertFeed />
      </div>

      {/* 하단 좌: 미션 대기열 */}
      <div className="col-span-6">
        <MissionQueue />
      </div>

      {/* 하단 우: 로봇 목록 */}
      <div className="col-span-6">
        <RobotList />
      </div>

      {/* 전체 하단: 2D 미니맵 */}
      <div className="col-span-12">
        <FleetMap />
      </div>
    </div>
  );
}
```

#### FleetOverview 컴포넌트

```typescript
// src/components/dashboard/FleetOverview.tsx
// 목적: 상태별 로봇 수를 실시간으로 표시한다 (IDLE/BUSY/ERROR/OFFLINE)
// 데이터: useRobotStore의 robots Map에서 집계
// 업데이트: WebSocket 텔레메트리로 자동 갱신 (robotStore 구독)
// UI: 4개 상태 카드 + 도넛 차트 (Recharts PieChart)

interface FleetOverviewProps {
  // 외부 props 없음 — 내부적으로 useRobotStore 구독
}

// 내부 상태:
// - statusCounts: Record<RobotStatus, number> — robotStore에서 실시간 계산
// 사용하는 라이브러리: Recharts (PieChart, Pie, Cell, Tooltip, Legend)
// 에러 처리: 로봇 데이터 없을 때 EmptyState 컴포넌트 표시
```

#### KPICards 컴포넌트

```typescript
// src/components/dashboard/KPICards.tsx
// 목적: 핵심 운영 지표를 카드 형태로 표시
// 카드 4개:
//   1. 오늘 완료 미션 수 (GET /api/missions?status=COMPLETED&since=today)
//   2. 평균 미션 소요 시간 (분) — 서버 계산값
//   3. 플릿 가동률 (%) — (IDLE+BUSY) / 전체 * 100
//   4. 활성 알림 수 — alertStore에서 집계
// 데이터: TanStack Query (REST) + Zustand (실시간)
// 업데이트 주기: REST 30초 간격 refetchInterval + 실시간 WebSocket 보완

interface KPIData {
  completedMissions: number;
  averageMissionTime: number; // minutes
  fleetUtilization: number;   // 0-100
  activeAlerts: number;
}
// Query Key: ['dashboard', 'kpis']
```

#### AlertFeed 컴포넌트

```typescript
// src/components/dashboard/AlertFeed.tsx
// 목적: 실시간 알림을 시간순으로 표시 (최신 상단)
// 데이터 소스: WebSocket 'alerts' 토픽 → alertStore에 추가
// 알림 유형:
//   - ROBOT_ERROR: 로봇 오류 (빨간)
//   - ZONE_VIOLATION: 금지 구역 침범 (주황)
//   - MISSION_FAILURE: 미션 실패 (빨간)
//   - LOW_BATTERY: 배터리 부족 (노랑)
//   - CONNECTION_LOST: 연결 끊김 (회색)
// 사용자 인터랙션:
//   - 알림 클릭 → 해당 로봇/미션 상세로 이동
//   - "읽음 처리" 버튼
//   - "모두 읽음" 버튼
//   - 유형별 필터
// 최대 표시: 100개, 가상 스크롤 적용 (큰 목록 성능)

interface Alert {
  id: string;
  type: AlertType;
  severity: 'info' | 'warning' | 'error' | 'critical';
  message: string;
  robotId?: string;
  missionId?: string;
  timestamp: string; // ISO 8601
  read: boolean;
}
```

#### MissionQueue 컴포넌트

```typescript
// src/components/dashboard/MissionQueue.tsx
// 목적: 대기 중인 미션과 실행 중인 미션을 표시
// 데이터: TanStack Query GET /api/missions?status=CREATED,ASSIGNED,EXECUTING
// 사용자 인터랙션:
//   - 드래그 앤 드롭으로 우선순위 변경 → PUT /api/missions/:id (priority 업데이트)
//   - 미션 클릭 → MissionDetail 모달
//   - "미션 취소" 버튼 → PUT /api/missions/:id (status: CANCELLED)
// 실시간 업데이트: WebSocket 'mission.status' 토픽으로 상태 변경 반영
// 드래그 앤 드롭: @dnd-kit/core 라이브러리 사용
```

#### RobotList 컴포넌트

```typescript
// src/components/dashboard/RobotList.tsx
// 목적: 등록된 모든 로봇을 테이블로 표시
// 데이터: TanStack Query GET /api/robots + robotStore 실시간 상태 병합
// 테이블 열: 이름, 모델, 상태, 배터리(%), 현재 미션, 위치, 마지막 업데이트
// 사용자 인터랙션:
//   - 열 헤더 클릭 → 정렬 (이름, 상태, 배터리)
//   - 검색바 → 이름 필터 (debounce 300ms)
//   - 상태 드롭다운 필터
//   - 행 클릭 → /viewer로 이동 (해당 로봇 선택 + 카메라 포커스)
//   - 행 호버 → 빠른 액션 (원격 제어, 미션 할당)
// StatusIndicator: 상태별 색상 dot + 라벨
//   - IDLE: 녹색 + "대기"
//   - BUSY: 파란색 + "작업중"
//   - ERROR: 빨간색 + "오류"
//   - OFFLINE: 회색 + "오프라인"
```

---

### 4.3 3D Viewer Module (최복잡 모듈)

#### 목적
로봇, 포인트 클라우드 맵, 장애물, 센서 데이터를 3D로 시각화한다. 실시간 텔레메트리 기반 로봇 포즈 업데이트, Potree LOD 포인트 클라우드 렌더링, 사용자 상호작용(선택, 측정)을 지원한다.

#### SceneManager.tsx — 씬 오케스트레이션

```typescript
// src/components/viewer3d/SceneManager.tsx
import { Canvas } from '@react-three/fiber';
import { CameraController } from './CameraController';
import { RobotRenderer } from './RobotRenderer';
import { PointCloudRenderer } from './PointCloudRenderer';
import { ObstacleLayer } from './ObstacleLayer';
import { SensorVisualization } from './SensorVisualization';
import { GridHelper } from './GridHelper';
import { SelectionManager } from './SelectionManager';
import { useRobotStore } from '@/stores/robotStore';
import { useViewerStore } from '@/stores/viewerStore';
import * as THREE from 'three';

interface SceneManagerProps {
  mapId: string;
}

export function SceneManager({ mapId }: SceneManagerProps) {
  const robots = useRobotStore((s) => s.robots);
  const viewMode = useViewerStore((s) => s.viewMode);

  return (
    <Canvas
      gl={{
        antialias: true,
        toneMapping: THREE.ACESFilmicToneMapping,
        outputColorSpace: THREE.SRGBColorSpace,
        powerPreference: 'high-performance',
      }}
      camera={{ position: [10, 10, 10], fov: 60, near: 0.1, far: 10000 }}
      dpr={[1, 2]} // 레티나 대응, 최대 2배
      performance={{ min: 0.5 }} // 성능 저하 시 해상도 자동 조절
      shadows
    >
      {/* 조명 설정 */}
      <ambientLight intensity={0.4} />
      <directionalLight
        position={[50, 50, 50]}
        intensity={0.8}
        castShadow
        shadow-mapSize={[2048, 2048]}
      />
      <hemisphereLight intensity={0.3} />

      {/* 카메라 컨트롤 */}
      <CameraController />

      {/* 그리드 + 축 */}
      <GridHelper />

      {/* 포인트 클라우드 (맵) */}
      {(viewMode === 'real' || viewMode === 'hybrid') && (
        <PointCloudRenderer mapId={mapId} />
      )}

      {/* 장애물 레이어 */}
      <ObstacleLayer mapId={mapId} />

      {/* 로봇 렌더링 (각 로봇별) */}
      {Array.from(robots.values()).map((robot) => (
        <RobotRenderer key={robot.id} robot={robot} />
      ))}

      {/* 센서 시각화 */}
      <SensorVisualization />

      {/* 선택 매니저 (raycasting) */}
      <SelectionManager />
    </Canvas>
  );
}
```

#### CameraController.tsx — 카메라 모드 전환

```typescript
// src/components/viewer3d/CameraController.tsx
// 세 가지 카메라 모드:
// 1. orbit: OrbitControls — 기본, 마우스 좌클릭 회전, 우클릭 팬, 휠 줌
// 2. fly: FlyControls — WASD 이동, 마우스 시점 회전 (자유 비행)
// 3. top-down: OrthographicCamera + 2D 팬/줌 (맵 편집 시 사용)

// 사용하는 Drei 컴포넌트: OrbitControls, FlyControls, OrthographicCamera, PerspectiveCamera
// 상태: useViewerStore.cameraMode

// 카메라 애니메이션: 모드 전환 시 gsap 또는 Three.js 내장 lerp로 부드러운 전환
// 로봇 포커스: selectRobot 시 카메라가 해당 로봇 위치로 이동 (lookAt 변경)

interface CameraControllerProps {
  // 외부 props 없음 — viewerStore 구독
}

// 구현 핵심:
// - orbit 모드: <OrbitControls enableDamping dampingFactor={0.05} maxPolarAngle={Math.PI / 2} />
// - fly 모드: <FlyControls movementSpeed={10} rollSpeed={0.5} />
// - top-down 모드: <OrthographicCamera makeDefault position={[0, 100, 0]} rotation={[-Math.PI/2, 0, 0]} zoom={10} />
```

#### RobotRenderer.tsx — 로봇 glTF 모델 렌더링

```typescript
// src/components/viewer3d/RobotRenderer.tsx
import { useGLTF, Html } from '@react-three/drei';
import { useFrame } from '@react-three/fiber';
import { useRef } from 'react';
import * as THREE from 'three';
import type { Robot } from '@/types/robot.types';

interface RobotRendererProps {
  robot: Robot;
}

// 구현 요구사항:
// 1. glTF 모델 로딩: useGLTF로 로봇 모델 로드 (modelId별 다른 모델)
// 2. 포즈 업데이트: robot.pose (position + quaternion) 변경 시 스무스 보간
//    - useFrame 내에서 lerp/slerp로 부드러운 이동 (10-30Hz 텔레메트리)
//    - lerpFactor = 0.1 (부드러운 보간) ~ 0.5 (빠른 추적)
// 3. 상태별 색상: IDLE=녹색, BUSY=파란색, ERROR=빨간색 빛 이펙트
// 4. 선택 시: 하이라이트 아웃라인 (Drei의 <Outlines />)
// 5. 라벨: Html 오버레이로 로봇 이름 + 상태 표시 (2D DOM 오버레이)
// 6. 궤적 트레일: RobotTrail 서브컴포넌트로 최근 N개 위치를 라인으로 표시

export function RobotRenderer({ robot }: RobotRendererProps) {
  const meshRef = useRef<THREE.Group>(null);
  const { scene: model } = useGLTF(`/api/assets/${robot.modelId}/file`);
  const targetPosition = new THREE.Vector3(
    robot.pose.position.x,
    robot.pose.position.y,
    robot.pose.position.z,
  );
  const targetQuaternion = new THREE.Quaternion(
    robot.pose.orientation.x,
    robot.pose.orientation.y,
    robot.pose.orientation.z,
    robot.pose.orientation.w,
  );

  useFrame((_, delta) => {
    if (meshRef.current == null) return;
    // 위치 보간
    meshRef.current.position.lerp(targetPosition, Math.min(delta * 10, 1));
    // 회전 보간
    meshRef.current.quaternion.slerp(targetQuaternion, Math.min(delta * 10, 1));
  });

  return (
    <group ref={meshRef}>
      <primitive object={model.clone()} scale={1} />
      {/* 이름 라벨 */}
      <Html position={[0, 2, 0]} center distanceFactor={10}>
        <div className="rounded bg-black/70 px-2 py-1 text-xs text-white whitespace-nowrap">
          {robot.name}
        </div>
      </Html>
      {/* 궤적 트레일 */}
      <RobotTrail robotId={robot.id} />
    </group>
  );
}
```

#### PointCloudRenderer.tsx — Potree 포인트 클라우드

```typescript
// src/components/viewer3d/PointCloudRenderer.tsx
// 목적: Potree 기반 LOD 포인트 클라우드 렌더링

// 구현 요구사항:
// 1. Potree octree 구조의 타일을 HTTP GET /api/maps/:id/tiles/:nodeId?lod=N 으로 로딩
// 2. 카메라 프러스텀 기반 가시성 판정 → 보이는 노드만 로딩
// 3. 카메라 거리에 따른 LOD 자동 조절 (가까울수록 상세, 멀수록 간략)
// 4. Point budget 제어: 최대 렌더링 포인트 수 (기본 5,000,000, 설정 가능)
// 5. 포인트 스타일: 크기, 색상 모드 (RGB/높이/강도/분류), 형태 (circle/square)
// 6. Web Worker에서 포인트 데이터 디코딩 (메인 스레드 블로킹 방지)

// 성능 최적화:
// - BufferGeometry + PointsMaterial 사용 (instanced rendering 불필요, 포인트 특성)
// - 타일 캐시: LRU 캐시로 최근 로드한 타일 유지 (메모리 한도 설정)
// - 프로그레시브 로딩: 낮은 LOD 먼저 표시 → 점진적으로 고해상도 추가
// - requestIdleCallback으로 비활성 시간에 프리페치

interface PointCloudRendererProps {
  mapId: string;
  pointBudget?: number;        // 기본 5_000_000
  pointSize?: number;          // 기본 1.0
  colorMode?: 'rgb' | 'height' | 'intensity' | 'classification';
}

// 상태 관리: usePointCloud 훅 사용
// 에러 처리: 타일 로딩 실패 시 재시도 (3회), 실패한 노드 표시
```

#### ObstacleLayer.tsx — 장애물 렌더링

```typescript
// src/components/viewer3d/ObstacleLayer.tsx
// 목적: 정적 장애물(맵에서 정의)과 동적 장애물(텔레메트리에서 수신)을 렌더링

// 정적 장애물: GET /api/maps/:id → obstacles 배열
//   - 렌더링: BoxGeometry 또는 CylinderGeometry (타입별)
//   - 머티리얼: 반투명 빨간색 (opacity 0.3)
// 동적 장애물: WebSocket 텔레메트리의 obstacles 필드
//   - 렌더링: InstancedMesh (다수의 동적 장애물 효율적 렌더링)
//   - 머티리얼: 반투명 주황색, 펄스 애니메이션
//   - 자동 만료: 3초 내 업데이트 없으면 fadeout

interface ObstacleLayerProps {
  mapId: string;
}

// 정적 장애물 타입:
interface StaticObstacle {
  id: string;
  type: 'box' | 'cylinder' | 'mesh';
  position: Vector3;
  rotation: Vector3; // 오일러 각 (degrees)
  dimensions: { width: number; height: number; depth: number } | { radius: number; height: number };
}
```

#### SensorVisualization.tsx — 센서 시각화

```typescript
// src/components/viewer3d/SensorVisualization.tsx
// 목적: 선택된 로봇의 센서 데이터를 시각화

// LiDAR 스캔 오버레이:
//   - 2D LiDAR: 팬(부채꼴) 형태 (ShapeGeometry)
//   - 3D LiDAR: 포인트 클라우드 (BufferGeometry + Points)
//   - 색상: 거리에 따른 그라데이션 (가까움=녹색, 멀음=빨간색)
//   - 업데이트 주기: 텔레메트리와 동일 (10Hz)

// 카메라 프러스텀:
//   - CameraHelper 또는 커스텀 LineSegments로 시야각 표시
//   - 반투명 영역으로 시야 범위 시각화

// 표시 조건: selectedRobotId가 설정되고 뷰어 설정에서 센서 표시가 활성화된 경우
```

#### SelectionManager.tsx — 오브젝트 선택 시스템

```typescript
// src/components/viewer3d/SelectionManager.tsx
// 목적: 3D 씬 내 오브젝트 클릭 선택 관리

// 구현:
// 1. useThree의 raycaster를 사용하여 마우스 클릭 시 raycast
// 2. 교차 오브젝트 중 가장 가까운 것 선택
// 3. 선택 가능 오브젝트: 로봇, 장애물, 웨이포인트 노드
// 4. 선택 시: viewerStore.selectedObjectId 업데이트 → InfoPanel 표시
// 5. Shift+클릭: 다중 선택
// 6. 빈 공간 클릭: 선택 해제
// 7. 더블 클릭: 카메라 해당 오브젝트로 포커스 이동

// 사용자에게 보이는 효과:
// - 선택된 오브젝트: 하이라이트 아웃라인 (Drei <Outlines />)
// - 호버: 커서 변경 (pointer), 미세한 하이라이트
```

#### 성능 최적화 전략

```
1. Instanced Rendering: 동일 모델의 다수 로봇은 InstancedMesh 사용
   - 50대 로봇: 50개 draw call → 1개 draw call
   - 각 인스턴스의 matrix (position/rotation/scale) 만 업데이트

2. Frustum Culling: Three.js 내장 프러스텀 컬링 활성화 (기본 켜짐)
   - 카메라 시야 밖 오브젝트는 렌더링 건너뜀

3. Level of Detail (LOD):
   - Three.js LOD 오브젝트로 거리별 모델 상세도 전환
   - 가까움: 원본 glTF / 멀음: simplified mesh / 매우 멀음: billboard sprite

4. Point Cloud 최적화:
   - Potree octree LOD: 카메라 거리/각도에 따른 동적 포인트 밀도 조절
   - Point budget: GPU 메모리 한도 내에서 포인트 수 제한
   - Web Worker 디코딩: 메인 스레드 프레임 드롭 방지

5. WebGPU Path (실험적):
   - navigator.gpu 지원 시 WebGPU 렌더러로 전환
   - Three.js WebGPURenderer 사용
   - 컴퓨트 셰이더로 포인트 클라우드 처리 가속

6. 메모리 관리:
   - glTF 모델 캐시 (useGLTF.preload)
   - 안 보이는 씬의 geometry/texture dispose
   - WeakRef 기반 텍스처 캐시
```

---

### 4.4 Map Editor Module

#### 목적
운영자가 포인트 클라우드 맵 위에 로드맵 그래프(웨이포인트 + 엣지)와 시맨틱 영역(금지구역, 속도제한 등)을 정의하고 편집할 수 있는 도구를 제공한다.

#### MapEditorPage.tsx — 편집 페이지 구성

```typescript
// src/components/map-editor/MapEditorPage.tsx
// 레이아웃: 좌측에 3D 뷰어 (70%), 우측에 속성 패널 (30%)
// 상단에 에디터 도구 모음

// 구조:
// ┌──────────────────────────────┬────────────────┐
// │      EditorToolbar           │                │
// ├──────────────────────────────┤ Properties     │
// │                              │ Panel          │
// │   3D Scene (SceneManager    │ (Node/Edge/    │
// │   + 에디터 오버레이)         │  Region 속성)  │
// │                              │                │
// ├──────────────────────────────┤                │
// │ MapVersionHistory            │                │
// └──────────────────────────────┴────────────────┘
```

#### EditorToolbar.tsx — 도구 모음

```typescript
// src/components/map-editor/EditorToolbar.tsx
// 도구 목록 (editorStore.tool 연동):
// 1. SELECT: 선택/이동 도구 (기본)
// 2. ADD_NODE: 클릭하여 웨이포인트 노드 추가
// 3. ADD_EDGE: 소스 노드 클릭 → 타겟 노드 클릭으로 엣지 추가
// 4. DRAW_REGION: 클릭하여 폴리곤 꼭짓점 추가, 더블클릭으로 완성
// 5. ADD_OBSTACLE: 클릭하여 정적 장애물 배치
// 6. MEASURE: 두 점 사이 거리 측정
// 7. DELETE: 클릭하여 오브젝트 삭제

// 추가 버튼:
// - Undo (Ctrl+Z) / Redo (Ctrl+Shift+Z)
// - Save (Ctrl+S)
// - Import Point Cloud
// - Grid 스냅 토글 (기본 ON, 0.5m 간격)
// - 카메라 모드 전환

type EditorTool = 'select' | 'addNode' | 'addEdge' | 'drawRegion' | 'addObstacle' | 'measure' | 'delete';
```

#### RoadmapGraphEditor.tsx — 로드맵 그래프 편집

```typescript
// src/components/map-editor/RoadmapGraphEditor.tsx
// 목적: 3D 씬 위에서 웨이포인트 노드와 엣지를 인터랙티브하게 편집

// 노드 추가 (ADD_NODE 도구):
// 1. 사용자가 3D 씬 클릭
// 2. Raycaster로 포인트 클라우드/바닥 교차점 계산
// 3. 교차점에 새 노드 생성 (기본 타입: waypoint)
// 4. editorStore에 AddNodeCommand 추가 (undo 지원)
// 5. 노드 ID는 uuid 생성

// 노드 이동 (SELECT 도구):
// 1. 노드 클릭 → 선택
// 2. 드래그 → TransformControls로 이동 (Drei <TransformControls />)
// 3. 드래그 완료 → MoveNodeCommand 추가

// 노드 삭제 (DELETE 도구 또는 Delete 키):
// 1. 노드 선택 상태에서 삭제
// 2. 연결된 엣지도 함께 삭제
// 3. DeleteNodeCommand 추가 (연결 엣지 정보 포함, undo 시 복원)

// 엣지 추가 (ADD_EDGE 도구):
// 1. 소스 노드 클릭 → 소스 하이라이트
// 2. 타겟 노드 클릭 → 엣지 생성
// 3. 동일 노드 쌍 중복 엣지 방지
// 4. AddEdgeCommand 추가

// 엣지 삭제:
// 1. 엣지 클릭 → 선택
// 2. Delete → DeleteEdgeCommand 추가
```

#### NodePropertiesPanel.tsx — 노드 속성 편집

```typescript
// src/components/map-editor/NodePropertiesPanel.tsx
// 선택된 노드의 속성을 편집하는 사이드 패널

// 편집 가능 속성:
interface NodeProperties {
  name: string;              // 노드 이름 (사용자 정의, 예: "충전소-A1")
  type: NodeType;            // waypoint | charging | dock | elevator | parking
  position: Vector3;         // x, y, z 좌표 (수치 입력 직접 편집 가능)
  waitBehavior: {
    enabled: boolean;        // 이 노드에서 대기 여부
    duration: number;        // 대기 시간 (초), 0 = 무한
    condition?: string;      // 조건식 (예: "door.open == true")
  };
  properties: Record<string, any>; // 추가 커스텀 속성 (key-value 편집기)
}

type NodeType = 'waypoint' | 'charging' | 'dock' | 'elevator' | 'parking';

// 변경 시 즉시 반영 (debounce 300ms → editorStore → 3D 씬 업데이트)
// 각 변경은 UpdateNodeCommand로 undo 지원
```

#### EdgePropertiesPanel.tsx — 엣지 속성 편집

```typescript
// src/components/map-editor/EdgePropertiesPanel.tsx
// 선택된 엣지의 속성을 편집하는 사이드 패널

interface EdgeProperties {
  maxSpeed: number;              // 최대 허용 속도 (m/s), 기본 1.0
  direction: 'uni' | 'bi';      // 단방향 / 양방향
  allowedRobotTypes: string[];   // 허용 로봇 타입 (비어있으면 전체 허용)
  weight: number;                // 경로 탐색 가중치 (기본 1.0, 높을수록 비선호)
  width: number;                 // 통로 폭 (m), 로봇 크기 대비 통과 가능 판단
}

// 방향 전환 시 3D 씬의 화살표 방향 즉시 업데이트
// allowedRobotTypes는 다중 선택 체크박스 (등록된 로봇 모델 목록에서 선택)
```

#### SemanticRegionEditor.tsx — 시맨틱 영역 편집

```typescript
// src/components/map-editor/SemanticRegionEditor.tsx
// 목적: 3D 씬 위에 폴리곤 영역을 그려서 시맨틱 의미를 부여

// 영역 그리기 (DRAW_REGION 도구):
// 1. 클릭할 때마다 폴리곤 꼭짓점 추가
// 2. 임시 라인으로 현재까지의 폴리곤 표시
// 3. 마우스 이동 시 마지막 꼭짓점 → 현재 마우스 위치 프리뷰 라인
// 4. 더블클릭 → 폴리곤 완성 (최소 3개 꼭짓점)
// 5. Esc → 그리기 취소
// 6. AddRegionCommand 추가

// 영역 렌더링:
// - ShapeGeometry (2D 폴리곤을 맵 표면에 매핑)
// - 반투명 머티리얼 (타입별 색상)
// - 외곽선 하이라이트 (LineLoop)

// 영역 타입별 색상:
type RegionType = 'noGo' | 'speedLimit' | 'charging' | 'loadingDock' | 'elevator' | 'custom';
// noGo: 빨간색 (opacity 0.3)
// speedLimit: 노란색 (opacity 0.2)
// charging: 녹색 (opacity 0.2)
// loadingDock: 파란색 (opacity 0.2)
// elevator: 보라색 (opacity 0.2)
// custom: 회색 (opacity 0.2)

// 영역 속성 (RegionPropertiesPanel):
interface RegionProperties {
  name: string;
  type: RegionType;
  speedLimit?: number;          // speedLimit 타입일 때 (m/s)
  maxRobots?: number;           // 동시 진입 가능 로봇 수 (0=무제한)
  priority?: number;            // 우선순위 규칙
  schedule?: {                  // 시간대별 활성화 (옵션)
    startTime: string;          // "HH:mm"
    endTime: string;            // "HH:mm"
    daysOfWeek: number[];       // 0=일, 1=월, ...
  };
}
```

#### UndoRedoManager — Command Pattern 구현

```typescript
// src/components/map-editor/UndoRedoManager.tsx (로직은 editorStore에)
// src/stores/editorStore.ts 내 구현

// Command 인터페이스:
interface EditorCommand {
  type: string;
  execute(): void;   // 실행 (redo)
  undo(): void;      // 실행 취소
  description: string; // 로그 표시용
}

// 구체 커맨드 예시:
class AddNodeCommand implements EditorCommand {
  type = 'ADD_NODE';
  description: string;
  constructor(private node: RoadmapNode, private store: MapStore) {
    this.description = `노드 추가: ${node.name}`;
  }
  execute() { this.store.addNode(this.node); }
  undo() { this.store.removeNode(this.node.id); }
}

class MoveNodeCommand implements EditorCommand {
  type = 'MOVE_NODE';
  description: string;
  constructor(
    private nodeId: string,
    private oldPosition: Vector3,
    private newPosition: Vector3,
    private store: MapStore,
  ) {
    this.description = `노드 이동: ${nodeId}`;
  }
  execute() { this.store.updateNodePosition(this.nodeId, this.newPosition); }
  undo() { this.store.updateNodePosition(this.nodeId, this.oldPosition); }
}

// editorStore 상태:
interface EditorState {
  tool: EditorTool;
  undoStack: EditorCommand[];     // 최대 100개
  redoStack: EditorCommand[];
  gridSnap: boolean;
  gridSize: number;               // 0.5m
  // Actions:
  setTool: (tool: EditorTool) => void;
  executeCommand: (cmd: EditorCommand) => void;
  undo: () => void;
  redo: () => void;
}

// executeCommand: cmd.execute() 호출 후 undoStack에 push, redoStack 비움
// undo: undoStack에서 pop, cmd.undo() 호출, redoStack에 push
// redo: redoStack에서 pop, cmd.execute() 호출, undoStack에 push
```

#### PointCloudImporter.tsx — 포인트 클라우드 업로드

```typescript
// src/components/map-editor/PointCloudImporter.tsx
// 목적: LAS/PLY/PCD 파일을 업로드하여 맵의 포인트 클라우드로 등록

// 플로우:
// 1. 파일 선택 (drag & drop 또는 파일 탐색기)
//    - 지원 형식: .las, .laz, .ply, .pcd
//    - 최대 파일 크기: 2GB
// 2. 파일 업로드: POST /api/assets/upload (multipart/form-data)
//    - 업로드 진행률 표시 (XMLHttpRequest progress event)
// 3. 서버 처리 대기: 업로드 완료 후 서버가 Potree 타일로 변환
//    - 처리 상태 폴링: GET /api/assets/:id (status: 'processing' → 'ready')
//    - 폴링 간격: 2초
// 4. 완료 시: 맵에 포인트 클라우드 연결 (PUT /api/maps/:id, pointCloudAssetId 설정)
// 5. 3D 씬에 로드

// UI:
// - 드래그 앤 드롭 영역 (점선 테두리, 파일 아이콘)
// - 파일 정보 (이름, 크기, 형식)
// - 업로드 프로그레스 바
// - 처리 상태 스피너 + 진행률
// - 완료 메시지 + "씬에 로드" 버튼
```

#### 맵 저장/로드 및 버전 관리

```typescript
// 저장: PUT /api/maps/:id/roadmap (roadmap graph) + PUT /api/maps/:id/semantic (regions)
// Ctrl+S 단축키 바인딩
// 저장 시 optimistic update + 서버 확인 후 확정
// 버전 관리: 서버에서 version 번호 자동 증가
// MapVersionHistory: GET /api/maps/:id/versions → 버전 목록 표시
// 특정 버전 복원: POST /api/maps/:id/restore?version=N
```

---

### 4.5 Mission Control Module

#### 목적
미션 생성, 할당, 모니터링, 교통 규칙 설정을 관리한다.

#### MissionCreator.tsx — 미션 생성 위자드

```typescript
// src/components/mission-control/MissionCreator.tsx
// 4단계 위자드 폼 (react-hook-form + zod 검증):

// Step 1: 로봇 선택
//   - 드롭다운으로 사용 가능한 로봇 선택 (status: IDLE만 표시)
//   - 또는 "자동 할당" 선택 (서버가 최적 로봇 결정)

// Step 2: 경로 설정
//   - 3D 미니 뷰어에서 시작 웨이포인트 클릭
//   - 도착 웨이포인트 클릭
//   - 중간 경유지 추가 (옵션)
//   - 서버 경로 미리보기: POST /api/maps/:id/pathfind (start, end) → 경로 표시

// Step 3: 작업 단계 추가 (옵션)
//   - 각 웨이포인트에서 수행할 작업 정의
//   - 작업 유형: 'wait' (대기), 'pickup' (화물 적재), 'dropoff' (화물 하역), 'charge' (충전), 'custom'
//   - 각 작업에 파라미터 설정 (대기 시간, 화물 ID 등)

// Step 4: 확인 및 제출
//   - 미션 요약 표시 (로봇, 경로, 작업 단계, 예상 소요시간)
//   - 우선순위 설정 (1-10, 기본 5)
//   - "미션 생성" 버튼 → POST /api/missions
//   - "미션 생성 및 즉시 할당" → POST /api/missions + POST /api/missions/:id/assign

// Zod 검증 스키마:
const missionCreateSchema = z.object({
  robotId: z.string().uuid().optional(), // 자동 할당이면 없음
  autoAssign: z.boolean().default(false),
  mapId: z.string().uuid(),
  startNodeId: z.string().uuid(),
  endNodeId: z.string().uuid(),
  waypointIds: z.array(z.string().uuid()).default([]),
  steps: z.array(z.object({
    nodeId: z.string().uuid(),
    action: z.enum(['wait', 'pickup', 'dropoff', 'charge', 'custom']),
    params: z.record(z.any()).default({}),
    timeout: z.number().positive().optional(), // seconds
  })).default([]),
  priority: z.number().int().min(1).max(10).default(5),
  scheduledAt: z.string().datetime().optional(), // 예약 실행 (옵션)
});
```

#### MissionList.tsx — 미션 목록

```typescript
// src/components/mission-control/MissionList.tsx
// 목적: 모든 미션을 테이블로 표시 (필터/정렬/페이지네이션)

// 테이블 열:
// | ID (짧은) | 로봇 이름 | 상태 | 우선순위 | 시작 시간 | 종료 시간 | 소요 시간 | 액션 |
// 데이터: TanStack Query GET /api/missions?page=1&limit=20&status=...&sort=createdAt&order=desc
// Query Key: ['missions', { page, limit, status, sort, order, search }]

// 필터:
// - 상태: 다중 선택 체크박스 (CREATED, ASSIGNED, EXECUTING, COMPLETED, FAILED, CANCELLED)
// - 로봇: 드롭다운
// - 날짜 범위: DatePicker

// 정렬: 열 헤더 클릭 (createdAt, priority, status)
// 페이지네이션: 하단 페이지 네비게이터 (20개/페이지)

// 행 액션:
// - 상세 보기: MissionDetail 모달
// - 취소: PUT /api/missions/:id (status: CANCELLED) — CREATED/ASSIGNED 상태만
// - 재시도: POST /api/missions (동일 파라미터로 새 미션 생성) — FAILED 상태만
```

#### MissionDetail.tsx — 미션 상세

```typescript
// src/components/mission-control/MissionDetail.tsx
// 목적: 개별 미션의 상세 정보와 실행 진행 상황을 표시

// 데이터: GET /api/missions/:id
// 실시간 업데이트: WebSocket 'mission.status' 토픽 구독

// 표시 내용:
// 1. 미션 헤더: ID, 상태 배지, 로봇 이름, 우선순위
// 2. 타임라인 (MissionTimeline 컴포넌트):
//    - 수직 타임라인으로 각 단계 표시
//    - 완료된 단계: 녹색 체크
//    - 현재 진행 중: 파란색 스피너
//    - 대기 중: 회색
//    - 실패: 빨간색 X
// 3. 시간 정보: 생성 시간, 시작 시간, 예상 완료 시간, 실제 완료 시간
// 4. 경로 시각화: 3D 미니 뷰어에 미션 경로 표시
// 5. 로봇 현재 위치: 미션 경로 위 실시간 표시

interface MissionDetailData {
  id: string;
  robotId: string;
  robotName: string;
  status: MissionStatus;
  priority: number;
  steps: MissionStepDetail[];
  createdAt: string;
  startedAt: string | null;
  estimatedEndAt: string | null;
  completedAt: string | null;
  errorMessage: string | null;
}

interface MissionStepDetail {
  id: string;
  nodeId: string;
  nodeName: string;
  action: string;
  status: 'pending' | 'executing' | 'completed' | 'failed' | 'skipped';
  startedAt: string | null;
  completedAt: string | null;
  error: string | null;
}
```

#### TrafficRuleEditor.tsx — 교통 규칙 편집

```typescript
// src/components/mission-control/TrafficRuleEditor.tsx
// 목적: 시맨틱 영역별 교통 규칙을 정의

// 규칙 정의:
interface TrafficRule {
  id: string;
  regionId: string;           // 적용 대상 시맨틱 영역
  maxRobots: number;          // 동시 진입 가능 로봇 수
  priority: 'fifo' | 'priority' | 'nearest'; // 우선순위 정책
  oneWay: boolean;            // 단방향 통행 여부
  speedLimit: number;         // 최대 속도 (m/s)
  allowedRobotTypes: string[]; // 진입 가능 로봇 타입 (비어있으면 전체)
}

// 충돌 해소 설정 (ConflictResolutionPanel):
interface ConflictResolution {
  strategy: 'wait' | 'reroute' | 'priority'; // 대기/우회/우선순위
  waitTimeout: number;        // 대기 전략 시 최대 대기 시간 (초)
  rerouteThreshold: number;   // 우회 결정까지의 대기 시간 (초)
  deadlockDetection: boolean; // 교착 상태 탐지 활성화
  deadlockTimeout: number;    // 교착 판정 시간 (초)
}

// UI: 시맨틱 영역 목록 + 각 영역별 규칙 편집 폼
// 저장: PUT /api/maps/:id/traffic-rules
```

#### MissionMonitor.tsx — 실시간 미션 모니터

```typescript
// src/components/mission-control/MissionMonitor.tsx
// 목적: 실행 중인 모든 미션을 2D/3D 맵 위에 실시간 오버레이

// 표시 내용:
// - 각 로봇의 현재 위치 (실시간)
// - 각 로봇의 할당된 경로 (라인)
// - 미션 진행률 (경로 위 진행 표시)
// - 충돌 위험 경고 (로봇 간 근접 시 빨간 원)

// 데이터 소스:
// - robotStore: 로봇 포즈 (WebSocket 텔레메트리)
// - missionStore: 활성 미션 목록
// - mapStore: 로드맵 그래프 (경로 렌더링)

// 뷰 모드: 2D (top-down Canvas2D) 또는 3D (SceneManager 재사용)
```

---

### 4.6 Remote Control Module

#### 목적
운영자가 특정 로봇을 원격으로 조종할 수 있는 인터페이스를 제공한다. 가상 조이스틱, 키보드 제어, 카메라 피드, 긴급 정지 기능을 포함한다.

#### RemoteControlPage.tsx — 원격 제어 페이지

```typescript
// src/components/remote-control/RemoteControlPage.tsx
// URL: /remote/:robotId
// 레이아웃:
// ┌───────────────────────────────────────────┐
// │ ConnectionStatus          EmergencyStop   │
// ├─────────────────────┬─────────────────────┤
// │                     │   TelemetryPanel    │
// │   CameraFeed        │   - 속도            │
// │   (WebRTC 영상)     │   - 배터리          │
// │                     │   - 위치            │
// │                     │   - 오류            │
// ├─────────────────────┤                     │
// │  JoystickController │   SpeedControl      │
// │  (가상 조이스틱)    │                     │
// └─────────────────────┴─────────────────────┘

// 진입 시:
// 1. robotId 파라미터 추출
// 2. WebSocket으로 해당 로봇 텔레메트리 구독
// 3. WebRTC로 카메라 스트림 연결
// 4. 원격 제어 모드 활성화: POST /api/robots/:id/remote-control (lock 획득)

// 이탈 시:
// 1. 속도 명령 0으로 전송 (안전 정지)
// 2. WebRTC 연결 해제
// 3. 원격 제어 모드 해제: DELETE /api/robots/:id/remote-control
// 4. useEffect cleanup에서 보장
```

#### JoystickController.tsx — 가상 조이스틱

```typescript
// src/components/remote-control/JoystickController.tsx
// 두 가지 입력 방법:

// 1. 화면 조이스틱 (nipplejs):
//    - 좌측: 전진/후진 (linear.x)
//    - 우측: 좌회전/우회전 (angular.z)
//    - 조이스틱 각도와 거리를 linear/angular 속도로 변환
//    - 터치/마우스 모두 지원

// 2. 키보드 (WASD):
//    - W: 전진 (+linear.x)
//    - S: 후진 (-linear.x)
//    - A: 좌회전 (+angular.z)
//    - D: 우회전 (-angular.z)
//    - Shift: 부스트 (속도 2배, maxSpeed 이내)
//    - Space: 즉시 정지

// 명령 전송:
// - WebSocket으로 velocity command 전송
// - 전송 주기: 10Hz (100ms 간격, useThrottle 사용)
// - 메시지 포맷: { type: 'remote_control.velocity', robotId, payload: { linear: {x,y,z}, angular: {x,y,z} } }
// - 조이스틱 놓으면 자동으로 속도 0 전송 (안전)

// maxSpeed 제한: SpeedControl 슬라이더 값으로 상한 제한
// linear.x 범위: [-maxSpeed, +maxSpeed]
// angular.z 범위: [-maxAngular, +maxAngular]
```

#### CameraFeed.tsx — WebRTC 카메라

```typescript
// src/components/remote-control/CameraFeed.tsx
// 목적: 로봇 카메라의 실시간 영상을 WebRTC로 수신하여 표시

// 연결 플로우:
// 1. 시그널링 시작: WebSocket 'webrtc.offer' 메시지 수신 대기
// 2. SDP Offer 수신 → RTCPeerConnection 생성
// 3. SDP Answer 생성 → WebSocket 'webrtc.answer' 전송
// 4. ICE candidate 교환 (trickle ICE)
// 5. 미디어 스트림 수신 → <video> 엘리먼트에 연결

// STUN/TURN 설정:
const rtcConfig: RTCConfiguration = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    {
      urls: 'turn:turn.example.com:3478',
      username: 'amr',
      credential: 'secret', // 환경변수에서 로딩
    },
  ],
};

// UI:
// - <video> 엘리먼트 (object-fit: cover)
// - 전체화면 토글 버튼
// - 연결 상태 오버레이 (연결 중: 스피너, 실패: 재연결 버튼)
// - 지연 시간 표시 (RTCPeerConnection.getStats로 측정)
// 에러 처리: 연결 실패 시 3초 후 자동 재시도 (최대 5회)
```

#### EmergencyStop.tsx — 긴급 정지

```typescript
// src/components/remote-control/EmergencyStop.tsx
// 목적: 즉시 로봇을 정지시키는 긴급 버튼

// UI: 큰 빨간 원형 버튼, 화면 우상단 고정
// 크기: 80px x 80px, 항상 최상위 z-index
// 디자인: 빨간 배경, 흰색 "STOP" 텍스트, 누를 때 약간 들어가는 효과
// 접근성: aria-label="긴급 정지", role="button"

// 동작:
// 1. 클릭 시 즉시 WebSocket 전송: { type: 'remote_control.emergency_stop', robotId }
// 2. REST 백업: POST /api/robots/:id/emergency-stop (WebSocket 실패 대비)
// 3. 화면에 "긴급 정지 활성화" 경고 오버레이 표시
// 4. 해제: "정지 해제" 버튼 클릭 → POST /api/robots/:id/resume
// 키보드 단축키: Escape 키로도 긴급 정지 발동
```

#### TelemetryPanel.tsx — 실시간 텔레메트리

```typescript
// src/components/remote-control/TelemetryPanel.tsx
// 표시 항목:
// - 선속도: linear velocity (m/s), 게이지 표시
// - 각속도: angular velocity (rad/s), 게이지 표시
// - 배터리: % + 아이콘 (색상: >50% 녹색, 20-50% 노랑, <20% 빨강)
// - 위치: x, y, z 좌표 (소수점 2자리)
// - 방향: yaw 각도 (degrees)
// - 오류 목록: 현재 활성 오류 (빨간 텍스트)
// - 업타임: 로봇 가동 시간

// 데이터 소스: robotStore (WebSocket 텔레메트리 구독)
// 업데이트 주기: 10Hz (100ms)
// 렌더링 최적화: memo + 개별 필드 구독 (Zustand selector)
```

---

### 4.7 Settings Module

#### 목적
시스템 관리 기능 제공: 로봇 등록/설정, 사용자 관리, 플러그인 관리, 시스템 설정.

#### SettingsPage.tsx — 탭 기반 설정

```typescript
// src/components/settings/SettingsPage.tsx
// Radix UI Tabs 기반
// 탭: 로봇 설정 | 사용자 관리 | 플러그인 | 시스템

// 접근 제어:
// - "사용자 관리" 탭: admin 역할만 표시
// - "시스템" 탭: admin 역할만 표시
// - "로봇 설정": admin + operator
// - "플러그인": admin
```

#### RobotConfig.tsx — 로봇 설정

```typescript
// src/components/settings/RobotConfig.tsx
// 목적: 로봇 등록, 수정, 삭제

// 기능:
// 1. 로봇 목록: GET /api/robots → 카드 또는 테이블 뷰
// 2. 로봇 등록: "새 로봇 추가" 버튼 → 모달 폼
//    - 필수: 이름, 모델 타입 (드롭다운), IP 주소
//    - 선택: 센서 구성, 주행 파라미터
//    - POST /api/robots
// 3. 로봇 수정: 카드 클릭 → 편집 모달
//    - 이름, 모델, 센서, 주행 파라미터 수정
//    - PUT /api/robots/:id
// 4. 로봇 삭제: 삭제 버튼 → 확인 다이얼로그 → DELETE /api/robots/:id

interface RobotConfig {
  name: string;
  modelId: string;               // glTF 모델 에셋 ID
  ipAddress: string;
  port: number;                  // 기본 9090
  sensors: SensorConfig[];
  drivingParams: {
    maxLinearSpeed: number;      // m/s
    maxAngularSpeed: number;     // rad/s
    acceleration: number;        // m/s^2
    deceleration: number;        // m/s^2
    footprint: Vector3[];        // 로봇 풋프린트 폴리곤 (충돌 판정용)
  };
}

interface SensorConfig {
  type: 'lidar2d' | 'lidar3d' | 'camera' | 'imu' | 'encoder';
  name: string;
  topic: string;                 // ROS 토픽명 또는 내부 토픽
  frameId: string;               // 센서 프레임 ID
  transform: Pose3D;             // 로봇 기준 센서 위치/방향
}
```

#### UserManagement.tsx — 사용자 관리

```typescript
// src/components/settings/UserManagement.tsx
// 목적: 사용자 CRUD + 역할 할당

// 역할:
// - admin: 모든 기능 접근 가능
// - operator: 로봇 제어, 미션 관리, 맵 편집 가능
// - viewer: 읽기 전용 (대시보드, 3D 뷰어 조회만)

// CRUD:
// - GET /api/users → 사용자 목록 테이블
// - POST /api/users → 새 사용자 등록 (이름, 이메일, 비밀번호, 역할)
// - PUT /api/users/:id → 사용자 정보 수정
// - DELETE /api/users/:id → 사용자 삭제 (확인 다이얼로그)

// 비밀번호 정책: 최소 8자, 영문+숫자+특수문자 조합 (zod 검증)
```

#### PluginManager.tsx — 플러그인 관리

```typescript
// src/components/settings/PluginManager.tsx
// 목적: 설치된 플러그인 목록 표시, 활성화/비활성화, 설정 관리

// 데이터: GET /api/plugins → 플러그인 목록
// 각 플러그인 카드:
//   - 이름, 설명, 버전, 상태 (active/inactive)
//   - 활성화 토글: PUT /api/plugins/:id/config { enabled: true/false }
//   - 설정 버튼: 플러그인별 설정 폼 (설정 스키마는 서버에서 제공)
//   - 플러그인 설정 JSON Schema → 동적 폼 렌더링 (react-hook-form + zod)
```

#### SystemSettings.tsx — 시스템 설정

```typescript
// src/components/settings/SystemSettings.tsx
// 설정 항목:

// 1. 테마: 라이트 / 다크 / 시스템 (라디오 버튼)
// 2. 언어: 한국어 / English (드롭다운) — i18n 대비
// 3. WebSocket 재접속 정책:
//    - 자동 재접속: 켜기/끄기 (Switch)
//    - 최대 재시도 횟수: 숫자 입력 (기본 10)
//    - 재시도 간격: 숫자 입력 (기본 3초)
//    - 백오프 배수: 숫자 입력 (기본 1.5)
// 4. 3D 뷰어 성능:
//    - Point budget: 슬라이더 (1M ~ 20M, 기본 5M)
//    - 안티앨리어싱: 켜기/끄기
//    - 그림자: 켜기/끄기
//    - FPS 제한: 30 / 60 / 무제한
// 5. 알림 설정:
//    - 브라우저 알림 허용 요청 버튼
//    - 알림 소리: 켜기/끄기

// 저장: 로컬 스토리지 (useLocalStorage 훅)
// 일부 설정(WebSocket)은 서버에도 저장: PUT /api/settings
```

---

## 5. API 인터페이스 상세

### 5.1 REST API (consumed)

모든 요청에는 `Authorization: Bearer <token>` 헤더를 포함한다.
기본 응답 형식: `{ data: T, meta?: { page, limit, total } }`.
에러 응답: `{ error: { code: string, message: string, details?: any } }`.

#### 5.1.1 인증 (Auth)

**POST /api/auth/login**
```typescript
// Request
{ email: string; password: string; }
// Response 200
{ data: { accessToken: string; refreshToken: string; expiresIn: number; } }
// Error 401
{ error: { code: 'INVALID_CREDENTIALS', message: '이메일 또는 비밀번호가 올바르지 않습니다.' } }
```

**POST /api/auth/refresh**
```typescript
// Request
{ refreshToken: string; }
// Response 200
{ data: { accessToken: string; refreshToken: string; expiresIn: number; } }
// Error 401 — refreshToken 만료 시
{ error: { code: 'TOKEN_EXPIRED', message: '토큰이 만료되었습니다. 다시 로그인하세요.' } }
```

**GET /api/auth/me**
```typescript
// Response 200
{ data: { id: string; email: string; name: string; role: 'admin' | 'operator' | 'viewer'; } }
// Error 401
{ error: { code: 'UNAUTHORIZED', message: '인증이 필요합니다.' } }
```

#### 5.1.2 로봇 (Robots)

**GET /api/robots**
```typescript
// Query params: ?page=1&limit=20&status=IDLE,BUSY&search=robot-1
// Response 200
{
  data: Robot[];
  meta: { page: number; limit: number; total: number; }
}
```

**POST /api/robots**
```typescript
// Request
{ name: string; modelId: string; ipAddress: string; port: number; config: RobotConfig; }
// Response 201
{ data: Robot }
// Error 400 — 유효성 검사 실패
// Error 409 — 동일 이름 존재
```

**GET /api/robots/:id**
```typescript
// Response 200
{ data: Robot }
// Error 404
```

**PUT /api/robots/:id**
```typescript
// Request (Partial)
{ name?: string; config?: Partial<RobotConfig>; }
// Response 200
{ data: Robot }
```

**DELETE /api/robots/:id**
```typescript
// Response 204 (No Content)
// Error 400 — 활성 미션이 있는 경우 삭제 불가
// Error 404
```

**POST /api/robots/:id/remote-control**
```typescript
// 원격 제어 잠금 획득
// Response 200
{ data: { lockId: string; expiresAt: string; } }
// Error 409 — 다른 사용자가 이미 제어 중
{ error: { code: 'ALREADY_LOCKED', message: '다른 운영자가 이 로봇을 제어 중입니다.', lockedBy: string } }
```

**DELETE /api/robots/:id/remote-control**
```typescript
// 원격 제어 잠금 해제
// Response 204
```

**POST /api/robots/:id/emergency-stop**
```typescript
// 긴급 정지
// Response 200
{ data: { success: true } }
```

**POST /api/robots/:id/resume**
```typescript
// 긴급 정지 해제
// Response 200
{ data: { success: true } }
```

#### 5.1.3 미션 (Missions)

**GET /api/missions**
```typescript
// Query: ?page=1&limit=20&status=EXECUTING,COMPLETED&robotId=xxx&sort=createdAt&order=desc&since=2026-03-21T00:00:00Z
// Response 200
{ data: Mission[]; meta: { page: number; limit: number; total: number; } }
```

**POST /api/missions**
```typescript
// Request
{
  robotId?: string;
  autoAssign: boolean;
  mapId: string;
  startNodeId: string;
  endNodeId: string;
  waypointIds: string[];
  steps: MissionStep[];
  priority: number;
  scheduledAt?: string;
}
// Response 201
{ data: Mission }
// Error 400 — 유효성 검사 실패
// Error 404 — robotId/mapId/nodeId 존재하지 않음
```

**GET /api/missions/:id**
```typescript
// Response 200
{ data: MissionDetailData }
// Error 404
```

**PUT /api/missions/:id**
```typescript
// Request (상태 변경 또는 우선순위 변경)
{ status?: MissionStatus; priority?: number; }
// Response 200
{ data: Mission }
// Error 400 — 유효하지 않은 상태 전환
```

**DELETE /api/missions/:id**
```typescript
// CREATED 상태에서만 삭제 가능
// Response 204
// Error 400 — 실행 중인 미션은 삭제 불가 (취소 먼저)
```

**POST /api/missions/:id/assign**
```typescript
// 미션을 로봇에 할당
// Request
{ robotId: string; }
// Response 200
{ data: Mission }
// Error 400 — 로봇이 사용 불가능 상태
// Error 404
```

#### 5.1.4 맵 (Maps)

**GET /api/maps**
```typescript
// Response 200
{ data: MapData[] }
```

**POST /api/maps**
```typescript
// Request
{ name: string; description?: string; }
// Response 201
{ data: MapData }
```

**GET /api/maps/:id**
```typescript
// Response 200
{ data: MapData & { pointCloudAssetId?: string; bounds: BoundingBox; } }
```

**PUT /api/maps/:id**
```typescript
// Request
{ name?: string; description?: string; pointCloudAssetId?: string; }
// Response 200
```

**DELETE /api/maps/:id**
```typescript
// Response 204
// Error 400 — 활성 미션에서 사용 중인 맵은 삭제 불가
```

**GET /api/maps/:id/roadmap**
```typescript
// Response 200
{ data: { nodes: RoadmapNode[]; edges: RoadmapEdge[]; } }
```

**PUT /api/maps/:id/roadmap**
```typescript
// Request
{ nodes: RoadmapNode[]; edges: RoadmapEdge[]; }
// Response 200
{ data: { version: number; } }
// Error 409 — 버전 충돌 (다른 사용자가 먼저 수정)
```

**GET /api/maps/:id/semantic**
```typescript
// Response 200
{ data: { regions: SemanticRegion[]; trafficRules: TrafficRule[]; } }
```

**PUT /api/maps/:id/semantic**
```typescript
// Request
{ regions: SemanticRegion[]; trafficRules: TrafficRule[]; }
// Response 200
{ data: { version: number; } }
```

**POST /api/maps/:id/pathfind**
```typescript
// Request
{ startNodeId: string; endNodeId: string; robotType?: string; }
// Response 200
{ data: { path: string[]; // nodeId 배열, distance: number; estimatedTime: number; } }
// Error 400 — 경로 없음
{ error: { code: 'NO_PATH', message: '경로를 찾을 수 없습니다.' } }
```

#### 5.1.5 에셋 (Assets)

**GET /api/assets**
```typescript
// Query: ?type=model,pointcloud
// Response 200
{ data: Asset[] }

interface Asset {
  id: string;
  name: string;
  type: 'model' | 'pointcloud' | 'texture' | 'other';
  mimeType: string;
  size: number;        // bytes
  status: 'uploading' | 'processing' | 'ready' | 'error';
  metadata: Record<string, any>;
  createdAt: string;
}
```

**POST /api/assets/upload**
```typescript
// multipart/form-data
// Fields: file (File), name (string), type ('model' | 'pointcloud')
// Response 201
{ data: Asset }
// Error 413 — 파일 크기 초과
```

**GET /api/assets/:id**
```typescript
// Response 200
{ data: Asset }
// status가 'processing'이면 아직 변환 중
```

**GET /api/assets/:id/file**
```typescript
// 바이너리 파일 다운로드
// Response 200 (application/octet-stream)
// Content-Type에 따라: model/gltf-binary, application/octet-stream 등
```

**DELETE /api/assets/:id**
```typescript
// Response 204
```

#### 5.1.6 포인트 클라우드 타일 (Point Cloud Tiles)

**GET /api/maps/:id/tiles/:nodeId?lod=N**
```typescript
// Potree octree 노드 데이터 요청
// nodeId: 예 "r", "r0", "r01", "r012" (octree 경로)
// lod: Level of Detail (0 = 최저, 높을수록 상세)
// Response 200 (application/octet-stream)
// 바이너리 포인트 데이터 (positions + colors + intensities)
// HTTP Range Requests 지원: 부분 로딩 가능
// Headers:
//   Accept-Ranges: bytes
//   Content-Length: <size>
//   X-Point-Count: <number of points>
//   X-Bounding-Box: <minX,minY,minZ,maxX,maxY,maxZ>
```

**GET /api/maps/:id/tiles/metadata**
```typescript
// Potree octree 메타데이터
// Response 200
{
  data: {
    version: string;         // "2.0"
    octreeDir: string;
    boundingBox: BoundingBox;
    tightBoundingBox: BoundingBox;
    pointAttributes: string[]; // ["POSITION_CARTESIAN", "COLOR_PACKED", "INTENSITY"]
    spacing: number;
    scale: number;
    hierarchyStepSize: number;
    pointCount: number;       // 전체 포인트 수
  }
}
```

#### 5.1.7 사용자 (Users)

**GET /api/users**
```typescript
// Response 200 (admin only)
{ data: User[] }
```

**POST /api/users**
```typescript
// Request
{ name: string; email: string; password: string; role: 'admin' | 'operator' | 'viewer'; }
// Response 201
{ data: User }
// Error 409 — 이메일 중복
```

**GET /api/users/:id**
```typescript
// Response 200
{ data: User }
```

**PUT /api/users/:id**
```typescript
// Request
{ name?: string; email?: string; role?: string; password?: string; }
// Response 200
{ data: User }
```

**DELETE /api/users/:id**
```typescript
// Response 204
// Error 400 — 자기 자신 삭제 불가
```

#### 5.1.8 플러그인 (Plugins)

**GET /api/plugins**
```typescript
// Response 200
{ data: Plugin[] }

interface Plugin {
  id: string;
  name: string;
  description: string;
  version: string;
  enabled: boolean;
  configSchema: JSONSchema;   // 설정 폼 동적 생성용
  config: Record<string, any>;
}
```

**PUT /api/plugins/:id/config**
```typescript
// Request
{ enabled?: boolean; config?: Record<string, any>; }
// Response 200
{ data: Plugin }
// Error 400 — config가 configSchema 검증 실패
```

---

### 5.2 WebSocket Messages (consumed/sent)

WebSocket 연결: `ws://HOST/ws?token=<accessToken>`

모든 메시지 포맷:
```typescript
interface WSMessage {
  type: string;           // 토픽 (dot notation)
  payload: unknown;       // 메시지 데이터
  timestamp: number;      // Unix epoch ms
}
```

#### 5.2.1 텔레메트리 (server → client)

```typescript
// type: 'telemetry.pose'
// 주기: 10Hz (100ms 간격) per robot
// 구독: 연결 시 자동 구독 (전체 로봇) 또는 특정 로봇만
// 구독 요청: { type: 'subscribe', payload: { topic: 'telemetry.pose', robotId?: string } }
// 구독 해제: { type: 'unsubscribe', payload: { topic: 'telemetry.pose', robotId?: string } }

interface TelemetryPoseMessage {
  type: 'telemetry.pose';
  payload: {
    robotId: string;
    pose: Pose3D;
    velocity: Twist;
    battery: number;            // 0-100 (%)
    status: RobotStatus;
    errors: string[];           // 현재 활성 오류 코드
    timestamp: number;
  };
  timestamp: number;
}
```

#### 5.2.2 센서 데이터 (server → client)

```typescript
// type: 'telemetry.lidar'
// 주기: 5Hz (200ms)
// 선택적 구독 (원격 제어 또는 3D 뷰어 센서 표시 시에만)

interface TelemetryLidarMessage {
  type: 'telemetry.lidar';
  payload: {
    robotId: string;
    ranges: Float32Array;       // 거리 값 배열 (binary로 전송)
    angleMin: number;           // 시작 각도 (rad)
    angleMax: number;           // 종료 각도 (rad)
    angleIncrement: number;     // 각도 증가분 (rad)
    rangeMin: number;           // 최소 거리 (m)
    rangeMax: number;           // 최대 거리 (m)
  };
  timestamp: number;
}
```

#### 5.2.3 미션 상태 업데이트 (server → client)

```typescript
// type: 'mission.status'
// 이벤트 기반 (상태 변경 시에만 전송)

interface MissionStatusMessage {
  type: 'mission.status';
  payload: {
    missionId: string;
    robotId: string;
    previousStatus: MissionStatus;
    currentStatus: MissionStatus;
    currentStepIndex: number;
    progress: number;           // 0-100 (%)
    estimatedRemainingTime: number; // seconds
    error?: string;
  };
  timestamp: number;
}
```

#### 5.2.4 알림 (server → client)

```typescript
// type: 'alert'
// 이벤트 기반

interface AlertMessage {
  type: 'alert';
  payload: Alert; // AlertFeed에서 정의한 Alert 타입과 동일
  timestamp: number;
}
```

#### 5.2.5 원격 제어 명령 (client → server)

```typescript
// type: 'remote_control.velocity'
// 주기: 10Hz (클라이언트에서 스로틀링)

interface RemoteControlVelocityMessage {
  type: 'remote_control.velocity';
  payload: {
    robotId: string;
    linear: Vector3;    // linear.x = 전진/후진 속도 (m/s)
    angular: Vector3;   // angular.z = 회전 속도 (rad/s)
  };
  timestamp: number;
}

// type: 'remote_control.emergency_stop'
interface RemoteControlStopMessage {
  type: 'remote_control.emergency_stop';
  payload: { robotId: string; };
  timestamp: number;
}
```

#### 5.2.6 맵 업데이트 알림 (server → client)

```typescript
// type: 'map.updated'
// 다른 사용자가 맵을 수정했을 때 전송

interface MapUpdatedMessage {
  type: 'map.updated';
  payload: {
    mapId: string;
    updatedBy: string;          // 수정한 사용자 ID
    version: number;            // 새 버전 번호
    changes: ('roadmap' | 'semantic' | 'pointcloud')[]; // 변경된 부분
  };
  timestamp: number;
}
```

---

### 5.3 WebRTC

#### 시그널링 플로우

WebSocket을 시그널링 채널로 사용한다.

```
Client                          Server                          Robot
  │                               │                               │
  │ ── ws: webrtc.request ──────> │                               │
  │    { robotId, streamType }    │ ── 로봇에 스트림 요청 ──────> │
  │                               │                               │
  │                               │ <── SDP Offer 전달 ────────── │
  │ <── ws: webrtc.offer ──────── │                               │
  │    { sdp, type: 'offer' }     │                               │
  │                               │                               │
  │ ── ws: webrtc.answer ───────> │                               │
  │    { sdp, type: 'answer' }    │ ── SDP Answer 전달 ─────────> │
  │                               │                               │
  │ <── ws: webrtc.ice ────────── │ ── ICE candidates 교환 ─────> │
  │ ── ws: webrtc.ice ──────────> │ <── ICE candidates 교환 ────  │
  │                               │                               │
  │ <═══════════ P2P 미디어 스트림 (직접 연결 또는 TURN 릴레이) ════════> │
```

#### STUN/TURN 설정

```typescript
// 환경변수로 설정 (개발/프로덕션 분리)
// .env.example
VITE_STUN_URL=stun:stun.l.google.com:19302
VITE_TURN_URL=turn:turn.amr-system.com:3478
VITE_TURN_USERNAME=amr
VITE_TURN_CREDENTIAL=secret

// 사용:
const rtcConfig: RTCConfiguration = {
  iceServers: [
    { urls: import.meta.env.VITE_STUN_URL },
    {
      urls: import.meta.env.VITE_TURN_URL,
      username: import.meta.env.VITE_TURN_USERNAME,
      credential: import.meta.env.VITE_TURN_CREDENTIAL,
    },
  ],
  iceCandidatePoolSize: 10,
};
```

---

## 6. 상태 관리 설계

### 6.1 Zustand Stores

#### useAuthStore

```typescript
// src/stores/authStore.ts
import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { User } from '@/types/user.types';

interface AuthState {
  user: User | null;
  accessToken: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;

  // Actions
  login: (accessToken: string, refreshToken: string, user: User) => void;
  logout: () => void;
  updateToken: (accessToken: string, refreshToken: string) => void;
  updateUser: (user: Partial<User>) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      accessToken: null,
      refreshToken: null,
      isAuthenticated: false,

      login: (accessToken, refreshToken, user) =>
        set({ accessToken, refreshToken, user, isAuthenticated: true }),

      logout: () =>
        set({ accessToken: null, refreshToken: null, user: null, isAuthenticated: false }),

      updateToken: (accessToken, refreshToken) =>
        set({ accessToken, refreshToken }),

      updateUser: (partial) =>
        set((state) => ({
          user: state.user != null ? { ...state.user, ...partial } : null,
        })),
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({
        accessToken: state.accessToken,
        refreshToken: state.refreshToken,
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    },
  ),
);
```

#### useRobotStore

```typescript
// src/stores/robotStore.ts
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type { Robot, Pose3D, RobotStatus, Twist } from '@/types';

interface RobotState {
  id: string;
  name: string;
  modelId: string;
  status: RobotStatus;
  pose: Pose3D;
  velocity: Twist;
  battery: number;
  errors: string[];
  lastUpdate: number;
}

interface RobotStoreState {
  robots: Map<string, RobotState>;
  selectedRobotId: string | null;

  // Actions
  setRobots: (robots: Robot[]) => void;
  updatePose: (robotId: string, pose: Pose3D, velocity: Twist, battery: number, status: RobotStatus, errors: string[]) => void;
  selectRobot: (robotId: string | null) => void;
  getSelectedRobot: () => RobotState | undefined;
  getRobotsByStatus: (status: RobotStatus) => RobotState[];
}

export const useRobotStore = create<RobotStoreState>()(
  subscribeWithSelector((set, get) => ({
    robots: new Map(),
    selectedRobotId: null,

    setRobots: (robots) =>
      set({
        robots: new Map(
          robots.map((r) => [
            r.id,
            {
              id: r.id,
              name: r.name,
              modelId: r.modelId,
              status: r.status,
              pose: r.pose,
              velocity: { linear: { x: 0, y: 0, z: 0 }, angular: { x: 0, y: 0, z: 0 } },
              battery: 100,
              errors: [],
              lastUpdate: Date.now(),
            },
          ]),
        ),
      }),

    updatePose: (robotId, pose, velocity, battery, status, errors) =>
      set((state) => {
        const newRobots = new Map(state.robots);
        const existing = newRobots.get(robotId);
        if (existing != null) {
          newRobots.set(robotId, {
            ...existing,
            pose,
            velocity,
            battery,
            status,
            errors,
            lastUpdate: Date.now(),
          });
        }
        return { robots: newRobots };
      }),

    selectRobot: (robotId) => set({ selectedRobotId: robotId }),

    getSelectedRobot: () => {
      const { robots, selectedRobotId } = get();
      return selectedRobotId != null ? robots.get(selectedRobotId) : undefined;
    },

    getRobotsByStatus: (status) => {
      const { robots } = get();
      return Array.from(robots.values()).filter((r) => r.status === status);
    },
  })),
);
```

#### useMapStore

```typescript
// src/stores/mapStore.ts
import { create } from 'zustand';
import type { MapData, RoadmapNode, RoadmapEdge, SemanticRegion, StaticObstacle } from '@/types';

interface MapStoreState {
  currentMap: MapData | null;
  nodes: Map<string, RoadmapNode>;
  edges: Map<string, RoadmapEdge>;
  regions: Map<string, SemanticRegion>;
  obstacles: Map<string, StaticObstacle>;
  isDirty: boolean;         // 저장되지 않은 변경 존재 여부

  // Actions
  loadMap: (map: MapData) => void;
  setRoadmap: (nodes: RoadmapNode[], edges: RoadmapEdge[]) => void;
  setSemantic: (regions: SemanticRegion[]) => void;

  // Node CRUD
  addNode: (node: RoadmapNode) => void;
  updateNode: (id: string, partial: Partial<RoadmapNode>) => void;
  updateNodePosition: (id: string, position: Vector3) => void;
  removeNode: (id: string) => void;

  // Edge CRUD
  addEdge: (edge: RoadmapEdge) => void;
  updateEdge: (id: string, partial: Partial<RoadmapEdge>) => void;
  removeEdge: (id: string) => void;

  // Region CRUD
  addRegion: (region: SemanticRegion) => void;
  updateRegion: (id: string, partial: Partial<SemanticRegion>) => void;
  removeRegion: (id: string) => void;

  // Obstacle CRUD
  addObstacle: (obstacle: StaticObstacle) => void;
  removeObstacle: (id: string) => void;

  // Utility
  markClean: () => void;
  getConnectedEdges: (nodeId: string) => RoadmapEdge[];
}
```

#### useMissionStore

```typescript
// src/stores/missionStore.ts
import { create } from 'zustand';
import type { Mission, MissionStatus } from '@/types';

interface MissionStoreState {
  missions: Map<string, Mission>;
  activeMissionIds: Set<string>;   // ASSIGNED + EXECUTING 상태

  // Actions
  setMissions: (missions: Mission[]) => void;
  updateMissionStatus: (missionId: string, status: MissionStatus, progress?: number) => void;
  removeMission: (missionId: string) => void;
  getActiveMissions: () => Mission[];
  getMissionsByRobot: (robotId: string) => Mission[];
}
```

#### useViewerStore

```typescript
// src/stores/viewerStore.ts
import { create } from 'zustand';

type CameraMode = 'orbit' | 'fly' | 'topDown';
type ViewMode = 'simulation' | 'real' | 'hybrid';

interface ViewerStoreState {
  cameraMode: CameraMode;
  viewMode: ViewMode;
  selectedObjectId: string | null;
  selectedObjectType: 'robot' | 'node' | 'edge' | 'region' | 'obstacle' | null;
  showGrid: boolean;
  showAxes: boolean;
  showSensors: boolean;
  showTrails: boolean;
  pointBudget: number;

  // Actions
  setCameraMode: (mode: CameraMode) => void;
  setViewMode: (mode: ViewMode) => void;
  selectObject: (id: string | null, type: string | null) => void;
  toggleGrid: () => void;
  toggleAxes: () => void;
  toggleSensors: () => void;
  toggleTrails: () => void;
  setPointBudget: (budget: number) => void;
}
```

#### useEditorStore

```typescript
// src/stores/editorStore.ts
import { create } from 'zustand';
import type { EditorCommand, EditorTool } from '@/types/editor.types';

interface EditorStoreState {
  tool: EditorTool;
  undoStack: EditorCommand[];
  redoStack: EditorCommand[];
  gridSnap: boolean;
  gridSize: number;               // meters
  isDrawing: boolean;             // 영역 그리기 진행 중
  drawingPoints: Vector3[];       // 현재 그리기 중인 폴리곤 꼭짓점들
  pendingEdgeSource: string | null; // 엣지 추가 시 소스 노드 ID

  // Actions
  setTool: (tool: EditorTool) => void;
  executeCommand: (cmd: EditorCommand) => void;
  undo: () => void;
  redo: () => void;
  canUndo: () => boolean;
  canRedo: () => boolean;
  toggleGridSnap: () => void;
  setGridSize: (size: number) => void;
  startDrawing: () => void;
  addDrawingPoint: (point: Vector3) => void;
  finishDrawing: () => Vector3[];
  cancelDrawing: () => void;
  setPendingEdgeSource: (nodeId: string | null) => void;
  clearHistory: () => void;
}

// 최대 undo 스택 크기: 100
// executeCommand: undoStack.push(cmd) + cmd.execute() + redoStack = []
// undo: undoStack.pop() → cmd.undo() → redoStack.push(cmd)
// redo: redoStack.pop() → cmd.execute() → undoStack.push(cmd)
```

#### useAlertStore

```typescript
// src/stores/alertStore.ts
import { create } from 'zustand';
import type { Alert } from '@/types';

interface AlertStoreState {
  alerts: Alert[];
  unreadCount: number;

  // Actions
  addAlert: (alert: Alert) => void;
  markAsRead: (alertId: string) => void;
  markAllAsRead: () => void;
  clearAlerts: () => void;
  getAlertsByType: (type: AlertType) => Alert[];
}

// 최대 alerts 배열 크기: 500 (FIFO, 초과 시 오래된 것 제거)
```

#### useUIStore

```typescript
// src/stores/uiStore.ts
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface UIStoreState {
  sidebarCollapsed: boolean;
  theme: 'light' | 'dark' | 'system';
  language: 'ko' | 'en';

  // Actions
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
  setLanguage: (lang: 'ko' | 'en') => void;
}

export const useUIStore = create<UIStoreState>()(
  persist(
    (set) => ({
      sidebarCollapsed: false,
      theme: 'system',
      language: 'ko',
      toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
      setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
      setTheme: (theme) => set({ theme }),
      setLanguage: (language) => set({ language }),
    }),
    { name: 'ui-storage' },
  ),
);
```

### 6.2 TanStack Query

#### Query Key 규칙

```typescript
// src/constants/queryKeys.ts
export const queryKeys = {
  // 로봇
  robots: {
    all: ['robots'] as const,
    lists: () => [...queryKeys.robots.all, 'list'] as const,
    list: (filters: RobotFilters) => [...queryKeys.robots.lists(), filters] as const,
    details: () => [...queryKeys.robots.all, 'detail'] as const,
    detail: (id: string) => [...queryKeys.robots.details(), id] as const,
  },
  // 미션
  missions: {
    all: ['missions'] as const,
    lists: () => [...queryKeys.missions.all, 'list'] as const,
    list: (filters: MissionFilters) => [...queryKeys.missions.lists(), filters] as const,
    details: () => [...queryKeys.missions.all, 'detail'] as const,
    detail: (id: string) => [...queryKeys.missions.details(), id] as const,
  },
  // 맵
  maps: {
    all: ['maps'] as const,
    lists: () => [...queryKeys.maps.all, 'list'] as const,
    detail: (id: string) => [...queryKeys.maps.all, 'detail', id] as const,
    roadmap: (id: string) => [...queryKeys.maps.all, 'roadmap', id] as const,
    semantic: (id: string) => [...queryKeys.maps.all, 'semantic', id] as const,
    tiles: (id: string, nodeId: string) => [...queryKeys.maps.all, 'tiles', id, nodeId] as const,
  },
  // 대시보드
  dashboard: {
    kpis: ['dashboard', 'kpis'] as const,
  },
  // 사용자
  users: {
    all: ['users'] as const,
    detail: (id: string) => ['users', 'detail', id] as const,
  },
  // 플러그인
  plugins: {
    all: ['plugins'] as const,
  },
  // 에셋
  assets: {
    all: ['assets'] as const,
    detail: (id: string) => ['assets', 'detail', id] as const,
  },
} as const;
```

#### Mutation 패턴 (Optimistic Update 예시)

```typescript
// src/hooks/useMission.ts
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { missionService } from '@/services/mission.service';
import { queryKeys } from '@/constants/queryKeys';
import type { Mission } from '@/types';

export function useUpdateMissionPriority() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ missionId, priority }: { missionId: string; priority: number }) =>
      missionService.updateMission(missionId, { priority }),

    // Optimistic Update
    onMutate: async ({ missionId, priority }) => {
      // 진행 중인 쿼리 취소
      await queryClient.cancelQueries({ queryKey: queryKeys.missions.all });

      // 이전 상태 백업
      const previousMissions = queryClient.getQueryData<Mission[]>(queryKeys.missions.lists());

      // 캐시를 즉시 업데이트
      queryClient.setQueryData<Mission[]>(queryKeys.missions.lists(), (old) =>
        old?.map((m) => (m.id === missionId ? { ...m, priority } : m)),
      );

      return { previousMissions };
    },

    // 에러 시 롤백
    onError: (_err, _vars, context) => {
      if (context?.previousMissions != null) {
        queryClient.setQueryData(queryKeys.missions.lists(), context.previousMissions);
      }
    },

    // 성공/실패 무관하게 쿼리 무효화 (서버와 동기화)
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.missions.all });
    },
  });
}
```

#### 캐시 무효화 전략

```
1. 생성(POST) 후: 목록 쿼리 무효화 (queryKeys.xxx.lists())
2. 수정(PUT) 후: 목록 + 상세 쿼리 무효화
3. 삭제(DELETE) 후: 목록 쿼리 무효화, 상세 캐시 제거
4. WebSocket 이벤트 수신 시: 해당 리소스 쿼리 무효화
   예: 'mission.status' 수신 → queryKeys.missions.detail(missionId) 무효화
5. 맵 에디터 저장 후: roadmap + semantic 쿼리 무효화
6. 포커스 복귀 시 자동 리페치: refetchOnWindowFocus: true (기본값)
7. staleTime 설정:
   - 로봇 목록: 30초 (실시간 데이터는 WebSocket으로 보완)
   - 미션 목록: 10초
   - 맵 데이터: 5분 (자주 변경되지 않음)
   - 사용자/플러그인: 10분
   - KPI: 30초
```

---

## 7. 핵심 TypeScript 타입/인터페이스

### 7.1 공통 타입 (common.types.ts)

```typescript
// src/types/common.types.ts

export interface Vector3 {
  x: number;
  y: number;
  z: number;
}

export interface Quaternion {
  x: number;
  y: number;
  z: number;
  w: number;
}

export interface Pose3D {
  position: Vector3;
  orientation: Quaternion;
}

export interface Twist {
  linear: Vector3;
  angular: Vector3;
}

export interface BoundingBox {
  min: Vector3;
  max: Vector3;
}

export interface PaginatedResponse<T> {
  data: T[];
  meta: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
}

export interface ApiResponse<T> {
  data: T;
}

export interface ApiError {
  error: {
    code: string;
    message: string;
    details?: unknown;
  };
}
```

### 7.2 로봇 타입 (robot.types.ts)

```typescript
// src/types/robot.types.ts
import type { Pose3D, Vector3 } from './common.types';

export enum RobotStatus {
  IDLE = 'IDLE',
  BUSY = 'BUSY',
  ERROR = 'ERROR',
  OFFLINE = 'OFFLINE',
}

export interface Robot {
  id: string;
  name: string;
  modelId: string;
  ipAddress: string;
  port: number;
  status: RobotStatus;
  pose: Pose3D;
  config: RobotConfig;
  createdAt: string;
  updatedAt: string;
}

export interface RobotConfig {
  maxLinearSpeed: number;
  maxAngularSpeed: number;
  acceleration: number;
  deceleration: number;
  footprint: Vector3[];
  sensors: SensorConfig[];
}

export interface SensorConfig {
  type: 'lidar2d' | 'lidar3d' | 'camera' | 'imu' | 'encoder';
  name: string;
  topic: string;
  frameId: string;
  transform: Pose3D;
}
```

### 7.3 맵 타입 (map.types.ts)

```typescript
// src/types/map.types.ts
import type { Vector3, BoundingBox } from './common.types';

export interface MapData {
  id: string;
  name: string;
  description: string;
  version: number;
  bounds: BoundingBox;
  pointCloudAssetId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface RoadmapNode {
  id: string;
  position: Vector3;
  type: NodeType;
  name: string;
  waitBehavior: {
    enabled: boolean;
    duration: number;
    condition?: string;
  };
  properties: Record<string, unknown>;
}

export type NodeType = 'waypoint' | 'charging' | 'dock' | 'elevator' | 'parking';

export interface RoadmapEdge {
  id: string;
  sourceId: string;
  targetId: string;
  maxSpeed: number;
  direction: 'uni' | 'bi';
  allowedRobotTypes: string[];
  weight: number;
  width: number;
}

export interface SemanticRegion {
  id: string;
  type: RegionType;
  name: string;
  polygon: Vector3[];
  properties: RegionProperties;
}

export type RegionType = 'noGo' | 'speedLimit' | 'charging' | 'loadingDock' | 'elevator' | 'custom';

export interface RegionProperties {
  speedLimit?: number;
  maxRobots?: number;
  priority?: number;
  schedule?: {
    startTime: string;
    endTime: string;
    daysOfWeek: number[];
  };
  [key: string]: unknown;
}

export interface TrafficRule {
  id: string;
  regionId: string;
  maxRobots: number;
  priority: 'fifo' | 'priority' | 'nearest';
  oneWay: boolean;
  speedLimit: number;
  allowedRobotTypes: string[];
}

export interface StaticObstacle {
  id: string;
  type: 'box' | 'cylinder' | 'mesh';
  position: Vector3;
  rotation: Vector3;
  dimensions:
    | { width: number; height: number; depth: number }
    | { radius: number; height: number };
}
```

### 7.4 미션 타입 (mission.types.ts)

```typescript
// src/types/mission.types.ts

export enum MissionStatus {
  CREATED = 'CREATED',
  ASSIGNED = 'ASSIGNED',
  EXECUTING = 'EXECUTING',
  COMPLETED = 'COMPLETED',
  FAILED = 'FAILED',
  CANCELLED = 'CANCELLED',
}

export interface Mission {
  id: string;
  robotId: string | null;
  mapId: string;
  status: MissionStatus;
  priority: number;
  startNodeId: string;
  endNodeId: string;
  waypointIds: string[];
  steps: MissionStep[];
  progress: number;
  createdAt: string;
  startedAt: string | null;
  completedAt: string | null;
  estimatedEndAt: string | null;
  errorMessage: string | null;
}

export interface MissionStep {
  id: string;
  nodeId: string;
  action: 'wait' | 'pickup' | 'dropoff' | 'charge' | 'custom';
  params: Record<string, unknown>;
  timeout?: number;
  status: 'pending' | 'executing' | 'completed' | 'failed' | 'skipped';
  startedAt: string | null;
  completedAt: string | null;
  error: string | null;
}
```

### 7.5 텔레메트리 타입 (telemetry.types.ts)

```typescript
// src/types/telemetry.types.ts
import type { Pose3D, Twist } from './common.types';
import type { RobotStatus } from './robot.types';

export interface TelemetryMessage {
  robotId: string;
  timestamp: number;
  pose: Pose3D;
  velocity: Twist;
  battery: number;
  status: RobotStatus;
  errors: string[];
}

export interface LidarScanMessage {
  robotId: string;
  timestamp: number;
  ranges: number[];
  angleMin: number;
  angleMax: number;
  angleIncrement: number;
  rangeMin: number;
  rangeMax: number;
}
```

### 7.6 사용자 타입 (user.types.ts)

```typescript
// src/types/user.types.ts

export type UserRole = 'admin' | 'operator' | 'viewer';

export interface User {
  id: string;
  name: string;
  email: string;
  role: UserRole;
  createdAt: string;
  updatedAt: string;
}
```

### 7.7 뷰어 타입 (viewer.types.ts)

```typescript
// src/types/viewer.types.ts

export type CameraMode = 'orbit' | 'fly' | 'topDown';
export type ViewMode = 'simulation' | 'real' | 'hybrid';

export interface ViewerSettings {
  showGrid: boolean;
  showAxes: boolean;
  showSensors: boolean;
  showTrails: boolean;
  pointBudget: number;
  antialias: boolean;
  shadows: boolean;
  fpsLimit: 30 | 60 | 0; // 0 = unlimited
}
```

### 7.8 에디터 타입 (editor.types.ts)

```typescript
// src/types/editor.types.ts
import type { Vector3 } from './common.types';

export type EditorTool = 'select' | 'addNode' | 'addEdge' | 'drawRegion' | 'addObstacle' | 'measure' | 'delete';

export interface EditorCommand {
  type: string;
  execute: () => void;
  undo: () => void;
  description: string;
}
```

### 7.9 WebSocket 타입 (websocket.types.ts)

```typescript
// src/types/websocket.types.ts

export interface WSMessage<T = unknown> {
  type: string;
  payload: T;
  timestamp: number;
}

export interface WSSubscription {
  topic: string;
  robotId?: string;
}

export type WSConnectionState = 'connecting' | 'connected' | 'disconnecting' | 'disconnected';
```

### 7.10 API 타입 (api.types.ts)

```typescript
// src/types/api.types.ts

export interface RobotFilters {
  page?: number;
  limit?: number;
  status?: string;
  search?: string;
}

export interface MissionFilters {
  page?: number;
  limit?: number;
  status?: string;
  robotId?: string;
  sort?: string;
  order?: 'asc' | 'desc';
  since?: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  accessToken: string;
  refreshToken: string;
  expiresIn: number;
}

export interface RefreshRequest {
  refreshToken: string;
}
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

### 9.1 단위 테스트 (Vitest + React Testing Library)

**목표**: hooks, stores, utils, services 80%+ 커버리지

#### Store 테스트 예시

```typescript
// tests/unit/stores/robotStore.test.ts
import { describe, it, expect, beforeEach } from 'vitest';
import { useRobotStore } from '@/stores/robotStore';
import { RobotStatus } from '@/types';

describe('robotStore', () => {
  beforeEach(() => {
    useRobotStore.setState({ robots: new Map(), selectedRobotId: null });
  });

  it('setRobots는 로봇 맵을 초기화한다', () => {
    const robots = [
      { id: '1', name: 'Robot-1', modelId: 'm1', status: RobotStatus.IDLE, pose: defaultPose, config: defaultConfig },
    ];
    useRobotStore.getState().setRobots(robots);
    expect(useRobotStore.getState().robots.size).toBe(1);
    expect(useRobotStore.getState().robots.get('1')?.name).toBe('Robot-1');
  });

  it('updatePose는 기존 로봇의 포즈를 업데이트한다', () => {
    // setup → updatePose → verify
  });

  it('selectRobot은 selectedRobotId를 설정한다', () => {
    useRobotStore.getState().selectRobot('1');
    expect(useRobotStore.getState().selectedRobotId).toBe('1');
  });

  it('getRobotsByStatus는 해당 상태의 로봇만 반환한다', () => {
    // setup with mixed statuses → verify filter
  });
});
```

#### Service 테스트 예시

```typescript
// tests/unit/services/api.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { apiClient } from '@/services/api';

describe('apiClient', () => {
  it('요청에 Authorization 헤더를 자동 첨부한다', async () => {
    // Mock authStore with token → make request → verify header
  });

  it('401 응답 시 토큰 리프레시를 시도한다', async () => {
    // Mock 401 response → verify refresh call → retry original request
  });

  it('리프레시 실패 시 로그아웃한다', async () => {
    // Mock refresh 401 → verify logout call
  });

  it('네트워크 에러 시 3회 재시도한다', async () => {
    // Mock network error → verify 3 retry attempts
  });
});
```

#### Hook 테스트 예시

```typescript
// tests/unit/hooks/useWebSocket.test.ts
import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useWebSocket } from '@/hooks/useWebSocket';

describe('useWebSocket', () => {
  it('연결 시 connected 상태로 전환한다', () => { /* ... */ });
  it('메시지 수신 시 콜백을 호출한다', () => { /* ... */ });
  it('연결 끊김 시 자동 재접속을 시도한다', () => { /* ... */ });
  it('구독 해제 시 클린업한다', () => { /* ... */ });
});
```

#### 유틸리티 함수 테스트

```typescript
// tests/unit/utils/math.test.ts
import { describe, it, expect } from 'vitest';
import { quaternionToEuler, distance3D, lerp } from '@/utils/math';

describe('math utils', () => {
  it('quaternionToEuler은 단위 쿼터니언을 (0,0,0)으로 변환한다', () => {
    const euler = quaternionToEuler({ x: 0, y: 0, z: 0, w: 1 });
    expect(euler.x).toBeCloseTo(0);
    expect(euler.y).toBeCloseTo(0);
    expect(euler.z).toBeCloseTo(0);
  });

  it('distance3D는 두 점 사이 유클리드 거리를 반환한다', () => {
    const d = distance3D({ x: 0, y: 0, z: 0 }, { x: 3, y: 4, z: 0 });
    expect(d).toBeCloseTo(5);
  });
});
```

### 9.2 컴포넌트 테스트 (Storybook + Chromatic)

```typescript
// src/components/common/Button.stories.tsx
import type { Meta, StoryObj } from '@storybook/react';
import { Button } from './Button';

const meta: Meta<typeof Button> = {
  title: 'Common/Button',
  component: Button,
  tags: ['autodocs'],
  argTypes: {
    variant: { control: 'select', options: ['primary', 'secondary', 'danger', 'ghost'] },
    size: { control: 'select', options: ['sm', 'md', 'lg'] },
    disabled: { control: 'boolean' },
    loading: { control: 'boolean' },
  },
};
export default meta;

type Story = StoryObj<typeof Button>;

export const Primary: Story = { args: { variant: 'primary', children: 'Primary Button' } };
export const Secondary: Story = { args: { variant: 'secondary', children: 'Secondary' } };
export const Danger: Story = { args: { variant: 'danger', children: 'Delete' } };
export const Loading: Story = { args: { variant: 'primary', children: 'Loading...', loading: true } };
export const Disabled: Story = { args: { variant: 'primary', children: 'Disabled', disabled: true } };
export const Small: Story = { args: { variant: 'primary', children: 'Small', size: 'sm' } };
export const Large: Story = { args: { variant: 'primary', children: 'Large', size: 'lg' } };
```

모든 공통 컴포넌트에 대해 스토리를 작성하며 각 상태(loading, error, empty, filled, disabled)에 대한 스토리를 포함한다. axe-core 접근성 검사를 Storybook addon으로 통합한다.

### 9.3 통합 테스트 (Vitest)

```typescript
// tests/integration/setup.ts
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import { render } from '@testing-library/react';

export function renderWithProviders(
  ui: React.ReactElement,
  { route = '/', ...options } = {},
) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[route]}>
        {ui}
      </MemoryRouter>
    </QueryClientProvider>,
    options,
  );
}
```

```typescript
// tests/integration/dashboard.test.tsx
import { describe, it, expect } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { renderWithProviders } from './setup';
import DashboardPage from '@/components/dashboard/DashboardPage';
// MSW handlers 자동 적용

describe('DashboardPage 통합 테스트', () => {
  it('KPI 카드가 서버 데이터로 렌더링된다', async () => {
    renderWithProviders(<DashboardPage />);
    await waitFor(() => {
      expect(screen.getByText(/완료 미션/)).toBeInTheDocument();
    });
  });

  it('로봇 목록이 로드되어 테이블에 표시된다', async () => {
    renderWithProviders(<DashboardPage />);
    await waitFor(() => {
      expect(screen.getByText('Robot-1')).toBeInTheDocument();
    });
  });
});
```

```typescript
// tests/integration/map-editor.test.tsx
describe('MapEditor Undo/Redo 통합 테스트', () => {
  it('노드 추가 후 Undo하면 노드가 제거된다', () => {
    // editorStore.executeCommand(AddNodeCommand) → verify node exists
    // editorStore.undo() → verify node removed
    // editorStore.redo() → verify node restored
  });

  it('연속 작업 후 여러 번 Undo 가능하다', () => {
    // 5개 작업 실행 → 3번 undo → 2번 redo → verify state
  });
});
```

### 9.4 E2E 테스트 (Playwright)

```typescript
// playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
  ],
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
  },
});
```

```typescript
// tests/e2e/auth.spec.ts
import { test, expect } from '@playwright/test';

test.describe('인증 플로우', () => {
  test('유효한 자격 증명으로 로그인 성공', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'admin@amr.com');
    await page.fill('[name="password"]', 'password123');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/dashboard');
    await expect(page.getByText('대시보드')).toBeVisible();
  });

  test('잘못된 자격 증명으로 로그인 실패', async ({ page }) => {
    await page.goto('/login');
    await page.fill('[name="email"]', 'wrong@amr.com');
    await page.fill('[name="password"]', 'wrongpassword');
    await page.click('button[type="submit"]');
    await expect(page.getByText('이메일 또는 비밀번호가 올바르지 않습니다')).toBeVisible();
  });

  test('인증 없이 대시보드 접근 시 로그인 페이지로 리다이렉트', async ({ page }) => {
    await page.goto('/dashboard');
    await expect(page).toHaveURL('/login');
  });
});
```

```typescript
// tests/e2e/mission.spec.ts
test.describe('미션 관리', () => {
  test.beforeEach(async ({ page }) => {
    // 로그인 수행
    await page.goto('/login');
    await page.fill('[name="email"]', 'admin@amr.com');
    await page.fill('[name="password"]', 'password123');
    await page.click('button[type="submit"]');
    await expect(page).toHaveURL('/dashboard');
  });

  test('미션 생성 위자드 전체 플로우', async ({ page }) => {
    await page.goto('/missions');
    await page.click('button:has-text("새 미션 생성")');
    // Step 1: 로봇 선택
    await page.selectOption('[name="robotId"]', { index: 1 });
    await page.click('button:has-text("다음")');
    // Step 2: 경로 설정 (미니 뷰어에서 클릭은 mocking)
    await page.click('button:has-text("다음")');
    // Step 3: 작업 단계 (건너뛰기)
    await page.click('button:has-text("다음")');
    // Step 4: 확인 및 제출
    await page.click('button:has-text("미션 생성")');
    await expect(page.getByText('미션이 생성되었습니다')).toBeVisible();
  });
});
```

```typescript
// tests/e2e/map-editor.spec.ts
test.describe('맵 에디터', () => {
  test('로드맵 그래프 편집 (노드 추가 → 엣지 추가 → 저장)', async ({ page }) => {
    await page.goto('/map-editor');
    // 노드 추가 도구 선택
    await page.click('[data-tool="addNode"]');
    // 3D 캔버스 클릭 (노드 추가) — 캔버스 좌표 클릭
    await page.click('canvas', { position: { x: 400, y: 300 } });
    await page.click('canvas', { position: { x: 600, y: 300 } });
    // 엣지 추가 도구 선택
    await page.click('[data-tool="addEdge"]');
    // 첫 번째 노드 클릭 → 두 번째 노드 클릭
    // ... (3D 인터랙션은 제한적이므로 가능한 수준까지)
    // 저장
    await page.click('button:has-text("저장")');
    await expect(page.getByText('저장되었습니다')).toBeVisible();
  });
});
```

### 9.5 성능 테스트

**Lighthouse CI 설정:**
```yaml
# lighthouserc.yml
ci:
  collect:
    url:
      - http://localhost:5173/dashboard
      - http://localhost:5173/viewer
    numberOfRuns: 3
  assert:
    assertions:
      categories:performance:
        - error
        - minScore: 0.8
      categories:accessibility:
        - error
        - minScore: 0.9
      categories:best-practices:
        - error
        - minScore: 0.9
```

**3D 뷰어 성능 벤치마크:**
- 10대 로봇 + 10M 포인트 클라우드: 30fps 이상 유지
- 50대 로봇 + 5M 포인트 클라우드: 30fps 이상 유지
- WebSocket 50 로봇 x 10Hz = 500 msg/s: 프레임 드롭 없음
- 측정 방법: `stats.js` 또는 R3F `<PerformanceMonitor>`로 FPS 모니터링
- CI에서 headless Chrome으로 자동 벤치마크 (Playwright + performance.now())

**메모리 누수 테스트:**
- Chrome DevTools 메모리 프로파일러 기반
- 1시간 연속 운영 후 힙 스냅샷 비교
- 주요 검증: Three.js geometry/material dispose, WebSocket 이벤트 리스너 해제, useEffect cleanup

---

## 10. 목 서버 / 스텁 전략

### 10.1 MSW (Mock Service Worker) REST 모킹

```typescript
// src/__mocks__/browser.ts
import { setupWorker } from 'msw/browser';
import { handlers } from './handlers';

export const worker = setupWorker(...handlers);

// main.tsx에서 개발 모드일 때만 활성화:
// if (import.meta.env.DEV) {
//   const { worker } = await import('./__mocks__/browser');
//   await worker.start({ onUnhandledRequest: 'warn' });
// }
```

```typescript
// src/__mocks__/handlers/robot.handlers.ts
import { http, HttpResponse } from 'msw';
import { mockRobots } from '../data/robots.mock';

export const robotHandlers = [
  http.get('/api/robots', () => {
    return HttpResponse.json({
      data: mockRobots,
      meta: { page: 1, limit: 20, total: mockRobots.length, totalPages: 1 },
    });
  }),

  http.get('/api/robots/:id', ({ params }) => {
    const robot = mockRobots.find((r) => r.id === params.id);
    if (robot == null) {
      return HttpResponse.json(
        { error: { code: 'NOT_FOUND', message: '로봇을 찾을 수 없습니다.' } },
        { status: 404 },
      );
    }
    return HttpResponse.json({ data: robot });
  }),

  http.post('/api/robots', async ({ request }) => {
    const body = await request.json();
    const newRobot = { id: crypto.randomUUID(), ...body, status: 'IDLE', createdAt: new Date().toISOString() };
    mockRobots.push(newRobot);
    return HttpResponse.json({ data: newRobot }, { status: 201 });
  }),
];
```

```typescript
// src/__mocks__/handlers/index.ts
import { authHandlers } from './auth.handlers';
import { robotHandlers } from './robot.handlers';
import { missionHandlers } from './mission.handlers';
import { mapHandlers } from './map.handlers';

export const handlers = [
  ...authHandlers,
  ...robotHandlers,
  ...missionHandlers,
  ...mapHandlers,
];
```

### 10.2 Mock WebSocket 서버

```typescript
// src/__mocks__/ws-mock-server.ts
// Node.js로 실행 (개발 시 별도 프로세스)
// npx tsx src/__mocks__/ws-mock-server.ts

import { WebSocketServer } from 'ws';
import { faker } from '@faker-js/faker';

const wss = new WebSocketServer({ port: 8081 });

const robots = [
  { id: 'robot-1', name: 'AMR-001', x: 0, y: 0, z: 0, yaw: 0 },
  { id: 'robot-2', name: 'AMR-002', x: 5, y: 0, z: 3, yaw: 1.57 },
  { id: 'robot-3', name: 'AMR-003', x: -3, y: 0, z: -2, yaw: 3.14 },
];

wss.on('connection', (ws) => {
  console.log('Client connected');

  // 10Hz 텔레메트리 전송
  const interval = setInterval(() => {
    for (const robot of robots) {
      // 로봇 위치를 약간씩 이동 (시뮬레이션)
      robot.x += (Math.random() - 0.5) * 0.1;
      robot.z += (Math.random() - 0.5) * 0.1;
      robot.yaw += (Math.random() - 0.5) * 0.05;

      const message = {
        type: 'telemetry.pose',
        payload: {
          robotId: robot.id,
          pose: {
            position: { x: robot.x, y: robot.y, z: robot.z },
            orientation: yawToQuaternion(robot.yaw),
          },
          velocity: {
            linear: { x: Math.random() * 0.5, y: 0, z: 0 },
            angular: { x: 0, y: 0, z: (Math.random() - 0.5) * 0.3 },
          },
          battery: 85 + Math.random() * 15,
          status: 'BUSY',
          errors: [],
        },
        timestamp: Date.now(),
      };
      ws.send(JSON.stringify(message));
    }
  }, 100); // 10Hz

  ws.on('close', () => {
    clearInterval(interval);
    console.log('Client disconnected');
  });
});

function yawToQuaternion(yaw: number) {
  return {
    x: 0,
    y: Math.sin(yaw / 2),
    z: 0,
    w: Math.cos(yaw / 2),
  };
}

console.log('Mock WebSocket server running on ws://localhost:8081');
```

### 10.3 Mock 데이터 팩토리

```typescript
// src/__mocks__/factories/robot.factory.ts
import { faker } from '@faker-js/faker';
import type { Robot } from '@/types';
import { RobotStatus } from '@/types';

export function createMockRobot(overrides: Partial<Robot> = {}): Robot {
  return {
    id: faker.string.uuid(),
    name: `AMR-${faker.number.int({ min: 1, max: 999 }).toString().padStart(3, '0')}`,
    modelId: faker.helpers.arrayElement(['model-a', 'model-b', 'model-c']),
    ipAddress: faker.internet.ipv4(),
    port: 9090,
    status: faker.helpers.arrayElement(Object.values(RobotStatus)),
    pose: {
      position: {
        x: faker.number.float({ min: -50, max: 50, fractionDigits: 2 }),
        y: 0,
        z: faker.number.float({ min: -50, max: 50, fractionDigits: 2 }),
      },
      orientation: { x: 0, y: 0, z: 0, w: 1 },
    },
    config: {
      maxLinearSpeed: 1.5,
      maxAngularSpeed: 1.0,
      acceleration: 0.5,
      deceleration: 1.0,
      footprint: [
        { x: 0.3, y: 0, z: 0.2 },
        { x: -0.3, y: 0, z: 0.2 },
        { x: -0.3, y: 0, z: -0.2 },
        { x: 0.3, y: 0, z: -0.2 },
      ],
      sensors: [],
    },
    createdAt: faker.date.past().toISOString(),
    updatedAt: faker.date.recent().toISOString(),
    ...overrides,
  };
}

export function createMockRobots(count: number): Robot[] {
  return Array.from({ length: count }, () => createMockRobot());
}
```

```typescript
// src/__mocks__/factories/mission.factory.ts
import { faker } from '@faker-js/faker';
import type { Mission } from '@/types';
import { MissionStatus } from '@/types';

export function createMockMission(overrides: Partial<Mission> = {}): Mission {
  const status = faker.helpers.arrayElement(Object.values(MissionStatus));
  return {
    id: faker.string.uuid(),
    robotId: faker.string.uuid(),
    mapId: faker.string.uuid(),
    status,
    priority: faker.number.int({ min: 1, max: 10 }),
    startNodeId: faker.string.uuid(),
    endNodeId: faker.string.uuid(),
    waypointIds: [],
    steps: [],
    progress: status === MissionStatus.COMPLETED ? 100 : faker.number.int({ min: 0, max: 99 }),
    createdAt: faker.date.past().toISOString(),
    startedAt: status !== MissionStatus.CREATED ? faker.date.past().toISOString() : null,
    completedAt: status === MissionStatus.COMPLETED ? faker.date.recent().toISOString() : null,
    estimatedEndAt: faker.date.future().toISOString(),
    errorMessage: status === MissionStatus.FAILED ? faker.lorem.sentence() : null,
    ...overrides,
  };
}
```

### 10.4 Mock Potree 데이터

개발용 소규모 포인트 클라우드 (1000 포인트):
- `public/mock-pointcloud/metadata.json`: Potree 메타데이터
- `public/mock-pointcloud/r.bin`: 루트 노드 바이너리 데이터
- 생성 스크립트: `scripts/generate-mock-pointcloud.ts`

### 10.5 Mock glTF 모델

- `public/models/robot-placeholder.glb`: 심플 박스 형태 로봇
- 크기: 0.6m x 0.4m x 0.3m (대략적인 AMR 크기)
- 색상: 파란색 몸체 + 녹색 전면 (방향 구분)
- Blender로 생성하거나, Three.js로 프로그래밍 방식 생성

---

## 11. 다른 팀과의 인터페이스 계약

### 11.1 Backend Team 인터페이스

**REST API:**
- 스키마 형식: OpenAPI 3.1 (YAML 또는 JSON)
- 스키마 위치: `api/openapi.yaml` (Git 관리)
- 코드 생성: openapi-typescript로 TypeScript 타입 자동 생성 가능 (선택적)
- API 버저닝: URL 경로 기반 `/api/v1/...` (초기에는 `/api/...`)
- 에러 응답: 통일된 `{ error: { code, message, details? } }` 형식

**WebSocket:**
- 메시지 스키마: JSON Schema로 정의
- 스키마 위치: `api/ws-messages.schema.json`
- 연결 URL: `ws://HOST/ws?token=<accessToken>`
- 하트비트: 서버가 30초 간격으로 `{ type: 'ping' }` 전송, 클라이언트는 `{ type: 'pong' }` 응답
- 미응답 시 서버에서 연결 종료 (60초 타임아웃)

**합의 사항:**
- API 스키마 변경 시 PR에 Frontend 팀 리뷰 필수
- Breaking change 시 최소 1주 전 공지
- 새 API 엔드포인트는 Mock 핸들러를 함께 제공

### 11.2 Proto Team 인터페이스

- Proto 파일 위치: `proto/` 디렉토리 (모노레포 루트)
- TypeScript 생성: `protoc-gen-ts` 플러그인 사용
- 생성 명령: `npm run generate-types` (buf generate)
- 생성 출력: `src/types/generated/`
- `.gitignore`에 `src/types/generated/` 추가 (CI에서 재생성)
- Proto 메시지와 REST API JSON 간 변환은 Backend가 처리
- Frontend는 생성된 TypeScript 타입만 사용

**buf.gen.yaml 예시:**
```yaml
version: v2
plugins:
  - remote: buf.build/community/timostamm-protobuf-ts
    out: src/types/generated
    opt:
      - long_type_string
      - output_javascript
```

### 11.3 Asset Manager Team 인터페이스

**glTF 모델 포맷 규약:**
- 포맷: glTF 2.0 Binary (.glb)
- 좌표계: Y-up (Three.js 기본)
- 단위: 미터 (1 unit = 1 meter)
- 원점: 로봇 바닥 중심
- 최대 폴리곤: 50,000 (LOD용 저폴리 모델 별도 제공 권장)
- 텍스처: 최대 2048x2048 (WebGL 호환)
- 애니메이션: 바퀴 회전 등 (옵션)

**에셋 메타데이터 JSON 스키마:**
```json
{
  "type": "object",
  "properties": {
    "modelType": { "type": "string", "enum": ["amr", "agv", "arm", "sensor"] },
    "dimensions": {
      "type": "object",
      "properties": {
        "width": { "type": "number" },
        "height": { "type": "number" },
        "depth": { "type": "number" }
      }
    },
    "lodLevels": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "distance": { "type": "number" },
          "assetId": { "type": "string" }
        }
      }
    }
  }
}
```

### 11.4 Map Manager Team 인터페이스

**Potree 타일 포맷:**
- Potree 2.0 octree 구조
- 노드 파일: 바이너리 (positions float32 x3, colors uint8 x3, intensity uint16)
- 메타데이터: JSON (boundingBox, pointCount, spacing, hierarchy)
- 타일 서빙: HTTP GET (CDN 캐시 가능)
- 좌표계: Map Manager가 정의 (Frontend는 변환 없이 사용)

**포인트 클라우드 메타데이터 스키마:**
```json
{
  "version": "2.0",
  "boundingBox": { "min": [x, y, z], "max": [x, y, z] },
  "tightBoundingBox": { "min": [x, y, z], "max": [x, y, z] },
  "pointAttributes": ["POSITION_CARTESIAN", "COLOR_PACKED", "INTENSITY"],
  "spacing": 0.01,
  "scale": 0.001,
  "hierarchyStepSize": 5,
  "pointCount": 100000000
}
```

### 11.5 Sim Engine Team 인터페이스

- 시뮬레이션 텔레메트리는 Backend가 중계 (WebSocket 동일 경로)
- 메시지 포맷: Proto 정의와 동일 (TelemetryPoseMessage)
- 시뮬레이션 모드 표시: `viewMode: 'simulation'`일 때 시뮬레이션 데이터 표시
- 하이브리드 모드: 실제 로봇과 시뮬레이션 로봇을 동시에 렌더링
  - 실제 로봇: 불투명 모델
  - 시뮬레이션 로봇: 반투명 모델 (opacity 0.5) + 점선 궤적
- 구분 기준: Backend가 메시지에 `source: 'sim' | 'real'` 필드 추가

---

## 부록 A: 핵심 서비스 구현 상세

### API 클라이언트 (api.ts)

```typescript
// src/services/api.ts
import { useAuthStore } from '@/stores/authStore';

const BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '/api';
const MAX_RETRIES = 3;
const RETRY_DELAY = 1000; // ms

interface RequestConfig extends RequestInit {
  params?: Record<string, string | number | boolean | undefined>;
  retry?: boolean;
}

class ApiClient {
  private isRefreshing = false;
  private refreshPromise: Promise<boolean> | null = null;

  async request<T>(path: string, config: RequestConfig = {}): Promise<T> {
    const { params, retry = true, ...fetchConfig } = config;

    // URL 구성
    const url = new URL(`${BASE_URL}${path}`, window.location.origin);
    if (params != null) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined) {
          url.searchParams.set(key, String(value));
        }
      });
    }

    // 헤더 설정
    const headers = new Headers(fetchConfig.headers);
    const token = useAuthStore.getState().accessToken;
    if (token != null) {
      headers.set('Authorization', `Bearer ${token}`);
    }
    if (!headers.has('Content-Type') && fetchConfig.body != null) {
      headers.set('Content-Type', 'application/json');
    }

    let lastError: Error | null = null;
    for (let attempt = 0; attempt <= (retry ? MAX_RETRIES : 0); attempt++) {
      try {
        const response = await fetch(url.toString(), { ...fetchConfig, headers });

        // 401: 토큰 리프레시 시도
        if (response.status === 401) {
          const refreshed = await this.refreshToken();
          if (refreshed) {
            // 토큰 갱신 후 재시도
            const newToken = useAuthStore.getState().accessToken;
            headers.set('Authorization', `Bearer ${newToken}`);
            const retryResponse = await fetch(url.toString(), { ...fetchConfig, headers });
            if (!retryResponse.ok) {
              throw await this.createError(retryResponse);
            }
            return (await retryResponse.json()) as T;
          } else {
            // 리프레시 실패 → 로그아웃
            useAuthStore.getState().logout();
            throw new Error('인증이 만료되었습니다.');
          }
        }

        if (!response.ok) {
          throw await this.createError(response);
        }

        if (response.status === 204) {
          return undefined as T;
        }

        return (await response.json()) as T;
      } catch (error) {
        lastError = error as Error;
        if (attempt < MAX_RETRIES && this.isRetryable(error)) {
          await this.delay(RETRY_DELAY * Math.pow(2, attempt));
          continue;
        }
        throw error;
      }
    }
    throw lastError;
  }

  private async refreshToken(): Promise<boolean> {
    if (this.isRefreshing) {
      return this.refreshPromise!;
    }
    this.isRefreshing = true;
    this.refreshPromise = (async () => {
      try {
        const refreshToken = useAuthStore.getState().refreshToken;
        if (refreshToken == null) return false;
        const response = await fetch(`${BASE_URL}/auth/refresh`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ refreshToken }),
        });
        if (!response.ok) return false;
        const data = await response.json();
        useAuthStore.getState().updateToken(data.data.accessToken, data.data.refreshToken);
        return true;
      } catch {
        return false;
      } finally {
        this.isRefreshing = false;
        this.refreshPromise = null;
      }
    })();
    return this.refreshPromise;
  }

  private isRetryable(error: unknown): boolean {
    if (error instanceof TypeError) return true; // 네트워크 에러
    return false;
  }

  private async createError(response: Response): Promise<Error> {
    try {
      const body = await response.json();
      return new Error(body.error?.message ?? `HTTP ${response.status}`);
    } catch {
      return new Error(`HTTP ${response.status}`);
    }
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  // 편의 메서드
  get<T>(path: string, params?: Record<string, any>) {
    return this.request<T>(path, { method: 'GET', params });
  }

  post<T>(path: string, body?: unknown) {
    return this.request<T>(path, { method: 'POST', body: body != null ? JSON.stringify(body) : undefined });
  }

  put<T>(path: string, body?: unknown) {
    return this.request<T>(path, { method: 'PUT', body: body != null ? JSON.stringify(body) : undefined });
  }

  delete<T>(path: string) {
    return this.request<T>(path, { method: 'DELETE' });
  }
}

export const apiClient = new ApiClient();
```

### WebSocket 연결 관리자 (websocket.ts)

```typescript
// src/services/websocket.ts
import type { WSMessage, WSConnectionState } from '@/types/websocket.types';

type MessageHandler = (message: WSMessage) => void;

interface WebSocketManagerOptions {
  url: string;
  token: string;
  maxRetries?: number;
  retryDelay?: number;
  backoffMultiplier?: number;
}

export class WebSocketManager {
  private ws: WebSocket | null = null;
  private handlers = new Map<string, Set<MessageHandler>>();
  private state: WSConnectionState = 'disconnected';
  private retryCount = 0;
  private retryTimer: ReturnType<typeof setTimeout> | null = null;
  private options: Required<WebSocketManagerOptions>;
  private stateListeners = new Set<(state: WSConnectionState) => void>();

  constructor(options: WebSocketManagerOptions) {
    this.options = {
      maxRetries: 10,
      retryDelay: 3000,
      backoffMultiplier: 1.5,
      ...options,
    };
  }

  connect(): void {
    if (this.state === 'connecting' || this.state === 'connected') return;
    this.setState('connecting');

    const url = `${this.options.url}?token=${this.options.token}`;
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      this.setState('connected');
      this.retryCount = 0;
      console.log('[WS] Connected');
    };

    this.ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data as string) as WSMessage;
        // ping/pong 처리
        if (message.type === 'ping') {
          this.send({ type: 'pong', payload: {}, timestamp: Date.now() });
          return;
        }
        // 해당 토픽 핸들러 호출
        const handlers = this.handlers.get(message.type);
        if (handlers != null) {
          handlers.forEach((handler) => handler(message));
        }
        // 와일드카드 핸들러 (모든 메시지 수신)
        const wildcardHandlers = this.handlers.get('*');
        if (wildcardHandlers != null) {
          wildcardHandlers.forEach((handler) => handler(message));
        }
      } catch (error) {
        console.error('[WS] Failed to parse message:', error);
      }
    };

    this.ws.onclose = (event) => {
      this.setState('disconnected');
      console.log(`[WS] Disconnected: code=${event.code}, reason=${event.reason}`);
      if (event.code !== 1000) {
        // 비정상 종료 → 재접속 시도
        this.scheduleReconnect();
      }
    };

    this.ws.onerror = (error) => {
      console.error('[WS] Error:', error);
    };
  }

  disconnect(): void {
    this.setState('disconnecting');
    if (this.retryTimer != null) {
      clearTimeout(this.retryTimer);
      this.retryTimer = null;
    }
    this.ws?.close(1000, 'Client disconnect');
    this.ws = null;
    this.setState('disconnected');
  }

  subscribe(topic: string, handler: MessageHandler): () => void {
    if (!this.handlers.has(topic)) {
      this.handlers.set(topic, new Set());
    }
    this.handlers.get(topic)!.add(handler);

    // 서버에 구독 요청
    this.send({ type: 'subscribe', payload: { topic }, timestamp: Date.now() });

    // 구독 해제 함수 반환
    return () => {
      this.handlers.get(topic)?.delete(handler);
      if (this.handlers.get(topic)?.size === 0) {
        this.handlers.delete(topic);
        this.send({ type: 'unsubscribe', payload: { topic }, timestamp: Date.now() });
      }
    };
  }

  send(message: WSMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    }
  }

  onStateChange(listener: (state: WSConnectionState) => void): () => void {
    this.stateListeners.add(listener);
    return () => { this.stateListeners.delete(listener); };
  }

  getState(): WSConnectionState {
    return this.state;
  }

  private setState(state: WSConnectionState): void {
    this.state = state;
    this.stateListeners.forEach((listener) => listener(state));
  }

  private scheduleReconnect(): void {
    if (this.retryCount >= this.options.maxRetries) {
      console.error('[WS] Max retries reached');
      return;
    }
    const delay = this.options.retryDelay * Math.pow(this.options.backoffMultiplier, this.retryCount);
    console.log(`[WS] Reconnecting in ${delay}ms (attempt ${this.retryCount + 1})`);
    this.retryTimer = setTimeout(() => {
      this.retryCount++;
      this.connect();
    }, delay);
  }
}
```

### cn 유틸리티 (클래스 병합)

```typescript
// src/utils/cn.ts
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

### 수학 유틸리티

```typescript
// src/utils/math.ts
import type { Vector3, Quaternion } from '@/types';

export function distance3D(a: Vector3, b: Vector3): number {
  return Math.sqrt((a.x - b.x) ** 2 + (a.y - b.y) ** 2 + (a.z - b.z) ** 2);
}

export function quaternionToEuler(q: Quaternion): Vector3 {
  // Roll (x)
  const sinr = 2.0 * (q.w * q.x + q.y * q.z);
  const cosr = 1.0 - 2.0 * (q.x * q.x + q.y * q.y);
  const x = Math.atan2(sinr, cosr);

  // Pitch (y)
  const sinp = 2.0 * (q.w * q.y - q.z * q.x);
  const y = Math.abs(sinp) >= 1 ? Math.sign(sinp) * (Math.PI / 2) : Math.asin(sinp);

  // Yaw (z)
  const siny = 2.0 * (q.w * q.z + q.x * q.y);
  const cosy = 1.0 - 2.0 * (q.y * q.y + q.z * q.z);
  const z = Math.atan2(siny, cosy);

  return { x, y, z };
}

export function eulerToQuaternion(euler: Vector3): Quaternion {
  const cy = Math.cos(euler.z * 0.5);
  const sy = Math.sin(euler.z * 0.5);
  const cp = Math.cos(euler.y * 0.5);
  const sp = Math.sin(euler.y * 0.5);
  const cr = Math.cos(euler.x * 0.5);
  const sr = Math.sin(euler.x * 0.5);

  return {
    w: cr * cp * cy + sr * sp * sy,
    x: sr * cp * cy - cr * sp * sy,
    y: cr * sp * cy + sr * cp * sy,
    z: cr * cp * sy - sr * sp * cy,
  };
}

export function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

export function lerpVector3(a: Vector3, b: Vector3, t: number): Vector3 {
  return {
    x: lerp(a.x, b.x, t),
    y: lerp(a.y, b.y, t),
    z: lerp(a.z, b.z, t),
  };
}

export function radToDeg(rad: number): number {
  return rad * (180 / Math.PI);
}

export function degToRad(deg: number): number {
  return deg * (Math.PI / 180);
}
```

---

## 부록 B: 환경변수

```bash
# .env.example

# API
VITE_API_BASE_URL=/api
VITE_WS_URL=ws://localhost:8080/ws

# WebRTC
VITE_STUN_URL=stun:stun.l.google.com:19302
VITE_TURN_URL=turn:turn.amr-system.com:3478
VITE_TURN_USERNAME=amr
VITE_TURN_CREDENTIAL=secret

# Feature Flags
VITE_ENABLE_WEBGPU=false
VITE_ENABLE_MOCK=true

# Development
VITE_MOCK_WS_URL=ws://localhost:8081
```

---

## 부록 C: 공통 컴포넌트 API 상세

### Button

```typescript
interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  loading?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

// 사용 예:
// <Button variant="primary" size="md" loading={isSubmitting}>저장</Button>
// <Button variant="danger" leftIcon={<TrashIcon />}>삭제</Button>
// <Button variant="ghost" size="sm">취소</Button>
```

### Modal

```typescript
interface ModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  closeOnOverlayClick?: boolean; // 기본 true
}

// 사용 예:
// <Modal open={isOpen} onOpenChange={setIsOpen} title="로봇 등록" size="lg">
//   <RobotRegistrationForm onSubmit={handleSubmit} />
// </Modal>
```

### DataTable

```typescript
interface DataTableProps<T> {
  data: T[];
  columns: ColumnDef<T>[];         // TanStack Table ColumnDef
  isLoading?: boolean;
  emptyMessage?: string;
  pagination?: {
    page: number;
    limit: number;
    total: number;
    onPageChange: (page: number) => void;
  };
  sorting?: {
    sortBy: string;
    order: 'asc' | 'desc';
    onSortChange: (sortBy: string, order: 'asc' | 'desc') => void;
  };
  onRowClick?: (row: T) => void;
  rowClassName?: (row: T) => string;
}
```

### StatusIndicator

```typescript
interface StatusIndicatorProps {
  status: RobotStatus;
  showLabel?: boolean;   // 기본 true
  size?: 'sm' | 'md';
}

// 렌더링:
// IDLE: 녹색 원 + "대기"
// BUSY: 파란색 원 (깜박임) + "작업중"
// ERROR: 빨간색 원 + "오류"
// OFFLINE: 회색 원 + "오프라인"
```

### Toast (사용법)

```typescript
// ToastProvider 내에서 useToast 훅 사용
import { useToast } from '@/hooks/useToast';

function MyComponent() {
  const { toast } = useToast();

  const handleSave = () => {
    toast({ title: '저장 완료', description: '맵이 성공적으로 저장되었습니다.', variant: 'success' });
  };

  const handleError = () => {
    toast({ title: '오류 발생', description: '서버와의 연결에 실패했습니다.', variant: 'error', duration: 5000 });
  };
}

// variant: 'success' | 'error' | 'warning' | 'info'
// duration: 자동 닫힘 시간 (ms), 기본 3000, 0이면 수동 닫기
```

---

> **문서 끝**
> 이 명세서를 기반으로 개발을 진행한다. 스키마 또는 요구사항 변경 시 이 문서를 먼저 업데이트하고 해당 팀에 공유한다.
