use core::net::Ipv4Addr;

use ascot_library::device::DeviceSerializer;

use axum::{response::Redirect, Router};

use tracing::info;

use crate::device::Device;
use crate::error::Result;
use crate::service::{Service, ServiceConfig};

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
const WELL_KNOWN_URI: &str = "/.well-known/server";

/// The `Ascot` server.
#[derive(Debug)]
pub struct AscotServer<'a> {
    // HTTP address.
    http_address: Ipv4Addr,
    // Server port.
    port: u16,
    // Scheme.
    scheme: &'a str,
    // Well-known URI.
    well_known_uri: &'a str,
    // Service configurator.
    service_config: Option<ServiceConfig<'a>>,
    // Device.
    device: Device,
}

impl<'a> AscotServer<'a> {
    /// Creates a new [`AscotServer`] instance.
    pub const fn new(device: Device) -> Self {
        Self {
            http_address: DEFAULT_HTTP_ADDRESS,
            port: DEFAULT_SERVER_PORT,
            scheme: DEFAULT_SCHEME,
            well_known_uri: WELL_KNOWN_URI,
            service_config: None,
            device,
        }
    }

    /// Sets server Ipv4 address.
    pub const fn address(mut self, http_address: Ipv4Addr) -> Self {
        self.http_address = http_address;
        self
    }

    /// Sets server port.
    pub const fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets server scheme.
    pub const fn scheme(mut self, scheme: &'a str) -> Self {
        self.scheme = scheme;
        self
    }

    /// Sets well-known URI.
    pub const fn well_known_uri(mut self, well_known_uri: &'a str) -> Self {
        self.well_known_uri = well_known_uri;
        self
    }

    /// Sets a service.
    #[inline]
    pub fn service(mut self, service_config: ServiceConfig<'a>) -> Self {
        self.service_config = Some(service_config);
        self
    }

    /// Runs a smart home device on the server.
    pub async fn run(self) -> Result<()> {
        // Create listener bind.
        let listener_bind = format!("{}:{}", self.http_address, self.port);

        // Create application.
        let router = self.build_app()?;

        // Print server Ip and port.
        info!("Device reachable at this HTTP address: {listener_bind}");

        // Create a new TCP socket which responds to the specified HTTP address
        // and port.
        let listener = tokio::net::TcpListener::bind(listener_bind).await?;

        // Print server start message
        info!("Starting server...");

        // Start the server
        axum::serve(listener, router).await?;

        Ok(())
    }

    // Build device routing.
    fn build_app(self) -> Result<Router> {
        // Serialize device information returning a json format.
        let device_info = serde_json::to_value(self.device.serialize_data())?;

        info!("Server route: [GET, \"/\"]");
        info!("Server route: [GET, \"{}\"]", self.well_known_uri);

        // Run a service if present.
        if let Some(service_config) = self.service_config {
            // Add server properties.
            let service_config = service_config
                .property(("scheme", self.scheme))
                .property(("path", self.well_known_uri));

            // Run service.
            Service::run(service_config, self.http_address, self.port)?;
        }

        // Create the main router.
        //
        //- Save device info as a json format which is returned when a query to
        //  the server root is requested.
        //- Redirect well-known URI to server root.
        Ok(Router::new()
            .route(
                "/",
                axum::routing::get(move || async { axum::Json(device_info) }),
            )
            .route(
                self.well_known_uri,
                axum::routing::get(move || async { Redirect::to("/") }),
            )
            .nest(self.device.main_route, self.device.router))
    }
}
