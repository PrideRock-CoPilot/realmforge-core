use authority_domain::{
    ActorId, AgentWorkPacket, CostBudget, PacketId, PacketPermissionScope, PacketScope,
    PacketStatus, ProjectId, TenantId, WorkPathGraph, WorkPathId, WorkPathNode, WorkPathNodeId,
    WorkPathNodeType,
};
use chrono::Utc;
use control_store::CoreStore;
use tracing::{info, instrument};

use crate::error::ServiceError;

/// Work Path service for managing executable planning graphs.
#[derive(Clone)]
pub struct WorkPathService {
    store: CoreStore,
}

impl WorkPathService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Create a new work path graph.
    #[instrument(skip(self), fields(id = %id, name = %name))]
    pub async fn create_work_path(
        &self,
        id: &WorkPathId,
        name: &str,
        description: &str,
    ) -> Result<WorkPathGraph, ServiceError> {
        let now = Utc::now();
        let graph = WorkPathGraph {
            id: id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            nodes: vec![],
            created_at: now,
            updated_at: now,
        };
        self.store.insert_work_path_graph(&graph).await?;
        info!(work_path = %id, "work path created");
        Ok(graph)
    }

    /// Add a node to an existing work path graph.
    #[instrument(skip(self), fields(work_path_id = %work_path_id, node_id = %id))]
    #[allow(clippy::too_many_arguments)]
    pub async fn add_node(
        &self,
        work_path_id: &WorkPathId,
        id: &WorkPathNodeId,
        node_type: WorkPathNodeType,
        name: &str,
        file_ids: Vec<String>,
        contract_ids: Vec<String>,
        test_ids: Vec<String>,
        trace_point_ids: Vec<String>,
        children: Vec<WorkPathNodeId>,
    ) -> Result<WorkPathNode, ServiceError> {
        // Verify the graph exists
        let mut graph = self
            .store
            .get_work_path_graph(work_path_id)
            .await?
            .ok_or_else(|| {
                ServiceError::Validation(format!("work path {work_path_id} not found"))
            })?;

        let node = WorkPathNode {
            id: id.clone(),
            work_path_id: work_path_id.clone(),
            node_type,
            name: name.to_string(),
            file_ids,
            contract_ids,
            test_ids,
            trace_point_ids,
            children,
            created_at: Utc::now(),
        };

        graph.nodes.push(node.clone());
        graph.updated_at = Utc::now();

        // Re-insert the full graph (replaces all nodes)
        self.store.insert_work_path_graph(&graph).await?;

        info!(node_id = %id, "work path node added");
        Ok(node)
    }

    /// Get the full work path graph.
    #[instrument(skip(self), fields(id = %id))]
    pub async fn get_work_path_graph(
        &self,
        id: &WorkPathId,
    ) -> Result<WorkPathGraph, ServiceError> {
        self.store
            .get_work_path_graph(id)
            .await?
            .ok_or_else(|| ServiceError::Validation(format!("work path {id} not found")))
    }

    /// Traverse a work path graph and produce a scoped AgentWorkPacket for a specific node.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(work_path_id = %work_path_id, node_id = %node_id, agent_id = %agent_id))]
    pub async fn traverse_to_packet(
        &self,
        work_path_id: &WorkPathId,
        node_id: &WorkPathNodeId,
        agent_id: &ActorId,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        objective: &str,
        cost_budget: Option<CostBudget>,
    ) -> Result<AgentWorkPacket, ServiceError> {
        let graph = self.get_work_path_graph(work_path_id).await?;
        let node = graph.get_node(node_id).ok_or_else(|| {
            ServiceError::Validation(format!("node {node_id} not found in work path"))
        })?;

        // Traverse from this node to collect all descendant scopes
        let mut visited = std::collections::HashSet::new();
        let mut traversal = Vec::new();
        collect_subtree(&graph, node, &mut visited, &mut traversal);

        let packet_scope = PacketScope::from_nodes(&traversal);

        let packet = AgentWorkPacket {
            id: PacketId::generate(),
            agent_id: agent_id.clone(),
            work_path_node_id: node_id.to_string(),
            objective: objective.to_string(),
            allowed_file_paths: packet_scope.allowed_file_ids,
            denied_file_paths: vec![],
            required_contracts: packet_scope.required_contract_ids,
            required_tests: packet_scope.required_test_ids,
            required_trace_points: packet_scope.required_trace_point_ids,
            rollback_anchor: None,
            cost_budget,
            permission_scope: PacketPermissionScope {
                allowed_actions: vec!["file.read".to_string(), "file.write".to_string()],
                denied_actions: vec![],
                max_concurrent_files: 5,
                allow_network: false,
            },
            created_at: Utc::now(),
            status: PacketStatus::Pending,
        };

        info!(
            packet_id = %packet.id,
            node_count = traversal.len(),
            "work packet generated from work path traversal"
        );

        Ok(packet)
    }
}

/// Collect a node and all its descendants into a traversal list.
fn collect_subtree<'a>(
    graph: &'a WorkPathGraph,
    node: &'a WorkPathNode,
    visited: &mut std::collections::HashSet<WorkPathNodeId>,
    result: &mut Vec<&'a WorkPathNode>,
) {
    if !visited.insert(node.id.clone()) {
        return;
    }
    result.push(node);
    for child_id in &node.children {
        if let Some(child) = graph.get_node(child_id) {
            collect_subtree(graph, child, visited, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph() -> WorkPathGraph {
        let wp_id = WorkPathId::new("wp-test").unwrap();
        let n1 = WorkPathNodeId::new("n1").unwrap();
        let n2 = WorkPathNodeId::new("n2").unwrap();
        let n3 = WorkPathNodeId::new("n3").unwrap();

        WorkPathGraph {
            id: wp_id,
            name: "Test WP".to_string(),
            description: "".to_string(),
            nodes: vec![
                WorkPathNode {
                    id: n1.clone(),
                    work_path_id: WorkPathId::new("wp-test").unwrap(),
                    node_type: WorkPathNodeType::Module,
                    name: "Root".to_string(),
                    file_ids: vec!["file-a.rs".to_string()],
                    contract_ids: vec!["ct-1".to_string()],
                    test_ids: vec![],
                    trace_point_ids: vec![],
                    children: vec![n2.clone(), n3.clone()],
                    created_at: Utc::now(),
                },
                WorkPathNode {
                    id: n2,
                    work_path_id: WorkPathId::new("wp-test").unwrap(),
                    node_type: WorkPathNodeType::Policy,
                    name: "Policy".to_string(),
                    file_ids: vec!["file-b.rs".to_string()],
                    contract_ids: vec![],
                    test_ids: vec!["test-1".to_string()],
                    trace_point_ids: vec![],
                    children: vec![],
                    created_at: Utc::now(),
                },
                WorkPathNode {
                    id: n3,
                    work_path_id: WorkPathId::new("wp-test").unwrap(),
                    node_type: WorkPathNodeType::Service,
                    name: "Service".to_string(),
                    file_ids: vec!["file-c.rs".to_string()],
                    contract_ids: vec!["ct-2".to_string()],
                    test_ids: vec![],
                    trace_point_ids: vec!["trace-x".to_string()],
                    children: vec![],
                    created_at: Utc::now(),
                },
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn collect_subtree_includes_all_descendants() {
        let graph = make_graph();
        let root = graph.get_node(&WorkPathNodeId::new("n1").unwrap()).unwrap();

        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::new();
        collect_subtree(&graph, root, &mut visited, &mut result);

        assert_eq!(result.len(), 3);
        let names: Vec<&str> = result.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"Root"));
        assert!(names.contains(&"Policy"));
        assert!(names.contains(&"Service"));
    }

    #[test]
    fn packet_scope_from_traversal_aggregates_correctly() {
        let graph = make_graph();
        let root = graph.get_node(&WorkPathNodeId::new("n1").unwrap()).unwrap();

        let mut visited = std::collections::HashSet::new();
        let mut traversal = Vec::new();
        collect_subtree(&graph, root, &mut visited, &mut traversal);

        let scope = PacketScope::from_nodes(&traversal);
        assert!(scope.allowed_file_ids.contains(&"file-a.rs".to_string()));
        assert!(scope.allowed_file_ids.contains(&"file-b.rs".to_string()));
        assert!(scope.allowed_file_ids.contains(&"file-c.rs".to_string()));
        assert!(scope.required_contract_ids.contains(&"ct-1".to_string()));
        assert!(scope.required_contract_ids.contains(&"ct-2".to_string()));
        assert!(scope.required_test_ids.contains(&"test-1".to_string()));
        assert!(scope
            .required_trace_point_ids
            .contains(&"trace-x".to_string()));
    }
}
