//! # Idea Stage Agent
//!
//! Owned by: `biz-user`, `pm`
//! Required artifact: Problem statement, user outcome, constraints, non-goals
//! Exit gate: The request is clear enough to plan or has named open questions

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, ArtifactKind, FindingSeverity, GateCheck, GateResult, LifecycleStage,
    StageArtifact, StageFinding, StageResult, WorkflowContext,
};

use crate::traits::StageAgent;

/// The Idea stage agent — validates that the work request has a clear problem
/// statement, user outcome, known constraints, and documented open questions.
#[derive(Clone, Debug)]
pub struct IdeaStageAgent;

impl IdeaStageAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IdeaStageAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageAgent for IdeaStageAgent {
    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Idea
    }

    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
        let mut findings = Vec::new();

        // Check 1: Problem statement exists
        let has_problem = ctx.metadata.contains_key("problem_statement")
            || ctx.metadata.contains_key("user_outcome");
        if !has_problem {
            findings.push(StageFinding::new(
                "IDEA-NO-PROBLEM",
                "No problem statement or user outcome defined",
                FindingSeverity::Blocker,
            ));
        }

        // Check 2: Constraints documented
        let has_constraints = ctx.metadata.contains_key("constraints");
        if !has_constraints {
            findings.push(StageFinding::new(
                "IDEA-NO-CONSTRAINTS",
                "No constraints documented for this work request",
                FindingSeverity::Warning,
            ));
        }

        // Check 3: Known actor
        let has_actor = ctx.metadata.contains_key("actor");
        if !has_actor {
            findings.push(StageFinding::new(
                "IDEA-NO-ACTOR",
                "No main user or system actor identified",
                FindingSeverity::Warning,
            ));
        }

        // Check 4: Open questions documented
        let has_open_questions = ctx.metadata.contains_key("open_questions");
        if !has_open_questions {
            findings.push(StageFinding::new(
                "IDEA-NO-OPEN-QUESTIONS",
                "No open questions documented — unclear what remains to be clarified",
                FindingSeverity::Note,
            ));
        }

        if findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Blocker)
        {
            StageResult::failed(LifecycleStage::Idea, findings)
        } else {
            StageResult::passed(LifecycleStage::Idea)
        }
    }

    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult {
        let mut checks = Vec::new();

        let has_problem = ctx
            .metadata
            .get("problem_statement")
            .map(|s| !s.is_empty())
            .unwrap_or(false);

        if has_problem {
            checks.push(GateCheck::passed("Problem statement is non-empty"));
        } else {
            checks.push(GateCheck::failed(
                "Problem statement is non-empty",
                "Problem statement is missing or empty",
            ));
        }

        let has_actor = ctx.metadata.contains_key("actor");
        if has_actor {
            checks.push(GateCheck::passed("Main actor identified"));
        } else {
            checks.push(GateCheck::failed(
                "Main actor identified",
                "No actor identified for this work",
            ));
        }

        let success_signal = ctx.metadata.contains_key("success_signal");
        if success_signal {
            checks.push(GateCheck::passed("Success signal defined"));
        } else {
            checks.push(GateCheck::failed(
                "Success signal defined",
                "No success signal — how do we know when this is done?",
            ));
        }

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            GateResult::satisfied("Idea stage is clear enough to proceed to planning")
        } else {
            GateResult::unsatisfied(checks, "Idea stage has unresolved gaps")
        }
    }

    fn required_artifact(&self) -> ArtifactDescriptor {
        ArtifactDescriptor {
            kind: ArtifactKind::ProblemStatement,
            description: "Problem statement, user outcome, constraints, non-goals",
        }
    }

    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
        let artifact = StageArtifact::new(
            ArtifactKind::ProblemStatement,
            "Idea stage — problem statement and user outcome",
            "biz-user",
        );
        ctx.add_artifact(artifact.clone());
        artifact
    }

    fn status_summary(&self, ctx: &WorkflowContext) -> String {
        match ctx.stage_status(&LifecycleStage::Idea) {
            authority_domain::StageStatus::Pending => "⏳ Idea stage not started".to_string(),
            authority_domain::StageStatus::InProgress => {
                "🔄 Clarifying problem statement and user outcomes...".to_string()
            }
            authority_domain::StageStatus::Completed => {
                "✅ Idea stage complete — problem statement defined".to_string()
            }
            authority_domain::StageStatus::Failed => {
                "❌ Idea stage failed — problem statement needs refinement".to_string()
            }
            authority_domain::StageStatus::Blocked => {
                "🔒 Idea stage blocked on open questions".to_string()
            }
            authority_domain::StageStatus::Bypassed => "⏭️ Idea stage bypassed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn idea_stage_passes_with_complete_metadata() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("problem_statement".into(), "Build login module".into());
        ctx.metadata
            .insert("user_outcome".into(), "Users can log in".into());
        ctx.metadata
            .insert("constraints".into(), "Must support SSO".into());
        ctx.metadata.insert("actor".into(), "end-user".into());
        ctx.metadata
            .insert("success_signal".into(), "Login flow passes e2e".into());

        let agent = IdeaStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(result.passed, "Idea should pass with complete metadata");
    }

    #[tokio::test]
    async fn idea_stage_fails_without_problem() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = IdeaStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(!result.passed, "Idea should fail without problem statement");
        assert!(result.findings.iter().any(|f| f.code == "IDEA-NO-PROBLEM"));
    }

    #[test]
    fn idea_exit_gate_checks() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("problem_statement".into(), "Build X".into());

        let agent = IdeaStageAgent::new();
        let gate = agent.check_exit_gate(&ctx);
        assert!(
            !gate.satisfied,
            "Gate should fail without actor and success signal"
        );

        ctx.metadata.insert("actor".into(), "user".into());
        ctx.metadata.insert("success_signal".into(), "done".into());
        let gate = agent.check_exit_gate(&ctx);
        assert!(
            gate.satisfied,
            "Gate should pass with all required metadata"
        );
    }
}
