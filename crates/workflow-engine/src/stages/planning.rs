//! # Planning Stage Agent
//!
//! Owned by: `pm`, `orchestrator`
//! Required artifact: Work slice, scope boundary, acceptance criteria, handoff map
//! Exit gate: The next implementable slice is named and blockers are visible

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, ArtifactKind, FindingSeverity, GateCheck, GateResult, LifecycleStage,
    StageArtifact, StageFinding, StageResult, WorkflowContext,
};

use crate::traits::StageAgent;

/// The Planning stage agent — validates that the work request has a clear
/// work slice, scope boundary, acceptance criteria, and visible blockers.
#[derive(Clone, Debug)]
pub struct PlanningStageAgent;

impl PlanningStageAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlanningStageAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageAgent for PlanningStageAgent {
    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Planning
    }

    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
        let mut findings = Vec::new();

        // Check 1: Work slice defined
        if ctx
            .metadata
            .get("work_slice")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "PLAN-NO-SLICE",
                "No named work slice — what is the smallest implementable unit?",
                FindingSeverity::Blocker,
            ));
        }

        // Check 2: Acceptance criteria defined
        if ctx
            .metadata
            .get("acceptance_criteria")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "PLAN-NO-AC",
                "No acceptance criteria — how will we know when this is done?",
                FindingSeverity::Blocker,
            ));
        }

        // Check 3: Scope boundary defined
        if !ctx.metadata.contains_key("scope_boundary") {
            findings.push(StageFinding::new(
                "PLAN-NO-SCOPE",
                "No scope boundary defined — in-scope and out-of-scope are unclear",
                FindingSeverity::Warning,
            ));
        }

        // Check 4: Blockers visible
        let blockers = ctx.metadata.get("blockers");
        if blockers.is_none() {
            findings.push(StageFinding::new(
                "PLAN-NO-BLOCKERS",
                "No blockers documented — may be hiding unresolved dependencies",
                FindingSeverity::Note,
            ));
        }

        if findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Blocker)
        {
            StageResult::failed(LifecycleStage::Planning, findings)
        } else {
            StageResult::passed(LifecycleStage::Planning)
        }
    }

    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult {
        let mut checks = Vec::new();

        let has_slice = ctx
            .metadata
            .get("work_slice")
            .is_some_and(|s| !s.is_empty());
        if has_slice {
            checks.push(GateCheck::passed("Work slice is named"));
        } else {
            checks.push(GateCheck::failed(
                "Work slice is named",
                "No named work slice for this phase",
            ));
        }

        let has_ac = ctx
            .metadata
            .get("acceptance_criteria")
            .is_some_and(|s| !s.is_empty());
        if has_ac {
            checks.push(GateCheck::passed("Acceptance criteria defined"));
        } else {
            checks.push(GateCheck::failed(
                "Acceptance criteria defined",
                "No acceptance criteria — cannot verify completion",
            ));
        }

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            GateResult::satisfied("Planning is complete — implementable slice is named")
        } else {
            GateResult::unsatisfied(checks, "Planning has unresolved gaps")
        }
    }

    fn required_artifact(&self) -> ArtifactDescriptor {
        ArtifactDescriptor {
            kind: ArtifactKind::WorkSlice,
            description: "Work slice, scope boundary, acceptance criteria, handoff map",
        }
    }

    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
        let artifact = StageArtifact::new(
            ArtifactKind::WorkSlice,
            "Planning stage — work slice and acceptance criteria",
            "pm",
        );
        ctx.add_artifact(artifact.clone());
        artifact
    }

    fn status_summary(&self, ctx: &WorkflowContext) -> String {
        match ctx.stage_status(&LifecycleStage::Planning) {
            authority_domain::StageStatus::Pending => "⏳ Planning stage not started".to_string(),
            authority_domain::StageStatus::InProgress => {
                "🔄 Defining work slice and scope...".to_string()
            }
            authority_domain::StageStatus::Completed => {
                "✅ Planning complete — work slice named".to_string()
            }
            authority_domain::StageStatus::Failed => {
                "❌ Planning failed — work slice or acceptance criteria missing".to_string()
            }
            authority_domain::StageStatus::Blocked => {
                "🔒 Planning blocked on unresolved decision".to_string()
            }
            authority_domain::StageStatus::Bypassed => "⏭️ Planning stage bypassed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn planning_passes_with_complete_metadata() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("work_slice".into(), "Login API endpoint".into());
        ctx.metadata.insert(
            "acceptance_criteria".into(),
            "POST /login returns 200".into(),
        );
        ctx.metadata
            .insert("scope_boundary".into(), "auth only".into());

        let agent = PlanningStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(result.passed, "Planning should pass with complete metadata");
    }

    #[tokio::test]
    async fn planning_fails_without_slice() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = PlanningStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(!result.passed, "Planning should fail without work slice");
        assert!(result.findings.iter().any(|f| f.code == "PLAN-NO-SLICE"));
    }

    #[test]
    fn planning_exit_gate_checks() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = PlanningStageAgent::new();
        let gate = agent.check_exit_gate(&ctx);
        assert!(
            !gate.satisfied,
            "Gate should fail without required metadata"
        );

        ctx.metadata.insert("work_slice".into(), "Slice 1".into());
        ctx.metadata
            .insert("acceptance_criteria".into(), "AC 1".into());
        let gate = agent.check_exit_gate(&ctx);
        assert!(gate.satisfied, "Gate should pass with work slice and AC");
    }
}
