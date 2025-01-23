mod fridge_mockup;

use core::net::Ipv4Addr;

use std::sync::Arc;

use async_lock::Mutex;
use serde::{Deserialize, Serialize};

// Ascot library.
use ascot_library::device::DeviceInfo;
use ascot_library::energy::{EnergyClass, EnergyEfficiencies, EnergyEfficiency};
use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::Route;

// Ascot axum.
use ascot_axum::actions::error::ErrorResponse;
use ascot_axum::actions::info::{info_stateful, InfoResponse};
use ascot_axum::actions::serial::{mandatory_serial_stateful, serial_stateful, SerialResponse};
use ascot_axum::devices::fridge::Fridge;
use ascot_axum::error::Error;
use ascot_axum::extract::{FromRef, Json, State};
use ascot_axum::server::Server;
use ascot_axum::service::ServiceConfig;

// Command line library
use clap::Parser;

// Tracing library.
use tracing_subscriber::filter::LevelFilter;

// A fridge implementation mock-up
use fridge_mockup::FridgeMockup;

#[derive(Clone)]
struct FridgeState {
    state: InternalState,
    info: FridgeInfoState,
}

impl FridgeState {
    fn new(state: FridgeMockup, info: DeviceInfo) -> Self {
        Self {
            state: InternalState::new(state),
            info: FridgeInfoState::new(info),
        }
    }
}

#[derive(Clone, Default)]
struct InternalState(Arc<Mutex<FridgeMockup>>);

impl InternalState {
    fn new(fridge: FridgeMockup) -> Self {
        Self(Arc::new(Mutex::new(fridge)))
    }
}

impl core::ops::Deref for InternalState {
    type Target = Arc<Mutex<FridgeMockup>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for InternalState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromRef<FridgeState> for InternalState {
    fn from_ref(fridge_state: &FridgeState) -> InternalState {
        fridge_state.state.clone()
    }
}

#[derive(Clone)]
struct FridgeInfoState {
    info: Arc<Mutex<DeviceInfo>>,
}

impl FridgeInfoState {
    fn new(info: DeviceInfo) -> Self {
        Self {
            info: Arc::new(Mutex::new(info)),
        }
    }
}

impl core::ops::Deref for FridgeInfoState {
    type Target = Arc<Mutex<DeviceInfo>>;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl core::ops::DerefMut for FridgeInfoState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

impl FromRef<FridgeState> for FridgeInfoState {
    fn from_ref(fridge_state: &FridgeState) -> FridgeInfoState {
        fridge_state.info.clone()
    }
}

#[derive(Deserialize)]
struct IncreaseTemperature {
    increment: f64,
}

#[derive(Serialize, Deserialize)]
struct ChangeTempResponse {
    temperature: f64,
}

async fn increase_temperature(
    State(state): State<InternalState>,
    Json(inputs): Json<IncreaseTemperature>,
) -> Result<SerialResponse<ChangeTempResponse>, ErrorResponse> {
    let mut fridge = state.lock().await;
    fridge.increase_temperature(inputs.increment);

    Ok(SerialResponse::new(ChangeTempResponse {
        temperature: fridge.temperature,
    }))
}

#[derive(Deserialize)]
struct DecreaseTemperature {
    decrement: f64,
}

async fn decrease_temperature(
    State(state): State<InternalState>,
    Json(inputs): Json<DecreaseTemperature>,
) -> Result<SerialResponse<ChangeTempResponse>, ErrorResponse> {
    let mut fridge = state.lock().await;
    fridge.decrease_temperature(inputs.decrement);

    Ok(SerialResponse::new(ChangeTempResponse {
        temperature: fridge.temperature,
    }))
}

async fn info(State(state): State<FridgeInfoState>) -> Result<InfoResponse, ErrorResponse> {
    // Retrieve fridge information state.
    let fridge_info = state.lock().await.clone();

    Ok(InfoResponse::new(fridge_info))
}

async fn update_energy_efficiency(
    State(state): State<FridgeState>,
) -> Result<InfoResponse, ErrorResponse> {
    // Retrieve internal state.
    let fridge = state.state.lock().await;

    // Retrieve fridge info state.
    let mut fridge_info = state.info.lock().await;

    // Compute a new energy efficiency according to the temperature value
    let energy_efficiency = if fridge.temperature.is_sign_negative() {
        EnergyEfficiency::new(5, EnergyClass::C)
    } else {
        EnergyEfficiency::new(-5, EnergyClass::D)
    };

    // Change energy efficiencies information replacing the old ones.
    fridge_info.energy.energy_efficiencies = Some(EnergyEfficiencies::init(energy_efficiency));

    Ok(InfoResponse::new(fridge_info.clone()))
}

#[derive(Parser)]
#[command(version, about, long_about = "A complete fridge device example.")]
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
    #[arg(short, long, default_value = "fridge")]
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

    // Define a state for the fridge.
    let state = FridgeState::new(FridgeMockup::default(), DeviceInfo::empty());

    // Increase temperature `PUT` route.
    let increase_temp_route = Route::put("/increase-temperature")
        .description("Increase temperature.")
        .with_slice_hazards(&[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood])
        .with_input(Input::rangef64_with_default("increment", (1., 4., 0.1), 2.));

    // Decrease temperature `PUT` route.
    let decrease_temp_route = Route::put("/decrease-temperature")
        .description("Decrease temperature.")
        .with_slice_hazards(&[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood])
        .with_input(Input::rangef64_with_default("decrement", (1., 4., 0.1), 2.));

    // Increase temperature `POST` route.
    let increase_temp_post_route = Route::post("/increase-temperature")
        .description("Increase temperature.")
        .with_slice_hazards(&[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood])
        .with_input(Input::rangef64_with_default("increment", (1., 4., 0.1), 2.));

    // Device info `GET` route.
    let info_route = Route::get("/info")
        .description("Get info about a fridge.")
        .with_hazard(Hazard::LogEnergyConsumption);

    // Update energy efficiency `GET` route.
    let update_energy_efficiency_route = Route::get("/update-energy")
        .description("Update energy efficiency.")
        .with_hazard(Hazard::LogEnergyConsumption);

    // A fridge device which is going to be run on the server.
    let device = Fridge::with_state(state)
        // This method is mandatory, if not called, a compiler error is raised.
        .increase_temperature(mandatory_serial_stateful(
            increase_temp_route,
            increase_temperature,
        ))
        // This method is mandatory, if not called, a compiler error is raised.
        .decrease_temperature(mandatory_serial_stateful(
            decrease_temp_route,
            decrease_temperature,
        ))
        .add_action(serial_stateful(
            increase_temp_post_route,
            increase_temperature,
        ))?
        .add_info_action(info_stateful(info_route, info))
        .add_info_action(info_stateful(
            update_energy_efficiency_route,
            update_energy_efficiency,
        ))
        .into_device();

    // Run a discovery service and the device on the server.
    Server::new(device)
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
