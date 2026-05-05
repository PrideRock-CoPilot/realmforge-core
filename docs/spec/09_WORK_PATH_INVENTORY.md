---
doc_id: DOC-SPEC-009
title: Work Path Inventory
status: draft
owner: pm
reviewers: [cto, domain-architect, qa, release-manager]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: work-paths
work_path_ids: [WP-WORKPATH-001]
related_decision_ids: []
related_file_ids: []
visual_node_ids: [VN-WORKPATH-INVENTORY]
visual_edge_ids: []
approval_state: pending
---

# Work Path Inventory

## Canonical Work Paths

| Work path ID | Name | Owner | Product module | Required grants | Required tests |
| --- | --- | --- | --- | --- | --- |
| `WP-DOCS-000` | Docs-first specification package | tech-writer | specification-system | `SGL-TECH-WRITER` | `TEST-DOCS-METADATA-001` |
| `WP-CORE-001` | Authority Core hardening | cto | Authority Core | `SGL-BACKEND-DOMAIN`, `SGL-BACKEND-POLICY`, `SGL-BACKEND-SERVICE` | `TEST-CORE-AUTHORITY-001` |
| `WP-CATALOG-001` | Global tenant app catalogs | domain-architect | Catalogs | `SGL-BACKEND-DOMAIN`, `SGL-DATA-POSTGRES` | `TEST-CATALOG-001` |
| `WP-SKILL-001` | Zero-capability skill grants | security-architect | Skill Grants | `SGL-BACKEND-POLICY`, `SGL-DATA-POSTGRES` | `TEST-SKILL-GRANT-001` |
| `WP-GATEWAY-001` | Agent execution gateway | security-architect | Agent Gateway | `SGL-BACKEND-GATEWAY`, `SGL-BACKEND-POLICY` | `TEST-GATEWAY-001` |
| `WP-WORKPATH-001` | Work path graph engine | domain-architect | Work Paths | `SGL-BACKEND-DOMAIN`, `SGL-DATA-PARQUET` | `TEST-WORKPATH-001` |
| `WP-BOARDS-001` | Boards product surface | frontend | Boards | `SGL-FRONTEND-SHELL` | `TEST-BOARDS-001` |
| `WP-KNOWLEDGE-001` | Scoped knowledge system | data-architect | Knowledge | `SGL-DATA-PARQUET`, `SGL-BACKEND-SERVICE` | `TEST-KNOWLEDGE-001` |
| `WP-BUILD-WATCH-001` | Build Watch construction monitor | backend | Build Watch | `SGL-BACKEND-WATCH` | `TEST-BUILD-WATCH-001` |
| `WP-RUNTIME-BUNDLE-001` | Signed runtime bundle builder | cto | Runtime Bundle | `SGL-BACKEND-RUNTIME`, `SGL-RELEASE` | `TEST-BUNDLE-001` |
| `WP-LIVE-RUNTIME-001` | Governed live runtime | cto | Live Runtime | `SGL-BACKEND-RUNTIME` | `TEST-LIVE-RUNTIME-001` |
| `WP-LIVE-WATCH-001` | Live Watch monitor | backend | Live Watch | `SGL-BACKEND-WATCH` | `TEST-LIVE-WATCH-001` |
| `WP-LOGIN-001` | First vertical Login module | pm | Login Module | `SGL-BACKEND-SERVICE`, `SGL-FRONTEND-SHELL`, `SGL-QA-VERIFY` | `TEST-LOGIN-E2E-001` |

## Login Work Path Nodes

| Node ID | Type | Name | Files | Tests | Traces |
| --- | --- | --- | --- | --- | --- |
| `WPN-LOGIN-MODULE` | module | Login module | `FILE-CATALOG-LOGIN-MODULE` | `TEST-LOGIN-MODULE-001` | `TRACE-LOGIN-MODULE-001` |
| `WPN-LOGIN-CONTRACTS` | runtime_contract | Login request/response contracts | `FILE-CONTRACT-LOGIN` | `TEST-LOGIN-CONTRACT-001` | `TRACE-LOGIN-CONTRACT-001` |
| `WPN-LOGIN-AUTH-POLICY` | policy | Login policy | `FILE-POLICY-LOGIN` | `TEST-LOGIN-POLICY-001` | `TRACE-LOGIN-POLICY-001` |
| `WPN-LOGIN-HANDLER` | service | Login handler binding | `FILE-HANDLER-LOGIN` | `TEST-LOGIN-HANDLER-001` | `TRACE-LOGIN-HANDLER-001` |
| `WPN-LOGIN-WATCH` | watch_signal | Login watch profile | `FILE-WATCH-LOGIN` | `TEST-LOGIN-WATCH-001` | `TRACE-LOGIN-WATCH-001` |
