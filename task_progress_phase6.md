# Phase 6: Boards And Build Watch — Task Progress ✅ COMPLETE

All items complete. Integration tests added 2026-05-05:
- `boards_integration.rs` — 7 tests (create/list, full lifecycle, reject, invalid state transitions)
- `build_watch_integration.rs` — 6 tests (events, violation, cost summary, dashboard, severity filter, violation detection)

- [x] Create authority-domain/src/boards.rs (types: BoardPlan, BoardApproval, ReleaseCommand, enums)
- [x] Create authority-domain/src/build_watch.rs (types: WatchEvent, WatchSeverity, WatchEventType, ViolationRecord, CostRecord)
- [x] Add new IDs (BoardPlanId, BoardApprovalId, ReleaseId, WatchEventId, ViolationId, CostRecordId) to ids.rs
- [x] Register boards + build_watch modules in authority-domain/src/lib.rs
- [x] Create control-store/src/boards.rs (Postgres persistence)
- [x] Create control-store/src/build_watch.rs (Postgres persistence)
- [x] Register modules + wire methods in control-store/src/lib.rs
- [x] Create control-service/src/boards_service.rs (business logic)
- [x] Create control-service/src/build_watch_service.rs (business logic)
- [x] Register modules + wire ServiceContext in control-service/src/lib.rs
- [x] Create control-api/src/routes/boards.rs
- [x] Create control-api/src/routes/build_watch.rs
- [x] Register routes in control-api/src/routes/mod.rs + lib.rs
- [x] Create operator-cli/src/commands/boards.rs
- [x] Create operator-cli/src/commands/watch.rs
- [x] Register CLI modules in mod.rs + main.rs
- [x] Create agent-mcp/src/tools/build_watch.rs
- [x] Register MCP build_watch tools in mod.rs + lib.rs
- [x] Create db/migrations/006_boards_build_watch.sql
- [x] Create boards_integration.rs — Board plan lifecycle DB integration tests (NEW)
- [x] Create build_watch_integration.rs — Build Watch DB integration tests (NEW)
- [x] cargo check -p control-service compiles clean (Gate 7 pre-check)
- [x] Mark completion gates in docs/plan/Phase6.md
