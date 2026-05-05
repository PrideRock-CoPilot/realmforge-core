---
doc_id: DOC-CODEX-006
title: Code Review Ledger
status: active
owner: code-review
reviewers: [peer-review, cto, qa, tech-writer]
created_at: 2026-05-05
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: codex-operations
work_path_ids: [WP-DOCS-000]
related_decision_ids: []
related_file_ids: [FILE-DOCS-CODE-REVIEW-LEDGER, FILE-DOCS-CODE-REVIEW-PROCESS, FILE-SKILL-CODE-REVIEW]
visual_node_ids: []
visual_edge_ids: []
approval_state: accepted
---

# Code Review Ledger

## Current Rule

No project-board area is `Code Review DONE` until every file in that area has a
completed `File Review Record` using `docs/codex/05_CODE_REVIEW_PROCESS.md`.

This ledger starts the code-review gate. It does not mark any code-review area
complete yet.

## Area Status

| Area | Code Review Status | Required before DONE |
| --- | --- | --- |
| Phase 0 - Documentation Certification | N/A | No implementation code-review gate. |
| Phase 1 - Workspace And Prerequisites | IN_REVIEW | Inventory all workspace/config files and crate manifests. |
| Phase 2 - Authority Core | PENDING | Inventory all authority, policy, audit, snapshot, store, service, API, CLI, MCP, and integration files. |
| Phase 3 - Catalogs And Work Paths | PENDING | Inventory catalog/work-path domain, store, service, transport, and tests. |
| Phase 4 - Agent Gateway And Skill Grants | PENDING | Inventory gateway, grant, policy, migration, and denial-path tests. |
| Phase 5 - Knowledge And Parquet Snapshots | PENDING | Inventory knowledge service, parquet-store, contracts, transports, and acceptance evidence. |
| Phase 6 - Boards And Build Watch | PENDING | Inventory boards, build-watch, frontend board surfaces, routes, CLI, MCP, and tests. |
| Phase 7 - Runtime Bundle And Live Runtime | PENDING | Inventory runtime-bundle, live-runtime, bundle/runtime service, transports, and tests. |
| Phase 8 - Live Watch | PENDING | Inventory live-watch crate, service, route, CLI, MCP, profile, remediation, and tests. |
| Phase 9 - Login Vertical | PENDING | Inventory Login catalog, contracts, policy, handler, API, CLI, MCP, tests, audit, and snapshot files. |

## Intake From Current Working Tree

These files are currently visible in the non-vendor working tree and must be
classified into area inventories before any code-review cell moves to `DONE`.

### Codex Operations And Skill System

- `.claude/skills/code-review/SKILL.md`
- `.claude/skills/peer-review/SKILL.md`
- `.claude/skills/realmforge-skill-creator/SKILL.md`
- `.clineinstructions`
- `.clinerules/00-company-index.md`
- `.clinerules/01-session-protocol.md`
- `.clinerules/02-routing-and-skills.md`
- `.clinerules/05-workflow-and-handoffs.md`
- `.clinerules/06-repo-hygiene-and-verification.md`
- `.clinerules/07-focused-workflow-lifecycle.md`
- `AGENTS.md`
- `CLAUDE.md`
- `docs/codex/04_CLINE_NEXT_PHASE_PLANNING_HANDOFF.md`
- `docs/codex/05_CODE_REVIEW_PROCESS.md`
- `docs/codex/06_CODE_REVIEW_LEDGER.md`
- `docs/plan/Phase0.md`
- `docs/plan/README.md`
- `docs/spec/00_INDEX.md`
- `docs/spec/04_METADATA_STANDARD.md`
- `skills/code-review/SKILL.md`
- `skills/peer-review/SKILL.md`
- `skills/realmforge-skill-creator/SKILL.md`

### Phase 3 - Catalogs And Work Paths

- `crates/control-service/tests/catalog_integration.rs`
- `crates/control-service/tests/work_path_integration.rs`
- `crates/control-store/src/work_path.rs`
- `docs/plan/Phase3.md`

### Phase 5 - Knowledge And Parquet Snapshots

- `docs/plan/Phase5.md`

### Phase 6 - Boards And Build Watch

- `docs/plan/Phase6.md`
- `frontend/package-lock.json`
- `frontend/package.json`
- `frontend/src/App.tsx`
- `frontend/src/components/ui/graph-canvas.tsx`
- `frontend/src/components/ui/visual-edge.tsx`
- `frontend/src/components/ui/visual-node.tsx`
- `frontend/src/features/boards/cost/cost-board.tsx`
- `frontend/src/features/boards/evidence/evidence-board.tsx`
- `frontend/src/features/boards/intake/intake-board.tsx`
- `frontend/src/features/boards/packet/packet-board.tsx`
- `frontend/src/features/boards/release/release-board.tsx`
- `frontend/src/features/boards/work-path/work-path-board.tsx`
- `frontend/src/features/visual-map/visual-map-page.tsx`
- `frontend/src/hooks/use-board-state.ts`
- `frontend/src/index.css`
- `frontend/src/lib/board-state.ts`
- `frontend/src/lib/error.ts`
- `frontend/src/lib/formatters.ts`
- `frontend/src/main.tsx`
- `frontend/src/store/ui.store.ts`
- `frontend/src/vite-env.d.ts`
- `frontend/vite.config.ts`

### Phase 7 - Runtime Bundle And Live Runtime

- `crates/live-runtime/src/executor.rs`
- `crates/live-runtime/src/lib.rs`
- `docs/plan/Phase7.md`

### Phase 8 - Live Watch

- `crates/live-watch/src/lib.rs`
- `docs/plan/Phase8.md`

### Phase 9 - Login Vertical

- `catalog/Login/catalog.json`
- `catalog/Login/contracts/login_response.json`
- `crates/agent-mcp/src/tools/login.rs`
- `crates/control-api/src/models.rs`
- `crates/control-api/src/routes/login.rs`
- `crates/control-service/src/login_handler.rs`
- `crates/control-service/tests/login_vertical.rs`
- `docs/plan/Phase9.md`
- `docs/spec/17_API_MCP_CLI_CONTRACTS.md`
- `docs/spec/19_FIRST_VERTICAL_LOGIN_MODULE.md`
- `docs/spec/20_IMPLEMENTATION_ROADMAP.md`
- `docs/spec/21_ACCEPTANCE_TEST_PLAN.md`

### Excluded From Code Review Evidence

- `frontend/node_modules/**` - local dependency install output.
- `frontend/tsconfig.tsbuildinfo` - TypeScript incremental build cache.
- `target/**` and `target-quality/**` - Rust build output.

## Next Action

Move one area at a time from `PENDING` to `IN_REVIEW`, expand its inventory from
the phase plan and metadata registry, then create one file review record per
file. Only then can that area's `Code Review` cell move toward `DONE`.
