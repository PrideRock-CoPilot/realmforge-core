---
doc_id: DOC-SPEC-016
title: Data Contracts
status: draft
owner: data-architect
reviewers: [infra-architect, backend, security-architect, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: data
work_path_ids: [WP-DATA-001]
related_decision_ids: [DEC-COUNCIL-003]
related_file_ids: []
visual_node_ids: [VN-DATA-CONTRACTS]
visual_edge_ids: []
approval_state: pending
---

# Data Contracts

## Postgres Control Tables

Required table groups:

- `organizations`, `tenants`, `apps`
- `catalogs`, `catalog_modules`, `module_file_links`, `module_contract_links`, `module_test_links`, `module_trace_links`
- `actors`, `sessions`, `skills`, `skill_grants`, `grant_tool_permissions`, `grant_file_permissions`, `grant_schema_permissions`
- `work_paths`, `work_path_nodes`, `work_path_edges`
- `work_packets`, `packet_files`, `packet_tests`, `packet_traces`, `packet_evidence`
- `bounded_commands`, `audit_events`
- `snapshots`, `snapshot_object_refs`, `snapshot_table_exports`
- `runtime_bundles`, `bundle_artifacts`
- `build_watch_events`, `live_watch_signals`
- `cost_events`

## Agent Views

Agents receive Postgres access through views only:

- `agent_visible_packets_v`
- `agent_visible_files_v`
- `agent_visible_work_path_nodes_v`
- `agent_visible_catalog_modules_v`
- `agent_visible_evidence_v`

Every view filters by active `skill_grant_id`, `agent_id`, `tenant_id`, `app_id`, packet state, and grant expiry.

## Parquet Datasets

| Dataset ID | File | Partition keys |
| --- | --- | --- |
| `PDS-FILES` | `files.parquet` | `tenant_id`, `app_id`, `snapshot_id` |
| `PDS-WORK-PATHS` | `work_paths.parquet` | `tenant_id`, `app_id`, `snapshot_id` |
| `PDS-WORK-PATH-NODES` | `work_path_nodes.parquet` | `tenant_id`, `app_id`, `snapshot_id` |
| `PDS-WORK-PATH-EDGES` | `work_path_edges.parquet` | `tenant_id`, `app_id`, `snapshot_id` |
| `PDS-CATALOG-MODULES` | `catalog_modules.parquet` | `tenant_id`, `catalog_id`, `snapshot_id` |
| `PDS-EVIDENCE` | `evidence.parquet` | `tenant_id`, `app_id`, `snapshot_id` |
| `PDS-TRACE-POINTS` | `trace_points.parquet` | `tenant_id`, `app_id`, `snapshot_id` |
| `PDS-COSTS` | `cost_events.parquet` | `tenant_id`, `app_id`, `snapshot_id` |

## Object Store Paths

` .realmforge/objects/sha256/{first_two}/{next_two}/{sha256}` is the required object path shape.
