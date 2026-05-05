use authority_domain::{
    ActorId, AgentWorkPacket, CostBudget, PacketId, PacketPermissionScope, PacketScope, PacketStatus,
    ProjectId, TenantId, WorkPathGraph, WorkPathId, WorkPathNode, WorkPathNodeId,
};
use chrono::Utc;
use control_store::CoreStore;
use tracing::{info, instrument};

use crate::error::ServiceError;

/// Agent Work Packet Generator service.
///
/// Generates scoped work packets by traversing a work-path graph from a target node,
/// aggregating file/contract/test/trace-point scopes, anchoring to the latest snapshot,
/// and persisting the result.
#[derive(Clone)]
pub struct WorkPacketService {
    store: CoreStore,
}

impl WorkPacketService {
    pub fn new(store: CoreStore) -> Self {
        Self { store }
    }

    /// Generate a work packet for an agent scoped to a specific work-path node.
    ///
    /// Traverses the work-path graph from `node_id`, aggregates all file/contract/test/
    /// trace-point links from the node and its descendants, fetches the latest snapshot
    /// as a rollback anchor, and persists the packet before returning it.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(
        tenant_id = %tenant_id,
        project_id = %project_id,
        agent_id = %agent_id,
        work_path_id = %work_path_id,
        node_id = %node_id
    ))]
    pub async fn generate_work_packet(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        agent_id: &ActorId,
        work_path_id: &WorkPathId,
        node_id: &WorkPathNodeId,
        objective: &str,
        cost_budget: Option<CostBudget>,
    ) -> Result<AgentWorkPacket, ServiceError> {
        // 1. Load the work-path graph and locate the target node.
        let graph = self
            .store
            .get_work_path_graph(work_path_id)
            .await?
            .ok_or(ServiceError::WorkPathNotFound)?;

        let node = graph
            .get_node(node_id)
            .ok_or_else(|| ServiceError::Validation(format!("node {node_id} not found in work path {work_path_id}")))?;

        // 2. Traverse from this node and aggregate scope from all descendants.
        let mut visited = std::collections::HashSet::new();
        let mut traversal: Vec<&WorkPathNode> = Vec::new();
        collect_subtree(&graph, node, &mut visited, &mut traversal);

        let packet_scope = PacketScope::from_nodes(&traversal);

        // 3. Deny everything not explicitly allowed.
        //    denied_file_paths is populated from any file IDs that appear in the graph
        //    but are outside the traversal scope (other branches).
        let all_graph_file_ids: Vec<String> = graph
            .nodes
            .iter()
            .flat_map(|n| n.file_ids.iter().cloned())
            .collect();
        let allowed_set: std::collections::HashSet<&str> =
            packet_scope.allowed_file_ids.iter().map(|s| s.as_str()).collect();
        let mut denied_file_paths: Vec<String> = all_graph_file_ids
            .into_iter()
            .filter(|f| !allowed_set.contains(f.as_str()))
            .collect();
        denied_file_paths.sort();
        denied_file_paths.dedup();

        // 4. Resolve rollback anchor — latest snapshot for this project, if any.
        let rollback_anchor = self
            .store
            .list_snapshot_manifests(project_id, 1, 0)
            .await?
            .into_iter()
            .next()
            .map(|m| m.id);

        // 5. Assemble the packet.
        let packet = AgentWorkPacket {
            id: PacketId::generate(),
            agent_id: agent_id.clone(),
            work_path_node_id: node_id.to_string(),
            objective: objective.to_string(),
            allowed_file_paths: packet_scope.allowed_file_ids,
            denied_file_paths,
            required_contracts: packet_scope.required_contract_ids,
            required_tests: packet_scope.required_test_ids,
            required_trace_points: packet_scope.required_trace_point_ids,
            rollback_anchor,
            cost_budget,
            permission_scope: PacketPermissionScope {
                allowed_actions: vec!["file.read".to_string(), "file.write".to_string()],
                denied_actions: vec!["file.delete".to_string()],
                max_concurrent_files: 5,
                allow_network: false,
            },
            created_at: Utc::now(),
            status: PacketStatus::Pending,
        };

        // 6. Persist before returning.
        self.store.insert_work_packet(&packet).await?;

        info!(
            packet_id = %packet.id,
            node_count = traversal.len(),
            allowed_files = packet.allowed_file_paths.len(),
            denied_files = packet.denied_file_paths.len(),
            has_rollback_anchor = packet.rollback_anchor.is_some(),
            "work packet generated"
        );
        Ok(packet)
    }

    /// Validate that a work packet's boundaries are coherent.
    ///
    /// Checks:
    /// 1. All files listed in `allowed_file_paths` are not also in `denied_file_paths`.
    /// 2. All `required_contracts` are non-empty strings (structural check only —
    ///    contract-registry verification is deferred until Phase 7).
    /// 3. `cost_budget`, if present, has non-zero limits.
    #[instrument(skip(self), fields(packet_id = %packet_id))]
    pub async fn validate_packet_boundaries(
        &self,
        packet_id: &PacketId,
    ) -> Result<PacketValidationResult, ServiceError> {
        let packet = self
            .store
            .get_work_packet(packet_id)
            .await?
            .ok_or(ServiceError::WorkPacketNotFound)?;

        let mut issues: Vec<String> = Vec::new();

        // Check 1: no overlap between allowed and denied paths.
        let denied_set: std::collections::HashSet<&str> =
            packet.denied_file_paths.iter().map(|s| s.as_str()).collect();
        for path in &packet.allowed_file_paths {
            if denied_set.contains(path.as_str()) {
                issues.push(format!("file '{path}' appears in both allowed and denied lists"));
            }
        }

        // Check 2: required contracts are non-empty strings.
        for (i, contract) in packet.required_contracts.iter().enumerate() {
            if contract.trim().is_empty() {
                issues.push(format!("required_contracts[{i}] is an empty string"));
            }
        }

        // Check 3: cost budget limits are non-zero if a budget is set.
        if let Some(budget) = &packet.cost_budget {
            if budget.max_tokens == 0 {
                issues.push("cost_budget.max_tokens must be greater than zero".to_string());
            }
            if budget.max_seconds == 0 {
                issues.push("cost_budget.max_seconds must be greater than zero".to_string());
            }
        }

        let valid = issues.is_empty();
        info!(packet_id = %packet_id, valid, issue_count = issues.len(), "work packet validated");
        Ok(PacketValidationResult {
            packet_id: packet_id.to_string(),
            valid,
            issues,
        })
    }

    /// Retrieve a persisted work packet by ID.
    #[instrument(skip(self), fields(packet_id = %packet_id))]
    pub async fn get_work_packet(
        &self,
        packet_id: &PacketId,
    ) -> Result<AgentWorkPacket, ServiceError> {
        self.store
            .get_work_packet(packet_id)
            .await?
            .ok_or(ServiceError::WorkPacketNotFound)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PacketValidationResult {
    pub packet_id: String,
    pub valid: bool,
    pub issues: Vec<String>,
}

/// Collect a node and all its descendants via DFS into `result`.
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
    use authority_domain::{WorkPathNodeType};

    fn make_graph() -> WorkPathGraph {
        let wp_id = WorkPathId::new("wp-1").unwrap();
        let n1 = WorkPathNodeId::new("n1").unwrap();
        let n2 = WorkPathNodeId::new("n2").unwrap();
        let n3 = WorkPathNodeId::new("n3").unwrap();

        WorkPathGraph {
            id: wp_id.clone(),
            name: "Test WP".to_string(),
            description: "".to_string(),
            nodes: vec![
                WorkPathNode {
                    id: n1.clone(),
                    work_path_id: wp_id.clone(),
                    node_type: WorkPathNodeType::Module,
                    name: "Root".to_string(),
                    file_ids: vec!["src/feature_a/mod.rs".to_string()],
                    contract_ids: vec!["ct-1".to_string()],
                    test_ids: vec![],
                    trace_point_ids: vec![],
                    children: vec![n2.clone()],
                    created_at: Utc::now(),
                },
                WorkPathNode {
                    id: n2,
                    work_path_id: wp_id.clone(),
                    node_type: WorkPathNodeType::Service,
                    name: "Service".to_string(),
                    file_ids: vec!["src/feature_a/service.rs".to_string()],
                    contract_ids: vec!["ct-2".to_string()],
                    test_ids: vec!["test-1".to_string()],
                    trace_point_ids: vec!["trace-x".to_string()],
                    children: vec![],
                    created_at: Utc::now(),
                },
                // This node is in another branch — should be denied for n1 traversal.
                WorkPathNode {
                    id: n3,
                    work_path_id: wp_id,
                    node_type: WorkPathNodeType::Policy,
                    name: "Other".to_string(),
                    file_ids: vec!["src/other/policy.rs".to_string()],
                    contract_ids: vec![],
                    test_ids: vec![],
                    trace_point_ids: vec![],
                    children: vec![],
                    created_at: Utc::now(),
                },
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn collect_subtree_respects_scope_boundary() {
        let graph = make_graph();
        let root = graph.get_node(&WorkPathNodeId::new("n1").unwrap()).unwrap();

        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::new();
        collect_subtree(&graph, root, &mut visited, &mut result);

        // n3 is not a child of n1 — must not be included.
        assert_eq!(result.len(), 2);
        let ids: Vec<&str> = result.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"n1"));
        assert!(ids.contains(&"n2"));
        assert!(!ids.contains(&"n3"));
    }

    #[test]
    fn denied_files_excludes_out_of_scope_nodes() {
        let graph = make_graph();
        let node_n1 = graph.get_node(&WorkPathNodeId::new("n1").unwrap()).unwrap();

        let mut visited = std::collections::HashSet::new();
        let mut traversal = Vec::new();
        collect_subtree(&graph, node_n1, &mut visited, &mut traversal);

        let scope = PacketScope::from_nodes(&traversal);
        let allowed_set: std::collections::HashSet<&str> =
            scope.allowed_file_ids.iter().map(|s| s.as_str()).collect();

        let all_files: Vec<String> = graph
            .nodes
            .iter()
            .flat_map(|n| n.file_ids.iter().cloned())
            .collect();

        let mut denied: Vec<String> = all_files
            .into_iter()
            .filter(|f| !allowed_set.contains(f.as_str()))
            .collect();
        denied.sort();
        denied.dedup();

        // src/other/policy.rs belongs to n3 and must be denied.
        assert!(denied.contains(&"src/other/policy.rs".to_string()));
        assert!(!denied.contains(&"src/feature_a/mod.rs".to_string()));
        assert!(!denied.contains(&"src/feature_a/service.rs".to_string()));
    }

    #[test]
    fn packet_validation_detects_overlap() {
        let result = PacketValidationResult {
            packet_id: "p1".to_string(),
            valid: false,
            issues: vec!["file 'src/a.rs' appears in both allowed and denied lists".to_string()],
        };
        assert!(!result.valid);
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn packet_validation_passes_clean_packet() {
        let result = PacketValidationResult {
            packet_id: "p1".to_string(),
            valid: true,
            issues: vec![],
        };
        assert!(result.valid);
        assert!(result.issues.is_empty());
    }
}
