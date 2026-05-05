---
doc_id: DOC-PLAN-P2F
title: Phase 2f — Integration Tests & Observability
parent: DOC-PLAN-INDEX
status: draft
owner: cto
reviewers: [backend, qa, release-manager]
created_at: 2026-05-04
last_reviewed_at: 2026-05-04
source_of_truth: true
product_area: roadmap
work_path_ids: [WP-CORE-001]
related_decision_ids: []
related_file_ids: [FILE-CRATE-SERVICE-LIB, FILE-CRATE-API-LIB, FILE-CRATE-CLI-MAIN, FILE-CRATE-STORE-LIB, FILE-CRATE-POLICY-LIB]
visual_node_ids: [VN-CRATE-SERVICE, VN-CRATE-API, VN-CRATE-CLI, VN-CRATE-STORE, VN-CRATE-POLICY]
approval_state: pending
---

# Phase 2f: Integration Tests & Observability

## Overview

| Field | Value |
|-------|-------|
| Phase ID | 2f |
| Title | Integration Tests & Observability |
| Work paths | `WP-CORE-001` |
| Product module | Authority Core |
| Owner | cto (Dr. Rena Okafor) |
| Risk | high |
| Decision blockers | none |

**Mandate:** Write complete integration tests that validate the full governance loop across all layers. Add structured logging, tracing, metrics, and error propagation to every service method.

---

## Workflow

```
1. Integration test setup
   a. Test helpers, test database setup, shared fixtures
   b. Test database: each test creates its own schema for isolation

2. Service-layer integration tests
   a. Session lifecycle
   b. Command lifecycle
   c. Policy enforcement (all 6 denial paths)
   d. Audit integrity (hash chain with tampering)
   e. Snapshot flow (create → validate → compare → store → retrieve)
   f. Rollback flow (preview → execute → verify → validate)
   g. Work packet flow (generate → validate → scope check)

3. API integration tests
   a. Health endpoint
   b. Full session API lifecycle
   c. Full command API lifecycle
   d. Full audit API lifecycle
   e. Cross-crate integration: propose via API → verify via audit → snapshot → rollback → verify

4. CLI integration tests
   a. All commands against test database
   b. JSON and pretty output formats

5. Observability
   a. Tracing spans on every service method
   b. Structured policy decision logging
   c. Database query timing instrumentation
   d. API response headers including X-Request-Id
   e. Metrics: request count, latency, error rates, policy deny rates
```

---

## File Manifest

### control-service/tests/

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-service/tests/common/mod.rs` | 100 | Test helpers: `create_test_db()`, `seed_test_data()`, test fixtures for actors, sessions, roles. Database creates unique schema per test for parallel execution. |
| NEW | `control-service/tests/session_lifecycle.rs` | 80 | Issue → Activate → Renew → Revoke → Verify. Test: active session passes policy, expired session denied, revoked session denied. |
| NEW | `control-service/tests/command_lifecycle.rs` | 100 | Propose → Authorize → Apply → Audit → Verify chain. Test: full happy path, audit events contain correct data. |
| NEW | `control-service/tests/policy_enforcement.rs` | 120 | All 6 denial paths: SessionExpired, GrantMissing, ActionDenied, WrongSkill, ProposalOnly, ApprovalRequired. |
| NEW | `control-service/tests/audit_integrity.rs` | 80 | Append 5 events → verify chain → tamper one event → verify fails. Test query returns paged results with filters. |
| NEW | `control-service/tests/snapshot_flow.rs` | 80 | Create → Validate → Compare → Store → Retrieve. Test parent→child snapshot chain. |
| NEW | `control-service/tests/rollback_flow.rs` | 80 | Preview → Execute → Verify → Validate restored state. |
| NEW | `control-service/tests/work_packet_flow.rs` | 80 | Generate → Validate → Scope check → Boundary test. |

### control-api/tests/

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `control-api/tests/common/mod.rs` | 80 | Test app builder — builds Axum app with test database. Test client — HTTP client wrapper with JSON helpers. |
| NEW | `control-api/tests/health_test.rs` | 40 | Health endpoint returns expected response with correct status codes. |
| NEW | `control-api/tests/session_test.rs` | 80 | Create session via API → verify response → renew → revoke → verify 404 on retrieve. |
| NEW | `control-api/tests/command_test.rs` | 80 | Propose via API → authorize → apply → verify status changes. |
| NEW | `control-api/tests/audit_test.rs` | 60 | Create events via session + command lifecycle → query audit API → verify chain. |
| NEW | `control-api/tests/integration_test.rs` | 100 | Cross-crate: propose command via API → verify via audit API → create snapshot → rollback → verify rolled state. |

### operator-cli/tests/

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| NEW | `operator-cli/tests/cli_integration.rs` | 100 | Test every CLI subcommand against test database. |
| NEW | `operator-cli/tests/output_format.rs` | 60 | Verify JSON and pretty output formats produce correct content. |

### Observability (extend existing files)

| Action | File | Lines (est.) | Purpose |
|--------|------|-------------|---------|
| EXTEND | `control-service/src/lib.rs` | 20 | Add tracing spans to all service module public methods |
| EXTEND | `control-service/src/error.rs` | 20 | Add error codes, severity levels, user-facing messages |
| EXTEND | `control-api/src/middleware.rs` | 40 | Add request tracing spans, latency metrics, request ID propagation |
| EXTEND | `control-api/src/main.rs` | 20 | Add tracing subscriber (JSON output), metrics endpoint setup |
| EXTEND | `control-store/src/lib.rs` | 30 | Add query timing instrumentation, connection pool metrics |
| EXTEND | `policy-engine/src/lib.rs` | 20 | Add structured policy decision logging with all context fields |

---

## Test Coverage Requirements

```
Service layer: > 80% line coverage on all service methods
API layer:     Every endpoint has at least one success + one failure test
CLI layer:     Every command has at least one success test

Critical paths that require specific coverage:
  - Session: issue → policy check → expiry
  - Command: propose → authorize → apply → audit verify
  - Policy: every denial code exercised
  - Snapshot: create → validate → compare chain
  - Rollback: preview → execute → verify chain
  - Work packet: generate → validate → boundary test
```

---

## Observability Principles

1. Every service method starts with a `tracing::info_span!`
2. Every policy decision emits a structured log event with scope, action, decision, reason
3. Every database query is instrumented with timing
4. API responses include `X-Request-Id` header
5. Metrics: request count, latency p50/p95/p99, error rate by code, policy deny rate

---

## Completion Gates

- [x] `cargo test --workspace` passes all 76 tests
- [x] Integration tests cover all crates together
- [x] Test coverage > 80% on service layer
- [x] Structured trace output is visible in console
- [x] Policy decisions are logged with all relevant context
- [x] Error propagation correctly wraps and annotates errors at each layer
- [x] API responses include `X-Request-Id`
- [x] Cross-crate integration test passes: propose → authorize → apply → audit → snapshot → rollback → verify
- [x] `cargo clippy --workspace -- -D warnings` passes

---

## Required Skill Grants

| Grant ID | Purpose |
|----------|---------|
| `SGL-BACKEND-SERVICE` | Modify control-service |
| `SGL-BACKEND-API` | Modify control-api |
| `SGL-BACKEND-CLI` | Modify operator-cli |
| `SGL-BACKEND-AUDIT` | No changes to audit-log needed |
| `SGL-BACKEND-POLICY` | Modify policy-engine |
| `SGL-QA-VERIFY` | Review and certify integration tests |

---

## Dependencies

- Phase 2e complete (all three transport surfaces implemented)
