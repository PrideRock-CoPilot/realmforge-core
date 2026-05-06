//! # Development Stage Agent
//!
//! Owned by: `backend`, `frontend`, `data-engineer`
//! Required artifact: Focused implementation diff within approved files
//! Exit gate: Code follows docs-first law and does not expand scope silently

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, ArtifactKind, FindingSeverity, GateCheck, GateResult, LifecycleStage,
    StageArtifact, StageFinding, StageResult, WorkflowContext,
};

use crate::traits::StageAgent;

/// The Development stage agent — validates that implementation stays within
/// approved scope, follows docs-first law, and produces a focused diff.
#[derive(Clone, Debug)]
pub struct DevelopmentStageAgent;

impl DevelopmentStageAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DevelopmentStageAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageAgent for DevelopmentStageAgent {
    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Development
    }

    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
        let mut findings = Vec::new();

        // Check 1: Implementation diff documented
        if ctx
            .metadata
            .get("implementation_diff")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "DEV-NO-DIFF",
                "No implementation diff documented — what changed?",
                FindingSeverity::Blocker,
            ));
        }

        // Check 2: Docs-first law verified
        let docs_first = ctx
            .metadata
            .get("docs_first_verified")
            .map(|v| v == "true")
            .unwrap_or(false);
        if !docs_first {
            findings.push(StageFinding::new(
                "DEV-NO-DOCS-FIRST",
                "Docs-first law not verified — spec, metadata, and tests must exist before implementation",
                FindingSeverity::Blocker,
            ));
        }

        // Check 3: Scope verification
        if !ctx.metadata.contains_key("scope_verified") {
            findings.push(StageFinding::new(
                "DEV-NO-SCOPE",
                "Scope not verified — does this implementation stay within the approved boundary?",
                FindingSeverity::Warning,
            ));
        }

        if findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Blocker)
        {
            StageResult::failed(LifecycleStage::Development, findings)
        } else {
            StageResult::passed(LifecycleStage::Development)
        }
    }

    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult {
        let mut checks = Vec::new();

        let has_diff = ctx
            .metadata
            .get("implementation_diff")
            .is_some_and(|s| !s.is_empty());
        if has_diff {
            checks.push(GateCheck::passed("Implementation diff documented"));
        } else {
            checks.push(GateCheck::failed(
                "Implementation diff documented",
                "No implementation diff — cannot verify what changed",
            ));
        }

        let docs_first = ctx
            .metadata
            .get("docs_first_verified")
            .map(|v| v == "true")
            .unwrap_or(false);
        if docs_first {
            checks.push(GateCheck::passed("Docs-first law verified"));
        } else {
            checks.push(GateCheck::failed(
                "Docs-first law verified",
                "Spec, metadata, and tests must exist before implementation",
            ));
        }

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            GateResult::satisfied("Code follows docs-first law and does not expand scope")
        } else {
            GateResult::unsatisfied(checks, "Development stage has unresolved gaps")
        }
    }

    fn required_artifact(&self) -> ArtifactDescriptor {
        ArtifactDescriptor {
            kind: ArtifactKind::ImplementationDiff,
            description: "Focused implementation diff within approved files",
        }
    }

    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
        let artifact = StageArtifact::new(
            ArtifactKind::ImplementationDiff,
            "Development stage — implementation diff and docs-first verification",
            "backend",
        );
        ctx.add_artifact(artifact.clone());
        artifact
    }

    fn status_summary(&self, ctx: &WorkflowContext) -> String {
        match ctx.stage_status(&LifecycleStage::Development) {
            authority_domain::StageStatus::Pending => {
                "⏳ Development stage not started".to_string()
            }
            authority_domain::StageStatus::InProgress => {
                "🔄 Implementing focused changes...".to_string()
            }
            authority_domain::StageStatus::Completed => {
                "✅ Development complete — diff produced and docs-first law verified".to_string()
            }
            authority_domain::StageStatus::Failed => {
                "❌ Development failed — missing diff or docs-first violation".to_string()
            }
            authority_domain::StageStatus::Blocked => {
                "🔒 Development blocked on unresolved architecture decision".to_string()
            }
            authority_domain::StageStatus::Bypassed => "⏭️ Development stage bypassed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn development_passes_with_complete_metadata() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("implementation_diff".into(), "Added login handler".into());
        ctx.metadata
            .insert("docs_first_verified".into(), "true".into());
        ctx.metadata.insert("scope_verified".into(), "true".into());

        let agent = DevelopmentStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(
            result.passed,
            "Development should pass with complete metadata"
        );
    }

    #[tokio::test]
    async fn development_fails_without_docs_first() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("implementation_diff".into(), "diff".into());

        let agent = DevelopmentStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(
            !result.passed,
            "Development should fail without docs-first law"
        );
        assert!(result
            .findings
            .iter()
            .any(|f| f.code == "DEV-NO-DOCS-FIRST"));
    }

    #[test]
    fn development_exit_gate_checks() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = DevelopmentStageAgent::new();
        let gate = agent.check_exit_gate(&ctx);
        assert!(!gate.satisfied, "Gate should fail without metadata");

        ctx.metadata
            .insert("implementation_diff".into(), "diff".into());
        ctx.metadata
            .insert("docs_first_verified".into(), "true".into());
        let gate = agent.check_exit_gate(&ctx);
        assert!(gate.satisfied, "Gate should pass with required metadata");
    }
}
