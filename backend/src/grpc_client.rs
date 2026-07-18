use anyhow::{Context, Result};
use shared::AgentServiceClient;
use tonic::transport::Channel;

pub async fn connect(addr: &str) -> Result<AgentServiceClient<Channel>> {
    let channel = Channel::from_shared(addr.to_string())
        .context("invalid gRPC address")?
        .connect()
        .await
        .context("gRPC connection to agent failed")?;

    Ok(AgentServiceClient::new(channel))
}
