pub mod pb {
    tonic::include_proto!("lyra.agent.v1");
}

pub use pb::agent_service_client::AgentServiceClient;
pub use pb::agent_service_server::{AgentService, AgentServiceServer};
pub use pb::{
    ListServicesRequest, ListServicesResponse, MetricsRequest, MetricsResponse, PingRequest,
    PingResponse, PtyResize, ServiceUnit, TerminalInput, TerminalOutput,
};

pub const DEFAULT_AGENT_GRPC_PORT: u16 = 50051;

pub const MDNS_SERVICE_TYPE: &str = "_lyra-agent._tcp.local.";
