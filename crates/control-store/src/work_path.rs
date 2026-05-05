use authority_domain::{WorkPathGraph, WorkPathId, WorkPathNode, WorkPathNodeId};
use sqlx::PgPool;
use tracing::instrument;

use crate::StoreError;

/// Insert a work path graph and all its nodes into the database.
#[instrument(skip(pool))]
pub async fn insert_work_path_graph(pool: &PgPool, graph: &WorkPathGraph) -> Result<(), StoreError> {
    sqlx::query(
        r#"
        INSERT INTO work_path_graphs (id, name, description, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(graph.id.as_str())
    .bind(&graph.name)
    .bind(&graph.description)
    .bind(graph.created_at)
    .bind(graph.updated_at)
    .execute(pool)
    .await?;

    for node in &graph.nodes {
        insert_work_path_node(pool, node).await?;
    }

    Ok(())
}

/// Insert a single work path node.
#[instrument(skip(pool))]
pub async fn insert_work_path_node(pool: &PgPool, node: &WorkPathNode) -> Result<(), StoreError> {
    let children_ids: serde_json::Value = serde_json::to_value(&node.children)
        .map_err(|e| StoreError::invalid_data(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO work_path_nodes (id, work_path_id, node_type, name, file_ids, contract_ids, test_ids, trace_point_ids, children_ids, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(node.id.as_str())
    .bind(node.work_path_id.as_str())
    .bind(node.node_type.to_string())
    .bind(&node.name)
    .bind(serde_json::to_value(&node.file_ids).map_err(|e| StoreError::invalid_data(e.to_string()))?)
    .bind(serde_json::to_value(&node.contract_ids).map_err(|e| StoreError::invalid_data(e.to_string()))?)
    .bind(serde_json::to_value(&node.test_ids).map_err(|e| StoreError::invalid_data(e.to_string()))?)
    .bind(serde_json::to_value(&node.trace_point_ids).map_err(|e| StoreError::invalid_data(e.to_string()))?)
    .bind(children_ids)
    .bind(node.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get the full work path graph (with all nodes) by ID.
#[instrument(skip(pool))]
pub async fn get_work_path_graph(
    pool: &PgPool,
    id: &WorkPathId,
) -> Result<Option<WorkPathGraph>, StoreError> {
    let graph_row = sqlx::query_as::<_, WorkPathGraphRow>(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM work_path_graphs
        WHERE id = $1
        "#,
    )
    .bind(id.as_str())
    .fetch_optional(pool)
    .await?;

    let Some(g) = graph_row else {
        return Ok(None);
    };

    let node_rows = sqlx::query_as::<_, WorkPathNodeRow>(
        r#"
        SELECT id, work_path_id, node_type, name, file_ids, contract_ids, test_ids, trace_point_ids, children_ids, created_at
        FROM work_path_nodes
        WHERE work_path_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(id.as_str())
    .fetch_all(pool)
    .await?;

    let mut nodes = Vec::with_capacity(node_rows.len());
    for row in node_rows {
        nodes.push(row.into_node()?);
    }

    Ok(Some(WorkPathGraph {
        id: WorkPathId::new(g.id).map_err(|e| StoreError::invalid_data(e.to_string()))?,
        name: g.name,
        description: g.description,
        nodes,
        created_at: g.created_at,
        updated_at: g.updated_at,
    }))
}

// ── Row types ──

#[derive(Debug, sqlx::FromRow)]
struct WorkPathGraphRow {
    id: String,
    name: String,
    description: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct WorkPathNodeRow {
    id: String,
    work_path_id: String,
    node_type: String,
    name: String,
    file_ids: serde_json::Value,
    contract_ids: serde_json::Value,
    test_ids: serde_json::Value,
    trace_point_ids: serde_json::Value,
    children_ids: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl WorkPathNodeRow {
    fn into_node(self) -> Result<WorkPathNode, StoreError> {
        let node_type = match self.node_type.as_str() {
            "module" => authority_domain::WorkPathNodeType::Module,
            "runtime_contract" => authority_domain::WorkPathNodeType::RuntimeContract,
            "policy" => authority_domain::WorkPathNodeType::Policy,
            "service" => authority_domain::WorkPathNodeType::Service,
            "watch_signal" => authority_domain::WorkPathNodeType::WatchSignal,
            "data_contract" => authority_domain::WorkPathNodeType::DataContract,
            "evidence" => authority_domain::WorkPathNodeType::Evidence,
            other => return Err(StoreError::invalid_data(format!("invalid node_type: {other}"))),
        };

        fn parse_str_vec(val: serde_json::Value) -> Result<Vec<String>, StoreError> {
            serde_json::from_value(val).map_err(|e| StoreError::invalid_data(e.to_string()))
        }

        let children: Vec<authority_domain::WorkPathNodeId> = serde_json::from_value(self.children_ids)
            .map_err(|e| StoreError::invalid_data(e.to_string()))?;

        Ok(WorkPathNode {
            id: WorkPathNodeId::new(self.id)
                .map_err(|e| StoreError::invalid_data(e.to_string()))?,
            work_path_id: WorkPathId::new(self.work_path_id)
                .map_err(|e| StoreError::invalid_data(e.to_string()))?,
            node_type,
            name: self.name,
            file_ids: parse_str_vec(self.file_ids)?,
            contract_ids: parse_str_vec(self.contract_ids)?,
            test_ids: parse_str_vec(self.test_ids)?,
            trace_point_ids: parse_str_vec(self.trace_point_ids)?,
            children,
            created_at: self.created_at,
        })
    }
}
