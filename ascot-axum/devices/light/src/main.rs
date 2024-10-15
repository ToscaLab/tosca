extern crate alloc;

mod light_mockup;

use core::net::Ipv4Addr;

use alloc::sync::Arc;

use async_lock::Mutex;
use serde::{Deserialize, Serialize};

// Ascot library.
use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::Route;

// Ascot axum device.
use ascot_axum::device::{DeviceAction, DeviceError, DevicePayload};
use ascot_axum::devices::light::Light;
use ascot_axum::error::Error;
use ascot_axum::extract::{Extension, Json};
use ascot_axum::server::AscotServer;
use ascot_axum::service::ServiceBuilder;

// Command line library
use clap::Parser;

// A light implementation mock-up
use light_mockup::LightMockup;

#[derive(Clone, Default)]
struct DeviceState(Arc<Mutex<LightMockup>>);

impl DeviceState {
    fn new(light: LightMockup) -> Self {
        Self(Arc::new(Mutex::new(light)))
    }
}

impl core::ops::Deref for DeviceState {
    type Target = Arc<Mutex<LightMockup>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for DeviceState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize)]
struct LightOnResponse {
    brightness: f64,
    #[serde(rename = "save-energy")]
    save_energy: bool,
}

#[derive(Deserialize)]
struct Inputs {
    brightness: f64,
    #[serde(alias = "save-energy")]
    save_energy: bool,
}

async fn turn_light_on(
    Extension(state): Extension<DeviceState>,
    Json(inputs): Json<Inputs>,
) -> Result<DevicePayload, DeviceError> {
    let mut light = state.lock().await;
    light.turn_light_on(inputs.brightness, inputs.save_energy);

    DevicePayload::new(LightOnResponse {
        brightness: light.brightness,
        save_energy: light.save_energy,
    })
}

async fn turn_light_off(
    Extension(state): Extension<DeviceState>,
) -> Result<DevicePayload, DeviceError> {
    state.lock().await.turn_light_off();
    Ok(DevicePayload::empty())
}

async fn toggle(Extension(state): Extension<DeviceState>) -> Result<DevicePayload, DeviceError> {
    state.lock().await.toggle();
    Ok(DevicePayload::empty())
    // Err(DeviceError::from_str(ascot_library::device::DeviceErrorKind::Wrong, "Error in toggling the light"))
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
    #[arg(short = 't', long = "type", default_value = "General Light")]
    service_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    // Configuration for the `PUT` turn light on route.
    let light_on_config = Route::put("/on").description("Turn light on.").inputs([
        Input::rangef64("brightness", (0., 20., 0.1, 0.)),
        Input::boolean("save-energy", false),
    ]);

    // Configuration for the `POST` turn light on route.
    let light_on_post_config = Route::post("/on").description("Turn light on.").inputs([
        Input::rangef64("brightness", (0., 20., 0.1, 0.)),
        Input::boolean("save-energy", false),
    ]);

    // Configuration for the turn light off route.
    let light_off_config = Route::put("/off").description("Turn light off.");

    // Configuration for the toggle route.
    let toggle_config = Route::put("/toggle").description("Toggle a light.");

    // A light device which is going to be run on the server.
    let device = Light::new(
        DeviceAction::with_hazard(light_on_config, turn_light_on, Hazard::FireHazard),
        DeviceAction::no_hazards(light_off_config, turn_light_off),
    )?
    .add_action(DeviceAction::no_hazards(toggle_config, toggle))?
    .add_action(DeviceAction::no_hazards(
        light_on_post_config,
        turn_light_on,
    ))?
    .state(DeviceState::new(LightMockup::default()))
    .build();

    // Run a discovery service and the device on the server.
    AscotServer::new(device)
        .address(cli.address)
        .port(cli.port)
        .service(
            ServiceBuilder::mdns_sd("light")
                .hostname(&cli.hostname)
                .domain_name(&cli.domain)
                .service_type(&cli.service_type),
        )
        .run()
        .await
}
