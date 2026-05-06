//! # Code Review Stage Agent
//!
//! Owned by: `code-review`
//! Required artifact: File review records and area code-review recommendation
//! Exit gate: Every in-scope file has a review record and no blocking finding remains

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, ArtifactKind, FindingSeverity, GateCheck, GateResult, LifecycleStage,
    StageArtifact, StageFinding, StageResult, WorkflowContext,
};

use crate::traits::StageAgent;

/// The Code Review stage agent — validates that every in-scope file has been
/// reviewed and no blocking findings remain before QA.
#[derive(Clone, Debug)]
pub struct CodeReviewStageAgent;

impl CodeReviewStageAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CodeReviewStageAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageAgent for CodeReviewStageAgent {
    fn stage(&self) -> LifecycleStage {
        LifecycleStage::CodeReview
    }

    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
        let mut findings = Vec::new();

        // Check 1: File review records exist
        if ctx
            .metadata
            .get("file_review_record")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "CR-NO-RECORDS",
                "No file review records — which files were reviewed?",
                FindingSeverity::Blocker,
            ));
        }

        // Check 2: No blocking findings remain
        let blocking = ctx
            .metadata
            .get("blocking_findings")
            .map(|v| v != "none")
            .unwrap_or(true);
        if blocking {
            findings.push(StageFinding::new(
                "CR-BLOCKING-FINDINGS",
                "Blocking findings remain — cannot proceed to QA",
                FindingSeverity::Blocker,
            ));
        }

        if findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Blocker)
        {
            StageResult::failed(LifecycleStage::CodeReview, findings)
        } else {
            StageResult::passed(LifecycleStage::CodeReview)
        }
    }

    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult {
        let mut checks = Vec::new();

        let has_records = ctx
            .metadata
            .get("file_review_record")
            .is_some_and(|s| !s.is_empty());
        if has_records {
            checks.push(GateCheck::passed("File review records exist"));
        } else {
            checks.push(GateCheck::failed(
                "File review records exist",
                "No review records for in-scope files",
            ));
        }

        let no_blockers = ctx
            .metadata
            .get("blocking_findings")
            .map(|v| v == "none")
            .unwrap_or(false);
        if no_blockers {
            checks.push(GateCheck::passed("No blocking findings remain"));
        } else {
            checks.push(GateCheck::failed(
                "No blocking findings remain",
                "Blocking findings still present — must resolve before QA",
            ));
        }

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            GateResult::satisfied("Every file has a review record and no blocking findings remain")
        } else {
            GateResult::unsatisfied(checks, "Code review has unresolved gaps")
        }
    }

    fn required_artifact(&self) -> ArtifactDescriptor {
        ArtifactDescriptor {
            kind: ArtifactKind::FileReviewRecord,
            description: "File review records and area code-review recommendation",
        }
    }

    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
        let artifact = StageArtifact::new(
            ArtifactKind::FileReviewRecord,
            "Code Review stage — file review records and no-blockers confirmation",
            "code-review",
        );
        ctx.add_artifact(artifact.clone());
        artifact
    }

    fn status_summary(&self, ctx: &WorkflowContext) -> String {
        match ctx.stage_status(&LifecycleStage::CodeReview) {
            authority_domain::StageStatus::Pending => {
                "⏳ Code Review stage not started".to_string()
            }
            authority_domain::StageStatus::InProgress => {
                "🔄 Reviewing files for implementation quality...".to_string()
            }
            authority_domain::StageStatus::Completed => {
                "✅ Code Review complete — all files reviewed, no blockers".to_string()
            }
            authority_domain::StageStatus::Failed => {
                "❌ Code Review failed — blocking findings or missing records".to_string()
            }
            authority_domain::StageStatus::Blocked => {
                "🔒 Code Review blocked on author fix".to_string()
            }
            authority_domain::StageStatus::Bypassed => "⏭️ Code Review stage bypassed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn code_review_passes_with_complete_metadata() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("file_review_record".into(), "src/main.rs reviewed".into());
        ctx.metadata
            .insert("blocking_findings".into(), "none".into());

        let agent = CodeReviewStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(
            result.passed,
            "Code review should pass with complete metadata"
        );
    }

    #[tokio::test]
    async fn code_review_fails_with_blocking_findings() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("file_review_record".into(), "src/main.rs reviewed".into());
        ctx.metadata.insert(
            "blocking_findings".into(),
            "unwrap used without comment".into(),
        );

        let agent = CodeReviewStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(
            !result.passed,
            "Code review should fail with blocking findings"
        );
        assert!(result
            .findings
            .iter()
            .any(|f| f.code == "CR-BLOCKING-FINDINGS"));
    }

    #[test]
    fn code_review_exit_gate_checks() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = CodeReviewStageAgent::new();
        let gate = agent.check_exit_gate(&ctx);
        assert!(!gate.satisfied, "Gate should fail without metadata");

        ctx.metadata
            .insert("file_review_record".into(), "reviewed".into());
        ctx.metadata
            .insert("blocking_findings".into(), "none".into());
        let gate = agent.check_exit_gate(&ctx);
        assert!(
            gate.satisfied,
            "Gate should pass with records and no blockers"
        );
    }
}
