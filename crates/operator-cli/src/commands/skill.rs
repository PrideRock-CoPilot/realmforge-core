use authority_domain::{ProjectId, SkillId, TenantId};
use clap::{Args, Subcommand};
use control_service::ServiceContext;
use serde_json::json;

use crate::util::{output_json, CliResult};

/// Skill registration and management commands.
#[derive(Subcommand)]
pub enum SkillCommand {
    Register(RegisterArgs),
    List,
    Validate(ValidateArgs),
}

#[derive(Args)]
pub struct RegisterArgs {
    pub skill_id: SkillId,
    pub tenant_id: TenantId,
    pub name: String,
    #[arg(long)]
    pub project_id: Option<ProjectId>,
    #[arg(long)]
    pub actions: Vec<String>,
}

#[derive(Args)]
pub struct ValidateArgs {
    pub skill_id: SkillId,
}

pub async fn handle_skill(cmd: SkillCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        SkillCommand::Register(args) => {
            let registration = ctx
                .skills
                .register_skill(
                    &args.skill_id,
                    &args.tenant_id,
                    args.project_id.as_ref(),
                    &args.name,
                    args.actions,
                )
                .await?;
            output_json(&registration);
        }
        SkillCommand::List => {
            output_json(&json!({"skills": []}));
        }
        SkillCommand::Validate(args) => {
            let msg = format!(
                "Skill validation for {} — requires store-backed lookup",
                args.skill_id
            );
            output_json(&json!({"status": "pending", "message": msg}));
        }
    }
    Ok(())
}
