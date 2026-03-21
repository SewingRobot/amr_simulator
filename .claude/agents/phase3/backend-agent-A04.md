# Agent Assignment: backend-agent — A-04 VDA5050 Real Robot

## Agent Profile
- **ID:** backend-agent | **Team:** Team 2 | **Phase:** 3 | **Duration:** Week 17-22
- **Feature:** A-04 VDA5050 Real Robot Integration

## Objective
Implement MQTT-based VDA5050 v2.0 communication with real AMR controllers.

## Week-by-Week Tasks

### Week 17-18: MQTT Client
- [ ] MQTT client (rumqttc crate), connection management, reconnection
- [ ] VDA5050 topic structure: `/v2/{manufacturer}/{serialNumber}/{topic}`
- [ ] Connection heartbeat message handling (online/offline detection)

### Week 19-20: Order & State
- [ ] Publish Order messages to MQTT (from Mission Service)
- [ ] Parse State messages → normalize to TelemetryMessage format
- [ ] Robot status sync (battery, errors, position from State)
- [ ] InstantAction support (e.g., emergency stop)

### Week 21-22: Reliability
- [ ] QoS handling (Order=1, State=0, Connection=1)
- [ ] Error/disconnect recovery, message replay
- [ ] Integration test with VDA5050 simulator (or Mosquitto mock)
- [ ] Telemetry from real robots feeds same pipeline as sim

## References
- `docs/features/advanced/A-04-vda5050-real-robot.md`
- `docs/integration/integration-spec.md` §vda5050

## Dependencies
- A-01 Mission System (generates orders)

## Definition of Done
- [ ] MQTT VDA5050 communication working
- [ ] Real robot telemetry normalized to same format as sim
- [ ] Online/offline detection functional
- [ ] Integration tests pass
