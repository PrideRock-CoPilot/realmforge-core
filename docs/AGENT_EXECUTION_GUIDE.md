# ⚔️ RealmForge Core — Agent Execution Guide

## The Deep Companion to MASTER_BUILD_PLAN.md

---

> *This document is the execution layer beneath the battle map.*
> *Where MASTER_BUILD_PLAN.md tells you WHAT to build, this document tells you HOW.*
> *Every agent that enters this workspace must read MASTER_BUILD_PLAN.md first,*
> *then this document before touching a single file.*

---

## How to Use This Guide

This document provides:
- **Current state audit** — exactly what exists in the skeleton
- **Exact Rust type signatures** — not "extend scope.rs" but the precise methods and types
- **Exact DB query shapes** — the sqlx patterns for every store operation
- **Work packet specifications** — what an agent receives per phase
- **Test case templates** — what each validation gate actually verifies
- **Inter-phase contracts** — what must be provably true before the next phase begins

Each phase is structured as a **RealmForge work path artifact**:
```yaml
phase:
  id: phase.N.name
  status: planned | in_progress | complete
  depends_on: [phase.N-1.name]
  validation_gate: [list of required evidence]
  agent_allowed_files: [exact paths]
  agent_denied_paths: [exact patterns]
```

---

## Current State Audit (as of 2026-05-02)

### What the Skeleton Has

| Crate | Files | Line Count | Status |
|-------|-------|-----------|--------|
| `rf-domain` | 7 files | ~435 lines | Skeleton — types defined, logic sparse |
| `rf-events` | 1 file | 107 lines | Solid — AuditEvent + hash chain working |
| `rf-policy` | 1 file | 172 lines | Solid — 6-check authorize_action() working |
| `rf-snapshot` | 3 files | 183 lines | Partial — manifest + file store, no validation |
| `rf-store` | 1 file | 139 lines | Partial — 3 methods only, no queries |
| `rf-api` | 2 files | 58 lines | Stub — health + authorize endpoints only |
| `rf-mcp` | 1 file | 128 lines | Stub — 5 tool defs, 1 handler |
| `rf-cli` | 1 file | 50 lines | Stub — 3 commands, no service wiring |

### What Does NOT Exist Yet

```
rf-service/          DOES NOT EXIST — the entire service layer is missing
rf-domain/error.rs   DOES NOT EXIST — no unified domain error type
rf-snapshot/rollback.rs  DOES NOT EXIST
rf-api/routes/       DOES NOT EXIST
rf-api/error.rs      DOES NOT EXIST
rf-api/middleware.rs DOES NOT EXIST
rf-api/models.rs     DOES NOT EXIST
rf-mcp/tools/        DOES NOT EXIST
rf-mcp/error.rs      DOES NOT EXIST
rf-cli/commands/     DOES NOT EXIST
```

### What the Database Has

**Migration 001 tables:** tenants, projects, actors, roles, actor_roles, skill_registrations,
sessions, skill_sessions, bounded_commands, core_audit_events

**Migration 002 tables:** snapshot_manifests, snapshot_object_refs, snapshot_table_exports,
rollback_previews

**What's MISSING from the schema:**
- No `work_packets` table (Phase 6 adds this)
- No `actor_scope_cache` table (Phase 2 adds this or we compute on-the-fly)
- No index on `core_audit_events.actor_id` (Phase 4 migration)
- No `session_state` check constraint (Phase 2 migration adds this)

---

## The Error Hierarchy Contract

This is the single most important architectural contract in the system. Every agent must
understand the full error propagation chain before writing a single `?` operator.

```
DomainError          (rf-domain/error.rs)  — pure domain violations, no IO
    ↑ wrapped by
EventError           (rf-events/lib.rs)    — serialization failures only
PolicyDenial         (rf-policy/lib.rs)    — authorization rejections (not errors)
    ↑ all wrapped by
StoreError           (rf-store/lib.rs)     — DB failures
    ↑ all wrapped by
ServiceError         (rf-service/error.rs) — orchestration failures (most important)
    ↑ all wrapped by
ApiError             (rf-api/error.rs)     — HTTP presentation layer
McpError             (rf-mcp/error.rs)     — MCP presentation layer
CliError             (rf-cli/error.rs)     — CLI exit code mapping
```

**The Rule:** Errors flow UP only. An error from a lower layer is wrapped (not swallowed)
at each layer crossing. The message at the top layer always includes the chain.

---

## Phase 0 — Foundation Hardening

```yaml
phase:
  id: phase.0.foundation-hardening
  status: planned
  depends_on: []
  validation_gate:
    - cargo_test_workspace_passes
    - cargo_clippy_zero_warnings
    - cargo_fmt_check_passes
    - all_files_under_300_lines
  agent_work_packet:
    allowed_crates: [rf-domain, rf-events, rf-policy, rf-snapshot]
    forbidden_crates: [rf-store, rf-service, rf-api, rf-cli, rf-mcp]
    allowed_actions: [extend_types, add_impls, add_tests, create_error_module]
    forbidden_actions: [add_io, add_db_queries, add_http_routes]
```

### Phase 0.1 — `rf-domain/src/error.rs` (NEW)

Create this file first. Everything else in Phase 0 references it.

```rust
// Target: ~40 lines

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid identifier: {reason}")]
    InvalidId { reason: String },

    #[error("invalid scope: {reason}")]
    InvalidScope { reason: String },

    #[error("illegal state transition from {from} to {to}")]
    IllegalStateTransition { from: String, to: String },

    #[error("skill integrity check failed: {reason}")]
    SkillIntegrityFailed { reason: String },

    #[error("session has expired")]
    SessionExpired,

    #[error("command in terminal state: {status}")]
    CommandTerminated { status: String },

    #[error("approval required but not present")]
    ApprovalRequired,
}
```

After creating this, add to `rf-domain/src/lib.rs`:
```rust
pub mod error;
pub use error::*;
```

### Phase 0.2 — `rf-domain/src/ids.rs` (EXTEND)

Current: 67 lines — typed ID newtypes with `new()`, `generate()`, `as_str()`
Missing: `Display`, `FromStr`, `Serialize`/`Deserialize` — WAIT, check first.

Look at the existing `ids.rs` before extending. If `Display`, `Serialize`, `Deserialize` are
already derived via the macro, only add:

```rust
// Add to the macro or each ID type:
impl std::fmt::Display for ActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
// Repeat pattern for all 12 ID types
```

**The 12 typed IDs that must all have Display:**
TenantId, ProjectId, ActorId, RoleId, SessionId, SkillId, SkillSessionId,
CommandId, AuditEventId, SnapshotId, SnapshotObjectId, PacketId (PacketId added Phase 6)

**Validation test to add:**
```rust
#[test]
fn id_display_round_trips() {
    let id = ActorId::new("actor-123").unwrap();
    assert_eq!(format!("{}", id), "actor-123");
    assert_eq!(id.to_string(), "actor-123");
}
```

### Phase 0.3 — `rf-domain/src/scope.rs` (EXTEND)

Current: 28 lines — ActorScope struct with `allows_action()`
Add these methods:

```rust
impl ActorScope {
    /// Returns true if the scope has not yet expired.
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        now < self.expires_at
    }

    /// Returns the number of seconds until expiry (negative if already expired).
    pub fn seconds_until_expiry(&self, now: DateTime<Utc>) -> i64 {
        (self.expires_at - now).num_seconds()
    }

    /// Returns true if the scope context is fresh enough.
    pub fn is_context_fresh(&self, now: DateTime<Utc>, max_age_seconds: i64) -> bool {
        let age = (now - self.context_updated_at).num_seconds();
        age <= max_age_seconds
    }

    /// Build a minimal read-only scope for a newly issued session.
    pub fn read_only(
        tenant_id: TenantId,
        project_id: ProjectId,
        actor_id: ActorId,
        session_id: SessionId,
        ttl_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            tenant_id,
            project_id,
            actor_id,
            roles: vec![],
            session_id,
            skill_session_id: None,
            requested_skill_id: None,
            active_skill_id: None,
            allowed_actions: vec![],
            approval_id: None,
            approval_state: ApprovalState::Pending,
            execution_mode: ExecutionMode::ReadOnly,
            context_updated_at: now,
            expires_at: now + Duration::seconds(ttl_seconds),
        }
    }
}
```

### Phase 0.4 — `rf-domain/src/state.rs` (EXTEND)

Current: 48 lines — enums (ExecutionMode, ApprovalState, CommandStatus, SnapshotStatus, etc.)
Add TryFrom conversions for DB string round-trips:

```rust
impl TryFrom<&str> for CommandStatus {
    type Error = DomainError;
    fn try_from(s: &str) -> Result<Self, DomainError> {
        match s {
            "proposed"   => Ok(Self::Proposed),
            "authorized" => Ok(Self::Authorized),
            "applied"    => Ok(Self::Applied),
            "denied"     => Ok(Self::Denied),
            "failed"     => Ok(Self::Failed),
            other => Err(DomainError::IllegalStateTransition {
                from: other.to_string(),
                to: "(unknown)".to_string(),
            }),
        }
    }
}

// Repeat pattern for: ApprovalState, ExecutionMode, SnapshotStatus
// Each enum maps to lowercase string (its DB column value)
```

Add Display for each enum (mirrors the TryFrom):
```rust
impl std::fmt::Display for CommandStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Proposed   => "proposed",
            Self::Authorized => "authorized",
            Self::Applied    => "applied",
            Self::Denied     => "denied",
            Self::Failed     => "failed",
        };
        write!(f, "{}", s)
    }
}
```

### Phase 0.5 — `rf-domain/src/command.rs` (EXTEND)

Current: 41 lines — BoundedCommand struct
Add state machine validation:

```rust
impl BoundedCommand {
    /// Validates that a status transition is legal.
    /// Returns Err(DomainError::IllegalStateTransition) for illegal moves.
    pub fn validate_transition(
        from: &CommandStatus,
        to: &CommandStatus,
    ) -> Result<(), DomainError> {
        let allowed = match from {
            CommandStatus::Proposed   => matches!(to, CommandStatus::Authorized | CommandStatus::Denied),
            CommandStatus::Authorized => matches!(to, CommandStatus::Applied | CommandStatus::Failed),
            CommandStatus::Applied    => false,  // terminal
            CommandStatus::Denied     => false,  // terminal
            CommandStatus::Failed     => false,  // terminal
        };
        if allowed {
            Ok(())
        } else {
            Err(DomainError::IllegalStateTransition {
                from: from.to_string(),
                to: to.to_string(),
            })
        }
    }
}
```

**Test to add:**
```rust
#[test]
fn proposed_to_authorized_is_legal() {
    assert!(BoundedCommand::validate_transition(
        &CommandStatus::Proposed,
        &CommandStatus::Authorized
    ).is_ok());
}

#[test]
fn applied_is_terminal() {
    assert!(BoundedCommand::validate_transition(
        &CommandStatus::Applied,
        &CommandStatus::Denied
    ).is_err());
}
```

### Phase 0.6 — `rf-events/src/lib.rs` (EXTEND)

Add event type categorization (needed by audit_service in Phase 3):

```rust
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EventCategory {
    Session,
    Command,
    Snapshot,
    Rollback,
    Policy,
    Skill,
    System,
}

impl AuditEvent {
    pub fn category(&self) -> EventCategory {
        match self.event_type.split('.').next().unwrap_or("") {
            "session"  => EventCategory::Session,
            "command"  => EventCategory::Command,
            "snapshot" => EventCategory::Snapshot,
            "rollback" => EventCategory::Rollback,
            "policy"   => EventCategory::Policy,
            "skill"    => EventCategory::Skill,
            _          => EventCategory::System,
        }
    }
}
```

### Phase 0.7 — `rf-policy/src/lib.rs` (EXTEND)

Add `DenialCode::RateLimited` and a chain evaluator that can run multiple checks in sequence:

```rust
// Add to DenialCode enum:
RateLimited,

// Add chain evaluator:
pub struct PolicyChain<'a> {
    scope: &'a ActorScope,
    action: &'a str,
    mutating: bool,
    approval_required: bool,
    now: DateTime<Utc>,
    max_context_age_seconds: i64,
}

impl<'a> PolicyChain<'a> {
    pub fn new(scope: &'a ActorScope, action: &'a str) -> Self {
        Self {
            scope,
            action,
            mutating: false,
            approval_required: false,
            now: Utc::now(),
            max_context_age_seconds: 300,
        }
    }

    pub fn mutating(mut self) -> Self { self.mutating = true; self }
    pub fn approval_required(mut self) -> Self { self.approval_required = true; self }
    pub fn at(mut self, now: DateTime<Utc>) -> Self { self.now = now; self }
    pub fn max_age(mut self, seconds: i64) -> Self { self.max_context_age_seconds = seconds; self }

    pub fn evaluate(self) -> PolicyDecision {
        authorize_action(
            self.scope,
            self.action,
            self.mutating,
            self.approval_required,
            self.now,
            self.max_context_age_seconds,
        )
    }
}
```

Usage pattern in service layer:
```rust
let decision = PolicyChain::new(&scope, "command.apply")
    .mutating()
    .approval_required()
    .evaluate();
```

### Phase 0.8 — `rf-snapshot/src/manifest.rs` (EXTEND)

Add validation and delta computation:

```rust
impl SnapshotManifest {
    /// Validates the manifest's own hash integrity.
    pub fn validate_hash(&self) -> Result<bool, serde_json::Error> {
        // recompute hash and compare to self.manifest_hash
    }

    /// Returns objects present in self but not in other (added in self).
    pub fn added_since(&self, other: &SnapshotManifest) -> Vec<&SnapshotObjectRef> {
        let other_paths: std::collections::HashSet<&str> =
            other.object_refs.iter().map(|r| r.logical_path.as_str()).collect();
        self.object_refs.iter()
            .filter(|r| !other_paths.contains(r.logical_path.as_str()))
            .collect()
    }

    /// Returns objects present in other but not in self (removed in self).
    pub fn removed_since(&self, other: &SnapshotManifest) -> Vec<&SnapshotObjectRef> {
        let self_paths: std::collections::HashSet<&str> =
            self.object_refs.iter().map(|r| r.logical_path.as_str()).collect();
        other.object_refs.iter()
            .filter(|r| !self_paths.contains(r.logical_path.as_str()))
            .collect()
    }

    /// Returns objects present in both but with different sha256 (changed in self).
    pub fn changed_since(&self, other: &SnapshotManifest) -> Vec<(&SnapshotObjectRef, &SnapshotObjectRef)> {
        // pair up matching logical_paths where sha256 differs
    }
}

/// The delta between two snapshots.
pub struct SnapshotDelta {
    pub from_id: SnapshotId,
    pub to_id: SnapshotId,
    pub added: Vec<SnapshotObjectRef>,
    pub removed: Vec<SnapshotObjectRef>,
    pub changed: Vec<(SnapshotObjectRef, SnapshotObjectRef)>,
}
```

### Phase 0 Validation Gate

The phase is complete when ALL of the following pass:

```
[ ] cargo fmt --all -- --check                         (zero formatting issues)
[ ] cargo clippy --workspace -- -D warnings            (zero warnings)
[ ] cargo test --workspace                             (all tests pass)
[ ] rf-domain/src/error.rs exists and compiles         (DomainError with 7 variants)
[ ] All 12 typed IDs have Display impl                 (tested by display_round_trips test)
[ ] CommandStatus::validate_transition() has 5 tests   (one per from-state)
[ ] TryFrom<&str> impls exist for all 4 state enums   (tested by round-trip tests)
[ ] PolicyChain builder pattern compiles               (tested by existing tests)
[ ] SnapshotDelta struct exists and compiles           (no behavior test yet)
[ ] No file in rf-domain, rf-events, rf-policy, rf-snapshot exceeds 300 lines
```

---

## Phase 1 — Service Layer Creation

```yaml
phase:
  id: phase.1.service-layer
  status: planned
  depends_on: [phase.0.foundation-hardening]
  precondition: Phase 0 validation gate must be 100% green
  validation_gate:
    - rf_service_crate_compiles
    - all_service_methods_have_doc_comments
    - no_direct_db_access_in_service
    - cargo_check_p_rf_service_passes
  agent_work_packet:
    new_crate: rf-service
    allowed_actions: [create_crate, define_service_interfaces, write_error_types]
    forbidden_actions: [implement_db_queries, add_http_routes, touch_rf_domain_files]
```

### Phase 1.1 — `rf-service/Cargo.toml` (NEW)

```toml
[package]
name = "rf-service"
version = "0.1.0"
edition.workspace = true
license.workspace = true
rust-version.workspace = true
authors.workspace = true

[dependencies]
rf-domain   = { workspace = true }
rf-events   = { workspace = true }
rf-policy   = { workspace = true }
rf-store    = { workspace = true }
rf-snapshot = { workspace = true }

chrono      = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
thiserror   = { workspace = true }
tracing     = { workspace = true }
tokio       = { workspace = true }
uuid        = { workspace = true }
```

**Also update root `Cargo.toml`** — add `"crates/rf-service"` to the `[workspace] members` array.
**Also update root `Cargo.toml`** — add `rf-service = { path = "crates/rf-service" }` to `[workspace.dependencies]`.

### Phase 1.2 — `rf-service/src/error.rs` (NEW)

```rust
// Target: ~60 lines
use rf_domain::DomainError;
use rf_events::EventError;
use rf_policy::PolicyDenial;
use crate::store::StoreError;  // re-exported from rf-store
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("domain rule violated: {0}")]
    Domain(#[from] DomainError),

    #[error("policy denied: {denial:?}")]
    PolicyDenied { denial: PolicyDenial },

    #[error("persistence failed: {0}")]
    Store(#[from] StoreError),

    #[error("event chain failed: {0}")]
    Event(#[from] EventError),

    #[error("entity not found: {entity_type} {id}")]
    NotFound { entity_type: &'static str, id: String },

    #[error("entity conflict: {reason}")]
    Conflict { reason: String },

    #[error("session expired or invalid")]
    SessionInvalid,

    #[error("operation not permitted in current state: {reason}")]
    InvalidState { reason: String },
}

impl ServiceError {
    pub fn not_found(entity_type: &'static str, id: impl ToString) -> Self {
        Self::NotFound { entity_type, id: id.to_string() }
    }

    pub fn conflict(reason: impl Into<String>) -> Self {
        Self::Conflict { reason: reason.into() }
    }

    pub fn invalid_state(reason: impl Into<String>) -> Self {
        Self::InvalidState { reason: reason.into() }
    }
}
```

### Phase 1.3 — `rf-service/src/lib.rs` (NEW)

```rust
// Target: ~25 lines — module declarations only
pub mod actor_service;
pub mod audit_service;
pub mod command_service;
pub mod error;
pub mod rollback_service;
pub mod session_service;
pub mod skill_service;
pub mod snapshot_service;

pub use error::ServiceError;

// Re-export store for convenience
pub use rf_store::CoreStore;
```

### Phase 1.4 — Service Method Signatures (ALL 7 SERVICE FILES)

Each service file at this phase is INTERFACE ONLY — method signatures with
`todo!()` bodies. Full implementation happens in Phases 2–6.

**`rf-service/src/session_service.rs`** (NEW, ~50 lines signatures):
```rust
use crate::error::ServiceError;
use rf_domain::{ActorId, ActorScope, ProjectId, SessionId, TenantId};

pub struct SessionService {
    store: rf_store::CoreStore,
}

impl SessionService {
    pub fn new(store: rf_store::CoreStore) -> Self { Self { store } }

    /// Issues a new session for the given actor. Returns the session ID and initial scope.
    pub async fn issue_session(
        &self,
        actor_id: ActorId,
        tenant_id: TenantId,
        project_id: ProjectId,
        ttl_seconds: i64,
    ) -> Result<(SessionId, ActorScope), ServiceError> {
        todo!("Phase 2: Session Lifecycle Engine")
    }

    /// Activates a previously issued session.
    pub async fn activate_session(&self, session_id: &SessionId) -> Result<ActorScope, ServiceError> {
        todo!("Phase 2")
    }

    /// Extends an active session's expiry by ttl_seconds.
    pub async fn renew_session(
        &self,
        session_id: &SessionId,
        ttl_seconds: i64,
    ) -> Result<ActorScope, ServiceError> {
        todo!("Phase 2")
    }

    /// Revokes an active session immediately.
    pub async fn revoke_session(
        &self,
        session_id: &SessionId,
        reason: &str,
    ) -> Result<(), ServiceError> {
        todo!("Phase 2")
    }

    /// Validates that a session is still active and returns its current scope.
    pub async fn validate_session(&self, session_id: &SessionId) -> Result<ActorScope, ServiceError> {
        todo!("Phase 2")
    }
}
```

**`rf-service/src/command_service.rs`** (NEW, ~70 lines signatures):
```rust
use crate::error::ServiceError;
use rf_domain::{ActorScope, BoundedCommand, CommandId, CommandStatus};
use serde_json::Value;

pub struct CommandService {
    store: rf_store::CoreStore,
}

impl CommandService {
    pub fn new(store: rf_store::CoreStore) -> Self { Self { store } }

    /// Proposes a new bounded command. Returns the CommandId.
    /// Policy is NOT checked here — this is a pure write.
    pub async fn propose_command(
        &self,
        scope: &ActorScope,
        action: impl Into<String>,
        target_type: impl Into<String>,
        target_id: impl Into<String>,
        payload: Value,
    ) -> Result<CommandId, ServiceError> {
        todo!("Phase 3")
    }

    /// Checks policy and transitions command from Proposed → Authorized | Denied.
    /// Always emits an audit event regardless of outcome.
    pub async fn authorize_command(
        &self,
        command_id: &CommandId,
        scope: &ActorScope,
    ) -> Result<CommandStatus, ServiceError> {
        todo!("Phase 3")
    }

    /// Applies an authorized command. Scope must match the session that authorized it.
    /// Transitions Authorized → Applied. Emits audit event. Creates snapshot anchor.
    pub async fn apply_command(
        &self,
        command_id: &CommandId,
        scope: &ActorScope,
    ) -> Result<BoundedCommand, ServiceError> {
        todo!("Phase 3")
    }

    /// Explicitly denies a command. Transitions Proposed → Denied.
    /// Emits audit denial event.
    pub async fn deny_command(
        &self,
        command_id: &CommandId,
        reason: impl Into<String>,
    ) -> Result<(), ServiceError> {
        todo!("Phase 3")
    }

    /// Retrieves the current state of a command.
    pub async fn get_command(&self, command_id: &CommandId) -> Result<BoundedCommand, ServiceError> {
        todo!("Phase 3")
    }
}
```

**`rf-service/src/audit_service.rs`** (NEW, ~60 lines signatures):
```rust
use crate::error::ServiceError;
use chrono::{DateTime, Utc};
use rf_domain::{ActorId, AuditEventId, ProjectId};
use rf_events::AuditEvent;
use serde_json::Value;

pub struct AuditService {
    store: rf_store::CoreStore,
}

pub struct AuditQuery {
    pub project_id: ProjectId,
    pub actor_id: Option<ActorId>,
    pub event_type_prefix: Option<String>,
    pub entity_type: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: i64,
    pub offset: i64,
}

pub struct ChainVerification {
    pub project_id: ProjectId,
    pub event_count: u64,
    pub integrity_valid: bool,
    pub first_event_id: Option<AuditEventId>,
    pub last_event_id: Option<AuditEventId>,
    pub tampered_at: Option<AuditEventId>,
}

impl AuditService {
    pub fn new(store: rf_store::CoreStore) -> Self { Self { store } }

    /// Appends a new audit event, linking it to the previous event's hash.
    /// The event_type convention is "entity.verb" e.g. "command.authorized"
    pub async fn append_event(
        &self,
        project_id: ProjectId,
        actor_id: ActorId,
        event_type: impl Into<String>,
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        payload: Value,
    ) -> Result<AuditEvent, ServiceError> {
        todo!("Phase 4")
    }

    /// Queries events with optional filters. Returns paged results.
    pub async fn query_events(&self, query: AuditQuery) -> Result<Vec<AuditEvent>, ServiceError> {
        todo!("Phase 4")
    }

    /// Recomputes all hashes in the project's audit chain.
    /// Returns integrity status and the ID of the first tampered event, if any.
    pub async fn verify_chain(&self, project_id: &ProjectId) -> Result<ChainVerification, ServiceError> {
        todo!("Phase 4")
    }
}
```

**`rf-service/src/snapshot_service.rs`** (NEW, ~55 lines signatures):
```rust
use crate::error::ServiceError;
use rf_domain::{ProjectId, SnapshotId, TenantId};
use rf_snapshot::{SnapshotDelta, SnapshotManifest};

pub struct SnapshotService {
    store: rf_store::CoreStore,
}

pub struct SnapshotValidation {
    pub snapshot_id: SnapshotId,
    pub manifest_hash_valid: bool,
    pub all_objects_present: bool,
    pub all_exports_present: bool,
    pub object_count: usize,
    pub missing_objects: Vec<String>,
}

impl SnapshotService {
    pub fn new(store: rf_store::CoreStore) -> Self { Self { store } }

    /// Creates a new snapshot manifest. Computes content-addressed refs for provided objects.
    pub async fn create_snapshot(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        reason: impl Into<String>,
        parent_id: Option<SnapshotId>,
    ) -> Result<SnapshotManifest, ServiceError> {
        todo!("Phase 5")
    }

    /// Validates a snapshot's manifest hash and all object references.
    pub async fn validate_snapshot(
        &self,
        snapshot_id: &SnapshotId,
    ) -> Result<SnapshotValidation, ServiceError> {
        todo!("Phase 5")
    }

    /// Returns snapshots for a project, ordered newest first.
    pub async fn list_snapshots(
        &self,
        project_id: &ProjectId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SnapshotManifest>, ServiceError> {
        todo!("Phase 5")
    }

    /// Computes the delta between two snapshots.
    pub async fn compare_snapshots(
        &self,
        from_id: &SnapshotId,
        to_id: &SnapshotId,
    ) -> Result<SnapshotDelta, ServiceError> {
        todo!("Phase 5")
    }
}
```

**`rf-service/src/rollback_service.rs`** (NEW, ~55 lines signatures):
```rust
use crate::error::ServiceError;
use rf_domain::SnapshotId;

pub struct RollbackService {
    store: rf_store::CoreStore,
}

pub struct RollbackPreview {
    pub from_id: SnapshotId,
    pub to_id: SnapshotId,
    pub objects_to_restore: Vec<String>,
    pub objects_to_lose: Vec<String>,
    pub conflicts: Vec<String>,
    pub blockers: Vec<String>,
    pub is_safe: bool,
}

pub struct RollbackVerification {
    pub snapshot_id: SnapshotId,
    pub metadata_consistent: bool,
    pub file_hashes_match: bool,
    pub entity_counts_match: bool,
    pub issues: Vec<String>,
}

impl RollbackService {
    pub fn new(store: rf_store::CoreStore) -> Self { Self { store } }

    /// Computes the impact of rolling back from from_id to to_id. Does NOT execute.
    pub async fn preview_rollback(
        &self,
        from_id: &SnapshotId,
        to_id: &SnapshotId,
    ) -> Result<RollbackPreview, ServiceError> {
        todo!("Phase 5")
    }

    /// Executes a rollback. Requires a preview_id to ensure impact was acknowledged.
    pub async fn execute_rollback(
        &self,
        from_id: &SnapshotId,
        to_id: &SnapshotId,
        preview_id: &str,
    ) -> Result<SnapshotId, ServiceError> {
        todo!("Phase 5")
    }

    /// Verifies the consistency of state after a rollback.
    pub async fn verify_rollback(
        &self,
        snapshot_id: &SnapshotId,
    ) -> Result<RollbackVerification, ServiceError> {
        todo!("Phase 5")
    }
}
```

**`rf-service/src/skill_service.rs`** (NEW, ~40 lines signatures):
```rust
use crate::error::ServiceError;
use rf_domain::{ProjectId, SessionId, SkillId, SkillRegistration, SkillSession, SkillSessionId};

pub struct SkillService {
    store: rf_store::CoreStore,
}

impl SkillService {
    pub fn new(store: rf_store::CoreStore) -> Self { Self { store } }

    /// Registers a new skill in the project catalog.
    pub async fn register_skill(
        &self,
        project_id: ProjectId,
        skill: SkillRegistration,
    ) -> Result<SkillId, ServiceError> {
        todo!("Phase 2")
    }

    /// Binds a skill to an active session.
    pub async fn activate_skill_session(
        &self,
        session_id: SessionId,
        skill_id: SkillId,
    ) -> Result<SkillSessionId, ServiceError> {
        todo!("Phase 2")
    }

    /// Validates that a skill's current state matches expected integrity.
    pub async fn validate_skill_integrity(
        &self,
        skill_id: &SkillId,
    ) -> Result<bool, ServiceError> {
        todo!("Phase 2")
    }
}
```

**`rf-service/src/actor_service.rs`** (NEW, ~40 lines signatures):
```rust
use crate::error::ServiceError;
use rf_domain::{ActorId, ActorScope, SessionId};

pub struct ActorService {
    store: rf_store::CoreStore,
}

impl ActorService {
    pub fn new(store: rf_store::CoreStore) -> Self { Self { store } }

    /// Builds a full ActorScope by loading actor roles, session state, and skill bindings.
    pub async fn get_scope(
        &self,
        actor_id: &ActorId,
        session_id: &SessionId,
    ) -> Result<ActorScope, ServiceError> {
        todo!("Phase 2")
    }

    /// Refreshes the scope's context_updated_at timestamp with fresh role data.
    pub async fn refresh_scope(
        &self,
        session_id: &SessionId,
    ) -> Result<ActorScope, ServiceError> {
        todo!("Phase 2")
    }
}
```

### Phase 1 Validation Gate

```
[ ] rf-service/ crate directory exists with Cargo.toml
[ ] root Cargo.toml includes "crates/rf-service" in workspace members
[ ] cargo check -p rf-service compiles with zero errors
[ ] All 7 service files exist with correct signatures
[ ] All service structs take CoreStore as their only constructor arg
[ ] All methods return Result<T, ServiceError>
[ ] No method bodies yet — all use todo!("Phase N") markers
[ ] All public items have /// doc comments
[ ] ServiceError covers all 8 variants defined above
[ ] No file in rf-service exceeds 100 lines at this phase (signatures only)
```

---

## Phase 2 — Session Lifecycle Engine

```yaml
phase:
  id: phase.2.session-lifecycle
  status: planned
  depends_on: [phase.1.service-layer]
  precondition: Phase 1 validation gate 100% green
  validation_gate:
    - session_lifecycle_test_passes
    - expired_session_returns_session_invalid
    - scope_reflects_execution_mode
    - all_session_mutations_emit_audit_events
  agent_work_packet:
    allowed_files:
      - crates/rf-service/src/session_service.rs
      - crates/rf-service/src/actor_service.rs
      - crates/rf-service/src/skill_service.rs
      - crates/rf-domain/src/scope.rs
      - crates/rf-store/src/lib.rs
      - crates/rf-store/src/session_queries.rs   (NEW)
    forbidden_paths:
      - crates/rf-api/**
      - crates/rf-mcp/**
      - crates/rf-cli/**
      - db/migrations/** (without CTO sign-off)
```

### Phase 2.1 — DB Queries Needed in `rf-store`

Add a new file `rf-store/src/session_queries.rs` (NEW, ~120 lines):

```rust
// These are the sqlx query shapes needed by SessionService and ActorService.
// All queries use named parameters bound in order.

// INSERT a new session
r#"
INSERT INTO sessions (id, tenant_id, project_id, actor_id, state, expires_at, created_at)
VALUES ($1, $2, $3, $4, 'issued', $5, NOW())
"#

// SELECT session by id (returns full row for scope building)
r#"
SELECT s.id, s.tenant_id, s.project_id, s.actor_id, s.state, s.expires_at,
       ss.skill_id as active_skill_id, ss.id as skill_session_id
FROM sessions s
LEFT JOIN skill_sessions ss ON ss.session_id = s.id AND ss.state = 'active'
WHERE s.id = $1
"#

// UPDATE session state (issued → active, active → revoked)
r#"
UPDATE sessions SET state = $2, updated_at = NOW() WHERE id = $1 AND state = $3
RETURNING id
"#

// Extend session expiry
r#"
UPDATE sessions SET expires_at = $2, updated_at = NOW()
WHERE id = $1 AND state = 'active'
RETURNING expires_at
"#

// SELECT actor roles for scope building
r#"
SELECT r.id as role_id
FROM actor_roles ar
JOIN roles r ON r.id = ar.role_id
WHERE ar.actor_id = $1 AND ar.project_id = $2
"#
```

### Phase 2.2 — Session Lifecycle Integration Test

Create `rf-service/tests/session_lifecycle.rs`:

```rust
// Test 1: Full lifecycle — issue → activate → renew → revoke
#[tokio::test]
async fn session_full_lifecycle() {
    let store = test_store().await;
    let svc = SessionService::new(store);

    // Issue
    let (session_id, scope) = svc.issue_session(
        actor_id(), tenant_id(), project_id(), 300
    ).await.unwrap();
    assert_eq!(scope.execution_mode, ExecutionMode::ReadOnly);

    // Activate
    let scope = svc.activate_session(&session_id).await.unwrap();
    assert!(scope.is_active(Utc::now()));

    // Renew
    let renewed = svc.renew_session(&session_id, 600).await.unwrap();
    assert!(renewed.seconds_until_expiry(Utc::now()) > 500);

    // Revoke
    svc.revoke_session(&session_id, "test teardown").await.unwrap();

    // Validate (after revoke)
    let err = svc.validate_session(&session_id).await.unwrap_err();
    assert!(matches!(err, ServiceError::SessionInvalid));
}

// Test 2: Expired session returns SessionInvalid
#[tokio::test]
async fn expired_session_is_invalid() {
    // issue with ttl=1, sleep 2s, validate
    // assert ServiceError::SessionInvalid
}

// Test 3: Scope execution mode starts as ReadOnly
#[tokio::test]
async fn issued_scope_is_read_only() {
    // issue → assert scope.execution_mode == ExecutionMode::ReadOnly
}
```

### Phase 2 Validation Gate

```
[ ] issue_session() inserts a session row in state='issued'
[ ] activate_session() transitions state='issued' → 'active'
[ ] renew_session() extends expires_at
[ ] revoke_session() transitions state='active' → 'revoked' AND emits audit event
[ ] validate_session() returns SessionInvalid if state != 'active' or NOW() > expires_at
[ ] get_scope() correctly builds ActorScope with roles from actor_roles table
[ ] All 3 integration tests pass
[ ] rf-store session queries are in separate session_queries.rs file (< 150 lines)
```

---

## Phase 3 — Command Lifecycle Engine

```yaml
phase:
  id: phase.3.command-lifecycle
  status: planned
  depends_on: [phase.2.session-lifecycle]
  validation_gate:
    - propose_authorize_apply_chain_passes
    - denial_emits_audit_event
    - applying_unauthorized_command_returns_error
    - state_transitions_enforced_by_domain_layer
```

### Phase 3 Critical Design Decisions

1. **`propose_command` does NOT check policy.** It is a pure write. Policy is checked in `authorize_command`.
2. **`authorize_command` ALWAYS emits an audit event** — whether allowed or denied.
3. **`apply_command` ALWAYS creates a snapshot anchor.** Specifically, it calls `SnapshotService::create_snapshot` with reason `"command.applied:{command_id}"`.
4. **The CommandService must use PolicyChain** (from Phase 0) — not raw `authorize_action` calls.

### Phase 3 — DB Queries Needed

Add `rf-store/src/command_queries.rs` (NEW, ~120 lines):

```rust
// INSERT bounded_command
r#"
INSERT INTO bounded_commands (
  id, tenant_id, project_id, actor_id, session_id, action,
  target_type, target_id, payload, status, created_at
)
VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'proposed',NOW())
"#

// SELECT command by id
r#"
SELECT id, tenant_id, project_id, actor_id, session_id, action,
       target_type, target_id, payload, status, created_at, updated_at
FROM bounded_commands WHERE id = $1
"#

// UPDATE command status (with previous state check for idempotency)
r#"
UPDATE bounded_commands
SET status = $2, updated_at = NOW()
WHERE id = $1 AND status = $3
RETURNING id, status
"#
```

### Phase 3 Validation Gate

```
[ ] propose_command() inserts row with status='proposed'
[ ] authorize_command() with authorized scope → status='authorized', audit event emitted
[ ] authorize_command() with expired scope → status='denied', audit event emitted, denial returned
[ ] apply_command() with status='authorized' → status='applied', audit event, snapshot anchor
[ ] apply_command() with status='proposed' → ServiceError::InvalidState (cannot skip auth)
[ ] deny_command() → status='denied', audit event with reason
[ ] BoundedCommand::validate_transition() is called on every status change
[ ] All 5 command lifecycle integration tests pass
```

---

## Phase 4 — Audit Trail Engine

```yaml
phase:
  id: phase.4.audit-trail
  status: planned
  depends_on: [phase.3.command-lifecycle]
  validation_gate:
    - 5_events_chain_integrity_verified
    - tampered_event_detected
    - paged_query_returns_correct_results
```

### Phase 4 — DB Queries Needed

Add `rf-store/src/audit_queries.rs` (NEW, ~100 lines):

```rust
// SELECT last event hash for a project (for chain linking)
r#"
SELECT event_hash FROM core_audit_events
WHERE project_id = $1
ORDER BY occurred_at DESC, id DESC
LIMIT 1
"#

// SELECT events with filters (dynamic query — use sqlx QueryBuilder)
// Filters: project_id (required), actor_id, event_type LIKE prefix, entity_type,
//          occurred_at range, LIMIT, OFFSET
// ORDER BY occurred_at ASC, id ASC

// SELECT all events for chain verification (ordered)
r#"
SELECT id, previous_hash, event_hash, payload, occurred_at
FROM core_audit_events
WHERE project_id = $1
ORDER BY occurred_at ASC, id ASC
"#
```

### Phase 4 Critical Design Decisions

1. **Chain linking:** `append_event` must fetch the last event's hash WITHIN THE SAME TRANSACTION as the insert, to prevent race conditions. Use `BEGIN SERIALIZABLE` or advisory lock.
2. **Hash verification:** `verify_chain` recomputes every hash from scratch. It does NOT trust stored hashes. It computes `AuditEvent::compute_hash()` for each event.
3. **Paged queries:** Use sqlx QueryBuilder for dynamic filter composition.

### Phase 4 Validation Gate

```
[ ] append_event() fetches previous hash in same transaction as insert
[ ] verify_chain() recomputes all hashes (does not trust stored values)
[ ] ChainVerification.tampered_at is set to first inconsistent event ID
[ ] query_events() respects limit/offset
[ ] query_events() filters by actor_id when provided
[ ] query_events() filters by event_type_prefix (LIKE 'command.%')
[ ] Integration test: append 5 events, verify chain → integrity_valid = true
[ ] Integration test: tamper event 3 payload → verify_chain → tampered_at = event_3_id
```

---

## Phase 5 — Snapshot & Rollback Engine

```yaml
phase:
  id: phase.5.snapshot-rollback
  status: planned
  depends_on: [phase.4.audit-trail]
  validation_gate:
    - snapshot_lifecycle_complete
    - rollback_lifecycle_complete
    - snapshot_chain_parent_links_valid
    - rollback_impact_report_correct
```

### Phase 5 — New File: `rf-snapshot/src/rollback.rs`

```rust
// ~80 lines
use rf_domain::SnapshotId;
use crate::manifest::{SnapshotManifest, SnapshotObjectRef};

/// The computed impact of rolling back from one snapshot to another.
pub struct RollbackPreview {
    pub preview_id: String,        // UUID — must be presented to execute_rollback
    pub from_id: SnapshotId,
    pub to_id: SnapshotId,
    pub objects_to_restore: Vec<SnapshotObjectRef>,
    pub objects_to_lose: Vec<SnapshotObjectRef>,
    pub net_object_delta: i64,     // positive = more objects after rollback
    pub is_safe: bool,             // false if there are blockers
    pub blockers: Vec<String>,     // reasons rollback should not proceed
    pub warnings: Vec<String>,     // non-blocking concerns
}

impl RollbackPreview {
    /// Computes a preview. Does NOT access the database — pure manifest comparison.
    pub fn compute(from: &SnapshotManifest, to: &SnapshotManifest) -> Self {
        // Use SnapshotManifest delta methods from Phase 0
        // blockers = empty (for V1 — future: check active command sessions)
        todo!()
    }
}
```

### Phase 5 Validation Gate

```
[ ] create_snapshot() → inserts manifest + object_refs + table_exports in single transaction
[ ] validate_snapshot() → verifies manifest hash matches recomputed hash
[ ] list_snapshots() → returns ordered, respects limit/offset
[ ] compare_snapshots() → returns correct added/removed/changed counts
[ ] RollbackPreview::compute() correctly identifies objects_to_restore and objects_to_lose
[ ] execute_rollback() requires a valid preview_id (computed within 10 minutes)
[ ] execute_rollback() creates a new snapshot with reason='rollback'
[ ] execute_rollback() emits audit events for every restored object
[ ] verify_rollback() checks manifest hash of the new snapshot
[ ] Integration test: full lifecycle passes
```

---

## Phase 6 — Agent Work Packet Generator

```yaml
phase:
  id: phase.6.work-packet-generator
  status: planned
  depends_on: [phase.5.snapshot-rollback]
  validation_gate:
    - packet_scopes_allowed_denied_files
    - packet_validation_detects_out_of_scope_files
    - packet_rollback_anchor_is_latest_snapshot
  new_files:
    - crates/rf-domain/src/work_packet.rs
    - crates/rf-service/src/work_packet_service.rs
  schema_migration_required:
    - 003_work_packets.sql  (adds agent_work_packets table)
```

### Phase 6 — Migration 003

Create `db/migrations/003_work_packets.sql`:

```sql
CREATE TABLE agent_work_packets (
  id                  TEXT PRIMARY KEY,
  agent_id            TEXT NOT NULL REFERENCES actors(id),
  project_id          TEXT NOT NULL REFERENCES projects(id),
  work_path_node_id   TEXT,
  objective           TEXT NOT NULL,
  allowed_file_paths  JSONB NOT NULL DEFAULT '[]',
  denied_file_paths   JSONB NOT NULL DEFAULT '[]',
  required_contracts  JSONB NOT NULL DEFAULT '[]',
  required_tests      JSONB NOT NULL DEFAULT '[]',
  required_trace_points JSONB NOT NULL DEFAULT '[]',
  rollback_anchor_id  TEXT REFERENCES snapshot_manifests(id),
  cost_budget_tokens  BIGINT,
  status              TEXT NOT NULL DEFAULT 'active'
                      CHECK (status IN ('active','consumed','expired','revoked')),
  created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at          TIMESTAMPTZ,
  consumed_at         TIMESTAMPTZ
);

CREATE INDEX agent_work_packets_agent_project
  ON agent_work_packets(agent_id, project_id, status);
```

---

## Phase 7 — Full API Surface

```yaml
phase:
  id: phase.7.full-api
  status: planned
  depends_on: [phase.6.work-packet-generator]
  validation_gate:
    - cargo_run_rf_api_starts
    - health_endpoint_200
    - all_endpoints_correct_status_codes
    - error_responses_rfc7807
```

### Phase 7 — API Error Standard

ALL error responses from `rf-api` follow RFC 7807 Problem Details:

```json
{
  "type": "urn:realmforge:error:policy_denied",
  "title": "Policy Denied",
  "status": 403,
  "detail": "scope execution mode cannot mutate state",
  "instance": "/v1/commands/cmd-123/authorize",
  "request_id": "req-abc-def-ghi",
  "denial_code": "PROPOSAL_ONLY",
  "corrective_action": "switch_execution_mode"
}
```

### Phase 7 — HTTP Status Code Contract

| Outcome | Code | Notes |
|---------|------|-------|
| Successful create | 201 | All POST mutations |
| Successful read | 200 | GET endpoints |
| Policy denied | 403 | Include denial in body |
| Not found | 404 | Entity not found |
| Invalid state | 409 | State machine conflict |
| Invalid input | 422 | Schema validation failure |
| Server error | 500 | Non-domain errors only |

### Phase 7 — Route Catalog

```
GET  /health                               → HealthResponse
GET  /v1/ready                             → ReadinessResponse (DB ping)

POST /v1/sessions                          → 201 SessionResponse
GET  /v1/sessions/:id                      → 200 SessionResponse
POST /v1/sessions/:id/activate             → 200 ActorScopeResponse
POST /v1/sessions/:id/renew                → 200 ActorScopeResponse
DELETE /v1/sessions/:id                    → 204 (revoke)

POST /v1/commands                          → 201 CommandResponse
GET  /v1/commands/:id                      → 200 CommandResponse
POST /v1/commands/:id/authorize            → 200 CommandResponse
POST /v1/commands/:id/apply                → 200 CommandResponse
POST /v1/commands/:id/deny                 → 200 CommandResponse

GET  /v1/audit/events                      → 200 AuditEventPage
GET  /v1/audit/chain/verify                → 200 ChainVerification

POST /v1/snapshots                         → 201 SnapshotManifestResponse
GET  /v1/snapshots                         → 200 SnapshotListResponse
GET  /v1/snapshots/:id                     → 200 SnapshotManifestResponse
POST /v1/snapshots/:id/validate            → 200 SnapshotValidation

POST /v1/rollback/preview                  → 201 RollbackPreviewResponse
POST /v1/rollback/execute                  → 201 SnapshotManifestResponse
GET  /v1/rollback/:id/verify               → 200 RollbackVerification

GET  /v1/actors/:id/scope                  → 200 ActorScopeResponse

POST /v1/work-packets/generate             → 201 WorkPacketResponse
POST /v1/work-packets/:id/validate         → 200 WorkPacketValidation
```

---

## Phase 8 — Full CLI Surface

```yaml
phase:
  id: phase.8.full-cli
  status: planned
  depends_on: [phase.7.full-api]
  validation_gate:
    - help_shows_all_subcommands
    - json_output_default
    - pretty_flag_works
    - exit_codes_correct
```

### Phase 8 — CLI Command Catalog

```
rf session issue      --actor-id --tenant-id --project-id --ttl
rf session activate   --session-id
rf session renew      --session-id --ttl
rf session revoke     --session-id --reason
rf session list       --project-id [--limit] [--offset]

rf command propose    --action --target-type --target-id --payload-json
rf command authorize  --command-id
rf command apply      --command-id
rf command deny       --command-id --reason
rf command get        --command-id

rf audit events       --project-id [--from] [--to] [--type] [--limit] [--offset]
rf audit verify-chain --project-id

rf snapshot create    --project-id --reason [--parent-id]
rf snapshot validate  --snapshot-id
rf snapshot list      --project-id [--limit]
rf snapshot compare   --from-id --to-id

rf rollback preview   --from-id --to-id
rf rollback execute   --from-id --to-id --preview-id
rf rollback verify    --snapshot-id

rf actor scope        --actor-id --session-id

rf work-packet generate --agent-id --project-id --objective
rf work-packet validate --packet-id

rf skill register     --project-id --name --description
rf skill list         --project-id

rf migrate up
rf migrate down       --target-version
rf migrate list

rf config init        --database-url
rf config show
rf config validate
```

### Phase 8 — Exit Code Contract

```
0 → success
1 → user input error (bad flags, missing required args)
2 → data error (entity not found, conflict, policy denied)
3 → runtime error (DB connection failure, internal error)
```

---

## Phase 9 — Full MCP Surface

```yaml
phase:
  id: phase.9.full-mcp
  status: planned
  depends_on: [phase.7.full-api]
  validation_gate:
    - all_20_tools_defined
    - unknown_tool_returns_mcp_error
    - every_tool_returns_valid_json
```

### Phase 9 — MCP Tool Catalog (20 tools)

```
core_issue_session
core_activate_session
core_renew_session
core_revoke_session
core_validate_session

core_propose_command
core_authorize_command     (already exists — extend, do not replace)
core_apply_command
core_deny_command
core_get_command

core_append_audit_event
core_query_audit_events
core_verify_audit_chain

core_create_snapshot
core_validate_snapshot
core_list_snapshots
core_compare_snapshots

core_preview_rollback
core_execute_rollback
core_verify_rollback

core_get_actor_scope
core_generate_work_packet
core_validate_work_packet

core_register_skill
core_activate_skill_session
```

Note: That's 25 tools. The MASTER_BUILD_PLAN targets "under 25" — correct. The test assertion
in rf-mcp's existing test (`<= 15`) must be updated to `<= 25` in Phase 9.

---

## Phase 10 — Integration Testing

```yaml
phase:
  id: phase.10.integration-testing
  status: planned
  depends_on: [phase.9.full-mcp]
  validation_gate:
    - cargo_test_workspace_80_plus_tests
    - all_tests_pass_under_60_seconds
    - cross_crate_happy_path_passes
    - security_boundary_tests_pass
```

### Phase 10 — Test Database Pattern

All integration tests use this pattern:

```rust
// rf-service/tests/common/mod.rs
pub async fn test_store() -> CoreStore {
    let url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/realmforge_test".to_string());
    let store = CoreStore::connect(&url).await.unwrap();
    // Run migrations
    // Create isolated schema: CREATE SCHEMA test_<uuid>
    // Set search_path to that schema
    store
}

pub fn tenant_id() -> TenantId { TenantId::new("test-tenant").unwrap() }
pub fn project_id() -> ProjectId { ProjectId::new("test-project").unwrap() }
pub fn actor_id() -> ActorId { ActorId::new("test-actor").unwrap() }
```

### Phase 10 — The Golden Path Test

This is the single most important integration test. It must pass before any phase
can be declared complete for production use.

```rust
// rf-api/tests/integration_test.rs

#[tokio::test]
async fn golden_path_propose_to_rollback() {
    let app = test_app().await;

    // 1. Issue session
    // 2. Activate session
    // 3. Propose command
    // 4. Authorize command → assert status=authorized
    // 5. Apply command → assert status=applied
    // 6. Query audit events → assert 2 events (authorized + applied)
    // 7. Verify audit chain → assert integrity_valid=true
    // 8. Create snapshot
    // 9. Validate snapshot → assert all valid
    // 10. Preview rollback
    // 11. Execute rollback
    // 12. Verify rollback → assert consistent
    // 13. Query audit events → assert rollback events present
}
```

---

## Phase 11 — Observability & Error Boundaries

```yaml
phase:
  id: phase.11.observability
  status: planned
  depends_on: [phase.10.integration-testing]
  validation_gate:
    - structured_trace_visible_in_console
    - policy_decisions_logged_with_context
    - error_chain_annotated_at_each_layer
    - x_request_id_header_present
```

### Phase 11 — Tracing Contract

Every service method MUST follow this pattern:

```rust
pub async fn authorize_command(
    &self,
    command_id: &CommandId,
    scope: &ActorScope,
) -> Result<CommandStatus, ServiceError> {
    let span = tracing::info_span!(
        "command_service.authorize_command",
        command_id = %command_id,
        actor_id = %scope.actor_id,
        project_id = %scope.project_id,
    );
    let _guard = span.enter();

    // ... implementation ...
}
```

### Phase 11 — Policy Decision Log Format

Every call to `authorize_action` in the service layer must be followed by:

```rust
tracing::info!(
    action = %action,
    allowed = %decision.allowed,
    denial_code = ?decision.denial.as_ref().map(|d| &d.code),
    actor_id = %scope.actor_id,
    execution_mode = ?scope.execution_mode,
    "policy_decision"
);
```

---

## Dependency Graph (Resolved)

```
rf-cli ─────────────────────────────────────────────────────┐
rf-api ──────────────────────────────────────────────────┐  │
rf-mcp ───────────────────────────────────────────────┐  │  │
                                                       │  │  │
                                                       ▼  ▼  ▼
                                                    rf-service
                                                    ├── rf-store
                                                    │     ├── rf-domain
                                                    │     ├── rf-events ──→ rf-domain
                                                    │     └── rf-snapshot ─→ rf-domain
                                                    ├── rf-policy ──────→ rf-domain
                                                    ├── rf-events ──────→ rf-domain
                                                    ├── rf-snapshot ────→ rf-domain
                                                    └── rf-domain  (pure — no rf-* deps)
```

**Layer Violation Scanner** — run this after every phase to verify no violations:

```bash
# These commands must return ZERO matches:
grep -r "use rf_store" crates/rf-domain/src/   # MUST be empty
grep -r "use rf_store" crates/rf-api/src/      # MUST be empty (use rf-service instead)
grep -r "use rf_policy" crates/rf-api/src/     # MUST be empty (use rf-service instead)
grep -r "sqlx" crates/rf-service/src/          # MUST be empty (use rf-store instead)
```

---

## Migration Plan

| Migration | Phase | What It Adds |
|-----------|-------|-------------|
| `001_core_foundation.sql` | ✅ Exists | Core tables |
| `002_snapshot_foundation.sql` | ✅ Exists | Snapshot tables |
| `003_work_packets.sql` | Phase 6 | agent_work_packets |
| `004_indexes.sql` | Phase 4 | Additional audit/session indexes |

Migration 004 adds:
```sql
-- Needed for audit query performance
CREATE INDEX core_audit_events_actor ON core_audit_events(actor_id);
CREATE INDEX core_audit_events_entity ON core_audit_events(entity_type, entity_id);
-- Needed for session validation performance
CREATE INDEX sessions_actor ON sessions(actor_id, state);
```

---

## The Promise

When all 11 phases are complete and all validation gates are green, any AI agent that
enters this workspace can be given a work packet and will be able to:

1. **Look up** what they are allowed to touch (work packet file manifest)
2. **Propose** a bounded command against their scope
3. **Receive** a policy decision (not a prompt — a database-enforced response)
4. **Execute** within their authorized file boundaries
5. **Emit** evidence into the audit chain
6. **Anchor** their work to a snapshot before and after
7. **Rollback** to a known-good state if anything goes wrong

No unscoped agents. No invisible work. No ungoverned runtime behavior.

*This is the heartbeat. This is the soul. This is RealmForge Core.*

---

*Generated by the RealmForge company planning session.*
*Alex Rivera (PM) — scoped and structured.*
*Dr. Rena Okafor (CTO) — architecture contracted and layer law enforced.*
*Dmitri Volkov (Backend) — implementation patterns validated.*
*Builder of Builders — catalog clean, registry intact.*
