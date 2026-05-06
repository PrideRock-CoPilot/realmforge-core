//! # Workflow Orchestrator
//!
//! The orchestrator sequences lifecycle stage agents through the focused workflow
//! lifecycle. It enforces stage ordering, exit gate checks, and handoff discipline.
//!
//! ## Handoff Discipline
//!
//! For each stage transition:
//! 1. Verify the next stage follows the canonical order
//! 2. Execute the stage agent
//! 3. Check the exit gate
//! 4. Record the result and artifacts
//! 5. Advance to the next stage
//!
//! If any exit gate is unsatisfied, the orchestrator records the block and stops.

use std::collections::HashMap;

use authority_domain::{GateResult, LifecycleStage, StageResult, StageStatus, WorkflowContext};

use crate::error::{Result, WorkflowError};
use crate::stages::{
    architecture::ArchitectureStageAgent, code_review::CodeReviewStageAgent,
    council::CouncilStageAgent, design::DesignStageAgent, development::DevelopmentStageAgent,
    documentation::DocumentationStageAgent, idea::IdeaStageAgent,
    peer_review::PeerReviewStageAgent, planning::PlanningStageAgent, testing::TestingStageAgent,
};
use crate::traits::StageAgent;

/// The orchestrator sequences stage agents through the workflow lifecycle.
///
/// Create one with `Orchestrator::new(ctx)` and then call `run_full_workflow()`
/// to execute all stages, or use stage-by-stage methods for manual control.
#[derive(Debug)]
pub struct Orchestrator {
    /// The workflow context shared across all stages.
    pub context: WorkflowContext,
    /// Registered stage agents, keyed by lifecycle stage.
    agents: HashMap<LifecycleStage, Box<dyn StageAgent>>,
}

impl Orchestrator {
    /// Create a new orchestrator with the given workflow context.
    ///
    /// Automatically registers all 10 stage agents (core + extended).
    /// Extended stages (Design, Council) are enabled/disabled via
    /// `context.include_extended`.
    pub fn new(context: WorkflowContext) -> Self {
        let mut agents: HashMap<LifecycleStage, Box<dyn StageAgent>> = HashMap::new();

        // Register all core stages
        agents.insert(LifecycleStage::Idea, Box::new(IdeaStageAgent::new()));
        agents.insert(
            LifecycleStage::Planning,
            Box::new(PlanningStageAgent::new()),
        );
        agents.insert(
            LifecycleStage::Architecture,
            Box::new(ArchitectureStageAgent::new()),
        );
        agents.insert(
            LifecycleStage::Development,
            Box::new(DevelopmentStageAgent::new()),
        );
        agents.insert(
            LifecycleStage::PeerReview,
            Box::new(PeerReviewStageAgent::new()),
        );
        agents.insert(
            LifecycleStage::CodeReview,
            Box::new(CodeReviewStageAgent::new()),
        );
        agents.insert(LifecycleStage::Testing, Box::new(TestingStageAgent::new()));
        agents.insert(
            LifecycleStage::Documentation,
            Box::new(DocumentationStageAgent::new()),
        );

        // Register extended stages (available even when not in sequence)
        agents.insert(LifecycleStage::Design, Box::new(DesignStageAgent::new()));
        agents.insert(LifecycleStage::Council, Box::new(CouncilStageAgent::new()));

        Self { context, agents }
    }

    /// Register a custom agent for a lifecycle stage.
    ///
    /// This replaces the default agent for the given stage. Useful for
    /// injecting mock agents in tests or custom workflows.
    pub fn register_agent(&mut self, stage: LifecycleStage, agent: Box<dyn StageAgent>) {
        self.agents.insert(stage, agent);
    }

    /// Execute a single stage by its agent.
    ///
    /// 1. Sets the stage status to `InProgress`
    /// 2. Calls `agent.execute()`
    /// 3. Records the result
    /// 4. Sets the stage status to `Completed` or `Failed`
    /// 5. Produces the artifact
    async fn execute_stage(&mut self, stage: &LifecycleStage) -> StageResult {
        let agent = match self.agents.get(stage) {
            Some(a) => a,
            None => {
                return StageResult::failed(
                    stage.clone(),
                    vec![authority_domain::StageFinding::new(
                        "NO-AGENT",
                        format!("No agent registered for stage {}", stage.label()),
                        authority_domain::FindingSeverity::Blocker,
                    )],
                );
            }
        };

        self.context
            .set_stage_status(stage.clone(), StageStatus::InProgress);

        let result = agent.execute(&mut self.context).await;

        if result.passed {
            self.context
                .set_stage_status(stage.clone(), StageStatus::Completed);
            // Produce artifact on success
            let artifact = agent.produce_artifact(&mut self.context);
            let result = StageResult::passed(stage.clone()).with_artifact(artifact);
            self.context.record_result(result.clone());
            result
        } else if result.blocked_on.is_some() {
            self.context
                .set_stage_status(stage.clone(), StageStatus::Blocked);
            self.context.record_result(result.clone());
            result
        } else {
            self.context
                .set_stage_status(stage.clone(), StageStatus::Failed);
            self.context.record_result(result.clone());
            result
        }
    }

    /// Check the exit gate for a specific stage.
    pub fn check_exit_gate(&self, stage: &LifecycleStage) -> GateResult {
        match self.agents.get(stage) {
            Some(agent) => agent.check_exit_gate(&self.context),
            None => GateResult::unsatisfied(
                vec![],
                format!("No agent registered for stage {}", stage.label()),
            ),
        }
    }

    /// Get a status summary for a specific stage.
    pub fn stage_summary(&self, stage: &LifecycleStage) -> String {
        match self.agents.get(stage) {
            Some(agent) => agent.status_summary(&self.context),
            None => format!("Unknown stage: {}", stage.label()),
        }
    }

    /// Run the full workflow from the current stage to completion.
    ///
    /// Returns the final `WorkflowContext` with all results recorded.
    /// If any stage fails or blocks, the orchestrator stops and returns
    /// the context in its current state.
    ///
    /// ## Errors
    ///
    /// Returns an error if:
    /// - A stage's exit gate is unsatisfied after execution
    /// - A stage order violation is detected
    pub async fn run_full_workflow(&mut self) -> Result<&WorkflowContext> {
        let max_stages = if self.context.include_extended { 10 } else { 8 };

        for _ in 0..max_stages {
            let current = self.context.current_stage.clone();

            // Skip extended stages if not included
            if current.is_extended() && !self.context.include_extended {
                self.context
                    .set_stage_status(current.clone(), StageStatus::Bypassed);
                if self.context.advance().is_err() {
                    break;
                }
                continue;
            }

            // Execute the stage
            let result = self.execute_stage(&current).await;

            // Check exit gate if stage passed
            if result.passed {
                let gate = self.check_exit_gate(&current);
                if !gate.satisfied {
                    return Err(WorkflowError::gate_unsatisfied(
                        current.clone(),
                        gate.summary,
                    ));
                }
            } else if result.blocked_on.is_some() {
                // Blocked — return early with context
                return Ok(&self.context);
            } else {
                // Failed — return early with context
                return Ok(&self.context);
            }

            // Advance to next stage
            if self.context.advance().is_err() {
                break;
            }
        }

        Ok(&self.context)
    }

    /// Execute a single stage and check its exit gate.
    ///
    /// Useful for manual, stage-by-stage execution.
    pub async fn execute_and_check(&mut self, stage: &LifecycleStage) -> Result<GateResult> {
        // Verify stage order
        if *stage != self.context.current_stage {
            return Err(WorkflowError::order_violation(
                stage.clone(),
                self.context.current_stage.clone(),
            ));
        }

        let result = self.execute_stage(stage).await;
        if !result.passed {
            return Ok(GateResult::unsatisfied(
                vec![],
                format!("Stage {} execution failed", stage.label()),
            ));
        }

        let gate = self.check_exit_gate(stage);
        if !gate.satisfied {
            return Err(WorkflowError::gate_unsatisfied(
                stage.clone(),
                gate.summary.clone(),
            ));
        }

        Ok(gate)
    }

    /// Advance to the next stage in the workflow.
    pub fn advance(&mut self) -> Result<()> {
        self.context
            .advance()
            .map_err(|e| WorkflowError::Internal(e.to_string()))
    }

    /// Get a summary of the entire workflow status.
    pub fn workflow_summary(&self) -> String {
        use LifecycleStage::*;
        let stages = if self.context.include_extended {
            vec![
                Idea,
                Planning,
                Design,
                Architecture,
                Council,
                Development,
                PeerReview,
                CodeReview,
                Testing,
                Documentation,
            ]
        } else {
            vec![
                Idea,
                Planning,
                Architecture,
                Development,
                PeerReview,
                CodeReview,
                Testing,
                Documentation,
            ]
        };

        let mut lines = Vec::new();
        lines.push(format!("Workflow: {}", self.context.workflow_id));
        lines.push(format!(
            "Current stage: {}",
            self.context.current_stage.label()
        ));
        lines.push(format!(
            "Include extended: {}",
            self.context.include_extended
        ));
        lines.push(String::new());
        lines.push("Stage statuses:".to_string());

        for stage in &stages {
            let status = self.context.stage_status(stage);
            let label = stage.label();
            let marker = if *stage == self.context.current_stage {
                " ← CURRENT"
            } else {
                ""
            };
            lines.push(format!("  {:20} {}{}", label, status, marker));
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn orchestrator_creates_and_runs_full_workflow() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        // Seed metadata so Idea stage passes
        ctx.metadata
            .insert("problem_statement".into(), "Build login".into());
        ctx.metadata
            .insert("user_outcome".into(), "Users log in".into());
        ctx.metadata
            .insert("constraints".into(), "SSO required".into());
        ctx.metadata.insert("actor".into(), "end-user".into());
        ctx.metadata
            .insert("success_signal".into(), "e2e passes".into());

        // Seed metadata for Planning
        ctx.metadata
            .insert("work_slice".into(), "Login API endpoint".into());
        ctx.metadata.insert(
            "acceptance_criteria".into(),
            "POST /login returns token".into(),
        );

        // Seed metadata for Architecture
        ctx.metadata
            .insert("boundary_contract".into(), "control-api".into());
        ctx.metadata.insert("affected_files".into(), "5".into());
        ctx.metadata
            .insert("rollback_path".into(), "git revert".into());
        ctx.metadata.insert("risks_assessed".into(), "true".into());
        ctx.metadata
            .insert("layer_law_verified".into(), "true".into());

        // Seed metadata for Development
        ctx.metadata
            .insert("implementation_diff".into(), "impl login".into());
        ctx.metadata
            .insert("docs_first_verified".into(), "true".into());

        // Seed metadata for Peer Review
        ctx.metadata.insert("area_readiness".into(), "ready".into());

        // Seed metadata for Code Review
        ctx.metadata
            .insert("file_review_record".into(), "reviewed".into());
        ctx.metadata
            .insert("blocking_findings".into(), "none".into());

        // Seed metadata for Testing
        ctx.metadata
            .insert("test_results".into(), "all passing".into());
        ctx.metadata
            .insert("acceptance_evidence".into(), "e2e passes".into());

        // Seed metadata for Documentation
        ctx.metadata.insert("docs_verified".into(), "true".into());

        let mut orchestrator = Orchestrator::new(ctx);
        let result = orchestrator.run_full_workflow().await;

        assert!(
            result.is_ok(),
            "Full workflow should complete: {:?}",
            result.err()
        );

        let ctx = result.unwrap();
        assert!(
            ctx.is_complete(),
            "Workflow should be complete after running all stages"
        );
    }

    #[tokio::test]
    async fn orchestrator_with_extended_stages() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-ext", tid, pid, true);

        // Full metadata for all 10 stages
        ctx.metadata
            .insert("problem_statement".into(), "Build login".into());
        ctx.metadata
            .insert("user_outcome".into(), "Users log in".into());
        ctx.metadata
            .insert("constraints".into(), "SSO required".into());
        ctx.metadata.insert("actor".into(), "end-user".into());
        ctx.metadata
            .insert("success_signal".into(), "e2e passes".into());
        ctx.metadata.insert("work_slice".into(), "Login API".into());
        ctx.metadata.insert(
            "acceptance_criteria".into(),
            "POST /login returns token".into(),
        );
        ctx.metadata
            .insert("design_document".into(), "design.md".into());
        ctx.metadata.insert("bounded_context".into(), "auth".into());
        ctx.metadata
            .insert("state_model".into(), "LoginFlow".into());
        ctx.metadata
            .insert("boundary_contract".into(), "control-api".into());
        ctx.metadata.insert("affected_files".into(), "5".into());
        ctx.metadata
            .insert("rollback_path".into(), "git revert".into());
        ctx.metadata.insert("risks_assessed".into(), "true".into());
        ctx.metadata
            .insert("layer_law_verified".into(), "true".into());
        ctx.metadata
            .insert("decision_record".into(), "DEC-001".into());
        ctx.metadata
            .insert("positions_summary".into(), "agreed".into());
        ctx.metadata.insert("resolution".into(), "approved".into());
        ctx.metadata
            .insert("implementation_diff".into(), "impl".into());
        ctx.metadata
            .insert("docs_first_verified".into(), "true".into());
        ctx.metadata.insert("area_readiness".into(), "ready".into());
        ctx.metadata
            .insert("file_review_record".into(), "reviewed".into());
        ctx.metadata
            .insert("blocking_findings".into(), "none".into());
        ctx.metadata
            .insert("test_results".into(), "all passing".into());
        ctx.metadata
            .insert("acceptance_evidence".into(), "e2e passes".into());
        ctx.metadata.insert("docs_verified".into(), "true".into());

        let mut orchestrator = Orchestrator::new(ctx);
        let result = orchestrator.run_full_workflow().await;

        assert!(
            result.is_ok(),
            "Extended workflow should complete: {:?}",
            result.err()
        );

        let ctx = result.unwrap();
        // is_complete() only checks core stages
        assert!(ctx.is_complete(), "Core stages should all be completed");
    }

    #[tokio::test]
    async fn orchestrator_stops_on_failure() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let ctx = WorkflowContext::new("wf-fail", tid, pid, false);

        let mut orchestrator = Orchestrator::new(ctx);
        let result = orchestrator.run_full_workflow().await;

        // Should fail at Idea stage (no metadata)
        assert!(result.is_ok(), "Should return context even on failure");

        let ctx = result.unwrap();
        assert_eq!(
            ctx.stage_status(&LifecycleStage::Idea),
            StageStatus::Failed,
            "Idea should be failed"
        );
        assert!(!ctx.is_complete(), "Workflow should not be complete");
    }

    #[tokio::test]
    async fn orchestrator_stage_summary() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let ctx = WorkflowContext::new("wf-summary", tid, pid, false);
        let orchestrator = Orchestrator::new(ctx);

        let summary = orchestrator.stage_summary(&LifecycleStage::Idea);
        assert!(summary.contains("Idea stage not started"));
    }

    #[tokio::test]
    async fn orchestrator_workflow_summary() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let ctx = WorkflowContext::new("wf-summary", tid, pid, false);
        let orchestrator = Orchestrator::new(ctx);

        let summary = orchestrator.workflow_summary();
        assert!(summary.contains("wf-summary"));
        assert!(summary.contains("Idea"));
        assert!(summary.contains("pending"));
    }

    #[tokio::test]
    async fn orchestrator_detects_stage_order_violation() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let ctx = WorkflowContext::new("wf-order", tid, pid, false);
        let mut orchestrator = Orchestrator::new(ctx);

        // Try to execute Planning before Idea
        let result = orchestrator
            .execute_and_check(&LifecycleStage::Planning)
            .await;
        assert!(result.is_err(), "Should reject out-of-order execution");
    }

    #[tokio::test]
    async fn orchestrator_register_custom_agent() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let ctx = WorkflowContext::new("wf-custom", tid, pid, false);
        let mut orchestrator = Orchestrator::new(ctx);

        // Register a mock agent that always passes
        let mock_agent = MockStageAgent::new(LifecycleStage::Idea);
        orchestrator.register_agent(LifecycleStage::Idea, Box::new(mock_agent));

        let gate = orchestrator.execute_and_check(&LifecycleStage::Idea).await;
        assert!(gate.is_ok(), "Mock agent should pass: {:?}", gate.err());
    }

    // ---- Mock agent for testing ----
    use async_trait::async_trait;
    use authority_domain::{ArtifactDescriptor, ArtifactKind, StageArtifact};

    #[derive(Debug)]
    struct MockStageAgent {
        stage: LifecycleStage,
    }

    impl MockStageAgent {
        fn new(stage: LifecycleStage) -> Self {
            Self { stage }
        }
    }

    #[async_trait]
    impl StageAgent for MockStageAgent {
        fn stage(&self) -> LifecycleStage {
            self.stage.clone()
        }

        async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
            ctx.metadata.insert("mock_executed".into(), "true".into());
            StageResult::passed(self.stage.clone())
        }

        fn check_exit_gate(&self, _ctx: &WorkflowContext) -> GateResult {
            GateResult::satisfied("mock gate always passes")
        }

        fn required_artifact(&self) -> ArtifactDescriptor {
            ArtifactDescriptor {
                kind: ArtifactKind::ProblemStatement,
                description: "Mock artifact",
            }
        }

        fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
            let artifact =
                StageArtifact::new(ArtifactKind::ProblemStatement, "mock artifact", "mock");
            ctx.add_artifact(artifact.clone());
            artifact
        }

        fn status_summary(&self, _ctx: &WorkflowContext) -> String {
            "✅ Mock stage complete".to_string()
        }
    }
}
