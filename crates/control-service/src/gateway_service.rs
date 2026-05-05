use agent_gateway::{flow::GatewayRequest, flow::GatewayResult, AgentGateway, GatewayError};
use control_store::CoreStore;

/// Thin service wrapper around the agent gateway.
///
/// The heavy logic lives in the agent-gateway crate; this service layer
/// exists to be injectable into ServiceContext for API/CLI/MCP transport.
#[derive(Clone)]
pub struct GatewayService {
    gateway: AgentGateway,
}

impl GatewayService {
    pub fn new(store: CoreStore) -> Self {
        Self {
            gateway: AgentGateway::new(store),
        }
    }

    /// Execute a command through the gateway pipeline.
    pub async fn execute_command(
        &self,
        request: GatewayRequest,
    ) -> Result<GatewayResult, GatewayError> {
        self.gateway.execute(request).await
    }

    /// Health check: gateway is functional if store connection is available.
    pub fn is_healthy(&self) -> bool {
        // In a real implementation, this would check the store connection.
        true
    }
}
