# Skill: API Design

Rules for designing service-to-service and client-server interfaces.

---

## 1. Protocol Selection

| Scenario | Protocol | Reason |
|----------|----------|--------|
| Frontend ↔ Backend CRUD | REST (JSON) | Browser-compatible, cacheable |
| Frontend ↔ Backend real-time | WebSocket (JSON) | Bidirectional, 10Hz telemetry |
| Frontend ↔ Backend video | WebRTC | Low-latency video stream |
| Backend ↔ internal services | gRPC (Protobuf) | Strongly typed, streaming, high-perf |
| Backend ↔ robots | MQTT (VDA5050 JSON) | Lightweight, industry standard |

---

## 2. REST API Rules

**URL pattern:** `{method} /api/{version}/{resource}[/{id}][/{sub-resource}]`
- Plural resources: `/api/v1/robots`, `/api/v1/maps`
- Hierarchy: `/api/v1/maps/{id}/roadmap`
- Actions: `POST /api/v1/missions/{id}/assign` (verbs only for non-CRUD)

**Response format:**
```json
{ "data": { ... }, "meta": { "total": 100, "page": 1, "limit": 20, "has_next": true } }
{ "error": { "code": "VALIDATION_ERROR", "message": "...", "details": {} } }
```

**Status codes:** 200 (OK), 201 (Created), 204 (Deleted), 400 (Validation), 401 (Unauth), 403 (Forbidden), 404 (Not Found), 409 (Conflict), 422 (Business Logic), 500 (Internal)

**Pagination:** `?page=1&limit=20&sort=name&order=asc` (page 1-based, limit max 100)

**Auth:** `Authorization: Bearer {jwt}`, access 1h, refresh 7d

---

## 3. gRPC Rules

**Naming:** `package amr.{domain}.v1; service {Domain}Service { ... }`

| Pattern | Use | Example |
|---------|-----|---------|
| Unary | Single CRUD | `GetRobot(Req) returns (Robot)` |
| Server streaming | Continuous data | `StreamTelemetry(Req) returns (stream Msg)` |
| Client streaming | Large upload | `UploadPointCloud(stream Chunk) returns (Job)` |
| Bidirectional | Real-time two-way | `StartSimulation(Req) returns (stream Event)` |

**Message rules:** enum first value `UNSPECIFIED = 0`, optional fields have documented defaults, gRPC status codes for errors.

---

## 4. WebSocket Rules

**Envelope:** `{"type": "telemetry|mission_update|alert|command", "topic": "...", "payload": {...}, "timestamp": ms}`

**Subscription:** `{"type": "subscribe", "topics": ["telemetry:*", "alerts"]}`

**Connection:** Heartbeat ping/pong 30s, 3 missed → disconnect, client reconnect with exponential backoff (1s→30s max)

---

## 5. MQTT / VDA5050 Rules

**Topics:** `/v2/{manufacturer}/{serialNumber}/{topic}` (order, instantActions, state, visualization, connection)

**QoS:** Order=1 (at-least-once), State=0 (at-most-once), Connection=1

---

## 6. Interface Change Process

1. Agent writes RFC (`docs/rfcs/`) → 2. Affected teams review → 3. `proto-agent` modifies proto/ → 4. All teams regenerate code → 5. Update integration-spec.md
