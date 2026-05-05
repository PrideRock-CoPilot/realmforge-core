// Integration tests for Board Plan lifecycle: Create → Submit → Approve → Release Command.
//
// Validates the full BoardsService workflow against a live PostgreSQL database.
// Tests gracefully skip when no database is available.

mod common;

use control_service::error::ServiceError;
use control_service::BoardsService;

#[tokio::test]
async fn test_board_plan_create_and_list() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = BoardsService::new(store);

    let plan = svc
        .create_plan(
            "Phase 6: Boards".to_string(),
            vec!["WP-BOARDS-001".to_string()],
        )
        .await
        .unwrap();

    assert_eq!(plan.title, "Phase 6: Boards");
    assert_eq!(plan.work_path_refs, vec!["WP-BOARDS-001"]);
    assert_eq!(format!("{}", plan.status), "draft");

    // List should return the plan
    let plans = svc.list_plans(None, 10, 0).await.unwrap();
    assert!(!plans.is_empty());
    assert!(plans.iter().any(|p| p.id == plan.id));
}

#[tokio::test]
async fn test_board_plan_full_lifecycle() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = BoardsService::new(store);

    // 1. Create
    let plan = svc
        .create_plan("Release v2.0".to_string(), vec!["WP-CORE-001".to_string()])
        .await
        .unwrap();
    assert_eq!(format!("{}", plan.status), "draft");

    // 2. Submit for approval (Draft → InReview)
    let submitted = svc.submit_for_approval(&plan.id).await.unwrap();
    assert_eq!(format!("{}", submitted.status), "in_review");

    // 3. Approve (InReview → Approved)
    let approved = svc
        .approve_plan(
            &plan.id,
            "approver-1".to_string(),
            Some("looks good".to_string()),
        )
        .await
        .unwrap();
    assert_eq!(format!("{}", approved.status), "approved");

    // 4. Submit release command (requires Approved)
    let release = svc
        .submit_release_command(
            &plan.id,
            "bundle-v2.0".to_string(),
            "approval-v2.0".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(format!("{}", release.status), "pending");
    assert_eq!(release.bundle_ref, "bundle-v2.0");
}

#[tokio::test]
async fn test_board_plan_reject_returns_to_draft() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = BoardsService::new(store);

    let plan = svc
        .create_plan("Rejected Plan".to_string(), vec![])
        .await
        .unwrap();

    let submitted = svc.submit_for_approval(&plan.id).await.unwrap();
    assert_eq!(format!("{}", submitted.status), "in_review");

    // Reject — should return to Draft
    let rejected = svc
        .reject_plan(
            &plan.id,
            "reviewer-1".to_string(),
            "needs more detail".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(format!("{}", rejected.status), "draft");
}

#[tokio::test]
async fn test_board_plan_submit_from_invalid_state() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = BoardsService::new(store);

    let plan = svc
        .create_plan("Invalid State Test".to_string(), vec![])
        .await
        .unwrap();

    // Submit, then try submitting again
    svc.submit_for_approval(&plan.id).await.unwrap();
    let result = svc.submit_for_approval(&plan.id).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ServiceError::Validation(msg) => {
            assert!(msg.contains("in_review") || msg.contains("only Draft"));
        }
        _ => panic!("expected Validation error"),
    }
}

#[tokio::test]
async fn test_board_plan_approve_from_invalid_state() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = BoardsService::new(store);

    // Try approving a plan that's still in Draft (not InReview)
    let plan = svc
        .create_plan("Direct Approve Fail".to_string(), vec![])
        .await
        .unwrap();

    let result = svc
        .approve_plan(&plan.id, "approver-1".to_string(), None)
        .await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ServiceError::Validation(msg) => {
            assert!(msg.contains("draft") || msg.contains("only InReview"));
        }
        _ => panic!("expected Validation error"),
    }
}

#[tokio::test]
async fn test_board_plan_release_from_non_approved_fails() {
    let store = match common::get_store().await {
        Some(s) => s,
        None => return,
    };

    let svc = BoardsService::new(store);

    // Create a draft plan — cannot release
    let plan = svc
        .create_plan("Cannot Release".to_string(), vec![])
        .await
        .unwrap();

    let result = svc
        .submit_release_command(&plan.id, "bundle".to_string(), "approval".to_string())
        .await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ServiceError::Validation(msg) => {
            assert!(msg.contains("draft") || msg.contains("only Approved"));
        }
        _ => panic!("expected Validation error"),
    }
}
