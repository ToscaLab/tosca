use core::net::{IpAddr, Ipv4Addr};

use ascot_library::device::DeviceSerializer;

use axum::{response::Redirect, Extension, Router};

use tracing::info;

use crate::device::Device;
use crate::error::Result;
use crate::service::{Service, ServiceBuilder};

// Default HTTP address.
//
// The entire local network is considered, so the Ipv4 unspecified address is
// used.
const DEFAULT_HTTP_ADDRESS: Ipv4Addr = Ipv4Addr::UNSPECIFIED;

// Default port.
pub(crate) const DEFAULT_SERVER_PORT: u16 = 3000;

// Default scheme is `http`.
const DEFAULT_SCHEME: &str = "http";

// Well-known URI.
// https://en.wikipedia.org/wiki/Well-known_URI
//
// Request to the server for well-known services or information are available
// at URLs consistent well-known locations across servers.
const WELL_KNOWN_URI: &str = "/.well-known/ascot";

/// The `Ascot` server.
#[derive(Debug)]
pub struct AscotServer<S>
where
    S: Clone + Send + Sync + 'static,
{
    // HTTP address.
    http_address: Ipv4Addr,
    // Server port.
    port: u16,
    // Scheme.
    scheme: &'static str,
    // Well-known URI.
    well_known_uri: &'static str,
    // Service
    service: Option<ServiceBuilder>,
    // Device.
    device: Device<S>,
}

impl<S> AscotServer<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new [`AscotServer`] instance.
    pub fn new(device: Device<S>) -> Self {
        // Initialize tracing subscriber.
        tracing_subscriber::fmt::init();

        let mut http_address = DEFAULT_HTTP_ADDRESS;

        // Retrieve the network IPs associated to the wlo1 interface.
        //
        // Only IPv4 addresses are considered.
        if let Ok(if_addresses) = if_addrs::get_if_addrs() {
            for iface in if_addresses {
                if iface.name == "wlo1" {
                    if let IpAddr::V4(ip) = iface.ip() {
                        http_address = ip;
                        break;
                    }
                }
            }
        }

        if http_address == DEFAULT_HTTP_ADDRESS {
            info!(
                "Cannot find any Ipv4 interface for the current device, use {DEFAULT_HTTP_ADDRESS}"
            );
        }

        Self {
            http_address,
            port: DEFAULT_SERVER_PORT,
            scheme: DEFAULT_SCHEME,
            well_known_uri: WELL_KNOWN_URI,
            service: None,
            device,
        }
    }

    /// Sets server port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets server scheme.
    pub fn scheme(mut self, scheme: &'static str) -> Self {
        self.scheme = scheme;
        self
    }

    /// Sets well-known URI.
    pub fn well_known_uri(mut self, well_known_uri: &'static str) -> Self {
        self.well_known_uri = well_known_uri;
        self
    }

    /// Sets a service.
    pub fn service(mut self, service: ServiceBuilder) -> Self {
        self.service = Some(service);
        self
    }

    /// Runs a smart home device on the server.
    pub async fn run(self) -> Result<()> {
        // Create listener bind.
        let listener_bind = format!("{}:{}", self.http_address, self.port);

        // Print server Ip and port.
        info!("Device reachable at this HTTP address: {listener_bind}");

        // Create application.
        let router = self.build_app()?;

        // Create a new TCP socket which responds to the specified HTTP address
        // and port.
        let listener = tokio::net::TcpListener::bind(listener_bind).await?;

        // Print server start message
        info!("Starting Ascot server...");

        // Start the server
        axum::serve(listener, router).await?;

        Ok(())
    }

    // Build device routing.
    fn build_app(self) -> Result<Router> {
        // Serialize device information returning a json format.
        let device_info = serde_json::to_value(self.device.serialize_data())?;

        // Finalize a device composing all correct routes.
        let device = self.device.finalize();

        // Run a service if present.
        if let Some(service) = self.service {
            // Add server properties.
            let service = service
                .port(self.port)
                .property(("scheme", self.scheme))
                .property(("path", self.well_known_uri));

            // Run service.
            Service::run(service, self.http_address)?;
        }

        // Create the main router.
        //
        //- Save device info as a json format which is returned when a query to
        //  the server root is requested.
        //- Redirect well-known URI to server root.
        let router = Router::new()
            .merge(device.router)
            .route(
                "/",
                axum::routing::get(move || async { axum::Json(device_info) }),
            )
            .route(
                self.well_known_uri,
                axum::routing::get(move || async { Redirect::to("/") }),
            );

        Ok(if let Some(state) = device.state {
            router.layer(Extension(state))
        } else {
            router
        })
    }
}
