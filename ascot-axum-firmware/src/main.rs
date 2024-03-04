extern crate alloc;

use alloc::sync::Arc;

use async_lock::Mutex;
use serde::Serialize;

// Library.
use ascot_library::hazards::Hazard;
use ascot_library::route::{Input, Route};

// Miscellaneous.
use ascot_axum::error::Error;
use ascot_axum::service::ServiceBuilder;

// Device.
use ascot_axum::axum::extract::{Extension, Json, Path};
use ascot_axum::device::{DeviceAction, ResponseError};
use ascot_axum::devices::light::Light;

// Server.
use ascot_axum::server::AscotServer;

mod device_light {
    #[derive(Clone)]
    pub(super) struct Light {
        pub(super) brightness: f64,
        pub(super) save_energy: bool,
    }

    impl Default for Light {
        fn default() -> Self {
            Self::init(4.0, true)
        }
    }

    impl Light {
        pub(super) const fn init(brightness: f64, save_energy: bool) -> Self {
            Self {
                brightness,
                save_energy,
            }
        }

        pub(super) fn turn_light_on(&mut self, brightness: f64, save_energy: bool) {
            self.brightness = brightness;
            self.save_energy = save_energy;
            println!(
                "Samsung turn light on with brightness {brightness} and save energy {save_energy}"
            );
        }

        pub(super) fn turn_light_off(&self) {
            println!("Run Samsung turn light off");
        }

        pub(super) fn toggle(&self) {
            println!("Run Samsung toggle");
        }
    }
}

#[derive(Clone, Default)]
struct DeviceState(Arc<Mutex<device_light::Light>>);

impl DeviceState {
    fn new(light: device_light::Light) -> Self {
        Self(Arc::new(Mutex::new(light)))
    }
}

impl core::ops::Deref for DeviceState {
    type Target = Arc<Mutex<device_light::Light>>;

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
    save_energy: bool,
}

async fn turn_light_on(
    Path((brightness, save_energy)): Path<(f64, bool)>,
    Extension(state): Extension<DeviceState>,
) -> Result<Json<LightOnResponse>, ResponseError> {
    let mut light = state.lock().await;
    light.turn_light_on(brightness, save_energy);

    Ok(Json(LightOnResponse {
        brightness: light.brightness,
        save_energy: light.save_energy,
    }))
}

async fn turn_light_off(Extension(state): Extension<DeviceState>) -> Result<(), ResponseError> {
    state.lock().await.turn_light_off();
    Ok(())
}

async fn toggle(Extension(state): Extension<DeviceState>) -> Result<(), ResponseError> {
    state.lock().await.toggle();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configuration for the turn light on route.
    let light_on_config = Route::put("/on/:brightness/:save-energy")
        .description("Turn light on.")
        .inputs([
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
    .add_action(DeviceAction::no_hazards(toggle_config, toggle))
    .state(DeviceState::new(device_light::Light::default()))
    .build();

    // Run a discovery service and the device on the server.
    AscotServer::new(device)
        .run_service(ServiceBuilder::new("mdns-sd"))?
        .run()
        .await
}
