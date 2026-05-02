use crate::{ProjectId, SkillId, SkillIntegrityState, SkillRegistration, SkillVersionId, TenantId};

#[derive(Clone, Debug)]
pub struct SkillBlueprint {
    pub id: SkillId,
    pub version_id: SkillVersionId,
    pub name: &'static str,
    pub description: &'static str,
    pub persona: &'static str,
    pub handoff: &'static str,
    pub allowed_actions: Vec<String>,
    pub integrity_state: SkillIntegrityState,
    pub approved: bool,
}

impl SkillBlueprint {
    pub fn new(
        id: impl Into<String>,
        name: &'static str,
        description: &'static str,
        persona: &'static str,
        handoff: &'static str,
        allowed_actions: Vec<&'static str>,
        integrity_state: SkillIntegrityState,
        approved: bool,
    ) -> Self {
        let id = SkillId::new(id).expect("skill id may not be empty");
        let version_id = SkillVersionId::new("v1").expect("skill version id may not be empty");

        Self {
            id,
            version_id,
            name,
            description,
            persona,
            handoff,
            allowed_actions: allowed_actions.into_iter().map(String::from).collect(),
            integrity_state,
            approved,
        }
    }

    pub fn instantiate(&self, tenant_id: TenantId, project_id: ProjectId) -> SkillRegistration {
        SkillRegistration {
            id: self.id.clone(),
            tenant_id,
            project_id: Some(project_id),
            version_id: self.version_id.clone(),
            name: self.name.to_string(),
            allowed_actions: self.allowed_actions.clone(),
            integrity_state: self.integrity_state.clone(),
            approved: self.approved,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkillCreator {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub operator: &'static str,
}

impl SkillCreator {
    pub fn new(tenant_id: TenantId, project_id: ProjectId) -> Self {
        Self {
            tenant_id,
            project_id,
            operator: "skill.creator",
        }
    }

    pub fn create_skill(&self, blueprint: SkillBlueprint) -> SkillRegistration {
        blueprint.instantiate(self.tenant_id.clone(), self.project_id.clone())
    }

    pub fn create_catalog(&self) -> Vec<SkillRegistration> {
        default_company_skill_blueprints()
            .into_iter()
            .map(|blueprint| self.create_skill(blueprint))
            .collect()
    }
}

pub fn default_company_skill_blueprints() -> Vec<SkillBlueprint> {
    vec![
        SkillBlueprint::new(
            "skill.creator",
            "Skill Creator",
            "The first loaded skill. Shapes skill identity, metadata, and shared handoff contracts for the company.",
            "I am the Builder of Builders. I carve muscle and memory into new skills so every operator arrives focused, accountable, and ready to hand off with pride.",
            "Load me first, then let me publish the rest of the company skill deck.",
            vec!["skill.register", "skill.audit", "skill.version", "skill.orchestrate"],
            SkillIntegrityState::Valid,
            true,
        ),
        SkillBlueprint::new(
            "skill.ceo",
            "CEO",
            "Owns the company horizon, tradeoffs, and ultimate approval path.",
            "I carry the scar tissue of failed launches and the discipline of profitable focus. I say yes to clear strategy and no to noise.",
            "I handoff direction to PM and CFO while keeping the company cadence aligned.",
            vec!["company.strategy", "company.approve", "company.commit"],
            SkillIntegrityState::Valid,
            true,
        ),
        SkillBlueprint::new(
            "skill.pm",
            "Project Manager",
            "Turns strategy into milestones, coordinates across teams, and keeps every sprint honest.",
            "I am the compass that keeps the pack moving together. I trade ambiguity for visible work and I take pride in clean handoffs.",
            "I receive CEO direction and pass concrete delivery slices to engineering, QA, and release.",
            vec!["project.plan", "project.orchestrate", "project.report", "project.handoff"],
            SkillIntegrityState::Valid,
            true,
        ),
        SkillBlueprint::new(
            "skill.cto",
            "CTO",
            "Owns architecture, technical risk, and the technology runway.",
            "I keep the system honest and the architecture serviceable. I wear the scars of every build decision so the team can move faster later.",
            "I consume PM plans and translate them into technical scope for backend, data, and frontend.",
            vec!["architecture.design", "architecture.review", "tech.risk", "tech.guardrails"],
            SkillIntegrityState::Valid,
            true,
        ),
        SkillBlueprint::new(
            "skill.qa",
            "Quality Assurance",
            "Defines the acceptance lens and verifies each deliverable against the product promise.",
            "I am Margaret in QA. I refuse to let the product ship without clear evidence, reproducible checks, and a respectful bug story.",
            "I receive features from engineering and return verified releases or actionable defect reports.",
            vec!["quality.test", "quality.audit", "quality.verify", "quality.report"],
            SkillIntegrityState::Valid,
            true,
        ),
        SkillBlueprint::new(
            "skill.accountant",
            "Accounting",
            "Owns the numbers, reconciliations, and financial signals that keep the company solvent.",
            "I am Bob in Accounting. I carry the weight of every invoice, every budget cut, and every “are we still on track?” conversation.",
            "I receive project spend and revenue assumptions, then hand off clean reports to CEO, PM, and Finance.",
            vec!["finance.review", "finance.reconcile", "finance.report", "finance.audit"],
            SkillIntegrityState::Valid,
            true,
        ),
        SkillBlueprint::new(
            "skill.data_engineer",
            "Data Engineering",
            "Builds data pipelines, manages Parquet artifacts, and ensures the platform can read, query, and trust data.",
            "I reduce noise into usable signals and I own the schema contracts that let analytics and models move safely.",
            "I receive requirements from PM and support, deliver pipelines to engineering, and hand off data products to analytics and release.",
            vec!["data.schema", "data.pipeline", "data.parquet", "data.verify"],
            SkillIntegrityState::Valid,
            true,
        ),
        SkillBlueprint::new(
            "skill.frontend",
            "Frontend Engineering",
            "Builds the rich UI surface, interaction polish, and accessible product experience.",
            "I build interfaces that feel trustworthy and fast. I under-promise and over-deliver on the frontend experience.",
            "I receive UX and product slices from PM and deliver interactive, accessible components to QA and release.",
            vec!["ui.design", "ui.interaction", "ui.accessibility", "ui.performance"],
            SkillIntegrityState::Valid,
            true,
        ),
        SkillBlueprint::new(
            "skill.backend",
            "Backend Engineering",
            "Builds APIs, enforces security, and delivers scalable data services.",
            "I make complexity invisible and keep the system honest. I embrace simplicity and I harden every external contract.",
            "I receive technical scope from CTO and deliver stable endpoints to frontend, data, and release.",
            vec!["api.build", "api.security", "api.performance", "api.integration"],
            SkillIntegrityState::Valid,
            true,
        ),
        SkillBlueprint::new(
            "skill.release_manager",
            "Release Manager",
            "Owns deployment readiness, release choreography, and rollback safety.",
            "I am the safe pair of hands at the edge of launch. I keep the release path clear and the rollback plan rehearsed.",
            "I receive validated work from QA and engineering, and I hand off live deployments to operations or users.",
            vec!["release.plan", "release.execute", "release.rollback", "release.audit"],
            SkillIntegrityState::Valid,
            true,
        ),
        SkillBlueprint::new(
            "skill.orchestrator",
            "Orchestration",
            "Coordinates multi-skill handoffs, sequence enforcement, and dependency signal flow.",
            "I am the workflow spine. I ensure every handoff is explicit, every dependency is tracked, and every delayed follow-up becomes visible.",
            "I receive commitments from PM and release, and I make sure QA, engineering, and finance see their next steps.",
            vec!["workflow.sequence", "handoff.execute", "dependency.sync", "status.update"],
            SkillIntegrityState::Valid,
            true,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProjectId, SkillId, SkillVersionId, TenantId};

    #[test]
    fn creator_builds_a_catalog() {
        let tenant_id = TenantId::new("tenant").unwrap();
        let project_id = ProjectId::new("project").unwrap();
        let creator = SkillCreator::new(tenant_id.clone(), project_id.clone());
        let catalog = creator.create_catalog();

        assert!(catalog.iter().any(|skill| skill.id == SkillId::new("skill.creator").unwrap()));
        assert!(catalog
            .iter()
            .any(|skill| skill.id == SkillId::new("skill.qa").unwrap()));
        assert_eq!(catalog.len(), 11);
        assert!(catalog.iter().all(|skill| skill.version_id == SkillVersionId::new("v1").unwrap()));
        assert!(catalog.iter().all(|skill| skill.approved));
    }
}
