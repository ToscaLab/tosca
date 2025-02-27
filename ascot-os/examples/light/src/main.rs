mod light_mockup;

use std::net::Ipv4Addr;
use std::sync::Arc;

use ascot::device::DeviceInfo;
use ascot::energy::{EnergyClass, EnergyEfficiencies, EnergyEfficiency};
use ascot::hazards::Hazard;
use ascot::parameters::Parameters;
use ascot::route::Route;

use ascot_os::actions::error::ErrorResponse;
use ascot_os::actions::info::{info_stateful, InfoResponse};
use ascot_os::actions::ok::{mandatory_ok_stateful, ok_stateful, OkResponse};
use ascot_os::actions::serial::{mandatory_serial_stateful, serial_stateful, SerialResponse};
use ascot_os::devices::light::Light;
use ascot_os::error::Error;
use ascot_os::extract::{FromRef, Json, State};
use ascot_os::server::Server;
use ascot_os::service::{ServiceConfig, TransportProtocol};

use async_lock::Mutex;

use clap::builder::ValueParser;
use clap::Parser;

use serde::{Deserialize, Serialize};

use tracing_subscriber::filter::LevelFilter;

use light_mockup::LightMockup;

#[derive(Clone)]
struct LightState {
    state: InternalState,
    info: LightInfoState,
}

impl LightState {
    fn new(state: LightMockup, info: DeviceInfo) -> Self {
        Self {
            state: InternalState::new(state),
            info: LightInfoState::new(info),
        }
    }
}

#[derive(Clone, Default)]
struct InternalState(Arc<Mutex<LightMockup>>);

impl InternalState {
    fn new(light: LightMockup) -> Self {
        Self(Arc::new(Mutex::new(light)))
    }
}

impl core::ops::Deref for InternalState {
    type Target = Arc<Mutex<LightMockup>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for InternalState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromRef<LightState> for InternalState {
    fn from_ref(light_state: &LightState) -> InternalState {
        light_state.state.clone()
    }
}

#[derive(Clone)]
struct LightInfoState {
    info: Arc<Mutex<DeviceInfo>>,
}

impl LightInfoState {
    fn new(info: DeviceInfo) -> Self {
        Self {
            info: Arc::new(Mutex::new(info)),
        }
    }
}

impl core::ops::Deref for LightInfoState {
    type Target = Arc<Mutex<DeviceInfo>>;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl core::ops::DerefMut for LightInfoState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

impl FromRef<LightState> for LightInfoState {
    fn from_ref(light_state: &LightState) -> LightInfoState {
        light_state.info.clone()
    }
}

#[derive(Serialize, Deserialize)]
struct LightOnResponse {
    brightness: i64,
    #[serde(rename = "save-energy")]
    save_energy: bool,
}

#[derive(Deserialize)]
struct Inputs {
    brightness: i64,
    #[serde(alias = "save-energy")]
    save_energy: bool,
}

async fn turn_light_on(
    State(state): State<InternalState>,
    Json(inputs): Json<Inputs>,
) -> Result<SerialResponse<LightOnResponse>, ErrorResponse> {
    let mut light = state.lock().await;
    light.turn_light_on(inputs.brightness, inputs.save_energy);

    Ok(SerialResponse::new(LightOnResponse {
        brightness: light.brightness,
        save_energy: light.save_energy,
    }))
}

async fn turn_light_off(State(state): State<InternalState>) -> Result<OkResponse, ErrorResponse> {
    state.lock().await.turn_light_off();
    Ok(OkResponse::ok())
}

async fn toggle(State(state): State<InternalState>) -> Result<OkResponse, ErrorResponse> {
    state.lock().await.toggle();
    Ok(OkResponse::ok())
}

async fn info(State(state): State<LightInfoState>) -> Result<InfoResponse, ErrorResponse> {
    // Retrieve light information state.
    let light_info = state.lock().await.clone();

    Ok(InfoResponse::new(light_info))
}

async fn update_energy_efficiency(
    State(state): State<LightState>,
) -> Result<InfoResponse, ErrorResponse> {
    // Retrieve internal state.
    let light = state.state.lock().await;

    // Retrieve light info state.
    let mut light_info = state.info.lock().await;

    // Compute a new energy efficiency according to the brightness value
    let energy_efficiency = if light.brightness > 15 {
        EnergyEfficiency::new(5, EnergyClass::C)
    } else {
        EnergyEfficiency::new(-5, EnergyClass::D)
    };

    // Change energy efficiencies information replacing the old ones.
    light_info.energy.energy_efficiencies = Some(EnergyEfficiencies::init(energy_efficiency));

    Ok(InfoResponse::new(light_info.clone()))
}

fn parse_transport_protocol(protocol: &str) -> Result<TransportProtocol, std::io::Error> {
    match protocol {
        "tcp" | "TCP" => Ok(TransportProtocol::TCP),
        "udp" | "UDP" => Ok(TransportProtocol::UDP),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{protocol:?} is not a supported protocol."),
        )),
    }
}

#[derive(Parser)]
#[command(version, about, long_about = "A complete light device example.")]
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
    #[arg(short = 'd', long = "domain")]
    service_domain: String,

    /// Service transport protocol.
    #[arg(short = 't', long = "protocol", default_value_t = TransportProtocol::TCP, value_parser = ValueParser::new(parse_transport_protocol))]
    service_transport_protocol: TransportProtocol,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing subscriber.
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    let cli = Cli::parse();

    // Define a state for the light.
    let state = LightState::new(LightMockup::default(), DeviceInfo::empty());

    // Turn light on `PUT` route.
    let light_on_route = Route::put("/on")
        .description("Turn light on.")
        .with_hazard(Hazard::ElectricEnergyConsumption)
        .with_parameters(
            Parameters::new()
                .rangef64("brightness", (0., 20., 0.1))
                .bool("save-energy", false),
        );

    // Turn light on `POST` route.
    let light_on_post_route = Route::post("/on")
        .description("Turn light on.")
        .with_hazard(Hazard::ElectricEnergyConsumption)
        .with_parameters(
            Parameters::new()
                .rangef64("brightness", (0., 20., 0.1))
                .bool("save-energy", false),
        );

    // Turn light off `PUT` route.
    let light_off_route = Route::put("/off").description("Turn light off.");

    // Toggle `PUT` route.
    let toggle_route = Route::put("/toggle")
        .description("Toggle a light.")
        .with_hazard(Hazard::ElectricEnergyConsumption);

    // Device info `GET` route.
    let info_route = Route::get("/info")
        .description("Get info about a light.")
        .with_hazard(Hazard::LogEnergyConsumption);

    // Update energy efficiency `GET` route.
    let update_energy_efficiency_route = Route::get("/update-energy")
        .description("Update energy efficiency.")
        .with_hazard(Hazard::LogEnergyConsumption);

    // A light device which is going to be run on the server.
    let device = Light::with_state(state)
        // This method is mandatory, if not called, a compiler error is raised.
        .turn_light_on(mandatory_serial_stateful(light_on_route, turn_light_on))
        // This method is mandatory, if not called, a compiler error is raised.
        .turn_light_off(mandatory_ok_stateful(light_off_route, turn_light_off))
        .add_action(serial_stateful(light_on_post_route, turn_light_on))?
        .add_action(ok_stateful(toggle_route, toggle))?
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
        .discovery_service(
            ServiceConfig::mdns_sd("light")
                .hostname(&cli.hostname)
                .domain(&cli.service_domain)
                .transport_protocol(cli.service_transport_protocol),
        )
        .run()
        .await
}
