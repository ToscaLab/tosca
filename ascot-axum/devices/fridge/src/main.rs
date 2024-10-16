extern crate alloc;

mod fridge_mockup;

use core::net::Ipv4Addr;

use alloc::sync::Arc;

use async_lock::Mutex;
use serde::{Deserialize, Serialize};

// Ascot library.
use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::Route;

// Ascot axum.
use ascot_axum::device::{DeviceAction, DeviceError, DevicePayload};
use ascot_axum::devices::fridge::Fridge;
use ascot_axum::error::Error;
use ascot_axum::extract::{Extension, Json};
use ascot_axum::server::AscotServer;
use ascot_axum::service::ServiceConfig;

// Command line library
use clap::Parser;

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
    Extension(state): Extension<DeviceState>,
    Json(inputs): Json<IncreaseTemperature>,
) -> Result<DevicePayload, DeviceError> {
    let mut fridge = state.lock().await;
    fridge.increase_temperature(inputs.increment);

    DevicePayload::new(ChangeTempResponse {
        temperature: fridge.temperature,
    })
}

#[derive(Deserialize)]
struct DecreaseTemperature {
    decrement: f64,
}

async fn decrease_temperature(
    Extension(state): Extension<DeviceState>,
    Json(inputs): Json<DecreaseTemperature>,
) -> Result<DevicePayload, DeviceError> {
    let mut fridge = state.lock().await;
    fridge.decrease_temperature(inputs.decrement);

    DevicePayload::new(ChangeTempResponse {
        temperature: fridge.temperature,
    })
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
    let cli = Cli::parse();

    // Configuration for the `PUT` increase temperature route.
    let increase_temp_config = Route::put("/increase-temperature")
        .description("Increase temperature.")
        .input(Input::rangef64("increment", (1., 4., 0.1, 2.)));

    // Configuration for the `POST` increase temperature route.
    let increase_temp_post_config = Route::post("/increase-temperature")
        .description("Increase temperature.")
        .input(Input::rangef64("increment", (1., 4., 0.1, 2.)));

    // Configuration for the `PUT` decrease temperature route.
    let decrease_temp_config = Route::put("/decrease-temperature")
        .description("Decrease temperature.")
        .input(Input::rangef64("decrement", (1., 4., 0.1, 2.)));

    // A fridge device which is going to be run on the server.
    let device = Fridge::new()
        .increase_temperature(DeviceAction::with_hazards(
            increase_temp_config,
            increase_temperature,
            &[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood],
        ))?
        .decrease_temperature(DeviceAction::with_hazards(
            decrease_temp_config,
            decrease_temperature,
            &[Hazard::ElectricEnergyConsumption],
        ))?
        .add_action(DeviceAction::no_hazards(
            increase_temp_post_config,
            increase_temperature,
        ))?
        .state(DeviceState::new(FridgeMockup::default()))
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
