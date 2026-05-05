use control_service::ServiceContext;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::instrument;

use crate::error::McpError;

/// Args for core_get_watch_signals
#[derive(Clone, Debug, Deserialize)]
pub struct GetWatchSignalsArgs {
    pub app_id: String,
    pub signal_type: Option<String>,
    pub severity: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Args for core_propose_remediation
#[derive(Clone, Debug, Deserialize)]
pub struct ProposeRemediationArgs {
    pub app_id: String,
}

/// Args for core_list_proposals
#[derive(Clone, Debug, Deserialize)]
pub struct ListProposalsArgs {
    pub app_id: String,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Args for core_approve_proposal
#[derive(Clone, Debug, Deserialize)]
pub struct ApproveProposalArgs {
    pub proposal_id: String,
}

/// Get recent watch signals for an app.
#[instrument(skip(ctx), fields(app_id = %args.app_id))]
pub async fn core_get_watch_signals(
    args: GetWatchSignalsArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let signals = ctx
        .live_watch
        .get_signals(
            &args.app_id,
            args.signal_type.as_deref(),
            args.severity.as_deref(),
            args.limit.unwrap_or(50),
            args.offset.unwrap_or(0),
        )
        .await?;
    Ok(json!({ "signals": signals, "count": signals.len() }))
}

/// Propose remediation for an app based on recent anomaly signals.
#[instrument(skip(ctx), fields(app_id = %args.app_id))]
pub async fn core_propose_remediation(
    args: ProposeRemediationArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let proposal = ctx.live_watch.propose_remediation(&args.app_id).await?;
    Ok(json!(proposal))
}

/// List remediation proposals for an app, optionally filtered by status.
#[instrument(skip(ctx), fields(app_id = %args.app_id))]
pub async fn core_list_proposals(
    args: ListProposalsArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    let status_parsed = args.status.as_deref().and_then(|s| match s {
        "proposed" => Some(authority_domain::ProposalStatus::Proposed),
        "approved" => Some(authority_domain::ProposalStatus::Approved),
        "rejected" => Some(authority_domain::ProposalStatus::Rejected),
        "executed" => Some(authority_domain::ProposalStatus::Executed),
        _ => None,
    });
    let proposals = ctx
        .live_watch
        .list_proposals(
            &args.app_id,
            status_parsed,
            args.limit.unwrap_or(50),
            args.offset.unwrap_or(0),
        )
        .await?;
    Ok(json!({ "proposals": proposals, "count": proposals.len() }))
}

/// Approve a remediation proposal, transitioning it to Approved status.
#[instrument(skip(ctx), fields(proposal_id = %args.proposal_id))]
pub async fn core_approve_proposal(
    args: ApproveProposalArgs,
    ctx: &ServiceContext,
) -> Result<Value, McpError> {
    use authority_domain::ProposalId;
    let pid = ProposalId::new(args.proposal_id)
        .map_err(|e| McpError::InvalidArgs(format!("invalid proposal_id: {e}")))?;
    let proposal = ctx.live_watch.approve_proposal(&pid).await?;
    Ok(json!(proposal))
}
