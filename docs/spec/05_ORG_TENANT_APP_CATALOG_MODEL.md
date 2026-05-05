---
doc_id: DOC-SPEC-005
title: Organization Tenant App Catalog Model
status: draft
owner: domain-architect
reviewers: [cto, data-architect, security-architect, pm]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: catalogs
work_path_ids: [WP-CATALOG-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-TENANCY-MODEL]
visual_edge_ids: []
approval_state: pending
---

# Organization Tenant App Catalog Model

## Hierarchy

```text
organization
  -> tenant
    -> app
      -> app catalog
    -> tenant catalog
global catalog
  -> approved reusable modules
```

## Governance Rules

- Organization owns billing, operator identity, and tenant membership.
- Tenant owns sharing rules, skill grants, tenant catalog, and app access.
- App owns app catalog, work paths, file registry, snapshots, bundles, evidence, and watchers.
- Global catalog is read-only to tenants. Tenants copy or fork approved modules into tenant/app catalogs.
- A module copied from global catalog must keep provenance: source module ID, source version, copy actor, copy time, tests, policies, traces, and approval evidence.

## Catalog Entity Contracts

| Entity | Required fields |
| --- | --- |
| Organization | `organization_id`, `display_name`, `status`, `created_at` |
| Tenant | `tenant_id`, `organization_id`, `display_name`, `sharing_mode`, `status`, `created_at` |
| App | `app_id`, `tenant_id`, `display_name`, `app_mode`, `status`, `created_at` |
| Catalog | `catalog_id`, `owner_kind`, `owner_id`, `catalog_kind`, `status`, `created_at` |
| Catalog Module | `module_id`, `catalog_id`, `name`, `module_kind`, `version`, `source_module_id`, `approval_state`, `risk_level` |
| Module Contract Link | `module_id`, `contract_id`, `direction`, `required` |
| Module File Link | `module_id`, `file_id`, `relationship`, `required` |
| Module Test Link | `module_id`, `test_id`, `required` |
| Module Trace Link | `module_id`, `trace_point_id`, `required` |

## Catalog Sharing Modes

| Mode | Meaning |
| --- | --- |
| `isolated` | Apps cannot share modules unless copied from global catalog or user-approved tenant copy. |
| `tenant_shared` | Apps can share tenant catalog modules approved for reuse. |
| `restricted_shared` | Apps can share only modules tagged with explicit app allow-list. |

## Visual Nodes

| Node ID | Label | Type |
| --- | --- | --- |
| `VN-ORG` | Organization | governance_container |
| `VN-TENANT` | Tenant | governance_boundary |
| `VN-APP` | App | governed_application |
| `VN-GLOBAL-CATALOG` | Global Catalog | reusable_inventory |
| `VN-TENANT-CATALOG` | Tenant Catalog | sharing_catalog |
| `VN-APP-CATALOG` | App Catalog | app_catalog |

## Traceability

| Trace field | IDs |
| --- | --- |
| Work path IDs | `WP-CATALOG-001` |
| Visual node IDs | `VN-TENANCY-MODEL`, `VN-MODULE-CATALOGS` |
| Visual edge IDs | `VE-ORG-OWNS-TENANT`, `VE-TENANT-OWNS-APP`, `VE-CATALOG-COPIES-GLOBAL-MODULE` |
