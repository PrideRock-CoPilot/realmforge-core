pub mod actor_service;
pub mod audit_service;
pub mod boards_service;
pub mod build_watch_service;
pub mod bundle_service;
pub mod catalog_service;
pub mod command_service;
pub mod error;
pub mod gateway_service;
pub mod intake_service;
pub mod knowledge_service;

pub mod live_watch_service;
pub mod login_handler;
pub mod rollback_service;
pub mod runtime_service;
pub mod session_service;
pub mod skill_service;
pub mod snapshot_service;
pub mod work_packet_service;
pub mod work_path_service;

pub use actor_service::ActorService;
pub use audit_service::AuditService;
pub use boards_service::BoardsService;
pub use build_watch_service::BuildWatchService;
pub use bundle_service::BundleService;
pub use catalog_service::CatalogService;
pub use command_service::CommandService;
pub use control_store::plan_store::InMemoryPlanStore;
pub use error::ServiceError;
pub use gateway_service::GatewayService;
pub use intake_service::IntakeService;
pub use knowledge_service::{KnowledgeQueryResult, KnowledgeService};
pub use live_watch_service::LiveWatchService;
pub use login_handler::LoginHandler;
pub use policy_engine::PolicyDecision;
pub use rollback_service::RollbackService;
pub use runtime_service::RuntimeService;
pub use session_service::SessionService;
pub use skill_service::SkillService;
pub use snapshot_service::SnapshotService;
pub use work_packet_service::WorkPacketService;
pub use work_path_service::WorkPathService;

use control_store::CoreStore;
use tracing::instrument;

/// Shared application context holding all service instances and the underlying store.
/// Passed to route handlers via Axum state injection — must be Clone.
#[derive(Clone)]
pub struct ServiceContext {
    store: CoreStore,
    pub sessions: SessionService,
    pub commands: CommandService,
    pub audit: AuditService,
    pub snapshots: SnapshotService,
    pub rollback: RollbackService,
    pub actors: ActorService,
    pub skills: SkillService,
    pub work_packets: WorkPacketService,
    pub catalog: CatalogService,
    pub work_paths: WorkPathService,
    pub gateway: GatewayService,
    pub knowledge: KnowledgeService,
    pub boards: BoardsService,
    pub build_watch: BuildWatchService,
    pub bundles: BundleService,
    pub live_watch: LiveWatchService,
    pub runtimes: RuntimeService,
    pub login: LoginHandler,
    pub intake: IntakeService,
}

impl ServiceContext {
    #[instrument(skip(store))]
    pub fn new(store: CoreStore) -> Self {
        let audit_service = AuditService::new(store.clone());
        let session_service = SessionService::new(store.clone(), audit_service.clone());
        let intake_store = std::sync::Arc::new(InMemoryPlanStore::new());
        Self {
            store: store.clone(),
            sessions: session_service.clone(),
            commands: CommandService::new(store.clone(), audit_service.clone()),
            audit: audit_service.clone(),
            snapshots: SnapshotService::new(store.clone()),
            rollback: RollbackService::new(store.clone(), audit_service.clone()),
            actors: ActorService::new(store.clone()),
            skills: SkillService::new(store.clone()),
            work_packets: WorkPacketService::new(store.clone()),
            catalog: CatalogService::new(store.clone()),
            work_paths: WorkPathService::new(store.clone()),
            gateway: GatewayService::new(store.clone()),
            knowledge: KnowledgeService::new(store.clone()),
            boards: BoardsService::new(store.clone()),
            build_watch: BuildWatchService::new(store.clone()),
            bundles: BundleService::new(store.clone()),
            live_watch: LiveWatchService::new(store.clone()),
            runtimes: RuntimeService::new(store.clone()),
            login: LoginHandler::new(store, session_service, audit_service),
            intake: IntakeService::new(intake_store),
        }
    }

    pub async fn connect(database_url: &str) -> Result<Self, ServiceError> {
        let store = CoreStore::connect(database_url).await?;
        Ok(Self::new(store))
    }

    pub async fn connect_and_migrate(database_url: &str) -> Result<Self, ServiceError> {
        let ctx = Self::connect(database_url).await?;
        ctx.store.migrate().await?;
        Ok(ctx)
    }

    pub fn connect_lazy(database_url: &str) -> Result<Self, ServiceError> {
        let store = CoreStore::connect_lazy(database_url)?;
        Ok(Self::new(store))
    }

    /// Access the underlying store (used by CLI grant commands).
    pub fn store(&self) -> &CoreStore {
        &self.store
    }
}
