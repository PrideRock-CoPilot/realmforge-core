---
doc_id: DOC-SPEC-001
title: RealmForge Product Definition
status: draft
owner: pm
reviewers: [ceo, cto, biz-user, tech-writer]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: product
work_path_ids: [WP-DOCS-000]
related_decision_ids: [DEC-COUNCIL-002]
related_file_ids: []
visual_node_ids: [VN-PRODUCT-REALMFORGE]
visual_edge_ids: []
approval_state: pending
---

# RealmForge Product Definition

## Product Thesis

RealmForge is an AI-native software construction and runtime governance ecosystem. Humans describe, approve, inspect, and monitor applications through familiar visual surfaces. Agents operate through hard-scoped backend state, skill grants, work packets, catalogs, evidence records, and rollback anchors.

RealmForge does not place AI inside a human-only development workflow. It defines a workflow where AI execution is governed by state, not by prompt obedience.

## Final Product Modules

| Module ID | Product module | Responsibility | Work path ID | Visual node ID |
| --- | --- | --- | --- | --- |
| `MOD-AUTHORITY-CORE` | Authority Core | Identity, policy, bounded commands, audit, snapshots, rollback anchors | `WP-CORE-001` | `VN-MODULE-AUTHORITY-CORE` |
| `MOD-CATALOGS` | Catalogs | Global, tenant, and app module/resource catalogs | `WP-CATALOG-001` | `VN-MODULE-CATALOGS` |
| `MOD-SKILL-GRANTS` | Skill Grants | Zero-capability agents and hard grants | `WP-SKILL-001` | `VN-MODULE-SKILL-GRANTS` |
| `MOD-AGENT-GATEWAY` | Agent Gateway | Enforced agent operation gateway | `WP-GATEWAY-001` | `VN-MODULE-AGENT-GATEWAY` |
| `MOD-WORK-PATHS` | Work Paths | Structured executable planning graph | `WP-WORKPATH-001` | `VN-MODULE-WORK-PATHS` |
| `MOD-BOARDS` | Boards | Human planning, approval, status, and release command surface | `WP-BOARDS-001` | `VN-MODULE-BOARDS` |
| `MOD-KNOWLEDGE` | Knowledge | Scoped ingestion, indexing, retrieval, and citations | `WP-KNOWLEDGE-001` | `VN-MODULE-KNOWLEDGE` |
| `MOD-BUILD-WATCH` | Build Watch | Construction-time monitoring, violations, evidence, cost | `WP-BUILD-WATCH-001` | `VN-MODULE-BUILD-WATCH` |
| `MOD-RUNTIME-BUNDLE` | Runtime Bundle Builder | Signed governed app bundle creation | `WP-RUNTIME-BUNDLE-001` | `VN-MODULE-RUNTIME-BUNDLE` |
| `MOD-LIVE-RUNTIME` | Live Runtime | Governed execution of approved bundles | `WP-LIVE-RUNTIME-001` | `VN-MODULE-LIVE-RUNTIME` |
| `MOD-LIVE-WATCH` | Live Watch | Production monitoring and proposed remediation packets | `WP-LIVE-WATCH-001` | `VN-MODULE-LIVE-WATCH` |
| `MOD-COST` | Cost Ledger | Token, build, storage, runtime, and rework costs | `WP-COST-001` | `VN-MODULE-COST` |
| `MOD-VISUAL-MAP` | Visual Map Metadata Layer | Graph IDs, nodes, edges, layout contracts, map testability | `WP-VISUAL-MAP-001` | `VN-MODULE-VISUAL-MAP` |

## Target Users

| User | Needs | Primary modules |
| --- | --- | --- |
| Founder/operator | Build and monitor custom apps with confidence | Boards, Catalogs, Build Watch, Live Watch |
| Product owner | Express intent, review plans, approve releases | Boards, Work Paths, Knowledge |
| Security reviewer | Prove no agent can exceed scope | Authority Core, Skill Grants, Agent Gateway, Audit |
| Engineer | Implement from exact contracts | Work Paths, Knowledge, Data Contracts, API/MCP/CLI |
| Release manager | Ship and roll back governed bundles | Runtime Bundle, Live Runtime, Snapshot Ledger |
| Finance reviewer | Understand AI and runtime cost per app/work path | Cost Ledger, Build Watch, Live Watch |

## Locked Product Decisions

- Workspace target is one Rust workspace rooted at `E:\realmforge`.
- Backend internals use capability names, not product branding or `rf-*` prefixes, per `DEC-COUNCIL-002`.
- `RealmForge` appears in user-facing product surfaces and human docs.
- Postgres controls live app authority and governance.
- Parquet and content-addressed objects store buildout history, lookup, catalogs, scoped context, snapshots, evidence, and rollback data.
- Organizations can own multiple tenants.
- Global catalog is approved reusable inventory.
- Tenant catalogs control sharing across apps.
- App catalogs hold app-local state and snapshots.
- Agents start with zero capabilities.
- Capabilities are granted only through skill grants.
- Watch has two products: Build Watch and Live Watch.
- Live Watch proposes fixes; it does not execute remediations in the first governed release.

## Non-Goals For First Buildout

- No automatic production remediation.
- No runtime execution from arbitrary Parquet code.
- No agent direct database access.
- No prompt-only permission enforcement.
- No frontend stack commitment before Council decision `DEC-COUNCIL-001`.
