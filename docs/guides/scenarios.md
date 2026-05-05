---
doc_id: DOC-GUIDE-SCENARIOS
title: "RealmForge Core — End-to-End Scenarios"
status: active
owner: tech-writer
reviewers: [tech-writer, backend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: false
product_area: workspace
work_path_ids: [WP-CORE-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---
# RealmForge Core — End-to-End Scenarios

This guide provides realistic, multi-step scenarios demonstrating how RealmForge Core's governance kernel operates in practice. Each scenario walks through a complete workflow using the HTTP API (curl) and MCP tool invocations side by side.

---

## Table of Contents

- [Scenario 1: Session → Command → Audit](#scenario-1-session--command--audit)
- [Scenario 2: Snapshot → Rollback → Verify](#scenario-2-snapshot--rollback--verify)
- [Scenario 3: Skill Registration → Work Packet → Validation](#scenario-3-skill-registration--work-packet--validation)
- [Scenario 4: Policy Denial — Unauthorized Actor](#scenario-4-policy-denial--unauthorized-actor)
- [Scenario 5: Session Lifecycle — Issue, Renew, Revoke](#scenario-5-session-lifecycle--issue-renew-revoke)

---

## Scenario 1: Session → Command → Audit

**Goal:** Deploy version 2.1.0 of the API gateway service using the full governance lifecycle — session, propose, authorize, apply, and audit.

### Step 1: Issue a Session

**HTTP API:**

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

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "core_issue_session",
    "arguments": {
      "actor_id": "alice",
      "tenant_id": "acme-corp",
      "project_id": "project-alpha",
      "ttl_seconds": 3600
    }
  }
}
```

**Result:** A new session `ses_abc123` is created, active for 1 hour.

---

### Step 2: Propose a Command

**HTTP API:**

```bash
SESSION_ID="ses_abc123"
curl -X POST http://localhost:8080/v1/commands \
  -H "Content-Type: application/json" \
  -d "{
    \"action\": \"deploy.service\",
    \"target_type\": \"service\",
    \"target_id\": \"api-gateway\",
    \"payload\": {
      \"version\": \"2.1.0\",
      \"strategy\": \"rolling\"
    }
  }"
```

**MCP (with full ActorScope):**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
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
}
```

**Result:** A command `cmd_xyz789` is created with status `proposed`. An audit event `command.proposed` is recorded.

---

### Step 3: Authorize the Command

**HTTP API:**

```bash
curl -X PUT http://localhost:8080/v1/commands/cmd_xyz789/authorize \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "actor_id": "alice",
    "roles": ["admin"],
    "session_id": "ses_abc123",
    "allowed_actions": ["*"],
    "approval_state": "NotRequired",
    "execution_mode": "Approved",
    "context_updated_at": "2026-05-03T14:30:00Z",
    "expires_at": "2026-05-03T15:30:00Z"
  }'
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
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
}
```

**Result:** Command transitions to `authorized`. The policy engine validates scope (session active, allowed actions contain `*`, execution mode is `Approved`). An audit event `command.authorized` is recorded.

---

### Step 4: Apply the Command

**HTTP API:**

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

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
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
}
```

**Result:** Command transitions to `applied`. An audit event `command.applied` is recorded with a hash linked to the previous event in the chain.

---

### Step 5: Verify the Audit Chain

**HTTP API:**

```bash
curl http://localhost:8080/v1/audit/chain/verify
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "core_verify_chain",
    "arguments": {
      "project_id": "project-alpha"
    }
  }
}
```

**Result:** The hash chain integrity is verified — all 3 events (proposed → authorized → applied) form a contiguous, tamper-evident chain.

---

## Scenario 2: Snapshot → Rollback → Verify

**Goal:** Create a snapshot before deployment, deploy, create another snapshot, then roll back to the pre-deployment state after discovering an issue.

### Step 1: Create a Pre-Deployment Snapshot

**HTTP API:**

```bash
curl -X POST http://localhost:8080/v1/snapshots \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "reason": "pre-deploy backup",
    "parent_snapshot_id": null
  }'
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "core_create_snapshot",
    "arguments": {
      "tenant_id": "acme-corp",
      "project_id": "project-alpha",
      "reason": "pre-deploy backup",
      "parent_snapshot_id": null
    }
  }
}
```

**Result:** Snapshot `snap_001` is created with status `draft`. It captures the current state of all tracked objects.

---

### Step 2: Validate the Snapshot

**HTTP API:**

```bash
curl -X POST http://localhost:8080/v1/snapshots/snap_001/validate
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "core_validate_snapshot",
    "arguments": {
      "snapshot_id": "snap_001"
    }
  }
}
```

**Result:** Validation confirms manifest hash is correct and all object references exist.

---

### Step 3: Perform Deployment (via Commands)

Execute the deployment using the command lifecycle shown in Scenario 1. After deployment, the system state has changed (new objects added, configuration updated).

### Step 4: Create a Post-Deployment Snapshot

```bash
curl -X POST http://localhost:8080/v1/snapshots \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "reason": "post-deploy state capture",
    "parent_snapshot_id": "snap_001"
  }'
```

**Result:** Snapshot `snap_002` is created with `parent_snapshot_id` pointing to `snap_001`.

---

### Step 5: Compare Snapshots

**HTTP API:**

```bash
curl "http://localhost:8080/v1/snapshots/compare?from_id=snap_001&to_id=snap_002"
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "core_compare_snapshots",
    "arguments": {
      "from_id": "snap_001",
      "to_id": "snap_002"
    }
  }
}
```

**Result:**
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

### Step 6: Preview the Rollback

**HTTP API:**

```bash
curl -X POST http://localhost:8080/v1/rollback/preview \
  -H "Content-Type: application/json" \
  -d '{
    "from_snapshot_id": "snap_002",
    "to_snapshot_id": "snap_001"
  }'
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "core_preview_rollback",
    "arguments": {
      "from_snapshot_id": "snap_002",
      "to_snapshot_id": "snap_001"
    }
  }
}
```

**Result:** Preview confirms 1 object will be restored: the API gateway service reverts to v2.0.0.

---

### Step 7: Execute the Rollback

**HTTP API:**

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

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "core_execute_rollback",
    "arguments": {
      "from_snapshot_id": "snap_002",
      "to_snapshot_id": "snap_001",
      "tenant_id": "acme-corp",
      "project_id": "project-alpha"
    }
  }
}
```

**Result:** Rollback executes successfully. A new snapshot `snap_003` is created recording the rolled-back state. The API gateway service has been restored to v2.0.0.

---

### Step 8: Verify the Rollback

**HTTP API:**

```bash
curl http://localhost:8080/v1/rollback/snap_003/verify
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "core_verify_rollback",
    "arguments": {
      "snapshot_id": "snap_003"
    }
  }
}
```

**Result:** Rollback snapshot `snap_003` is verified — chain_consistent: true, integrity_ok: true.

---

## Scenario 3: Skill Registration → Work Packet → Validation

**Goal:** Register a custom skill ("Deploy Engineer"), activate a skill session, generate a scoped work packet for an AI agent, and validate the packet's boundaries.

### Step 1: Register a Skill

**HTTP API:**

```bash
curl -X POST http://localhost:8080/v1/skills/register \
  -H "Content-Type: application/json" \
  -d '{
    "skill_id": "skill.deploy_engineer",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "name": "Deploy Engineer",
    "allowed_actions": [
      "deploy.service",
      "deploy.rollback",
      "config.read",
      "config.write"
    ]
  }'
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
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
}
```

**Result:** Skill `skill.deploy_engineer` is registered with `integrity_state: "pending"` and `approved: true`.

---

### Step 2: Activate a Skill Session

**HTTP API:**

```bash
curl -X POST http://localhost:8080/v1/skills/activate \
  -H "Content-Type: application/json" \
  -d '{
    "skill_id": "skill.deploy_engineer",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "ttl_seconds": 1800
  }'
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "core_activate_skill_session",
    "arguments": {
      "skill_id": "skill.deploy_engineer",
      "tenant_id": "acme-corp",
      "project_id": "project-alpha",
      "ttl_seconds": 1800
    }
  }
}
```

**Result:** A skill session `ss_001` is created with state `active`, expiring in 30 minutes.

---

### Step 3: Generate a Work Packet

**HTTP API:**

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

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
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
}
```

**Result:** A work packet `pkt_001` is generated with:
- **Cost budget:** 100,000 tokens, 50 API calls, 600 seconds max
- **Permission scope:** deploy.service, config.read actions; 3 concurrent files; no network
- **Rollback anchor:** `None` (no snapshot created yet)
- **Status:** `pending`

---

### Step 4: Validate the Work Packet

**HTTP API:**

```bash
curl -X POST http://localhost:8080/v1/work-packets/pkt_001/validate
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "core_validate_work_packet",
    "arguments": {
      "packet_id": "pkt_001"
    }
  }
}
```

**Result:** Validation passes:
```json
{
  "packet_id": "pkt_001",
  "scope_valid": true,
  "allowed_actions": ["deploy.service", "config.read"],
  "restricted_paths": ["/deploy/k8s/api-gateway.yaml"],
  "cost_budget": {
    "max_tokens": 100000,
    "max_api_calls": 50,
    "max_seconds": 600
  }
}
```

The AI agent can now execute within these boundaries, knowing its scope is verified and enforced.

---

## Scenario 4: Policy Denial — Unauthorized Actor

**Goal:** Demonstrate what happens when an actor attempts an action they are not authorized to perform.

### Step 1: Issue a Session with Limited Actions

**HTTP API:**

```bash
curl -X POST http://localhost:8080/v1/session \
  -H "Content-Type: application/json" \
  -d '{
    "actor_id": "bob-readonly",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "ttl_seconds": 3600
  }'
```

### Step 2: Attempt to Propose a Command with Insufficient Privileges

The scope allows only `config.read` actions, but the actor tries to deploy a service.

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "core_propose_command",
    "arguments": {
      "scope": {
        "tenant_id": "acme-corp",
        "project_id": "project-alpha",
        "actor_id": "bob-readonly",
        "roles": ["viewer"],
        "session_id": "ses_readonly_001",
        "allowed_actions": ["config.read"],
        "approval_state": "not_required",
        "execution_mode": "read_only",
        "context_updated_at": "2026-05-03T14:30:00Z",
        "expires_at": "2026-05-03T15:30:00Z"
      },
      "action": "deploy.service",
      "target_type": "service",
      "target_id": "api-gateway",
      "payload": { "version": "2.1.0" }
    }
  }
}
```

**Result:** The command is proposed (scoped to the action), but when `core_authorize_command_action` is called:

```json
{
  "success": true,
  "data": {
    "id": "cmd_rejected_001",
    "status": "denied",
    "denial": {
      "code": "ActorUnauthorized",
      "message": "Actor 'bob-readonly' is not authorized to perform 'deploy.service'",
      "detail": null
    }
  },
  "error": null
}
```

**What happened:** The policy engine checked:
1. **Allowed actions:** `config.read` — does not include `deploy.service` ❌
2. **Execution mode:** `read_only` — does not allow mutation ❌

The command was denied and an audit event `command.denied` was recorded with the denial code.

---

### Step 3: Verify the Denial in the Audit Log

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "core_query_events",
    "arguments": {
      "project_id": "project-alpha",
      "event_type": "command.denied",
      "actor_id": "bob-readonly"
    }
  }
}
```

**Result:** The audit log confirms the denial with the full denial context, preserving an immutable record of the attempted action.

---

## Scenario 5: Session Lifecycle — Issue, Renew, Revoke

**Goal:** Walk through the complete session lifecycle: issue a session, use it, extend it, and finally revoke it.

### Step 1: Issue a Session

```bash
curl -X POST http://localhost:8080/v1/session \
  -H "Content-Type: application/json" \
  -d '{
    "actor_id": "charlie",
    "tenant_id": "acme-corp",
    "project_id": "project-alpha",
    "ttl_seconds": 900
  }'
```

**Result:** Session `ses_charlie_001` issued, expires in 15 minutes (900 seconds).

---

### Step 2: Get Actor Scope

**HTTP API:**

```bash
curl http://localhost:8080/v1/actors/charlie/scope
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "core_get_actor_scope",
    "arguments": {
      "actor_id": "charlie",
      "session_id": "ses_charlie_001"
    }
  }
}
```

**Result:** Returns charlie's full scope — roles, allowed actions (from their default profile), session expiry.

---

### Step 3: Renew the Session (Extend by Another 30 Minutes)

```bash
curl -X POST http://localhost:8080/v1/session/ses_charlie_001/renew \
  -H "Content-Type: application/json" \
  -d '{ "ttl_seconds": 1800 }'
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "core_renew_session",
    "arguments": {
      "session_id": "ses_charlie_001",
      "ttl_seconds": 1800
    }
  }
}
```

**Result:** Session expiry extended to 30 minutes from now. The session remains `active`.

---

### Step 4: Revoke the Session

```bash
curl -X DELETE http://localhost:8080/v1/session/ses_charlie_001
```

**MCP:**

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "core_revoke_session",
    "arguments": {
      "session_id": "ses_charlie_001",
      "reason": "End of shift — rotating credentials"
    }
  }
}
```

**Result:** Session state changes to `revoked`. Any subsequent command attempts with this session will be denied at the policy layer.

---

## Appendix: Common Denial Codes

| Denial Code | Meaning | Typical Cause |
|-------------|---------|---------------|
| `SessionExpired` | The session TTL has elapsed | Session was used after expiry |
| `ActorUnauthorized` | The action is not in `allowed_actions` | Actor lacks the skill or permission |
| `ApprovalRequired` | Command requires explicit approval | `approval_state` is `Required` but no approval ID provided |
| `ProposalOnly` | Scope only allows proposing, not applying | `execution_mode` is `Proposal` |
| `StaleContext` | The scope context is too old | `context_updated_at` exceeds max age |
| `WrongSkill` | The active skill does not match the requested action | Skill session is for a different domain |
