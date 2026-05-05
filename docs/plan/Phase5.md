---
doc_id: DOC-PLAN-P5
title: Phase 5 — Knowledge And Parquet Snapshots
parent: DOC-PLAN-INDEX
status: draft
owner: data-architect
reviewers: [cto, data-engineer, backend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-KNOWLEDGE-001]
related_decision_ids: [DEC-COUNCIL-003]
related_file_ids: [FILE-CRATE-PARQUET-LIB, FILE-CRATE-KNOWLEDGE-LIB]
visual_node_ids: [VN-MODULE-KNOWLEDGE]
approval_state: pending
---

# Phase 5: Knowledge And Parquet Snapshots

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 5 |
| Title | Knowledge And Parquet Snapshots |
| Work paths | `WP-KNOWLEDGE-001` |
| Product module | Knowledge |
| Owner | data-architect (Chen Wei) |
| Risk | high |
| Decision blockers | `DEC-COUNCIL-003` (Parquet library and query engine) |

**Mandate:** Build the scoped knowledge ingestion, indexing, and retrieval system backed by Parquet snapshots. Knowledge answers cite governed records and respect grant scope.

---

## Workflow

```
Phase split into two tracks (parallel where possible):

Track A (was blocked by DEC-COUNCIL-003, now resolved):
  1. Parquet schema definitions — KnowledgeRecord, EvidenceRecord, CatalogSnapshot
  2. Parquet writer/reader crate (parquet-store) using Arrow + DataFusion
  3. Parquet dataset versioning
  4. Parquet→Postgres reconciliation

Track B (complete):
  1. Knowledge domain types (authority-domain)
  2. Knowledge service interface (control-service)
  3. Knowledge store interface (control-store — Postgres metadata)
  4. Knowledge API/CLI/MCP surfaces
  5. Acceptance tests for knowledge retrieval
```

---

## File Manifest

### authority-domain

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| DONE | `authority-domain/src/knowledge.rs` | 87 | `KnowledgeRecord`, `KnowledgeQuery`, `Citation`, `DatasetInfo` |
| DONE | `authority-domain/src/lib.rs` | 5 | Export `knowledge` module. |

### Parquet crate (parquet-store)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `parquet-store/Cargo.toml` | 20 | Dependencies: datafusion, parquet, arrow, chrono, serde, thiserror, tokio. |
| NEW | `parquet-store/src/lib.rs` | 250 | `ParquetDataset`, `DatasetVersion`, `ParquetDatasetConfig` |
| NEW | `parquet-store/src/writer.rs` | 220 | `KnowledgeWriter`, `EvidenceWriter`, `SchemaValidator` |
| NEW | `parquet-store/src/reader.rs` | 280 | `KnowledgeReader`, `DatasetReader`, `ParquetKnowledgeQuery` |
| NEW | `parquet-store/src/schemas.rs` | 140 | Schema definitions: `knowledge_schema()`, `evidence_schema()`, `catalog_snapshot_schema()`, `work_path_schema()` |
| NEW | `parquet-store/src/error.rs` | 40 | `ParquetStoreError` enum |

### control-service

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| UPDATED | `control-service/src/knowledge_service.rs` | 145 | Now uses `parquet-store` when configured (write + query Parquet) |
| UPDATED | `control-service/src/error.rs` | 130 | Added `Parquet(#[from] ParquetStoreError)` variant |

### control-store / control-api / operator-cli / agent-mcp

All Track B surfaces were implemented in Phase 5 prior to DEC-COUNCIL-003 resolution.

---

## Key Types

### KnowledgeRecord (NEW)
```rust
pub struct KnowledgeRecord {
    pub id: KnowledgeId,
    pub scope: KnowledgeScope,        // Global, Tenant, App, WorkPath
    pub source_type: SourceType,      // Catalog, File, Decision, Evidence, Trace
    pub source_id: String,
    pub content_hash: String,
    pub indexed_at: DateTime<Utc>,
    pub citations: Vec<Citation>,
}

pub struct KnowledgeQuery {
    pub scopes: Vec<KnowledgeScope>,
    pub source_types: Option<Vec<SourceType>>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub text_search: Option<String>,
    pub limit: u32,
    pub offset: u32,
}

pub struct Citation {
    pub source_document_id: String,
    pub excerpt: String,
    pub confidence: f32,              // 0.0–1.0
}
```

---

## Completion Gates

- [ ] `TEST-KNOWLEDGE-001` — Knowledge answer cites governed records and respects grant scope
- [x] Knowledge query returns scoped results — agent cannot see records outside its grant scope
- [x] Ingest validates scope before writing
- [x] Parquet dataset versioning works (write version N, read version N, list versions)
- [x] Postgres index reconciliation finds orphaned or missing datasets
- [x] API, CLI, and MCP surfaces all support knowledge query + ingest
- [x] Parquet crate (parquet-store) created with Arrow + DataFusion per DEC-COUNCIL-003
- [x] KnowledgeWriter writes KnowledgeRecords to Parquet with schema validation
- [x] KnowledgeReader queries Parquet with scope, source_type, date range, text search filters
- [x] ParquetDataset supports versioned writes and query
- [x] Service wired: KnowledgeService uses parquet-store when configured with `new_with_parquet()`
- [x] Error propagation: ServiceError::Parquet variant for parquet-store errors
- [ ] `cargo test --workspace` passes with 0 failures
- [ ] `cargo clippy --workspace -- -D warnings` passes

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-DATA-PARQUET` | Create and modify Parquet crate (granted per DEC-COUNCIL-003) |
| `SGL-DATA-POSTGRES` | Add knowledge metadata to control-store |
| `SGL-BACKEND-SERVICE` | Add knowledge_service to control-service |
| `SGL-BACKEND-API` | Add knowledge route group to control-api |
| `SGL-BACKEND-CLI` | Add knowledge commands to operator-cli |
| `SGL-BACKEND-DOMAIN` | Add knowledge types to authority-domain |

---

## Dependencies

- Phase 3 complete (Catalogs and Work Paths — knowledge ingests from catalog records)
- Phase 4 complete (Agent Gateway — knowledge respects grant scopes in queries)
- `DEC-COUNCIL-003` resolved (Parquet library and query engine decision — Arrow + DataFusion)
