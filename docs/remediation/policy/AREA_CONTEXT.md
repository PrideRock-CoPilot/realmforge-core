# Policy Engine — Area Context Document

**Purpose:** Define the intent, responsibilities, and boundaries of the Policy Engine to guide audit question generation.

**Status:** 📋 CONTEXT DEFINITION  
**Created:** 2025-01-XX  
**Author:** Architecture Team  
**Reviewers:** Fatima Al-Hassan (Security Architect), Yusuf Osman (Domain Architect), Rena Okafor (CTO)

**Important:** This document captures WHAT the Policy Engine is supposed to do (intent), NOT what currently exists (implementation). Questions will be generated from this context to test production readiness against standards, not against current code.

---

## 1. Purpose

The Policy Engine is RealmForge's authorization enforcement layer - the guardian that answers "can this actor perform this action on this resource?" before any command executes. It exists as the mandatory checkpoint between all user/agent requests and their execution, ensuring that every operation is explicitly authorized according to grants, conditions, and policies.

The Policy Engine solves three critical problems:
1. **Security Enforcement**: Prevents unauthorized operations by requiring explicit grants before execution
2. **Policy Composition**: Evaluates complex conditional logic (time-based, scope-based, approval-required) across multiple policy dimensions
3. **Auditability**: Provides deny reasons and decision trails for every authorization check

The Policy Engine is NOT a database or storage layer - it is a pure evaluation engine that consumes grant state and produces authorization decisions.

---

## 2. Responsibilities

The Policy Engine owns:

* **Authorization Evaluation**: Determine if a (subject, action, resource, context) tuple is authorized
* **Policy Decision Output**: Return explicit ALLOW or DENY with detailed reasoning
* **Multi-Check Pipeline**: Evaluate authorization through sequential checks (expiry, staleness, permissions, approval state)
* **Denial Code Generation**: Provide machine-readable denial reasons (Expired, Unauthorized, ApprovalRequired, etc.)
* **Conditional Grant Logic**: Evaluate time windows, scope constraints, execution mode requirements
* **Policy Chain Evaluation**: Compose multiple policy checks into a single authorization pipeline
* **Rate Limiting Enforcement**: Check and enforce rate limits on command execution (future capability)
* **Context-Aware Decisions**: Consider session state, skill context, proposal vs. execution mode

---

## 3. Boundaries (What This Area Does NOT Own)

The Policy Engine explicitly does NOT:

* **Grant Storage**: Does not persist grants - consumes grant state from authority domain
* **Audit Logging**: Does not write audit events - returns decisions for audit service to log
* **Session Management**: Does not create or manage sessions - validates provided session context
* **Command Execution**: Does not execute commands - only authorizes them
* **User Interface**: Has no UI - accessed only through service layer
* **Grant Lifecycle**: Does not create, modify, or revoke grants - only reads them for evaluation
* **Skill Registration**: Does not manage skill catalog - validates skill context from authority domain

---

## 4. Programming Language(s)

* **Primary:** Rust
  * Chosen for: Type safety in authorization logic, zero-cost abstractions, fearless concurrency for parallel policy evaluation
  * Critical for security layer where bugs can cause privilege escalation or denial of service

**Why Rust?**  
Authorization decisions are binary and permanent - a bug that allows unauthorized access cannot be rolled back. Rust's type system ensures that policy evaluation is correct by construction. The borrow checker prevents race conditions in concurrent policy checks. The performance enables sub-millisecond authorization latency even with complex policy chains.

---

## 5. High-Level Architecture

* **Layer:** Policy/Authorization Layer (between service layer and domain model)
* **Position:** Mandatory checkpoint - all commands flow through policy engine before execution

**Key Components:**

1. **Policy Evaluator**
   * Accepts authorization request: (actor_scope, action, resource, command_context)
   * Executes policy check pipeline (6 checks: expired, stale, unauthorized, wrong_skill, proposal_only, approval_required)
   * Returns PolicyDecision (Allowed) or PolicyDenial (Denied with code and reason)

2. **Check Pipeline**
   * Sequential checks, short-circuit on first failure
   * Each check returns Result<(), PolicyDenial>
   * Checks: session expiry, grant staleness, action authorization, skill validation, mode validation, approval state

3. **Denial Code System**
   * Machine-readable codes: Expired, StaleGrant, Unauthorized, WrongSkill, ProposalOnly, ApprovalRequired, RateLimited (future)
   * Human-readable messages for UI display
   * Context-rich details for audit trail

4. **Context Evaluator**
   * Considers execution mode (Proposal vs. Execution)
   * Validates skill session binding
   * Checks temporal constraints (time windows, expiry)

**Data Flow:**
* Service layer → authorize_action(scope, action, context) → Policy Engine → Check Pipeline → PolicyDecision/PolicyDenial → Service Layer

**No persistence** - all data flows in-memory through the evaluation pipeline.

---

## 6. Key Concepts

* **PolicyDecision**: Successful authorization result (Allowed with grant details)
* **PolicyDenial**: Failed authorization with DenialCode + human message
* **DenialCode**: Enum of machine-readable failure reasons (Expired, Unauthorized, etc.)
* **Authorization Request**: Tuple of (ActorScope, Action, Resource, CommandContext)
* **Check Pipeline**: Sequential policy checks that short-circuit on first failure
* **Execution Mode**: Proposal (preview only) vs. Execution (actual command)
* **Approval State**: ProposalOnly vs. ReadyForApproval vs. Approved (state machine gates)
* **Conditional Grant**: Grant with time windows, scope constraints, or approval requirements

---

## 7. Success Criteria

**Correctness:**
* Every authorization decision is deterministic (same inputs → same output)
* No false positives (unauthorized operations never allowed)
* Clear denial reasons for every rejection (no ambiguous failures)
* Policy checks compose correctly (no check ordering bugs)

**Performance:**
* Authorization latency p99 < 1ms (single policy check)
* Authorization latency p99 < 5ms (complex policy chain with 10+ checks)
* Support 10K+ authorization checks per second per node

**Reliability:**
* Policy engine must be stateless (no hidden state between evaluations)
* Authorization failure must be fail-closed (deny by default)
* Policy evaluation must never panic or throw unhandled errors

**Security:**
* No policy bypass possible (all operations must go through engine)
* Denial reasons must not leak sensitive information to unauthorized actors
* Policy checks must be atomic (no partial evaluations)

---

## 8. Dependencies

The Policy Engine depends on:

* **Authority Domain**: Consumes ActorScope, BoundedCommand, and grant state for evaluation
  * ActorScope provides: actor_id, session_id, project_id, tenant_id, active skills, roles
  * BoundedCommand provides: command metadata, approval state, expiry

* **Audit Log Types**: Uses AuditEvent structure for decision logging (but does not write events itself)
  * Policy Engine returns decisions; service layer logs them

* **Chrono**: For time-based policy checks (expiry, time windows)

---

## 9. Consumers

The Policy Engine is consumed by:

* **Control Service (Command Service)**: Authorizes every command before execution (propose_command, authorize_command, apply_command)
* **Session Service**: Validates session expiry during authorization
* **Skill Service**: Validates skill session binding during authorization
* **API Layer**: Checks authorization before exposing endpoints
* **MCP Server**: Authorizes tool invocations before execution
* **CLI**: Authorizes operator commands before execution

All consumers interact through the service layer - none call policy engine directly from transport layers.

---

## 10. Production Readiness Considerations

**Data Integrity:**
* Policy decisions must be reproducible (same inputs → same decision)
* Check pipeline ordering must be consistent (no flaky authorization)
* Denial reasons must match actual failure cause

**Failure Modes:**
* **Policy Evaluation Error**: Must fail-closed (deny on error, never allow)
* **Missing Grant Data**: Must deny (no implicit allows)
* **Invalid Context**: Must deny with clear error (not fail open)

**Observability:**
* Metrics: authorization latency (p50/p95/p99), allow/deny rates, denial code distribution, policy check failures
* Logs: every authorization decision (actor, action, resource, result, denial code if denied)
* Traces: distributed tracing from service layer through policy checks

**Scalability:**
* Horizontal scaling: stateless policy engine can be replicated across nodes
* Vertical scaling: optimize check pipeline for minimal CPU per evaluation
* Caching: authority domain may cache grant lookups, but policy engine itself is stateless

---

## 11. Risk Profile

**Risk 1: Authorization Bypass (CRITICAL)**
* **Scenario**: Bug in check pipeline allows unauthorized command to execute
* **Impact**: Privilege escalation, unauthorized data access, security breach
* **Mitigation**: Exhaustive unit tests for all policy checks, integration tests for check composition, fail-closed defaults

**Risk 2: False Denials (HIGH)**
* **Scenario**: Overly strict policy check rejects legitimate operations
* **Impact**: System unusable, blocking legitimate workflows, operational disruption
* **Mitigation**: Clear denial reasons, comprehensive testing of valid operation paths, rejection metrics

**Risk 3: Performance Degradation (HIGH)**
* **Scenario**: Slow policy evaluation blocks all operations
* **Impact**: High latency, timeout failures, system unavailability
* **Mitigation**: Performance benchmarks, latency monitoring, optimized check pipeline, profiling

**Risk 4: Information Leakage (MEDIUM)**
* **Scenario**: Denial messages reveal sensitive information to unauthorized actors
* **Impact**: Information disclosure, reconnaissance aid for attackers
* **Mitigation**: Sanitized denial messages, context-aware detail levels, security review of error messages

**Risk 5: Policy Check Ordering Bug (MEDIUM)**
* **Scenario**: Incorrect check ordering causes inconsistent authorization decisions
* **Impact**: Unpredictable authorization behavior, security gaps
* **Mitigation**: Documented check ordering, sequential pipeline validation, integration tests for check composition

---

## 12. Open Questions (If Any)

1. **Rate Limiting Implementation**: Should rate limiting be in policy engine or a separate middleware layer?
   * Decision needed by: Security Architect (Fatima) + CTO (Rena)

2. **Policy Caching**: Should policy engine cache grant lookups or rely on authority domain caching?
   * Decision needed by: CTO (Rena) + Infrastructure Architect (Nadia)

3. **Audit Integration**: Should policy engine write audit events directly or always through service layer?
   * Decision needed by: Security Architect (Fatima) + Domain Architect (Yusuf)

4. **Conditional Grant Language**: Do we need a DSL for complex conditional grants (beyond time windows and scopes)?
   * Decision needed by: Security Architect (Fatima) + API Architect (Marcus)

---

## Appendix: Timeline and Status

* **Document Created:** 2025-01-XX
* **Last Updated:** 2025-01-XX
* **Author:** Architecture Team
* **Reviewers:**
  * [ ] Fatima Al-Hassan (Security Architect) - Policy logic and security enforcement
  * [ ] Yusuf Osman (Domain Architect) - Integration with authority domain
  * [ ] Rena Okafor (CTO) - Architecture alignment with layered system

**Next Step:** CTO review and sign-off before proceeding to Session 2 (question generation)

---

**END OF POLICY ENGINE AREA CONTEXT**
