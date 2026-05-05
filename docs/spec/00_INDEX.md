---
doc_id: DOC-SPEC-000
title: RealmForge Specification Index
status: draft
owner: tech-writer
reviewers: [pm, cto, domain-architect, security-architect, api-architect, data-architect, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: specification-system
work_path_ids: [WP-DOCS-000]
related_decision_ids: []
related_file_ids: [FILE-DOCS-CODEX-SETUP, FILE-DOCS-CODEX-SESSION, FILE-DOCS-CODEX-MAPPING, FILE-DOCS-CODEX-SKILL-INSTALL]
visual_node_ids: [VN-SPEC-INDEX]
visual_edge_ids: []
approval_state: pending
---

# RealmForge Specification Index

## Purpose

This folder is the canonical product and implementation specification for RealmForge. It defines the final product before buildout begins. Implementation agents must treat these docs as the source of truth whenever current code or older docs disagree.

The source vision input is `docs/realm_forge_ai_native_path_forward.md`.

## Required Reading Order

1. `docs/realm_forge_ai_native_path_forward.md`
2. `docs/spec/00_INDEX.md`
3. `docs/spec/01_PRODUCT_DEFINITION.md`
4. `docs/spec/02_FINAL_SYSTEM_MAP.md`
5. `docs/spec/03_UBIQUITOUS_LANGUAGE.md`
6. `docs/spec/04_METADATA_STANDARD.md`
7. `docs/spec/05_ORG_TENANT_APP_CATALOG_MODEL.md`
8. `docs/spec/06_AGENT_SKILL_GRANT_MODEL.md`
9. `docs/spec/07_AGENT_EXECUTION_GATEWAY.md`
10. Remaining docs by product area.

## Build Plan Directory

Detailed per-phase executable planning files live at `docs/plan/`. Each phase file provides:
- Workflow with step-by-step build sequence
- Complete file manifest (NEW, EXTEND, UPDATE, VERIFY)
- Rust type references and code sketches
- Completion gates with acceptance test IDs
- Required skill grants and dependencies

**Start with `docs/plan/README.md`** for the full dependency graph, decision lockout table, risk assessment, and phase file index.

## Implementation Rule

No code, migration, API, CLI command, MCP tool, runtime bundle, visual node, test, or watcher event may be created unless it is described here with stable IDs, ownership, metadata, acceptance tests, and permission rules.

## Document Map

| Doc | Owns | Primary visual node |
| --- | --- | --- |
| `01_PRODUCT_DEFINITION.md` | Final product, users, non-goals | `VN-PRODUCT-REALMFORGE` |
| `02_FINAL_SYSTEM_MAP.md` | Module graph and subsystem edges | `VN-SYSTEM-ECOSYSTEM` |
| `03_UBIQUITOUS_LANGUAGE.md` | Domain language and forbidden synonyms | `VN-DOMAIN-LANGUAGE` |
| `04_METADATA_STANDARD.md` | Metadata schemas and file registry contract | `VN-METADATA-STANDARD` |
| `05_ORG_TENANT_APP_CATALOG_MODEL.md` | Organization, tenant, app, catalog hierarchy | `VN-TENANCY-MODEL` |
| `06_AGENT_SKILL_GRANT_MODEL.md` | Agent power model and skill grants | `VN-SKILL-GRANT-MODEL` |
| `07_AGENT_EXECUTION_GATEWAY.md` | Agent command gateway and evidence flow | `VN-AGENT-GATEWAY` |
| `08_WORK_PATH_MODEL.md` | Work path schema | `VN-WORKPATH-MODEL` |
| `09_WORK_PATH_INVENTORY.md` | Canonical work paths | `VN-WORKPATH-INVENTORY` |
| `10_BOARDS_PRODUCT_SPEC.md` | Human command surface | `VN-MODULE-BOARDS` |
| `11_KNOWLEDGE_PRODUCT_SPEC.md` | Scoped knowledge and retrieval | `VN-MODULE-KNOWLEDGE` |
| `12_BUILD_WATCH_PRODUCT_SPEC.md` | Construction-time watcher | `VN-MODULE-BUILD-WATCH` |
| `13_RUNTIME_BUNDLE_PRODUCT_SPEC.md` | Signed governed runtime bundle | `VN-MODULE-RUNTIME-BUNDLE` |
| `14_LIVE_WATCH_PRODUCT_SPEC.md` | Production watcher and remediation proposals | `VN-MODULE-LIVE-WATCH` |
| `15_AUTHORITY_CORE_SPEC.md` | Authority, policy, audit, snapshot core | `VN-MODULE-AUTHORITY-CORE` |
| `16_DATA_CONTRACTS.md` | Postgres, Parquet, object store contracts | `VN-DATA-CONTRACTS` |
| `17_API_MCP_CLI_CONTRACTS.md` | External operation contracts | `VN-INTERFACE-CONTRACTS` |
| `18_SECURITY_AND_SEPARATION_OF_DUTIES.md` | Threat model and enforcement | `VN-SECURITY-SOD` |
| `19_FIRST_VERTICAL_LOGIN_MODULE.md` | First vertical slice | `VN-MODULE-LOGIN` |
| `20_IMPLEMENTATION_ROADMAP.md` | Sequenced buildout | `VN-ROADMAP` |
| `21_ACCEPTANCE_TEST_PLAN.md` | Required validation | `VN-ACCEPTANCE-TESTS` |
| `22_OPEN_DECISIONS.md` | Blocked decisions | `VN-OPEN-DECISIONS` |
| `23_FRONTEND_VERTICAL_SPEC.md` | Frontend stack and Boards vertical implementation contract | `VN-MODULE-BOARDS` |

## Doc Quality Gate

Before implementation starts, run a documentation review that confirms:

- Every spec doc has front matter matching `04_METADATA_STANDARD.md`.
- Every future file path is listed in the planned file registry.
- Every public operation has an API, MCP, or CLI contract.
- Every agent action maps to a skill grant, bounded command, audit event, evidence record, and rollback anchor.
- Every unresolved product or architecture choice is listed in `22_OPEN_DECISIONS.md`.
