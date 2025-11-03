use std::time::Duration;

use tosca_controller::controller::Controller;
use tosca_controller::discovery::{Discovery, TransportProtocol};
use tosca_controller::error::Error;

use clap::Parser;

use tracing::info;
use tracing_subscriber::filter::LevelFilter;

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "A controller to receive events from `tosca` devices in a network."
)]
struct Cli {
    /// Service domain.
    #[arg(short = 'd', long = "domain", default_value = "tosca")]
    service_domain: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing subscriber.
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    let cli = Cli::parse();

    let discovery = Discovery::new(&cli.service_domain)
        .timeout(Duration::from_secs(2))
        // TODO: Implement a "TCP-UDP" strategy to retrieve both UDP and TCP
        // mDNS-SD sockets that may coexist within the same environment.
        .transport_protocol(TransportProtocol::UDP)
        .disable_ipv6()
        .disable_network_interface("docker0");

    // Create a controller.
    let mut controller = Controller::new(discovery);

    // Run discovery process.
    controller.discover().await?;

    // Get devices.
    let devices = controller.devices();

    info!("Number of discovered devices: {}", devices.len());

    let mut receiver = controller
        .start_event_receivers(100)
        .await
        .expect("Missing receiver");

    while let Some(event) = receiver.recv().await {
        println!("{event:?}");
    }

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl-C");

    controller.shutdown().await;

    Ok(())
}
