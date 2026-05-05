---
doc_id: DOC-PLAN-P2A
title: Phase 2a — Foundation Hardening
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
related_file_ids: [FILE-CRATE-DOMAIN-LIB, FILE-CRATE-DOMAIN-IDS, FILE-CRATE-DOMAIN-SCOPE, FILE-CRATE-POLICY-LIB, FILE-CRATE-AUDIT-LIB, FILE-CRATE-SNAPSHOT-LIB]
visual_node_ids: [VN-CRATE-DOMAIN, VN-CRATE-POLICY, VN-CRATE-AUDIT, VN-CRATE-SNAPSHOT, VN-CRATE-STORE]
approval_state: pending
---

# Phase 2a: Foundation Hardening

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 2a |
| Title | Foundation Hardening |
| Work paths | `WP-CORE-001` |
| Product module | Authority Core |
| Owner | cto (Dr. Rena Okafor) |
| Risk | critical |
| Decision blockers | none |

**Mandate:** Harden the skeleton crates — domain types, policy engine, audit log, snapshot ledger. Enrich existing types, add error modules, implement state machines, add validation. No service orchestration yet; that comes in Phase 2b.

---

## Workflow

```
Crate dependency order:
  1. authority-domain (pure types — no crate deps)
  2. audit-log (depends on authority-domain)
  3. policy-engine (depends on authority-domain)
  4. snapshot-ledger (depends on authority-domain)
  5. control-store (depends on authority-domain, audit-log, snapshot-ledger)

Implementation order per crate:
  a. Error types first
  b. Enum/type enrichment next
  c. Validation and state machines next
  d. Store adapter queries last
```

---

## File Manifest

### authority-domain

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `authority-domain/src/error.rs` | 60 | Unified domain error type: `DomainError` enum with `ValidationError`, `StateTransitionError`, `NotFound`, `Conflict` variants. Implements `Display`, `Error`. |
| EXTEND | `authority-domain/src/ids.rs` | 80 | Add `Display` for all 12 typed IDs. Add `FromStr`, `Serialize`, `Deserialize` implementations. Test round-trip serialization. |
| EXTEND | `authority-domain/src/scope.rs` | 90 | Add `ActorScope::validate()` — checks session expiry, execution mode, tenant/project match. Add `ActorScope::has_action(action)` — checks if action is in allowed actions. Add scope factory methods. |
| EXTEND | `authority-domain/src/state.rs` | 60 | Add `TryFrom<&str>` and `TryFrom<String>` implementations between enum variants and DB string values. Add `as_db_string()` for serialization. |
| EXTEND | `authority-domain/src/command.rs` | 100 | Add `CommandStatus::can_transition_to(target)` state machine. Enforce: Proposed → {Authorized, Denied}, Authorized → {Applied, Failed}, {Applied, Denied, Failed} → terminal. |
| EXTEND | `authority-domain/src/skill.rs` | 80 | Add `SkillRegistration::validate_integrity()`. Add `SkillSession::is_active()`. Add approval lifecycle transitions. |
| EXTEND | `authority-domain/src/skill_creator.rs` | 80 | Add catalog versioning — each mutable operation increments version. Add `SkillCreator::register_skill()`, `SkillCreator::update_skill_grants()`. |
| UPDATE | `authority-domain/src/lib.rs` | 10 | Export `error` module. Re-export all public types. |

### audit-log

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `audit-log/src/lib.rs` | 80 | Add `EventType` categorization enum (SessionEvent, CommandEvent, PolicyEvent, SnapshotEvent, RollbackEvent). Add `EventError` variants. Add `AuditEvent::categorize()` method. Add `compute_hash()` and `verify_hash()` integrity methods. |

### policy-engine

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `policy-engine/src/lib.rs` | 100 | Add `DenialCode::RateLimited`. Add `PolicyChainEvaluator` — runs multiple policy checks in sequence, collects all denials (not short-circuit). Add `authorize_action()` with 6 checks: expired, stale, unauthorized, wrong skill, proposal only, approval required. |

### snapshot-ledger

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `snapshot-ledger/src/lib.rs` | 10 | Re-export all public modules. |
| EXTEND | `snapshot-ledger/src/manifest.rs` | 80 | Add `SnapshotManifest::validate()` — verify hash, verify all object refs exist. Add `SnapshotDelta` struct for comparing two manifests (added, removed, changed objects). |
| EXTEND | `snapshot-ledger/src/object_store.rs` | 60 | Add `FileObjectStore::delete(ref)`, `FileObjectStore::list(prefix)`, `FileObjectStore::stat(ref)` operations. |

### control-store

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-store/src/lib.rs` | 60 | Add `append_audit_event()` with hash-chain linking. Add `insert_snapshot_manifest()`. Add `get_snapshot_manifest()`. Add session queries and scope queries. |

---

## Rust Types Reference

### DomainError (NEW)
```rust
#[derive(Debug, Clone)]
pub enum DomainError {
    ValidationError { field: String, reason: String },
    StateTransitionError { from: String, to: String, reason: String },
    NotFound { entity_type: &'static str, id: String },
    Conflict { entity_type: &'static str, id: String, reason: String },
}
impl std::fmt::Display for DomainError { ... }
impl std::error::Error for DomainError { ... }
```

### DenialCode (EXTEND)
```rust
pub enum DenialCode {
    SessionExpired,
    GrantMissing,
    GrantExpired,
    GrantRevoked,
    ActionDenied,
    WrongSkill,
    ProposalOnly,
    ApprovalRequired,
    RateLimited,             // NEW
    SeparationOfDutiesDenied,
}
```

### EventType (NEW in audit-log)
```rust
pub enum EventType {
    SessionEvent,
    CommandEvent,
    PolicyEvent,
    SnapshotEvent,
    RollbackEvent,
}
```

---

## Completion Gates

- [x] `TEST-DOMAIN-001` — Domain error types serialize/deserialize correctly
- [x] `TEST-DOMAIN-IDS-001` — All typed IDs round-trip through Display/FromStr
- [x] `TEST-POLICY-001` — Policy chain evaluator returns all denials, not just the first
- [x] `TEST-AUDIT-CHAIN-001` — Tampered audit chain fails verification
- [x] `TEST-SNAPSHOT-001` — Snapshot manifest validates hash and object refs
- [x] `TEST-CORE-AUTHORITY-001` — Actor scope validates session expiry correctly
- [x] All source files under 300 lines (500 hard cap)
- [x] No `unwrap()` without `INVARIANT` comment. No `unsafe` without `SAFETY:` comment
- [x] `cargo test --workspace` passes with 0 failures
- [x] `cargo clippy --workspace -- -D warnings` passes

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-DOMAIN` | Modify authority-domain source files |
| `SGL-BACKEND-POLICY` | Modify policy-engine source files |
| `SGL-BACKEND-AUDIT` | Modify audit-log source files |
| `SGL-BACKEND-SNAPSHOT` | Modify snapshot-ledger source files |

---

## Dependencies

- Phase 1 complete (workspace compiles, Postgres accessible)
