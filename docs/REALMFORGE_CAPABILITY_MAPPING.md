# RealmForge Capability Mapping: Solutions to Software Development Pain Points

**Document Purpose:**  
This document maps RealmForge capabilities to specific software development lifecycle pain points, identifies coverage gaps, and highlights innovation areas.

**Created:** 2025-05-08  
**Status:** Capability Mapping Analysis  
**Source Documents:** 
* SOFTWARE_DEVELOPMENT_LANDSCAPE.md (pain points and SDLC)
* REALMFORGE_OVERVIEW.md (capabilities and areas)

---

## Mapping Structure

For each pain point from SOFTWARE_DEVELOPMENT_LANDSCAPE.md:
* **Pain Point:** Description of the problem
* **RealmForge Solution:** How RealmForge addresses it
* **Implementing Areas:** Which of the 10 areas provide this capability
* **Coverage Level:** ✅ Fully Addressed | 🟡 Partially Addressed | ❌ Not Addressed | 🆕 Gap Identified

---

## Section 1: Persistent Bottlenecks → RealmForge Solutions

### 2.1 Requirements and Planning: "What Are We Building?"

**Pain Point:**
* Requirements vague, incomplete, change mid-sprint
* Engineers build the wrong thing correctly
* 30-50% of sprint time spent on rework

**RealmForge Solution:**
* **Living Map** extracts actors, capabilities, screens, workflows from conversation
* **Planning Module** captures intent before implementation starts
* **Planning Gates** block execution until: intake complete, route map complete, file touch map complete, test plan complete, risk review complete, user signoff complete
* **Work Paths** create structured, visual feature blueprints that connect design to implementation

**Implementing Areas:**
* `frontend` - Living Map visualization, interactive requirement capture
* `control-plane` - Planning gate enforcement, work path orchestration
* `catalog` - Resource discovery, namespace hierarchy

**Coverage Level:** ✅ **Fully Addressed**

**Impact:** Transforms vague intent into structured, approved execution map before any code is written

---

### 2.2 Context Switching: "What Was I Working On?"

**Pain Point:**
* Engineers context switch 10-20 times per day
* Takes 15-30 minutes to rebuild mental model after interruption
* "Where did I leave off?" costs 5-10 hours per week per engineer

**RealmForge Solution:**
* **Work Path State Tracking** - Every work path node has current status
* **Agent Work Packets** - Scoped context includes exactly what agent needs (files, contracts, tests, rollback anchor)
* **Living Map** - Visual representation shows current state of all work
* **Snapshot History** - Return to exact state at any prior point

**Implementing Areas:**
* `control-plane` - Work path state management, agent work packet creation
* `frontend` - Visual work path status, "resume work" interface
* `snapshot` - State capture and restoration

**Coverage Level:** 🟡 **Partially Addressed**

**Gap:** Addresses agent context switching well, less clear how human engineers benefit from these features in day-to-day IDE use

---

### 2.3 Blast Radius: "What Will This Break?"

**Pain Point:**
* Engineers don't know the full impact of their changes
* 20-40% of deployments cause incidents
* No one knows which services depend on a given API

**RealmForge Solution:**
* **File Registry** - Every file tracks: used by nodes, used by routes, required tests, allowed modifiers, risk level
* **Dependency Tracking** - Catalog tracks which resources depend on which
* **Lineage Tracker** - Shows upstream and downstream dependencies
* **Agent Work Packets** - Include blast radius information before agent acts

**Implementing Areas:**
* `catalog` - File registry, dependency tracking, lineage
* `control-plane` - Blast radius calculation, impact analysis
* `frontend` - Visual dependency graph

**Coverage Level:** ✅ **Fully Addressed**

**Impact:** Every change shows "what else is affected" before execution

---

### 2.4 Permissions and Security: "Can I Touch That?"

**Pain Point:**
* Engineers have overly broad permissions
* Security reviews happen late, block releases
* "Who approved this change to the auth logic?" is unanswerable

**RealmForge Solution:**
* **Database-Backed Permissions** - Agent credentials with scoped views (frontend_agent_view, backend_auth_agent_view, etc.)
* **Agent Work Packets** - Explicit allowed file paths, explicit denied paths
* **Policy Engine** - Evaluates "is actor X allowed to do Y on resource Z?" before every operation
* **Audit Log** - Immutable, hash-chained record of who did what when with what approval
* **File Ownership** - Every file has allowed modifiers and requires review by specific roles

**Implementing Areas:**
* `policy` - Authorization evaluation, conditional grants, denial reasons
* `audit` - Immutable event log, forensic queries
* `control-plane` - Agent credential management, scoped work packets
* `catalog` - File ownership registry

**Coverage Level:** ✅ **Fully Addressed**

**Impact:** Security is design constraint, not late gate. Every action is audited and permission-checked.

---

### 2.5 Testing Gaps: "Tests Pass But Production Breaks"

**Pain Point:**
* Unit tests test mocks, not reality
* 40-60% of production bugs could have been caught
* Tests rot faster than code

**RealmForge Solution:**
* **Test-First Planning** - Work paths require test plan before implementation
* **Test Registry** - Required tests tracked in file registry and work path nodes
* **Evidence-Based Progression** - Can't mark work complete without test evidence
* **Planning Gates** - test_plan_complete gate blocks execution
* **Snapshot Validation** - Post-rollback automated test execution

**Implementing Areas:**
* `control-plane` - Planning gate enforcement, evidence validation
* `catalog` - Test registry, required test tracking
* `snapshot` - Post-rollback test execution

**Coverage Level:** 🟡 **Partially Addressed**

**Gap:** Addresses *when* tests are required, but doesn't solve test quality problems (mocks drifting from reality, flaky tests). RealmForge ensures tests *exist* and *run*, but can't force tests to be *good*.

---

### 2.6 Documentation Rot: "Docs Are Wrong"

**Pain Point:**
* Documentation written after the fact, if at all
* Docs in separate repos, disconnected from code
* Onboarding takes weeks because docs are outdated

**RealmForge Solution:**
* **Living Map** - Always-current visual representation of system
* **Work Paths** - Documentation is part of work path structure (purpose, contracts, decisions)
* **Catalog** - Resource metadata includes purpose, usage, owner
* **Generated Docs** - System map, capability documentation in every runtime bundle
* **Snapshot History** - Documentation state tracked in every snapshot

**Implementing Areas:**
* `frontend` - Living Map visualization
* `catalog` - Resource metadata, purpose documentation
* `control-plane` - Documentation requirements in work paths

**Coverage Level:** 🟡 **Partially Addressed**

**Gap:** Provides structural documentation (what exists, how connected) but not behavioral documentation (how to use, edge cases, troubleshooting). Living Map shows architecture, doesn't replace API docs or runbooks.

---

### 2.7 Deployment and Rollback: "How Do We Undo This?"

**Pain Point:**
* Rollback is "redeploy old code" (but data migrations? config changes?)
* Database migrations are one-way
* "What was the state at 2pm yesterday?" is unanswerable
* MTTR: 2-6 hours

**RealmForge Solution:**
* **Snapshot-Based Rollback** - Restores full project state: metadata, map, files, locks, decisions, tests, render manifests, agent task state, runtime policy state
* **Rollback Granularity** - Can rollback: full project, capability, work path, file, agent step, runtime bundle
* **Post-Rollback Validation** - Automated: metadata consistency checks, file hash verification, standards rules, affected tests, trace replay
* **Governed Runtime Bundle** - Signed, versioned bundle with manifest, policies, contracts, evidence

**Implementing Areas:**
* `snapshot` - Snapshot creation, version chains, rollback engine, validation framework
* `rfsource` - Content-addressed file storage, integrity verification
* `audit` - Event log for what was changed when
* `control-plane` - Rollback orchestration

**Coverage Level:** ✅ **Fully Addressed**

**Impact:** True semantic rollback. "Restore Authentication to last known-good state" works.

**Note:** Database migration rollback still challenging (RealmForge tracks *what should be rolled back* but database schema changes require careful design)

---

### 2.8 Cost Tracking: "How Much Did This Cost?"

**Pain Point:**
* No one knows cost of a feature until after it's built
* Cloud bills opaque (which service? team? feature?)
* 20-40% of cloud spend is waste

**RealmForge Solution:**
* **Built-In Cost Tracking** - Token spend, storage spend, build spend, runtime spend, rework spend tracked per work path
* **Cost per Task** - Agent work packets track LLM token usage
* **Cost per Flow** - Work paths accumulate cost across all nodes
* **Reuse Savings** - Track when modules/patterns are reused vs. rebuilt
* **Cost Reports** - Included in every snapshot and runtime bundle

**Implementing Areas:**
* `control-plane` - Cost accumulation per work path, agent work packet cost tracking
* `snapshot` - Cost metadata in every snapshot
* `frontend` - Cost visualization, cost reports dashboard

**Coverage Level:** ✅ **Fully Addressed**

**Impact:** "Was this feature worth building?" is answerable with data

**Note:** Tracks *RealmForge* costs (agent tokens, storage, build time). Doesn't automatically track downstream costs (AWS/Azure bills for deployed apps) - that's separate concern.

---

### 2.9 AI Agent Integration: "AI Edited My Code, Now What?"

**Pain Point:**
* AI tools treat code as text files, not governed artifacts
* No built-in permission boundaries for AI
* No audit trail of "AI did this because user asked for X"
* Engineers spend 30-50% of time reviewing/fixing AI code

**RealmForge Solution:**
* **Agent Work Packets** - AI agents receive scoped permissions, explicit allowed/denied files
* **Database-Backed Credentials** - Agent roles with read/write views
* **Audit Trail** - Every agent action logged: who (agent), what (files modified), when, why (work path node), under which approval
* **Intent Tracking** - Every change linked back to user request, work path, capability
* **Blast Radius Awareness** - Agent knows impact before making changes
* **Evidence Requirements** - Agent must submit test evidence, can't mark work complete without proof

**Implementing Areas:**
* `control-plane` - Agent credential management, work packet creation, evidence validation
* `policy` - Agent permission evaluation
* `audit` - Agent action logging
* `catalog` - File ownership, blast radius tracking

**Coverage Level:** ✅ **Fully Addressed**

**Impact:** AI agents become first-class, governed participants in development. Not "AI assistants that help humans edit Git repos", but "AI agents operating in permission-scoped contexts with audit trails."

---

## Section 2: SDLC Phases → RealmForge Coverage

### 3.1 Discovery and Planning Phase

**Traditional Bottlenecks:**
* Vague requirements
* Unclear success criteria
* Missing technical feasibility
* Stakeholder misalignment

**RealmForge Coverage:**
* ✅ **Living Map** extracts requirements from conversation
* ✅ **Planning Module** captures: what, why, which area, routes, files, tests, agents, assumptions, open questions
* ✅ **Planning Gates** enforce completeness before execution
* 🟡 **Success Metrics** - Captured in work paths but no automated tracking

**Implementing Areas:** `frontend`, `control-plane`, `catalog`

---

### 3.2 Design Phase

**Traditional Bottlenecks:**
* Design disconnected from implementation planning
* No mechanism for design approval before build starts
* Designs change mid-implementation
* No feedback loop from engineering to design

**RealmForge Coverage:**
* ✅ **Living Map** shows design in context of implementation
* ✅ **Work Paths** connect design to files, tests, agents
* ❌ **Design Approval Workflow** - **USER-IDENTIFIED GAP**
  * No mechanism to route design for human sign-off before build
  * No workflow for design revision based on feedback
  * Once approved, no automatic trigger to implementation
  * No granular documentation as design evolves

**Implementing Areas:** `frontend`, `control-plane`

**CRITICAL GAP:** Design approval and revision workflow not yet addressed

---

### 3.3 Implementation Planning Phase

**Traditional Bottlenecks:**
* Tasks too large or vague
* Dependencies not identified until mid-sprint
* Test strategy is "we'll test it later"

**RealmForge Coverage:**
* ✅ **Work Paths** break design into structured nodes
* ✅ **Dependency Tracking** identifies dependencies up front
* ✅ **Test-First Planning** - Tests required before implementation
* ✅ **Planning Gates** block execution until planning complete

**Implementing Areas:** `control-plane`, `catalog`

---

### 3.4 Implementation Phase

**Traditional Bottlenecks:**
* Context switching destroys productivity
* Unclear ownership of files
* No blast radius awareness

**RealmForge Coverage:**
* 🟡 **Context Switching** - Work path state helps agents, unclear for human engineers
* ✅ **File Ownership** - Every file has clear owner, allowed modifiers
* ✅ **Blast Radius** - File registry tracks dependencies, usage, impact

**Implementing Areas:** `control-plane`, `catalog`, `frontend`

---

### 3.5 Code Review Phase

**Traditional Bottlenecks:**
* Reviews are slow (24-72 hours)
* Reviewers don't have context
* Architectural issues found late

**RealmForge Coverage:**
* ✅ **Work Path Context** - Reviewers see: why this change, what it affects, what was approved
* ✅ **File Registry** - Shows ownership, risk level, who must review
* 🟡 **Review Speed** - RealmForge provides context, doesn't automate review itself

**Implementing Areas:** `catalog`, `frontend`, `audit`

---

### 3.6 Testing/QA Phase

**Traditional Bottlenecks:**
* Tests are flaky
* Test coverage unknown
* QA is bottleneck (single shared team)
* Bugs found late, require rework

**RealmForge Coverage:**
* ✅ **Test Registry** - Required tests tracked per work path node
* ✅ **Evidence-Based Progression** - Can't mark complete without test evidence
* ❌ **Incremental Review Gates** - **USER-IDENTIFIED GAP**
  * No structured review at 30%, 70%, 90% completion
  * No mechanism for AI self-assessment followed by human fine-tuning
  * No sprint schedule for CR reviews
  * Review is binary (done/not done), not staged

**Implementing Areas:** `control-plane`, `catalog`

**CRITICAL GAP:** Incremental review gates not yet implemented

---

### 3.7 Deployment Phase

**Traditional Bottlenecks:**
* Deployment is manual and error-prone
* No clear rollback procedure
* No audit trail of who deployed what when

**RealmForge Coverage:**
* ✅ **Governed Runtime Bundle** - Signed, versioned, validated bundle
* ✅ **Deployment Manifest** - Clear record of what's in this deployment
* ✅ **Audit Trail** - Who deployed, when, under which approval
* ✅ **Rollback Procedure** - Restore to previous snapshot

**Implementing Areas:** `snapshot`, `audit`, `infrastructure`, `control-plane`

---

### 3.8 Monitoring and Maintenance Phase

**Traditional Bottlenecks:**
* Alerts noisy or missing
* Debugging requires tribal knowledge
* Root cause hard to identify

**RealmForge Coverage:**
* ✅ **Planned Trace Points** - Defined in work paths, not added after the fact
* ✅ **Living Map with Runtime Behavior** - Shows which trace points passed/failed
* ✅ **Observability Stack** - Prometheus, Grafana, Loki, Jaeger
* 🟡 **Root Cause Analysis** - RealmForge provides trace data, doesn't do automated RCA

**Implementing Areas:** `infrastructure`, `frontend`, `control-plane`

---

### 3.9 Rollback and Recovery Phase

**Traditional Bottlenecks:**
* Rollback is slow and scary
* Not clear what "rollback" means
* No snapshot of "known good state"
* MTTR: 30 minutes to 6 hours

**RealmForge Coverage:**
* ✅ **Snapshot-Based Rollback** - Restore full project state
* ✅ **Known-Good State** - Every snapshot tagged with validation status
* ✅ **Fast Rollback** - Restore from snapshot, automated validation
* ✅ **Clear Semantics** - Rollback = restore this snapshot

**Implementing Areas:** `snapshot`, `rfsource`, `control-plane`

---

### 3.10 Retrospective and Improvement Phase

**Traditional Bottlenecks:**
* Action items not tracked or completed
* Same problems discussed repeatedly
* No systemic fixes

**RealmForge Coverage:**
* 🟡 **Cost Reports** - Show where time/money was wasted
* 🟡 **Snapshot History** - Can compare "before" and "after" states
* ❌ **Action Item Tracking** - Not explicitly addressed
* ❌ **Retrospective Workflow** - Not a core capability

**Implementing Areas:** `frontend` (for reports), `snapshot` (for history)

**Gap:** RealmForge provides data for retrospectives but doesn't manage retrospective process itself

---

## Section 3: Missing Capabilities (from Landscape Doc) → RealmForge Coverage

### 4.1 Intent and Context Management

**Pain Point:** Git knows "this line changed" but not "why" or "what's affected"

**RealmForge Solution:**
* ✅ Living Map connects user request → work path → files → tests → deployment
* ✅ Audit log records: who, what, when, why, under which approval
* ✅ Work path nodes include: purpose, owner capability, affected files

**Coverage:** ✅ **Fully Addressed**

---

### 4.2 Scoped AI Permissions

**Pain Point:** AI can modify any file it sees, no permission model

**RealmForge Solution:**
* ✅ Agent work packets with explicit allowed/denied files
* ✅ Database-backed agent credentials and views
* ✅ Policy engine evaluates permissions before every action

**Coverage:** ✅ **Fully Addressed**

---

### 4.3 Governed Project State

**Pain Point:** Code is truth, requirements/decisions/ownership are separate

**RealmForge Solution:**
* ✅ Governed project state is source of truth
* ✅ Code is one rendering of state
* ✅ All metadata in queryable stores (catalog, audit, snapshot)

**Coverage:** ✅ **Fully Addressed**

---

### 4.4 True Rollback

**Pain Point:** Rollback = redeploy code, doesn't restore decisions/permissions/config

**RealmForge Solution:**
* ✅ Snapshot-based rollback restores full project state
* ✅ Includes: metadata, files, locks, decisions, tests, policies
* ✅ Post-rollback automated validation

**Coverage:** ✅ **Fully Addressed**

---

### 4.5 Built-In Cost Tracking

**Pain Point:** No connection between work and cost

**RealmForge Solution:**
* ✅ Cost tracked per work path: token, storage, build, runtime, rework spend
* ✅ Cost reports in snapshots and bundles
* ✅ Reuse savings tracked

**Coverage:** ✅ **Fully Addressed**

---

### 4.6 Runtime-Governed Behavior

**Pain Point:** All behavior is compiled code, simple changes require full deployment

**RealmForge Solution:**
* ✅ Runtime behavior governed by signed Parquet tables
* ✅ Parquet-only updates (no recompilation): permissions, feature flags, trace verbosity, policies, navigation, rate limits
* ✅ Runtime engine enforces governance from tables

**Coverage:** ✅ **Fully Addressed**

---

### 4.7 Design Approval and Revision Workflow

**Pain Point:** Design happens in isolation, engineering discovers problems too late

**RealmForge Solution:**
* ❌ **Not yet addressed**
* **USER-IDENTIFIED GAP:**
  * No mechanism to route design for human sign-off
  * No workflow for design revision based on feedback
  * Once approved, no automatic trigger to implementation
  * No granular docs as design evolves

**Coverage:** ❌ **Not Addressed - CRITICAL GAP**

**Required Areas:** Likely `frontend` (approval UI) + `control-plane` (approval workflow orchestration)

---

### 4.8 Incremental Review Gates

**Pain Point:** Review is binary (done/not done), QA sees work only when "done"

**RealmForge Solution:**
* 🟡 **Partially addressed**
* ✅ Planning gates exist (before execution starts)
* ❌ **USER-IDENTIFIED GAP:**
  * No review at 30%, 70%, 90% completion during execution
  * No AI self-assessment checkpoint
  * No sprint schedule for CR reviews
  * No staged review workflow

**Coverage:** 🟡 **Partially Addressed - CRITICAL GAP**

**Required Areas:** `control-plane` (incremental gate enforcement) + `frontend` (progress tracking, review UI)

---

## Section 4: Coverage Summary Matrix

| Pain Point | RealmForge Coverage | Implementing Areas | Notes |
|-----------|-------------------|-------------------|-------|
| **Requirements & Planning** | ✅ Fully | frontend, control-plane, catalog | Living Map + Planning Module |
| **Context Switching** | 🟡 Partial | control-plane, frontend, snapshot | Good for agents, unclear for humans |
| **Blast Radius** | ✅ Fully | catalog, control-plane, frontend | File registry + dependency tracking |
| **Permissions & Security** | ✅ Fully | policy, audit, control-plane, catalog | Database-backed agent credentials |
| **Testing Gaps** | 🟡 Partial | control-plane, catalog, snapshot | Ensures tests exist/run, not quality |
| **Documentation Rot** | 🟡 Partial | frontend, catalog, control-plane | Structural docs, not behavioral |
| **Deployment & Rollback** | ✅ Fully | snapshot, rfsource, audit, control-plane | True semantic rollback |
| **Cost Tracking** | ✅ Fully | control-plane, snapshot, frontend | Per work path, agent, feature |
| **AI Agent Integration** | ✅ Fully | control-plane, policy, audit, catalog | Scoped permissions + audit trail |
| **Design Approval Workflow** | ❌ Gap | **MISSING** | USER-IDENTIFIED CRITICAL GAP |
| **Incremental Review Gates** | 🟡 Partial | control-plane, frontend | Planning gates exist, execution gates missing |
| **Intent & Context Management** | ✅ Fully | catalog, audit, control-plane | Every change linked to why |
| **Scoped AI Permissions** | ✅ Fully | policy, control-plane | Database-backed scoped credentials |
| **Governed Project State** | ✅ Fully | ALL | State is source of truth |
| **True Rollback** | ✅ Fully | snapshot, rfsource | Full state restoration |
| **Runtime Governance** | ✅ Fully | control-plane, snapshot | Parquet-backed policies/routes |

---

## Section 5: Innovation Areas (RealmForge Capabilities Beyond Known Pain Points)

### 5.1 Living Map with Multiple Modes
**What it is:** Adaptive visual system representation (Web App, API, Workflow, Knowledge, Platform modes)

**Pain point it addresses:** Partially addresses documentation rot, but goes beyond - this is a *new way to understand systems*

**Innovation:** Most tools show one view (code, or architecture diagram, or API docs). RealmForge adapts view to project type and consumer (human vs. agent vs. security vs. finance).

---

### 5.2 Agent Work Packets
**What it is:** Scoped, permissioned, evidence-required work units for AI agents

**Pain point it addresses:** Scoped AI permissions + blast radius awareness

**Innovation:** Not "AI assistant that helps you code" - this is "AI agent as first-class development participant with its own credential, scope, and audit trail"

---

### 5.3 Governed Runtime Bundle
**What it is:** Signed, versioned bundle with compiled code + Parquet governance tables + evidence + docs

**Pain point it addresses:** Runtime governance + deployment

**Innovation:** Deployment artifact is not just "the code" - it's a *governed execution package* with policies, contracts, evidence, and rollback manifest included

---

### 5.4 Snapshot-Based Development
**What it is:** Every state transition creates immutable snapshot with metadata + files + decisions + tests + cost

**Pain point it addresses:** True rollback + cost tracking

**Innovation:** Development history is not just Git commits - it's *queryable project state over time* with first-class cost, test evidence, and policy metadata

---

### 5.5 Database-Backed Development Governance
**What it is:** PostgreSQL as the source of truth for project state, not Git

**Pain point it addresses:** Governed project state

**Innovation:** Most tools put metadata in wikis/spreadsheets/tribal knowledge. RealmForge makes metadata *first-class, queryable, versioned data*

---

### 5.6 Cost as First-Class Metric
**What it is:** Every work path, agent action, and feature tracks: token spend, storage spend, build spend, runtime spend, rework spend

**Pain point it addresses:** Cost tracking

**Innovation:** FinOps built in from the start, not bolted on after cloud bills surprise you

---

## Section 6: Critical Gaps Analysis

### Gap 1: Design Approval and Revision Workflow (USER-IDENTIFIED)

**What's Missing:**
1. **Human Signoff Routing**
   * No mechanism to route design spec to stakeholders for approval
   * No approval queue/dashboard
   * No notification when design needs review

2. **Design Revision Workflow**
   * No workflow for: stakeholder reviews → provides feedback → design revised → re-routed for approval
   * No tracking of design versions and which version was approved
   * No diff view between design revisions

3. **Business Language Documentation**
   * Design specs not written in business-friendly language
   * No mechanism to translate technical design to stakeholder-readable format

4. **Approval-to-Implementation Trigger**
   * Once design approved, no automatic creation of work path
   * No automatic assignment of agents/engineers
   * Manual handoff from "approved design" to "start implementation"

5. **Granular Design Documentation Evolution**
   * No tracking of design decisions and rationale as design evolves
   * No documentation of "we tried X, didn't work, went with Y because Z"

**Impact If Not Addressed:**
* Designs approved that can't be built (discovered during implementation)
* Stakeholder expectations mismatch with delivered product
* Rework when design flaws found late
* "Shadow design changes" not captured or approved

**Recommended Solution:**
* **New capability:** Design Approval Workflow
* **Implementing areas:**
  * `frontend` - Approval dashboard UI, design revision diff view, stakeholder notification
  * `control-plane` - Approval state machine, routing logic, auto-trigger to work path creation
  * `catalog` - Design version tracking, approval history
  * `audit` - Who approved what when

---

### Gap 2: Incremental Review Gates (USER-IDENTIFIED)

**What's Missing:**
1. **Staged Completion Checkpoints**
   * No review at 30% complete ("is direction correct?")
   * No review at 70% complete ("is implementation on track?")
   * No review at 90% complete ("ready for final QA?")

2. **AI Self-Assessment**
   * No mechanism for AI to evaluate its own work mid-stream
   * No "confidence score" or "completion quality" self-report
   * Human review is manual, not triggered by AI self-assessment

3. **Sprint CR Review Schedule**
   * No scheduled change request review cadence
   * No enforcement of "review every X days" or "review at sprint boundaries"
   * Ad-hoc review scheduling

4. **Staged Review Workflow**
   * Current: work is not-started → in-progress → complete (binary)
   * Missing: not-started → in-progress-30% → review-30% → in-progress-70% → review-70% → in-progress-100% → final-review → complete

**Impact If Not Addressed:**
* Entire sprints of work discarded because direction was wrong (caught too late)
* AI agents produce large bodies of code that are fundamentally wrong (no mid-course correction)
* Rework costs 3-5x more than early feedback would have
* Human reviewers overwhelmed with "done" work that's not actually done

**Recommended Solution:**
* **New capability:** Incremental Review Gates
* **Implementing areas:**
  * `control-plane` - Staged review gate enforcement, AI self-assessment triggers, sprint review scheduling
  * `frontend` - Progress visualization (30%, 70%, 90%), review queue by stage, self-assessment display
  * `catalog` - Partial completion state tracking

---

### Gap 3: Human Engineer Day-to-Day Workflow

**What's Missing:**
RealmForge is well-designed for *AI agents* operating in governed contexts. Less clear how *human engineers* benefit in their day-to-day IDE work.

**Questions:**
* Does a human engineer use RealmForge UI to see their work path, then edit code in VSCode?
* Is there VSCode/IntelliJ integration that shows file blast radius, ownership, required tests?
* How does a human engineer say "I'm working on this work path node now" and get the scoped context?
* Does the human engineer commit to Git, and RealmForge syncs? Or commit to RealmForge directly?

**Impact If Not Addressed:**
* RealmForge becomes "the AI system" not "the development platform"
* Human engineers bypass RealmForge and work in Git, losing governance benefits

**Recommended Solution:**
* Define IDE integration strategy
* Define Git interop model (is Git still used? as shadow? as source?)
* Define human engineer workflow: how they claim work, get context, submit evidence

---

### Gap 4: Test Quality vs. Test Existence

**What's Addressed:**
* ✅ RealmForge ensures tests *exist* (test registry, required tests)
* ✅ RealmForge ensures tests *run* (evidence-based progression)

**What's Not Addressed:**
* ❌ RealmForge doesn't ensure tests are *good*
  * Mocks still drift from reality
  * Tests can pass but not cover critical paths
  * Flaky tests still flake

**Impact:**
* False confidence: "all required tests passing" doesn't mean "production will work"

**Possible Solution:**
* Contract testing integration (ensure mocks match real service contracts)
* Mutation testing (ensure tests actually catch bugs, not just pass)
* Flaky test detection (flag tests with inconsistent results)
* Test quality metrics (coverage, edge case coverage, failure injection)

---

## Section 7: Prioritization Recommendations

### P0 - Critical Gaps (Block Production Use)
1. **Design Approval and Revision Workflow** (USER-IDENTIFIED)
   * Without this: designs approved that can't be built, stakeholder misalignment
   * Blocks: Any project with non-technical stakeholders

2. **Incremental Review Gates** (USER-IDENTIFIED)
   * Without this: entire sprints discarded, late discovery of fundamental issues
   * Blocks: Any project with complex, multi-week features

3. **RFSource Storage Model Correction**
   * Current AREA_CONTEXT.md incorrectly describes it as "Parquet-backed"
   * Blocks: Correct architectural understanding

### P1 - High Value (Enhance Core Capabilities)
4. **Human Engineer Workflow Integration**
   * Without this: RealmForge is "AI-only", human engineers bypass it
   * Needed for: Developer adoption

5. **Test Quality Assurance**
   * Without this: false confidence from passing tests
   * Needed for: Production reliability

### P2 - Nice to Have (Incremental Improvements)
6. **Retrospective Workflow Management**
   * RealmForge provides data, but doesn't manage retrospective process
   * Workaround: Use existing retro tools, import RealmForge reports

7. **Automated Root Cause Analysis**
   * RealmForge provides trace data, but doesn't do automated RCA
   * Workaround: Human analysis of trace data

---

## Section 8: Next Steps

1. **Address P0 Gaps:**
   * Design approval workflow - define requirements, identify implementing areas
   * Incremental review gates - define checkpoint criteria, self-assessment model
   * Correct rfsource AREA_CONTEXT.md - read architectural docs, clarify storage model

2. **Define Human Engineer Workflow:**
   * How do human engineers interact with RealmForge?
   * IDE integration strategy?
   * Git interop model?

3. **Create Remediation Plans:**
   * For each P0 gap, create detailed remediation plan
   * Similar to the 210-question audit framework for existing areas
   * Define success criteria, implementing areas, validation approach

4. **Update Area Contexts:**
   * Add design approval capabilities to relevant areas
   * Add incremental review gates to relevant areas
   * Correct rfsource description

---

## Document Status

**COMPLETE** — Capability mapping analysis finished

**Key Findings:**
* **7 pain points fully addressed** (requirements, blast radius, permissions, rollback, cost, AI integration, intent)
* **5 pain points partially addressed** (context switching, testing, docs, code review, monitoring)
* **2 critical gaps identified** (design approval workflow, incremental review gates)
* **6 innovation areas** beyond known pain points (living map, agent work packets, governed bundles, snapshot dev, DB-backed governance, cost-first)

**Recommended Action:** Prioritize P0 gaps (design approval, incremental review, rfsource correction) before proceeding to Session 2 (210-question audit framework)
