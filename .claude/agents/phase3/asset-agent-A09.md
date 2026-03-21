# Agent Assignment: asset-agent — A-09 OpenUSD Pipeline

## Agent Profile
- **ID:** asset-agent | **Team:** Team 4 | **Phase:** 3 | **Duration:** Week 17-22
- **Feature:** A-09 OpenUSD Pipeline

## Objective
Build full OpenUSD conversion pipeline: URDF→USD, SDF→USD, USD→glTF, with UsdPhysics schema mapping.

## Week-by-Week Tasks

### Week 17-18: URDF→USD Converter
- [ ] OpenUSD SDK integration (Python pxr library)
- [ ] URDF parser → USD scene graph (links as Xform, joints as UsdPhysics joints)
- [ ] Mesh embedding (STL/DAE → USD mesh prims)
- [ ] Material/color mapping

### Week 19-20: SDF→USD & Physics
- [ ] SDF world parser → USD stage (models, links, collisions)
- [ ] UsdPhysics schema mapping (mass, inertia, collision shapes)
- [ ] Physics material properties (friction, restitution)
- [ ] World-level USD (ground plane, gravity)

### Week 21-22: USD→glTF & Integration
- [ ] USD→glTF/GLB converter for web rendering
- [ ] Mesh optimization (Draco compression, texture resize)
- [ ] ConvertAsset RPC updated for USD flows
- [ ] Integration test: URDF upload → USD → glTF → Frontend renders
- [ ] Sim Engine loads USD files (coordinate with sim-agent)

## References
- `docs/features/advanced/A-09-openusd-pipeline.md`
- `docs/teams/team4-asset-manager.md`

## Dependencies
- C-05 Asset Core (upload/download infrastructure)
- Coordination with sim-agent for USD loading in engine

## Definition of Done
- [ ] URDF→USD conversion with physics properties
- [ ] SDF→USD conversion
- [ ] USD→glTF for web rendering
- [ ] Integration tests pass
