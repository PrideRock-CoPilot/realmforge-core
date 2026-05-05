use authority_domain::{ActorId, AgentWorkPacket, PacketId, SnapshotId};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::StoreError;

#[derive(sqlx::FromRow)]
struct WorkPacketRaw {
    id: String,
    agent_id: String,
    work_path_node_id: String,
    objective: String,
    allowed_file_paths: serde_json::Value,
    denied_file_paths: serde_json::Value,
    required_contracts: serde_json::Value,
    required_tests: serde_json::Value,
    required_trace_points: serde_json::Value,
    rollback_anchor: Option<String>,
    cost_budget: serde_json::Value,
    permission_scope: serde_json::Value,
    created_at: DateTime<Utc>,
    status: String,
}

impl TryFrom<WorkPacketRaw> for AgentWorkPacket {
    type Error = StoreError;

    fn try_from(raw: WorkPacketRaw) -> Result<Self, Self::Error> {
        use authority_domain::PacketStatus;
        // DB stores status as bare TEXT (e.g. "pending"), wrap as JSON string to deserialize.
        let status: PacketStatus =
            serde_json::from_value(serde_json::Value::String(raw.status.clone())).map_err(|e| {
                StoreError::InvalidData(format!("invalid status '{}': {}", raw.status, e))
            })?;
        Ok(AgentWorkPacket {
            id: PacketId::new(raw.id).map_err(|e| StoreError::InvalidData(e.to_string()))?,
            agent_id: ActorId::new(raw.agent_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            work_path_node_id: raw.work_path_node_id,
            objective: raw.objective,
            allowed_file_paths: serde_json::from_value(raw.allowed_file_paths)?,
            denied_file_paths: serde_json::from_value(raw.denied_file_paths)?,
            required_contracts: serde_json::from_value(raw.required_contracts)?,
            required_tests: serde_json::from_value(raw.required_tests)?,
            required_trace_points: serde_json::from_value(raw.required_trace_points)?,
            rollback_anchor: raw
                .rollback_anchor
                .map(|s| SnapshotId::new(s).map_err(|e| StoreError::InvalidData(e.to_string())))
                .transpose()?,
            cost_budget: serde_json::from_value(raw.cost_budget)?,
            permission_scope: serde_json::from_value(raw.permission_scope)?,
            created_at: raw.created_at,
            status,
        })
    }
}

/// Helper: convert PacketStatus to its JSON string representation (bare, no JSON wrapping).
fn status_to_db_string(status: &authority_domain::PacketStatus) -> String {
    serde_json::to_value(status)
        .ok()
        .and_then(|v| match v {
            serde_json::Value::String(s) => Some(s),
            _ => None,
        })
        .unwrap_or_else(|| "pending".to_string())
}

pub async fn insert_work_packet(pool: &PgPool, packet: &AgentWorkPacket) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO work_packets (id, agent_id, work_path_node_id, objective, \
         allowed_file_paths, denied_file_paths, required_contracts, required_tests, \
         required_trace_points, rollback_anchor, cost_budget, permission_scope, \
         created_at, status) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
    )
    .bind(packet.id.as_str())
    .bind(packet.agent_id.as_str())
    .bind(&packet.work_path_node_id)
    .bind(&packet.objective)
    .bind(serde_json::to_value(&packet.allowed_file_paths)?)
    .bind(serde_json::to_value(&packet.denied_file_paths)?)
    .bind(serde_json::to_value(&packet.required_contracts)?)
    .bind(serde_json::to_value(&packet.required_tests)?)
    .bind(serde_json::to_value(&packet.required_trace_points)?)
    .bind(packet.rollback_anchor.as_ref().map(|s| s.as_str()))
    .bind(serde_json::to_value(&packet.cost_budget)?)
    .bind(serde_json::to_value(&packet.permission_scope)?)
    .bind(packet.created_at)
    .bind(status_to_db_string(&packet.status))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_work_packet(
    pool: &PgPool,
    packet_id: &PacketId,
) -> Result<Option<AgentWorkPacket>, StoreError> {
    let row: Option<WorkPacketRaw> = sqlx::query_as(
        "SELECT id, agent_id, work_path_node_id, objective, \
         allowed_file_paths, denied_file_paths, required_contracts, required_tests, \
         required_trace_points, rollback_anchor, cost_budget, permission_scope, \
         created_at, status \
         FROM work_packets WHERE id = $1",
    )
    .bind(packet_id.as_str())
    .fetch_optional(pool)
    .await?;

    row.map(AgentWorkPacket::try_from).transpose()
}

// ── CoreStore impl ───────────────────────────────────────────────────────────

use crate::CoreStore;

impl CoreStore {
    /// Insert a new work packet.
    #[tracing::instrument(skip(self))]
    pub async fn insert_work_packet(&self, packet: &AgentWorkPacket) -> Result<(), StoreError> {
        insert_work_packet(&self.pool, packet).await
    }

    /// Get a work packet by ID.
    #[tracing::instrument(skip(self))]
    pub async fn get_work_packet(
        &self,
        packet_id: &PacketId,
    ) -> Result<Option<AgentWorkPacket>, StoreError> {
        get_work_packet(&self.pool, packet_id).await
    }
}
