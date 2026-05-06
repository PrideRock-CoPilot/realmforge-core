---
decision_id: ADR-0008
title: "Intake Engine Architecture — Crate Scope, Layer Compliance, and Integration Contract"
status: accepted
owner: cto
participants: [cto, domain-architect, backend, api-architect, tech-writer]
decided_at: 2026-05-06
supersedes: []
related_spec_ids: [DOC-SPEC-025, DOC-SPEC-022]
---

# ADR-0008: Intake Engine Architecture

## Context

The Intake System Design (DOC-SPEC-025) was approved by Council (DEC-COUNCIL-INTAKE-001) with the mandate to build a structured deterministic intake engine. Before implementation begins, Rena (CTO) performed a full architecture review to define crate scopes, verify layer compliance, assess reversal cost, and produce an engineering contract for Dmitri (Backend).

## Decision

### 1. Crate Structure

| Crate | Package Name | Purpose | Dependencies |
|---|---|---|---|
| `crates/intake-engine` | `intake-engine` | Pure decision tree evaluation — parse JSON, walk tree, evaluate conditions, map features | `serde`, `serde_json`, `thiserror` only |
| `crates/control-store/src/intake/` | (in `control-store`) | Persistence for 4 tables: trees, sessions, observations, app_types | `control-store` internals |
| `crates/control-service/src/intake_service.rs` | (in `control-service`) | EXTEND — add structured flow methods alongside existing `create_plan()` | `intake-engine`, `control-store::intake` |
| `crates/control-service/src/intake_observation_service.rs` | (in `control-service`) | NEW — observation recording + promotion thresholds | `control-store::intake` |
| `crates/control-api/src/routes/intake.rs` | (in `control-api`) | NEW — 9 REST endpoints | `control-service` |
| `crates/agent-mcp/src/tools/intake.rs` | (in `agent-mcp`) | NEW — 6 MCP intake tools | `control-service` |

### 2. Layer Compliance

```
intake-engine (pure, no IO)
    ↓
control-store::intake (persistence)
    ↓
control-service::intake_service (orchestration)
    ↓
control-api/routes/intake  +  agent-mcp/tools/intake  +  operator-cli/commands/intake
```

All layer boundaries respected. `intake-engine` has zero RealmForge crate dependencies.

### 3. Key Architectural Rules

1. **`intake-engine` is independent of `authority-domain`** — The intake system is a pre-processing engine, not a domain bounded context. It must not depend on RealmForge domain types. If domain types are needed at integration points, the adaptation happens in `control-service`.

2. **Existing `create_plan()` is preserved** — The old API stays unchanged. Both the old structured flow and new decision-tree flow coexist side by side.

3. **Session snapshots the tree at start** — When a session begins, the tree version at that moment is snapshotted. Tree updates apply only to new sessions, never mid-session.

4. **Migration 013** — `db/migrations/013_intake_system.sql` creates 4 tables, following the existing migration numbering scheme.

5. **File size limit** — If `intake_service.rs` (currently 407 lines) exceeds 500 lines after extension, split into `intake_service.rs` (orchestration, <300 lines) and `intake_engine_service.rs` (engine integration, <200 lines).

### 4. Reversibility

**Total blast radius: 10 person-days to undo entirely.** All new code is additive — deleting the new crate, module, routes, migration, and reverting extension points restores the previous state. No downstream callers are affected because `create_plan()` is preserved.

### 5. Integration with Existing Pipeline

The new structured intake flow feeds into the existing 6-stage pipeline at Stage 1 (Intake):

1. `create_intake_session()` → walks decision tree → collects answers
2. `intake_complete()` → creates Plan pre-populated with `core_areas`, `constraints`, `decisions` from feature mappings
3. Plan enters Stage 1 with `current_stage: PipelineStage::Intake` (same as old path)
4. Existing `refine_plan()`, `set_architecture()`, etc. continue unchanged

### 6. Phase 1 Scope

Phase 1 (Foundation, Weeks 1-4) is architecture-approved for:
- `intake-engine` crate — tree parsing, walking, condition evaluation, feature mapping
- Migration 013 — all 4 tables
- `control-store::intake` module — full CRUD for trees, sessions, observations
- `control-service` extensions — structured flow methods (new, additive)
- `control-api` routes — all 9 endpoints
- 3 starter trees: `tree-static-site.json`, `tree-rest-api.json`, `tree-web-app.json`
- Integration tests for 3 types

### 7. Conditions

1. Session must snapshot tree version at session start
2. `intake_service.rs` must stay under 500 lines; split if exceeding
3. Old `create_plan()` API must remain unchanged
4. All intake types must go through schema validation before persistence

## Consequences

- Backend owns Phase 1 implementation per this contract
- Tech Writer owns ADR alignment and spec freshness
- Peer Review (Nora) will verify area readiness before Code Review
- Code Review (Owen) will verify layer compliance at file level
- QA (Meg) will validate Phase 1 acceptance criteria
