---
doc_id: DOC-PLAN-P4
title: Phase 4 — Agent Gateway And Skill Grants
parent: DOC-PLAN-INDEX
status: certified
owner: security-architect
reviewers: [cto, backend, api-architect, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-SKILL-001, WP-GATEWAY-001]
related_decision_ids: []
related_file_ids: [FILE-CRATE-GATEWAY-LIB, FILE-CRATE-DOMAIN-GRANT, FILE-CRATE-POLICY-SOD, FILE-CRATE-API-LIB, FILE-CRATE-CLI-MAIN, FILE-CRATE-MCP-LIB]
visual_node_ids: [VN-MODULE-SKILL-GRANTS, VN-MODULE-AGENT-GATEWAY, VN-CRATE-GATEWAY]
approval_state: pending
---

# Phase 4: Agent Gateway And Skill Grants

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 4 |
| Title | Agent Gateway And Skill Grants |
| Work paths | `WP-SKILL-001`, `WP-GATEWAY-001` |
| Product module | Skill Grants, Agent Gateway |
| Owner | security-architect (Fatima Al-Hassan) |
| Risk | critical |
| Decision blockers | none |

**Mandate:** Build the zero-capability skill grant system and the agent execution gateway. The gateway is the **only** path for agent-visible mutation. Every write goes through: authenticate → load grant → load packet → validate state → authorize command → verify scope → create audit → execute → require evidence → anchor snapshot.

---

## Workflow

```
CREATE NEW CRATE: agent-gateway (the enforcement layer)

Crate dependency order:
  1. agent-gateway — new crate with gateway enforcement logic
  2. authority-domain — add SkillGrant, SeparationOfDuties types
  3. policy-engine — add SOD validation, grant validation
  4. control-store — add grant and gateway persistence
  5. control-service — add gateway_service module (wraps gateway)
  6. control-api — add gateway route group
  7. operator-cli — add gateway commands
  8. agent-mcp — add gateway tools

Implementation order:
  a. Gateway denial codes and error types
  b. Gateway flow engine (authenticate → authorize → execute cycle)
  c. Skill grant domain types and persistence
  d. Separation of duties validation
  e. File scope validation
  f. Thin transports
```

---

## File Manifest

### agent-gateway (NEW CRATE)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `agent-gateway/Cargo.toml` | 20 | Dependencies: authority-domain, policy-engine, audit-log, control-store, snapshot-ledger, chrono, serde, thiserror, tracing. |
| NEW | `agent-gateway/src/lib.rs` | 40 | Module declarations, `AgentGateway` struct holding references to store and policy engine. `GatewayConfig` — timeout, max packet size, allowed transports. |
| NEW | `agent-gateway/src/error.rs` | 60 | `GatewayError` enum with all denial codes mapped. Implements `Display`, `Error`. Maps to RFC 7807 responses. |
| NEW | `agent-gateway/src/flow.rs` | 180 | **The heart of the gateway.** `execute_gateway_flow(request)` — runs the full 9-step pipeline: authenticate → load grant → load packet → validate packet state → authorize command → verify file scope → create audit event → execute operation → require evidence → anchor snapshot. Returns `GatewayResult`. |
| NEW | `agent-gateway/src/scope_validator.rs` | 100 | `validate_file_scope(file_path, grant)` — deny list wins, rejects absolute paths, rejects path traversal, checks artifact class. `validate_schema_scope(view_name, grant)` — only views allowed, not base tables. |

### authority-domain

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `authority-domain/src/skill.rs` | 80 | Add `SkillGrant` struct — all fields from spec 06 (allow/deny lists, budget, expiry, separation group). `GrantState` — Active, Expired, Revoked, Suspended. |
| NEW | `authority-domain/src/separation.rs` | 80 | `SeparationOfDuties` struct with `conflicting_groups` pairs. `validate(actor_id, grant_id, action, current_grants)` — checks no conflicting duty is held by same actor. |

### policy-engine

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `policy-engine/src/lib.rs` | 80 | Add `validate_grant_active(grant)` — checks state and expiry. Add `validate_grant_action(grant, action)` — checks allowed_actions list. Add `validate_sod(actor, action, grants)` — invokes separation module. Add `GrantDenial` variant to `PolicyDenial`. |

### control-store

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-store/src/lib.rs` | 80 | `insert_skill_grant()`, `get_active_grant(agent_id)`, `list_grants(agent_id)`, `revoke_grant(grant_id)`. `insert_gateway_request()`, `get_gateway_log()`. |

### control-service

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-service/src/gateway_service.rs` | 80 | `execute_command(request)` — wraps `AgentGateway::execute_gateway_flow()`. `get_gateway_status()` — health of gateway pipeline. Thin wrapper — the heavy logic lives in agent-gateway crate. |
| UPDATE | `control-service/src/lib.rs` | 10 | Declare `gateway_service` module. |

### control-api

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-api/src/routes/gateway.rs` | 60 | `POST /v1/gateway/execute` — single endpoint for all agent command execution. |
| EXTEND | `control-api/src/routes/mod.rs` | 5 | Register gateway route group. |

### operator-cli

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `operator-cli/src/commands/grant.rs` | 80 | `grant create`, `grant list`, `grant revoke`, `grant inspect`. |
| EXTEND | `operator-cli/src/commands/mod.rs` | 5 | Register grant command module. |

### agent-mcp

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `agent-mcp/src/tools/gateway.rs` | 60 | `core_execute_command` — single tool that routes all agent command execution through the gateway. |
| EXTEND | `agent-mcp/src/tools/mod.rs` | 5 | Register gateway tool module. |

---

## Gateway Denial Codes

All denial codes from spec 07, mapped to `GatewayError`:

| Denial Code | HTTP Status | Description |
|-------------|-------------|-------------|
| `GRANT_MISSING` | 403 | Agent has no active grant |
| `GRANT_EXPIRED` | 403 | Grant has expired |
| `GRANT_REVOKED` | 403 | Grant was revoked |
| `PACKET_MISSING` | 400 | No work packet found for request |
| `PACKET_NOT_ASSIGNED` | 403 | Packet not assigned to this agent |
| `PACKET_SCOPE_DENIED` | 403 | Action outside packet scope |
| `ACTION_DENIED` | 403 | Action not in grant's allowed_actions |
| `FILE_SCOPE_DENIED` | 403 | File not in allowed or in denied list |
| `SCHEMA_SCOPE_DENIED` | 403 | Schema access denied by grant |
| `BUDGET_EXCEEDED` | 429 | Token or time budget exceeded |
| `EVIDENCE_REQUIRED` | 400 | Evidence record required before execute |
| `APPROVAL_REQUIRED` | 403 | Human approval required for this action |
| `SEPARATION_OF_DUTIES_DENIED` | 403 | SOD conflict with existing grants |
| `SNAPSHOT_ANCHOR_REQUIRED` | 400 | No rollback anchor exists |

---

## Completion Gates

- [x] `TEST-SKILL-GRANT-001` — Agent without active grant has zero capabilities
- [x] `TEST-SOD-001` — Implementer cannot approve own high-risk packet
- [x] `TEST-GATEWAY-001` — Gateway denies file outside packet scope
- [x] All 14 denial codes produce correct `GatewayError` variants
- [x] Gateway pipeline runs all 9 steps and produces audit + evidence + anchor
- [x] File scope validator rejects absolute paths and path traversal
- [x] API, CLI, and MCP surfaces all expose gateway execution
- [x] `cargo test --workspace` passes with 0 failures
- [x] `cargo clippy --workspace -- -D warnings` passes

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-GATEWAY` | Create and modify agent-gateway crate |
| `SGL-BACKEND-POLICY` | Extend policy-engine with grant and SOD validation |
| `SGL-BACKEND-DOMAIN` | Extend authority-domain with grant and separation types |
| `SGL-SECURITY-REVIEW` | Review gateway threat model and denial code coverage |
| `SGL-DATA-POSTGRES` | Add grant and gateway persistence to control-store |

---

## Dependencies

- Phase 3 complete (Catalogs and Work Paths — work packet types exist for gateway to validate against)
