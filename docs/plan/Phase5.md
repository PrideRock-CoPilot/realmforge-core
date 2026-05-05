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

⚠️ **DECISION BLOCKER:** This phase requires `DEC-COUNCIL-003` to be resolved before Parquet work can start. The Council must decide which Parquet library and query engine to standardize on. Until then, only type definitions, service interfaces, and spec work are permitted.

---

## Workflow

```
Phase split into two tracks (parallel where possible):

Track A (blocked by DEC-COUNCIL-003):
  1. Parquet schema definitions — KnowledgeRecord, EvidenceRecord, CatalogSnapshot
  2. Parquet writer/reader crate (once library decision is made)
  3. Parquet dataset versioning
  4. Parquet→Postgres reconciliation

Track B (not blocked):
  1. Knowledge domain types (authority-domain)
  2. Knowledge service interface (control-service)
  3. Knowledge store interface (control-store — Postgres metadata)
  4. Knowledge API/CLI/MCP surfaces
  5. Acceptance tests for knowledge retrieval
```

---

## File Manifest

### authority-domain (not blocked)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `authority-domain/src/knowledge.rs` | 100 | `KnowledgeRecord` — id, scope, source_type (catalog, file, decision, evidence), content_hash, indexed_at, citations. `KnowledgeQuery` — filters by scope, source_type, date range, text. `Citation` — source document ID, excerpt, confidence. |
| UPDATE | `authority-domain/src/lib.rs` | 5 | Export `knowledge` module. |

### Parquet crate (NEW — blocked on DEC-COUNCIL-003)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `parquet-store/Cargo.toml` | 20 | Dependencies: Parquet library (TBD by Council), chrono, serde, thiserror, tokio. |
| NEW | `parquet-store/src/lib.rs` | 60 | `ParquetDataset` — open, read, write dataset. `DatasetVersion` — versioning support. Schema definitions for knowledge, evidence, catalog snapshots. |
| NEW | `parquet-store/src/writer.rs` | 80 | `KnowledgeWriter` — batch write KnowledgeRecords. `EvidenceWriter` — batch write EvidenceRecords. `SchemaValidator` — validates records against schema. |
| NEW | `parquet-store/src/reader.rs` | 80 | `KnowledgeReader` — query by scope, source_type, date range, text. `DatasetReader` — stream records with pagination. |
| NEW | `parquet-store/src/schemas.rs` | 60 | Schema definitions: `knowledge_schema()`, `evidence_schema()`, `catalog_snapshot_schema()`, `work_path_schema()`. |

### control-store (not blocked)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-store/src/lib.rs` | 60 | `insert_knowledge_metadata()` — Postgres metadata for Parquet datasets. `query_knowledge_metadata()` — list available datasets, version info. `reconcile_parquet_index()` — compare Postgres index with Parquet datasets. |

### control-service (not blocked)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-service/src/knowledge_service.rs` | 120 | `ingest_knowledge(source, records)` — validate scope, write to Parquet, update Postgres index. `query_knowledge(query)` — search across Parquet datasets, apply grant scope filtering, return ranked results with citations. `get_dataset_info(dataset_id)` — version, size, scope, last indexed. |
| UPDATE | `control-service/src/lib.rs` | 5 | Declare `knowledge_service` module. |

### control-api (not blocked)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-api/src/routes/knowledge.rs` | 80 | `POST /v1/knowledge/ingest`, `GET /v1/knowledge/query`, `GET /v1/knowledge/datasets`, `GET /v1/knowledge/datasets/:id`. |
| EXTEND | `control-api/src/routes/mod.rs` | 5 | Register knowledge route group. |

### operator-cli (not blocked)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `operator-cli/src/commands/knowledge.rs` | 80 | `knowledge ingest`, `knowledge query`, `knowledge datasets`, `knowledge reconcile`. |
| EXTEND | `operator-cli/src/commands/mod.rs` | 5 | Register knowledge command module. |

### agent-mcp (not blocked)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `agent-mcp/src/tools/knowledge.rs` | 60 | `core_query_knowledge` — query with scope filter, returns ranked results with citations. `core_ingest_knowledge` — batch ingest with validation. |
| EXTEND | `agent-mcp/src/tools/mod.rs` | 5 | Register knowledge tool module. |

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
- [ ] Parquet dataset versioning works (write version N, read version N, list versions)
- [x] Postgres index reconciliation finds orphaned or missing datasets
- [x] API, CLI, and MCP surfaces all support knowledge query + ingest
- [x] `cargo test --workspace` passes with 0 failures
- [x] `cargo clippy --workspace -- -D warnings` passes

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-DATA-PARQUET` | Create and modify Parquet crate (blocked on Council) |
| `SGL-DATA-POSTGRES` | Add knowledge metadata to control-store |
| `SGL-BACKEND-SERVICE` | Add knowledge_service to control-service |
| `SGL-BACKEND-API` | Add knowledge route group to control-api |
| `SGL-BACKEND-CLI` | Add knowledge commands to operator-cli |
| `SGL-BACKEND-DOMAIN` | Add knowledge types to authority-domain |

---

## Dependencies

- Phase 3 complete (Catalogs and Work Paths — knowledge ingests from catalog records)
- Phase 4 complete (Agent Gateway — knowledge respects grant scopes in queries)
- `DEC-COUNCIL-003` resolved (Parquet library and query engine decision)
