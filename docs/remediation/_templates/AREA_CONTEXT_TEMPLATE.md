# [AREA NAME] — Area Context Document

**Purpose:** Define the intent, responsibilities, and boundaries of this area to guide audit question generation.

**Status:** 📋 CONTEXT DEFINITION

**Important:** This document captures WHAT this area is supposed to do (intent), NOT what currently exists (implementation). Questions will be generated from this context to test production readiness against standards, not against current code.

---

## 1. Purpose

**What is this area for? Why does it exist?**

[2-3 paragraph description of the area's purpose in the RealmForge system. Focus on the problem it solves and the value it provides. Avoid implementation details.]

**Example:**
> The Authority domain defines the governance model for RealmForge. It exists to answer "who can do what" by modeling grants, scopes, and policy evaluation. This area is the source of truth for all authorization decisions across the system.

---

## 2. Responsibilities

**What specific capabilities does this area own?**

* [Responsibility 1 - be specific about capabilities]
* [Responsibility 2]
* [Responsibility 3]
* [...]

**Example (for Authority domain):**
* Define and enforce the grant model (subjects, resources, actions, conditions)
* Manage grant lifecycle (creation, revocation, expiration)
* Evaluate authorization requests ("can Subject X perform Action Y on Resource Z?")
* Maintain grant consistency and integrity constraints

---

## 3. Boundaries (What This Area Does NOT Own)

**What is explicitly out of scope?**

* [Boundary 1 - what this area does NOT do]
* [Boundary 2]
* [Boundary 3]
* [...]

**Example (for Authority domain):**
* Does NOT store grants persistently (that's the responsibility of the store layer)
* Does NOT handle authentication (that's the responsibility of the identity layer)
* Does NOT implement UI or API endpoints (those are thin transport layers)

---

## 4. Programming Language(s)

**What languages are used in this area?**

* **Primary:** [e.g., Rust, Python, SQL]
* **Secondary:** [if applicable]

**Why these languages?**
[Brief justification — e.g., "Rust for type safety and performance in domain logic"]

---

## 5. High-Level Architecture

**What is the structural design of this area?**

* **Layer:** [e.g., Domain layer, Service layer, Infrastructure layer]
* **Key Components:** [List major logical components without file names]
  * [Component 1 and its purpose]
  * [Component 2 and its purpose]
  * [Component 3 and its purpose]
* **Data Flow:** [High-level description of how data moves through this area]

**Example (for Authority domain):**
* **Layer:** Domain layer (pure logic, no I/O)
* **Key Components:**
  * Grant model: Represents authorization grants with typed subjects, resources, actions
  * Policy evaluator: Determines if a grant satisfies an authorization request
  * Grant registry: In-memory collection of active grants with fast lookup
* **Data Flow:** Grants enter via service layer → stored in domain registry → evaluated on authorization requests → results returned to caller

---

## 6. Key Concepts

**What are the core domain concepts, types, or abstractions?**

* **[Concept 1]:** [Definition and purpose]
* **[Concept 2]:** [Definition and purpose]
* **[Concept 3]:** [Definition and purpose]
* [...]

**Example (for Authority domain):**
* **Grant:** A tuple (Subject, Resource, Action, Conditions) representing permission
* **Subject:** The entity performing an action (user, service, agent)
* **Resource:** The entity being acted upon (object, collection, namespace)
* **Action:** The operation being performed (read, write, delete, execute)
* **Scope:** Hierarchical namespace for grant organization and policy inheritance

---

## 7. Success Criteria

**How do you know this area is working correctly?**

Focus on **behavioral outcomes**, not implementation details.

* **Correctness:** [What correct behavior looks like]
* **Performance:** [Expected performance characteristics]
* **Reliability:** [How it should behave under failure]
* **Security:** [Security guarantees it must provide]

**Example (for Authority domain):**
* **Correctness:** Authorization decisions are deterministic, transitive, and audit-traceable
* **Performance:** Grant evaluation completes in <1ms for 99th percentile requests
* **Reliability:** Grant state remains consistent even under concurrent modifications
* **Security:** No bypass possible; all authorization must go through policy evaluation

---

## 8. Dependencies

**What other areas does this area depend on?**

* **[Dependency 1]:** [Why it's needed, what's consumed]
* **[Dependency 2]:** [Why it's needed, what's consumed]
* [...]

**Example (for Authority domain):**
* **Policy Engine:** Evaluates complex policy rules for conditional grants
* **Audit Log:** Records all grant changes and authorization decisions for compliance
* **Store Layer:** Persists grant state to PostgreSQL for durability

---

## 9. Consumers

**What other areas depend on this area?**

* **[Consumer 1]:** [What they consume, why]
* **[Consumer 2]:** [What they consume, why]
* [...]

**Example (for Authority domain):**
* **Control Plane:** Uses authority for orchestration authorization
* **API Layer:** All REST endpoints check authorization before executing
* **CLI:** Commands must be authorized before execution

---

## 10. Production Readiness Considerations

**What are the critical requirements for this area to be production-ready?**

* **Data Integrity:** [What must remain consistent]
* **Failure Modes:** [How it should behave when things go wrong]
* **Observability:** [What must be visible to operators]
* **Scalability:** [Expected load and growth patterns]

**Example (for Authority domain):**
* **Data Integrity:** Grant state must never allow unauthorized access due to race conditions
* **Failure Modes:** If policy evaluation fails, default to deny (fail-closed)
* **Observability:** All authorization decisions logged with grant ID, subject, resource, action, result
* **Scalability:** Support 10K+ grants with <1ms evaluation time; horizontally scalable via read replicas

---

## 11. Risk Profile

**What are the highest-risk failure scenarios for this area?**

* **Risk 1:** [Scenario and impact]
* **Risk 2:** [Scenario and impact]
* **Risk 3:** [Scenario and impact]

**Example (for Authority domain):**
* **Privilege Escalation:** Incorrect grant evaluation could allow unauthorized actions (CRITICAL)
* **Performance Degradation:** Slow authorization checks block all operations (HIGH)
* **Grant Loss:** Database failure without WAL could lose recent grant changes (HIGH)

---

## 12. Open Questions (If Any)

**What aspects of this area's design are still undecided?**

* [Question 1]
* [Question 2]
* [...]

**Note:** If significant design questions remain open, consider resolving them before generating audit questions.

---

## Appendix: Timeline and Status

* **Document Created:** [YYYY-MM-DD]
* **Last Updated:** [YYYY-MM-DD]
* **Author:** [Name or Role]
* **Reviewers:** [List of reviewers who validated this context]

---

**END OF AREA CONTEXT TEMPLATE**
