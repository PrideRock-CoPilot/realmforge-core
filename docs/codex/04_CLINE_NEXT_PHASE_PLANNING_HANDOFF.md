---
doc_id: DOC-CODEX-004
title: Cline Next Phase Planning Handoff — Planning Package Completed
status: draft
owner: pm
reviewers: [orchestrator, cto, peer-review, code-review, qa, tech-writer]
created_at: 2026-05-05
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: codex-operations
work_path_ids: [WP-DOCS-000, WP-CATALOG-001, WP-WORKPATH-001]
related_decision_ids: []
related_file_ids: [FILE-DOCS-CLINE-NEXT-PHASE-HANDOFF, FILE-SKILL-PEER-REVIEW, FILE-SKILL-CODE-REVIEW, FILE-DOCS-CODE-REVIEW-PROCESS, FILE-DOCS-CODE-REVIEW-LEDGER, FILE-CRATE-DOMAIN-CATALOG, FILE-CRATE-DOMAIN-WORKPATH, FILE-CRATE-SERVICE-LIB]
visual_node_ids: [VN-CLINE-NEXT-PHASE-HANDOFF]
visual_edge_ids: [VE-CLINE-PLANS-NEXT-PHASES]
approval_state: accepted
---

# Cline Next Phase Planning Package — Completed 2026-05-05
> This file was updated by the planning session to include the planning package output.
> The original handoff template is preserved above. Below is the output of this session.

---

## Current Lifecycle Stage Completed

`Planning`

---

## 1. Current State Audit

### Evidence Sources

| Source | Status |
|--------|--------|
| `git status --short` | 29 modified tracked files, 5 untracked files |
| `git ls-files --others --exclude-standard` | 5 untracked: `.clinerules/06*`, `.clinerules/07*`, `catalog_integration.rs`, `work_path_integration.rs`, `docs/codex/04*` |
| `cargo test --workspace` | **0 failures** with `--target-dir target-quality` |
| `cargo clippy --workspace --all-targets -- -D warnings` | **Passes** — no warnings |
| `docs/spec/22_OPEN_DECISIONS.md` | **0 open decisions** — all 6 decisions closed |
| Workspace crates | **14 crates** compile: agent-gateway, audit-log, authority-domain, control-store, snapshot-ledger, policy-engine, control-api, control-service, parquet-store, operator-cli, agent-mcp, runtime-bundle, live-runtime, live-watch |
| DB migrations | **11 migrations** exist (001-011), covering all phases through Login + seed data |
| `catalog/Login/` directory | **EXISTS** with catalog.json, contracts, policies, watch profile |

### Phase-by-Phase Audit Table

| Phase | Documented Status | Evidence Present | Open Gates | Blockers | Next Lifecycle Stage |
|-------|-------------------|------------------|-----------|----------|----------------------|
| **0** | Gates unchecked, needs re-cert | All spec docs have frontmatter; decisions all closed; 31 file registry entries; 4 doc tests defined | All 4 doc tests unchecked; human QA/release signoff | Human QA + Release Manager signoff | Testing (human) |
| **1-2** | Gates checked | Workspace compiles; all foundation crates exist; Session/Command/Audit/Snapshot/Transport/Integration code done | None | None | Documentation (close phase) |
| **3** | Acceptance gates mostly checked | **catalog_integration.rs** passes `TEST-CATALOG-001`; **work_path_integration.rs** passes `TEST-WORKPATH-001`; catalog/work path services and persistence compile; workspace test/clippy gates green | API/CLI/MCP CRUD operation proof | None | Testing (interface CRUD evidence) |
| **4** | Gates checked | Gateway code exists; grants/sod code exists; policy enforcement tests exist | None | None | Documentation (close phase) |
| **5** | Developed; TEST-KNOWLEDGE-001 OPEN | KnowledgeService, parquet-store, writers/readers/schemas, reconciliation, and API/CLI/MCP surfaces exist and compile | `TEST-KNOWLEDGE-001` | None blocking code — QA test needs certification | Testing (Knowledge answer QA) |
| **6** | Functional gates checked; formal acceptance pending | Boards domain types + service + API/CLI/MCP exist; BuildWatch code exists; integration tests at `boards_integration.rs` and `build_watch_integration.rs` | Formal owner/reviewer acceptance | Human signoff | Documentation (pending acceptance) |
| **7** | Developed; formal QA partial | runtime-bundle builder/signer/verifier/tests; live-runtime executor/loader/health; API/CLI/MCP surfaces exist and workspace gates pass | Interface-level bundle/runtime QA | None blocking code | Testing |
| **8** | Developed; formal QA partial | live-watch signals/profile/remediation engine exists; current-cycle anomaly proposal bug fixed; tests pass | DB-backed interval and collect→detect→propose QA | None blocking code | Testing |
| **9** | Developed; E2E QA partial | Login catalog/contracts/policy/watch profile, handler, API/CLI/MCP, integration tests, audit event, and snapshot anchor response exist | `TEST-LOGIN-E2E-001` full catalog-to-rollback loop | E2E certification waits on review/QA | Testing (vertical E2E) |

### Project Board Matrix

This document is the temporary project board until RealmForge has a dedicated board surface. Status values are intentionally conservative: do not move a cell to `DONE` without an evidence source in a spec, plan file, test result, reviewer note, QA certification, or UAT record.

| Area | Designed | Designed Approved | Developed | Peer Review | Code Review | QA Test | QA Sign Off | UAT | UAT Signoff |
|------|----------|-------------------|-----------|-------------|-------------|---------|-------------|-----|-------------|
| Phase 0 - Documentation Certification | DONE | DONE | N/A | DONE | N/A | PENDING | PENDING | PENDING | PENDING |
| Phase 1 - Workspace And Prerequisites | DONE | DONE | DONE | DONE | PENDING | DONE | PENDING | N/A | N/A |
| Phase 2 - Authority Core | DONE | DONE | DONE | DONE | PENDING | DONE | PENDING | N/A | N/A |
| Phase 3 - Catalogs And Work Paths | DONE | DONE | DONE | DONE | PENDING | PARTIAL | PENDING | PENDING | PENDING |
| Phase 4 - Agent Gateway And Skill Grants | DONE | DONE | DONE | DONE | PENDING | DONE | PENDING | N/A | N/A |
| Phase 5 - Knowledge And Parquet Snapshots | DONE | DONE | DONE | DONE | PENDING | PARTIAL | PENDING | PENDING | PENDING |
| Phase 6 - Boards And Build Watch | DONE | DONE | DONE | DONE | PENDING | DONE | PENDING | PENDING | PENDING |
| Phase 7 - Runtime Bundle And Live Runtime | DONE | DONE | DONE | DONE | PENDING | PARTIAL | PENDING | PENDING | PENDING |
| Phase 8 - Live Watch | DONE | DONE | DONE | DONE | PENDING | PARTIAL | PENDING | PENDING | PENDING |
| Phase 9 - Login Vertical | DONE | DONE | DONE | DONE | PENDING | PARTIAL | PENDING | BLOCKED | BLOCKED |
Legend: `DONE` = evidence is recorded and the relevant gate is complete; `PARTIAL` = some evidence exists but one or more gates remain open; `PENDING` = no sufficient evidence or signoff is recorded; `BLOCKED` = do not advance until the upstream gate, review, or dependency is resolved; `N/A` = the stage does not apply to this internal area.
Update rules: design approval is covered by closed Council/user decisions in `docs/spec/22_OPEN_DECISIONS.md`, architect-reviewed phase plans, and user confirmation on 2026-05-05. Peer/code review, QA signoff, UAT, and release gates remain separate and require their own evidence. Code Review `DONE` requires file-level records in `docs/codex/06_CODE_REVIEW_LEDGER.md` using `docs/codex/05_CODE_REVIEW_PROCESS.md`.

### Key Findings

1. **Code far ahead of documented gate status.** Domain types, services, API/CLI/MCP surfaces, and even tests exist for phases 3, 5, 6, 7, 8, and 9 — but the plan checkboxes remain unchecked.
2. **Phase 9 is further along than documented.** A `login_vertical.rs` integration test already exists in control-service tests and passes.
3. **File registry stale paths were corrected.** The Parquet and Login registry entries now point to the current `parquet-store` and `catalog/Login` implementation surfaces.
4. **Acceptance tests still needing certification:** TEST-KNOWLEDGE-001 and TEST-LOGIN-E2E-001. TEST-BUNDLE-001 has runtime-bundle/live-runtime coverage; Phase 3 now has TEST-CATALOG-001 and TEST-WORKPATH-001 coverage; API/CLI/MCP CRUD proof remains open.

### Peer Review Evidence

| Area | Peer Review Result | Evidence Reviewed | Residual Risk / Next Gate |
|------|--------------------|-------------------|---------------------------|
| Phase 0 | DONE | Spec index, metadata standard, closed decisions, documentation gates | QA/release signoff still open |
| Phase 1 | DONE | Workspace plan gates, migrations, workspace quality commands | Code review pending |
| Phase 2 | DONE | Authority Core plans, integration tests, service/API/CLI/MCP evidence | Code review pending |
| Phase 3 | DONE | Catalog/work-path tests, metadata registry, transport files | API/CLI/MCP CRUD QA remains partial |
| Phase 4 | DONE | Gateway/skill grant plan gates and denial-path tests | Code review pending |
| Phase 5 | DONE | KnowledgeService, parquet-store, DEC-COUNCIL-003, roadmap evidence | TEST-KNOWLEDGE-001 QA remains partial |
| Phase 6 | DONE | Boards/Build Watch tests, routes, CLI, MCP, plan gates | Formal owner/QA signoff pending |
| Phase 7 | DONE | Runtime bundle signature tests, live-runtime governance receipts, transports | Interface QA and code review pending |
| Phase 8 | DONE | Live Watch tests, current-cycle anomaly fix, DEC-USER-006 constraints | DB-backed E2E QA pending |
| Phase 9 | DONE | Login catalog/contracts/policy/watch profile, handler, transports, snapshot anchor test | Full catalog-to-rollback QA pending |

Progress 2026-05-05: All `Designed Approved` cells moved to `DONE`; Phase 3 and Phases 5/7/8/9 `Developed` moved to `DONE`; all Peer Review cells moved to `DONE` after Nora Patel peer-review pass. Code Review now has Owen Brooks, a file-level process, and an intake ledger; no Code Review cells move to `DONE` until every in-scope file record is complete. Phase 3 tests were added, `work_path.rs` graph updates were hardened, Live Watch current-cycle anomaly proposal was fixed, Login now returns a persisted snapshot anchor, and workspace test/clippy gates passed.

---

## 2. Earliest Unblocked Slice

### Chosen Slice: **Phase 3 — Catalogs And Work Paths Acceptance Test Certification**

**Why this slice?**
- Phase 3 is the **earliest roadmap item** with open gates that are **not** blocked by decisions, human QA signoff, or upstream dependencies.
- The code **already exists** — domain types, services, migrations, and transport surfaces are implemented. The gap is purely acceptance/integration test coverage for `TEST-CATALOG-001` and `TEST-WORKPATH-001`.
- Closing Phase 3 gates unlocks confidence in downstream phases (5, 6, 7, 8, 9) that depend on catalog and work path infrastructure.
- Smallest blast radius: focused test writing in existing crates, no new architecture needed.

**Why not another slice?**
- Phase 0: blocked by human QA/release signoff.
- Phase 5/7/8: later in the dependency chain; Phase 3 should close first.
- Phase 9: explicitly should not start until upstream gates are honest.

---

## 3. Planning Package

### Outcome Statement

Phase 3 Catalogs and Work Paths acceptance test certification. Write two integration tests to prove `TEST-CATALOG-001` (Tenant copies approved global module with provenance) and `TEST-WORKPATH-001` (Work path node generates scoped packet with correct boundaries). Both tests should be runnable with Postgres and verify the full store → service → domain flow.

### In-Scope Boundaries

- Write `TEST-CATALOG-001` integration test for catalog copy with provenance
- Write `TEST-WORKPATH-001` integration test for work path traversal → packet generation
- Add test files under `crates/control-service/tests/` following the existing pattern (e.g., `catalog_integration.rs`, `work_path_integration.rs`)
- Use existing test helpers and DB setup patterns from `session_lifecycle.rs`, `boards_integration.rs`, etc.
- Update Phase 3 completion gates to checked after evidence
- Update `docs/spec/20_IMPLEMENTATION_ROADMAP.md` roadmap position for Phase 3
- Update `docs/spec/21_ACCEPTANCE_TEST_PLAN.md` with current test status
- Update `docs/spec/04_METADATA_STANDARD.md` if new test files need registry entries

### Out-of-Scope Boundaries

- Do NOT modify any domain types, service logic, store queries, API/CLI/MCP surfaces, or migrations
- Do NOT refactor existing catalog or work path code
- Do NOT add new capability beyond what's needed to verify existing code
- Do NOT touch Phase 5, 7, 8, or 9

### Required Skills and Handoff Order

1. **`qa` (Meg Thompson)** — Define test requirements and acceptance criteria
2. **`backend` (Dmitri Volkov)** — Write integration tests in control-service
3. **`qa` (Meg Thompson)** — Verify tests pass and certify
4. **`tech-writer` (Clara Mills)** — Update phase docs, roadmap, acceptance test plan, metadata registry

### Affected Files

| File | Action | Purpose |
|------|--------|---------|
| `crates/control-service/tests/catalog_integration.rs` | NEW | TEST-CATALOG-001: copy global module to tenant, verify provenance recorded |
| `crates/control-service/tests/work_path_integration.rs` | NEW | TEST-WORKPATH-001: create work path, add nodes, traverse → verify scoped packet |
| `docs/plan/Phase3.md` | UPDATE | Check TEST-CATALOG-001 and TEST-WORKPATH-001 boxes; add verification note |
| `docs/spec/20_IMPLEMENTATION_ROADMAP.md` | UPDATE | Update Phase 3 roadmap position |
| `docs/spec/21_ACCEPTANCE_TEST_PLAN.md` | UPDATE | Add verification note for TEST-CATALOG-001 and TEST-WORKPATH-001 |
| `docs/spec/04_METADATA_STANDARD.md` | UPDATE | Add registry entries for new test files if not already covered |

### Architecture Questions for `cto`

**None required.** This slice is pure test verification of already-approved domain/service code. The architecture is established and documented in Phase 3 plan.

### Acceptance Criteria Mapped to Test IDs

| Test ID | Criterion | Verification |
|---------|-----------|--------------|
| `TEST-CATALOG-001` | Tenant copies approved global module with provenance recorded | Integration test: create global module → approve → copy to tenant → verify provenance field populated and links back to source |
| `TEST-WORKPATH-001` | Work path node generates scoped packet with correct boundaries | Integration test: create work path → add nodes with file/contract/test links → traverse → verify AgentWorkPacket has correct file scope, contract IDs, and test IDs |

### Verification Commands

```powershell
# Required before closing:
$env:PATH = "C:\Users\PLiek\.cargo\bin;$env:PATH"
cargo test --workspace --target-dir target-quality   # 0 failures
cargo clippy --workspace --all-targets --target-dir target-quality -- -D warnings   # passes
cargo fmt --all -- --check   # formatting clean
git diff --check   # no whitespace errors
git status --short   # only intended files modified
```

### Documentation Updates Required After Implementation

- `docs/plan/Phase3.md` — Check TEST-CATALOG-001, TEST-WORKPATH-001, cargo test, cargo clippy gates; add verification note with date
- `docs/spec/20_IMPLEMENTATION_ROADMAP.md` — Update Phase 3 next action from "Reconcile implementation evidence" to "Test certification complete"
- `docs/spec/21_ACCEPTANCE_TEST_PLAN.md` — Add verification note stating both tests pass with evidence date
- `docs/spec/04_METADATA_STANDARD.md` — Add registry entries for the two new test files, or confirm they're covered by the existing `FILE-CRATE-SERVICE-LIB` entry

### Rollback or Recovery Consideration

- **Minimal risk.** New test files only. If a test uncovers a bug in existing code, file a defect and do not fix during test-only work — return to Development with the named defect.
- **Rollback:** `git revert` the test file additions and doc updates. No migration or schema impact.
- **Recovery:** Tests require a running Postgres instance matching the `DATABASE_URL` convention used by other integration tests.

---

## 4. Readiness Decision

### `READY_FOR_DEVELOPMENT`

**Rationale:**
- Planning is complete (this package)
- Architecture is documented and approved (Phase 3 plan file)
- All 6 decisions in `22_OPEN_DECISIONS.md` are CLOSED
- Code exists and compiles — only acceptance test coverage is missing
- Work is scoped to two integration tests and doc updates
- No new architecture contracts or layer-boundary decisions needed

**Immediate next step:** Load `qa` skill, then `backend` skill, and write the two integration tests following the existing patterns in `crates/control-service/tests/`.

---

## Final Report

### Lifecycle Stage Completed
- **Planning** — Complete. Produced current state audit, identified earliest unblocked slice, and produced full planning package.

### Chosen Next Slice
- **Phase 3 Catalogs And Work Paths — Acceptance Test Certification**
- Focused on writing TEST-CATALOG-001 and TEST-WORKPATH-001 integration tests
- Three docs to update after implementation

### Verified Readiness Criteria
- `docs/spec/22_OPEN_DECISIONS.md`: **0 open decisions** — all clear
- `cargo test --workspace`: **0 failures** across all 48 test result groups
- `cargo clippy --workspace -- -D warnings`: **Passes**
- Workspace: 14 crates, 11 migrations, all compile without error
- Repository: 29 modified tracked files and 5 untracked files; includes pre-existing repo workflow edits plus this session's Phase 3/5/7/8/9 code and documentation updates.

### Repository Status
```
Tracked changes include repo workflow files, Login catalog/contracts, login API/MCP/service/test updates, Live Watch engine fix, Phase 3 acceptance tests, and roadmap/phase/spec board docs.
Untracked changes: .clinerules/06*, .clinerules/07*, crates/control-service/tests/catalog_integration.rs, crates/control-service/tests/work_path_integration.rs, docs/codex/04_CLINE_NEXT_PHASE_PLANNING_HANDOFF.md.
```

Do not revert unrelated repo workflow edits; they predate this matrix update.

### Proposed Next Lifecycle Stage
- **Testing and review** — certify the remaining partial QA gates, then collect peer/code review and signoff evidence.

### Open Decisions or Blockers
- **None.** All decisions are closed. All workspace quality gates pass.
- **Open quality work:** TEST-KNOWLEDGE-001 and TEST-LOGIN-E2E-001 remain partial; QA signoff, UAT, and release-manager signoff are not complete.
