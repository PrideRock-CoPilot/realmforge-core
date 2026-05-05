---
doc_id: DOC-PLAN-P2C
title: Phase 2c — Session & Command Lifecycle Engines
parent: DOC-PLAN-INDEX
status: draft
owner: cto
reviewers: [domain-architect, security-architect, backend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-CORE-001]
related_decision_ids: []
related_file_ids: [FILE-CRATE-DOMAIN-SCOPE, FILE-CRATE-STORE-LIB, FILE-CRATE-SERVICE-LIB]
visual_node_ids: [VN-CRATE-SERVICE, VN-CRATE-DOMAIN, VN-CRATE-STORE]
approval_state: pending
---

# Phase 2c: Session & Command Lifecycle Engines

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 2c |
| Title | Session & Command Lifecycle Engines |
| Work paths | `WP-CORE-001` |
| Product module | Authority Core |
| Owner | cto (Dr. Rena Okafor) |
| Risk | critical |
| Decision blockers | none |

**Mandate:** Fully implement the session lifecycle (issue → activate → renew → revoke → expire) and the bounded command lifecycle (propose → authorize → apply → deny → fail). This is the operational heart of the governance kernel.

---

## Workflow

```
1. Session lifecycle implementation
   a. scope.rs — Add session→scope factory methods
   b. control-store — Add session CRUD queries
   c. session_service.rs — Full lifecycle implementation

2. Command lifecycle implementation
   a. command.rs — Add CommandStatus state machine validation
   b. policy-engine — Add command-specific policy checks
   c. control-store — Add bounded command queries
   d. command_service.rs — Full lifecycle implementation

3. Test both lifecycles
   a. Session: issue → activate → renew → revoke
   b. Command: propose → authorize → apply → audit chain
   c. Denial: propose → authorize (with scope that denies) → verify denial
```

---

## File Manifest

### authority-domain

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `authority-domain/src/scope.rs` | 60 | Add `ActorScope::from_session(session, roles, grants)` factory. Add `ActorScope::execution_mode()` — maps session state to ExecutionMode (Issued→ReadOnly, Active→ReadWrite, Revoked→NoAccess, Expired→NoAccess). |

### policy-engine

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `policy-engine/src/lib.rs` | 60 | Add `authorize_command(action, target_type, scope) → PolicyDecision` — checks action against allowed_actions, checks target_type against allowed_targets, checks scope execution mode. |

### control-store

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-store/src/lib.rs` | 80 | Add `insert_session()`, `get_session()`, `update_session_state()`, `list_sessions()`. Add `insert_command()`, `get_command()`, `update_command_status()`, `list_commands()`. All with typed error variants. |

### control-service (FULL IMPLEMENTATION)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| FULL | `control-service/src/session_service.rs` | 120 | `issue_session(actor_id, tenant_id, project_id, ttl)` → creates Session with state=Issued, returns ActorScope with ExecutionMode=ReadOnly. `renew_session(session_id, new_ttl)` → extends expires_at. `revoke_session(session_id, reason)` → transitions to Revoked, emits audit event. `validate_session(session_id)` → checks expiry, returns session if active. |
| FULL | `control-service/src/command_service.rs` | 140 | `propose_command(scope, action, target, payload)` → creates BoundedCommand status=Proposed. `authorize_command(command_id, scope)` → calls policy engine, if allowed→Authorized, if denied→Denied+audit event. `apply_command(command_id)` → validates Authorized status, executes effects, sets Applied, emits audit event, creates snapshot anchor. `deny_command(command_id, denial)` → sets Denied, emits audit denial event. |

---

## Session Lifecycle State Machine

```text
                  ┌──────────┐
                  │  Issued  │
                  └────┬─────┘
                       │ activate()
                       ▼
                  ┌──────────┐
         ┌────────│  Active  │◄────────┐
         │        └────┬─────┘        │
         │             │              │
         │      expire()│   renew()───┘
         │             │
         │             ▼
         │        ┌──────────┐
         │        │ Expired  │
         │        └──────────┘
         │
         │ revoke()
         ▼
    ┌──────────┐
    │ Revoked  │
    └──────────┘
```

## Command Lifecycle State Machine

```text
                  ┌──────────┐
                  │ Proposed │
                  └────┬─────┘
                       │
                ┌──────┴──────┐
                │             │
           authorize()    authorize()
           (allowed)      (denied)
                │             │
                ▼             ▼
          ┌──────────┐  ┌──────────┐
          │Authorized│  │  Denied  │
          └────┬─────┘  └──────────┘
               │
          ┌────┴────┐
          │         │
     apply()    apply()
     (success)  (failure)
          │         │
          ▼         ▼
     ┌────────┐ ┌────────┐
     │Applied │ │ Failed │
     └────────┘ └────────┘
```

---

## Completion Gates

- [x] Session issue → activate → renew → revoke lifecycle test passes
- [x] Expired session returns `SessionExpired` denial
- [x] Scope correctly reflects execution mode (ReadOnly for Issued, ReadWrite for Active)
- [x] Full command lifecycle: propose → authorize → apply → verify audit trail
- [x] Denial test: propose → authorize (with denying scope) → verify denial response
- [x] Attempting to apply an unauthorized command returns error
- [x] `cargo test --workspace` passes with 0 failures

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-DOMAIN` | Modify authority-domain scope and command types |
| `SGL-BACKEND-POLICY` | Modify policy-engine command authorization |
| `SGL-BACKEND-SERVICE` | Implement service layer session and command logic |

---

## Dependencies

- Phase 2b complete (control-service crate exists with skeleton signatures)
