extern crate alloc;

mod light_mockup;

use core::net::Ipv4Addr;

use alloc::sync::Arc;

use async_lock::Mutex;
use serde::{Deserialize, Serialize};

// Ascot library.
use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::{Route, RouteHazards};

// Ascot axum device.
use ascot_axum::actions::{ActionError, EmptyAction, EmptyPayload, SerialAction, SerialPayload};
use ascot_axum::devices::light::Light;
use ascot_axum::error::Error;
use ascot_axum::extract::{Json, State};
use ascot_axum::server::AscotServer;
use ascot_axum::service::ServiceConfig;

// Command line library
use clap::Parser;

// Tracing library
use tracing_subscriber::filter::LevelFilter;

// A light implementation mock-up
use light_mockup::LightMockup;

#[derive(Clone, Default)]
struct LightState(Arc<Mutex<LightMockup>>);

impl LightState {
    fn new(light: LightMockup) -> Self {
        Self(Arc::new(Mutex::new(light)))
    }
}

impl core::ops::Deref for LightState {
    type Target = Arc<Mutex<LightMockup>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for LightState {
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
    State(state): State<LightState>,
    Json(inputs): Json<Inputs>,
) -> Result<SerialPayload<LightOnResponse>, ActionError> {
    let mut light = state.lock().await;
    light.turn_light_on(inputs.brightness, inputs.save_energy);

    Ok(SerialPayload::new(LightOnResponse {
        brightness: light.brightness,
        save_energy: light.save_energy,
    }))
}

async fn turn_light_off(State(state): State<LightState>) -> Result<EmptyPayload, ActionError> {
    state.lock().await.turn_light_off();
    Ok(EmptyPayload::new("Turn light off worked perfectly"))
}

async fn toggle(State(state): State<LightState>) -> Result<EmptyPayload, ActionError> {
    state.lock().await.toggle();
    Ok(EmptyPayload::new("Toggle worked perfectly"))
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
    // Initialize tracing subscriber.
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    let cli = Cli::parse();

    let state = LightState::new(LightMockup::default());

    // Turn light on action invoked by a `PUT` route.
    let light_on_action = SerialAction::stateful(
        RouteHazards::single_hazard(
            Route::put("/on").description("Turn light on.").inputs([
                Input::rangef64("brightness", (0., 20., 0.1, 0.)),
                Input::boolean("save-energy", false),
            ]),
            Hazard::FireHazard,
        ),
        turn_light_on,
        state.clone(),
    );

    // Turn light on action invoked by a `POST` route.
    let light_on_post_action = SerialAction::stateful(
        RouteHazards::no_hazards(Route::post("/on").description("Turn light on.").inputs([
            Input::rangef64("brightness", (0., 20., 0.1, 0.)),
            Input::boolean("save-energy", false),
        ])),
        turn_light_on,
        state.clone(),
    );

    // Turn light off action invoked by a `PUT` route.
    let light_off_action = EmptyAction::stateful(
        RouteHazards::no_hazards(Route::put("/off").description("Turn light off.")),
        turn_light_off,
        state.clone(),
    );

    // Toggle action invoked by a `PUT` route.
    let toggle_action = EmptyAction::stateful(
        RouteHazards::no_hazards(Route::put("/toggle").description("Toggle a light.")),
        toggle,
        state,
    );

    // A light device which is going to be run on the server.
    let device = Light::new(light_on_action, light_off_action)?
        .add_action(toggle_action)?
        .add_action(light_on_post_action)?
        .build();

    // Run a discovery service and the device on the server.
    AscotServer::new(device)
        .address(cli.address)
        .port(cli.port)
        .service(
            ServiceConfig::mdns_sd("light")
                .hostname(&cli.hostname)
                .domain_name(&cli.domain)
                .service_type(&cli.service_type),
        )
        .run()
        .await
}
