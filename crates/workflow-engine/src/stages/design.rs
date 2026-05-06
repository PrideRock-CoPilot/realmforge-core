//! # Design Stage Agent (Extended)
//!
//! Owned by: `domain-architect`
//! Required artifact: Design document, bounded context, state model
//! Exit gate: Design is reviewed and approved by domain-architect

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, ArtifactKind, FindingSeverity, GateCheck, GateResult, LifecycleStage,
    StageArtifact, StageFinding, StageResult, WorkflowContext,
};

use crate::traits::StageAgent;

/// The Design stage agent (extended) — validates design documentation,
/// bounded context boundaries, and state models.
#[derive(Clone, Debug)]
pub struct DesignStageAgent;

impl DesignStageAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DesignStageAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageAgent for DesignStageAgent {
    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Design
    }

    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
        let mut findings = Vec::new();

        // Check 1: Design document exists
        if ctx
            .metadata
            .get("design_document")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "DESIGN-NO-DOC",
                "No design document provided",
                FindingSeverity::Blocker,
            ));
        }

        // Check 2: Bounded context identified
        if !ctx.metadata.contains_key("bounded_context") {
            findings.push(StageFinding::new(
                "DESIGN-NO-CONTEXT",
                "No bounded context identified for this design",
                FindingSeverity::Blocker,
            ));
        }

        // Check 3: State model described
        if !ctx.metadata.contains_key("state_model") {
            findings.push(StageFinding::new(
                "DESIGN-NO-STATE",
                "No state model described — transitions and lifecycle unclear",
                FindingSeverity::Warning,
            ));
        }

        if findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Blocker)
        {
            StageResult::failed(LifecycleStage::Design, findings)
        } else {
            StageResult::passed(LifecycleStage::Design)
        }
    }

    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult {
        let mut checks = Vec::new();

        let has_design = ctx
            .metadata
            .get("design_document")
            .is_some_and(|s| !s.is_empty());
        if has_design {
            checks.push(GateCheck::passed("Design document exists"));
        } else {
            checks.push(GateCheck::failed(
                "Design document exists",
                "No design document provided",
            ));
        }

        let has_context = ctx.metadata.contains_key("bounded_context");
        if has_context {
            checks.push(GateCheck::passed("Bounded context identified"));
        } else {
            checks.push(GateCheck::failed(
                "Bounded context identified",
                "No bounded context identified",
            ));
        }

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            GateResult::satisfied("Design is reviewed and approved")
        } else {
            GateResult::unsatisfied(checks, "Design has unresolved gaps")
        }
    }

    fn required_artifact(&self) -> ArtifactDescriptor {
        ArtifactDescriptor {
            kind: ArtifactKind::DesignDocument,
            description: "Design document, bounded context, state model",
        }
    }

    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
        let artifact = StageArtifact::new(
            ArtifactKind::DesignDocument,
            "Design stage — design document and bounded context",
            "domain-architect",
        );
        ctx.add_artifact(artifact.clone());
        artifact
    }

    fn status_summary(&self, ctx: &WorkflowContext) -> String {
        match ctx.stage_status(&LifecycleStage::Design) {
            authority_domain::StageStatus::Pending => "⏳ Design stage not started".to_string(),
            authority_domain::StageStatus::InProgress => {
                "🔄 Drafting design document...".to_string()
            }
            authority_domain::StageStatus::Completed => {
                "✅ Design complete — reviewed and approved".to_string()
            }
            authority_domain::StageStatus::Failed => {
                "❌ Design failed — design document or bounded context missing".to_string()
            }
            authority_domain::StageStatus::Blocked => {
                "🔒 Design blocked on review feedback".to_string()
            }
            authority_domain::StageStatus::Bypassed => {
                "⏭️ Design stage bypassed (extended stage)".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn design_passes_with_complete_metadata() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("design_document".into(), "design.md".into());
        ctx.metadata.insert("bounded_context".into(), "auth".into());
        ctx.metadata
            .insert("state_model".into(), "LoginFlow".into());

        let agent = DesignStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(result.passed, "Design should pass with complete metadata");
    }

    #[tokio::test]
    async fn design_fails_without_document() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = DesignStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(!result.passed, "Design should fail without design document");
    }

    #[test]
    fn design_exit_gate_checks() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = DesignStageAgent::new();
        let gate = agent.check_exit_gate(&ctx);
        assert!(!gate.satisfied, "Gate should fail without metadata");

        ctx.metadata
            .insert("design_document".into(), "doc.md".into());
        ctx.metadata.insert("bounded_context".into(), "ctx".into());
        let gate = agent.check_exit_gate(&ctx);
        assert!(
            gate.satisfied,
            "Gate should pass with all required metadata"
        );
    }
}
