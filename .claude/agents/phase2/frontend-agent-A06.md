# Agent Assignment: frontend-agent — A-06 Map Editor

## Agent Profile
- **ID:** frontend-agent (or frontend-agent-editor if split)
- **Team:** Team 1 | **Phase:** 2 | **Duration:** Week 9-16
- **Feature:** A-06 Map Editor

## Objective
Build interactive map editor with Potree LOD viewer, roadmap graph editing, semantic region drawing, and undo/redo.

## Week-by-Week Tasks

### Week 9-10: Potree Integration
- [ ] Potree octree loader (progressive LOD tile loading)
- [ ] Point budget control (limit visible points for performance)
- [ ] Camera-aware tile loading (load tiles in view frustum)
- [ ] Point cloud rendering in R3F scene

### Week 11-12: Roadmap Editing
- [ ] Add waypoint node: click on 3D scene → create node
- [ ] Move node: drag to reposition
- [ ] Delete node: select + delete key
- [ ] Draw edge: click source → click target
- [ ] Edge properties panel: max speed, direction, allowed robot types
- [ ] Node properties: name, type (waypoint/charging/dock)

### Week 13-14: Semantic Regions
- [ ] Polygon drawing tool: click vertices on 3D scene
- [ ] Region types: no-go, speed-limit, charging, loading dock
- [ ] Properties panel per region type
- [ ] Obstacle placement: box/cylinder shapes
- [ ] Visual rendering (semi-transparent colored overlays)

### Week 15-16: Undo/Redo & Save
- [ ] Command pattern: AddNode, MoveNode, DeleteNode, AddEdge, AddRegion, etc.
- [ ] Undo stack + Redo stack
- [ ] Ctrl+Z / Ctrl+Shift+Z shortcuts
- [ ] Save map version (PUT to Backend)
- [ ] Load map version (GET from Backend)

## References
- `docs/features/advanced/A-06-map-editor.md`

## Dependencies
- C-02 3D Viewer, C-04 Map Core (tile serving, roadmap API)

## Definition of Done
- [ ] Potree LOD point cloud rendering (>1M points smooth)
- [ ] Roadmap graph full CRUD in 3D
- [ ] Semantic region drawing and editing
- [ ] Undo/redo working
- [ ] Save/load map versions
