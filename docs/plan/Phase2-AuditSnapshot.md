---
doc_id: DOC-PLAN-P2D
title: Phase 2d — Audit Trail, Snapshot, Rollback & Work Packets
parent: DOC-PLAN-INDEX
status: draft
owner: cto
reviewers: [domain-architect, security-architect, data-architect, backend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-CORE-001]
related_decision_ids: []
related_file_ids: [FILE-CRATE-AUDIT-LIB, FILE-CRATE-SNAPSHOT-LIB, FILE-CRATE-STORE-LIB, FILE-CRATE-SERVICE-LIB, FILE-CRATE-DOMAIN-WORKPATH]
visual_node_ids: [VN-CRATE-SERVICE, VN-CRATE-AUDIT, VN-CRATE-SNAPSHOT, VN-CRATE-STORE]
approval_state: pending
---

# Phase 2d: Audit Trail, Snapshot, Rollback & Work Packets

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 2d |
| Title | Audit Trail, Snapshot, Rollback & Work Packets |
| Work paths | `WP-CORE-001` |
| Product module | Authority Core |
| Owner | cto (Dr. Rena Okafor) |
| Risk | critical |
| Decision blockers | none |

**Mandate:** Fully implement the audit trail engine (append-only with hash-chain verification), snapshot orchestration (create, validate, compare), rollback engine (preview, execute, verify), and agent work packet generator (scope generation, permission boundary calculation).

---

## Workflow

```
1. Audit trail engine
   a. audit-log — batch append, chain verification
   b. control-store — audit query by project/time/event_type, paged results
   c. audit_service.rs — full implementation

2. Snapshot orchestration
   a. snapshot-ledger — SnapshotDelta, rollback.rs module
   b. control-store — snapshot manifest list/compare queries
   c. snapshot_service.rs — full implementation

3. Rollback engine
   a. snapshot-ledger/src/rollback.rs — new module
   b. control-store — rollback preview queries
   c. rollback_service.rs — full implementation

4. Work packet generator
   a. authority-domain/src/work_packet.rs — new module
   b. work_packet_service.rs — full implementation
   c. control-store — work packet persistence

5. Skill service
   a. skill_service.rs — full implementation (register, activate, validate)
   b. actor_service.rs — full implementation (scope building from DB state)
```

---

## File Manifest

### audit-log

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `audit-log/src/lib.rs` | 80 | Add `AuditEvent::categorize()` — returns EventType. Add `AuditChainVerifier` — `verify_chain(events)` recomputes all hashes, returns first tampered event. Add `batch_append()` for bulk insert. |

### snapshot-ledger

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `snapshot-ledger/src/manifest.rs` | 60 | Add `SnapshotDelta` — `added_objects: Vec<String>`, `removed_objects: Vec<String>`, `changed_objects: Vec<String>`, `table_row_counts: HashMap<String, (u64, u64)>`. Add `compare(from: &SnapshotManifest, to: &SnapshotManifest) -> SnapshotDelta`. |
| NEW | `snapshot-ledger/src/rollback.rs` | 100 | Add `RollbackPreview` — `from_snapshot_id`, `to_snapshot_id`, `objects_to_restore: Vec<String>`, `objects_to_lose: Vec<String>`, `blockers: Vec<RollbackBlocker>`. Add `RollbackPlan` — sequence of operations to execute. Add `RollbackBlocker` — type of blocking condition (Conflict, MissingRef, IntegrityViolation). |
| EXTEND | `snapshot-ledger/src/object_store.rs` | 40 | Add `delete(path)`, `list(prefix)`, `stat(path)` operations with typed errors. |

### authority-domain

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `authority-domain/src/work_packet.rs` | 100 | `AgentWorkPacket` struct — `id: PacketId`, `agent_id: ActorId`, `work_path_node_id: String`, `objective: String`, `allowed_file_paths`, `denied_file_paths`, `required_contracts`, `required_tests`, `required_trace_points`, `rollback_anchor: Option<SnapshotId>`, `cost_budget`, `permission_scope: PacketPermissionScope`, `created_at`, `status: PacketStatus`. |
| EXTEND | `authority-domain/src/lib.rs` | 5 | Add `pub mod work_packet;` |

### control-store

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-store/src/lib.rs` | 80 | Add audit query methods: `query_events(project_id, filters)` with time range, event_type, actor_id, entity_type filters. Add `verify_chain(project_id)`. Add snapshot manifest list/compare queries. Add rollback preview queries (`insert_rollback_preview`, `get_rollback_preview`). Add work packet persistence (`insert_work_packet`, `get_work_packet`, `update_packet_status`). |

### control-service (FULL IMPLEMENTATION)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| FULL | `control-service/src/audit_service.rs` | 100 | `append_event(event)` → stores with hash chain link. `query_events(project_id, filters)` → time range, event_type, actor_id, entity_type. `verify_chain(project_id)` → recompute all hashes, detect tampering. `get_chain_anchors(project_id)` → first, last, count, integrity status. |
| FULL | `control-service/src/snapshot_service.rs` | 120 | `create_snapshot(project_id, reason, parent_id)` → scans registered objects, computes content-addressed refs, creates SnapshotManifest with hash chain, stores in DB + object store. `validate_snapshot(id)` → verify hash, verify all object refs exist, verify table exports. `list_snapshots(project_id)` → ordered by time. `compare_snapshots(from_id, to_id)` → compute SnapshotDelta. |
| FULL | `control-service/src/rollback_service.rs` | 120 | `preview_rollback(from_snapshot, to_snapshot)` → computes what will be restored/lost, identifies conflicts, returns RollbackPreview. `execute_rollback(from_snapshot, to_snapshot, preview_id)` → validates preview, restores objects + table exports, creates new snapshot with reason=rollback, emits audit events. `verify_rollback(snapshot_id)` → runs consistency checks, verifies file hashes, verifies entity counts. |
| FULL | `control-service/src/work_packet_service.rs` | 80 | `generate_work_packet(scope, node_id, objective)` → compute allowed/denied files from node→file links, extract required contracts/tests/traces, compute rollback anchor from latest snapshot, return AgentWorkPacket. `validate_packet_boundaries(packet_id)` → verify files within allowed scope, verify cost budget, verify contracts exist. |
| FULL | `control-service/src/skill_service.rs` | 80 | `register_skill(name, template)` → create SkillRegistration. `activate_skill_session(skill_id, agent_id, scope)` → bind skill to session, return SkillSession. `validate_skill_integrity(skill_id)` → check skill template against registered state. |
| FULL | `control-service/src/actor_service.rs` | 80 | `get_scope(actor_id, session_id)` → load actor's roles, active grants, build ActorScope from DB state. `update_scope(scope_id)` → refresh context (grants may have changed). |

---

## Completion Gates

- [x] 5 audit events appended → chain integrity verified
- [x] Tampering with one event's payload → chain verification fails
- [x] Audit query returns correct paged results with filters
- [x] Full snapshot lifecycle: create → validate → compare → store → retrieve
- [x] Full rollback lifecycle: preview → execute → verify
- [x] Snapshot chain integrity: parent→child links are valid
- [x] Rollback impact report correctly lists what changes
- [x] Packet generation correctly scopes allowed/denied files
- [x] Packet validation detects files outside allowed scope
- [x] Packet correctly identifies rollback anchor from latest snapshot
- [x] `cargo test --workspace` passes with 0 failures

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-DOMAIN` | Modify authority-domain work_packet module |
| `SGL-BACKEND-AUDIT` | Modify audit-log hash-chain verification |
| `SGL-BACKEND-SNAPSHOT` | Modify snapshot-ledger (manifest, rollback, object_store) |
| `SGL-BACKEND-SERVICE` | Implement all service methods |

---

## Dependencies

- Phase 2c complete (session and command lifecycles operational)
