//! # Testing Stage Agent
//!
//! Owned by: `qa`
//! Required artifact: Test results, acceptance evidence, defects or residual risk
//! Exit gate: Required gates are run or explicitly blocked

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, ArtifactKind, FindingSeverity, GateCheck, GateResult, LifecycleStage,
    StageArtifact, StageFinding, StageResult, WorkflowContext,
};

use crate::traits::StageAgent;

/// The Testing stage agent — validates that tests have been run, acceptance
/// evidence is provided, and any defects or residual risks are documented.
#[derive(Clone, Debug)]
pub struct TestingStageAgent;

impl TestingStageAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TestingStageAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageAgent for TestingStageAgent {
    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Testing
    }

    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
        let mut findings = Vec::new();

        // Check 1: Test results provided
        if ctx
            .metadata
            .get("test_results")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "TEST-NO-RESULTS",
                "No test results provided — what tests were run?",
                FindingSeverity::Blocker,
            ));
        }

        // Check 2: Acceptance evidence provided
        if ctx
            .metadata
            .get("acceptance_evidence")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "TEST-NO-EVIDENCE",
                "No acceptance evidence — how do we know acceptance criteria are met?",
                FindingSeverity::Blocker,
            ));
        }

        // Check 3: Defects documented
        if !ctx.metadata.contains_key("defects") {
            findings.push(StageFinding::new(
                "TEST-NO-DEFECTS",
                "No defects documented — either zero known defects or unknown state",
                FindingSeverity::Note,
            ));
        }

        if findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Blocker)
        {
            StageResult::failed(LifecycleStage::Testing, findings)
        } else {
            StageResult::passed(LifecycleStage::Testing)
        }
    }

    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult {
        let mut checks = Vec::new();

        let has_results = ctx
            .metadata
            .get("test_results")
            .is_some_and(|s| !s.is_empty());
        if has_results {
            checks.push(GateCheck::passed("Test results provided"));
        } else {
            checks.push(GateCheck::failed(
                "Test results provided",
                "No test results — required gates were not run",
            ));
        }

        let has_evidence = ctx
            .metadata
            .get("acceptance_evidence")
            .is_some_and(|s| !s.is_empty());
        if has_evidence {
            checks.push(GateCheck::passed("Acceptance evidence provided"));
        } else {
            checks.push(GateCheck::failed(
                "Acceptance evidence provided",
                "No acceptance evidence — acceptance criteria are not verified",
            ));
        }

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            GateResult::satisfied("Required gates are run and acceptance evidence is provided")
        } else {
            GateResult::unsatisfied(checks, "Testing stage has unresolved gaps")
        }
    }

    fn required_artifact(&self) -> ArtifactDescriptor {
        ArtifactDescriptor {
            kind: ArtifactKind::TestResults,
            description: "Test results, acceptance evidence, defects or residual risk",
        }
    }

    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
        let artifact = StageArtifact::new(
            ArtifactKind::TestResults,
            "Testing stage — test results and acceptance evidence",
            "qa",
        );
        ctx.add_artifact(artifact.clone());
        artifact
    }

    fn status_summary(&self, ctx: &WorkflowContext) -> String {
        match ctx.stage_status(&LifecycleStage::Testing) {
            authority_domain::StageStatus::Pending => "⏳ Testing stage not started".to_string(),
            authority_domain::StageStatus::InProgress => {
                "🔄 Running tests and gathering evidence...".to_string()
            }
            authority_domain::StageStatus::Completed => {
                "✅ Testing complete — all gates run and acceptance evidence provided".to_string()
            }
            authority_domain::StageStatus::Failed => {
                "❌ Testing failed — missing test results or acceptance evidence".to_string()
            }
            authority_domain::StageStatus::Blocked => {
                "🔒 Testing blocked on unresolved defect".to_string()
            }
            authority_domain::StageStatus::Bypassed => "⏭️ Testing stage bypassed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn testing_passes_with_complete_metadata() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("test_results".into(), "32 passed, 0 failed".into());
        ctx.metadata
            .insert("acceptance_evidence".into(), "e2e tests pass".into());
        ctx.metadata.insert("defects".into(), "none".into());

        let agent = TestingStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(result.passed, "Testing should pass with complete metadata");
    }

    #[tokio::test]
    async fn testing_fails_without_results() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = TestingStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(!result.passed, "Testing should fail without results");
        assert!(result.findings.iter().any(|f| f.code == "TEST-NO-RESULTS"));
    }

    #[test]
    fn testing_exit_gate_checks() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = TestingStageAgent::new();
        let gate = agent.check_exit_gate(&ctx);
        assert!(!gate.satisfied, "Gate should fail without metadata");

        ctx.metadata
            .insert("test_results".into(), "all pass".into());
        ctx.metadata
            .insert("acceptance_evidence".into(), "evidence".into());
        let gate = agent.check_exit_gate(&ctx);
        assert!(gate.satisfied, "Gate should pass with results and evidence");
    }
}
