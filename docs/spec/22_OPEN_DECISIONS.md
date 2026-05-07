---
doc_id: DOC-SPEC-022
title: Open Decisions
status: draft
owner: council
reviewers: [ceo, pm, cto, tech-writer]
created_at: 2026-05-04
last_reviewed_at: 2026-05-06
closed_decisions: [DEC-COUNCIL-001, DEC-COUNCIL-003, DEC-COUNCIL-RFSOURCE-001, DEC-COUNCIL-RFSOURCE-002, DEC-COUNCIL-RFSOURCE-003, DEC-COUNCIL-CODE-REVIEW-001, DEC-COUNCIL-INTAKE-001, DEC-COUNCIL-WFE-001, DEC-USER-004, DEC-USER-005, DEC-USER-006, DEC-USER-RFSOURCE-001]

source_of_truth: true
product_area: decisions
work_path_ids: [WP-DOCS-000, WP-RFSOURCE-001]
related_decision_ids: [DEC-COUNCIL-RFSOURCE-001, DEC-COUNCIL-RFSOURCE-002, DEC-USER-RFSOURCE-001]
related_file_ids: []
visual_node_ids: [VN-OPEN-DECISIONS]
visual_edge_ids: []
approval_state: pending
---

# Open Decisions

This is the only spec file where unresolved decisions may appear.

## Council Decisions

| Decision ID | Marker | Question | Blocked area | Required participants |
| --- | --- | --- | --- | --- |

## User Approval Decisions

| Decision ID | Marker | Question | Blocked area |
| --- | --- | --- | --- |

## Closed Decisions

| Decision ID | Decision | Resolution | Date Closed | Linked Spec |
| --- | --- | --- | --- | --- |
| `DEC-COUNCIL-001` | React + TypeScript frontend stack, web-first shell | React (Vite + React 19 + TypeScript), shadcn/ui (internal) + ui/ package (public), React Flow for graph rendering, OpenAPI-generated TypeScript client. No Tauri until specific capability requires it with Fatima threat model review. | 2026-05-04 | `ADR-0002` |
| `DEC-COUNCIL-003` | Which Parquet library and query engine? | Apache Arrow + DataFusion (`datafusion` crate from `apache/arrow-rs`). Pure Rust, no C/FFI bindings. Same ecosystem as existing `parquet` crate. New `parquet-store` crate. | 2026-05-05 | `ADR-0006` |
| `DEC-COUNCIL-CODE-REVIEW-001` | Automated Code Review Module Architecture | **Adopted with conditions** — Three-axis taxonomy (language x framework x project config), dedicated `crates/code-review/` engine crate, `code-review-standards/` YAML data directory. Framework detection via explicit `.code-review.yaml` only (no auto-detection from extensions or manifests). Phase 1: Rust + crate engine. Phase 2: Python. Phase 3+: TypeScript, frameworks per project adoption. | 2026-05-06 | `DEC-COUNCIL-CODE-REVIEW-001` |
| `DEC-COUNCIL-INTAKE-001` | Intake Engine Architecture | **Adopted with conditions** — Structured deterministic intake engine (95% rule-driven, <5% AI-assisted). Decision tree format: data-driven JSON (no DSL). Max tree depth: 5. AI observation promotion requires admin + security review for module/capability-affecting questions. Phased rollout: 3 types → 5 more → remaining 7 (10 weeks). Crate placement: `intake-engine` (pure), `intake-store` in `control-store`, observations in `control-service`. Admin surface: standard `control-api` routes. | 2026-05-06 | `DOC-SPEC-025` |
| `DEC-USER-004` | Catalog modules after Login | Follow existing roadmap order: Phase 3 (Catalogs & Work Paths) → Phase 5 (Knowledge) → Phase 6 (Boards) → Phase 7 (Runtime Bundle) → Phase 8 (Live Watch). Build catalog infrastructure first; module content defined by implementing phases. | 2026-05-05 | `DOC-PLAN-P3`, `DOC-PLAN-P5`, `DOC-PLAN-P6`, `DOC-PLAN-P7`, `DOC-PLAN-P8` |
| `DEC-USER-005` | Brand removal from backend names | No brand prefix in any code — clean names like `LoginHandler`, `LoginRequest`, `login_service.rs`. | 2026-05-04 | `DOC-SPEC-019` |
| `DEC-COUNCIL-RFSOURCE-001` | Source Storage — Parquet removal, .rfsource adoption | **Remove Parquet entirely** from the RealmForge storage stack. Only `.rfsource` + PostgreSQL going forward. The `parquet-store` crate is removed from the workspace. Supersedes `DEC-COUNCIL-003`. | 2026-05-06 | `DEC-COUNCIL-RFSOURCE-001` |
| `DEC-COUNCIL-RFSOURCE-002` | Artifact Registry naming | **"Artifact Registry"** chosen as the name for the PostgreSQL metadata layer. Table prefix: `ar_`. API route: `/v1/artifacts/`. | 2026-05-06 | `DEC-COUNCIL-RFSOURCE-002` |
| `DEC-USER-006` | When may Live Watch auto-remediate? | Tier 0 (default): reports/suggests only, no auto-remediation. Tier 1 (opt-in per app): pre-approved low-severity actions with bounded blast radius, explicit operator opt-in per app. Tiers 2-3 deferred. | 2026-05-05 | `DOC-PLAN-P8` |
| `DEC-USER-RFSOURCE-001` | Governance model for rfsource grants | **Option B** — New dedicated `SGL-RFSOURCE-*` grant namespace. Clean separation from backend grants. Initial grants: `SGL-RFSOURCE-COMMIT` (write artifacts), `SGL-RFSOURCE-READ` (read/search artifacts), `SGL-RFSOURCE-GOVERN` (manage grants/policies). | 2026-05-06 | `DOC-SPEC-022`, `DOC-SPEC-004` |
| `DEC-COUNCIL-RFSOURCE-003` | RFSource Source Control Feature Release Approval | **APPROVED for release** — with conditions: file-size refactor before next feature addition; known limitations remain open and tracked. Dissent recorded from Security (F-003 residual risk) and QA (F-007 atomicity gap). | 2026-05-06 | `DEC-COUNCIL-RFSOURCE-003` |
| `DEC-COUNCIL-WFE-001` | Workflow Engine crate adoption | **Adopted as-is** with accepted file-size condition. All upstream gates (Architecture → Peer Review → Code Review → QA) passed. orchestrator.rs file-size finding accepted-with-risk by CTO. | 2026-05-06 | `DOC-SPEC-027`, `DEC-COUNCIL-WFE-001` |

## Lockout Rule


Implementation that depends on a decision in this file cannot proceed until the decision record is closed and linked from the relevant spec.

## Blocker Triage Rule

Do not force every listed decision through Council at once. Route a decision to Council when active work reaches its lockout boundary, then record and link only that blocker before continuing.
