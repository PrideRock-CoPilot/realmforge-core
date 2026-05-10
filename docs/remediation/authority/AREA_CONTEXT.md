# Authority Domain — Area Context Document

**Purpose:** Define the intent, responsibilities, and boundaries of the Authority Domain to guide audit question generation.

**Status:** 📋 CONTEXT DEFINITION  
**Created:** 2025-01-XX  
**Author:** Architecture Team  
**Reviewers:** Yusuf Osman (Domain Architect), Fatima Al-Hassan (Security Architect), Rena Okafor (CTO)

**Important:** This document captures WHAT the Authority Domain is supposed to do (intent), NOT what currently exists (implementation). Questions will be generated from this context to test production readiness against standards, not against current code.

---

## 1. Purpose

The Authority Domain is RealmForge's identity and permission core - the pure domain model that defines who can do what, when, and under what conditions. It exists to provide the authoritative types, state machines, and invariants for all authorization decisions without coupling to infrastructure, policy evaluation, or persistence.

The Authority Domain solves three critical problems:
1. **Type Safety**: Provides strongly-typed IDs (ActorID, SessionID, ProjectID, etc.) that prevent confusion and injection
2. **State Machines**: Defines valid state transitions for commands, sessions, and skills to prevent invalid operations
3. **Domain Invariants**: Encodes business rules (session expiry, approval requirements) as compile-time guarantees

The Authority Domain is NOT an implementation - it is pure logic with no I/O, no database access, and no external dependencies beyond Rust std.

---

## 2. Responsibilities

The Authority Domain owns:

* **Typed IDs**: Strong types for all entity identifiers (ActorID, SessionID, ProjectID, TenantID, CommandID, SkillID, etc.)
* **ActorScope**: Authorization context with actor, session, project, tenant, skills, and roles
* **BoundedCommand**: Command value object with metadata, approval state, and expiry
* **State Machines**: Valid state transitions (CommandStatus, ApprovalState, ExecutionMode)
* **SkillRegistration**: Skill metadata with integrity hash and catalog
* **SkillSession**: Binding between skill and actor session
* **SkillCreator**: Skill catalog management and versioning
* **Domain Invariants**: Business rules encoded as types (e.g., session must not be expired)
* **Value Objects**: Immutable domain types (no setters, construction validates invariants)

---

## 3. Boundaries (What This Area Does NOT Own)

The Authority Domain explicitly does NOT:

* **Persistence**: Does not read/write to PostgreSQL - pure in-memory types
* **Policy Evaluation**: Does not make authorization decisions - provides types for policy engine
* **Audit Logging**: Does not write audit events - provides types for audit log
* **External Dependencies**: No Tokio, no HTTP, no database drivers - only Rust std + serde + chrono
* **I/O Operations**: No file access, network calls, or database queries
* **User Interface**: Has no UI - provides types for all layers

---

## 4. Programming Language(s)

* **Primary:** Rust
  * Chosen for: Type safety, zero-cost abstractions, algebraic data types (enums, structs)
  * Critical for domain layer where bugs can cause authorization bypass or data corruption

**Why Rust?**  
Domain models are the source of truth for business rules - a bug here corrupts the entire system. Rust's type system ensures domain invariants are enforced at compile time. The borrow checker prevents accidental mutation of value objects. The enum system provides exhaustive pattern matching for state machines.

---

## 5. High-Level Architecture

* **Layer:** Domain Layer (pure logic, no I/O)
* **Position:** Core types consumed by policy engine, service layer, and transport layers

**Key Components:**

1. **Typed IDs (ids.rs)**
   * ActorID, SessionID, ProjectID, TenantID, CommandID, SkillID, ScopeID, RoleID
   * Each ID is a newtype wrapper around Uuid
   * Prevents mixing IDs (e.g., using ActorID where SessionID expected)

2. **ActorScope (scope.rs)**
   * Complete authorization context: actor_id, session_id, project_id, tenant_id, skill_sessions, roles
   * Used by policy engine to evaluate authorization

3. **BoundedCommand (command.rs)**
   * Value object: command_id, actor_id, action, resource, status, approval_state, expiry
   * State machine: Proposed → Authorized → Applied
   * Approval state: ProposalOnly → ReadyForApproval → Approved

4. **State Enums (state.rs)**
   * CommandStatus: Proposed, Authorized, Applied, Denied, Revoked
   * ApprovalState: ProposalOnly, ReadyForApproval, Approved
   * ExecutionMode: Proposal, Execution

5. **SkillRegistration (skill.rs)**
   * Skill metadata: skill_id, name, version, integrity_hash, approval_state
   * Skill integrity: hash of skill code ensures no tampering

6. **SkillSession (skill.rs)**
   * Binding: skill_id, session_id, activated_at, expires_at
   * Validates skill is active during command execution

7. **SkillCreator (skill_creator.rs)**
   * Skill catalog: register, update, version skills
   * Catalog versioning: track skill evolution over time

**Data Flow:**
* Domain types are constructed by service layer from database records
* Domain types are passed to policy engine for evaluation
* Domain types are passed to audit log for event formatting
* Domain types are serialized by transport layers for API/CLI output

**No I/O** - all data flows in-memory as Rust structs.

---

## 6. Key Concepts

* **Typed ID**: Newtype wrapper around Uuid preventing ID confusion
* **ActorScope**: Complete authorization context for policy evaluation
* **BoundedCommand**: Command with metadata, state machine, and approval lifecycle
* **State Machine**: Valid state transitions enforced by type system
* **Value Object**: Immutable domain type with validated invariants
* **Skill**: Registered capability with integrity hash
* **Domain Invariant**: Business rule encoded as compile-time constraint

---

## 7. Success Criteria

**Correctness:**
* All domain types enforce invariants (no invalid construction)
* State machines prevent invalid transitions (compile-time errors)
* Typed IDs prevent confusion (cannot use ActorID as SessionID)

**Performance:**
* Domain type construction is zero-cost (no allocation, just stack)
* State machine transitions are zero-cost (enum discriminant change)
* Serialization/deserialization is efficient (serde)

**Maintainability:**
* Domain types are self-documenting (type names, field names)
* State machines are exhaustive (compiler enforces all cases)
* Domain logic is testable (pure functions, no I/O)

---

## 8. Dependencies

The Authority Domain depends on:

* **Rust std**: Core types (String, Vec, HashMap)
* **uuid**: UUID generation and parsing
* **serde**: Serialization/deserialization for transport
* **chrono**: Timestamp handling for expiry checks

**No I/O dependencies** - no Tokio, no database drivers, no HTTP clients.

---

## 9. Consumers

The Authority Domain is consumed by:

* **Policy Engine**: Uses ActorScope, BoundedCommand for authorization evaluation
* **Service Layer**: Constructs domain types from database records
* **Audit Log**: Uses domain types for event formatting
* **Transport Layers**: Serializes domain types for API/CLI output
* **Control Store**: Persists domain types to PostgreSQL

---

## 10. Production Readiness Considerations

**Data Integrity:**
* Domain invariants must be enforced (no invalid domain types)
* State transitions must be validated (no illegal states)
* Typed IDs must prevent confusion

**Observability:**
* Domain types implement Display for debugging
* State machines log transitions (in service layer)

---

## 11. Risk Profile

**Risk 1: Domain Invariant Violation (CRITICAL)**
* **Scenario**: Domain type constructed with invalid state
* **Impact**: Authorization bypass, data corruption, security breach
* **Mitigation**: Constructor validation, exhaustive testing, property-based testing

**Risk 2: State Machine Bug (HIGH)**
* **Scenario**: Invalid state transition allowed
* **Impact**: Commands in illegal states, operational confusion
* **Mitigation**: Enum exhaustiveness, state machine tests, validation

**Risk 3: Type Confusion (MEDIUM)**
* **Scenario**: Using wrong ID type (ActorID instead of SessionID)
* **Impact**: Authorization failures, data corruption
* **Mitigation**: Newtype wrappers, compiler type checking

---

## 12. Open Questions (If Any)

1. **Session Expiry**: Should sessions have fixed expiry or sliding window expiry?
   * Decision needed by: Domain Architect (Yusuf) + Security Architect (Fatima)

2. **Approval State Transitions**: Can commands skip ReadyForApproval and go directly to Approved?
   * Decision needed by: Domain Architect (Yusuf) + CTO (Rena)

3. **Skill Versioning**: How should skill version conflicts be handled (multiple versions active)?
   * Decision needed by: Domain Architect (Yusuf) + Backend Engineer (Dmitri)

---

## Appendix: Timeline and Status

* **Document Created:** 2025-01-XX
* **Last Updated:** 2025-01-XX
* **Author:** Architecture Team
* **Reviewers:**
  * [ ] Yusuf Osman (Domain Architect) - Domain model and state machines
  * [ ] Fatima Al-Hassan (Security Architect) - Authorization types and invariants
  * [ ] Rena Okafor (CTO) - Architecture alignment with layered system

**Next Step:** CTO review and sign-off before proceeding to Session 2 (question generation)

---

**END OF AUTHORITY DOMAIN AREA CONTEXT**
