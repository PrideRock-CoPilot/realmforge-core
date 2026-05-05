// Integration tests for Phase 3 Work Paths acceptance.
//
// Validates TEST-WORKPATH-001 against a live PostgreSQL database.
// Tests gracefully skip when no database is available.

mod common;

use authority_domain::{TenantId, WorkPathId, WorkPathNodeId, WorkPathNodeType};
use control_service::WorkPathService;

#[tokio::test]
async fn test_work_path_node_generates_scoped_packet() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = WorkPathService::new(store);
    let work_path_id = WorkPathId::generate();
    let prefix = work_path_id.as_str();
    let root_id = WorkPathNodeId::new(format!("{prefix}-root")).unwrap();
    let contract_id = WorkPathNodeId::new(format!("{prefix}-contract")).unwrap();
    let unrelated_id = WorkPathNodeId::new(format!("{prefix}-unrelated")).unwrap();

    svc.create_work_path(
        &work_path_id,
        "Login Work Path",
        "Acceptance test work path",
    )
    .await
    .unwrap();

    svc.add_node(
        &work_path_id,
        &root_id,
        WorkPathNodeType::Module,
        "Login Module",
        vec!["catalog/Login/catalog.json".to_string()],
        vec!["contract.login.module".to_string()],
        vec!["TEST-CATALOG-001".to_string()],
        vec!["trace.login.module".to_string()],
        vec![contract_id.clone()],
    )
    .await
    .unwrap();

    svc.add_node(
        &work_path_id,
        &contract_id,
        WorkPathNodeType::RuntimeContract,
        "Login Contract",
        vec!["catalog/Login/contracts/login.v1.json".to_string()],
        vec!["contract.login.v1".to_string()],
        vec!["TEST-WORKPATH-001".to_string()],
        vec!["trace.login.contract".to_string()],
        vec![],
    )
    .await
    .unwrap();

    svc.add_node(
        &work_path_id,
        &unrelated_id,
        WorkPathNodeType::Policy,
        "Unrelated Policy",
        vec!["catalog/Other/policies/other.policy.json".to_string()],
        vec!["contract.other".to_string()],
        vec!["TEST-OTHER-001".to_string()],
        vec!["trace.other".to_string()],
        vec![],
    )
    .await
    .unwrap();

    let graph = svc.get_work_path_graph(&work_path_id).await.unwrap();
    assert_eq!(graph.nodes.len(), 3);
    assert!(graph.get_node(&root_id).is_some());
    assert!(graph.get_node(&contract_id).is_some());
    assert!(graph.get_node(&unrelated_id).is_some());

    let tenant = TenantId::generate();
    let packet = svc
        .traverse_to_packet(
            &work_path_id,
            &root_id,
            &common::test_actor(),
            &tenant,
            &common::test_project(),
            "Implement login module",
            None,
        )
        .await
        .unwrap();

    assert_eq!(packet.work_path_node_id, root_id.as_str());
    assert_eq!(packet.objective, "Implement login module");
    assert!(packet
        .allowed_file_paths
        .contains(&"catalog/Login/catalog.json".to_string()));
    assert!(packet
        .allowed_file_paths
        .contains(&"catalog/Login/contracts/login.v1.json".to_string()));
    assert!(!packet
        .allowed_file_paths
        .contains(&"catalog/Other/policies/other.policy.json".to_string()));
    assert!(packet
        .required_contracts
        .contains(&"contract.login.module".to_string()));
    assert!(packet
        .required_contracts
        .contains(&"contract.login.v1".to_string()));
    assert!(packet
        .required_tests
        .contains(&"TEST-CATALOG-001".to_string()));
    assert!(packet
        .required_tests
        .contains(&"TEST-WORKPATH-001".to_string()));
    assert!(packet
        .required_trace_points
        .contains(&"trace.login.module".to_string()));
    assert!(packet
        .required_trace_points
        .contains(&"trace.login.contract".to_string()));
    assert!(packet
        .permission_scope
        .allowed_actions
        .contains(&"file.write".to_string()));
}
