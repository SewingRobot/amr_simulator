# Agent Assignment: proto-agent — Proto & Interface Management

## Agent Profile
- **ID:** proto-agent
- **Team:** Integration
- **Phase:** 1 (Prototype)
- **Feature:** Cross-cutting (proto/, integration-spec)
- **Duration:** Week 1-8

## Objective
Define and maintain all shared Protobuf schemas, process RFCs, and ensure interface consistency across teams.

## Week-by-Week Tasks

### Week 1: Schema Drafting
- [x] Create `proto/` directory structure: common/, simulation/, asset/, map/, mission/
- [x] Draft `common/geometry.proto` (Vector3, Quaternion, Pose3D, Pose2D, Twist, BoundingBox)
- [x] Draft `common/telemetry.proto` (TelemetryMessage, RobotState, BatteryStatus)
- [x] Draft `common/identifiers.proto` (wrapper types for UUIDs)
- [x] Draft `simulation/sim_service.proto` (SimulationService RPCs)
- [x] Draft `asset/asset_service.proto` (AssetService RPCs)
- [x] Draft `map/map_service.proto` (MapService RPCs)
- [x] Draft `mission/mission_types.proto` (Mission, MissionStep, MissionStatus)
- [x] Submit for all-team review (implicit)
- **Deliverable:** Complete proto draft for review

### Week 2: Proto Freeze
- [x] Collect team feedback on proto drafts
- [x] Resolve conflicts and finalize schemas
- [ ] Verify code generation — PARTIALLY (sim engine had protoc version mismatch)
- [ ] Tag: `proto-freeze-v1` — NOT DONE
- [x] Update `docs/integration/integration-spec.md` with final schemas
- **Deliverable:** Proto Freeze complete, all teams can generate code

### Week 3-8: Maintenance
- [ ] Process incoming RFCs — NO RFCs CREATED
- [ ] Apply approved changes to proto files
- [ ] Verify backward compatibility (buf breaking check) — NOT DONE
- [ ] Notify teams of proto updates
- [x] Maintain integration-spec.md consistency
- **Deliverable:** Proto files stay consistent and up-to-date

## Reference Documents
- Integration spec: `docs/integration/integration-spec.md`
- All feature specs: `docs/features/core/C-01~C-06`
- RFC process: `.claude/skills/communication.md` §RFC

## Dependencies
- None (this agent runs first)

## Constraints
- Exclusive write access to `proto/`
- All changes must be backward-compatible after Proto Freeze
- Breaking changes require RFC with all-team approval

## Definition of Done
- [x] All proto files defined — code generation PARTIALLY verified (protoc version mismatch)
- [ ] Proto Freeze tag created — NOT DONE
- [x] integration-spec.md updated
- [ ] All RFCs processed — NO RFCs CREATED
