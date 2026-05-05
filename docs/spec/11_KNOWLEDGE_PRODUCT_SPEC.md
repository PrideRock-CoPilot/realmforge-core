---
doc_id: DOC-SPEC-011
title: Knowledge Product Spec
status: draft
owner: data-architect
reviewers: [tech-writer, security-architect, backend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: knowledge
work_path_ids: [WP-KNOWLEDGE-001]
related_decision_ids: [DEC-COUNCIL-003]
related_file_ids: []
visual_node_ids: [VN-MODULE-KNOWLEDGE]
visual_edge_ids: []
approval_state: pending
---

# Knowledge Product Spec

## Responsibility

Knowledge provides scoped retrieval over docs, source inventory, work paths, catalogs, decisions, evidence, traces, and runtime signals. It must cite governed records and never widen an agent scope.

## Ingestion Sources

| Source ID | Source | Storage target | Scope rule |
| --- | --- | --- | --- |
| `SRC-DOCS` | Markdown docs | `knowledge_documents.parquet` | doc metadata |
| `SRC-FILES` | File registry | `files.parquet` | file grant scope |
| `SRC-WORKPATHS` | Work path graph | `work_paths.parquet`, `work_path_nodes.parquet`, `work_path_edges.parquet` | packet graph scope |
| `SRC-CATALOGS` | Global, tenant, app catalogs | `catalog_modules.parquet` | catalog sharing mode |
| `SRC-EVIDENCE` | Test and build evidence | `evidence.parquet` | app and packet scope |
| `SRC-WATCH` | Build and live watch events | `watch_events.parquet` | watcher and app scope |

## Retrieval Contract

Request fields: `actor_id`, `skill_grant_id`, `app_id`, `query`, `allowed_dataset_ids`, `max_records`, `citation_required`.

Response fields: `answer`, `citations`, `records_used`, `scope_applied`, `denied_record_count`, `evidence_quality`.

## Citation Rule

Every Knowledge answer used for implementation must cite at least one governed source record. Uncited implementation guidance is rejected by the Agent Gateway.

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-KNOWLEDGE-001` |
| Visual node IDs | `VN-MODULE-KNOWLEDGE` |
| Visual edge IDs | `VE-KNOWLEDGE-TO-CATALOGS` |
