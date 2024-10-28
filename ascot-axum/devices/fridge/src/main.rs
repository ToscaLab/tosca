extern crate alloc;

mod fridge_mockup;

use core::net::Ipv4Addr;

use alloc::sync::Arc;

use async_lock::Mutex;
use serde::{Deserialize, Serialize};

// Ascot library.
use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::{Route, RouteHazards};

// Ascot axum.
use ascot_axum::actions::{ActionError, SerialAction, SerialPayload};
use ascot_axum::devices::fridge::Fridge;
use ascot_axum::error::Error;
use ascot_axum::extract::{Json, State};
use ascot_axum::server::AscotServer;
use ascot_axum::service::ServiceConfig;

// Command line library
use clap::Parser;

// Tracing library.
use tracing_subscriber::filter::LevelFilter;

// A fridge implementation mock-up
use fridge_mockup::FridgeMockup;

#[derive(Clone, Default)]
struct DeviceState(Arc<Mutex<FridgeMockup>>);

impl DeviceState {
    fn new(fridge: FridgeMockup) -> Self {
        Self(Arc::new(Mutex::new(fridge)))
    }
}

impl core::ops::Deref for DeviceState {
    type Target = Arc<Mutex<FridgeMockup>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for DeviceState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Deserialize)]
struct IncreaseTemperature {
    increment: f64,
}

#[derive(Serialize)]
struct ChangeTempResponse {
    temperature: f64,
}

async fn increase_temperature(
    State(state): State<DeviceState>,
    Json(inputs): Json<IncreaseTemperature>,
) -> Result<SerialPayload<ChangeTempResponse>, ActionError> {
    let mut fridge = state.lock().await;
    fridge.increase_temperature(inputs.increment);

    Ok(SerialPayload::new(ChangeTempResponse {
        temperature: fridge.temperature,
    }))
}

#[derive(Deserialize)]
struct DecreaseTemperature {
    decrement: f64,
}

async fn decrease_temperature(
    State(state): State<DeviceState>,
    Json(inputs): Json<DecreaseTemperature>,
) -> Result<SerialPayload<ChangeTempResponse>, ActionError> {
    let mut fridge = state.lock().await;
    fridge.decrease_temperature(inputs.decrement);

    Ok(SerialPayload::new(ChangeTempResponse {
        temperature: fridge.temperature,
    }))
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Server address.
    ///
    /// Only an `Ipv4` address is accepted.
    #[arg(short, long, default_value_t = Ipv4Addr::UNSPECIFIED)]
    address: Ipv4Addr,

    /// Server host name.
    #[arg(short = 'n', long)]
    hostname: String,

    /// Server port.
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Service domain.
    #[arg(short, long, default_value = "device")]
    domain: String,

    /// Service type.
    #[arg(short = 't', long = "type", default_value = "General Fridge")]
    service_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing subscriber.
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    let cli = Cli::parse();

    let state = DeviceState::new(FridgeMockup::default());

    // Increase temperature action invoked by a `PUT` route.
    let increase_temp_action = SerialAction::stateful(
        RouteHazards::with_hazards(
            Route::put("/increase-temperature")
                .description("Increase temperature.")
                .input(Input::rangef64("increment", (1., 4., 0.1, 2.))),
            &[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood],
        ),
        increase_temperature,
        state.clone(),
    );

    // Decrease temperature action invoked by a `PUT` route.
    let decrease_temp_action = SerialAction::stateful(
        RouteHazards::single_hazard(
            Route::put("/decrease-temperature")
                .description("Decrease temperature.")
                .input(Input::rangef64("decrement", (1., 4., 0.1, 2.))),
            Hazard::ElectricEnergyConsumption,
        ),
        decrease_temperature,
        state.clone(),
    );

    // Increase temperature action invoked by a `POST` route.
    let increase_temp_post_action = SerialAction::stateful(
        RouteHazards::no_hazards(
            Route::post("/increase-temperature")
                .description("Increase temperature.")
                .input(Input::rangef64("increment", (1., 4., 0.1, 2.))),
        ),
        increase_temperature,
        state,
    );

    // A fridge device which is going to be run on the server.
    let device = Fridge::new()
        .increase_temperature(increase_temp_action)?
        .decrease_temperature(decrease_temp_action)?
        .add_action(increase_temp_post_action)?
        .build()?;

    // Run a discovery service and the device on the server.
    AscotServer::new(device)
        .address(cli.address)
        .port(cli.port)
        .service(
            ServiceConfig::mdns_sd("fridge")
                .hostname(&cli.hostname)
                .domain_name(&cli.domain)
                .service_type(&cli.service_type),
        )
        .run()
        .await
}
