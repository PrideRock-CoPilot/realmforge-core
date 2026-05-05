---
doc_id: DOC-SPEC-020
title: Implementation Roadmap
status: draft
owner: pm
reviewers: [ceo, cto, qa, release-manager]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-DOCS-000, WP-CORE-001, WP-CATALOG-001, WP-SKILL-001, WP-GATEWAY-001, WP-WORKPATH-001, WP-BOARDS-001, WP-KNOWLEDGE-001, WP-BUILD-WATCH-001, WP-RUNTIME-BUNDLE-001, WP-LIVE-RUNTIME-001, WP-LIVE-WATCH-001, WP-LOGIN-001]
related_decision_ids: [DEC-COUNCIL-001, DEC-COUNCIL-002, DEC-COUNCIL-003, DEC-USER-004, DEC-USER-005, DEC-USER-006]
related_file_ids: [FILE-ROOT-CARGO, FILE-CRATE-DOMAIN-LIB, FILE-CRATE-DOMAIN-IDS, FILE-CRATE-DOMAIN-SCOPE, FILE-CRATE-DOMAIN-WORKPATH, FILE-CRATE-DOMAIN-CATALOG, FILE-CRATE-DOMAIN-GRANT, FILE-CRATE-POLICY-LIB, FILE-CRATE-POLICY-SOD, FILE-CRATE-AUDIT-LIB, FILE-CRATE-SNAPSHOT-LIB, FILE-CRATE-STORE-LIB, FILE-CRATE-SERVICE-LIB, FILE-CRATE-GATEWAY-LIB, FILE-CRATE-API-LIB, FILE-CRATE-MCP-LIB, FILE-CRATE-CLI-MAIN, FILE-CRATE-PARQUET-LIB, FILE-CRATE-BUNDLE-LIB, FILE-CRATE-BUILD-WATCH-LIB, FILE-CRATE-LIVE-WATCH-LIB, FILE-DB-001, FILE-DB-002, FILE-DB-003, FILE-DB-004, FILE-PARQUET-FILES, FILE-PARQUET-WORKPATHS, FILE-PARQUET-EVIDENCE, FILE-CATALOG-LOGIN-MODULE, FILE-CONTRACT-LOGIN, FILE-POLICY-LOGIN, FILE-HANDLER-LOGIN, FILE-WATCH-LOGIN]
visual_node_ids: [VN-ROADMAP, VN-MODULE-AUTHORITY-CORE, VN-MODULE-CATALOGS, VN-MODULE-SKILL-GRANTS, VN-MODULE-AGENT-GATEWAY, VN-MODULE-WORK-PATHS, VN-MODULE-BOARDS, VN-MODULE-KNOWLEDGE, VN-MODULE-BUILD-WATCH, VN-MODULE-RUNTIME-BUNDLE, VN-MODULE-LIVE-RUNTIME, VN-MODULE-LIVE-WATCH, VN-MODULE-LOGIN]
visual_edge_ids: [VE-PHASE-0-TO-1, VE-PHASE-1-TO-2, VE-PHASE-2-TO-3, VE-PHASE-3-TO-4, VE-PHASE-4-TO-5, VE-PHASE-5-TO-6, VE-PHASE-6-TO-7, VE-PHASE-7-TO-8, VE-PHASE-8-TO-9]
approval_state: pending
---

# Implementation Roadmap

## Purpose

This document is the **master index** for the RealmForge buildout sequence. Detailed per-phase planning files live in `docs/plan/`. This page provides the dependency graph, decision lockout table, risk assessment, and phase-level pointers for quick navigation.

**Every agent, stakeholder, or reviewer should start with the plan files at `docs/plan/README.md`** before examining implementation code.

---

## Phase Dependency Graph

```text
Phase 0: Documentation Certification
  └──► Phase 1: Workspace And Prerequisites
        └──► Phase 2: Authority Core
              ├──► Phase 3: Catalogs And Work Paths
              ├──► Phase 4: Agent Gateway And Skill Grants
              │     └──► Phase 5: Knowledge And Parquet Snapshots
              │           └──► Phase 6: Boards And Build Watch
              │                 └──► Phase 7: Runtime Bundle And Live Runtime
              │                       └──► Phase 8: Live Watch
              └──► Phase 9: Login Vertical
```

---

## Decision Lockout Boundaries

| Decision ID | Blocks | Required before | Status |
|-------------|--------|-----------------|--------|
| `DEC-COUNCIL-001` | Boards product surface | Phase 6 | `ACCEPTED` (closed 2026-05-04 — ADR-0002) |
| `DEC-COUNCIL-002` | Backend crate names | Phase 1 | `ACCEPTED` (closed) |
| `DEC-COUNCIL-003` | Parquet library/engine | Phase 5 | `ACCEPTED` (closed 2026-05-05 — ADR-0006) |
| `DEC-USER-004` | Catalog modules beyond Login | Post-Phase 9 | `USER_APPROVED` (closed 2026-05-05) |
| `DEC-USER-005` | Brand removal threshold | Throughout | `USER_APPROVED` (closed 2026-05-04) |
| `DEC-USER-006` | Live Watch auto-remediation | Phase 8 | `USER_APPROVED` (closed 2026-05-05 — ADR-0007) |

---

## Phase Index

| Phase | ID | Owner | Risk | Plan file |
|-------|----|-------|------|-----------|
| 0 | Documentation Certification | tech-writer | low | `docs/plan/Phase0.md` |
| 1 | Workspace And Prerequisites | backend | medium | `docs/plan/Phase1.md` |
| 2a | Foundation Hardening | cto | critical | `docs/plan/Phase2-Foundation.md` |
| 2b | Control Service Layer | cto | critical | `docs/plan/Phase2-ServiceLayer.md` |
| 2c | Session & Command Lifecycle | cto | critical | `docs/plan/Phase2-SessionCommand.md` |
| 2d | Audit, Snapshot, Rollback, Work Packets | cto | critical | `docs/plan/Phase2-AuditSnapshot.md` |
| 2e | API, CLI & MCP Surfaces | cto | critical | `docs/plan/Phase2-Transport.md` |
| 2f | Integration Tests & Observability | cto | high | `docs/plan/Phase2-Integration.md` |
| 3 | Catalogs And Work Paths | domain-architect | high | `docs/plan/Phase3.md` |
| 4 | Agent Gateway And Skill Grants | security-architect | critical | `docs/plan/Phase4.md` |
| 5 | Knowledge And Parquet Snapshots | data-architect | high | `docs/plan/Phase5.md` |
| 6 | Boards And Build Watch | frontend/backend | medium | `docs/plan/Phase6.md` |
| 7 | Runtime Bundle And Live Runtime | cto | critical | `docs/plan/Phase7.md` |
| 8 | Live Watch | backend | medium | `docs/plan/Phase8.md` |
| 9 | Login Vertical | pm | high | `docs/plan/Phase9.md` |

---

## Phase Order Rationale

1. **Documentation first (Phase 0).** Docs-First Law — no implementation without approved specs.
2. **Workspace (Phase 1).** Every crate depends on workspace compilation and Postgres access.
3. **Authority Core (Phase 2).** Policy, audit, snapshot, rollback are foundational. Phase 2 is subdivided into 6 subphases for file-size discipline.
4. **Catalogs parallel to Grants (Phase 3 & Phase 4).** Independent of each other, both depend on Phase 2.
5. **Knowledge after both (Phase 5).** Needs catalogs (sources), gateway (scoped access), and the accepted Arrow/DataFusion Parquet path from `DEC-COUNCIL-003`.
6. **Boards & Build Watch (Phase 6).** Watch needs evidence storage. Boards use the accepted React/TypeScript frontend stack from `DEC-COUNCIL-001`.
7. **Runtime & Live Runtime (Phase 7).** Bundles package everything into signed deployables.
8. **Live Watch (Phase 8).** Monitors deployed bundles. No auto-remediation in first release.
9. **Login vertical (Phase 9).** Integration proof — every layer end-to-end.

---

## Required Reading Order

1. `docs/plan/README.md` — Build plan index with dependency graph
2. `docs/plan/Phase{0-9}.md` — The specific phase you are implementing
3. `docs/spec/21_ACCEPTANCE_TEST_PLAN.md` — Required acceptance tests
4. `docs/MASTER_BUILD_PLAN.md` — Reference architecture and file manifests

---

## Traceability

| Trace field | IDs |
|-------------|-----|
| Work path IDs | `WP-DOCS-000`, `WP-CORE-001`, `WP-CATALOG-001`, `WP-SKILL-001`, `WP-GATEWAY-001`, `WP-WORKPATH-001`, `WP-BOARDS-001`, `WP-KNOWLEDGE-001`, `WP-BUILD-WATCH-001`, `WP-RUNTIME-BUNDLE-001`, `WP-LIVE-RUNTIME-001`, `WP-LIVE-WATCH-001`, `WP-LOGIN-001` |
| Visual node IDs | `VN-ROADMAP`, `VN-MODULE-AUTHORITY-CORE`, `VN-MODULE-CATALOGS`, `VN-MODULE-SKILL-GRANTS`, `VN-MODULE-AGENT-GATEWAY`, `VN-MODULE-WORK-PATHS`, `VN-MODULE-BOARDS`, `VN-MODULE-KNOWLEDGE`, `VN-MODULE-BUILD-WATCH`, `VN-MODULE-RUNTIME-BUNDLE`, `VN-MODULE-LIVE-RUNTIME`, `VN-MODULE-LIVE-WATCH`, `VN-MODULE-LOGIN` |
| Visual edge IDs | `VE-PHASE-0-TO-1`, `VE-PHASE-1-TO-2`, `VE-PHASE-2-TO-3`, `VE-PHASE-3-TO-4`, `VE-PHASE-4-TO-5`, `VE-PHASE-5-TO-6`, `VE-PHASE-6-TO-7`, `VE-PHASE-7-TO-8`, `VE-PHASE-8-TO-9` |
