use authority_domain::login::{LoginCredentials, LoginPolicyConfig, Scope};
use authority_domain::{ActorId, TenantId};
use clap::{Args, Subcommand};
use control_service::ServiceContext;
use serde_json::json;

use crate::util::{output_json, output_pretty, CliResult};

/// Login management commands.
#[derive(Subcommand)]
pub enum LoginCommand {
    /// Authenticate an actor and issue a session.
    Login(LoginArgs),
    /// Set the login policy for a tenant.
    PolicySet(PolicySetArgs),
    /// Get the login policy for a tenant.
    PolicyGet(PolicyGetArgs),
    /// List active login blocks for a tenant.
    BlocksList(BlocksListArgs),
}

#[derive(Args)]
pub struct LoginArgs {
    pub tenant_id: TenantId,
    pub project_id: authority_domain::ProjectId,
    pub actor_id: ActorId,
    /// Raw credential string to authenticate with.
    pub credential: String,
    /// Scope string matching allowlist patterns.
    pub scope: String,
}

#[derive(Args)]
pub struct PolicySetArgs {
    pub tenant_id: TenantId,
    /// Maximum failed login attempts before auto-block.
    #[arg(long, default_value = "5")]
    pub max_failed_attempts: u32,
    /// Time window (seconds) for counting failed attempts.
    #[arg(long, default_value = "60")]
    pub window_seconds: u64,
    /// Duration (seconds) of an auto-block.
    #[arg(long, default_value = "300")]
    pub block_duration_seconds: u64,
    /// Maximum requests per window before rate limiting.
    #[arg(long, default_value = "10")]
    pub max_requests_per_window: u32,
    /// Enable credential validation (constant-time compare).
    #[arg(long, default_value = "true")]
    pub credential_validation_enabled: bool,
}

#[derive(Args)]
pub struct PolicyGetArgs {
    pub tenant_id: TenantId,
}

#[derive(Args)]
pub struct BlocksListArgs {
    pub tenant_id: TenantId,
}

pub async fn handle_login_command(
    cmd: LoginCommand,
    ctx: &ServiceContext,
    pretty: bool,
) -> CliResult {
    match cmd {
        LoginCommand::Login(args) => {
            let scope = Scope::new(&args.scope)?;
            let credentials = LoginCredentials {
                actor_id: args.actor_id,
                credential: args.credential.into_bytes(),
                scope,
            };
            let response = ctx
                .login
                .handle_login(args.tenant_id, args.project_id, credentials)
                .await?;
            if pretty {
                output_pretty(&response);
            } else {
                output_json(&response);
            }
            Ok(())
        }
        LoginCommand::PolicySet(args) => {
            let config = LoginPolicyConfig {
                max_failed_attempts: args.max_failed_attempts,
                window_seconds: args.window_seconds,
                block_duration_seconds: args.block_duration_seconds,
                max_requests_per_window: args.max_requests_per_window,
                credential_validation_enabled: args.credential_validation_enabled,
            };
            ctx.login.set_policy(&args.tenant_id, &config).await?;
            let result = json!({"status": "policy updated", "tenant_id": args.tenant_id.as_str()});
            output_json(&result);
            Ok(())
        }
        LoginCommand::PolicyGet(args) => {
            let policy = ctx.login.get_policy(&args.tenant_id).await?;
            let result = json!({
                "tenant_id": args.tenant_id.as_str(),
                "config": policy,
            });
            output_json(&result);
            Ok(())
        }
        LoginCommand::BlocksList(args) => {
            let blocks = ctx.login.list_blocks(&args.tenant_id).await?;
            let result = json!({
                "tenant_id": args.tenant_id.as_str(),
                "blocks": blocks,
            });
            output_json(&result);
            Ok(())
        }
    }
}
