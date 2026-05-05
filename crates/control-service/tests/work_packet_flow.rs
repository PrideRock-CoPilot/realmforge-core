// Integration tests for work packet flow: Graph setup → Generate → Validate → Scope check.
//
// Validates the WorkPacketService workflow against a live PostgreSQL database.
// Tests gracefully skip when no database is available.

mod common;

use authority_domain::{
    CostBudget, WorkPathGraph, WorkPathId, WorkPathNode, WorkPathNodeId, WorkPathNodeType,
};
use chrono::Utc;
use control_service::{error::ServiceError, WorkPacketService};

/// Build a small test graph with 3 nodes: n1 → n2 (one branch), n3 (separate branch).
fn make_test_graph(wp_id: &WorkPathId) -> WorkPathGraph {
    let n1 = WorkPathNodeId::new("n1").unwrap();
    let n2 = WorkPathNodeId::new("n2").unwrap();
    let n3 = WorkPathNodeId::new("n3").unwrap();

    WorkPathGraph {
        id: wp_id.clone(),
        name: "Test Work Path".to_string(),
        description: "Integration test graph".to_string(),
        nodes: vec![
            WorkPathNode {
                id: n1.clone(),
                work_path_id: wp_id.clone(),
                node_type: WorkPathNodeType::Module,
                name: "Feature Root".to_string(),
                file_ids: vec![
                    "src/feature/mod.rs".to_string(),
                    "src/feature/impl.rs".to_string(),
                ],
                contract_ids: vec!["ct-feature".to_string()],
                test_ids: vec!["test-feature".to_string()],
                trace_point_ids: vec![],
                children: vec![n2.clone()],
                created_at: Utc::now(),
            },
            WorkPathNode {
                id: n2.clone(),
                work_path_id: wp_id.clone(),
                node_type: WorkPathNodeType::Service,
                name: "Query Service".to_string(),
                file_ids: vec!["src/queries/service.rs".to_string()],
                contract_ids: vec!["ct-queries".to_string()],
                test_ids: vec!["test-queries".to_string()],
                trace_point_ids: vec!["trace-1".to_string()],
                children: vec![],
                created_at: Utc::now(),
            },
            // n3 is on a separate branch — files here must be denied when scoping to n1.
            WorkPathNode {
                id: n3.clone(),
                work_path_id: wp_id.clone(),
                node_type: WorkPathNodeType::Policy,
                name: "Other Branch".to_string(),
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

#[tokio::test]
async fn test_work_packet_generate() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let wp_id = WorkPathId::generate();
    let graph = make_test_graph(&wp_id);
    store.insert_work_path_graph(&graph).await.unwrap();

    let svc = WorkPacketService::new(store);
    let node_id = WorkPathNodeId::new("n1").unwrap();

    let packet = svc
        .generate_work_packet(
            &common::test_tenant(),
            &common::test_project(),
            &common::test_actor(),
            &wp_id,
            &node_id,
            "Implement feature X",
            None,
        )
        .await
        .unwrap();

    assert!(!packet.id.as_str().is_empty());
    assert_eq!(packet.objective, "Implement feature X");
    assert_eq!(packet.agent_id, common::test_actor());

    // n1 + n2 (child) file IDs are allowed
    assert!(packet.allowed_file_paths.contains(&"src/feature/mod.rs".to_string()));
    assert!(packet.allowed_file_paths.contains(&"src/queries/service.rs".to_string()));

    // n3 is out-of-scope: its files must be denied
    assert!(packet.denied_file_paths.contains(&"src/other/policy.rs".to_string()));
}

#[tokio::test]
async fn test_work_packet_generate_with_budget() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let wp_id = WorkPathId::generate();
    let graph = make_test_graph(&wp_id);
    store.insert_work_path_graph(&graph).await.unwrap();

    let svc = WorkPacketService::new(store);
    let budget = CostBudget {
        max_tokens: 10000,
        max_api_calls: 50,
        max_seconds: 300,
    };

    let node_id = WorkPathNodeId::new("n2").unwrap();

    let packet = svc
        .generate_work_packet(
            &common::test_tenant(),
            &common::test_project(),
            &common::test_actor(),
            &wp_id,
            &node_id,
            "Optimize query performance",
            Some(budget.clone()),
        )
        .await
        .unwrap();

    assert_eq!(packet.objective, "Optimize query performance");
    match packet.cost_budget {
        Some(ref b) => {
            assert_eq!(b.max_tokens, budget.max_tokens);
            assert_eq!(b.max_api_calls, budget.max_api_calls);
            assert_eq!(b.max_seconds, budget.max_seconds);
        }
        None => panic!("expected cost_budget to be Some"),
    }
}

#[tokio::test]
async fn test_work_packet_validate_not_found() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    use authority_domain::PacketId;
    let svc = WorkPacketService::new(store);
    let fake_id = PacketId::generate();
    let result = svc.validate_packet_boundaries(&fake_id).await;
    assert!(matches!(result.unwrap_err(), ServiceError::WorkPacketNotFound));
}

#[tokio::test]
async fn test_work_packet_validate_persisted() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let wp_id = WorkPathId::generate();
    let graph = make_test_graph(&wp_id);
    store.insert_work_path_graph(&graph).await.unwrap();

    let svc = WorkPacketService::new(store);
    let node_id = WorkPathNodeId::new("n1").unwrap();

    let packet = svc
        .generate_work_packet(
            &common::test_tenant(),
            &common::test_project(),
            &common::test_actor(),
            &wp_id,
            &node_id,
            "Validate this packet",
            None,
        )
        .await
        .unwrap();

    let result = svc.validate_packet_boundaries(&packet.id).await.unwrap();
    assert!(result.valid, "issues: {:?}", result.issues);
    assert_eq!(result.packet_id, packet.id.as_str());
    assert!(result.issues.is_empty());
}

#[tokio::test]
async fn test_work_packet_permission_scope() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let wp_id = WorkPathId::generate();
    let graph = make_test_graph(&wp_id);
    store.insert_work_path_graph(&graph).await.unwrap();

    let svc = WorkPacketService::new(store);
    let node_id = WorkPathNodeId::new("n1").unwrap();

    let packet = svc
        .generate_work_packet(
            &common::test_tenant(),
            &common::test_project(),
            &common::test_actor(),
            &wp_id,
            &node_id,
            "Refactor module",
            None,
        )
        .await
        .unwrap();

    assert!(
        packet
            .permission_scope
            .allowed_actions
            .contains(&"file.read".to_string()),
        "expected file.read in allowed_actions"
    );
    assert!(
        packet
            .permission_scope
            .allowed_actions
            .contains(&"file.write".to_string()),
        "expected file.write in allowed_actions"
    );
    assert!(
        !packet.permission_scope.allow_network,
        "network should be disallowed by default"
    );
    assert_eq!(packet.permission_scope.max_concurrent_files, 5);
}
