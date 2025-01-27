use std::net::Ipv4Addr;

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

// Default service name needed to compose a well-known URI.
// https://en.wikipedia.org/wiki/Well-known_URI
//
// Requests to the servers for well-known services or information are available
// at URLs consistent well-known locations across servers.
const DEFAULT_WELL_KNOWN_SERVICE: &str = "ascot";

/// A [`Device`] server.
#[derive(Debug)]
pub struct Server<'a, S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    // HTTP address.
    http_address: Ipv4Addr,
    // Server port.
    port: u16,
    // Scheme.
    scheme: &'a str,
    // Well-known service.
    well_known_service: &'a str,
    // Service configurator.
    service_config: Option<ServiceConfig<'a>>,
    // Device.
    device: Device<S>,
}

impl<'a, S> Server<'a, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a [`Server`].
    pub const fn new(device: Device<S>) -> Self {
        Self {
            http_address: DEFAULT_HTTP_ADDRESS,
            port: DEFAULT_SERVER_PORT,
            scheme: DEFAULT_SCHEME,
            well_known_service: DEFAULT_WELL_KNOWN_SERVICE,
            service_config: None,
            device,
        }
    }

    /// Sets server Ipv4 address.
    #[must_use]
    pub const fn address(mut self, http_address: Ipv4Addr) -> Self {
        self.http_address = http_address;
        self
    }

    /// Sets server port.
    #[must_use]
    pub const fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets server scheme.
    #[must_use]
    pub const fn scheme(mut self, scheme: &'a str) -> Self {
        self.scheme = scheme;
        self
    }

    /// Sets the service name which will compose the well-known URI.
    #[must_use]
    pub fn well_known_service(mut self, service_name: &'a str) -> Self {
        self.well_known_service = service_name;
        self
    }

    /// Sets a discovery service.
    #[must_use]
    #[inline]
    pub fn discovery_service(mut self, service_config: ServiceConfig<'a>) -> Self {
        self.service_config = Some(service_config);
        self
    }

    /// Runs a smart home device on the server.
    ///
    /// # Errors
    ///
    /// It returns an error whether a server fails to start.
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
        // Consume a device returning all server information.
        let (device_main_route, device_info, device_router) = self.device.finalize();

        // Serialize device information returning a json format.
        let device_info = serde_json::to_value(device_info)?;

        // Construct well-known URI.
        let well_known_uri = format!("/.well-known/{}", self.well_known_service);

        info!("Server route: [GET, \"/\"]");
        info!("Server route: [GET, \"{}\"]", well_known_uri);

        // Run a discovery service if present.
        if let Some(service_config) = self.service_config {
            // Add server properties.
            let service_config = service_config
                .property(("scheme", self.scheme))
                .property(("path", well_known_uri.to_string()));

            // Run service.
            Service::run(&service_config, self.http_address, self.port)?;
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
                &well_known_uri,
                axum::routing::get(move || async { Redirect::to("/") }),
            )
            .nest(device_main_route, device_router))
    }
}
