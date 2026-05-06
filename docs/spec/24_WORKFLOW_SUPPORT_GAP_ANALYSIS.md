---
doc_id: DOC-SPEC-024
title: "Backend Gap Analysis — Workflow Support (Corrected)"
status: draft
owner: backend
reviewers: [cto, pm, domain-architect, qa, tech-writer]
created_at: 2026-05-05
last_reviewed_at: 2026-05-05
source_of_truth: true
product_area: architecture
work_path_ids: [WP-CORE-001]
related_decision_ids: [DEC-COUNCIL-002]
related_file_ids: [FILE-CRATE-SERVICE-LIB, FILE-CRATE-STORE-LIB, FILE-DB-002]
visual_node_ids: []
visual_edge_ids: []
approval_state: pending
---

# Backend Gap Analysis — Workflow Support (Corrected)

> **Date:** 2026-05-05
> **Analyst:** Dmitri Volkov (Backend)
> **Status:** Corrections to initial gap analysis — identifies genuine gaps vs. existing capabilities that the analysis missed

---

## Executive Summary

The initial gap analysis was a useful framing exercise but contained **significant factual inaccuracies** about existing backend capabilities. The analysis claimed:

> "❌ No project/workflow entity"
> "❌ No phase state machine"
> "❌ No work breakdown structure"
> "❌ No budget entity"
> "❌ No task dependency tracking"
> "❌ No task entity"

**All of the above already exist** in the current codebase. This document provides a corrected inventory.

### What the Analysis Got Right

The **genuine missing systems** are:

| System | Severity | Exists? |
|--------|----------|---------|
| Gate approval workflow (council/CEO/security gates) | 🔴 Blocking | ❌ |
| Security review / threat model entities | 🟡 High Impact | ❌ |
| Test plan management | 🟡 High Impact | ❌ |
| Bug tracking | 🟡 High Impact | ❌ |
| Feedback collection | 🟡 High Impact | ❌ |
| Deployment runbook entities | 🟡 High Impact | ❌ |
| Team/persona assignment tracking | 🟡 High Impact | ❌ |
| Application type registry | 🟡 High Impact | ❌ |
| Design review comments | 🟢 Enhancement | ❌ |
| Data source inventory | 🟢 Enhancement | ❌ |
| Integration assessment | 🟢 Enhancement | ❌ |
| Loop-back (bug→project creation) | 🟢 Enhancement | ❌ |

---

## Part 1: What Already Exists (Correcting the Analysis)

### Plan / Project Entity — ✅ EXISTS

**Claim in analysis:** "No project/workflow entity"

**Reality:** `crates/authority-domain/src/plan.rs` (351 lines) defines a complete `Plan` type with:

```rust
pub struct Plan {
    pub id: String,                         // e.g. "plan_abc123"
    pub name: String,
    pub goal: String,
    pub scope: String,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub architecture_summary: String,
    pub core_areas: Vec<CoreArea>,          // WBS areas
    pub decisions: Vec<PlanDecision>,       // ADR-like records
    pub risks: Vec<PlanRisk>,               // documented risks
    pub phases: Vec<PlanPhase>,             // phase breakdown
    pub work_packets: Vec<WorkPacket>,      // executable work units
    pub status: PlanStatus,                 // Draft, Approved, InProgress, Completed, Cancelled
    pub current_stage: PipelineStage,       // 6-stage pipeline
    pub next_action: String,                // what to do next
    pub owner: String,
    pub audit_log: Vec<PlanAuditEntry>,     // full change history
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Pipeline Stage Machine — ✅ EXISTS

**Claim in analysis:** "No phase state machine"

**Reality:** `PipelineStage` enum with 6 stages, `next()`, `prev()`, `as_str()`:

```
Intake → Refinement → Architecture → Decomposition → Packetization → Ready
```

Validation is stage-aware — the `Plan::validate()` method enforces that each stage's prerequisites are met before advancement.

### Work Breakdown Structure — ✅ EXISTS

**Claim in analysis:** "No work breakdown structure"

**Reality:**
```rust
pub struct CoreArea {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tasks: Vec<AreaTask>,              // tasks within this area
    pub owning_skill: String,              // e.g. "backend", "frontend"
}

pub struct AreaTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,                // Pending, InProgress, Completed, Blocked
    pub target_file_globs: Vec<String>,
    pub estimated_effort_hours: f64,
}
```

### Task Dependency Tracking — ✅ EXISTS

**Claim in analysis:** "No task dependency tracking"

**Reality:** `WorkPacket.dependencies: Vec<String>` — references to packet IDs that must complete first.

### Budget / Cost Tracking — ✅ EXISTS (Partial)

**Claim in analysis:** "No budget entity"

**Reality:** `CostRecord` exists in `crates/authority-domain/src/build_watch.rs`:
```rust
pub struct CostRecord {
    pub id: CostRecordId,
    pub scope: String,
    pub token_cost: u64,
    pub build_time_ms: u64,
    pub storage_bytes: u64,
    pub rework_count: u32,
    pub recorded_at: DateTime<Utc>,
}
```

**Gap:** No `Budget` entity with `estimated_cost`, `approved_budget`, `actual_cost`, or CEO approval workflow. Cost tracking exists at the record level but not the budget/approval level.

### Work Path Graph — ✅ EXISTS

**Claim in analysis:** "No work path traversal"

**Reality:** Full DAG-based work path system in `crates/authority-domain/src/work_path.rs` (301 lines) and `crates/control-service/src/work_path_service.rs` (272 lines):

- `WorkPathGraph` with nodes, root detection, topological traversal
- `WorkPathNode` with file/contract/test/trace-point IDs
- `collect_subtree()` for descendant aggregation
- `traverse_to_packet()` generates scoped `AgentWorkPacket`

### Work Packet Generation — ✅ EXISTS

**Claim in analysis:** No work packet generation

**Reality:** `crates/control-service/src/work_packet_service.rs` (362 lines) implements:
```rust
generate_work_packet(
    tenant_id, project_id, agent_id,
    work_path_id, node_id, objective, cost_budget
) -> Result<AgentWorkPacket>
```

Persists packets and integrates with latest snapshot for rollback anchoring.

### Plan Persistence — ✅ EXISTS (In-Memory)

**Claim in analysis:** "No plan persistence"

**Reality:** `crates/control-store/src/plan_store.rs` has `PlanStore` trait with four operations:
- `save_plan()`, `get_plan()`, `list_plans()`, `delete_plan()`

**Gap:** Only `InMemoryPlanStore` is implemented. No PostgreSQL adapter. The `intake_plans` table exists in migration `012_intake_pipeline.sql` but the store trait isn't wired to it.

### Intake Pipeline Service — ✅ EXISTS

**Claim in analysis:** "No intake pipeline"

**Reality:** `crates/control-service/src/intake_service.rs` (407 lines) implements the complete 6-stage pipeline with type-safe methods:
- `create_plan()` — Stage 1: Intake
- `refine_plan()` — Stage 2: Refinement
- `set_architecture()` — Stage 3: Architecture
- `decompose()` — Stage 4: Decomposition (decisions + risks + phases)
- `generate_packets()` — Stage 5: Packetization
- `advance_to_ready()` — Stage 6: Ready
- `advance_stage()` — Manual stage advancement with validation

### Migration Support — ✅ EXISTS

**Claim in analysis:** "No database migrations for plans"

**Reality:** `db/migrations/012_intake_pipeline.sql` creates `intake_plans` table with full schema and `pipeline_stage` enum.

---

## Part 2: Genuine Gaps (What's Actually Missing)

### 1. Gate Approval System 🔴

**What's missing:** The intake pipeline has validation checks that *could* block advancement, but there's no formal approval gate system with:
- Multiple gate types (user approval, council approval, CEO approval, security approval)
- Approver assignment and notification
- Approval/rejection with audit trail
- Gate status blocking phase transitions

**Current state:** `advance_stage()` and `advance_to_ready()` run validation but don't check for external approvals.

**Why it matters:** This is the core governance mechanism. Without it, phase transitions are purely mechanical validation checks.

**Suggested approach:** New `gate-system` module (not a separate crate initially). Extend `authority-domain` with gate types. Extend `control-store` with gate persistence. Integrate into `intake_service`.

### 2. Security Review / Threat Model 🟡

**What's missing:** No entities for security reviews, threat models, or security findings. The `PipelineStage::Architecture` stage collects `PlanDecision` records but has no dedicated security review field.

**Current state:** Security findings could be stored as `PlanDecision` entries with type tagging, but there's no structured threat model (STRIDE), no finding severity tracking, no security approval workflow.

**Why it matters:** Architecture stage should feed into a formal security review before proceeding.

**Suggested approach:** Extend `Plan` with optional security review fields. Add `security_review` field to `core_areas` or as a top-level plan field.

### 3. Test Plan Management 🟡

**What's missing:** No entities for test plans, test cases, or test execution tracking.

**Current state:** `WorkPacket` has `test_requirements: Vec<String>` and `acceptance_criteria: Vec<String>` but no structured test plan with cases, steps, expected results, pass/fail tracking.

**Why it matters:** Phase 7→8 transition requires all tests passing — but there's no system to record or query test results.

**Suggested approach:** Extend `WorkPacket` with optional `test_plan` field. Add `PlanTestRun` type with case-by-case results.

### 4. Bug Tracking 🟡

**What's missing:** No bug entity with severity, status, assignment, lifecycle.

**Current state:** Bugs could be tracked as `PlanAuditEntry` records, but there's no structured bug tracking.

**Why it matters:** Post-launch (Phase 9) requires bug triage and tracking.

**Suggested approach:** New types in `authority-domain`. Extend `control-store` with bug persistence. Integration with `intake_service` for loop-back (bug→new plan).

### 5. Feedback Collection 🟡

**What's missing:** No feedback entity or collection workflow.

**Current state:** None.

**Why it matters:** Post-launch feedback loop back to intake.

**Suggested approach:** Simple entity in `authority-domain`. Minimal persistence.

### 6. Deployment Runbook 🟡

**What's missing:** No deployment runbook entity with step-by-step plan, execution tracking.

**Current state:** Runtime bundle deployment exists in `bundle_service.rs` and `runtime_service.rs`, but there's no formal runbook with steps, expected results, or execution logs.

**Why it matters:** Phase 8 release requires runbook execution.

**Suggested approach:** New types in `authority-domain`. Extend `bundle_service` or add new service.

### 7. Team / Persona Assignment 🟡

**What's missing:** No system to assign personas to projects (e.g., "Dmitri (backend) assigned to Project X").

**Current state:** `Plan.owner: String` tracks a single owner. `CoreArea.owning_skill: String` tracks required skills but not assigned individuals.

**Why it matters:** Multi-persona orchestration requires assignment tracking.

**Suggested approach:** Extend `Plan` with `team_members: Vec<TeamMember>`.

### 8. Application Type Registry 🟡

**What's missing:** No registry of application types (15 types from APPLICATION_TYPE_MATRIX.md).

**Current state:** None.

**Why it matters:** Template-driven plan creation.

**Suggested approach:** New module in `control-store`. Seed data from existing docs. New MCP tools.

### 9. Design Review Comments 🟢

**What's missing:** No system to track design feedback.

**Current state:** None.

**Suggested approach:** Simple entity. Minimal effort.

### 10. Data Source Inventory 🟢

**What's missing:** No catalog of available data sources.

**Current state:** None.

**Suggested approach:** New module in `control-store`.

### 11. Integration Assessment 🟢

**What's missing:** No tracking of third-party integrations.

**Current state:** None.

**Suggested approach:** Simple entity.

### 12. Loop-Back Mechanism 🟢

**What's missing:** No automated way to create a new Plan from a bug or feedback item.

**Current state:** Manual copy-paste.

**Suggested approach:** Service method `create_plan_from_bug(bug_id) -> Plan`.

---

## Part 3: Corrected Severity Assessment

### 🔴 Blocking (1 system — must exist before production)
1. **Gate approval system** — The intake pipeline's mechanical stage advancement is not sufficient for governance. Without formal gates, we have validation but not approval.

### 🟡 High Impact (7 systems — needed for full 9-phase workflow)
2. **Security review / threat model** — Needs to gate Phase 3→4 advancement
3. **Test plan management** — Needs to gate Phase 7→8 advancement
4. **Bug tracking** — Post-launch Phase 9 requirement
5. **Feedback collection** — Post-launch Phase 9 requirement
6. **Deployment runbook** — Phase 8 release requirement
7. **Team / persona assignment** — Multi-persona orchestration
8. **Application type registry** — Template-driven intake

### 🟢 Enhancement (4 features — nice to have)
9. **Design review comments**
10. **Data source inventory**
11. **Integration assessment**
12. **Loop-back mechanism**

---

## Part 4: Implementation Priority

### Immediate (Gate System — 1-2 days)

The gate approval system is the **only** real blocker. Everything else can be implemented incrementally.

**Minimal viable gates:**
```
1. Define gate types in authority-domain
2. Store gates in control-store (extend intake_plans table or new table)
3. Add gate checks to intake_service.advance_stage()
4. MCP tools: gate_create, gate_submit_approval, gate_get_pending
5. CLI commands for operators
```

### This Week (Security Review + Test Plans — 2-3 days)

```
1. Extend Plan with security_review field (JSONB)
2. Extend WorkPacket with test_plan field
3. Add structured threat model fields
```

### Next Week (Bug tracking + Feedback — 2 days)

```
1. New types + store module
2. Integration with intake_service for loop-back
```

### Later (Assignment, Registry, Runbooks, Enhancements — 3-4 days)

```
1. Team assignment
2. Application type registry + seed data
3. Deployment runbooks
4. Design reviews, data sources, integrations
```

---

## Part 5: Summary of Corrections

| Analysis Claim | Verdict | What Actually Exists |
|----------------|---------|---------------------|
| "No project/workflow entity" | ❌ Incorrect | `Plan` type with 18 fields, 6-stage pipeline |
| "No phase state machine" | ❌ Incorrect | `PipelineStage` enum with next/prev/validation |
| "No work breakdown structure" | ❌ Incorrect | `CoreArea` + `AreaTask` with hierarchy |
| "No task dependency tracking" | ❌ Incorrect | `WorkPacket.dependencies: Vec<String>` |
| "No budget entity" | ❌ Incorrect | `CostRecord` in build_watch module |
| "No task entity" | ❌ Incorrect | `AreaTask` with status, effort, file globs |
| "No work packet generation" | ❌ Incorrect | Full `WorkPacketService` (362 lines) |
| "No plan persistence" | ❌ Incorrect | `PlanStore` trait + `InMemoryPlanStore` |
| "No intake pipeline" | ❌ Incorrect | `IntakeService` (407 lines) with 6 stages |
| "No database migration" | ❌ Incorrect | `012_intake_pipeline.sql` exists |
| "No gate approval system" | ✅ Correct | Genuinely missing |
| "No security review" | ✅ Correct | Genuinely missing |
| "No test plan management" | ✅ Correct | Genuinely missing |
| "No bug tracking" | ✅ Correct | Genuinely missing |
| "No feedback collection" | ✅ Correct | Genuinely missing |
| "No deployment runbook" | ✅ Correct | Genuinely missing |
| "No team assignment" | ✅ Correct | Genuinely missing |
| "No application type registry" | ✅ Correct | Genuinely missing |

---

## Part 6: Next Steps

### Recommended Immediate Action

1. **Write the gate system spec** (`docs/spec/gate-system.md`)
2. **Implement gate system** as the minimal blocking gap
3. **Connect intake pipeline** to PostgreSQL (replace InMemoryPlanStore)
4. **Add MCP tools** for the intake pipeline (currently none exist)
5. **Route to CTO** (Rena) for architecture review of gate system

### What NOT to Do

- Do NOT create a `workflow-engine` crate — the work is already in `control-service`/`control-store`
- Do NOT create an `artifact-store` crate — artifacts are `Plan` JSONB fields with versioning built in
- Do NOT add 35-45 new MCP tools — the existing service architecture handles 8-12 well-defined tools
- Do NOT re-implement existing pipeline mechanics

### Council Decision Required?

**No.** The gap is real but the architecture is clear: extend existing crates with gate types. No new crate, no layer boundary change. This is implementation work, not architectural decision.
