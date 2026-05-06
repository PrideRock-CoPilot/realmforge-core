//! # Architecture Stage Agent
//!
//! Owned by: `cto`
//! Required artifact: Boundary contract, affected crates/files, rollback path, risks
//! Exit gate: Layer law is satisfied and unresolved decisions are recorded

use async_trait::async_trait;
use authority_domain::{
    ArtifactDescriptor, ArtifactKind, FindingSeverity, GateCheck, GateResult, LifecycleStage,
    StageArtifact, StageFinding, StageResult, WorkflowContext,
};

use crate::traits::StageAgent;

/// The Architecture stage agent — validates architecture contracts,
/// layer law compliance, risk assessment, and decision recording.
#[derive(Clone, Debug)]
pub struct ArchitectureStageAgent;

impl ArchitectureStageAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ArchitectureStageAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageAgent for ArchitectureStageAgent {
    fn stage(&self) -> LifecycleStage {
        LifecycleStage::Architecture
    }

    async fn execute(&self, ctx: &mut WorkflowContext) -> StageResult {
        let mut findings = Vec::new();

        // Check 1: Boundary contract defined
        if ctx
            .metadata
            .get("boundary_contract")
            .map_or(true, |s| s.is_empty())
        {
            findings.push(StageFinding::new(
                "ARCH-NO-CONTRACT",
                "No boundary contract defined — what interfaces are affected?",
                FindingSeverity::Blocker,
            ));
        }

        // Check 2: Affected files/crates identified
        if !ctx.metadata.contains_key("affected_files") {
            findings.push(StageFinding::new(
                "ARCH-NO-FILES",
                "No affected files or crates identified",
                FindingSeverity::Warning,
            ));
        }

        // Check 3: Rollback path defined
        if !ctx.metadata.contains_key("rollback_path") {
            findings.push(StageFinding::new(
                "ARCH-NO-ROLLBACK",
                "No rollback path defined — how do we undo this if it fails?",
                FindingSeverity::Warning,
            ));
        }

        // Check 4: Layer law verified
        let layer_law = ctx
            .metadata
            .get("layer_law_verified")
            .map(|v| v == "true")
            .unwrap_or(false);
        if !layer_law {
            findings.push(StageFinding::new(
                "ARCH-LAYER-LAW",
                "Layer law not verified — transport, policy, and store layers must be respected",
                FindingSeverity::Blocker,
            ));
        }

        // Check 5: Risks assessed
        if !ctx.metadata.contains_key("risks_assessed") {
            findings.push(StageFinding::new(
                "ARCH-NO-RISKS",
                "No risk assessment documented",
                FindingSeverity::Warning,
            ));
        }

        if findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Blocker)
        {
            StageResult::failed(LifecycleStage::Architecture, findings)
        } else {
            StageResult::passed(LifecycleStage::Architecture)
        }
    }

    fn check_exit_gate(&self, ctx: &WorkflowContext) -> GateResult {
        let mut checks = Vec::new();

        let has_contract = ctx
            .metadata
            .get("boundary_contract")
            .is_some_and(|s| !s.is_empty());
        if has_contract {
            checks.push(GateCheck::passed("Boundary contract defined"));
        } else {
            checks.push(GateCheck::failed(
                "Boundary contract defined",
                "No boundary contract defined",
            ));
        }

        let layer_law_ok = ctx
            .metadata
            .get("layer_law_verified")
            .map(|v| v == "true")
            .unwrap_or(false);
        if layer_law_ok {
            checks.push(GateCheck::passed("Layer law satisfied"));
        } else {
            checks.push(GateCheck::failed(
                "Layer law satisfied",
                "Layer law not verified — transport crates must not contain business logic",
            ));
        }

        let risks_assessed = ctx.metadata.contains_key("risks_assessed");
        if risks_assessed {
            checks.push(GateCheck::passed("Risks assessed"));
        } else {
            checks.push(GateCheck::failed(
                "Risks assessed",
                "No risk assessment documented",
            ));
        }

        let all_passed = checks.iter().all(|c| c.passed);
        if all_passed {
            GateResult::satisfied("Layer law is satisfied and unresolved decisions are recorded")
        } else {
            GateResult::unsatisfied(checks, "Architecture stage has unresolved gaps")
        }
    }

    fn required_artifact(&self) -> ArtifactDescriptor {
        ArtifactDescriptor {
            kind: ArtifactKind::ArchitectureContract,
            description: "Boundary contract, affected crates/files, rollback path, risks",
        }
    }

    fn produce_artifact(&self, ctx: &mut WorkflowContext) -> StageArtifact {
        let artifact = StageArtifact::new(
            ArtifactKind::ArchitectureContract,
            "Architecture stage — boundary contract and layer law verification",
            "cto",
        );
        ctx.add_artifact(artifact.clone());
        artifact
    }

    fn status_summary(&self, ctx: &WorkflowContext) -> String {
        match ctx.stage_status(&LifecycleStage::Architecture) {
            authority_domain::StageStatus::Pending => {
                "⏳ Architecture stage not started".to_string()
            }
            authority_domain::StageStatus::InProgress => {
                "🔄 Evaluating architecture contracts and layer law...".to_string()
            }
            authority_domain::StageStatus::Completed => {
                "✅ Architecture complete — contracts defined and layer law verified".to_string()
            }
            authority_domain::StageStatus::Failed => {
                "❌ Architecture failed — boundary contract or layer law violated".to_string()
            }
            authority_domain::StageStatus::Blocked => {
                "🔒 Architecture blocked on unresolved decision".to_string()
            }
            authority_domain::StageStatus::Bypassed => "⏭️ Architecture stage bypassed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authority_domain::{ProjectId, TenantId};

    #[tokio::test]
    async fn architecture_passes_with_complete_metadata() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("boundary_contract".into(), "control-api v2".into());
        ctx.metadata
            .insert("affected_files".into(), "5 crates".into());
        ctx.metadata
            .insert("rollback_path".into(), "git revert".into());
        ctx.metadata
            .insert("layer_law_verified".into(), "true".into());
        ctx.metadata.insert("risks_assessed".into(), "yes".into());

        let agent = ArchitectureStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(
            result.passed,
            "Architecture should pass with complete metadata"
        );
    }

    #[tokio::test]
    async fn architecture_fails_without_layer_law() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);
        ctx.metadata
            .insert("boundary_contract".into(), "contract".into());

        let agent = ArchitectureStageAgent::new();
        let result = agent.execute(&mut ctx).await;
        assert!(!result.passed, "Architecture should fail without layer law");
        assert!(result.findings.iter().any(|f| f.code == "ARCH-LAYER-LAW"));
    }

    #[test]
    fn architecture_exit_gate_checks() {
        let tid = TenantId::new("t1").unwrap();
        let pid = ProjectId::new("p1").unwrap();
        let mut ctx = WorkflowContext::new("wf-1", tid, pid, false);

        let agent = ArchitectureStageAgent::new();
        let gate = agent.check_exit_gate(&ctx);
        assert!(!gate.satisfied, "Gate should fail without metadata");

        ctx.metadata
            .insert("boundary_contract".into(), "contract".into());
        ctx.metadata
            .insert("layer_law_verified".into(), "true".into());
        ctx.metadata.insert("risks_assessed".into(), "yes".into());
        let gate = agent.check_exit_gate(&ctx);
        assert!(gate.satisfied, "Gate should pass with required metadata");
    }
}
