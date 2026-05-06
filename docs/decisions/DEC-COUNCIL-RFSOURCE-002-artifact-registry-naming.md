---
doc_id: DOC-DECISION-RFSOURCE-002
title: "Council Decision: Artifact Registry Naming"
status: accepted
owner: council
reviewers: [cto, pm, domain-architect, api-architect, tech-writer]
created_at: 2026-05-06
last_reviewed_at: 2026-05-06
source_of_truth: true
product_area: architecture
work_path_ids: [WP-RFSOURCE-001]
related_decision_ids: [DEC-COUNCIL-RFSOURCE-001]
related_file_ids: [DOC-SPEC-022]
visual_node_ids: [VN-RFSOURCE-DECISIONS]
visual_edge_ids: []
approval_state: accepted
---

# Council Decision: Artifact Registry Naming

## Decision ID
`DEC-COUNCIL-RFSOURCE-002`

## Status
**ACCEPTED** — DRI: Dr. Rena Okafor (CTO), Council convened 2026-05-06.

## Context

The system that manages source artifact metadata — previously conceptualized as "Forge Catalog" or "Unity Catalog" — needed a finalized name. This is the indexing layer in PostgreSQL that tracks what artifacts exist, their versions, grants, policies, and metadata. It is the bridge between `.rfsource` files and the governance system.

## Decision

**Name: "Artifact Registry"** — the PostgreSQL-backed metadata layer that indexes artifacts stored in `.rfsource` files.

### Naming Convention

| Domain concept | Name | Example |
|----------------|------|---------|
| PostgreSQL schema | `artifact_registry` | `artifact_registry.artifacts` |
| Rust module | `artifact_registry` | `rfsource-catalog/src/artifact_registry/mod.rs` |
| API routes | `/v1/artifacts/*` | `GET /v1/artifacts/:id` |
| SQL table prefix | `ar_` | `ar_artifacts`, `ar_versions`, `ar_grants` |
| MCP tools | `artifact_registry_*` | `artifact_registry_list`, `artifact_registry_get` |

## Rationale

1. **Descriptive**: "Artifact Registry" clearly communicates what it is — a registry of artifacts. No confusion with data catalogs, source catalogs, or Unity Catalog.
2. **Distinct from .rfsource**: The Registry is metadata; `.rfsource` is data. The naming makes this distinction clear.
3. **API-friendly**: `/v1/artifacts/*` is intuitive and RESTful.
4. **Industry familiarity**: "Registry" is well-understood in developer tooling (container registries, package registries, schema registries).

## Consequences

1. All PostgreSQL table names use the `ar_` prefix.
2. All Rust code modules use `artifact_registry` naming.
3. API routes use `/v1/artifacts/` path prefix.
4. No brand prefix in names (per DEC-USER-005).
