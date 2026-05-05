---
doc_id: DOC-PLAN-INDEX
title: RealmForge Build Plan Index
status: draft
owner: pm
reviewers: [ceo, cto, qa, release-manager]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-DOCS-000, WP-CORE-001, WP-CATALOG-001, WP-SKILL-001, WP-GATEWAY-001, WP-WORKPATH-001, WP-BOARDS-001, WP-KNOWLEDGE-001, WP-BUILD-WATCH-001, WP-RUNTIME-BUNDLE-001, WP-LIVE-RUNTIME-001, WP-LIVE-WATCH-001, WP-LOGIN-001]
related_decision_ids: [DEC-COUNCIL-001, DEC-COUNCIL-002, DEC-COUNCIL-003, DEC-USER-004, DEC-USER-005, DEC-USER-006]
related_file_ids: [FILE-ROOT-CARGO, FILE-CRATE-DOMAIN-LIB, FILE-CRATE-DOMAIN-IDS, FILE-CRATE-DOMAIN-SCOPE, FILE-CRATE-DOMAIN-WORKPATH, FILE-CRATE-DOMAIN-CATALOG, FILE-CRATE-DOMAIN-GRANT, FILE-CRATE-POLICY-LIB, FILE-CRATE-POLICY-SOD, FILE-CRATE-AUDIT-LIB, FILE-CRATE-SNAPSHOT-LIB, FILE-CRATE-STORE-LIB, FILE-CRATE-SERVICE-LIB, FILE-CRATE-GATEWAY-LIB, FILE-CRATE-API-LIB, FILE-CRATE-MCP-LIB, FILE-CRATE-CLI-MAIN, FILE-CRATE-PARQUET-LIB, FILE-CRATE-BUNDLE-LIB, FILE-CRATE-BUILD-WATCH-LIB, FILE-CRATE-LIVE-WATCH-LIB, FILE-DB-001, FILE-DB-002, FILE-DB-003, FILE-DB-004, FILE-PARQUET-FILES, FILE-PARQUET-WORKPATHS, FILE-PARQUET-EVIDENCE, FILE-CATALOG-LOGIN-MODULE, FILE-CONTRACT-LOGIN, FILE-POLICY-LOGIN, FILE-HANDLER-LOGIN, FILE-WATCH-LOGIN]
visual_node_ids: [VN-ROADMAP, VN-MODULE-AUTHORITY-CORE, VN-MODULE-CATALOGS, VN-MODULE-SKILL-GRANTS, VN-MODULE-AGENT-GATEWAY, VN-MODULE-WORK-PATHS, VN-MODULE-BOARDS, VN-MODULE-KNOWLEDGE, VN-MODULE-BUILD-WATCH, VN-MODULE-RUNTIME-BUNDLE, VN-MODULE-LIVE-RUNTIME, VN-MODULE-LIVE-WATCH, VN-MODULE-LOGIN]
visual_edge_ids: [VE-PHASE-0-TO-1, VE-PHASE-1-TO-2, VE-PHASE-2-TO-3, VE-PHASE-3-TO-4, VE-PHASE-4-TO-5, VE-PHASE-5-TO-6, VE-PHASE-6-TO-7, VE-PHASE-7-TO-8, VE-PHASE-8-TO-9]
approval_state: pending
---

# RealmForge Build Plan Index

## Purpose

This directory contains the complete sequenced buildout plan for RealmForge. Each phase is a self-contained planning file that any agent, reviewer, or stakeholder can open and immediately understand: what is being built, why, in what order, with what files, and how it is validated.

The canonical spec roadmap lives at `docs/spec/20_IMPLEMENTATION_ROADMAP.md`. These plan files are the **executable companion** — they provide the implementation-level detail that drives each phase.

## How To Read These Plans

Start here. Read the dependency graph and decision lockout table below. Then open the phase file for the phase you care about.

## Phase Dependency Graph

```text
Phase 0: Documentation Certification
  └──► Phase 1: Workspace And Prerequisites
        └──► Phase 2: Authority Core
              ├──► Phase2-Foundation.md          — Domain, policy, audit, snapshot enrichment
              ├──► Phase2-ServiceLayer.md        — Control service crate
              ├──► Phase2-SessionCommand.md      — Session + command lifecycle engines
              ├──► Phase2-AuditSnapshot.md       — Audit trail, snapshot, rollback, work packets
              ├──► Phase2-Transport.md           — API + CLI + MCP surfaces
              └──► Phase2-Integration.md         — Integration tests + observability
                    ├──► Phase 3 (Catalogs & Work Paths)
                    ├──► Phase 4 (Gateway & Skill Grants)
                    │     └──► Phase 5 (Knowledge & Parquet)
                    │           └──► Phase 6 (Boards & Build Watch)
                    │                 └──► Phase 7 (Runtime Bundle & Live Runtime)
                    │                       └──► Phase 8 (Live Watch)
                    └──► Phase 9 (Login Vertical)
```

## Decision Lockout Boundaries

| Decision ID | Blocks | Required before | Current status |
| --- | --- | --- | --- |
| `DEC-COUNCIL-001` | Boards product surface, visual map rendering | Phase 6 start | `COUNCIL_DECISION_REQUIRED` |
| `DEC-COUNCIL-002` | Backend crate/package names | Phase 1 start | `ACCEPTED` (closed) |
| `DEC-COUNCIL-003` | Parquet writer, Knowledge, snapshots | Phase 5 start | `COUNCIL_DECISION_REQUIRED` |
| `DEC-USER-004` | Catalog modules beyond Login | Post-Phase 9 | `USER_APPROVAL_REQUIRED` |
| `DEC-USER-005` | Brand removal threshold in naming | Throughout | `USER_APPROVAL_REQUIRED` |
| `DEC-USER-006` | Live Watch auto-remediation authority | Phase 8 | `USER_APPROVAL_REQUIRED` |

## Risk Assessment

| Phase | Risk | Rationale |
| --- | --- | --- |
| Phase 0 | low | Pure documentation — no code changes |
| Phase 1 | medium | Cargo, Git, Postgres configuration; workspace flattening |
| Phase 2 (all subphases) | critical | Authority Core is the security foundation — errors cascade to all downstream phases |
| Phase 3 | high | Catalog and work path data model affects every module's storage |
| Phase 4 | critical | Gateway is the single agent mutation path — must be correct by construction |
| Phase 5 | high | Parquet engine decision blocked on Council; data consistency risk |
| Phase 6 | medium | Boards blocked on frontend Council decision; Build Watch is lower risk |
| Phase 7 | critical | Bundle signing governs what runs in production |
| Phase 8 | medium | Live Watch monitors only — no auto-remediation in first release |
| Phase 9 | high | First vertical slice touches every layer |

## Phase File Index

| File | Phase | Owner | Work paths |
| --- | --- | --- | --- |
| `Phase0.md` | 0: Documentation Certification | tech-writer | `WP-DOCS-000` |
| `Phase1.md` | 1: Workspace And Prerequisites | backend | — |
| `Phase2-Foundation.md` | 2a: Foundation Hardening | cto | `WP-CORE-001` |
| `Phase2-ServiceLayer.md` | 2b: Control Service Layer | cto | `WP-CORE-001` |
| `Phase2-SessionCommand.md` | 2c: Session & Command Lifecycle | cto | `WP-CORE-001` |
| `Phase2-AuditSnapshot.md` | 2d: Audit, Snapshot, Rollback, Work Packets | cto | `WP-CORE-001` |
| `Phase2-Transport.md` | 2e: API, CLI, MCP Surfaces | cto | `WP-CORE-001` |
| `Phase2-Integration.md` | 2f: Integration Tests & Observability | cto | `WP-CORE-001` |
| `Phase3.md` | 3: Catalogs And Work Paths | domain-architect | `WP-CATALOG-001`, `WP-WORKPATH-001` |
| `Phase4.md` | 4: Agent Gateway And Skill Grants | security-architect | `WP-SKILL-001`, `WP-GATEWAY-001` |
| `Phase5.md` | 5: Knowledge And Parquet Snapshots | data-architect | `WP-KNOWLEDGE-001` |
| `Phase6.md` | 6: Boards And Build Watch | frontend/backend | `WP-BOARDS-001`, `WP-BUILD-WATCH-001` |
| `Phase7.md` | 7: Runtime Bundle And Live Runtime | cto | `WP-RUNTIME-BUNDLE-001`, `WP-LIVE-RUNTIME-001` |
| `Phase8.md` | 8: Live Watch | backend | `WP-LIVE-WATCH-001` |
| `Phase9.md` | 9: Login Vertical | pm | `WP-LOGIN-001` |

## Convention For Every Phase File

Every phase file follows this structure:

```markdown
## Overview
- Phase ID, title, work paths, owner, risk, blockers
- Decision lockout resolution

## Workflow
- Step-by-step build sequence with crate dependency order

## File Manifest
- Every file to create (NEW), modify (EXTEND), update (UPDATE), or verify (VERIFY)
- Grouped by crate

## Completion Gates
- Every acceptance test, CLI command, and manual check required to close the phase
- Checkable boxes

## Required Skill Grants
- Which grants the implementor needs

## Dependencies
- What must be complete before this phase starts
