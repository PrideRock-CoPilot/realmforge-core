//! CoreStore implementation - facade methods delegating to domain modules.

use crate::{
    audit, boards, build_watch, bundles, catalog, commands, error::StoreError, grants, knowledge,
    live_watch, login, rollbacks, sessions, snapshots, work_packets, work_path, ChainBounds,
    CoreStore,
};
use audit_log::AuditEvent;
use authority_domain::{
    ActorId, AgentWorkPacket, AuditEventId, BoardApproval, BoardPlan, BoardPlanId, BundleId,
    BundleManifest, BundleStatus, CommandId, CostRecord, DatasetInfo, GrantId, KnowledgeRecord,
    KnowledgeScope, PacketId, ProjectId, ProposalId, ProposalStatus, ReleaseCommand,
    RemediationProposal, RuntimeId, RuntimeInstance, SessionId, SkillGrant, SnapshotId,
    SourceType, ViolationRecord, WatchEvent, WatchProfile, WatchSignal,
};
use chrono::{DateTime, Utc};
use snapshot_ledger::SnapshotManifest;

impl CoreStore {
    // ── Audit events ──
    #[tracing::instrument(skip(self))]
    pub async fn append_audit_event(&self, event: &AuditEvent) -> Result<(), StoreError> {
        audit::append_audit_event(&self.pool, event).await
    }

    // ── Snapshot manifests ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_snapshot_manifest(
        &self,
        manifest: &SnapshotManifest,
    ) -> Result<(), StoreError> {
        snapshots::insert_snapshot_manifest(&self.pool, manifest).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_snapshot_manifest(
        &self,
        snapshot_id: &SnapshotId,
    ) -> Result<Option<SnapshotManifest>, StoreError> {
        snapshots::get_snapshot_manifest(&self.pool, snapshot_id).await
    }

    // ── Session operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_session(&self, row: &sessions::SessionRow) -> Result<(), StoreError> {
        sessions::insert_session(&self.pool, row).await
    }
    #[tracing::instrument(skip(self))]
    pub async fn get_session(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<sessions::SessionRow>, StoreError> {
        sessions::get_session(&self.pool, session_id).await
    }
    #[tracing::instrument(skip(self))]
    pub async fn update_session_state(
        &self,
        session_id: &SessionId,
        new_state: &str,
    ) -> Result<(), StoreError> {
        sessions::update_session_state(&self.pool, session_id, new_state).await
    }
    #[tracing::instrument(skip(self))]
    pub async fn update_session_expiry(
        &self,
        session_id: &SessionId,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), StoreError> {
        sessions::update_session_expiry(&self.pool, session_id, expires_at).await
    }

    // ── Command operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_command(&self, row: &commands::CommandRow) -> Result<(), StoreError> {
        commands::insert_command(&self.pool, row).await
    }
    #[tracing::instrument(skip(self))]
    pub async fn get_command(
        &self,
        command_id: &CommandId,
    ) -> Result<Option<commands::CommandRow>, StoreError> {
        commands::get_command(&self.pool, command_id).await
    }
    #[tracing::instrument(skip(self))]
    pub async fn update_command_status(
        &self,
        command_id: &CommandId,
        new_status: &str,
    ) -> Result<(), StoreError> {
        commands::update_command_status(&self.pool, command_id, new_status).await
    }
    #[tracing::instrument(skip(self))]
    pub async fn list_commands(
        &self,
        project_id: &ProjectId,
        status_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<commands::CommandRow>, StoreError> {
        commands::list_commands(&self.pool, project_id, status_filter, limit, offset).await
    }

    // ── Session listing ──
    #[tracing::instrument(skip(self))]
    pub async fn list_sessions(
        &self,
        tenant_id: &authority_domain::TenantId,
        project_id: &ProjectId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<sessions::SessionRow>, StoreError> {
        sessions::list_sessions(&self.pool, tenant_id, project_id, limit, offset).await
    }

    // ── Audit query operations ──
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self))]
    pub async fn query_audit_events(
        &self,
        project_id: &ProjectId,
        event_type: Option<&str>,
        actor_id: Option<&ActorId>,
        entity_type: Option<&str>,
        from_time: Option<DateTime<Utc>>,
        to_time: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<AuditEvent>, u64), StoreError> {
        audit::query_audit_events(
            &self.pool,
            project_id,
            event_type,
            actor_id,
            entity_type,
            from_time,
            to_time,
            limit,
            offset,
        )
        .await
    }
    #[tracing::instrument(skip(self))]
    pub async fn get_latest_event_hash(
        &self,
        project_id: &ProjectId,
    ) -> Result<Option<String>, StoreError> {
        audit::get_latest_event_hash(&self.pool, project_id).await
    }
    #[tracing::instrument(skip(self))]
    pub async fn get_all_audit_events(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<AuditEvent>, StoreError> {
        audit::get_all_events(&self.pool, project_id).await
    }
    #[tracing::instrument(skip(self))]
    pub async fn get_chain_bounds(
        &self,
        project_id: &ProjectId,
    ) -> Result<ChainBounds, StoreError> {
        audit::get_chain_bounds(&self.pool, project_id).await
    }

    // ── Snapshot list operations ──
    #[tracing::instrument(skip(self))]
    pub async fn list_snapshot_manifests(
        &self,
        project_id: &ProjectId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SnapshotManifest>, StoreError> {
        snapshots::list_snapshot_manifests(&self.pool, project_id, limit, offset).await
    }

    // ── Rollback preview operations ──
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self))]
    pub async fn insert_rollback_preview(
        &self,
        id: &str,
        tenant_id: &authority_domain::TenantId,
        project_id: &ProjectId,
        from_snapshot_id: &SnapshotId,
        to_snapshot_id: &SnapshotId,
        status: &str,
        preview_json: &serde_json::Value,
    ) -> Result<(), StoreError> {
        rollbacks::insert_rollback_preview(
            &self.pool,
            id,
            tenant_id,
            project_id,
            from_snapshot_id,
            to_snapshot_id,
            status,
            preview_json,
        )
        .await
    }
    #[tracing::instrument(skip(self))]
    pub async fn get_rollback_preview(
        &self,
        preview_id: &str,
    ) -> Result<Option<rollbacks::RollbackPreviewRow>, StoreError> {
        rollbacks::get_rollback_preview(&self.pool, preview_id).await
    }

    // ── Catalog operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_catalog_entry(
        &self,
        entry: &authority_domain::CatalogEntry,
    ) -> Result<(), StoreError> {
        catalog::insert_catalog_entry(&self.pool, entry).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_catalog_entry(
        &self,
        id: &authority_domain::CatalogId,
    ) -> Result<Option<authority_domain::CatalogEntry>, StoreError> {
        catalog::get_catalog_entry(&self.pool, id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_catalog_entries(
        &self,
        scope: &authority_domain::CatalogScope,
    ) -> Result<Vec<authority_domain::CatalogEntry>, StoreError> {
        catalog::list_catalog_entries(&self.pool, scope).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn copy_catalog_entry(
        &self,
        source_id: &authority_domain::CatalogId,
        new_id: &authority_domain::CatalogId,
        new_name: &str,
        new_parent_id: Option<&authority_domain::CatalogId>,
        new_scope: &authority_domain::CatalogScope,
        copied_by: &ActorId,
    ) -> Result<authority_domain::CatalogEntry, StoreError> {
        catalog::copy_catalog_entry(
            &self.pool,
            source_id,
            new_id,
            new_name,
            new_parent_id,
            new_scope,
            copied_by,
        )
        .await
    }

    // ── Work Path operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_work_path_graph(
        &self,
        graph: &authority_domain::WorkPathGraph,
    ) -> Result<(), StoreError> {
        work_path::insert_work_path_graph(&self.pool, graph).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_work_path_graph(
        &self,
        id: &authority_domain::WorkPathId,
    ) -> Result<Option<authority_domain::WorkPathGraph>, StoreError> {
        work_path::get_work_path_graph(&self.pool, id).await
    }

    // ── Grant operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_grant(&self, grant: &SkillGrant) -> Result<(), StoreError> {
        grants::insert_grant(&self.pool, grant).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_grant(
        &self,
        grant_id: &GrantId,
    ) -> Result<Option<SkillGrant>, StoreError> {
        grants::get_grant(&self.pool, grant_id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_grants_for_actor(
        &self,
        actor_id: &ActorId,
    ) -> Result<Vec<SkillGrant>, StoreError> {
        grants::list_grants_for_actor(&self.pool, actor_id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn revoke_grant(&self, grant_id: &GrantId) -> Result<(), StoreError> {
        grants::revoke_grant(&self.pool, grant_id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_grant_state(
        &self,
        grant_id: &GrantId,
        state: &authority_domain::GrantState,
    ) -> Result<(), StoreError> {
        grants::update_grant_state(&self.pool, grant_id, state).await
    }

    // ── Work Packet operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_work_packet(
        &self,
        packet: &AgentWorkPacket,
    ) -> Result<(), StoreError> {
        work_packets::insert_work_packet(&self.pool, packet).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_work_packet(
        &self,
        packet_id: &PacketId,
    ) -> Result<Option<AgentWorkPacket>, StoreError> {
        work_packets::get_work_packet(&self.pool, packet_id).await
    }

    // ── Knowledge metadata operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_knowledge_metadata(
        &self,
        record: &KnowledgeRecord,
    ) -> Result<(), StoreError> {
        knowledge::insert_knowledge_metadata(&self.pool, record).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn query_knowledge_metadata(
        &self,
        scopes: &[KnowledgeScope],
        source_type: Option<&SourceType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<KnowledgeRecord>, StoreError> {
        knowledge::query_knowledge_metadata(&self.pool, scopes, source_type, limit, offset).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_dataset_info(
        &self,
        dataset_id: &str,
    ) -> Result<Option<DatasetInfo>, StoreError> {
        knowledge::get_dataset_info(&self.pool, dataset_id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn reconcile_datasets(&self) -> Result<Vec<String>, StoreError> {
        knowledge::reconcile_datasets(&self.pool).await
    }

    // ── Board operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_plan(&self, plan: &BoardPlan) -> Result<(), StoreError> {
        boards::insert_plan(&self.pool, plan).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_plans(
        &self,
        status_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BoardPlan>, StoreError> {
        boards::list_plans(&self.pool, status_filter, limit, offset).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_plan_status(
        &self,
        plan_id: &BoardPlanId,
        status: &str,
    ) -> Result<(), StoreError> {
        boards::update_plan_status(&self.pool, plan_id, status).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn submit_approval(
        &self,
        approval: &BoardApproval,
    ) -> Result<(), StoreError> {
        boards::submit_approval(&self.pool, approval).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn insert_release_command(
        &self,
        cmd: &ReleaseCommand,
    ) -> Result<(), StoreError> {
        boards::insert_release_command(&self.pool, cmd).await
    }

    // ── Build Watch operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_watch_event(&self, event: &WatchEvent) -> Result<(), StoreError> {
        build_watch::insert_watch_event(&self.pool, event).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn query_watch_events(
        &self,
        scope: Option<&str>,
        event_type: Option<&str>,
        severity: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WatchEvent>, StoreError> {
        build_watch::query_watch_events(&self.pool, scope, event_type, severity, limit, offset).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn count_watch_events(&self) -> Result<i64, StoreError> {
        build_watch::count_watch_events(&self.pool).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn insert_violation(
        &self,
        violation: &ViolationRecord,
    ) -> Result<(), StoreError> {
        build_watch::insert_violation(&self.pool, violation).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn insert_cost_record(&self, record: &CostRecord) -> Result<(), StoreError> {
        build_watch::insert_cost_record(&self.pool, record).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_cost_summary(
        &self,
    ) -> Result<Vec<build_watch::CostSummaryRow>, StoreError> {
        build_watch::get_cost_summary(&self.pool).await
    }

    // ── Bundle operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_bundle_manifest(
        &self,
        manifest: &BundleManifest,
    ) -> Result<(), StoreError> {
        bundles::insert_bundle_manifest(&self.pool, manifest).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_bundle(
        &self,
        bundle_id: &BundleId,
    ) -> Result<Option<BundleManifest>, StoreError> {
        bundles::get_bundle(&self.pool, bundle_id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_bundles(
        &self,
        app_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BundleManifest>, StoreError> {
        bundles::list_bundles(&self.pool, app_id, limit, offset).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_bundle_status(
        &self,
        bundle_id: &BundleId,
        status: &BundleStatus,
    ) -> Result<(), StoreError> {
        bundles::update_bundle_status(&self.pool, bundle_id, status).await
    }

    // ── Runtime instance operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_runtime_instance(
        &self,
        instance: &RuntimeInstance,
    ) -> Result<(), StoreError> {
        bundles::insert_runtime_instance(&self.pool, instance).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_runtime_status(
        &self,
        runtime_id: &RuntimeId,
        status: &str,
    ) -> Result<(), StoreError> {
        bundles::update_runtime_status(&self.pool, runtime_id, status).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_runtime_health(
        &self,
        runtime_id: &RuntimeId,
    ) -> Result<Option<RuntimeInstance>, StoreError> {
        bundles::get_runtime_health(&self.pool, runtime_id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_runtime_instances(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RuntimeInstance>, StoreError> {
        bundles::list_runtime_instances(&self.pool, limit, offset).await
    }

    // ── Live Watch signal operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_watch_signal(&self, signal: &WatchSignal) -> Result<(), StoreError> {
        live_watch::insert_watch_signal(&self.pool, signal).await
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(self))]
    pub async fn query_watch_signals(
        &self,
        app_id: &str,
        signal_type: Option<&str>,
        severity: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WatchSignal>, StoreError> {
        live_watch::query_watch_signals(&self.pool, app_id, signal_type, severity, limit, offset)
            .await
    }

    // ── Live Watch remediation proposal operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_remediation_proposal(
        &self,
        proposal: &RemediationProposal,
    ) -> Result<(), StoreError> {
        live_watch::insert_remediation_proposal(&self.pool, proposal).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_remediation_proposals(
        &self,
        app_id: &str,
        status_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RemediationProposal>, StoreError> {
        live_watch::list_remediation_proposals(&self.pool, app_id, status_filter, limit, offset)
            .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn update_proposal_status(
        &self,
        proposal_id: &ProposalId,
        status: &ProposalStatus,
    ) -> Result<(), StoreError> {
        live_watch::update_proposal_status(&self.pool, proposal_id, status).await
    }

    // ── Live Watch profile operations ──
    #[tracing::instrument(skip(self))]
    pub async fn get_watch_profile(
        &self,
        app_id: &str,
    ) -> Result<Option<WatchProfile>, StoreError> {
        live_watch::get_watch_profile(&self.pool, app_id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn upsert_watch_profile(
        &self,
        profile: &WatchProfile,
    ) -> Result<(), StoreError> {
        live_watch::upsert_watch_profile(&self.pool, profile).await
    }

    // ── Login operations ──
    #[tracing::instrument(skip(self))]
    pub async fn insert_login_attempt(
        &self,
        tenant_id: &authority_domain::TenantId,
        record: &authority_domain::login::LoginAttemptRecord,
    ) -> Result<(), StoreError> {
        login::insert_login_attempt(&self.pool, tenant_id, record).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_login_attempts(
        &self,
        actor_id: &ActorId,
        tenant_id: &authority_domain::TenantId,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<authority_domain::login::LoginAttemptRecord>, StoreError> {
        login::get_login_attempts(&self.pool, actor_id, tenant_id, since).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn insert_login_block(
        &self,
        tenant_id: &authority_domain::TenantId,
        record: &authority_domain::login::LoginBlockRecord,
    ) -> Result<(), StoreError> {
        login::insert_login_block(&self.pool, tenant_id, record).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_login_block(
        &self,
        actor_id: &ActorId,
        tenant_id: &authority_domain::TenantId,
    ) -> Result<Option<authority_domain::login::LoginBlockRecord>, StoreError> {
        login::get_login_block(&self.pool, actor_id, tenant_id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_active_blocks(
        &self,
        tenant_id: &authority_domain::TenantId,
    ) -> Result<Vec<authority_domain::login::LoginBlockRecord>, StoreError> {
        login::list_active_blocks(&self.pool, tenant_id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn insert_login_policy(
        &self,
        tenant_id: &authority_domain::TenantId,
        config: &authority_domain::login::LoginPolicyConfig,
    ) -> Result<(), StoreError> {
        login::insert_login_policy(&self.pool, tenant_id, config).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_login_policy(
        &self,
        tenant_id: &authority_domain::TenantId,
    ) -> Result<Option<authority_domain::login::LoginPolicyConfig>, StoreError> {
        login::get_login_policy(&self.pool, tenant_id).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn upsert_credential(
        &self,
        tenant_id: &authority_domain::TenantId,
        actor_id: &ActorId,
        credential_hash: &str,
    ) -> Result<(), StoreError> {
        login::upsert_credential(&self.pool, tenant_id, actor_id, credential_hash).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_credential(
        &self,
        actor_id: &ActorId,
        tenant_id: &authority_domain::TenantId,
    ) -> Result<Option<String>, StoreError> {
        login::get_credential(&self.pool, actor_id, tenant_id).await
    }
}
