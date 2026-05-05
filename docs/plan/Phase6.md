---
doc_id: DOC-PLAN-P6
title: Phase 6 — Boards And Build Watch
parent: DOC-PLAN-INDEX
status: draft
owner: frontend
reviewers: [backend, biz-user, pm, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-BOARDS-001, WP-BUILD-WATCH-001]
related_decision_ids: [DEC-COUNCIL-001]
related_file_ids: [FILE-CRATE-BUILD-WATCH-LIB]
visual_node_ids: [VN-MODULE-BOARDS, VN-MODULE-BUILD-WATCH]
approval_state: pending
---

# Phase 6: Boards And Build Watch

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 6 |
| Title | Boards And Build Watch |
| Work paths | `WP-BOARDS-001`, `WP-BUILD-WATCH-001` |
| Product module | Boards, Build Watch |
| Owner | frontend (Kai Sato) / backend |
| Risk | medium |
| Decision blockers | none; `DEC-COUNCIL-001` is closed |

**Mandate:** Build the first human-facing planning and monitoring surfaces. Boards provide human planning, approval, status, and release command. Build Watch monitors construction-time events and records violations, evidence, and cost.

**Decision resolution:** `DEC-COUNCIL-001` is closed. Boards frontend work uses React, Vite, React 19, TypeScript, shadcn/ui internally, the public `ui/` package, React Flow, and generated OpenAPI TypeScript clients.

---

## Workflow

```
Phase split into two parallel tracks:

Track A (Boards — frontend stack resolved by DEC-COUNCIL-001):
  1. Backend: Boards domain types, store, and service API
  2. Backend: Boards API endpoints (CRUD for plans, approvals, releases)
  3. CLI: Boards commands
  4. Frontend: Board UI shell

Track B (Build Watch — not blocked):
  1. Backend: BuildWatch domain types
  2. Backend: BuildWatch store
  3. Backend: BuildWatch service (monitor, record violations, evidence, cost)
  4. Backend: BuildWatch API endpoints
  5. CLI: BuildWatch commands
  6. MCP: BuildWatch tools
```

---

## File Manifest

### authority-domain

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `authority-domain/src/boards.rs` | 80 | `BoardPlan` — plan id, title, work path references, status (Draft, InReview, Approved, InProgress, Completed). `BoardApproval` — approval id, plan ref, approver, decision, timestamp. `ReleaseCommand` — release id, bundle ref, approval ref, status. |
| NEW | `authority-domain/src/build_watch.rs` | 100 | `WatchEvent` — event id, scope, type (FileMutation, PacketSubmission, PolicyViolation, CostAnomaly), severity (Info, Warning, Violation), detail, timestamp. `ViolationRecord` — violation id, rule, severity, evidence ref. `CostRecord` — token cost, build time, storage, rework. |
| UPDATE | `authority-domain/src/lib.rs` | 10 | Export `boards` and `build_watch` modules. |

### control-store

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-store/src/lib.rs` | 100 | `insert_board_plan()`, `list_board_plans(scope)`, `submit_approval()`. `insert_watch_event()`, `query_watch_events(scope, severity, time_range)`, `insert_violation()`, `insert_cost_record()`, `get_cost_summary(scope)`. |

### control-service

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-service/src/boards_service.rs` | 100 | `create_plan()`, `submit_for_approval()`, `approve_plan()`, `reject_plan()`, `list_plans(scope)`. `submit_release_command()` — triggers bundle build pipeline signal. |
| NEW | `control-service/src/build_watch_service.rs` | 120 | `record_event(event)` — validates and stores watch event. `detect_violation(event)` — checks against active rules. `summarize_cost(scope, time_range)` — aggregates cost records. `get_watch_dashboard(scope)` — returns current status, recent violations, cost trend. |
| UPDATE | `control-service/src/lib.rs` | 10 | Declare `boards_service` and `build_watch_service` modules. |

### control-api

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-api/src/routes/boards.rs` | 80 | `POST /v1/boards/plans`, `GET /v1/boards/plans`, `POST /v1/boards/plans/:id/approve`, `POST /v1/boards/releases`. |
| NEW | `control-api/src/routes/build_watch.rs` | 80 | `POST /v1/watch/events`, `GET /v1/watch/events`, `GET /v1/watch/dashboard`, `GET /v1/watch/cost-summary`. |
| EXTEND | `control-api/src/routes/mod.rs` | 10 | Register boards and build_watch route groups. |

### operator-cli

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `operator-cli/src/commands/boards.rs` | 80 | `boards plan create`, `boards plan list`, `boards plan approve`, `boards release submit`. |
| NEW | `operator-cli/src/commands/watch.rs` | 80 | `watch events`, `watch violations`, `watch dashboard`, `watch cost-summary`. |
| EXTEND | `operator-cli/src/commands/mod.rs` | 10 | Register boards and watch command modules. |

### agent-mcp

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `agent-mcp/src/tools/build_watch.rs` | 60 | `core_record_watch_event`, `core_get_watch_dashboard`, `core_get_cost_summary`. |
| EXTEND | `agent-mcp/src/tools/mod.rs` | 5 | Register build_watch tool module. |

---

## Key Types

### WatchEvent (NEW)
```rust
pub enum WatchSeverity { Info, Warning, Violation, Critical }

pub enum WatchEventType {
    FileMutation, PacketSubmission, PolicyViolation, CostAnomaly,
    BuildFailure, TestFailure, EvidenceGap,
}

pub struct WatchEvent {
    pub id: WatchEventId,
    pub scope: EventScope,
    pub event_type: WatchEventType,
    pub severity: WatchSeverity,
    pub detail: String,
    pub evidence_ref: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

---

## Completion Gates (QA Re-Verification Required)

- [x] `TEST-BUILD-WATCH-001` — Build Watch records unauthorized file attempt as Violation (requires DB integration test)
- [x] Board plan lifecycle works: Create → Submit → Approve → Release Command (requires DB integration test)
- [x] Watch dashboard returns current status, recent violations, and cost trend (requires DB integration test)
- [x] Violation detection fires on configurable rules (requires DB integration test)
- [x] Cost summary aggregates correctly by time range and scope (requires DB integration test)
- [x] API, CLI, and MCP surfaces all functional for both modules — compiles clean, routes registered, CLI subcommands wired, MCP tools defined
- [ ] `cargo test --workspace` passes with 0 failures
- [ ] `cargo clippy --workspace -- -D warnings` passes

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-DOMAIN` | Add boards and build_watch types to authority-domain |
| `SGL-BACKEND-SERVICE` | Add boards_service and build_watch_service |
| `SGL-BACKEND-API` | Add route groups for both modules |
| `SGL-BACKEND-CLI` | Add command modules for both modules |
| `SGL-FRONTEND-SHELL` | Build Boards UI using the accepted frontend stack |
| `SGL-DATA-POSTGRES` | Add persistence for both modules |

---

## Dependencies

- Phase 4 complete (Agent Gateway — Build Watch monitors gateway enforcement events)
- `DEC-COUNCIL-001` resolved (for Boards frontend work)
