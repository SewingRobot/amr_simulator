# Skill: Coding Standards

---

## 1. Common Principles
Consistency · Explicitness · Minimal dependencies · Explicit error handling · Prefer immutability

---

## 2. TypeScript / React (Frontend)

**Tools:** ESLint + Prettier, `strict: true`, `noUncheckedIndexedAccess: true`

**Components:** Functional only, named export, Props interface above component
```typescript
export interface RobotInfoPanelProps {
  robot: Robot;
  onClose: () => void;
}
export function RobotInfoPanel({ robot, onClose }: RobotInfoPanelProps) { ... }
```
**Files:** `PascalCase.tsx` (components), `camelCase.ts` (utils/hooks)
**State:** Zustand (1 file per domain), TanStack Query (server state, custom hook wrapper). No prop drilling beyond 2 levels.

---

## 3. Rust (Backend, Asset Manager, Map Manager)

**Tools:** `rustfmt` + `clippy` (`#![warn(clippy::all)]`), Edition 2024

**Errors:** `thiserror` (define) + `anyhow` (propagate, handlers only). No `.unwrap()` in prod.
```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Robot not found: {0}")] NotFound(Uuid),
    #[error("Unauthorized")] Unauthorized,
    #[error(transparent)] Internal(#[from] anyhow::Error),
}
```
**Architecture:** Handler → Service → Repository (3-layer), trait-based repos
**Naming:** Modules `snake_case`, Types `PascalCase`, Functions `snake_case`, Constants `SCREAMING_SNAKE_CASE`

---

## 4. C++ (Simulation Engine)

**Tools:** `clang-format` (Google base) + `clang-tidy`
**Naming:** Classes `PascalCase`, Methods `camelCase`, Members `m_camelCase`, Constants `kPascalCase`, Files `snake_case.h/.cpp`
**Memory:** `unique_ptr` (ownership), `shared_ptr` (shared only), raw ptr (observation), RAII
**Header order:** Standard → Third-party → Project

---

## 5. Python (Processing Pipelines)

**Tools:** `ruff` (format + lint), type hints required
**Patterns:** Functions > classes, `pathlib.Path` (no string concat), `logging` (no print), config via `dataclass`/`pydantic`

---

## 6. Common Patterns

**Logging:** Structured JSON — fields: `timestamp`, `level`, `service`, `message`
**Config:** env vars > config files > hardcoded defaults. Secrets in env vars only.
**REST response:** `{data: {...}}` or `{error: {code, message, details}}`
**Test naming:** `test_{what}_{condition}_{expected}`
**TODO:** `// TODO(team-id): description [feature-ref]`
**Code docs:** Public API required (JSDoc/rustdoc/Doxygen/docstring), internal only for non-obvious logic
**Proto codegen:** Not in git, auto-generated at build (`tonic-build`, `protoc-gen-ts`, `grpc_tools`)
