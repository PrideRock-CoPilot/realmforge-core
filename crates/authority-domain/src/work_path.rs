use crate::{WorkPathId, WorkPathNodeId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The type classification for a work path node.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkPathNodeType {
    Module,
    RuntimeContract,
    Policy,
    Service,
    WatchSignal,
    DataContract,
    Evidence,
}

impl std::fmt::Display for WorkPathNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Module => write!(f, "module"),
            Self::RuntimeContract => write!(f, "runtime_contract"),
            Self::Policy => write!(f, "policy"),
            Self::Service => write!(f, "service"),
            Self::WatchSignal => write!(f, "watch_signal"),
            Self::DataContract => write!(f, "data_contract"),
            Self::Evidence => write!(f, "evidence"),
        }
    }
}

/// A single node in a work path graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkPathNode {
    pub id: WorkPathNodeId,
    pub work_path_id: WorkPathId,
    pub node_type: WorkPathNodeType,
    pub name: String,
    pub file_ids: Vec<String>,
    pub contract_ids: Vec<String>,
    pub test_ids: Vec<String>,
    pub trace_point_ids: Vec<String>,
    pub children: Vec<WorkPathNodeId>,
    pub created_at: DateTime<Utc>,
}

/// An ordered DAG of work path nodes defining an executable planning graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkPathGraph {
    pub id: WorkPathId,
    pub name: String,
    pub description: String,
    pub nodes: Vec<WorkPathNode>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkPathGraph {
    /// Get a node by its ID.
    pub fn get_node(&self, node_id: &WorkPathNodeId) -> Option<&WorkPathNode> {
        self.nodes.iter().find(|n| n.id == *node_id)
    }

    /// Get the root nodes (nodes that are not referenced as children elsewhere).
    pub fn root_nodes(&self) -> Vec<&WorkPathNode> {
        let all_child_ids: std::collections::HashSet<&WorkPathNodeId> =
            self.nodes.iter().flat_map(|n| &n.children).collect();
        self.nodes
            .iter()
            .filter(|n| !all_child_ids.contains(&n.id))
            .collect()
    }

    /// Perform a topological traversal of the graph from root nodes.
    pub fn traverse(&self) -> Vec<&WorkPathNode> {
        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::new();

        for root in self.root_nodes() {
            self.traverse_from(root, &mut visited, &mut result);
        }

        result
    }

    fn traverse_from<'a>(
        &'a self,
        node: &'a WorkPathNode,
        visited: &mut std::collections::HashSet<WorkPathNodeId>,
        result: &mut Vec<&'a WorkPathNode>,
    ) {
        if !visited.insert(node.id.clone()) {
            return;
        }
        result.push(node);
        for child_id in &node.children {
            if let Some(child) = self.get_node(child_id) {
                self.traverse_from(child, visited, result);
            }
        }
    }
}

/// Packet scope derived from work path node traversal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PacketScope {
    pub allowed_file_ids: Vec<String>,
    pub required_contract_ids: Vec<String>,
    pub required_test_ids: Vec<String>,
    pub required_trace_point_ids: Vec<String>,
}

impl PacketScope {
    /// Aggregate scopes from a list of work path nodes into a single packet scope.
    pub fn from_nodes(nodes: &[&WorkPathNode]) -> Self {
        let mut allowed_file_ids: Vec<String> = Vec::new();
        let mut required_contract_ids: Vec<String> = Vec::new();
        let mut required_test_ids: Vec<String> = Vec::new();
        let mut required_trace_point_ids: Vec<String> = Vec::new();

        for node in nodes {
            allowed_file_ids.extend(node.file_ids.clone());
            required_contract_ids.extend(node.contract_ids.clone());
            required_test_ids.extend(node.test_ids.clone());
            required_trace_point_ids.extend(node.trace_point_ids.clone());
        }

        // Deduplicate
        allowed_file_ids.sort();
        allowed_file_ids.dedup();
        required_contract_ids.sort();
        required_contract_ids.dedup();
        required_test_ids.sort();
        required_test_ids.dedup();
        required_trace_point_ids.sort();
        required_trace_point_ids.dedup();

        Self {
            allowed_file_ids,
            required_contract_ids,
            required_test_ids,
            required_trace_point_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node(
        id: &str,
        work_path_id: &str,
        node_type: WorkPathNodeType,
        name: &str,
        children: Vec<WorkPathNodeId>,
    ) -> WorkPathNode {
        WorkPathNode {
            id: WorkPathNodeId::new(id).unwrap(),
            work_path_id: WorkPathId::new(work_path_id).unwrap(),
            node_type,
            name: name.to_string(),
            file_ids: Vec::new(),
            contract_ids: Vec::new(),
            test_ids: Vec::new(),
            trace_point_ids: Vec::new(),
            children,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn work_path_node_type_display() {
        assert_eq!(WorkPathNodeType::Module.to_string(), "module");
        assert_eq!(
            WorkPathNodeType::RuntimeContract.to_string(),
            "runtime_contract"
        );
        assert_eq!(WorkPathNodeType::Policy.to_string(), "policy");
        assert_eq!(WorkPathNodeType::Service.to_string(), "service");
        assert_eq!(WorkPathNodeType::WatchSignal.to_string(), "watch_signal");
        assert_eq!(WorkPathNodeType::DataContract.to_string(), "data_contract");
        assert_eq!(WorkPathNodeType::Evidence.to_string(), "evidence");
    }

    #[test]
    fn work_path_graph_traversal() {
        let wp_id = WorkPathId::new("wp-1").unwrap();
        let _n1 = WorkPathNodeId::new("n1").unwrap();
        let n2 = WorkPathNodeId::new("n2").unwrap();
        let n3 = WorkPathNodeId::new("n3").unwrap();

        let graph = WorkPathGraph {
            id: wp_id.clone(),
            name: "Test WP".to_string(),
            description: "A test work path".to_string(),
            nodes: vec![
                sample_node(
                    "n1",
                    "wp-1",
                    WorkPathNodeType::Module,
                    "Module A",
                    vec![n2.clone()],
                ),
                sample_node(
                    "n2",
                    "wp-1",
                    WorkPathNodeType::Policy,
                    "Policy B",
                    vec![n3.clone()],
                ),
                sample_node(
                    "n3",
                    "wp-1",
                    WorkPathNodeType::Evidence,
                    "Evidence C",
                    vec![],
                ),
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(graph.root_nodes().len(), 1);
        assert_eq!(graph.root_nodes()[0].name, "Module A");

        let traversal = graph.traverse();
        assert_eq!(traversal.len(), 3);
        assert_eq!(traversal[0].name, "Module A");
        assert_eq!(traversal[1].name, "Policy B");
        assert_eq!(traversal[2].name, "Evidence C");
    }

    #[test]
    fn packet_scope_from_nodes_deduplicates() {
        let wp_id = WorkPathId::new("wp-1").unwrap();

        let n1 = WorkPathNode {
            id: WorkPathNodeId::new("n1").unwrap(),
            work_path_id: wp_id.clone(),
            node_type: WorkPathNodeType::Module,
            name: "Module A".to_string(),
            file_ids: vec!["file-a.rs".to_string(), "file-b.rs".to_string()],
            contract_ids: vec!["contract-1".to_string()],
            test_ids: vec!["test-1".to_string()],
            trace_point_ids: vec![],
            children: vec![],
            created_at: Utc::now(),
        };

        let n2 = WorkPathNode {
            id: WorkPathNodeId::new("n2").unwrap(),
            work_path_id: wp_id,
            node_type: WorkPathNodeType::Policy,
            name: "Policy B".to_string(),
            file_ids: vec!["file-b.rs".to_string(), "file-c.rs".to_string()],
            contract_ids: vec!["contract-2".to_string()],
            test_ids: vec![],
            trace_point_ids: vec!["trace-abc".to_string()],
            children: vec![],
            created_at: Utc::now(),
        };

        let scope = PacketScope::from_nodes(&[&n1, &n2]);

        // file-b deduplicated
        assert_eq!(scope.allowed_file_ids.len(), 3);
        assert!(scope.allowed_file_ids.contains(&"file-a.rs".to_string()));
        assert!(scope.allowed_file_ids.contains(&"file-b.rs".to_string()));
        assert!(scope.allowed_file_ids.contains(&"file-c.rs".to_string()));

        assert_eq!(scope.required_contract_ids.len(), 2);
        assert_eq!(scope.required_test_ids.len(), 1);
        assert_eq!(scope.required_trace_point_ids.len(), 1);
    }

    #[test]
    fn work_path_graph_get_node() {
        let graph = WorkPathGraph {
            id: WorkPathId::new("wp-1").unwrap(),
            name: "Test".to_string(),
            description: "".to_string(),
            nodes: vec![sample_node(
                "n1",
                "wp-1",
                WorkPathNodeType::Service,
                "Svc",
                vec![],
            )],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let found = graph.get_node(&WorkPathNodeId::new("n1").unwrap());
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Svc");

        let missing = graph.get_node(&WorkPathNodeId::new("nope").unwrap());
        assert!(missing.is_none());
    }
}
