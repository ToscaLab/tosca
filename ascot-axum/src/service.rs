//! The firmware can be discovered in the local network, which also represents
//! the trusted network, through the `mDNS` protocol.

use core::net::Ipv4Addr;

use heapless::FnvIndexMap;

use crate::error::Result;
use crate::server::DEFAULT_SERVER_PORT;

// Maximum stack elements.
const MAXIMUM_ELEMENTS: usize = 16;

/// A service builder.
#[derive(Debug)]
pub struct ServiceBuilder {
    /// Instance name.
    pub(crate) instance_name: &'static str,
    /// Service port.
    pub(crate) port: u16,
    /// Service properties.
    pub(crate) properties: FnvIndexMap<String, String, MAXIMUM_ELEMENTS>,
}

impl ServiceBuilder {
    /// Creates a new [`ServiceBuilder`] for a `mDNS-SD` service.
    pub fn mdns_sd(instance_name: &'static str) -> Self {
        Self {
            instance_name,
            port: DEFAULT_SERVER_PORT,
            properties: FnvIndexMap::new(),
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

    // Sets service port.
    pub(crate) fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

// A new service.
pub(crate) struct Service;

impl Service {
    // Runs a service.
    #[inline]
    pub(crate) fn run(service_builder: ServiceBuilder, main_http_address: Ipv4Addr) -> Result<()> {
        #[cfg(feature = "mdns-sd-service")]
        crate::services::mdns_sd::run(service_builder, main_http_address)
    }
}
