use authority_domain::{ActorId, CommandId, CommandStatus, ProjectId, TenantId};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{types::Json, PgPool};
use tracing::instrument;

use crate::StoreError;

/// Row representation of a bounded command in the database.
#[derive(Debug)]
pub struct CommandRow {
    pub id: CommandId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub actor_id: ActorId,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub payload: Value,
    pub status: CommandStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct CommandRaw {
    id: String,
    tenant_id: String,
    project_id: String,
    actor_id: String,
    action: String,
    target_type: String,
    target_id: String,
    payload: Json<Value>,
    status: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<CommandRaw> for CommandRow {
    type Error = StoreError;

    fn try_from(raw: CommandRaw) -> Result<Self, Self::Error> {
        let status =
            CommandStatus::try_from(raw.status.as_str()).map_err(StoreError::InvalidData)?;
        Ok(CommandRow {
            id: CommandId::new(raw.id).map_err(|e| StoreError::InvalidData(e.to_string()))?,
            tenant_id: TenantId::new(raw.tenant_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            project_id: ProjectId::new(raw.project_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            actor_id: ActorId::new(raw.actor_id)
                .map_err(|e| StoreError::InvalidData(e.to_string()))?,
            action: raw.action,
            target_type: raw.target_type,
            target_id: raw.target_id,
            payload: raw.payload.0,
            status,
            created_at: raw.created_at,
        })
    }
}

/// Insert a new command record.
#[instrument(skip(pool), fields(command_id = %row.id, action = %row.action))]
pub async fn insert_command(pool: &PgPool, row: &CommandRow) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO bounded_commands \
         (id, tenant_id, project_id, actor_id, action, target_type, target_id, payload, status, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(row.id.as_str())
    .bind(row.tenant_id.as_str())
    .bind(row.project_id.as_str())
    .bind(row.actor_id.as_str())
    .bind(&row.action)
    .bind(&row.target_type)
    .bind(&row.target_id)
    .bind(Json(&row.payload))
    .bind(row.status.as_db_str())
    .bind(row.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a command by ID.
#[instrument(skip(pool), fields(command_id = %command_id))]
pub async fn get_command(
    pool: &PgPool,
    command_id: &CommandId,
) -> Result<Option<CommandRow>, StoreError> {
    let raw = sqlx::query_as::<_, CommandRaw>(
        "SELECT id, tenant_id, project_id, actor_id, action, target_type, target_id, \
         payload, status, created_at FROM bounded_commands WHERE id = $1",
    )
    .bind(command_id.as_str())
    .fetch_optional(pool)
    .await?;

    raw.map(CommandRow::try_from).transpose()
}

/// List commands for a project with optional status filter and pagination.
#[instrument(skip(pool), fields(project_id = %project_id, status_filter = ?status_filter))]
pub async fn list_commands(
    pool: &PgPool,
    project_id: &ProjectId,
    status_filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<CommandRow>, StoreError> {
    let rows = match status_filter {
        Some(status) => {
            sqlx::query_as::<_, CommandRaw>(
                "SELECT id, tenant_id, project_id, actor_id, action, target_type, target_id, \
                 payload, status, created_at FROM bounded_commands \
                 WHERE project_id = $1 AND status = $2 \
                 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
            )
            .bind(project_id.as_str())
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, CommandRaw>(
                "SELECT id, tenant_id, project_id, actor_id, action, target_type, target_id, \
                 payload, status, created_at FROM bounded_commands \
                 WHERE project_id = $1 \
                 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(project_id.as_str())
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
    };

    rows.into_iter().map(CommandRow::try_from).collect()
}

/// Update a command's status.
#[instrument(skip(pool), fields(command_id = %command_id, new_status = %new_status))]
pub async fn update_command_status(
    pool: &PgPool,
    command_id: &CommandId,
    new_status: &str,
) -> Result<(), StoreError> {
    sqlx::query("UPDATE bounded_commands SET status = $1 WHERE id = $2")
        .bind(new_status)
        .bind(command_id.as_str())
        .execute(pool)
        .await?;
    Ok(())
}
