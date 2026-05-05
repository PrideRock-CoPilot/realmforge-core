---
doc_id: DOC-PLAN-P2B
title: Phase 2b — Control Service Layer
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
related_file_ids: [FILE-CRATE-SERVICE-LIB]
visual_node_ids: [VN-CRATE-SERVICE]
approval_state: pending
---

# Phase 2b: Control Service Layer

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 2b |
| Title | Control Service Layer |
| Work paths | `WP-CORE-001` |
| Product module | Authority Core |
| Owner | cto (Dr. Rena Okafor) |
| Risk | critical |
| Decision blockers | none |

**Mandate:** Create the `control-service` crate — the orchestration layer between transports (API/CLI/MCP) and the store/domain/policy/audit layers. Every service method enforces: policy check first → audit event after mutation → snapshot anchor for rollback.

---

## Workflow

```
1. Create control-service crate scaffold
   a. Cargo.toml with dependencies
   b. lib.rs module declarations
   c. error.rs — ServiceError enum

2. Implement service modules (skeleton signatures first, fill later phases)
   a. session_service.rs — identity lifecycle
   b. command_service.rs — bounded command lifecycle
   c. audit_service.rs — audit trail operations
   d. snapshot_service.rs — snapshot operations
   e. rollback_service.rs — rollback operations
   f. skill_service.rs — skill registration and validation
   g. actor_service.rs — scope building
   h. work_packet_service.rs — packet generation

3. Add control-service to workspace Cargo.toml members

4. Verify crate compiles
```

---

## File Manifest

### New Crate: control-service

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-service/Cargo.toml` | 20 | Dependencies: authority-domain, audit-log, policy-engine, control-store, snapshot-ledger, chrono, serde, thiserror, tracing, uuid |
| NEW | `control-service/src/lib.rs` | 20 | Module declarations for all service modules |
| NEW | `control-service/src/error.rs` | 80 | `ServiceError` enum with typed error propagation. Variants: DomainError, PolicyDenied, AuditError, StoreError, SnapshotError, RollbackError, NotFound, Conflict, ValidationError, InternalError. Each wraps the underlying crate error type. |
| NEW | `control-service/src/session_service.rs` | 40 | Skeleton: `issue_session()`, `renew_session()`, `revoke_session()`, `validate_session()` — all return `Result<T, ServiceError>` |
| NEW | `control-service/src/command_service.rs` | 40 | Skeleton: `propose_command()`, `authorize_command()`, `apply_command()`, `deny_command()` — all return `Result<T, ServiceError>` |
| NEW | `control-service/src/audit_service.rs` | 30 | Skeleton: `append_event()`, `query_events()`, `verify_chain()` — all return `Result<T, ServiceError>` |
| NEW | `control-service/src/snapshot_service.rs` | 30 | Skeleton: `create_snapshot()`, `validate_snapshot()`, `compare_snapshots()` — all return `Result<T, ServiceError>` |
| NEW | `control-service/src/rollback_service.rs` | 30 | Skeleton: `preview_rollback()`, `execute_rollback()`, `verify_rollback()` — all return `Result<T, ServiceError>` |
| NEW | `control-service/src/skill_service.rs` | 30 | Skeleton: `register_skill()`, `activate_skill_session()`, `validate_skill_integrity()` — all return `Result<T, ServiceError>` |
| NEW | `control-service/src/actor_service.rs` | 30 | Skeleton: `get_scope()`, `update_scope()` — all return `Result<T, ServiceError>` |

### Workspace Root

| Action | File | Purpose |
|--------|------|---------|
| EXTEND | `Cargo.toml` | Add `control-service` to `[workspace.members]` |

---

## ServiceError Design

```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Domain validation error: {0}")]
    DomainError(#[from] authority_domain::DomainError),

    #[error("Policy denied: {denial}")]
    PolicyDenied { denial: PolicyDenial },

    #[error("Audit error: {0}")]
    AuditError(#[from] audit_log::AuditError),

    #[error("Store error: {0}")]
    StoreError(#[from] control_store::StoreError),

    #[error("Snapshot error: {0}")]
    SnapshotError(#[from] snapshot_ledger::SnapshotError),

    #[error("{entity_type} not found: {id}")]
    NotFound { entity_type: &'static str, id: String },

    #[error("{entity_type} conflict: {reason}")]
    Conflict { entity_type: &'static str, id: String, reason: String },

    #[error("Validation error: {field}: {reason}")]
    ValidationError { field: String, reason: String },

    #[error("Internal error: {0}")]
    InternalError(#[from] anyhow::Error),
}
```

---

## Completion Gates

- [x] `cargo check -p control-service` passes
- [x] All service modules have doc comments explaining preconditions/postconditions
- [x] Every service method returns `Result<T, ServiceError>` — no panics, no unwraps
- [x] No direct DB access — all persistence goes through control-store
- [x] Source files under 300 lines each
- [x] `cargo clippy --workspace -- -D warnings` passes

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-DOMAIN` | Depend on authority-domain types |
| `SGL-BACKEND-POLICY` | Depend on policy-engine |
| `SGL-BACKEND-SERVICE` | Create and own the control-service crate |

---

## Dependencies

- Phase 2a complete (foundation hardening — error types, enriched domain, policy, audit, snapshot)
