use anyhow::{Context, Result};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use shared::MDNS_SERVICE_TYPE;

pub fn announce(instance_name: &str, port: u16) -> Result<ServiceDaemon> {
    let daemon = ServiceDaemon::new().context("starting the mDNS daemon")?;

    let host_ipv4 = local_ipv4().unwrap_or_else(|| "0.0.0.0".to_string());
    let hostname = format!("{instance_name}.local.");

    let service_info = ServiceInfo::new(
        MDNS_SERVICE_TYPE,
        instance_name,
        &hostname,
        host_ipv4,
        port,
        None,
    )
    .context("building the mDNS ServiceInfo")?;

    daemon
        .register(service_info)
        .context("registering the mDNS service")?;

    Ok(daemon)
}

fn local_ipv4() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}
