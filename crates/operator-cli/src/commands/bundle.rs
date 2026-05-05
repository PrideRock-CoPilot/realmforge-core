use clap::Subcommand;
use control_service::ServiceContext;
use serde_json::Value;

use crate::util::CliResult;

/// Subcommands for bundle lifecycle management.
#[derive(Subcommand)]
pub enum BundleCommand {
    /// Create a new bundle manifest
    Create {
        bundle_id: String,
        version: String,
        app_id: String,
        #[arg(long)]
        governance_signature: String,
        #[arg(long)]
        release_approval_ref: Option<String>,
        /// Artifact hashes as key=value pairs, e.g. --hash "main.wasm=abc123"
        #[arg(long = "hash", value_parser = parse_hash)]
        hashes: Vec<(String, String)>,
    },
    /// List bundles for an app
    List {
        app_id: String,
        #[arg(long, default_value = "20")]
        limit: i64,
        #[arg(long, default_value = "0")]
        offset: i64,
    },
    /// Get a bundle by ID
    Get {
        bundle_id: String,
    },
    /// Verify a bundle (update status)
    Verify {
        bundle_id: String,
        status: String,
    },
    /// Deploy a bundle
    Deploy {
        bundle_id: String,
    },
}

fn parse_hash(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() == 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err(format!("Invalid hash format: {s} — expected key=value"))
    }
}

pub async fn handle_bundle(cmd: BundleCommand, ctx: &ServiceContext) -> CliResult {
    match cmd {
        BundleCommand::Create {
            bundle_id,
            version,
            app_id,
            governance_signature,
            release_approval_ref,
            hashes,
        } => {
            let manifest = authority_domain::BundleManifest {
                bundle_id: authority_domain::BundleId::new(bundle_id)?,
                version,
                app_id,
                artifact_hashes: hashes,
                governance_signature,
                release_approval_ref: release_approval_ref
                    .map(authority_domain::ApprovalId::new)
                    .transpose()?,
                built_at: chrono::Utc::now(),
                status: authority_domain::BundleStatus::Building,
            };
            ctx.bundles.record_bundle(&manifest).await?;
            println!("{}", serde_json::to_string_pretty(&manifest)?);
        }
        BundleCommand::List {
            app_id,
            limit,
            offset,
        } => {
            let bundles = ctx.bundles.list_bundles(&app_id, limit, offset).await?;
            println!("{}", serde_json::to_string_pretty(&bundles)?);
        }
        BundleCommand::Get { bundle_id } => {
            let manifest = ctx
                .bundles
                .get_bundle(&authority_domain::BundleId::new(bundle_id)?)
                .await?;
            println!("{}", serde_json::to_string_pretty(&manifest)?);
        }
        BundleCommand::Verify { bundle_id, status } => {
            let bundle_status = match status.as_str() {
                "verified" => authority_domain::BundleStatus::Verified,
                "signed" => authority_domain::BundleStatus::Signed,
                _ => authority_domain::BundleStatus::Failed(status.clone()),
            };
            ctx.bundles
                .update_bundle_status(
                    &authority_domain::BundleId::new(bundle_id.clone())?,
                    &bundle_status,
                )
                .await?;
            let result: Value = serde_json::json!({
                "bundle_id": bundle_id,
                "status": status,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        BundleCommand::Deploy { bundle_id } => {
            let bundle_id = authority_domain::BundleId::new(bundle_id)?;
            let manifest = ctx.bundles.get_bundle(&bundle_id).await?;
            ctx.bundles
                .update_bundle_status(&bundle_id, &authority_domain::BundleStatus::Deployed)
                .await?;
            let result: Value = serde_json::json!({
                "bundle_id": bundle_id.to_string(),
                "status": "Deployed",
                "version": manifest.version,
                "app_id": manifest.app_id,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }
    Ok(())
}
