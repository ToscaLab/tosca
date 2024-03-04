use ascot_library::device::DeviceKind;
use ascot_library::hazards::Hazard;

use axum::handler::Handler;

use crate::device::{Device, DeviceAction};
use crate::error::{Error, ErrorKind, Result};

// The default main route for a light.
const LIGHT_MAIN_ROUTE: &str = "/light";

/// A smart home light.
///
/// The default server main route for a light is `light`.
///
/// If a smart home needs more lights, each light **MUST** provide a
/// **different** main route in order to be registered.
pub struct Light<S>
where
    S: Clone + Send + Sync + 'static,
{
    // Main server route for light routes.
    main_route: &'static str,
    // Light state.
    state: Option<S>,
    // Device
    device: Device<S>,
}

impl<S> Light<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new [`Light`] instance.
    pub fn new<H, T, H1, T1>(
        turn_light_on: DeviceAction<H, T>,
        turn_light_off: DeviceAction<H1, T1>,
    ) -> Result<Self>
    where
        H: Handler<T, ()>,
        T: 'static,
        H1: Handler<T1, ()>,
        T1: 'static,
    {
        // Raise an error whether turn light_on does not contain a
        // fire hazard.
        if turn_light_on.miss_hazard(Hazard::FireHazard) {
            return Err(Error::new(
                ErrorKind::Light,
                "No fire hazard for the `turn_light_on` route",
            ));
        }

        // Create a new device.
        let device = Device::new(DeviceKind::Light)
            .add_action(turn_light_on)
            .add_action(turn_light_off);

        Ok(Self {
            main_route: LIGHT_MAIN_ROUTE,
            device,
            state: None,
        })
    }

    /// Sets a new main route.
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds an additional action for a [`Light`].
    pub fn add_action<H, T>(mut self, light_action: DeviceAction<H, T>) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        self.device = self.device.add_action(light_action);
        self
    }

    /// Sets a state for a [`Light`].
    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    /// Builds a new [`Device`].
    pub fn build(self) -> Device<S> {
        let mut device = self.device.main_route(self.main_route).finalize();
        device.state = self.state;
        device
    }
}
