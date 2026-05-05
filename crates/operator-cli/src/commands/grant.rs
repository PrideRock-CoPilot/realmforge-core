use authority_domain::{ActorId, GrantId, GrantState, SkillGrant, TenantId};
use chrono::{Duration, Utc};
use clap::Subcommand;
use control_service::ServiceContext;

use crate::util::CliResult;

/// Grant lifecycle management commands.
#[derive(Subcommand)]
pub enum GrantCommand {
    /// Create a new skill grant for an actor
    Create {
        /// Grant ID (optional — auto-generated if omitted)
        #[arg(long)]
        id: Option<String>,
        /// Actor ID
        #[arg(long)]
        actor_id: String,
        /// Tenant ID
        #[arg(long)]
        tenant_id: String,
        /// Allowed actions (comma-separated)
        #[arg(long)]
        allowed_actions: String,
        /// Denied actions (comma-separated)
        #[arg(long)]
        denied_actions: Option<String>,
        /// Allowed file patterns (comma-separated)
        #[arg(long)]
        allowed_file_patterns: Option<String>,
        /// Denied file patterns (comma-separated)
        #[arg(long)]
        denied_file_patterns: Option<String>,
        /// Budget — max operations
        #[arg(long)]
        budget_operations: Option<u64>,
        /// Separation group
        #[arg(long)]
        separation_group: Option<String>,
        /// TTL in hours (default: 24)
        #[arg(long, default_value = "24")]
        ttl_hours: i64,
    },
    /// List all grants for an actor
    List {
        /// Actor ID
        actor_id: String,
    },
    /// Revoke a grant
    Revoke {
        /// Grant ID
        grant_id: String,
    },
    /// Inspect a grant's details
    Inspect {
        /// Grant ID
        grant_id: String,
    },
}

pub async fn handle_grant(cmd: GrantCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        GrantCommand::Create {
            id,
            actor_id,
            tenant_id,
            allowed_actions,
            denied_actions,
            allowed_file_patterns,
            denied_file_patterns,
            budget_operations,
            separation_group,
            ttl_hours,
        } => {
            create_grant(
                id,
                actor_id,
                tenant_id,
                allowed_actions,
                denied_actions,
                allowed_file_patterns,
                denied_file_patterns,
                budget_operations,
                separation_group,
                ttl_hours,
                ctx,
            )
            .await
        }
        GrantCommand::List { actor_id } => list_grants(actor_id, ctx).await,
        GrantCommand::Revoke { grant_id } => revoke_grant(grant_id, ctx).await,
        GrantCommand::Inspect { grant_id } => inspect_grant(grant_id, ctx).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn create_grant(
    id: Option<String>,
    actor_id: String,
    tenant_id: String,
    allowed_actions: String,
    denied_actions: Option<String>,
    allowed_file_patterns: Option<String>,
    denied_file_patterns: Option<String>,
    budget_operations: Option<u64>,
    separation_group: Option<String>,
    ttl_hours: i64,
    ctx: &ServiceContext,
) -> CliResult {
    let grant_id = match id {
        Some(v) => GrantId::new(v).map_err(|e| format!("invalid grant id: {}", e))?,
        None => GrantId::generate(),
    };

    let grant = SkillGrant {
        id: grant_id,
        actor_id: ActorId::new(&actor_id).map_err(|e| format!("invalid actor id: {}", e))?,
        tenant_id: TenantId::new(&tenant_id).map_err(|e| format!("invalid tenant id: {}", e))?,
        allowed_actions: allowed_actions
            .split(',')
            .map(|s| s.trim().to_string())
            .collect(),
        denied_actions: denied_actions
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect(),
        allowed_file_patterns: allowed_file_patterns
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect(),
        denied_file_patterns: denied_file_patterns
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect(),
        budget_tokens: None,
        budget_operations,
        separation_group: separation_group.unwrap_or_else(|| "default".to_string()),
        state: GrantState::Active,
        expires_at: Utc::now() + Duration::hours(ttl_hours),
        created_at: Utc::now(),
    };

    ctx.store().insert_grant(&grant).await?;
    println!("Grant created: {}", grant.id);
    println!("  Actor: {}", grant.actor_id);
    println!("  Tenant: {}", grant.tenant_id);
    println!("  State: {}", grant.state);
    println!("  Expires: {}", grant.expires_at);
    Ok(())
}

async fn list_grants(actor_id: String, ctx: &ServiceContext) -> CliResult {
    let actor_id = ActorId::new(&actor_id).map_err(|e| format!("invalid actor id: {}", e))?;
    let grants = ctx.store().list_grants_for_actor(&actor_id).await?;

    if grants.is_empty() {
        println!("No grants found for actor {}", actor_id);
        return Ok(());
    }

    println!("Grants for actor {}:", actor_id);
    for grant in &grants {
        println!(
            "  {} — state: {}, expires: {}",
            grant.id, grant.state, grant.expires_at
        );
        println!("    Allowed actions: {:?}", grant.allowed_actions);
        println!("    Group: {}", grant.separation_group);
    }
    Ok(())
}

async fn revoke_grant(grant_id: String, ctx: &ServiceContext) -> CliResult {
    let grant_id = GrantId::new(&grant_id).map_err(|e| format!("invalid grant id: {}", e))?;
    ctx.store().revoke_grant(&grant_id).await?;
    println!("Grant {} revoked", grant_id);
    Ok(())
}

async fn inspect_grant(grant_id: String, ctx: &ServiceContext) -> CliResult {
    let grant_id = GrantId::new(&grant_id).map_err(|e| format!("invalid grant id: {}", e))?;
    let grant = ctx
        .store()
        .get_grant(&grant_id)
        .await?
        .ok_or_else(|| format!("Grant not found: {}", grant_id))?;

    println!("Grant: {}", grant.id);
    println!("  Actor: {}", grant.actor_id);
    println!("  Tenant: {}", grant.tenant_id);
    println!("  State: {}", grant.state);
    println!("  Group: {}", grant.separation_group);
    println!("  Allowed actions: {:?}", grant.allowed_actions);
    println!("  Denied actions: {:?}", grant.denied_actions);
    println!("  Allowed file patterns: {:?}", grant.allowed_file_patterns);
    println!("  Denied file patterns: {:?}", grant.denied_file_patterns);
    println!("  Budget ops: {:?}", grant.budget_operations);
    println!("  Expires: {}", grant.expires_at);
    println!("  Created: {}", grant.created_at);
    Ok(())
}
