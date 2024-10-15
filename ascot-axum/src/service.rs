//! The firmware can be discovered in the local network, which also represents
//! the trusted network, through the `mDNS` protocol.

use core::net::Ipv4Addr;

use heapless::FnvIndexMap;

use crate::error::Result;

// Maximum stack elements.
const MAXIMUM_ELEMENTS: usize = 8;

// Service type.
//
// It defines the type of a service. The default value is `Firmware`.
const SERVICE_TYPE: &str = "Firmware";

/// A service builder.
#[derive(Debug)]
pub struct ServiceBuilder<'a> {
    // Instance name.
    pub(crate) instance_name: &'a str,
    // Service properties.
    pub(crate) properties: FnvIndexMap<String, String, MAXIMUM_ELEMENTS>,
    // Service host name
    pub(crate) hostname: &'a str,
    // Service domain name.
    pub(crate) domain_name: Option<&'a str>,
    // Service type.
    pub(crate) service_type: &'a str,
    // Disable Ipv6.
    pub(crate) disable_ipv6: bool,
    // Disable docker network.
    pub(crate) disable_docker: bool,
}

impl<'a> ServiceBuilder<'a> {
    /// Creates a new [`ServiceBuilder`] for a `mDNS-SD` service.
    pub const fn mdns_sd(instance_name: &'a str) -> Self {
        Self {
            instance_name,
            properties: FnvIndexMap::new(),
            hostname: instance_name,
            domain_name: None,
            service_type: SERVICE_TYPE,
            disable_ipv6: false,
            disable_docker: false,
        }
    }

    /// Sets a service property.
    pub fn property(mut self, property: (impl Into<String>, impl Into<String>)) -> Self {
        // If an equivalent key already exists in the map: the key remains and
        // retains in its place in the order.
        // Its corresponding value is updated with value and the older value
        // is returned inside.
        let _ = self.properties.insert(property.0.into(), property.1.into());
        self
    }

    /// Sets the service host name.
    pub const fn hostname(mut self, hostname: &'a str) -> Self {
        self.hostname = hostname;
        self
    }

    /// Sets the service domain name.
    pub const fn domain_name(mut self, domain_name: &'a str) -> Self {
        self.domain_name = Some(domain_name);
        self
    }

    /// Sets the service type.
    ///
    /// This allows to detect the type of firmware associated with a service.
    pub const fn service_type(mut self, service_type: &'a str) -> Self {
        self.service_type = service_type;
        self
    }

    /// Disables IPv6 addresses.
    pub const fn ipv6(mut self) -> Self {
        self.disable_ipv6 = true;
        self
    }

    /// Disables docker bridge.
    pub const fn docker(mut self) -> Self {
        self.disable_docker = true;
        self
    }
}

// A new service.
pub(crate) struct Service;

impl Service {
    // Runs a service.
    #[inline]
    pub(crate) fn run(
        service_builder: ServiceBuilder,
        server_address: Ipv4Addr,
        port: u16,
    ) -> Result<()> {
        #[cfg(feature = "mdns-sd-service")]
        crate::services::mdns_sd::run(service_builder, server_address, port)
    }
}
