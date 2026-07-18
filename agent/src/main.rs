mod dbus;
mod discovery;
mod grpc;
mod metrics;
mod metrics_history;
mod pam_auth;
mod pty;
mod systemd;

use anyhow::Result;
use grpc::AgentServiceImpl;
use shared::pb::agent_service_server::AgentServiceServer;
use shared::DEFAULT_AGENT_GRPC_PORT;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = format!("0.0.0.0:{DEFAULT_AGENT_GRPC_PORT}").parse()?;
    let hostname = whoami_hostname();

    let _mdns_daemon = match discovery::announce(&hostname, DEFAULT_AGENT_GRPC_PORT) {
        Ok(daemon) => {
            info!(%hostname, port = DEFAULT_AGENT_GRPC_PORT, "agent announced over mDNS");
            Some(daemon)
        }
        Err(err) => {
            tracing::warn!(error = %err, "mDNS announcement unavailable (network without multicast?), continuing without auto-discovery");
            None
        }
    };

    match dbus::system_hostname().await {
        Ok(name) => info!(hostname = %name, "D-Bus hostname resolved"),
        Err(err) => {
            tracing::warn!(error = %err, "could not reach org.freedesktop.hostname1 (D-Bus unavailable in this environment?)")
        }
    }

    let history = metrics_history::start();

    info!(%addr, "starting the agent's gRPC server");
    Server::builder()
        .add_service(AgentServiceServer::new(AgentServiceImpl { history }))
        .serve(addr)
        .await?;

    Ok(())
}

fn whoami_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "lyra-agent".to_string())
}
