//! # Council Stage Agent (Extended)
//!
//! Owned by: `council`
//! Required artifact: Decision record, positions, resolution
//! Exit gate: Decision is closed or deferred with documented rationale

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, ArtifactKind, FindingSeverity, GateCheck, GateResult, LifecycleStage,
    StageArtifact, StageFinding, StageResult, WorkflowContext,
};

use crate::traits::StageAgent;

/// The Council stage agent (extended) — validates that contested decisions
/// are resolved with documented positions, rationale, and a clear outcome.
#[derive(Clone, Debug)]
pub struct CouncilStageAgent;

impl CouncilStageAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CouncilStageAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageAgent for CouncilStageAgent {
    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Council
    }

    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
        let mut findings = Vec::new();

        // Check 1: Decision record exists
        if ctx
            .metadata
            .get("decision_record")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "COUNCIL-NO-DECISION",
                "No decision record — what decision was made?",
                FindingSeverity::Blocker,
            ));
        }

        // Check 2: Positions documented
        if !ctx.metadata.contains_key("positions_summary") {
            findings.push(StageFinding::new(
                "COUNCIL-NO-POSITIONS",
                "No positions documented — what were the arguments?",
                FindingSeverity::Warning,
            ));
        }

        // Check 3: Resolution documented
        if ctx
            .metadata
            .get("resolution")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "COUNCIL-NO-RESOLUTION",
                "No resolution documented — what was the outcome?",
                FindingSeverity::Blocker,
            ));
        }

        if findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Blocker)
        {
            StageResult::failed(LifecycleStage::Council, findings)
        } else {
            StageResult::passed(LifecycleStage::Council)
        }
    }

    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult {
        let mut checks = Vec::new();

        let has_decision = ctx
            .metadata
            .get("decision_record")
            .is_some_and(|s| !s.is_empty());
        if has_decision {
            checks.push(GateCheck::passed("Decision record exists"));
        } else {
            checks.push(GateCheck::failed(
                "Decision record exists",
                "No decision record — decision not documented",
            ));
        }

        let has_resolution = ctx
            .metadata
            .get("resolution")
            .is_some_and(|s| !s.is_empty());
        if has_resolution {
            checks.push(GateCheck::passed("Resolution is documented"));
        } else {
            checks.push(GateCheck::failed(
                "Resolution is documented",
                "No resolution — outcome is unclear",
            ));
        }

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            GateResult::satisfied("Council decision is closed with documented rationale")
        } else {
            GateResult::unsatisfied(checks, "Council stage has unresolved outcomes")
        }
    }

    fn required_artifact(&self) -> ArtifactDescriptor {
        ArtifactDescriptor {
            kind: ArtifactKind::DecisionRecord,
            description: "Decision record, positions, resolution",
        }
    }

    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
        let artifact = StageArtifact::new(
            ArtifactKind::DecisionRecord,
            "Council stage — decision record and resolution",
            "council",
        );
        ctx.add_artifact(artifact.clone());
        artifact
    }

    fn status_summary(&self, ctx: &WorkflowContext) -> String {
        match ctx.stage_status(&LifecycleStage::Council) {
            authority_domain::StageStatus::Pending => "⏳ Council stage not started".to_string(),
            authority_domain::StageStatus::InProgress => {
                "🔄 Convening council for decision...".to_string()
            }
            authority_domain::StageStatus::Completed => {
                "✅ Council complete — decision documented with rationale".to_string()
            }
            authority_domain::StageStatus::Failed => {
                "❌ Council failed — missing decision record or resolution".to_string()
            }
            authority_domain::StageStatus::Blocked => {
                "🔒 Council blocked — awaiting stakeholder input".to_string()
            }
            authority_domain::StageStatus::Bypassed => {
                "⏭️ Council stage bypassed (extended stage)".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn council_passes_with_complete_metadata() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("decision_record".into(), "DEC-001".into());
        ctx.metadata.insert(
            "positions_summary".into(),
            "Team agreed on approach A".into(),
        );
        ctx.metadata
            .insert("resolution".into(), "Approved approach A".into());

        let agent = CouncilStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(result.passed, "Council should pass with complete metadata");
    }

    #[tokio::test]
    async fn council_fails_without_decision() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = CouncilStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(
            !result.passed,
            "Council should fail without decision record"
        );
        assert!(result
            .findings
            .iter()
            .any(|f| f.code == "COUNCIL-NO-DECISION"));
    }

    #[test]
    fn council_exit_gate_checks() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = CouncilStageAgent::new();
        let gate = agent.check_exit_gate(&ctx);
        assert!(!gate.satisfied, "Gate should fail without metadata");

        ctx.metadata
            .insert("decision_record".into(), "DEC-001".into());
        ctx.metadata.insert("resolution".into(), "Approved".into());
        let gate = agent.check_exit_gate(&ctx);
        assert!(
            gate.satisfied,
            "Gate should pass with decision and resolution"
        );
    }
}
