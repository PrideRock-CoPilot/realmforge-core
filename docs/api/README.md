---
doc_id: DOC-API-README
title: "RealmForge Core — HTTP API Reference"
status: active
owner: api-architect
reviewers: [api-architect, backend, frontend]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: interfaces
work_path_ids: [WP-API-001]
related_decision_ids: []
related_file_ids: [FILE-CRATE-API-LIB]
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---
# RealmForge Core — HTTP API Reference

**Base URL:** `http://localhost:{port}`  
**Content-Type:** `application/json`  
**Auth:** Currently session-based via ActorScope (passed in request bodies where applicable)

---

## Table of Contents

- [Health](#health)
- [Sessions](#sessions)
- [Commands](#commands)
- [Audit](#audit)
- [Snapshots](#snapshots)
- [Rollback](#rollback)
- [Actors](#actors)
- [Work Packets](#work-packets)

---

## Health

### `GET /health`

Simple liveness check.

```bash
curl http://localhost:8080/health
```

**Response — 200 OK**
```json
{
  "status": "ok",
  "service": "control-api",
  "production_deploy_enabled": false
}
```

---

### `GET /v1/health/ready`

Readiness check — confirms the service is ready to accept traffic.

```bash
curl http://localhost:8080/v1/health/ready
```

**Response — 200 OK**
```json
{
  "status": "ok",
  "service": "control-api",
  "production_deploy_enabled": false
}
```

---

### `GET /v1/health/live`

Liveness check — confirms the process is alive.

```bash
curl http://localhost:8080/v1/health/live
```

**Response — 200 OK**
```json
{
  "status": "ok",
  "service": "control-api",
  "production_deploy_enabled": false
}
```

---

## Sessions

### `POST /v1/session`

Issue a new session for an actor within a tenant and project.

```bash
curl -X POST http://localhost:8080/v1/session \
  -H "Content-Type: application/json" \
  -d '{
    "actor_id": "alice",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "ttl_seconds": 3600
  }'
```

**Response — 201 Created**
```json
{
  "session": {
    "id": "ses_abc123",
    "session_id": "ses_abc123",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "actor_id": "alice",
    "state": "active",
    "issued_at": "2026-05-03T14:30:00Z",
    "expires_at": "2026-05-03T15:30:00Z",
    "ttl_seconds": 3600,
    "revoked_at": null
  }
}
```

---

### `GET /v1/session/{id}`

Get session details by ID. Validates that the session exists and has not expired.

```bash
curl http://localhost:8080/v1/session/ses_abc123
```

**Response — 200 OK**
```json
{
  "session": {
    "id": "ses_abc123",
    "session_id": "ses_abc123",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "actor_id": "alice",
    "state": "active",
    "issued_at": "2026-05-03T14:30:00Z",
    "expires_at": "2026-05-03T15:30:00Z",
    "revoked_at": null
  }
}
```

**Response — 404 Not Found**
```json
{
  "error": "SessionNotFound",
  "message": "session not found: ses_unknown"
}
```

---

### `DELETE /v1/session/{id}`

Revoke a session, marking it as revoked and preventing further use.

```bash
curl -X DELETE http://localhost:8080/v1/session/ses_abc123
```

**Response — 200 OK**
```json
{
  "status": "revoked",
  "session_id": "ses_abc123"
}
```

**Errors:** `SessionNotFound` (404)

---

## Commands

### `POST /v1/commands`

Propose a new bounded command within the command lifecycle.

```bash
curl -X POST http://localhost:8080/v1/commands \
  -H "Content-Type: application/json" \
  -d '{
    "action": "deploy.service",
    "target_type": "service",
    "target_id": "api-gateway",
    "payload": {
      "version": "2.1.0",
      "strategy": "rolling"
    }
  }'
```

**Response — 201 Created**
```json
{
  "command_id": "cmd_xyz789",
  "status": "Proposed",
  "action": "deploy.service",
  "target_type": "service",
  "target_id": "api-gateway"
}
```

---

### `PUT /v1/commands/{id}/authorize`

Authorize a proposed command against the policy engine. Requires an `ActorScope` in the request body.

```bash
curl -X PUT http://localhost:8080/v1/commands/cmd_xyz789/authorize \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "actor_id": "alice",
    "roles": [],
    "session_id": "ses_abc123",
    "allowed_actions": ["*"],
    "approval_state": "NotRequired",
    "execution_mode": "Approved",
    "context_updated_at": "2026-05-03T14:30:00Z",
    "expires_at": "2026-05-03T15:30:00Z"
  }'
```

**Response — 200 OK**
```json
{
  "command_id": "cmd_xyz789",
  "status": "Authorized",
  "action": "deploy.service",
  "target_type": "service",
  "target_id": "api-gateway"
}
```

**Errors:** `PolicyDenied` (403), `CommandNotFound` (404)

---

### `PUT /v1/commands/{id}/apply`

Apply an authorized command, executing its effects.

```bash
curl -X PUT http://localhost:8080/v1/commands/cmd_xyz789/apply \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "actor_id": "alice",
    "session_id": "ses_abc123",
    "allowed_actions": ["*"],
    "approval_state": "NotRequired",
    "execution_mode": "Approved"
  }'
```

**Response — 200 OK**
```json
{
  "command_id": "cmd_xyz789",
  "status": "Applied",
  "action": "deploy.service",
  "target_type": "service",
  "target_id": "api-gateway"
}
```

**Errors:** `CommandNotFound` (404), `CommandNotAuthorized` (400)

---

## Audit

### `GET /v1/audit/events`

Query audit events for a project with optional filters.

```bash
# All events (default limit 50)
curl "http://localhost:8080/v1/audit/events"

# Filtered by event type
curl "http://localhost:8080/v1/audit/events?event_type=command.applied"

# With pagination
curl "http://localhost:8080/v1/audit/events?limit=10&offset=20"

# Filtered by actor
curl "http://localhost:8080/v1/audit/events?actor_id=alice&entity_type=service"
```

**Response — 200 OK**
```json
{
  "events": [
    {
      "event_id": "evt_001",
      "previous_hash": null,
      "event_hash": "a1b2c3d4...",
      "event_type": "command.proposed",
      "actor_id": "alice",
      "entity_type": "command",
      "entity_id": "cmd_xyz789",
      "payload": { "action": "deploy.service" },
      "tenant_id": "acme-corp",
      "project_id": "project-alpha",
      "category": "command",
      "timestamp": "2026-05-03T14:30:00Z"
    }
  ],
  "total": 42
}
```

---

### `GET /v1/audit/chain/verify`

Verify the integrity of the full audit hash chain for the default project.

```bash
curl http://localhost:8080/v1/audit/chain/verify
```

**Response — 200 OK**
```json
{
  "chain_integrity": true,
  "event_count": 42
}
```

**When tampered:**
```json
{
  "chain_integrity": false,
  "event_count": 42
}
```

---

## Snapshots

### `POST /v1/snapshots`

Create a new content-addressed snapshot manifest.

```bash
curl -X POST http://localhost:8080/v1/snapshots \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "reason": "pre-deploy backup",
    "parent_snapshot_id": null,
    "previous_manifest_hash": null
  }'
```

**Response — 201 Created**
```json
{
  "snapshot_id": "snap_001",
  "status": "Created",
  "reason": "pre-deploy backup",
  "object_count": 0
}
```

---

### `GET /v1/snapshots`

List snapshots for the default project with pagination.

```bash
curl "http://localhost:8080/v1/snapshots?limit=10&offset=0"
```

**Response — 200 OK**
```json
[
  {
    "snapshot_id": "snap_001",
    "status": "Created",
    "reason": "pre-deploy backup",
    "object_count": 0
  },
  {
    "snapshot_id": "snap_002",
    "status": "KnownGood",
    "reason": "post-deploy verify",
    "object_count": 12
  }
]
```

---

### `GET /v1/snapshots/{id}`

Get a single snapshot by ID (returns validation report).

```bash
curl http://localhost:8080/v1/snapshots/snap_001
```

**Response — 200 OK**
```json
{
  "report": {
    "snapshot_id": "snap_001",
    "manifest_hash_valid": true,
    "object_refs_integrity": true,
    "status": "Created",
    "valid": true
  }
}
```

---

### `GET /v1/snapshots/compare`

Compare two snapshots and return the delta (added, removed, changed objects).

```bash
curl "http://localhost:8080/v1/snapshots/compare?from_id=snap_001&to_id=snap_002"
```

**Response — 200 OK**
```json
{
  "delta": {
    "added": ["/services/api-gateway/v2.1.0"],
    "removed": ["/services/api-gateway/v2.0.0"],
    "changed": ["/config/deploy.yaml"],
    "unchanged": ["/infra/network.tf"]
  }
}
```

---

### `POST /v1/snapshots/{id}/validate`

Validate a snapshot's manifest hash integrity and object reference consistency.

```bash
curl -X POST http://localhost:8080/v1/snapshots/snap_001/validate
```

**Response — 200 OK**
```json
{
  "report": {
    "snapshot_id": "snap_001",
    "manifest_hash_valid": true,
    "object_refs_integrity": true,
    "status": "Created",
    "valid": true
  }
}
```

---

## Rollback

### `POST /v1/rollback/preview`

Preview what a rollback between two snapshots would restore. Does not execute.

```bash
curl -X POST http://localhost:8080/v1/rollback/preview \
  -H "Content-Type: application/json" \
  -d '{
    "from_snapshot_id": "snap_002",
    "to_snapshot_id": "snap_001"
  }'
```

**Response — 200 OK**
```json
{
  "preview": {
    "id": "prev_001",
    "from_snapshot_id": "snap_002",
    "to_snapshot_id": "snap_001",
    "delta": {
      "added": ["/services/api-gateway/v2.0.0"],
      "removed": ["/services/api-gateway/v2.1.0"],
      "changed": [],
      "unchanged": ["/infra/network.tf"]
    },
    "objects_to_restore": 1,
    "estimated_impact": "1 object will be restored",
    "blockers": [],
    "created_at": "2026-05-03T14:35:00Z"
  }
}
```

---

### `POST /v1/rollback/execute`

Execute a rollback from one snapshot to another, restoring affected objects.

```bash
curl -X POST http://localhost:8080/v1/rollback/execute \
  -H "Content-Type: application/json" \
  -d '{
    "from_snapshot_id": "snap_002",
    "to_snapshot_id": "snap_001",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha"
  }'
```

**Response — 201 Created**
```json
{
  "result": {
    "new_snapshot_id": "snap_003",
    "objects_restored": 1,
    "tables_restored": 0
  }
}
```

---

### `GET /v1/rollback/{id}/verify`

Verify a rollback snapshot's consistency and hash integrity.

```bash
curl http://localhost:8080/v1/rollback/snap_003/verify
```

**Response — 200 OK**
```json
{
  "verification": {
    "snapshot_id": "snap_003",
    "chain_consistent": true,
    "integrity_ok": true,
    "status": "Verified"
  }
}
```

---

## Actors

### `GET /v1/actors/{id}/scope?session_id={session_id}`

Get the full ActorScope for an actor and session.

```bash
curl "http://localhost:8080/v1/actors/alice/scope?session_id=ses_abc123"
```

**Response — 200 OK**
```json
{
  "scope": {
    "tenant_id": "default",
    "project_id": "default",
    "actor_id": "alice",
    "allowed_actions": ["*"],
    "approval_state": "NotRequired",
    "execution_mode": "Approved",
    "session_id": "ses_abc123"
  }
}
```

---

## Work Packets

### `POST /v1/work-packets/generate`

Generate a scoped work packet for an AI agent from a work path node.

```bash
curl -X POST http://localhost:8080/v1/work-packets/generate \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "actor_id": "agent-bob",
    "work_path_node_id": "deploy-v2-orchestration",
    "objective": "Deploy version 2.1.0 of the API gateway service using rolling update strategy",
    "allowed_file_paths": [
      "/deploy/k8s/api-gateway.yaml",
      "/config/api-gateway.env"
    ]
  }'
```

**Response — 201 Created**
```json
{
  "packet_id": "pkt_001",
  "status": "Generated"
}
```

---

### `POST /v1/work-packets/{id}/validate`

Validate a work packet's scope boundaries and cost budget.

```bash
curl -X POST http://localhost:8080/v1/work-packets/pkt_001/validate
```

**Response — 200 OK**
```json
{
  "result": {
    "packet_id": "pkt_001",
    "scope_valid": true,
    "allowed_actions": ["deploy.service"],
    "restricted_paths": ["/deploy/k8s/api-gateway.yaml"],
    "cost_budget": {
      "max_tokens": 100000,
      "max_api_calls": 50,
      "max_seconds": 600
    }
  }
}
```

---

## Error Response Format

All API errors follow this structure:

```json
{
  "error": "ErrorKind",
  "message": "Human-readable description of what went wrong"
}
```

**Common Error Codes:**

| HTTP Status | Error Kind | Description |
|-------------|-----------|-------------|
| 400 | `BadRequest` | Invalid request parameters or body |
| 404 | `CommandNotFound` | The requested resource was not found |
| 403 | `PolicyDenied` | The action was denied by policy engine |
| 500 | `InternalError` | Unexpected server error |

---

## Quick Start: End-to-End Flow

```bash
# 1. Issue a session
SESSION=$(curl -s -X POST http://localhost:8080/v1/session \
  -H "Content-Type: application/json" \
  -d '{"actor_id":"alice","tenant_id":"acme","project_id":"proj","ttl_seconds":3600}' \
  | jq -r '.session.id')

# 2. Propose a command
CMD=$(curl -s -X POST http://localhost:8080/v1/commands \
  -H "Content-Type: application/json" \
  -d '{"action":"deploy.service","target_type":"service","target_id":"gw","payload":{}}' \
  | jq -r '.command_id')

# 3. Authorize the command
SCOPE='{"tenant_id":"acme","project_id":"proj","actor_id":"alice","allowed_actions":["*"],"approval_state":"NotRequired","execution_mode":"Approved"}'

curl -s -X PUT "http://localhost:8080/v1/commands/$CMD/authorize" \
  -H "Content-Type: application/json" \
  -d "$SCOPE" | jq

# 4. Apply the command
curl -s -X PUT "http://localhost:8080/v1/commands/$CMD/apply" \
  -H "Content-Type: application/json" \
  -d "$SCOPE" | jq

# 5. Verify the audit chain
curl -s http://localhost:8080/v1/audit/chain/verify | jq
```
