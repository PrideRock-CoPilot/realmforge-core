---
doc_id: DOC-PLAN-P2E
title: Phase 2e — API, CLI & MCP Transport Surfaces
parent: DOC-PLAN-INDEX
status: draft
owner: cto
reviewers: [api-architect, security-architect, backend, qa]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-CORE-001]
related_decision_ids: []
related_file_ids: [FILE-CRATE-API-LIB, FILE-CRATE-CLI-MAIN, FILE-CRATE-MCP-LIB]
visual_node_ids: [VN-CRATE-API, VN-CRATE-CLI, VN-CRATE-MCP]
approval_state: pending
---

# Phase 2e: API, CLI & MCP Transport Surfaces

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 2e |
| Title | API, CLI & MCP Transport Surfaces |
| Work paths | `WP-CORE-001` |
| Product module | Authority Core |
| Owner | cto (Dr. Rena Okafor) |
| Risk | high |
| Decision blockers | none |

**Mandate:** Complete the REST API, operator CLI, and MCP tool contracts. All three transports are thin adapters over `control-service` — they contain no business logic, only request parsing, response formatting, and error mapping.

---

## Workflow

```
1. API Surface (control-api)
   a. Add middleware (tracing, error handling, request ID)
   b. Add route groups: health, session, command, audit, snapshot, rollback, actor, work_packet
   c. Add request/response models
   d. Add error mapping (ApiError → HTTP status code)

2. CLI Surface (operator-cli)
   a. Extend main.rs with all subcommands
   b. Add command modules: session, command, audit, snapshot, rollback, actor, work_packet, skill, migrate, config

3. MCP Surface (agent-mcp)
   a. Add error types
   b. Add shared input/output types
   c. Add tool modules for every service method
```

---

## File Manifest

### control-api

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-api/src/lib.rs` | 20 | Add all route group declarations |
| EXTEND | `control-api/src/main.rs` | 30 | Add middleware stack (CORS, tracing, request ID), metrics endpoint |
| NEW | `control-api/src/error.rs` | 80 | `ApiError` enum — maps each ServiceError variant to HTTP status code and RFC 7807 Problem Details response. Status mapping: PolicyDenied→403, NotFound→404, ValidationError→422, Conflict→409, InternalError→500 |
| NEW | `control-api/src/middleware.rs` | 60 | Request tracing span, error handler middleware, request ID generation and propagation via `X-Request-Id` header |
| NEW | `control-api/src/models.rs` | 100 | Request/response types for all endpoints. JSON serializable. Pagination support (limit, offset, sort_by, order) for list operations. |
| NEW | `control-api/src/routes/mod.rs` | 10 | Route group declarations |
| NEW | `control-api/src/routes/health.rs` | 40 | `GET /health`, `GET /ready`, `GET /live` — health check endpoints |
| NEW | `control-api/src/routes/session.rs` | 80 | `POST /v1/session` (issue), `GET /v1/session/:id`, `PUT /v1/session/:id/renew`, `DELETE /v1/session/:id` (revoke) |
| NEW | `control-api/src/routes/command.rs` | 80 | `POST /v1/commands` (propose), `PUT /v1/commands/:id/authorize`, `PUT /v1/commands/:id/apply`, `PUT /v1/commands/:id/deny` |
| NEW | `control-api/src/routes/audit.rs` | 60 | `GET /v1/audit/events`, `GET /v1/audit/chain/verify` |
| NEW | `control-api/src/routes/snapshot.rs` | 80 | `POST /v1/snapshots`, `GET /v1/snapshots`, `GET /v1/snapshots/:id`, `POST /v1/snapshots/:id/validate`, `GET /v1/snapshots/:id/compare` |
| NEW | `control-api/src/routes/rollback.rs` | 80 | `POST /v1/rollback/preview`, `POST /v1/rollback/execute`, `GET /v1/rollback/:id/verify` |
| NEW | `control-api/src/routes/actor.rs` | 40 | `GET /v1/actors/:id/scope` |
| NEW | `control-api/src/routes/work_packet.rs` | 60 | `POST /v1/work-packets/generate`, `POST /v1/work-packets/:id/validate` |

### operator-cli

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `operator-cli/src/main.rs` | 40 | Add all subcommand declarations to clap parser |
| NEW | `operator-cli/src/commands/mod.rs` | 10 | Command module declarations |
| NEW | `operator-cli/src/commands/session.rs` | 60 | `session issue`, `session renew`, `session revoke`, `session list` |
| NEW | `operator-cli/src/commands/command.rs` | 60 | `command propose`, `command authorize`, `command apply`, `command deny` |
| NEW | `operator-cli/src/commands/audit.rs` | 50 | `audit events`, `audit verify-chain`, `audit export` |
| NEW | `operator-cli/src/commands/snapshot.rs` | 60 | `snapshot create`, `snapshot validate`, `snapshot list`, `snapshot compare` |
| NEW | `operator-cli/src/commands/rollback.rs` | 60 | `rollback preview`, `rollback execute`, `rollback verify` |
| NEW | `operator-cli/src/commands/actor.rs` | 30 | `actor scope` |
| NEW | `operator-cli/src/commands/work_packet.rs` | 50 | `work-packet generate`, `work-packet validate` |
| NEW | `operator-cli/src/commands/skill.rs` | 50 | `skill register`, `skill list`, `skill validate` |
| NEW | `operator-cli/src/commands/migrate.rs` | 50 | `migrate up`, `migrate down`, `migrate list` (extend from existing ListMigrations) |
| NEW | `operator-cli/src/commands/config.rs` | 40 | `config init`, `config show`, `config validate` |

### agent-mcp

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `agent-mcp/src/lib.rs` | 30 | Add all tool definitions to `handle_tool()` dispatch. Add `core_tool_definitions()` returning all tools. |
| NEW | `agent-mcp/src/error.rs` | 60 | `McpError` enum — variants for UnknownTool, InvalidInput, ServiceError, InternalError. Each implements Display with corrective action messages. |
| NEW | `agent-mcp/src/types.rs` | 80 | Shared input/output structs for all MCP tools, derived from service types. JsonSchema derive for automatic input validation. |
| NEW | `agent-mcp/src/tools/mod.rs` | 10 | Tool module declarations |
| NEW | `agent-mcp/src/tools/session.rs` | 60 | `core_issue_session`, `core_renew_session`, `core_revoke_session` |
| NEW | `agent-mcp/src/tools/command.rs` | 60 | `core_propose_command`, `core_authorize_command`, `core_apply_command` |
| NEW | `agent-mcp/src/tools/audit.rs` | 40 | `core_query_events`, `core_verify_chain` |
| NEW | `agent-mcp/src/tools/snapshot.rs` | 60 | `core_create_snapshot`, `core_validate_snapshot`, `core_compare_snapshots` |
| NEW | `agent-mcp/src/tools/rollback.rs` | 60 | `core_preview_rollback`, `core_execute_rollback`, `core_verify_rollback` |
| NEW | `agent-mcp/src/tools/actor.rs` | 30 | `core_get_actor_scope` |
| NEW | `agent-mcp/src/tools/skill.rs` | 50 | `core_register_skill`, `core_activate_skill_session` |
| NEW | `agent-mcp/src/tools/work_packet.rs` | 50 | `core_generate_work_packet`, `core_validate_work_packet` |

---

## API Design Principles

1. Every route handler is < 30 lines (calls service, formats response)
2. Every response includes `request_id` for trace correlation
3. Error responses follow RFC 7807 Problem Details format
4. All mutation endpoints return 201 (not 200)
5. All list endpoints support pagination (limit, offset, sort_by, order)
6. All endpoints are versioned under `/v1/`

## CLI Design Principles

1. Every command has `--help` that explains purpose and all flags
2. Output is machine-readable (JSON) by default, human-readable with `--pretty`
3. Exit codes: 0=success, 1=user error, 2=data error, 3=runtime error

## MCP Design Principles

1. Each tool maps one-to-one with a service method
2. Tool descriptions are complete enough for an AI agent to use without examples
3. Input schemas are strict — every required field is listed, types are explicit
4. Error messages include corrective actions when possible
5. Total tool count stays under 25 (current: 5, target: ~20)

---

## Completion Gates

- [x] `cargo run -p control-api` starts and responds to health check (compiles)
- [x] All API endpoints return correct HTTP status codes
- [x] Error responses include proper RFC 7807 problem details format
- [x] `cargo run -p operator-cli -- --help` shows all subcommands
- [x] Every CLI subcommand works end-to-end against a test database
- [x] `--pretty` and default JSON output both work on CLI
- [x] `core_tool_definitions()` returns all 19 tools with complete schemas
- [x] Every MCP tool can be called and returns valid JSON
- [x] Unknown MCP tool returns `McpError::UnknownTool`
- [x] API handlers are < 30 lines each (thin transport only)
- [x] `cargo clippy --workspace -- -D warnings` passes

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-API` | Modify control-api crate |
| `SGL-BACKEND-CLI` | Modify operator-cli crate |
| `SGL-BACKEND-MCP` | Modify agent-mcp crate |

---

## Dependencies

- Phase 2d complete (all service layer methods have full implementations)
