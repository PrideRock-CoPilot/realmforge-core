//! # Documentation Stage Agent
//!
//! Owned by: `tech-writer`, `pm`, `qa`
//! Required artifact: Updated specs, phase gates, roadmap notes, verification evidence
//! Exit gate: Docs match implementation and do not over-certify

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, ArtifactKind, FindingSeverity, GateCheck, GateResult, LifecycleStage,
    StageArtifact, StageFinding, StageResult, WorkflowContext,
};

use crate::traits::StageAgent;

/// The Documentation stage agent — validates that specs, metadata, roadmaps,
/// and phase plans are updated to match the implemented behavior.
#[derive(Clone, Debug)]
pub struct DocumentationStageAgent;

impl DocumentationStageAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DocumentationStageAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageAgent for DocumentationStageAgent {
    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Documentation
    }

    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
        let mut findings = Vec::new();

        // Check 1: Docs verified against implementation
        let docs_verified = ctx
            .metadata
            .get("docs_verified")
            .map(|v| v == "true")
            .unwrap_or(false);
        if !docs_verified {
            findings.push(StageFinding::new(
                "DOCS-NOT-VERIFIED",
                "Documentation has not been verified against implementation",
                FindingSeverity::Blocker,
            ));
        }

        // Check 2: Specs updated
        if !ctx.metadata.contains_key("specs_updated") {
            findings.push(StageFinding::new(
                "DOCS-NO-SPECS",
                "No spec update record — were relevant spec docs updated?",
                FindingSeverity::Warning,
            ));
        }

        // Check 3: Roadmap/phase gates updated
        if !ctx.metadata.contains_key("roadmap_updated") {
            findings.push(StageFinding::new(
                "DOCS-NO-ROADMAP",
                "No roadmap update record — were phase gates updated?",
                FindingSeverity::Note,
            ));
        }

        if findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Blocker)
        {
            StageResult::failed(LifecycleStage::Documentation, findings)
        } else {
            StageResult::passed(LifecycleStage::Documentation)
        }
    }

    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult {
        let mut checks = Vec::new();

        let docs_match = ctx
            .metadata
            .get("docs_verified")
            .map(|v| v == "true")
            .unwrap_or(false);
        if docs_match {
            checks.push(GateCheck::passed("Documentation matches implementation"));
        } else {
            checks.push(GateCheck::failed(
                "Documentation matches implementation",
                "Docs have not been verified — risk of drift between docs and code",
            ));
        }

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            GateResult::satisfied("Docs match implementation and do not over-certify")
        } else {
            GateResult::unsatisfied(checks, "Documentation stage has unresolved gaps")
        }
    }

    fn required_artifact(&self) -> ArtifactDescriptor {
        ArtifactDescriptor {
            kind: ArtifactKind::DocumentationUpdate,
            description: "Updated specs, phase gates, roadmap notes, verification evidence",
        }
    }

    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
        let artifact = StageArtifact::new(
            ArtifactKind::DocumentationUpdate,
            "Documentation stage — spec updates and verification evidence",
            "tech-writer",
        );
        ctx.add_artifact(artifact.clone());
        artifact
    }

    fn status_summary(&self, ctx: &WorkflowContext) -> String {
        match ctx.stage_status(&LifecycleStage::Documentation) {
            authority_domain::StageStatus::Pending => {
                "⏳ Documentation stage not started".to_string()
            }
            authority_domain::StageStatus::InProgress => {
                "🔄 Updating specs, roadmaps, and verification evidence...".to_string()
            }
            authority_domain::StageStatus::Completed => {
                "✅ Documentation complete — docs match implementation".to_string()
            }
            authority_domain::StageStatus::Failed => {
                "❌ Documentation failed — docs not verified against implementation".to_string()
            }
            authority_domain::StageStatus::Blocked => {
                "🔒 Documentation blocked on clarification from development".to_string()
            }
            authority_domain::StageStatus::Bypassed => {
                "⏭️ Documentation stage bypassed".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn documentation_passes_with_verification() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata.insert("docs_verified".into(), "true".into());
        ctx.metadata.insert("specs_updated".into(), "true".into());
        ctx.metadata.insert("roadmap_updated".into(), "true".into());

        let agent = DocumentationStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(result.passed, "Documentation should pass with verification");
    }

    #[tokio::test]
    async fn documentation_fails_without_verification() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = DocumentationStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(
            !result.passed,
            "Documentation should fail without verification"
        );
        assert!(result
            .findings
            .iter()
            .any(|f| f.code == "DOCS-NOT-VERIFIED"));
    }

    #[test]
    fn documentation_exit_gate_checks() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = DocumentationStageAgent::new();
        let gate = agent.check_exit_gate(&ctx);
        assert!(!gate.satisfied, "Gate should fail without verification");

        ctx.metadata.insert("docs_verified".into(), "true".into());
        let gate = agent.check_exit_gate(&ctx);
        assert!(gate.satisfied, "Gate should pass with verification");
    }
}
