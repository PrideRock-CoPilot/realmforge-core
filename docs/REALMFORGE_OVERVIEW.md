# RealmForge: AI-Native Software Construction Platform

**Document Purpose:**  
This document provides a comprehensive overview of RealmForge - what it is, what problems it solves, its core capabilities, architecture, and component areas. This serves as the foundation for mapping RealmForge capabilities to software development pain points.

**Created:** 2025-05-08  
**Status:** Overview Foundation  
**Source Documents:** realm_forge_ai_native_path_forward.md + 10 Area AREA_CONTEXT.md files

---

## Executive Summary

**RealmForge is an AI-native software construction platform that makes AI software execution governable.**

While Git made human software collaboration scalable, RealmForge makes AI software development governable, traceable, and safe. It is not just an AI coding assistant or low-code builder - it is a new software construction substrate where:

* **Humans** describe and inspect systems through familiar visual interfaces
* **AI agents** operate through a backend designed for scoped context, permissions, traceability, rollback, and cost control
* **Project state** is the source of truth, not just the codebase
* **Runtime behavior** is governed by signed, versioned, queryable state
* **Rollback** restores complete project state, not just source files

---

## Core Thesis

**"Git made human software collaboration scalable. RealmForge should make AI software execution governable."**

Current AI development tools place AI into human-native workflows (Git, PRs, CI/CD). This is useful but not AI-native.

Git tracks text changes but doesn't understand:
* Intent (why a change was made)
* Capability ownership
* Work paths
* Agent permissions
* Runtime policies
* Semantic rollback
* Test evidence
* Trace points
* Feature-level state
* Agent cost
* Blast radius

RealmForge addresses these gaps with a governed project state model designed for AI agents from the ground up.

---

## What Problems Does RealmForge Solve?

### 1. **Intent and Context Loss**
* **Problem:** Git knows "this line changed" but not "why" or "what else is affected"
* **RealmForge Solution:** Every change is linked to user intent, work path, capability, approval, and blast radius

### 2. **AI Permission Boundaries**
* **Problem:** AI tools can modify any file they see, with no built-in permission model
* **RealmForge Solution:** Database-backed agent credentials with scoped file access, read/write permissions, and capability boundaries

### 3. **Incomplete Rollback**
* **Problem:** Rollback is "redeploy old code" - doesn't restore decisions, permissions, config, or state
* **RealmForge Solution:** Snapshot-based rollback that restores full project state (metadata, map, files, locks, decisions, tests, policies)

### 4. **Unknown Blast Radius**
* **Problem:** Engineers don't know what their changes will break
* **RealmForge Solution:** File registry tracks dependencies, usage, ownership, risk level, and required tests

### 5. **Design-Implementation Disconnect**
* **Problem:** Design happens in isolation, then engineering discovers it can't be built
* **RealmForge Solution:** Living map connects design → approval → implementation → tests → deployment with feedback loops

### 6. **Opaque Cost**
* **Problem:** Cloud bills and engineering time are not tracked by feature or work path
* **RealmForge Solution:** Built-in cost tracking: token spend, storage spend, build spend, runtime spend, rework spend per work path

### 7. **Static Compilation for Everything**
* **Problem:** Changing a permission or rate limit requires full deployment cycle
* **RealmForge Solution:** Runtime-governed behavior via signed Parquet tables (policies, routes, permissions, contracts, trace points)

### 8. **Late QA and No Incremental Review**
* **Problem:** QA sees work only when "done", entire sprints discarded when direction was wrong
* **RealmForge Solution:** Planning gates, incremental review checkpoints (30%, 70%, 90%), evidence-based progression

---

## Core Capabilities

### **1. Living Map (The Visual Brain)**

A dynamic, queryable representation of the entire software system that adapts to project type.

**Capabilities:**
* Extracts actors, capabilities, screens, workflows, data objects, permissions, routes, tests, risks from user conversation
* Supports multiple map modes:
  * **Web App Mode:** Screens, routes, components, API calls, state, permissions
  * **API/System Mode:** Endpoints, services, workers, queues, databases, events, policies
  * **Workflow Mode:** Actors, steps, approvals, decisions, handoffs, exceptions, SLA/rules
  * **Document/Knowledge Mode:** Sources, ingestion, chunking, indexing, retrieval
  * **Platform/OS Mode:** Subsystems, capabilities, services, contracts, data stores, runtime boundaries
* Real-time visualization of runtime behavior (trace events, policy results, failure points)

**Why it matters:** Replaces tribal knowledge with queryable, always-current system understanding

---

### **2. Work Paths (Structured Feature Blueprints)**

Visual, reusable, agent-ready execution maps that connect design to implementation.

**Capabilities:**
* Work path nodes represent: features, screens, components, API routes, services, database tables, permissions, tests, decisions, risks, release gates
* Work path edges represent: depends_on, calls, renders, reads_from, writes_to, protects, configures, tests, documents, requires_decision
* Each node includes:
  * Files it creates/modifies
  * Contracts it consumes/produces
  * Tests required
  * Allowed/forbidden agents
  * Risk level
  * Owner capability

**Why it matters:** Gives agents exact scoped construction paths instead of "here's the whole repo, good luck"

---

### **3. File Registry (Blast Radius Awareness)**

Every file knows why it exists and who uses it.

**Capabilities:**
* Track file metadata: owner capability, risk level, lock state, generated/managed/hand-authored
* Track usage: used by nodes, used by routes, required tests, trace points
* Track permissions: allowed agents, allowed modifiers, requires review by whom
* Track versions: last approved hash, last modified actor, rollback snapshots

**Why it matters:** Agents know blast radius before making changes

---

### **4. Planning Module (Intent Capture)**

Turns user intent into agent-ready execution map before implementation starts.

**Capabilities:**
* Captures: what is being built, why it exists, which system area owns it, which routes involved, which files created/modified, which tests required, which agents allowed, assumptions locked, open questions blocking execution
* Enforces planning gates before execution:
  * intake_complete
  * system_scan_complete
  * route_map_complete
  * file_touch_map_complete
  * test_plan_complete
  * risk_review_complete
  * council_review_complete
  * user_signoff_complete
* Planning lifecycle: Draft → Mapped → Reviewed → Approved → In Execution → Completed → Released

**Why it matters:** No implementation starts until plan is approved and all blockers resolved

---

### **5. Agent Work Packets (Scoped Permissions)**

Agents receive minimal permissioned context instead of full repo access.

**Capabilities:**
* Scoped credentials per agent role (frontend_agent, backend_auth_agent, qa_agent, security_reviewer, release_manager)
* Explicit allowed file paths (only files they should modify)
* Explicit denied paths (cannot touch)
* Required contracts to follow
* Required trace points to emit
* Rollback anchor for this work
* Read permissions (what they can see)
* Write permissions (what they can modify)

**Why it matters:** Database-backed permission model, not prompt-only rules

---

### **6. Snapshot-Based Rollback (True State Restoration)**

Immutable, queryable snapshots of complete project state.

**Capabilities:**
* Snapshot contents: work paths, nodes, edges, files, routes, tests, decisions, agent permissions, render manifests, evidence runs, trace points, standards checks, violations, lock states, file hashes, cost metadata
* Snapshot granularity: full project, capability, work path, file, agent step, runtime bundle
* Validation after rollback: metadata consistency, file hash verification, standards rules, affected tests, trace replay, map status reconciliation
* Parquet-backed storage for fast, queryable, column-oriented access

**Why it matters:** Rollback restores the "project brain", not just source files

---

### **7. Runtime Governance (Parquet-Backed Behavior)**

Runtime behavior governed by signed, versioned, queryable Parquet tables instead of only compiled code.

**Capabilities:**
* Runtime tables: routes, flows, policies, permissions, contracts, trace_points, feature_flags, file_registry, work_path_nodes, work_path_edges, standards
* Runtime engine loads signed bundle, validates hashes, enforces policies before handler execution
* Parquet-only updates (no recompilation): permissions, feature flags, trace verbosity, some policies, navigation, rate limits, workflow ordering
* Compiled code for: handler logic, password hashing, database adapters, custom business logic, external integrations, UI components

**Why it matters:** Separate "what can change without recompilation" from "what requires rebuild"

---

### **8. Governed Runtime Bundle**

Build output is not just compiled code - it's a signed, versioned, validated bundle.

**Capabilities:**
* Bundle contents:
  * `compiled/` - Compiled source for web, api, workers
  * `runtime/` - Parquet tables for routes, policies, permissions, contracts, trace points
  * `evidence/` - Build report, standards report, test report, dependency report, cost report
  * `docs/` - System map, capability documentation, release notes
  * `manifest/` - Build manifest, render manifest, rollback manifest
* Runtime refuses to boot if: snapshot unsigned, manifest hash fails, required tables missing, approved status false, handler bindings don't match compiled handlers

**Why it matters:** Runtime enforces governance, not just "trust the code"

---

### **9. Cost Tracking (Built-In FinOps)**

Cost is tracked at work path, agent step, and feature level.

**Capabilities:**
* Track: token spend (LLM calls), storage spend (snapshots, objects), build spend (CI/CD), runtime spend (hosting), rework spend (discarded work)
* Cost per task, cost per flow, cost per agent
* Reuse savings (when modules or patterns are reused)
* Cost reports in every snapshot and runtime bundle

**Why it matters:** Feedback loop to improve estimation and identify waste

---

### **10. Tracing and Observability**

Every flow defines planned trace points. Runtime emits redacted trace events.

**Capabilities:**
* Trace points defined in work paths (not added after the fact)
* Redaction policies (forbidden fields: password, token, session_secret)
* Map shows live runtime behavior (which trace points passed/failed)
* Trace replay for rollback validation

**Why it matters:** Observability is designed in, not bolted on

---

## Architecture: The 10 Core Areas

RealmForge is built as a layered Rust architecture with 10 core areas:

### **Transport Layer (External Interfaces)**

#### **1. api-mcp-cli (API, MCP, CLI Transport)**
* **Owner:** Marcus Webb (API Architect)
* **Purpose:** Thin transport adapters for external access
* **Components:**
  * **REST API** (Axum) - HTTP/JSON API for web/mobile clients
  * **MCP Server** (Model Context Protocol) - LLM tool surface for AI agents
  * **Operator CLI** (Clap) - Command-line interface for operators
* **Responsibility:** Route requests to control-plane, serialize/deserialize, auth token validation
* **Does NOT:** Business logic, database access, policy evaluation

#### **2. frontend (React UI)**
* **Owner:** Kai Sato (Frontend Engineer)
* **Purpose:** Human-facing visual interface for RealmForge
* **Components:**
  * **Living Map Visualizer** - Interactive system map with multiple view modes
  * **Work Path Editor** - Visual editor for feature blueprints
  * **Approval Dashboard** - Review and approve designs, plans, changes
  * **Cost Reports** - Real-time cost tracking by work path
  * **Trace Viewer** - Runtime trace event visualization
* **Responsibility:** WCAG 2.1 AA accessible UI, responsive design, performance budgets
* **Does NOT:** Business logic, direct database access, policy enforcement

---

### **Service Layer (Orchestration)**

#### **3. control-plane (Command Lifecycle Orchestration)**
* **Owner:** Rena Okafor (CTO)
* **Purpose:** Service layer that orchestrates command execution across layers
* **Components:**
  * **Command Lifecycle Manager** - Routes commands through validation → policy → execution → audit → snapshot
  * **Work Path Orchestrator** - Sequences agent work packets, manages handoffs
  * **Planning Gate Enforcer** - Blocks execution until all gates satisfied
  * **Agent Credential Manager** - Issues scoped credentials to agents
* **Responsibility:** Workflow coordination, layer discipline enforcement, agent permission scoping
* **Does NOT:** Authorization logic (delegates to policy), persistence (delegates to stores)

---

### **Policy and Domain Layers (Core Logic)**

#### **4. policy (Authorization Engine)**
* **Owner:** Fatima Al-Hassan (Security Architect)
* **Purpose:** Authorization evaluation before every privileged operation
* **Components:**
  * **Policy Rule Engine** - Evaluates rules (condition → grant/deny)
  * **Conditional Grant Handler** - Time-limited, scope-limited, revocable grants
  * **Denial Reason Emitter** - Clear, actionable denial codes
  * **Policy Cache** - Hot cache for frequently evaluated rules
* **Responsibility:** Answer "is actor X allowed to do Y on resource Z?"
* **Does NOT:** Persistence (uses policy store), identity (uses authority domain)

#### **5. authority (Domain Model)**
* **Owner:** Yusuf Osman (Domain Architect)
* **Purpose:** Pure Rust domain types, state machines, invariants
* **Components:**
  * **Typed IDs** - ActorID, SessionID, CommandID, ResourceID, SnapshotID, PolicyID
  * **ActorScope** - Actor identity with session and bounded grants
  * **BoundedCommand** - Command scoped to actor, resource, timestamp
  * **State Machines** - Command lifecycle, planning lifecycle, approval workflow
* **Responsibility:** Domain language, invariants, type safety
* **Does NOT:** I/O, database access, HTTP, file system

---

### **Persistence Layer (State Management)**

#### **6. catalog (Resource Metadata)**
* **Owner:** Chen Wei (Data Architect)
* **Purpose:** Queryable resource metadata, namespace hierarchy, discovery, lineage
* **Components:**
  * **Resource Registry** - All resources (files, routes, services, tests)
  * **Namespace Hierarchy** - Projects → capabilities → work paths → nodes
  * **Discovery Index** - Fast search by name, tag, owner, type
  * **Lineage Tracker** - Which resources depend on which
* **Responsibility:** "What resources exist?" and "How are they related?"
* **Does NOT:** File contents (uses rfsource), audit events (uses audit), policies (uses policy)

#### **7. audit (Immutable Event Log)**
* **Owner:** Fatima Al-Hassan (Security Architect)
* **Purpose:** Tamper-evident audit trail for compliance and forensics
* **Components:**
  * **Hash-Chained Event Log** - Each event includes hash of previous event
  * **Event Schema** - who, what, when, where, why, result, denial_reason
  * **Retention Policy** - Automated archival after retention period
  * **Forensic Query API** - Time-range, actor, resource, outcome queries
* **Responsibility:** "What happened?" with cryptographic integrity guarantee
* **Does NOT:** Authorization (uses policy), real-time alerting (separate concern)

#### **8. snapshot (Version Ledger)**
* **Owner:** Dmitri Volkov (Backend Engineer)
* **Purpose:** Point-in-time state capture for rollback, audit, and comparison
* **Components:**
  * **Snapshot Creator** - Captures full project state at trigger events
  * **Version Chain Tracker** - Parent-child relationships between snapshots
  * **Rollback Engine** - Restores project state from snapshot
  * **Validation Framework** - Post-rollback integrity checks
* **Responsibility:** "What was the state at time T?" and "Restore to snapshot S"
* **Does NOT:** File contents (uses rfsource), real-time data (uses catalog)

#### **9. rfsource (Content-Addressed Object Store)**
* **Owner:** Dmitri Volkov (Backend Engineer)
* **Purpose:** Immutable, content-addressed storage for file versions
* **Components:**
  * **Content Addresser** - SHA-256 hash-based addressing
  * **Immutable Block Storage** - Write-once, read-many storage
  * **Deduplication** - Same content = same hash = stored once
  * **Retrieval API** - Fetch by hash, verify integrity
* **Responsibility:** "Store file content" and "Retrieve file content by hash"
* **Does NOT:** Metadata (uses catalog), versioning logic (uses snapshot)

**Note:** rfsource AREA_CONTEXT.md currently describes it as "Parquet-backed" which needs correction based on user feedback.

---

### **Infrastructure Layer (Deployment)**

#### **10. infrastructure (Database, Observability, Deployment)**
* **Owner:** Nadia Kovacs (Infrastructure Architect)
* **Purpose:** PostgreSQL database architecture, observability stack, deployment automation
* **Components:**
  * **PostgreSQL Database** - Primary persistence for all metadata
  * **Observability Stack:**
    * Prometheus (metrics)
    * Grafana (dashboards)
    * Loki (logs)
    * Jaeger (traces)
  * **Deployment Automation:**
    * Terraform (infrastructure as code)
    * CI/CD pipelines
    * Database migrations
  * **Operational Runbooks** - Incident response, backup/restore, scaling procedures
* **Responsibility:** Infrastructure provisioning, observability, operational procedures
* **Does NOT:** Application logic (separate layers)

---

## Data Flow: How the Layers Work Together

### Example: User Requests "Add Login Feature"

```
1. FRONTEND (Kai)
   User describes intent → Living Map extracts requirements
   ↓
2. CONTROL-PLANE (Rena)
   Creates work path → Validates planning gates → Issues agent work packet
   ↓
3. POLICY (Fatima)
   Checks "Can agent modify auth files?" → Grant with bounded scope
   ↓
4. AUTHORITY (Yusuf)
   Creates BoundedCommand with ActorScope → Validates invariants
   ↓
5. CATALOG (Chen)
   Queries which files exist → Returns auth-related file registry
   ↓
6. CONTROL-PLANE
   Scopes agent work packet: allowed files = [login.py, auth_service.py, test_login.py]
   ↓
7. AGENT EXECUTION
   Agent writes code → Tests → Submits evidence
   ↓
8. AUDIT (Fatima)
   Records: agent X modified files Y under approval Z
   ↓
9. SNAPSHOT (Dmitri)
   Captures state: work path nodes, file hashes, test results
   ↓
10. RFSOURCE (Dmitri)
    Stores file contents by SHA-256 hash
    ↓
11. FRONTEND (Kai)
    Updates Living Map → Shows "Login feature 80% complete"
```

---

## Layer Discipline (The Rust Law)

**Strict layer ordering enforced by Rena (CTO):**

```
api/mcp/cli  →  control-plane  →  policy  →  authority (domain)
                                              ↓
                                         catalog, audit, snapshot, rfsource
                                              ↓
                                         PostgreSQL
```

**Rules:**
* Transport layer does NOT call store layer directly (must go through control-plane)
* Policy engine does NOT do persistence (uses policy store)
* Domain layer does NOT do I/O (pure Rust types)
* Store layers do NOT do authorization (use policy)

**Why:** Prevents bypass paths, makes testing easier, enables layer replacement

---

## Key Differentiators vs. Traditional Tools

| Traditional Approach | RealmForge Approach |
|---------------------|-------------------|
| Source code is truth | Governed project state is truth |
| Git tracks text changes | RealmForge tracks intent, context, blast radius |
| AI can modify any file | AI has database-backed scoped permissions |
| Rollback = redeploy old code | Rollback = restore full project state (metadata + files + config) |
| Tests written after code | Tests required before implementation starts |
| Design in mockups, disconnect from build | Living map connects design → approval → implementation |
| Cost tracked in cloud bills | Cost tracked per work path, agent step, feature |
| All behavior is compiled code | Runtime behavior governed by signed Parquet tables |
| Permissions hardcoded | Permissions are runtime-updateable data |
| QA sees work when "done" | Incremental review gates at 30%, 70%, 90% |
| No agent permission model | Agents receive scoped work packets with explicit allowed/denied files |

---

## Production Readiness Philosophy

Every area has explicit success criteria across 4 dimensions:

1. **Correctness:** Does it do what it's supposed to do?
2. **Performance:** Does it meet latency, throughput, and resource targets?
3. **Reliability:** Does it handle failures gracefully?
4. **Security:** Does it prevent unauthorized operations?

Every area defines:
* **Critical failure scenarios** (what happens when this breaks)
* **Mitigation strategies** (how to prevent or recover)
* **Observability requirements** (metrics, logs, traces)
* **Scalability considerations** (how it grows with load)

---

## Open Questions and Decisions

RealmForge uses a structured decision register:
* `docs/spec/22_OPEN_DECISIONS.md` tracks unresolved design decisions
* Decisions tagged: `USER_APPROVAL_REQUIRED` or `COUNCIL_DECISION_REQUIRED`
* No implementation starts until blocking decisions resolved

**Current decision areas:**
* Frontend stack selection
* Parquet engine/library choice
* Catalog order beyond Login capability
* Brand-removal threshold (when to remove RealmForge branding from generated apps)
* Live Watch remediation tiers (autonomy levels for AI agents)

---

## Identified Gaps (From User Feedback)

### Gap 1: Design Approval and Revision Workflow
* **Problem:** No mechanism to route design for human sign-off before build
* **Current State:** Not yet addressed in any area
* **Required Area:** Likely frontend (UI for approval routing) + control-plane (approval workflow orchestration)

### Gap 2: Incremental Review Gates
* **Problem:** No structured review at 30%, 70%, 90% completion
* **Current State:** Planning gates exist, but no incremental checkpoints during execution
* **Required Area:** control-plane (gate enforcement) + frontend (progress visualization)

### Gap 3: RFSource Storage Model
* **Problem:** rfsource AREA_CONTEXT.md describes it as "Parquet-backed" which is incorrect
* **Current State:** Needs architectural review to clarify actual storage model
* **Action Required:** Read architectural docs, correct AREA_CONTEXT.md

---

## Summary: What is RealmForge?

RealmForge is a **governed project state platform** that makes AI-assisted software development:

* **Visible:** Living map shows what exists and how it's related
* **Scoped:** Agents receive minimal permissioned work packets
* **Traceable:** Every change linked to intent, approval, blast radius
* **Rollbackable:** Snapshots restore full project state, not just code
* **Governed:** Runtime behavior controlled by signed, versioned tables
* **Cost-Aware:** Track spend by work path, agent, feature
* **Safe:** Layer discipline prevents bypass paths, policy enforced before execution

It's not replacing Git - it's adding the governance layer that Git wasn't designed to provide.

**Target users:**
* **Humans** - See visual maps, approve plans, review costs, inspect traces
* **AI Agents** - Receive scoped work packets, emit evidence, respect boundaries
* **Security teams** - Audit trail, policy enforcement, permission boundaries
* **Finance teams** - Cost attribution by feature, rework tracking
* **Operations teams** - Rollback capability, runtime governance, observability

---

## Document Status

**COMPLETE** — Ready for capability mapping to SOFTWARE_DEVELOPMENT_LANDSCAPE.md pain points

**Next Step:** Create REALMFORGE_CAPABILITY_MAPPING.md that maps:
1. Each RealmForge capability to specific SDLC pain points it solves
2. Which areas implement which capabilities
3. What gaps remain (pain points RealmForge doesn't yet address)
4. What new capabilities RealmForge introduces (innovation beyond known pain points)
