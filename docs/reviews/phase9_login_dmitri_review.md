---
doc_id: DOC-REVIEW-001
title: "Phase 9 Login Vertical — Backend Review (Dmitri Volkov)"
status: accepted
owner: backend
reviewers: [cto, qa]
created_at: 2026-05-05
last_reviewed_at: 2026-05-05
source_of_truth: false
product_area: login
work_path_ids: [WP-LOGIN-001]
related_decision_ids: []
related_file_ids: [FILE-CRATE-SERVICE-LIB, FILE-CRATE-STORE-LIB]
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---
# Phase 9 Login Vertical — Backend Review (Dmitri Volkov)

## Review Inputs

- `docs/plan/Phase9.md` — Implementation plan
- `docs/spec/19_FIRST_VERTICAL_LOGIN_MODULE.md` — Login module spec
- `docs/reviews/phase9_login_iris_review.md` — Business user review (Iris Park)
- `docs/reviews/phase9_login_fatima_review.md` — Security review (Fatima Al-Hassan)
- Existing crate implementations: control-store, control-service, control-api, operator-cli, agent-mcp

## Proposed File Manifest (Reviewed)

### authority-domain — Login types and policy types

| File | Assessment |
|------|------------|
| `authority-domain/src/login.rs` | ✅ CORRECT LAYER — pure domain types, no IO |
| `authority-domain/src/login_policy.rs` | ✅ CORRECT LAYER — policy decisions, pure functions |
| `authority-domain/src/lib.rs` (extend) | ✅ STANDARD — module export |

### catalog metadata

| File | Assessment |
|------|------------|
| `catalog/Login/catalog.json` | ✅ DATA — catalog entry, not code |
| `catalog/Login/contracts/login_request.json` | ✅ DATA — contract schema |
| `catalog/Login/contracts/login_response.json` | ✅ DATA — contract schema |
| `catalog/Login/policy/login_policy.json` | ✅ DATA — policy definition |

### control-store — Persistence layer

| File | Assessment |
|------|------------|
| `control-store/src/lib.rs` (extend) | ✅ CORRECT LAYER — store adapter, SQL persistence behind traits |

### control-service — Service orchestration

| File | Assessment |
|------|------------|
| `control-service/src/login_handler.rs` (NEW) | ✅ CORRECT LAYER — service logic, orchestrates session_service + audit + snapshot |
| `control-service/src/lib.rs` (extend) | ✅ STANDARD — module declaration |

### control-api — REST transport

| File | Assessment |
|------|------------|
| `control-api/src/routes/login.rs` (NEW) | ✅ CORRECT LAYER — thin route, no business logic. Delegates to login_handler. |
| `control-api/src/routes/mod.rs` (extend) | ✅ STANDARD — route registration |

### operator-cli — CLI adapter

| File | Assessment |
|------|------------|
| `operator-cli/src/commands/login.rs` (NEW) | ✅ CORRECT LAYER — CLI adapter, no business logic |
| `operator-cli/src/commands/mod.rs` (extend) | ✅ STANDARD — command registration |

### agent-mcp — MCP adapter

| File | Assessment |
|------|------------|
| `agent-mcp/src/tools/login.rs` (NEW) | ✅ CORRECT LAYER — MCP tool dispatch, no business logic |
| `agent-mcp/src/tools/mod.rs` (extend) | ✅ STANDARD — tool registration |

### live-watch profile

| File | Assessment |
|------|------------|
| `catalog/Login/watch_profile.json` | ✅ DATA — watch profile definition |

### Integration test

| File | Assessment |
|------|------------|
| `tests/e2e/login_vertical.rs` (NEW) | ✅ TEST — end-to-end integration test |

## Layer Dependency Flow (Verified)

```
control-api (POST /v1/login)
   │
   ▼  (thin dispatch, no business logic)
control-service::login_handler::handle_login()
   │
   ├──► login_policy::evaluate(credentials, context)
   │        │
   │        └──► control-store: get_login_policy, get_login_attempts
   │
   ├──► session_service::issue_session(actor_id, scope)
   │        │
   │        └──► control-store: insert_session (existing)
   │
   ├──► audit_service::record_event(event)
   │        │
   │        └──► audit-log crate (existing)
   │
   ├──► snapshot_service::create_anchor(label)
   │        │
   │        └──► snapshot-ledger crate (existing)
   │
   └──► control-store: record_login_attempt, record_login_block
```

✅ **No layer violations**: API → Service → Policy → Store. Business logic stays in service layer. Policy evaluation stays in policy layer. Persistence stays in store layer.

## Code Standards Review

### Types — Login domain types

```rust
// authority-domain/src/login.rs

/// Typed identifier for an actor attempting login
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActorId(pub Uuid);

/// Login credentials — consumed immediately after validation
#[derive(Debug, Clone)]
pub struct LoginCredentials {
    pub actor_id: ActorId,
    pub credential: Vec<u8>,  // NOT String — credential bytes, discarded after validation
    pub scope: Scope,
}

/// Typed scope — validated against allowlist
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope(Arc<str>);

/// Login attempt outcome
#[derive(Debug, Clone, PartialEq)]
pub enum LoginAttemptOutcome {
    Success { actor_id: ActorId, scope: Scope },
    RateLimited { actor_id: ActorId, retry_after_secs: u64 },
    Blocked { actor_id: ActorId, blocked_until: Timestamp },
    InvalidCredentials { actor_id: Option<ActorId> },
}

/// Session token response
pub struct SessionToken(String);  // signed token — validation happens at gateway layer
```

### Errors — Typed errors with thiserror

```rust
#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("rate limited — retry after {0} seconds")]
    RateLimited(u64),
    #[error("account temporarily blocked — retry after {0}")]
    Blocked(Timestamp),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("store failure: {0}")]
    Store(#[from] StoreError),
    #[error("internal error")]
    Internal(String),  // no details leaked to caller
}
```

✅ `InvalidCredentials` carries no details — generic error for Fatima's SR-1.4 requirement.

### Security Requirements Mapping

| Security Req | Implementation |
|-------------|----------------|
| SR-1.1: Validate fields before policy check | Route layer validates field lengths; handler validates structure before policy call |
| SR-1.2: Never log credential values | `LoginCredentials.credential` is `Vec<u8>` — opaque type, no `Display` or `Debug` that prints value |
| SR-1.3: Request body size limit | 64KB max at control-api route layer |
| SR-1.4: Generic errors | `LoginError::InvalidCredentials` — no distinction between "wrong password" and "unknown user" |
| SR-2.1: Constant-time validation | Use `subtle::ConstantTimeEq` for credential comparison |
| SR-2.2: Secure comparison function | `credential.ct_eq(&stored_value)` |
| SR-2.3: Discard credentials after validation | `LoginCredentials` dropped at end of `handle_login()` scope; `Vec<u8> is zeroed on drop` |
| SR-3.1: Rate limit before credential validation | Rate limit check as first operation after input validation |
| SR-3.2: Indistinguishable rate-limit response | RateLimited error uses generic message — no account enumeration |
| SR-4.1: Signed session token | Token signed by authority-domain key via existing session_service |
| SR-4.2: Scope in signed payload | Scope encoded inside signed token, validated on every request |
| SR-4.3: Explicit token expiry | session_service handles expiry |
| SR-4.4: Token not in logs/audit | Audit event records event_id, not token value |
| SR-5.1: Audit before response | handler calls audit_service::record_event before returning response |
| SR-5.2: Audit event fields | actor_id, timestamp, outcome, audit_event_id |
| SR-5.3: No credentials in audit | Audit event carries outcome, not credential data |
| SR-6.1: Field length limits | Max lengths: actor_id UUID format, scope max 1024 chars, credential max 2048 bytes |
| SR-6.2: Scope allowlist | Scope validated against known patterns at route layer |
| SR-6.3: Cache-Control header | Added to response headers by control-api route |

## Dmitri's Verdict

**Overall assessment: ✅ IMPLEMENTATION READY**

The architecture is clean. No layer violations. The types carry the domain model correctly. The security requirements from Fatima are all implementable within the existing crate boundaries.

**Two technical concerns:**

1. **`credential: Vec<u8>`** — `Vec<u8>` does not zero memory on drop by default. I need to use `zeroize::Zeroizing<Vec<u8>>` or a custom wrapper that implements `Drop` to zero memory. This is a Rust-specific requirement for CR-2.3 (credential discard after validation).

2. **Rate limit granularity** — Fatima's SR-3 recommends rate limiting per-actor AND per-IP. Phase 9 should implement per-actor rate limiting (simpler, uses existing identity from LoginCredentials). Per-IP rate limiting requires connection metadata passthrough and can be deferred to a follow-up.

**Simplified contract fields per Iris/Fatima alignment:**

Phase 9 LoginRequest:
```json
{
    "actor_id": "uuid",
    "credential": "base64-encoded-bytes",
    "scope": "string (allowlist-matched)"
}
```

Phase 9 LoginResponse:
```json
{
    "session_token": "signed-jwt-string",
    "actor_id": "uuid",
    "scope": "string",
    "expires_at": "ISO 8601 timestamp",
    "audit_event_id": "uuid"
}
```

**Handoff from Dmitri Volkov:** Architecture review complete. Approving the plan with the two technical notes above. Ready for implementation.
