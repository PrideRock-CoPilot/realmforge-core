---
doc_id: DOC-SPEC-023
title: RFSource Threat Model
status: draft
owner: security-architect
reviewers: [cto, backend]
created_at: 2026-05-06
last_reviewed_at: 2026-05-06
source_of_truth: true
product_area: rfsource
work_path_ids: [WP-RFSOURCE-002]
related_decision_ids: [DEC-USER-RFSOURCE-001]
related_file_ids: []
visual_node_ids: [VN-RFSOURCE-THREAT-MODEL]
visual_edge_ids: []
approval_state: pending
---

# RFSource Threat Model

## Scope

This threat model covers the full `.rfsource` single-file source ledger stack:

- `rfsource-core` — domain types, model definitions
- `rfsource-store` — file I/O, commit, branch, proposal, comment, time-warp operations
- `rfsource-service` — service orchestration layer
- `rfsource-catalog` — Artifact Registry (in-memory, PostgreSQL target)
- `rfsource-governance` — governance validation rules
- POC features to be ported (branch CRUD, proposals, comments, time warp, compare_refs)

## Assets

| Asset | Description | Sensitivity |
|---|---|---|
| `.rfsource` file on disk | Canonical source of truth — all code, all history, all grants | **Critical** |
| Grant bindings | SGL-RFSOURCE-* grants determining read/write/govern access | **Critical** |
| Commit chain | Immutable append-only history | **High** |
| Branch records | Named pointers into the commit chain | **High** |
| Proposals (merge requests) | Pending changes between branches | **Medium** |
| Code review comments | Attached to artifact versions | **Low-Medium** |
| Time warp capability | Rollback to any previous state | **Critical** |

## Trust Zones

```
Untrusted:     External callers, CLI users, MCP tool callers, HTTP request bodies
Edge-trust:    Authenticated session tokens with validated grants
Policy-trust:  Grants evaluated at the service/store layer
System-trust:  .rfsource files on disk (trusted but crash-vulnerable)
```

## STRIDE Findings

### F-001 [CRITICAL]: commit_artifact() bypasses grant check

**Location:** `crates/rfsource-store/src/rf_source.rs` lines 119-121

```rust
pub fn commit_artifact(&self, req: CommitArtifactRequest) -> Result<CommitOutcome> {
    self.commit_artifact_on_branch(MAIN_BRANCH_NAME, req, None)
}
```

`None` is passed as `actor_grant`, causing every direct call to `commit_artifact()` to skip all grant enforcement. Any caller with access to `RFSource` can write to main without authorization.

**STRIDE:** Elevation of Privilege

**Requirement:** Remove `commit_artifact()` as a public API or require an explicit `actor_grant` parameter that cannot be `None`.

### F-002 [CRITICAL]: Empty allowed_grants bypasses authorization

**Location:** `crates/rfsource-store/src/rf_source.rs` lines 148-158

```rust
if let Some(grant) = actor_grant {
    if !req.allowed_grants.is_empty()          // <-- BYPASS
        && !req.allowed_grants.contains(&grant.to_string())
        && !req.allowed_grants.contains(&"*".to_string())
```

When `allowed_grants` is empty (the default in tests), the entire authorization check is skipped. An attacker submits `allowed_grants: vec![]` to write to any artifact.

**STRIDE:** Elevation of Privilege

**Requirement:** Default-deny when `allowed_grants` is empty. Only skip the check if `actor_grant` is `*`.

### F-003 [CRITICAL]: No actor authentication

**Location:** All methods across `rfsource-store` and POC code.

`actor` is a freeform string set by the caller with zero verification. No token validation, no signature, no session. Any caller can impersonate any user.

**STRIDE:** Spoofing, Repudiation

**Requirement:** Actor identity must come from an authenticated session context at the transport layer, not from the request body. The `actor` field should be populated from the session token.

### F-004 [MEDIUM]: TOCTOU race in commit_artifacts_on_branch

**Location:** `crates/rfsource-store/src/rf_source.rs` lines 289-298

Each call to `commit_artifact_on_branch` internally calls `read_valid_state()` separately. Under concurrent access, callers may see the same `head_commit_id`, producing sibling commits with the same parent.

**STRIDE:** Tampering

**Requirement:** Build all bundles from a single state read and write atomically.

### F-005 [MEDIUM]: Catalog registry test uses wrong grant name

**Location:** `crates/rfsource-catalog/src/registry.rs` line 168

Test helper `sample_entry()` uses `SGL-BACKEND-READ` instead of `SGL-RFSOURCE-READ` (per `DEC-USER-RFSOURCE-001`, Option B).

**STRIDE:** Configuration

**Requirement:** Update to `SGL-RFSOURCE-READ`.

### F-006 [MEDIUM]: Time warp scope lacks grant gating

**Location:** POC `source_control.rs` `preview_time_warp()` lines 290-305

`"file"`, `"branch"`, and `"project"` scopes are accepted but not gated by grant level. A caller with `SGL-RFSOURCE-COMMIT` can perform a project-wide time warp.

**STRIDE:** Elevation of Privilege

**Requirement:** `SGL-RFSOURCE-GOVERN` required for project/branch scope. `SGL-RFSOURCE-COMMIT` allowed for file-scoped time warp on artifacts where the grant is in `allowed_grants`.

### F-007 [MEDIUM]: Proposal apply has atomicity gap

**Location:** POC `source_control.rs` `apply_proposal()` lines 203-209

Two separate `append_frames` calls: bundles first, proposal status second. A crash after the first write leaves bundles committed but the proposal still "open", allowing re-apply with duplicate commits.

**STRIDE:** Tampering

**Requirement:** Use single atomic frame write, or implement WAL/checkpoint layer before the feature ships.

### F-008 [LOW]: Production VersionView has empty content_hash

**Location:** `crates/rfsource-store/src/tree.rs` lines 59-63

```rust
content_hash: String::new()
```

Tree comparisons cannot detect actual content changes since the view doesn't carry the content hash.

**Requirement:** Populate `content_hash` from `CommitBundle.version.content_hash` during tree construction.

### F-009 [LOW]: No content size limits

`CommitArtifactRequest.content` is unbounded. An attacker could commit multi-GB artifacts.

**Requirement:** Add configurable content size limit (default 10 MB).

### F-010 [LOW]: No concurrent-write safety

`.rfsource` append-only format has no file locking. Concurrent writers can corrupt the file.

**Requirement:** Document as known limitation. Add file-level advisory lock when concurrency model is introduced.

## Security Requirements Summary

| # | Requirement | Priority | Source |
|---|---|---|---|
| R-001 | `commit_artifact()` must require `actor_grant` — no `None` bypass | **Blocking** | F-001 |
| R-002 | Grant check must default-deny when `allowed_grants` is empty | **Blocking** | F-002 |
| R-003 | Actor identity must come from authenticated context, not request body | **Blocking** | F-003 |
| R-004 | `commit_artifacts_on_branch` must build all bundles from one state read | **Required** | F-004 |
| R-005 | Update catalog test to use `SGL-RFSOURCE-READ` | **Required** | F-005 |
| R-006 | Time warp scope must be gated by grant level (GOVERN for project/branch) | **Required** | F-006 |
| R-007 | Proposal apply must use atomic frame writes | **Required** | F-007 |
| R-008 | Populate `content_hash` in production `VersionView` | **Improvement** | F-008 |
| R-009 | Add content size limits (default 10 MB) | **Improvement** | F-009 |
| R-010 | Document concurrent-write safety limitations | **Improvement** | F-010 |

## Residual Risk

1. **Single-user trust model.** The current rfsource implementation assumes a trusted caller is on the other side. Until authentication (R-003) is implemented, this is a fundamental gap. All other controls are layered on top of an unauthenticated foundation.
2. **No file encryption at rest.** `.rfsource` files are stored as plain JSON frames. Anyone with filesystem access can read all grants and all code.
3. **No rate limiting.** No protection against rapid-fire commits, branch creation, or proposal spam.
