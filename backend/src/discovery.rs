use mdns_sd::{ServiceDaemon, ServiceEvent};
use shared::MDNS_SERVICE_TYPE;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub type DiscoveredAgents = Arc<RwLock<HashMap<String, String>>>;

pub fn start() -> mdns_sd::Result<(DiscoveredAgents, ServiceDaemon)> {
    let daemon = ServiceDaemon::new()?;
    let receiver = daemon.browse(MDNS_SERVICE_TYPE)?;
    let agents: DiscoveredAgents = Arc::new(RwLock::new(HashMap::new()));

    let agents_task = agents.clone();
    tokio::spawn(async move {
        while let Ok(event) = receiver.recv_async().await {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let Some(addr) = info.get_addresses_v4().into_iter().next() else {
                        continue;
                    };
                    let uri = format!("http://{}:{}", addr, info.get_port());
                    info!(instance = %info.get_fullname(), %uri, "agent discovered via mDNS");
                    agents_task
                        .write()
                        .await
                        .insert(info.get_fullname().to_string(), uri);
                }
                ServiceEvent::ServiceRemoved(_ty, fullname) => {
                    agents_task.write().await.remove(&fullname);
                    warn!(%fullname, "agent gone (mDNS)");
                }
                _ => {}
            }
        }
    });

    Ok((agents, daemon))
}
