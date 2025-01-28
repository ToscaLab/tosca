//! The firmware can be discovered in the local network, which also represents
//! the trusted network, through the `mDNS` protocol.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

use crate::error::Result;

// Service type.
//
// It defines the default type of a service.
const SERVICE_TYPE: &str = "_ascot._tcp.local.";

/// A service configurator.
#[derive(Debug)]
pub struct ServiceConfig<'a> {
    // Instance name.
    pub(crate) instance_name: &'a str,
    // Service properties.
    pub(crate) properties: HashMap<String, String>,
    // Service host name
    pub(crate) hostname: &'a str,
    // Service type.
    pub(crate) service_type: &'a str,
    // Disable IPv6.
    pub(crate) disable_ipv6: bool,
    // Disable IP.
    pub(crate) disable_ip: Option<IpAddr>,
    // Disable network interface.
    pub(crate) disable_network_interface: Option<&'a str>,
}

impl<'a> ServiceConfig<'a> {
    /// Creates a [`ServiceConfig`] for a `mDNS-SD` service.
    #[must_use]
    pub fn mdns_sd(instance_name: &'a str) -> Self {
        Self {
            instance_name,
            properties: HashMap::new(),
            hostname: instance_name,
            service_type: SERVICE_TYPE,
            disable_ipv6: false,
            disable_ip: None,
            disable_network_interface: None,
        }
    }

    /// Sets a service property.
    #[must_use]
    pub fn property(mut self, property: (impl Into<String>, impl Into<String>)) -> Self {
        self.properties.insert(property.0.into(), property.1.into());
        self
    }

    /// Sets the service host name.
    #[must_use]
    pub const fn hostname(mut self, hostname: &'a str) -> Self {
        self.hostname = hostname;
        self
    }

    /// Sets the service type.
    ///
    /// This allows to detect the type of firmware associated with a service.
    #[must_use]
    pub const fn service_type(mut self, service_type: &'a str) -> Self {
        self.service_type = service_type;
        self
    }

    /// Disables `IPv6` interfaces.
    #[must_use]
    pub const fn disable_ipv6(mut self) -> Self {
        self.disable_ipv6 = true;
        self
    }

    /// Disables the given IP address.
    #[must_use]
    #[inline]
    pub fn disable_ip(mut self, ip: impl Into<IpAddr>) -> Self {
        self.disable_ip = Some(ip.into());
        self
    }

    /// Disables the given network interface.
    #[must_use]
    pub const fn disable_network_interface(mut self, network_interface: &'a str) -> Self {
        self.disable_network_interface = Some(network_interface);
        self
    }
}

// A new service.
pub(crate) struct Service;

impl Service {
    // Runs a service.
    #[inline]
    pub(crate) fn run(
        service_config: ServiceConfig,
        server_address: Ipv4Addr,
        port: u16,
    ) -> Result<()> {
        #[cfg(feature = "mdns-sd-service")]
        crate::services::mdns_sd::run(service_config, server_address, port)
    }
}
