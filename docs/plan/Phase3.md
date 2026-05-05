---
doc_id: DOC-PLAN-P3
title: Phase 3 — Catalogs And Work Paths
parent: DOC-PLAN-INDEX
status: draft
owner: domain-architect
reviewers: [cto, data-architect, backend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-CATALOG-001, WP-WORKPATH-001]
related_decision_ids: []
related_file_ids: [FILE-CRATE-DOMAIN-CATALOG, FILE-CRATE-DOMAIN-WORKPATH, FILE-CRATE-API-LIB, FILE-CRATE-CLI-MAIN, FILE-CRATE-MCP-LIB, FILE-CRATE-SERVICE-LIB, FILE-CRATE-STORE-LIB]
visual_node_ids: [VN-MODULE-CATALOGS, VN-MODULE-WORK-PATHS]
approval_state: pending
---

# Phase 3: Catalogs And Work Paths

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 3 |
| Title | Catalogs And Work Paths |
| Work paths | `WP-CATALOG-001`, `WP-WORKPATH-001` |
| Product module | Catalogs, Work Paths |
| Owner | domain-architect (Dr. Yusuf Osman) |
| Risk | high |
| Decision blockers | none |

**Mandate:** Build the catalog system (global → tenant → app hierarchy) and the work path graph engine. Catalogs define reusable inventory. Work paths define structured executable planning graphs that produce scoped agent work packets.

---

## Workflow

```
Crate dependency order:
  1. authority-domain — add CatalogEntry, CatalogNode, WorkPathNode types
  2. control-store — add catalog and work path persistence
  3. control-service — add catalog_service and work_path_service modules
  4. control-api — add catalog and work path route groups
  5. operator-cli — add catalog and work-path commands
  6. agent-mcp — add catalog and work_path tools

Implementation order per crate:
  a. Domain types first (catalog hierarchy, work path graph)
  b. Store adapter queries next
  c. Service orchestration next
  d. Thin transports last
```

---

## File Manifest

### authority-domain (types only)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `authority-domain/src/catalog.rs` | 100 | `CatalogEntry` enum — GlobalModule, TenantModule, AppModule. `CatalogNode` — scoped node with parents/children. `CatalogCopyProvenance` — tracks source global→tenant→app copy lineage. |
| NEW | `authority-domain/src/work_path.rs` | 120 | `WorkPathNode` — type (module, runtime_contract, policy, service, watch_signal, data_contract, evidence), file links, contract links, test links, trace links. `WorkPathGraph` — ordered DAG of nodes. `PacketScope` — derived from node traversal. |
| UPDATE | `authority-domain/src/lib.rs` | 10 | Export `catalog` and `work_path` modules. |

### control-store

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-store/src/lib.rs` | 100 | `insert_catalog_entry()`, `get_catalog_entry()`, `list_catalog_entries(scope)`, `copy_catalog_entry()` with provenance. `insert_work_path_node()`, `get_work_path_graph()` — fetch full DAG for a work path. |

### control-service

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-service/src/catalog_service.rs` | 100 | `copy_module(from_scope, to_scope, module_id)` — validates source exists, copies with provenance. `list_modules(scope)`, `get_module_provenance(module_id)`. |
| NEW | `control-service/src/work_path_service.rs` | 120 | `create_work_path()`, `add_node()`, `link_node_to_file()`, `get_work_path_graph()`, `traverse_to_packet()` — walks graph and produces scoped `AgentWorkPacket` (uses Phase 2 work_packet types). |
| UPDATE | `control-service/src/lib.rs` | 10 | Declare `catalog_service` and `work_path_service` modules. |

### control-api

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-api/src/routes/catalog.rs` | 80 | `GET /v1/catalog`, `POST /v1/catalog/copy`, `GET /v1/catalog/:id/provenance`. |
| NEW | `control-api/src/routes/work_path.rs` | 80 | `POST /v1/work-paths`, `GET /v1/work-paths/:id`, `POST /v1/work-paths/:id/traverse`. |
| EXTEND | `control-api/src/routes/mod.rs` | 10 | Register catalog and work_path route groups. |

### operator-cli

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `operator-cli/src/commands/catalog.rs` | 80 | `catalog list`, `catalog copy`, `catalog provenance`. |
| NEW (extend existing) | `operator-cli/src/commands/work_path.rs` | 80 | `work-path create`, `work-path add-node`, `work-path traverse`. |
| EXTEND | `operator-cli/src/commands/mod.rs` | 10 | Register catalog and work_path command modules. |

### agent-mcp

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `agent-mcp/src/tools/catalog.rs` | 60 | `core_copy_catalog_module`, `core_list_catalog_modules`. |
| NEW (extend existing) | `agent-mcp/src/tools/work_packet.rs` | 40 | Extend existing work_packet tool to accept work path node ID. |
| EXTEND | `agent-mcp/src/tools/mod.rs` | 10 | Register catalog tool module. |

---

## Key Types

### CatalogEntry (NEW)
```rust
pub enum CatalogScope { Global, Tenant(TenantId), App(AppId) }

pub struct CatalogEntry {
    pub id: CatalogId,
    pub name: String,
    pub scope: CatalogScope,
    pub parent_id: Option<CatalogId>,
    pub module_type: CatalogModuleType,  // Module, Contract, Policy, Handler, Watch
    pub provenance: Option<CatalogCopyProvenance>,
    pub created_at: DateTime<Utc>,
}
```

### WorkPathNode (NEW)
```rust
pub enum WorkPathNodeType {
    Module, RuntimeContract, Policy, Service, WatchSignal, DataContract, Evidence,
}

pub struct WorkPathNode {
    pub id: WorkPathNodeId,
    pub work_path_id: WorkPathId,
    pub node_type: WorkPathNodeType,
    pub name: String,
    pub file_ids: Vec<String>,
    pub contract_ids: Vec<String>,
    pub test_ids: Vec<String>,
    pub trace_point_ids: Vec<String>,
    pub children: Vec<WorkPathNodeId>,
}
```

---

## Completion Gates

- [ ] `TEST-CATALOG-001` — Tenant copies approved global module with provenance recorded
- [ ] `TEST-WORKPATH-001` — Work path node generates scoped packet with correct boundaries
- [ ] Catalog traversal returns correct hierarchy (global → tenant → app)
- [ ] Work path DAG correctly links files, contracts, and tests
- [ ] Packet generated from work path node has correct file scope
- [ ] All CRUD operations work via API, CLI, and MCP
- [ ] `cargo test --workspace` passes with 0 failures
- [ ] `cargo clippy --workspace -- -D warnings` passes

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-DOMAIN` | Modify authority-domain (add catalog.rs, work_path.rs) |
| `SGL-BACKEND-SERVICE` | Add catalog_service, work_path_service to control-service |
| `SGL-BACKEND-API` | Add route groups to control-api |
| `SGL-BACKEND-CLI` | Add command modules to operator-cli |
| `SGL-DATA-POSTGRES` | Add catalog and work path persistence to control-store |

---

## Dependencies

- Phase 2 complete (Authority Core — domain types, service layer, store, transports all operational)
