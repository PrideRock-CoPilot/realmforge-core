pub mod error;
pub mod metrics;
pub mod middleware;
pub mod models;
pub mod routes;

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};
use control_service::ServiceContext;
use tower_http::{services::ServeDir, trace::TraceLayer};

/// OpenAPI document covering all annotated RealmForge API routes.
/// Used by the generate-schema binary to emit openapi.json.
#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "RealmForge Control API",
        version = "1.0.0",
        description = "Governance kernel REST API — session, command, audit, snapshot, rollback, boards, and more.",
    ),
    paths(
        // Health
        routes::health::health,
        routes::health::ready,
        routes::health::live,
        // Sessions
        routes::session::issue_session,
        routes::session::get_session,
        routes::session::revoke_session,
        routes::session::activate_session,
        routes::session::renew_session,
        // Commands
        routes::command::propose_command,
        routes::command::authorize_command,
        routes::command::apply_command,
        routes::command::get_command,
        routes::command::deny_command,
        // Audit
        routes::audit::query_events,
        routes::audit::verify_chain,
        routes::audit::stream_audit_events,
        // Snapshots
        routes::snapshot::create_snapshot,
        routes::snapshot::list_snapshots,
        routes::snapshot::get_snapshot,
        routes::snapshot::validate_snapshot,
        routes::snapshot::compare_snapshots,
        // Rollback
        routes::rollback::preview_rollback,
        routes::rollback::execute_rollback,
        routes::rollback::verify_rollback,
        // Actors
        routes::actor::get_actor_scope,
        // Work Packets
        routes::work_packet::generate_work_packet,
        routes::work_packet::validate_work_packet,
        // Boards
        routes::boards::create_plan,
        routes::boards::list_plans,
        routes::boards::submit_plan,
        routes::boards::approve_plan,
        routes::boards::reject_plan,
        routes::boards::submit_release,
        // Catalog
        routes::catalog::list_catalog,
        routes::catalog::copy_catalog,
        routes::catalog::get_catalog_provenance,
        // Gateway
        routes::gateway::execute_gateway,
        // Knowledge
        routes::knowledge::list_datasets,
        routes::knowledge::get_dataset,
        routes::knowledge::query_knowledge,
        // Work Paths
        routes::work_path::create_work_path,
        routes::work_path::get_work_path,
        routes::work_path::traverse_work_path,
        // Bundles
        routes::bundle::create_bundle,
        routes::bundle::list_bundles,
        routes::bundle::get_bundle,
        routes::bundle::verify_bundle,
        routes::bundle::deploy_bundle,
        // Runtime
        routes::runtime::list_runtimes,
        routes::runtime::deploy_runtime,
        routes::runtime::get_runtime_status,
        routes::runtime::stop_runtime,
        // Login
        routes::login::login,
        routes::login::set_login_policy,
        routes::login::get_login_policy,
        routes::login::list_login_blocks,
        // Live Watch
        routes::live_watch::start_monitoring,
        routes::live_watch::stop_monitoring,
        routes::live_watch::record_signal,
        routes::live_watch::get_signals,
        routes::live_watch::propose_remediation,
        routes::live_watch::list_proposals,
        routes::live_watch::approve_proposal,
        routes::live_watch::get_profile,
        routes::live_watch::update_profile,
    ),
    components(
        schemas(
            // Error
            error::ProblemDetails,
            // Models
            models::IssueSessionRequest,
            models::SessionResponse,
            models::RenewSessionRequest,
            models::RevokeSessionResponse,
            models::ProposeCommandRequest,
            models::DenyCommandRequest,
            models::CommandResponse,
            models::QueryEventsParams,
            models::ProjectParams,
            models::EventsListResponse,
            models::ChainVerifyResponse,
            models::CreateSnapshotRequest,
            models::SnapshotResponse,
            models::SnapshotValidateResponse,
            models::SnapshotCompareResponse,
            models::RollbackPreviewRequest,
            models::RollbackPreviewResponse,
            models::RollbackExecuteRequest,
            models::RollbackExecuteResponse,
            models::RollbackVerifyResponse,
            models::ActorScopeResponse,
            models::WorkPacketGenerateResponse,
            models::WorkPacketValidateResponse,
            models::GenerateWorkPacketRequest,
            models::HealthResponse,
            models::LoginResponse,
            models::AuthorizeRequest,
            models::CommandScopeRequest,
            // Boards
            models::CreateBoardPlanRequest,
            models::BoardPlanResponse,
            models::BoardPlanListResponse,
            models::ListPlansQuery,
            models::BoardApprovalRequest,
            models::BoardReleaseRequest,
            models::BoardReleaseResponse,
            // Work packet route request
            routes::work_packet::GenerateWorkPacketRequest,
            // Catalog route types
            routes::catalog::ListCatalogParams,
            routes::catalog::CopyCatalogRequest,
            routes::catalog::CatalogEntryResponse,
            routes::catalog::ProvenanceResponse,
            // Work path route types
            routes::work_path::CreateWorkPathRequest,
            routes::work_path::TraverseWorkPathRequest,
            routes::work_path::WorkPathGraphResponse,
            routes::work_path::TraverseResponse,
            // Bundle route types
            routes::bundle::CreateBundleRequest,
            routes::bundle::ListBundlesQuery,
            routes::bundle::VerifyBundleRequest,
            // Runtime route types
            routes::runtime::ListRuntimesQuery,
            routes::runtime::DeployBundleRequest,
            // Login route types
            routes::login::LoginRequest,
            routes::login::SetPolicyRequest,
            // Live Watch route types
            routes::live_watch::SignalQuery,
            routes::live_watch::ProposalQuery,
            routes::live_watch::RecordSignalBody,
            routes::live_watch::ProfileBody,
            routes::live_watch::ThresholdJson,
        )
    ),
    modifiers(&BearerSecurityAddon),
    tags(
        (name = "health", description = "Health and readiness probes"),
        (name = "sessions", description = "Session lifecycle — issue, activate, renew, revoke"),
        (name = "commands", description = "Bounded command lifecycle — propose, authorize, apply, deny"),
        (name = "audit", description = "Audit trail — query events, verify hash chain"),
        (name = "snapshots", description = "Content-addressed snapshots — create, validate, compare"),
        (name = "rollback", description = "Rollback engine — preview, execute, verify"),
        (name = "actors", description = "Actor scope retrieval"),
        (name = "work-packets", description = "Agent work packet generation and validation"),
        (name = "boards", description = "Boards — human planning, approval, and release command surface"),
        (name = "catalog", description = "Catalog — content-addressed module registry with provenance"),
        (name = "gateway", description = "Agent gateway — single entry point for all agent writes"),
        (name = "knowledge", description = "Knowledge store — dataset indexing and query"),
        (name = "work-paths", description = "Work paths — graph-based agent scope traversal"),
        (name = "bundles", description = "Runtime bundles — artifact manifests and deployment signing"),
        (name = "runtime", description = "Runtime instances — deploy, monitor, stop"),
        (name = "login", description = "Login — authentication, session issuance, policy management"),
        (name = "live-watch", description = "Live Watch — runtime signal monitoring, remediation proposals"),
    )
)]
pub struct ApiDoc;

/// Adds Bearer token security scheme to the OpenAPI document.
struct BearerSecurityAddon;

impl utoipa::Modify for BearerSecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("session_token")
                    .build(),
            ),
        );
    }
}

/// Build the full API router with all routes and middleware.
pub fn router(state: ServiceContext) -> Router {
    Router::new()
        // Observability
        .route("/metrics", get(metrics::get_metrics))
        // Health
        .route("/health", get(routes::health::health))
        .route("/v1/health/ready", get(routes::health::ready))
        .route("/v1/health/live", get(routes::health::live))
        // Session
        .route("/v1/session", post(routes::session::issue_session))
        .route("/v1/session/:id", get(routes::session::get_session))
        .route("/v1/session/:id", delete(routes::session::revoke_session))
        .route(
            "/v1/session/:id/activate",
            post(routes::session::activate_session),
        )
        .route(
            "/v1/session/:id/renew",
            post(routes::session::renew_session),
        )
        // Commands
        .route("/v1/commands", post(routes::command::propose_command))
        .route("/v1/commands/:id", get(routes::command::get_command))
        .route(
            "/v1/commands/:id/authorize",
            put(routes::command::authorize_command),
        )
        .route(
            "/v1/commands/:id/apply",
            put(routes::command::apply_command),
        )
        .route("/v1/commands/:id/deny", post(routes::command::deny_command))
        // Audit
        .route("/v1/audit/events", get(routes::audit::query_events))
        .route("/v1/audit/chain/verify", get(routes::audit::verify_chain))
        .route("/v1/audit/stream", get(routes::audit::stream_audit_events))
        // Snapshots — literal routes before parameterized routes
        .route("/v1/snapshots", post(routes::snapshot::create_snapshot))
        .route("/v1/snapshots", get(routes::snapshot::list_snapshots))
        .route(
            "/v1/snapshots/compare",
            get(routes::snapshot::compare_snapshots),
        )
        .route("/v1/snapshots/:id", get(routes::snapshot::get_snapshot))
        .route(
            "/v1/snapshots/:id/validate",
            post(routes::snapshot::validate_snapshot),
        )
        // Rollback
        .route(
            "/v1/rollback/preview",
            post(routes::rollback::preview_rollback),
        )
        .route(
            "/v1/rollback/execute",
            post(routes::rollback::execute_rollback),
        )
        .route(
            "/v1/rollback/:id/verify",
            get(routes::rollback::verify_rollback),
        )
        // Actors
        .route("/v1/actors/:id/scope", get(routes::actor::get_actor_scope))
        // Work Packets
        .route(
            "/v1/work-packets/generate",
            post(routes::work_packet::generate_work_packet),
        )
        .route(
            "/v1/work-packets/:id/validate",
            post(routes::work_packet::validate_work_packet),
        )
        // Boards (previously unregistered — wired in as part of utoipa annotation work)
        .route("/v1/boards/plans", post(routes::boards::create_plan))
        .route("/v1/boards/plans", get(routes::boards::list_plans))
        .route(
            "/v1/boards/plans/:id/submit",
            post(routes::boards::submit_plan),
        )
        .route(
            "/v1/boards/plans/:id/approve",
            post(routes::boards::approve_plan),
        )
        .route(
            "/v1/boards/plans/:id/reject",
            post(routes::boards::reject_plan),
        )
        .route("/v1/boards/releases", post(routes::boards::submit_release))
        // Catalog
        .route("/v1/catalog", get(routes::catalog::list_catalog))
        .route("/v1/catalog/copy", post(routes::catalog::copy_catalog))
        .route(
            "/v1/catalog/:id/provenance",
            get(routes::catalog::get_catalog_provenance),
        )
        // Gateway
        .route(
            "/v1/gateway/execute",
            post(routes::gateway::execute_gateway),
        )
        // Knowledge
        .route(
            "/v1/knowledge/datasets",
            get(routes::knowledge::list_datasets),
        )
        .route(
            "/v1/knowledge/datasets/:id",
            get(routes::knowledge::get_dataset),
        )
        .route(
            "/v1/knowledge/query",
            post(routes::knowledge::query_knowledge),
        )
        // Work Paths
        .route("/v1/work-paths", post(routes::work_path::create_work_path))
        .route("/v1/work-paths/:id", get(routes::work_path::get_work_path))
        .route(
            "/v1/work-paths/:id/traverse",
            post(routes::work_path::traverse_work_path),
        )
        // Bundles
        .route("/v1/bundles", post(routes::bundle::create_bundle))
        .route("/v1/bundles", get(routes::bundle::list_bundles))
        .route("/v1/bundles/:id", get(routes::bundle::get_bundle))
        .route(
            "/v1/bundles/:id/verify",
            post(routes::bundle::verify_bundle),
        )
        .route(
            "/v1/bundles/:id/deploy",
            post(routes::bundle::deploy_bundle),
        )
        // Runtime
        .route("/v1/runtime", get(routes::runtime::list_runtimes))
        .route("/v1/runtime/deploy", post(routes::runtime::deploy_runtime))
        .route("/v1/runtime/:id", get(routes::runtime::get_runtime_status))
        .route("/v1/runtime/:id/stop", post(routes::runtime::stop_runtime))
        // Login
        .route("/v1/login", post(routes::login::login))
        .route("/v1/login/policy", post(routes::login::set_login_policy))
        .route(
            "/v1/login/policy/:tenant_id",
            get(routes::login::get_login_policy),
        )
        .route(
            "/v1/login/blocks/:tenant_id",
            get(routes::login::list_login_blocks),
        )
        // Live Watch
        .route(
            "/v1/live-watch/:app_id/start",
            post(routes::live_watch::start_monitoring),
        )
        .route(
            "/v1/live-watch/:app_id/stop",
            post(routes::live_watch::stop_monitoring),
        )
        .route(
            "/v1/live-watch/:app_id/signals",
            post(routes::live_watch::record_signal),
        )
        .route(
            "/v1/live-watch/:app_id/signals",
            get(routes::live_watch::get_signals),
        )
        .route(
            "/v1/live-watch/:app_id/propose",
            post(routes::live_watch::propose_remediation),
        )
        .route(
            "/v1/live-watch/:app_id/proposals",
            get(routes::live_watch::list_proposals),
        )
        .route(
            "/v1/live-watch/:app_id/proposals/:id/approve",
            post(routes::live_watch::approve_proposal),
        )
        .route(
            "/v1/live-watch/profiles/:app_id",
            get(routes::live_watch::get_profile),
        )
        .route(
            "/v1/live-watch/profiles/:app_id",
            put(routes::live_watch::update_profile),
        )
        // Static frontend assets (Phase 1: same-origin hosting, ADR-0005)
        // frontend/dist/ is served at / — catch-all for client-side routing
        .fallback_service(ServeDir::new("frontend/dist").append_index_html_on_directories(true))
        // Middleware — order matters: bearer runs after tracing so request_id is set
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::bearer_auth_middleware,
        ))
        .layer(axum_middleware::from_fn(
            middleware::request_tracing_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
