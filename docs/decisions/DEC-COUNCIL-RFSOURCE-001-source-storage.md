---
doc_id: DOC-DECISION-RFSOURCE-001
title: "Council Decision: Source Storage — Remove Parquet, Adopt .rfsource + PostgreSQL"
status: accepted
owner: council
reviewers: [cto, backend, data-architect, security-architect, domain-architect, api-architect, pm]
created_at: 2026-05-06
last_reviewed_at: 2026-05-06
source_of_truth: true
product_area: architecture
work_path_ids: [WP-RFSOURCE-001]
related_decision_ids: [DEC-COUNCIL-003]
related_file_ids: [DOC-SPEC-022]
visual_node_ids: [VN-RFSOURCE-DECISIONS]
visual_edge_ids: []
approval_state: accepted
---

# Council Decision: Source Storage — Remove Parquet, Adopt `.rfsource` + PostgreSQL

## Decision ID
`DEC-COUNCIL-RFSOURCE-001`

## Status
**ACCEPTED** — DRI: Dr. Rena Okafor (CTO), Council convened 2026-05-06.

## Context

RealmForge currently uses Apache Parquet as a storage backend for source artifacts. This was established under `DEC-COUNCIL-003` (2026-05-05, Apache Arrow + DataFusion). However, Parquet — a columnar analytical format — is architecturally unsuitable for:

- Append-only versioned source storage with per-chunk grant-scoped access
- Branch, proposal, comment, and time-warp operations
- Single-file portable workspaces
- Governance-verified source artifact management

The POC at `e:\rfsource` demonstrates a working alternative: a single-file append-only format (`.rfsource`) with compressed frames, typed IDs, version trees, grant-scoped reads, and full governance validation.

## Decision

**Remove Parquet entirely from the RealmForge storage stack.** The storage architecture going forward consists of exactly two systems:

1. **`.rfsource` single-file format** — Canonical source-of-truth for all source artifacts, commits, branches, proposals, comments, and time-warp state. Compact, append-only, portable, content-addressed.

2. **PostgreSQL** — Artifact Registry metadata, governance state, session/actor state, audit trail, snapshot manifests. All existing `control-store` tables remain.

### What This Means

| System | Before | After |
|--------|--------|-------|
| Source artifacts | Parquet files | `.rfsource` files (one per project/repo) |
| Artifact metadata | Implicit in Parquet schema | PostgreSQL Artifact Registry tables |
| Knowledge/evidence | Parquet datasets | PostgreSQL + `.rfsource` |
| Catalog/snapshots | Parquet datasets | PostgreSQL |
| Work path state | Parquet datasets | PostgreSQL |
| The `parquet-store` crate | Active | **Removed from workspace** |

### What Stays Unchanged

- PostgreSQL remains the operational database for all governance, session, audit, and snapshot state
- The existing `control-store` crate continues to own all PostgreSQL persistence
- All existing table schemas (sessions, commands, audit events, etc.) remain unchanged
- All existing crate responsibilities per the Crate Law remain

## Rationale

1. **Architectural fit**: `.rfsource` was designed for this exact use case — append-only versioned source storage with grant-scoped access. Parquet was adapted for it.
2. **Performance**: Single-file `.rfsource` avoids the multi-file scatter of per-version Parquet files. All artifacts, chunks, and symbols live in one append-only stream.
3. **Portability**: A `.rfsource` file is a complete, self-validating project snapshot — no directory structure needed.
4. **Governance integration**: Per-chunk grants, policy bindings, and version trees are first-class concepts in `.rfsource` — they had to be layered awkwardly on top of Parquet.
5. **Simplified storage stack**: Two systems (`.rfsource` + PostgreSQL) instead of three (`.rfsource` + Parquet + PostgreSQL).

## Consequences

1. **Migration**: Existing Parquet data must be migrated to `.rfsource` format. This is a one-time migration during Phase 1 of rfsource adoption.
2. **Dependency removal**: `datafusion`, `parquet`, `arrow`, and `arrow-schema` crate dependencies can be removed from the workspace.
3. **Spec updates**: All spec docs referencing Parquet as a storage layer must be updated.
4. **DEC-COUNCIL-003 superseded**: The Parquet library decision is superseded by this decision. The new source storage decision replaces it.

## Dissenting Opinions

- **Security Architect (Fatima Al-Hassan)**: The `.rfsource` format has no formal threat model. Requesting a threat model be produced before production adoption. **Response**: Accepted — the threat model will be produced as part of the rfsource format crate development.
- **Data Architect (Chen Wei)**: The split data story needs clear sync semantics between `.rfsource` and PostgreSQL. **Response**: Accepted — the Artifact Registry layer (PostgreSQL) is the index; `.rfsource` is the canonical data. Sync is always `.rfsource` → PostgreSQL index. Never bidirectional.

## Linked Documents

- Supersedes: `DEC-COUNCIL-003` (Parquet library decision)
- Related ADR: To be produced by Tech Writer (Clara Mills)
- Spec docs: New `docs/spec/` documents for rfsource crate family TBD
