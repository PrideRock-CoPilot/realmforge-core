//! # Peer Review Stage Agent
//!
//! Owned by: `peer-review`
//! Required artifact: Area readiness evidence and next-gate recommendation
//! Exit gate: Work is coherent enough to enter file-level code review

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, ArtifactKind, FindingSeverity, GateCheck, GateResult, LifecycleStage,
    StageArtifact, StageFinding, StageResult, WorkflowContext,
};

use crate::traits::StageAgent;

/// The Peer Review stage agent — validates area readiness before
/// file-level code review. Checks coherence and readiness evidence.
#[derive(Clone, Debug)]
pub struct PeerReviewStageAgent;

impl PeerReviewStageAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PeerReviewStageAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageAgent for PeerReviewStageAgent {
    fn stage(&self) -> LifecycleStage {
        LifecycleStage::PeerReview
    }

    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
        let mut findings = Vec::new();

        // Check 1: Area readiness documented
        if ctx
            .metadata
            .get("area_readiness")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "PEER-NO-READINESS",
                "No area readiness evidence — has the work been reviewed for coherence?",
                FindingSeverity::Blocker,
            ));
        }

        // Check 2: Next-gate recommendation exists (code review or rework)
        if !ctx.metadata.contains_key("next_gate_recommendation") {
            findings.push(StageFinding::new(
                "PEER-NO-RECOMMENDATION",
                "No next-gate recommendation — is this ready for code review?",
                FindingSeverity::Warning,
            ));
        }

        if findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Blocker)
        {
            StageResult::failed(LifecycleStage::PeerReview, findings)
        } else {
            StageResult::passed(LifecycleStage::PeerReview)
        }
    }

    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult {
        let mut checks = Vec::new();

        let has_readiness = ctx
            .metadata
            .get("area_readiness")
            .is_some_and(|s| !s.is_empty());
        if has_readiness {
            checks.push(GateCheck::passed("Area readiness evidence provided"));
        } else {
            checks.push(GateCheck::failed(
                "Area readiness evidence provided",
                "No readiness evidence — area coherence has not been verified",
            ));
        }

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            GateResult::satisfied("Work is coherent enough to enter file-level code review")
        } else {
            GateResult::unsatisfied(checks, "Peer review has unresolved gaps")
        }
    }

    fn required_artifact(&self) -> ArtifactDescriptor {
        ArtifactDescriptor {
            kind: ArtifactKind::ReviewEvidence,
            description: "Area readiness evidence and next-gate recommendation",
        }
    }

    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
        let artifact = StageArtifact::new(
            ArtifactKind::ReviewEvidence,
            "Peer Review stage — area readiness evidence",
            "peer-review",
        );
        ctx.add_artifact(artifact.clone());
        artifact
    }

    fn status_summary(&self, ctx: &WorkflowContext) -> String {
        match ctx.stage_status(&LifecycleStage::PeerReview) {
            authority_domain::StageStatus::Pending => {
                "⏳ Peer Review stage not started".to_string()
            }
            authority_domain::StageStatus::InProgress => {
                "🔄 Reviewing area readiness...".to_string()
            }
            authority_domain::StageStatus::Completed => {
                "✅ Peer Review complete — ready for code review".to_string()
            }
            authority_domain::StageStatus::Failed => {
                "❌ Peer Review failed — area coherence issues found".to_string()
            }
            authority_domain::StageStatus::Blocked => {
                "🔒 Peer Review blocked on author clarification".to_string()
            }
            authority_domain::StageStatus::Bypassed => "⏭️ Peer Review stage bypassed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn peer_review_passes_with_readiness() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata.insert("area_readiness".into(), "ready".into());
        ctx.metadata.insert(
            "next_gate_recommendation".into(),
            "proceed to code review".into(),
        );

        let agent = PeerReviewStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(result.passed, "Peer review should pass with readiness");
    }

    #[tokio::test]
    async fn peer_review_fails_without_readiness() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = PeerReviewStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(!result.passed, "Peer review should fail without readiness");
    }

    #[test]
    fn peer_review_exit_gate_checks() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = PeerReviewStageAgent::new();
        let gate = agent.check_exit_gate(&ctx);
        assert!(!gate.satisfied, "Gate should fail without readiness");

        ctx.metadata.insert("area_readiness".into(), "ready".into());
        let gate = agent.check_exit_gate(&ctx);
        assert!(gate.satisfied, "Gate should pass with readiness");
    }
}
