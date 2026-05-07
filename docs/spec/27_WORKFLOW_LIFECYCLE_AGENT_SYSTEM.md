---
doc_id: DOC-SPEC-027
title: "Workflow Lifecycle Agent System"
status: active
owner: orchestrator
reviewers: [cto, pm, backend, domain-architect]
created_at: 2026-05-06
last_reviewed_at: 2026-05-06
source_of_truth: true
product_area: workflow-engine
work_path_ids: [WP-WF-001]
related_decision_ids: [DEC-COUNCIL-WFE-001]
related_file_ids:
  [
    FILE-CRATE-WORKFLOW-ENGINE-CARGO,
    FILE-CRATE-WORKFLOW-ENGINE-LIB,
    FILE-CRATE-WORKFLOW-ENGINE-ERROR,
    FILE-CRATE-WORKFLOW-ENGINE-TRAITS,
    FILE-CRATE-WORKFLOW-ENGINE-ORCHESTRATOR,
    FILE-CRATE-WORKFLOW-ENGINE-STAGES-MOD,
    FILE-CRATE-WORKFLOW-ENGINE-STAGE-IDEA,
    FILE-CRATE-WORKFLOW-ENGINE-STAGE-PLANNING,
    FILE-CRATE-WORKFLOW-ENGINE-STAGE-DESIGN,
    FILE-CRATE-WORKFLOW-ENGINE-STAGE-ARCHITECTURE,
    FILE-CRATE-WORKFLOW-ENGINE-STAGE-COUNCIL,
    FILE-CRATE-WORKFLOW-ENGINE-STAGE-DEVELOPMENT,
    FILE-CRATE-WORKFLOW-ENGINE-STAGE-PEER-REVIEW,
    FILE-CRATE-WORKFLOW-ENGINE-STAGE-CODE-REVIEW,
    FILE-CRATE-WORKFLOW-ENGINE-STAGE-TESTING,
    FILE-CRATE-WORKFLOW-ENGINE-STAGE-DOCUMENTATION,
  ]
visual_node_ids: [VN-WORKFLOW-ENGINE]
visual_edge_ids: []
approval_state: accepted
---

# Workflow Lifecycle Agent System

> *"Each lifecycle stage is not a doc — it's an agent. It lives, it validates, it hands off."*

## 1. Purpose

Turn the RealmForge focused workflow lifecycle stages (Idea, Planning, Design, Council, Architecture, Development, Peer Review, Code Review, Testing, Documentation) into **actual runnable modules** in the codebase. Each lifecycle stage becomes its own agent that:

- Owns its stage-specific validation logic
- Enforces its exit gate before allowing handoff
- Produces structured artifacts that downstream stages can consume
- Is runnable independently (unit-testable stage execution)
- Is chainable via a workflow orchestrator

## 2. Architecture

```
workflow-engine crate
├── StageAgent trait          — common interface for all stage agents
├── Context / Artifact types  — shared data types for stage execution
├── Orchestrator              — sequences stages, enforces gates, tracks state
├── stages/
│   ├── idea.rs               — IdeaStageAgent
│   ├── planning.rs           — PlanningStageAgent
│   ├── design.rs             — DesignStageAgent (extended flow)
│   ├── council.rs            — CouncilStageAgent (extended flow)
│   ├── architecture.rs       — ArchitectureStageAgent
│   ├── development.rs        — DevelopmentStageAgent
│   ├── peer_review.rs        — PeerReviewStageAgent
│   ├── code_review.rs        — CodeReviewStageAgent
│   ├── testing.rs            — TestingStageAgent
│   └── documentation.rs      — DocumentationStageAgent
└── error.rs                  — typed error types

authority-domain/src/workflow.rs  — domain types: LifecycleStage, StageStatus, StageArtifact, GateResult
```

### Stage Trait

```rust
#[async_trait]
pub trait StageAgent: Send + Sync {
    /// The lifecycle stage this agent represents.
    fn stage(&self) -> LifecycleStage;

    /// Run the stage's validation logic against the current context.
    /// Returns a StageResult indicating pass/fail/blocked with findings.
    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult;

    /// Check whether the stage's exit gate is satisfied.
    /// Called after execute() to determine if the handoff can proceed.
    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult;

    /// The artifact this stage is responsible for producing.
    fn required_artifact(&self) -> ArtifactDescriptor;

    /// Human-readable stage status for reporting.
    fn status_summary(&self, ctx: &WorkflowContext) -> String;
}
```

## 3. Stage Definitions

### Core Lifecycle (from 07-focused-workflow-lifecycle.md)

| Stage | Owner | Required Artifact | Exit Gate |
|-------|-------|-------------------|-----------|
| Idea | `biz-user`, `pm` | Problem statement, user outcome, constraints | Request is clear enough to plan |
| Planning | `pm`, `orchestrator` | Work slice, scope boundary, acceptance criteria | Next implementable slice is named |
| Architecture | `cto` | Boundary contract, affected files, risks | Layer law satisfied |
| Development | `backend`, `frontend`, `data-engineer` | Focused implementation diff | Docs-first law followed |
| Peer Review | `peer-review` | Area readiness evidence | Work coherent enough for code review |
| Code Review | `code-review` | File review records | No blocking findings |
| Testing | `qa` | Test results, acceptance evidence | Required gates run |
| Documentation | `tech-writer`, `pm`, `qa` | Updated specs, roadmaps, verification | Docs match implementation |

### Extended Stages (user-requested)

| Stage | Owner | Required Artifact | Exit Gate |
|-------|-------|-------------------|-----------|
| Design | `domain-architect` | Design document, bounded context, state model | Design is reviewed and approved |
| Council | `council` | Decision record, positions, resolution | Decision is closed or deferred |

## 4. Workflow Orchestration

The `Orchestrator` coordinates stage sequencing:

```
Idea → Planning → [Design] → Architecture → [Council] → Development → Peer Review → Code Review → Testing → Documentation
```

**Flow rules:**
- Core stages (Idea → Planning → Architecture → Development → Peer Review → Code Review → Testing → Documentation) are always present.
- Extended stages (Design, Council) are optional and configured per workflow.
- A stage cannot start until the previous stage's exit gate is satisfied.
- If `execute()` returns `StageResult::Failed`, the workflow pauses and records the findings.
- If `execute()` returns `StageResult::Blocked(deferred_decision)`, the decision is recorded in `docs/spec/22_OPEN_DECISIONS.md`.

## 5. Stage Execution Model

Each stage agent follows this contract:

1. **Pre-fly check**: Validate that all preconditions are met (previous stage completed, required inputs exist).
2. **Execute**: Run the stage's validation logic against the workflow context.
3. **Artifact production**: The stage produces or validates its required artifact.
4. **Exit gate check**: Validate that the stage's exit criteria are satisfied.
5. **Handoff**: Mark the stage as complete and update workflow state for the next stage.

## 6. Artifact Types

```rust
pub enum ArtifactKind {
    ProblemStatement,
    WorkSlice,
    ScopeBoundary,
    AcceptanceCriteria,
    DesignDocument,
    DecisionRecord,
    ArchitectureContract,
    ImplementationDiff,
    ReviewEvidence,
    FileReviewRecord,
    TestResults,
    DocumentationUpdate,
}

pub struct StageArtifact {
    pub kind: ArtifactKind,
    pub description: String,
    pub file_paths: Vec<String>,      // paths to artifact files
    pub validated_at: DateTime<Utc>,
    pub validator: String,            // skill name that validated
}
```

## 7. Error Model

```
WorkflowError
├── StageExecutionFailed { stage, reason }
├── ExitGateUnsatisfied { stage, gate_checks: Vec<GateCheck> }
├── BlockedOnDecision { decision_id, stage }
├── StageOrderViolation { attempted, expected }
├── MissingArtifact { stage, expected_artifact }
├── HandoffRejected { from, to, reason }
└── Internal { message }
```

## 8. Integration

- The `workflow-engine` crate depends on `authority-domain` (for domain types).
- Orchestrator can be called from `control-api`, `agent-mcp`, and `operator-cli`.
- Stage agents are registered at build time via a static registry.
- Each stage agent maps to the skill that owns it (e.g., `IdeaStageAgent` maps to `biz-user` and `pm` skills).

## 9. Security & Governance

- Each stage agent enforces that only authorized skills can advance their stage.
- The orchestrator enforces stage ordering — no skipping without an explicit bypass decision.
- Bypassing a stage requires a council decision recorded in the workflow context.
- Every handoff between stages is logged as an audit event.
