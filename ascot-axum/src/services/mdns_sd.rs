use std::net::{IpAddr, Ipv4Addr};

use mdns_sd::{IfKind, ServiceDaemon, ServiceInfo};

use tracing::info;

use crate::error::{Error, ErrorKind};
use crate::service::ServiceConfig;

impl From<mdns_sd::Error> for Error {
    fn from(e: mdns_sd::Error) -> Self {
        Self::new(ErrorKind::Service, e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorKind::NotFoundAddress, e.to_string())
    }
}

pub(crate) fn run(
    service_config: ServiceConfig,
    server_address: Ipv4Addr,
    server_port: u16,
) -> std::result::Result<(), Error> {
    // Create a new mDNS service daemon
    let mdns = ServiceDaemon::new()?;

    // Disable IPv6.
    if service_config.disable_ipv6 {
        mdns.disable_interface(IfKind::IPv6)?;
    }

    // Disable IP address.
    if let Some(ip) = service_config.disable_ip {
        mdns.disable_interface(ip)?;
    }

    // Disable network interface.
    if let Some(network_interface) = service_config.disable_network_interface {
        mdns.disable_interface(network_interface)?;
    }

    // Add the .local domain as hostname suffix when not present.
    //
    // .local is a special domain name for hostnames in local area networks
    // which can be resolved via the Multicast DNS name resolution protocol.
    let hostname = if service_config.hostname.ends_with(".local.") {
        service_config.hostname
    } else {
        &format!("{}.local.", service_config.hostname)
    };

    info!("Service instance name: {}", service_config.instance_name);
    info!("Service port: {}", server_port);
    info!("Service type: {}", service_config.service_type);
    info!(
        "Device reachable at this hostname: {}:{}",
        &hostname[0..hostname.len() - 1],
        server_port
    );

    let service = ServiceInfo::new(
        // Domain label and service type
        service_config.service_type,
        // Service instance name
        service_config.instance_name,
        // DNS hostname.
        //
        // For the same hostname in the same local network, the service resolves
        // in the same addresses. It is used for A (IPv4) and AAAA (IPv6)
        // records.
        hostname,
        // Considered IP address which allow to reach out the service.
        IpAddr::V4(server_address),
        // Port on which the service listens to. It has to be same of the
        // server.
        server_port,
        // Service properties
        service_config.properties,
    )?
    .enable_addr_auto();

    mdns.register(service)?;

    Ok(())
}
