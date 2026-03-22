# Changelog

## [0.1.0-phase1] - 2026-03-22

### Added
- feat(sim): Custom lightweight physics engine with differential drive (C-01)
- feat(sim): 2D LiDAR ray casting with wall/obstacle/robot detection (C-01)
- feat(sim): TCP telemetry server (10Hz) and command server (C-01)
- feat(sim): Wall collision with bounce behavior (C-01)
- feat(frontend): Three.js/R3F 3D viewer with grid, walls, obstacles (C-02)
- feat(frontend): Robot model rendering with glTF fallback (C-02)
- feat(frontend): Camera controls (orbit, top-down, 2D/3D toggle) (C-02)
- feat(frontend): Real-time pose interpolation (10Hz→60fps lerp/slerp) (C-02)
- feat(frontend): WebSocket telemetry connection with auto-reconnect (C-02)
- feat(frontend): Connection status indicator (C-02)
- feat(frontend): 30 unit tests (robotStore, math, websocket, integration) (C-02)
- feat(backend): Axum REST API (auth, robots, maps CRUD) (C-03)
- feat(backend): JWT authentication with argon2 password hashing (C-03)
- feat(backend): WebSocket relay with topic subscription (C-03)
- feat(backend): Sim Engine TCP bridge with auto-reconnect (C-03)
- feat(backend): Asset/Map proxy endpoints (C-03)
- feat(backend): 10 auth integration tests (C-03)
- feat(map): A* pathfinding with Euclidean heuristic, 11 tests (C-04)
- feat(map): PostGIS schema (maps, roadmap_nodes, roadmap_edges) (C-04)
- feat(map): Python point cloud pipeline (Open3D) (C-04)
- feat(asset): gRPC skeleton, S3 storage, metadata CRUD (C-05)
- feat(asset): URDF→glTF Python conversion pipeline (C-05)
- feat(proto): 12 protobuf schema files (common, sim, asset, map, mission)
- feat(infra): Docker Compose (PostgreSQL+PostGIS, Redis, MinIO, MQTT)
- feat(e2e): Sim→Backend→Frontend telemetry pipeline (C-06)
- docs: Complete project planning (architecture, features, teams, integration, governance)

### Fixed
- fix(map): uuid v4 feature missing in Cargo.toml
- fix(backend): JWT leeway set to 0 for strict expiration
- fix(backend): sqlx::raw_sql for multi-statement migrations
- fix(frontend): React 18 downgrade for R3F compatibility
- fix(frontend): WebSocket subscribe field name (action→type)
- fix(backend): Telemetry JSON array parsing + 2D→3D pose conversion
- fix(sim): Wall collision clamping with 90° bounce
- fix(frontend): Wall/grid coordinates aligned with sim world (0-20, 0-15)
