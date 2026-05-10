# Control Plane — Area Context Document

**Purpose:** Define the intent, responsibilities, and boundaries of the Control Plane to guide audit question generation.

**Status:** 📋 CONTEXT DEFINITION  
**Created:** 2025-01-XX  
**Author:** Architecture Team  
**Reviewers:** Rena Okafor (CTO), Yusuf Osman (Domain Architect), Alex Rivera (Project Manager)

**Important:** This document captures WHAT the Control Plane is supposed to do (intent), NOT what currently exists (implementation). Questions will be generated from this context to test production readiness against standards, not against current code.

---

## 1. Purpose

The Control Plane is RealmForge's orchestration layer - the service layer that coordinates all domain logic, policy enforcement, audit logging, and snapshot management to execute commands safely and governably. It exists to ensure that every operation flows through the correct sequence of authorization, execution, audit, and snapshot checkpoints without any layer violations.

The Control Plane solves three critical problems:
1. **Orchestration**: Coordinates multi-step workflows (propose → authorize → apply → audit → snapshot)
2. **Layer Discipline**: Enforces the architectural contract that transport layers (API/MCP/CLI) never bypass policy or domain logic
3. **Error Boundary**: Provides consistent error handling and observability across all operations

The Control Plane is NOT a monolith - it is the service layer that composes domain, policy, audit, and snapshot capabilities into coherent workflows.

---

## 2. Responsibilities

The Control Plane owns:

* **Command Lifecycle Orchestration**: Coordinate propose → authorize → apply → audit → snapshot flow
* **Session Management**: Issue, renew, revoke, validate sessions (identity lifecycle)
* **Authorization Orchestration**: Route authorization requests through policy engine
* **Audit Event Coordination**: Trigger audit log writes for all significant operations
* **Snapshot Coordination**: Trigger snapshot creation after major state changes
* **Scope Management**: Build ActorScope from session, project, tenant, and skill context
* **Skill Lifecycle**: Register skills, activate skill sessions, validate skill integrity
* **Error Propagation**: Translate domain/policy/audit errors into service-level errors
* **Transaction Management**: Coordinate database transactions across store adapter calls
* **Service API**: Expose domain capabilities to transport layers (API/MCP/CLI)

---

## 3. Boundaries (What This Area Does NOT Own)

The Control Plane explicitly does NOT:

* **Business Logic**: Does not contain domain logic - delegates to authority-domain, policy-engine
* **Authorization Decisions**: Does not make authorization decisions - calls policy engine
* **Persistence**: Does not access PostgreSQL directly - uses control-store adapter
* **Audit Event Generation**: Does not create audit events - formats and passes to audit log
* **Snapshot Logic**: Does not implement snapshot capture - calls snapshot ledger
* **Transport Logic**: Does not handle HTTP/MCP/CLI parsing - consumed by API/MCP/CLI layers
* **User Interface**: Has no UI - provides service API for transport layers

---

## 4. Programming Language(s)

* **Primary:** Rust
  * Chosen for: Type safety in orchestration logic, compile-time verification of layer discipline, async/await for I/O coordination
  * Critical for service layer where flow control bugs can cause security bypasses or data corruption

**Why Rust?**  
The Control Plane is the central nervous system - a bug here can cascade through the entire system. Rust's type system ensures that orchestration flows are correct by construction. The borrow checker prevents race conditions in concurrent operations. The async/await model provides efficient I/O without callback hell or thread explosion.

---

## 5. High-Level Architecture

* **Layer:** Service Layer (between transport layers and domain/policy/audit/snapshot)
* **Position:** Mandatory orchestration - all commands flow through control plane services

**Key Components:**

1. **Session Service**
   * `issue_session()`: Create new session for actor, store in PostgreSQL
   * `renew_session()`: Extend session expiry if valid
   * `revoke_session()`: Invalidate session immediately
   * `validate_session()`: Check if session is valid (not expired, not revoked)

2. **Command Service**
   * `propose_command()`: Create BoundedCommand in ProposalOnly state
   * `authorize_command()`: Route to policy engine, store PolicyDecision
   * `apply_command()`: Execute command, transition state, trigger audit + snapshot
   * `deny_command()`: Record denial in audit log

3. **Actor Service**
   * `get_scope()`: Build ActorScope from session + project + tenant + skills
   * `update_scope()`: Refresh scope after skill activation or role change

4. **Skill Service**
   * `register_skill()`: Store SkillRegistration in PostgreSQL
   * `activate_skill_session()`: Create SkillSession binding to actor session
   * `validate_skill_integrity()`: Check skill hash matches expected value

5. **Audit Service**
   * `append_event()`: Format and write AuditEvent to audit log
   * `query_events()`: Fetch paged audit events with filters
   * `verify_chain()`: Validate hash chain integrity for project

6. **Snapshot Service**
   * `create_snapshot()`: Trigger snapshot capture (database + object refs)
   * `validate_snapshot()`: Verify snapshot integrity
   * `list_snapshots()`: Query snapshots by project
   * `compare_snapshots()`: Compute delta between two snapshots

7. **Rollback Service**
   * `preview_rollback()`: Compute rollback impact without execution
   * `execute_rollback()`: Restore state to snapshot
   * `verify_rollback()`: Confirm restored state matches snapshot

**Data Flow:**
* API/MCP/CLI → Control Plane Service → Policy Engine → Domain Model → Control Store → PostgreSQL
* All operations flow through service layer - no transport-to-store shortcuts

---

## 6. Key Concepts

* **Service Layer**: Orchestration and coordination logic above domain, below transport
* **Command Lifecycle**: Propose → Authorize → Apply → Audit → Snapshot (5-phase flow)
* **ActorScope**: Complete authorization context (actor, session, project, tenant, skills, roles)
* **Orchestration**: Multi-step workflows coordinated by service layer
* **Error Boundary**: Consistent error propagation from domain/policy/audit/snapshot to transport
* **Transaction Coordination**: Database transaction management across store adapter calls
* **Layer Discipline**: Service layer enforces no bypass paths (transport → domain requires service mediation)

---

## 7. Success Criteria

**Correctness:**
* Every command flows through correct lifecycle (propose → authorize → apply → audit → snapshot)
* No layer violations (transport layers never bypass service layer)
* Error propagation is consistent (domain errors translated to service errors)
* Transaction boundaries are correct (all-or-nothing database updates)

**Performance:**
* Command orchestration latency p99 < 50ms (full propose → authorize → apply flow)
* Session validation latency p99 < 5ms
* Scope construction latency p99 < 10ms

**Reliability:**
* Service layer gracefully handles domain/policy/audit failures
* Partial transactions are rolled back (no orphaned state)
* Service layer never panics (all errors are Result-typed)

**Security:**
* No authorization bypass possible (all commands go through policy engine)
* Session validation is mandatory (no anonymous operations)
* Audit events are written for all significant operations

---

## 8. Dependencies

The Control Plane depends on:

* **Authority Domain**: Consumes ActorScope, BoundedCommand, SkillRegistration types
* **Policy Engine**: Routes authorization requests through policy engine
* **Audit Log**: Writes audit events through audit service
* **Snapshot Ledger**: Triggers snapshot operations
* **Control Store**: Persists all state to PostgreSQL via store adapter
* **Tokio**: Async runtime for I/O-bound service methods
* **Thiserror**: Typed error propagation

---

## 9. Consumers

The Control Plane is consumed by:

* **Control API (REST)**: Exposes service methods as HTTP endpoints
* **Agent MCP (Model Context Protocol)**: Exposes service methods as MCP tools
* **Operator CLI**: Exposes service methods as CLI commands
* **Integration Tests**: Tests service orchestration end-to-end

All transport layers consume the service layer - none bypass it to access domain/policy/audit/snapshot directly.

---

## 10. Production Readiness Considerations

**Data Integrity:**
* Command lifecycle state transitions are atomic (no partial state)
* Audit events are written before command completion (durability)
* Snapshot anchors are recorded correctly

**Failure Modes:**
* **Policy Engine Failure**: Command denied, clear error returned
* **Audit Log Failure**: Command queued or failed, never silently executed without audit
* **Snapshot Failure**: Command succeeds, snapshot failure logged and alerted
* **Database Failure**: Transaction rolled back, clear error, retryable

**Observability:**
* Metrics: command lifecycle latency (propose/authorize/apply), session operations, error rates by service
* Logs: every service method invocation (method, actor, result, latency)
* Traces: distributed tracing through service layer to domain/policy/audit/snapshot

**Scalability:**
* Horizontal scaling: stateless services can be replicated
* Vertical scaling: more CPU for orchestration logic, more connections to PostgreSQL
* Async I/O: enables high concurrency without thread explosion

---

## 11. Risk Profile

**Risk 1: Layer Violation (CRITICAL)**
* **Scenario**: Transport layer bypasses service layer to access domain/policy/store directly
* **Impact**: Authorization bypass, audit gaps, data corruption
* **Mitigation**: Code review enforcement, architectural tests, layer dependency constraints

**Risk 2: Orchestration Bug (CRITICAL)**
* **Scenario**: Service layer skips authorization or audit step
* **Impact**: Unauthorized operations, compliance failure, security breach
* **Mitigation**: Integration tests for every command lifecycle, state machine validation

**Risk 3: Transaction Leak (HIGH)**
* **Scenario**: Database transaction not committed or rolled back
* **Impact**: Connection exhaustion, deadlocks, system unavailability
* **Mitigation**: Transaction guards (Drop impl), timeouts, connection pool monitoring

**Risk 4: Error Propagation Bug (MEDIUM)**
* **Scenario**: Service layer swallows errors or returns wrong error type
* **Impact**: Silent failures, incorrect diagnostics, operational confusion
* **Mitigation**: Typed errors (Result<T, ServiceError>), error propagation tests

**Risk 5: Performance Degradation (MEDIUM)**
* **Scenario**: Synchronous orchestration blocks on slow I/O
* **Impact**: High latency, timeout failures, poor user experience
* **Mitigation**: Async service methods, timeout enforcement, performance benchmarks

---

## 12. Open Questions (If Any)

1. **Async Orchestration**: Should service layer use async/await or synchronous blocking calls?
   * Decision needed by: CTO (Rena) + Backend Engineer (Dmitri)

2. **Transaction Scope**: Should transactions span entire command lifecycle or be narrower?
   * Decision needed by: Domain Architect (Yusuf) + Infrastructure Architect (Nadia)

3. **Error Granularity**: How detailed should service-level errors be (expose domain errors vs. abstract)?
   * Decision needed by: API Architect (Marcus) + CTO (Rena)

4. **Snapshot Trigger**: Should snapshots be triggered synchronously (block command) or asynchronously?
   * Decision needed by: Data Architect (Chen) + Backend Engineer (Dmitri)

5. **Scope Caching**: Should ActorScope be cached or rebuilt on every authorization request?
   * Decision needed by: Security Architect (Fatima) + Infrastructure Architect (Nadia)

---

## Appendix: Timeline and Status

* **Document Created:** 2025-01-XX
* **Last Updated:** 2025-01-XX
* **Author:** Architecture Team
* **Reviewers:**
  * [ ] Rena Okafor (CTO) - Architecture alignment and layer discipline
  * [ ] Yusuf Osman (Domain Architect) - Service-domain integration
  * [ ] Alex Rivera (Project Manager) - Workflow coordination and handoffs

**Next Step:** CTO review and sign-off before proceeding to Session 2 (question generation)

---

**END OF CONTROL PLANE AREA CONTEXT**
