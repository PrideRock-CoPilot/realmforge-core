use crate::{ActorId, PacketId, SnapshotId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A work packet defines the exact scope an AI agent is allowed to work in.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentWorkPacket {
    pub id: PacketId,
    pub agent_id: ActorId,
    pub work_path_node_id: String,
    pub objective: String,
    pub allowed_file_paths: Vec<String>,
    pub denied_file_paths: Vec<String>,
    pub required_contracts: Vec<String>,
    pub required_tests: Vec<String>,
    pub required_trace_points: Vec<String>,
    pub rollback_anchor: Option<SnapshotId>,
    pub cost_budget: Option<CostBudget>,
    pub permission_scope: PacketPermissionScope,
    pub created_at: DateTime<Utc>,
    pub status: PacketStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostBudget {
    pub max_tokens: u64,
    pub max_api_calls: u64,
    pub max_seconds: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PacketPermissionScope {
    pub allowed_actions: Vec<String>,
    pub denied_actions: Vec<String>,
    pub max_concurrent_files: u32,
    pub allow_network: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PacketStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Revoked,
}

impl AgentWorkPacket {
    /// Check if a file path is within the allowed scope.
    pub fn is_path_allowed(&self, path: &str) -> bool {
        if self.denied_file_paths.iter().any(|d| path.starts_with(d)) {
            return false;
        }
        self.allowed_file_paths.iter().any(|a| path.starts_with(a))
    }

    /// Check if an action is permitted by this packet.
    pub fn is_action_allowed(&self, action: &str) -> bool {
        if self
            .permission_scope
            .denied_actions
            .contains(&action.to_string())
        {
            return false;
        }
        self.permission_scope
            .allowed_actions
            .contains(&action.to_string())
    }

    /// Approximate cost of full work packet execution.
    pub fn estimated_cost_description(&self) -> String {
        if let Some(budget) = &self.cost_budget {
            format!(
                "{} tokens, {} API calls, {} seconds max",
                budget.max_tokens, budget.max_api_calls, budget.max_seconds
            )
        } else {
            "no budget defined".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_packet() -> AgentWorkPacket {
        AgentWorkPacket {
            id: PacketId::new("packet-1").unwrap(),
            agent_id: ActorId::new("agent-x").unwrap(),
            work_path_node_id: "node-42".to_string(),
            objective: "Implement file A".to_string(),
            allowed_file_paths: vec!["src/feature_a".to_string()],
            denied_file_paths: vec!["src/feature_a/secret.rs".to_string()],
            required_contracts: vec![],
            required_tests: vec![],
            required_trace_points: vec![],
            rollback_anchor: None,
            cost_budget: Some(CostBudget {
                max_tokens: 100_000,
                max_api_calls: 50,
                max_seconds: 600,
            }),
            permission_scope: PacketPermissionScope {
                allowed_actions: vec!["file.read".to_string(), "file.write".to_string()],
                denied_actions: vec!["file.delete".to_string()],
                max_concurrent_files: 3,
                allow_network: false,
            },
            created_at: Utc::now(),
            status: PacketStatus::Pending,
        }
    }

    #[test]
    fn allows_path_in_scope() {
        let packet = test_packet();
        assert!(packet.is_path_allowed("src/feature_a/main.rs"));
    }

    #[test]
    fn denies_path_not_in_scope() {
        let packet = test_packet();
        assert!(!packet.is_path_allowed("src/feature_b/main.rs"));
    }

    #[test]
    fn denies_explicitly_forbidden_path() {
        let packet = test_packet();
        assert!(!packet.is_path_allowed("src/feature_a/secret.rs"));
    }

    #[test]
    fn allows_permitted_action() {
        let packet = test_packet();
        assert!(packet.is_action_allowed("file.write"));
    }

    #[test]
    fn denies_forbidden_action() {
        let packet = test_packet();
        assert!(!packet.is_action_allowed("file.delete"));
    }
}
