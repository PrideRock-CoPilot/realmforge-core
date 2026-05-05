# RealmForge Core — MCP Tool Reference

This document describes every tool exposed by the `agent-mcp` crate. These tools are designed to be consumed by AI agents through the Model Context Protocol (MCP).

**JSON-RPC Invocation Pattern:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "<tool_name>",
    "arguments": { ... }
  }
}
```

All tools return an `McpToolResult` wrapper:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

On failure:

```json
{
  "success": false,
  "data": null,
  "error": "<error message>"
}
```

**Common Error Categories:**

| Error | Meaning |
|-------|---------|
| `unknown tool: <name>` | Tool name not recognized |
| `invalid arguments: ...` | JSON deserialization failure or type mismatch |
| `service error: ...` | Underlying service/database failure |
| `<entity> not found` | Requested session/command/snapshot/actor/skill not found |
| `policy denied: ...` | Action blocked by policy engine |

---

## Table of Contents

- [Session Tools](#session-tools)
- [Command Tools](#command-tools)
- [Audit Tools](#audit-tools)
- [Snapshot Tools](#snapshot-tools)
- [Rollback Tools](#rollback-tools)
- [Actor Tools](#actor-tools)
- [Skill Tools](#skill-tools)
- [Work Packet Tools](#work-packet-tools)

---

## Session Tools

### core_issue_session

Issue a new session for an actor within a tenant and project.

**Required args:** `actor_id`, `tenant_id`, `project_id`, `ttl_seconds`

```json
{
  "name": "core_issue_session",
  "arguments": {
    "actor_id": "alice",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "ttl_seconds": 3600
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
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
  },
  "error": null
}
```

**Errors:** Service errors if the store is unreachable.

---

### core_renew_session

Renew an existing session with a new TTL. Extends the session's expiration time.

**Required args:** `session_id`, `ttl_seconds`

```json
{
  "name": "core_renew_session",
  "arguments": {
    "session_id": "ses_abc123",
    "ttl_seconds": 7200
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "id": "ses_abc123",
    "session_id": "ses_abc123",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "actor_id": "alice",
    "state": "active",
    "issued_at": "2026-05-03T14:30:00Z",
    "expires_at": "2026-05-03T16:30:00Z",
    "ttl_seconds": 7200,
    "revoked_at": null
  },
  "error": null
}
```

**Errors:** `session not found` (404 equivalent)

---

### core_revoke_session

Revoke a session, marking it as revoked and preventing further use.

**Required args:** `session_id`, `reason`

```json
{
  "name": "core_revoke_session",
  "arguments": {
    "session_id": "ses_abc123",
    "reason": "Session rotation before deployment"
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "status": "revoked",
    "session_id": "ses_abc123"
  },
  "error": null
}
```

**Errors:** `session not found`

---

## Command Tools

### core_propose_command

Propose a new bounded command. Commands follow a lifecycle: Proposed → Authorized → Applied.

**Required args:** `scope`, `action`, `target_type`, `target_id`, `payload`

The `scope` argument is an `ActorScope` object that provides authorization context (tenant, project, actor, session, allowed actions, etc.).

```json
{
  "name": "core_propose_command",
  "arguments": {
    "scope": {
      "tenant_id": "acme-corp",
      "project_id": "project-alpha",
      "actor_id": "alice",
      "roles": ["admin"],
      "session_id": "ses_abc123",
      "skill_session_id": null,
      "requested_skill_id": null,
      "active_skill_id": null,
      "allowed_actions": ["*"],
      "approval_id": null,
      "approval_state": "not_required",
      "execution_mode": "approved",
      "context_updated_at": "2026-05-03T14:30:00Z",
      "expires_at": "2026-05-03T15:30:00Z"
    },
    "action": "deploy.service",
    "target_type": "service",
    "target_id": "api-gateway",
    "payload": {
      "version": "2.1.0",
      "strategy": "rolling"
    }
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "id": "cmd_xyz789",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "actor_id": "alice",
    "action": "deploy.service",
    "target_type": "service",
    "target_id": "api-gateway",
    "payload": {
      "version": "2.1.0",
      "strategy": "rolling"
    },
    "status": "proposed",
    "created_at": "2026-05-03T14:30:00Z"
  },
  "error": null
}
```

**Errors:** `policy denied` (if the scope's execution mode is ReadOnly or Proposal), `validation error` (if payload fails schema checks)

---

### core_authorize_command_action

Authorize a proposed command against the policy engine. Transitions a command from `Proposed` to `Authorized` (or `Denied`).

**Required args:** `command_id`, `scope`

```json
{
  "name": "core_authorize_command_action",
  "arguments": {
    "command_id": "cmd_xyz789",
    "scope": {
      "tenant_id": "acme-corp",
      "project_id": "project-alpha",
      "actor_id": "alice",
      "roles": ["admin"],
      "session_id": "ses_abc123",
      "allowed_actions": ["*"],
      "approval_state": "not_required",
      "execution_mode": "approved",
      "context_updated_at": "2026-05-03T14:30:00Z",
      "expires_at": "2026-05-03T15:30:00Z"
    }
  }
}
```

**Expected return (authorized):**

```json
{
  "success": true,
  "data": {
    "id": "cmd_xyz789",
    "status": "authorized",
    "action": "deploy.service",
    "target_type": "service",
    "target_id": "api-gateway",
    "payload": { "version": "2.1.0", "strategy": "rolling" },
    "actor_id": "alice"
  },
  "error": null
}
```

**Expected return (denied):**

```json
{
  "success": true,
  "data": {
    "id": "cmd_xyz789",
    "status": "denied",
    "denial": {
      "code": "ActorUnauthorized",
      "message": "Actor 'alice' is not authorized to perform 'deploy.service'",
      "detail": null
    }
  },
  "error": null
}
```

**Errors:** `command not found`, `policy denied`

---

### core_apply_command

Apply an authorized command, executing its effects. Transitions a command from `Authorized` to `Applied` (or `Failed`).

**Required args:** `command_id`, `scope`

```json
{
  "name": "core_apply_command",
  "arguments": {
    "command_id": "cmd_xyz789",
    "scope": {
      "tenant_id": "acme-corp",
      "project_id": "project-alpha",
      "actor_id": "alice",
      "session_id": "ses_abc123",
      "allowed_actions": ["*"],
      "approval_state": "not_required",
      "execution_mode": "approved",
      "context_updated_at": "2026-05-03T14:30:00Z",
      "expires_at": "2026-05-03T15:30:00Z"
    }
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "id": "cmd_xyz789",
    "status": "applied",
    "action": "deploy.service",
    "target_type": "service",
    "target_id": "api-gateway",
    "payload": { "version": "2.1.0", "strategy": "rolling" },
    "actor_id": "alice"
  },
  "error": null
}
```

**Errors:** `command not found`, `policy denied` (if scope is stale or expired), `invalid command transition` (if command is not in Authorized state)

---

## Audit Tools

### core_query_events

Query audit events for a project with optional filters.

**Required args:** `project_id`
**Optional args:** `event_type`, `actor_id`, `entity_type`, `limit` (default: 50), `offset` (default: 0)

```json
{
  "name": "core_query_events",
  "arguments": {
    "project_id": "project-alpha",
    "event_type": "command.applied",
    "actor_id": "alice",
    "entity_type": "service",
    "limit": 10,
    "offset": 0
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "events": [
      {
        "event_id": "evt_001",
        "previous_hash": null,
        "event_hash": "a1b2c3d4e5f6...",
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
  },
  "error": null
}
```

**Errors:** Service errors if the store is unreachable.

---

### core_verify_chain

Verify the integrity of the full audit hash chain for a project. Checks that every event's `previous_hash` matches the hash of the preceding event, and that `event_hash` values are consistent.

**Required args:** `project_id`

```json
{
  "name": "core_verify_chain",
  "arguments": {
    "project_id": "project-alpha"
  }
}
```

**Expected return (chain intact):**

```json
{
  "success": true,
  "data": {
    "chain_integrity": true,
    "event_count": 42,
    "anchor_hash": "f1e2d3c4..."
  },
  "error": null
}
```

**Expected return (tampered chain):**

```json
{
  "success": true,
  "data": {
    "chain_integrity": false,
    "event_count": 42,
    "breach_index": 17,
    "anchor_hash": null
  },
  "error": null
}
```

**Errors:** Service errors.

---

## Snapshot Tools

### core_create_snapshot

Create a new content-addressed snapshot manifest. A snapshot captures the state of tracked objects at a point in time.

**Required args:** `tenant_id`, `project_id`, `reason`
**Optional args:** `parent_snapshot_id`

```json
{
  "name": "core_create_snapshot",
  "arguments": {
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "reason": "pre-deploy backup",
    "parent_snapshot_id": "snap_001"
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "snapshot_id": "snap_002",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "status": "draft",
    "reason": "pre-deploy backup",
    "parent_snapshot_id": "snap_001",
    "object_manifest": {},
    "manifest_hash": "abc123def456...",
    "created_at": "2026-05-03T14:30:00Z"
  },
  "error": null
}
```

**Errors:** Service errors.

---

### core_validate_snapshot

Validate a snapshot's manifest hash integrity and object reference consistency. Checks that:
- The manifest hash matches the recomputed hash of manifest contents
- All referenced object IDs exist in the object store

**Required args:** `snapshot_id`

```json
{
  "name": "core_validate_snapshot",
  "arguments": {
    "snapshot_id": "snap_002"
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "snapshot_id": "snap_002",
    "manifest_hash_valid": true,
    "object_refs_integrity": true,
    "status": "draft",
    "valid": true
  },
  "error": null
}
```

**Errors:** `snapshot not found`

---

### core_compare_snapshots

Compare two snapshots and return the delta — identifying added, removed, and changed objects between them.

**Required args:** `from_id`, `to_id`

```json
{
  "name": "core_compare_snapshots",
  "arguments": {
    "from_id": "snap_001",
    "to_id": "snap_002"
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "delta": {
      "added": ["/services/api-gateway/v2.1.0", "/config/deploy-v2.yaml"],
      "removed": ["/services/api-gateway/v2.0.0"],
      "changed": ["/infra/network.tf"],
      "unchanged": ["/infra/dns.tf", "/config/base.yaml"]
    }
  },
  "error": null
}
```

**Errors:** `snapshot not found` (for either id)

---

## Rollback Tools

### core_preview_rollback

Preview what a rollback between two snapshots would restore. Does NOT apply any changes. Useful for impact assessment before execution.

**Required args:** `from_snapshot_id`, `to_snapshot_id`

```json
{
  "name": "core_preview_rollback",
  "arguments": {
    "from_snapshot_id": "snap_002",
    "to_snapshot_id": "snap_001"
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
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
  },
  "error": null
}
```

**Errors:** `snapshot not found`, `rollback preview not found`

---

### core_execute_rollback

Execute a rollback from one snapshot to another, restoring affected objects. Creates a new snapshot recording the rollback state.

**Required args:** `from_snapshot_id`, `to_snapshot_id`, `tenant_id`, `project_id`

```json
{
  "name": "core_execute_rollback",
  "arguments": {
    "from_snapshot_id": "snap_002",
    "to_snapshot_id": "snap_001",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha"
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "new_snapshot_id": "snap_003",
    "objects_restored": 1,
    "tables_restored": 0,
    "status": "completed"
  },
  "error": null
}
```

**Errors:** `snapshot not found`, service errors during restore.

---

### core_verify_rollback

Verify a rollback snapshot's consistency and hash integrity. Checks that the snapshot created by a rollback is internally consistent and has verifiable hashes.

**Required args:** `snapshot_id`

```json
{
  "name": "core_verify_rollback",
  "arguments": {
    "snapshot_id": "snap_003"
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "snapshot_id": "snap_003",
    "chain_consistent": true,
    "integrity_ok": true,
    "status": "Verified"
  },
  "error": null
}
```

**Errors:** `snapshot not found`

---

## Actor Tools

### core_get_actor_scope

Get the full `ActorScope` for an actor and session. Returns the actor's roles, allowed actions, execution mode, approval state, and session expiry information.

**Required args:** `actor_id`, `session_id`

```json
{
  "name": "core_get_actor_scope",
  "arguments": {
    "actor_id": "alice",
    "session_id": "ses_abc123"
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "actor_id": "alice",
    "roles": ["admin", "operator"],
    "session_id": "ses_abc123",
    "skill_session_id": null,
    "requested_skill_id": null,
    "active_skill_id": null,
    "allowed_actions": ["*"],
    "approval_id": null,
    "approval_state": "not_required",
    "execution_mode": "approved",
    "context_updated_at": "2026-05-03T14:30:00Z",
    "expires_at": "2026-05-03T15:30:00Z"
  },
  "error": null
}
```

**Errors:** `actor not found`, `session not found`

---

## Skill Tools

### core_register_skill

Register a new skill within a tenant/project. A skill defines a named set of allowed actions and is bound to a tenant.

**Required args:** `skill_id`, `tenant_id`, `name`, `allowed_actions`
**Optional args:** `project_id`

```json
{
  "name": "core_register_skill",
  "arguments": {
    "skill_id": "skill.deploy_engineer",
    "tenant_id": "acme-corp",
    "name": "Deploy Engineer",
    "project_id": "project-alpha",
    "allowed_actions": [
      "deploy.service",
      "deploy.rollback",
      "config.read",
      "config.write"
    ]
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "id": "skill.deploy_engineer",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "version_id": "v1",
    "name": "Deploy Engineer",
    "allowed_actions": [
      "deploy.service",
      "deploy.rollback",
      "config.read",
      "config.write"
    ],
    "integrity_state": "pending",
    "approved": true
  },
  "error": null
}
```

**Errors:** Service errors.

---

### core_activate_skill_session

Activate a skill session, binding a registered skill to a new session with a TTL. The skill must be registered and in a usable state (approved + integrity valid).

**Required args:** `skill_id`, `tenant_id`, `project_id`, `ttl_seconds`

```json
{
  "name": "core_activate_skill_session",
  "arguments": {
    "skill_id": "skill.deploy_engineer",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "ttl_seconds": 1800
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "id": "ss_001",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "skill_id": "skill.deploy_engineer",
    "state": "active",
    "issued_at": "2026-05-03T14:30:00Z",
    "expires_at": "2026-05-03T15:00:00Z"
  },
  "error": null
}
```

**Errors:** `skill not found`, service errors if skill integrity check fails.

---

## Work Packet Tools

### core_generate_work_packet

Generate a scoped work packet for an AI agent from a work path node. The work packet defines exact boundaries (file paths, objective, cost budget) for agent execution.

**Required args:** `tenant_id`, `project_id`, `actor_id`, `work_path_node_id`, `objective`, `allowed_file_paths`

```json
{
  "name": "core_generate_work_packet",
  "arguments": {
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "actor_id": "agent-bob",
    "work_path_node_id": "deploy-v2-orchestration",
    "objective": "Deploy version 2.1.0 of the API gateway service using rolling update strategy",
    "allowed_file_paths": [
      "/deploy/k8s/api-gateway.yaml",
      "/config/api-gateway.env"
    ]
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "id": "pkt_001",
    "agent_id": "agent-bob",
    "work_path_node_id": "deploy-v2-orchestration",
    "objective": "Deploy version 2.1.0 of the API gateway service using rolling update strategy",
    "allowed_file_paths": [
      "/deploy/k8s/api-gateway.yaml",
      "/config/api-gateway.env"
    ],
    "denied_file_paths": [],
    "required_contracts": [],
    "required_tests": [],
    "required_trace_points": [],
    "rollback_anchor": null,
    "cost_budget": {
      "max_tokens": 100000,
      "max_api_calls": 50,
      "max_seconds": 600
    },
    "permission_scope": {
      "allowed_actions": ["deploy.service", "config.read"],
      "denied_actions": [],
      "max_concurrent_files": 3,
      "allow_network": false
    },
    "created_at": "2026-05-03T14:30:00Z",
    "status": "pending"
  },
  "error": null
}
```

**Errors:** Service errors.

---

### core_validate_work_packet

Validate a work packet's scope boundaries and cost budget. Checks that:
- The packet exists and is in a valid state
- File paths within the allowed scope are reachable
- Cost budget values are within acceptable limits

**Required args:** `packet_id`

```json
{
  "name": "core_validate_work_packet",
  "arguments": {
    "packet_id": "pkt_001"
  }
}
```

**Expected return:**

```json
{
  "success": true,
  "data": {
    "packet_id": "pkt_001",
    "scope_valid": true,
    "allowed_actions": ["deploy.service", "config.read"],
    "restricted_paths": ["/deploy/k8s/api-gateway.yaml"],
    "cost_budget": {
      "max_tokens": 100000,
      "max_api_calls": 50,
      "max_seconds": 600
    }
  },
  "error": null
}
```

**Errors:** Service errors.

---

## Quick Reference: Tool Summary

| # | Tool Name | Required Args | Category |
|---|-----------|---------------|----------|
| 1 | `core_issue_session` | actor_id, tenant_id, project_id, ttl_seconds | Session |
| 2 | `core_renew_session` | session_id, ttl_seconds | Session |
| 3 | `core_revoke_session` | session_id, reason | Session |
| 4 | `core_propose_command` | scope, action, target_type, target_id, payload | Command |
| 5 | `core_authorize_command_action` | command_id, scope | Command |
| 6 | `core_apply_command` | command_id, scope | Command |
| 7 | `core_query_events` | project_id (+ optional filters) | Audit |
| 8 | `core_verify_chain` | project_id | Audit |
| 9 | `core_create_snapshot` | tenant_id, project_id, reason | Snapshot |
| 10 | `core_validate_snapshot` | snapshot_id | Snapshot |
| 11 | `core_compare_snapshots` | from_id, to_id | Snapshot |
| 12 | `core_preview_rollback` | from_snapshot_id, to_snapshot_id | Rollback |
| 13 | `core_execute_rollback` | from_snapshot_id, to_snapshot_id, tenant_id, project_id | Rollback |
| 14 | `core_verify_rollback` | snapshot_id | Rollback |
| 15 | `core_get_actor_scope` | actor_id, session_id | Actor |
| 16 | `core_register_skill` | skill_id, tenant_id, name, allowed_actions | Skill |
| 17 | `core_activate_skill_session` | skill_id, tenant_id, project_id, ttl_seconds | Skill |
| 18 | `core_generate_work_packet` | tenant_id, project_id, actor_id, work_path_node_id, objective, allowed_file_paths | Work Packet |
| 19 | `core_validate_work_packet` | packet_id | Work Packet |

---

## Quick Start: End-to-End MCP Flow

Below is a sequence of tool calls that demonstrates a complete session → command → audit workflow:

```json
// Step 1: Issue a session
{
  "name": "core_issue_session",
  "arguments": {
    "actor_id": "alice",
    "tenant_id": "acme",
    "project_id": "proj",
    "ttl_seconds": 3600
  }
}

// Step 2: Propose a command
{
  "name": "core_propose_command",
  "arguments": {
    "scope": {
      "tenant_id": "acme",
      "project_id": "proj",
      "actor_id": "alice",
      "roles": ["admin"],
      "session_id": "ses_abc123",
      "allowed_actions": ["*"],
      "approval_state": "not_required",
      "execution_mode": "approved",
      "context_updated_at": "2026-05-03T14:30:00Z",
      "expires_at": "2026-05-03T15:30:00Z"
    },
    "action": "deploy.service",
    "target_type": "service",
    "target_id": "gw",
    "payload": { "version": "2.1.0" }
  }
}

// Step 3: Authorize the command
{
  "name": "core_authorize_command_action",
  "arguments": {
    "command_id": "cmd_xyz789",
    "scope": { /* same scope as step 2 */ }
  }
}

// Step 4: Apply the command
{
  "name": "core_apply_command",
  "arguments": {
    "command_id": "cmd_xyz789",
    "scope": { /* same scope as step 2 */ }
  }
}

// Step 5: Verify the audit chain
{
  "name": "core_verify_chain",
  "arguments": { "project_id": "proj" }
}
```
