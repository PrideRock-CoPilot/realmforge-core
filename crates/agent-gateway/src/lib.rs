pub mod error;
pub mod flow;
pub mod scope_validator;

pub use error::{DenialCode, GatewayError};
pub use flow::{execute_gateway_flow, GatewayContext, GatewayRequest, GatewayResult};
pub use scope_validator::{validate_file_scope, validate_schema_scope, FileScopeResult, SchemaScopeResult};

use control_store::CoreStore;

/// Configuration for the agent gateway.
#[derive(Clone, Debug)]
pub struct GatewayConfig {
    /// Maximum time (in seconds) a gateway request may take.
    pub timeout_seconds: u64,
    /// Maximum payload size in bytes.
    pub max_payload_size_bytes: u64,
    /// Allowed transport protocols.
    pub allowed_transports: Vec<String>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_payload_size_bytes: 1_048_576, // 1 MB
            allowed_transports: vec!["mcp".to_string(), "api".to_string(), "cli".to_string()],
        }
    }
}

/// The agent gateway — the **only** path for agent-visible mutation.
///
/// All agent commands go through this gateway. It orchestrates the full
/// pipeline: authenticate → load grant → load packet → validate state
/// → authorize → verify scope → create audit → execute → anchor snapshot.
#[derive(Clone)]
pub struct AgentGateway {
    store: CoreStore,
    config: GatewayConfig,
}

impl AgentGateway {
    pub fn new(store: CoreStore) -> Self {
        Self {
            store,
            config: GatewayConfig::default(),
        }
    }

    pub fn with_config(store: CoreStore, config: GatewayConfig) -> Self {
        Self { store, config }
    }

    pub fn store(&self) -> &CoreStore {
        &self.store
    }

    pub fn config(&self) -> &GatewayConfig {
        &self.config
    }

    /// Execute a gateway request through the full pipeline.
    pub async fn execute(
        &self,
        request: flow::GatewayRequest,
    ) -> Result<flow::GatewayResult, GatewayError> {
        flow::execute_gateway_flow(request, &self.store, chrono::Utc::now()).await
    }
}
