---
doc_id: DOC-PLAN-P9
title: Phase 9 — Login Vertical
parent: DOC-PLAN-INDEX
status: draft
owner: pm
reviewers: [ceo, cto, security-architect, frontend, backend, qa, release-manager, biz-user]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-LOGIN-001]
related_decision_ids: [DEC-USER-004, DEC-USER-005]
related_file_ids: [FILE-CATALOG-LOGIN-MODULE, FILE-CONTRACT-LOGIN, FILE-CONTRACT-LOGIN-RESPONSE, FILE-POLICY-LOGIN, FILE-HANDLER-LOGIN, FILE-WATCH-LOGIN]
visual_node_ids: [VN-MODULE-LOGIN]
approval_state: pending
---

# Phase 9: Login Vertical

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 9 |
| Title | Login Vertical |
| Work paths | `WP-LOGIN-001` |
| Product module | Login Module |
| Owner | pm (Alex Rivera) |
| Risk | high |
| Decision blockers | none; `DEC-USER-004` and `DEC-USER-005` are closed |

**Mandate:** Build the first complete vertical slice of RealmForge — a Login module that demonstrates the entire governance loop from catalog to rollback. This is the **integration proof** that every layer works end-to-end.

**Decision resolution:** `DEC-USER-005` requires clean backend names without a RealmForge brand prefix. `DEC-USER-004` keeps the roadmap order after Login: catalog infrastructure first, then module content defined by implementing phases.

---

## Workflow

```
Login vertical implements the full work path flow:

Catalog        → Define Login module in catalog
Contracts      → Define Login request/response contracts
Policy         → Define Login authorization policy
Service        → Implement Login handler binding
Watch          → Define Login watch profile
                   ↓
Work Path      → Traverse Login work path → generate scoped work packet
Gateway        → Execute packet through gateway with grants
Audit          → Every action has audit trail
Snapshot       → Create snapshot anchor before/after mutations
Rollback       → Demonstrate rollback to pre-Login state

No new crate is needed for Login. It uses existing crates:
  - authority-domain types for Login-specific domain logic
  - control-service handler for Login execution
  - control-api routes for Login endpoints
  - control-store for Login state persistence
  - live-watch profile for Login monitoring
```

---

## File Manifest

### authority-domain

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `authority-domain/src/login.rs` | 80 | `LoginRequest` — standard request type with credentials, scope. `LoginResponse` — session token, scope, expiry. `LoginState` — LoginAttempt, LoginBlock, CredentialStore (typed wrappers). |
| NEW | `authority-domain/src/login_policy.rs` | 60 | `LoginPolicy` — rate limit rules, credential validation rules, block rules. `LoginPolicyDecision` — Approved, RateLimited, Blocked, InvalidCredentials. |
| UPDATE | `authority-domain/src/lib.rs` | 5 | Export `login` and `login_policy` modules. |

### catalog

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `catalog/Login/catalog.json` | 20 | Login module catalog entry — module type, files, contracts, tests, traces. |
| NEW | `catalog/Login/contracts/login_request.json` | 30 | LoginRequest contract schema — fields, types, validation rules. |
| NEW | `catalog/Login/contracts/login_response.json` | 30 | LoginResponse contract schema — session token, scope, expiry. |
| NEW | `catalog/Login/policy/login_policy.json` | 40 | Login policy definition — allowed actions, rate limits, approval rules. |

### control-store

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-store/src/lib.rs` | 60 | `insert_login_attempt()`, `record_login_block()`, `get_login_attempts(actor, window)`. `insert_login_policy()`, `get_login_policy(tenant_id)`. |

### control-service

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-service/src/login_handler.rs` | 120 | `handle_login(request)` — validate credentials against policy, issue session via session_service, record audit event, create snapshot anchor, return response with scope. Full governance loop: authenticate → authorize → audit → snapshot. |
| UPDATE | `control-service/src/lib.rs` | 5 | Declare `login_handler` module. |

### control-api

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-api/src/routes/login.rs` | 60 | `POST /v1/login` — single endpoint. Validates request, delegates to login_handler, returns session response. |
| EXTEND | `control-api/src/routes/mod.rs` | 5 | Register login route group. |

### operator-cli

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `operator-cli/src/commands/login.rs` | 60 | `login test` — simulate login attempt for testing. `login policy set` — configure login policy. `login blocks list` — list blocked actors. |
| EXTEND | `operator-cli/src/commands/mod.rs` | 5 | Register login command module. |

### agent-mcp

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `agent-mcp/src/tools/login.rs` | 40 | `core_login` — authenticate and return session. |
| EXTEND | `agent-mcp/src/tools/mod.rs` | 5 | Register login tool module. |

### live-watch profile

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `catalog/Login/watch_profile.json` | 30 | Login watch profile — signals: login failure rate, latency, block events. Thresholds and severity mappings. |

### Integration test

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `tests/e2e/login_vertical.rs` | 150 | **The crown jewel test.** Full end-to-end: catalog entry exists → traverse work path → generate packet → gateway execute → audit verify → create snapshot → rollback → verify restored state. Proves every layer works together. |

---

## Login Vertical Flow

```text
1. Catalog defines Login module
   ├── Contracts: LoginRequest, LoginResponse
   ├── Policy: LoginPolicy (rate limits, validation rules)
   ├── Handler: login_handler.rs
   └── Watch profile: login failure rate, latency

2. Work path traversal
   → Walks Login work path node
   → Generates scoped AgentWorkPacket
   → Packet includes: allowed files, required contracts, required tests, rollback anchor

3. Gateway execution
   → Agent requests packet via gateway
   → Gateway: authenticate → load grant → load packet → validate → authorize → execute
   → Every mutation produces audit event + evidence + snapshot anchor

4. Login lifecycle
   → POST /v1/login with credentials
   → Validate against LoginPolicy
   → Issue session via session_service
   → Record audit event
   → Create snapshot anchor
   → Return LoginResponse with session token + scope

5. Audit verification
   → Query audit trail for login attempt
   → Verify hash chain integrity
   → Confirm all events are linked

6. Snapshot and rollback
   → Create snapshot of system state
   → Rollback to pre-login snapshot
   → Verify login state is restored
   → Verify audit trail still intact
```

---

## Completion Gates

- [ ] `TEST-LOGIN-E2E-001` — Login vertical completes catalog-to-rollback loop
- [x] Catalog entry exists for Login module with all required metadata
- [ ] Work path traversal correctly generates Login packet
- [ ] Gateway executes Login packet with grants + audit + evidence + anchor
- [x] Login endpoint returns valid session token with correct scope
- [x] Login policy enforces rate limits
- [ ] Audit trail records all login events with hash chain integrity
- [ ] Snapshot captures login state correctly
- [ ] Rollback restores system to pre-login state
- [x] Login handler creates a snapshot anchor before session issuance
- [x] Watch profile collects login failure rate signals
- [x] `cargo test --workspace` passes with 0 failures (verified 2026-05-05 with `--target-dir target-quality`)
- [x] `cargo clippy --workspace -- -D warnings` passes (verified 2026-05-05 with `--all-targets --target-dir target-quality`)

### Current Verification Note (2026-05-05)

Development is complete for the Phase 9 implementation surface: Login catalog files, request/response contracts, policy, watch profile, domain types, persistence, handler, API, CLI, MCP, and integration tests all exist. The handler now returns a persisted snapshot anchor in addition to the session and audit event. Phase QA remains partial because the full catalog-to-rollback E2E loop, gateway packet execution, full login-event audit chain proof, and rollback restoration proof are not yet certified.

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-DOMAIN` | Add login types and login_policy to authority-domain |
| `SGL-BACKEND-SERVICE` | Add login_handler to control-service |
| `SGL-BACKEND-API` | Add login route group |
| `SGL-BACKEND-CLI` | Add login commands |
| `SGL-DATA-POSTGRES` | Add login persistence to control-store |
| `SGL-QA-VERIFY` | Certify end-to-end test passes |
| `SGL-RELEASE` | Review login release readiness |
| `SGL-SECURITY-REVIEW` | Review login policy and credential handling |

---

## Dependencies

- Phase 3 complete (Catalogs — Login must be a catalog entry)
- Phase 4 complete (Gateway — Login executes through gateway)
- Phase 7 complete (Live Runtime — Login runs in governed runtime context)
- `DEC-USER-005` resolved (clean backend names without brand prefix)
- `DEC-USER-004` resolved (post-Login catalog work follows the roadmap order)
