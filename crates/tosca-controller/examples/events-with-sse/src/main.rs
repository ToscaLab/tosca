use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use askama::Template;
use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    response::{ErrorResponse, Html, IntoResponse},
    routing::get,
    Router,
};

use clap::Parser;

use futures::stream::Stream;

use tokio::signal;
use tokio::sync::Mutex;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

use tracing::{error, info};
use tracing_subscriber::filter::LevelFilter;

use tosca_controller::controller::Controller;
use tosca_controller::discovery::{Discovery, TransportProtocol};

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "A controller that scans the network for `tosca` devices, \
                  subscribes to their brokers to receive events, and \
                  and displays their data in real-time on a web page."
)]
struct Cli {
    /// Controller `IPv4` address (defaults to "localhost" address).
    ///
    /// Only `IPv4` addresses are accepted.
    #[arg(long, default_value_t = Ipv4Addr::LOCALHOST)]
    ip: Ipv4Addr,
    /// Web controller port (defaults to port 8123).
    #[arg(long, default_value_t = 8123)]
    port: u16,
    /// Service domain (defaults to "tosca").
    #[arg(short = 'd', long = "domain", default_value = "tosca")]
    service_domain: String,
}

#[derive(Clone, Template)]
#[template(path = "index.html")]
struct DeviceConsoles {
    device_ids: Vec<String>,
}

impl DeviceConsoles {
    const fn new(device_ids: Vec<String>) -> Self {
        Self { device_ids }
    }
}

#[derive(Clone)]
struct AppState {
    device_consoles: DeviceConsoles,
    controller: Arc<Mutex<Controller>>,
}

impl AppState {
    fn new(device_consoles: DeviceConsoles, controller: Controller) -> Self {
        Self {
            device_consoles,
            controller: Arc::new(Mutex::new(controller)),
        }
    }
}

#[inline]
async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let rendered_data = match state.device_consoles.render() {
        Ok(template_rendered) => template_rendered,
        Err(e) => format!("<html><body>Something went wrong: {e}</body></html>"),
    };

    Html(rendered_data)
}

async fn event_stream(
    Path(device_id): Path<usize>,
    State(state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ErrorResponse> {
    let controller = state.controller.lock().await;

    let device = controller.devices().get(device_id).ok_or_else(|| {
        let err = format!("Device `{device_id}` does not exist");
        error!(err);
        ErrorResponse::from(err)
    })?;

    let receiver = device.resuscribe_to_receiver().ok_or_else(|| {
        let err = format!("Impossible to resubscribe to device `{device_id}`");
        error!(err);
        ErrorResponse::from(err)
    })?;

    let stream = BroadcastStream::new(receiver);

    // Convert the stream into SSE events
    let sse_stream = stream
        .map(move |event| {
            let data = format!("{event:?}");
            Ok(Event::default().id(device_id.to_string()).data(data))
        })
        .throttle(Duration::from_secs(1));

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default().interval(Duration::from_secs(1))))
}

#[inline]
async fn shutdown_controller() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to run a `tosca-controller` method")]
    Tosca(#[source] tosca_controller::error::Error),
    #[error("failed to bind to the socket")]
    Bind(#[source] std::io::Error),
    #[error("failed to run the server")]
    Run(#[source] std::io::Error),
}

#[inline]
async fn cleanup(controller: Arc<Mutex<Controller>>) {
    if let Some(controller) = Arc::into_inner(controller) {
        controller.into_inner().shutdown().await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing subscriber.
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    let cli = Cli::parse();

    let discovery = Discovery::new(cli.service_domain)
        .timeout(Duration::from_secs(2))
        // TODO: Implement a "TCP-UDP" strategy to retrieve both UDP and TCP
        // mDNS-SD sockets that may coexist within the same environment.
        .transport_protocol(TransportProtocol::UDP)
        .disable_ipv6()
        .disable_network_interface("docker0");

    // Create a controller.
    let mut controller = Controller::new(discovery);

    // Run discovery process.
    controller.discover().await.map_err(Error::Tosca)?;

    let devices = controller.devices_mut();

    info!("Number of discovered devices: {}", devices.len());

    if devices.is_empty() {
        info!("No devices discovered. Terminating the process without any errors.");
        return Ok(());
    }

    let mut device_ids = Vec::new();
    // FIXME: Using enumerata is an hack because IDs have not implemented yet.
    for (id, device) in devices.iter_mut().enumerate() {
        if device.start_event_receiver(id, 100).await.is_some() {
            device_ids.push(id.to_string());
        }
    }
    tokio::time::sleep(Duration::from_millis(100)).await;

    let state = AppState::new(DeviceConsoles::new(device_ids), controller);
    let controller_clone = state.controller.clone();
    let app = Router::new()
        .route("/", get(index))
        .route("/events/{device_id}", get(event_stream))
        .with_state(state);

    // Creates the web controller listener bind.
    let listener_bind = SocketAddr::new(IpAddr::V4(cli.ip), cli.port);

    // Creates listener.
    let listener = tokio::net::TcpListener::bind(&listener_bind)
        .await
        .map_err(Error::Bind)?;

    info!("Start running server on address: {}:{}", cli.ip, cli.port);

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_controller())
        .await
        .map_err(Error::Run)?;

    // Cleanup controller
    cleanup(controller_clone).await;

    Ok(())
}
