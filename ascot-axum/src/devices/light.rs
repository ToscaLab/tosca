use ascot_library::device::DeviceKind;
use ascot_library::hazards::Hazard;

use crate::actions::{DeviceAction, MandatoryAction};
use crate::device::Device;
use crate::error::{Error, ErrorKind, Result};

// The default main route for a light.
const LIGHT_MAIN_ROUTE: &str = "/light";

// Mandatory actions hazards.
const TURN_LIGHT_ON: Hazard = Hazard::FireHazard;

// Allowed hazards.
const ALLOWED_HAZARDS: &[Hazard] = &[Hazard::FireHazard, Hazard::ElectricEnergyConsumption];

/// A smart home light.
///
/// The default server main route for a light is `light`.
///
/// If a smart home needs more lights, each light **MUST** provide a
/// **different** main route in order to be registered.
pub struct Light<M1 = (), M2 = (), S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    // Internal device.
    device: Device<S>,
    // Turn light on action.
    turn_light_on: MandatoryAction<M1>,
    // Turn light off action.
    turn_light_off: MandatoryAction<M2>,
    // Allowed light hazards.
    allowed_hazards: &'static [Hazard],
}

impl Default for Light<(), (), ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Light<(), (), ()> {
    /// Creates a [`Light`] instance without a state.
    #[inline(always)]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl<S> Light<(), (), S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a [`Light`] instance with a state.
    #[inline]
    pub fn with_state(state: S) -> Self {
        // Create a new device.
        let device = Device::init(DeviceKind::Light, state).main_route(LIGHT_MAIN_ROUTE);

        Self {
            device,
            turn_light_on: MandatoryAction::empty(),
            turn_light_off: MandatoryAction::empty(),
            allowed_hazards: ALLOWED_HAZARDS,
        }
    }

    /// Adds a turn light on action for a [`Light`].
    ///
    /// **This method is mandatory, if not called, a compilation
    /// error is raised.**.
    pub fn turn_light_on(
        self,
        turn_light_on: impl FnOnce(S) -> MandatoryAction<()>,
    ) -> Result<Light<u8, (), S>> {
        let turn_light_on = turn_light_on(self.device.state.clone());

        // Raise an error whether turn light_on does not contain a
        // fire hazard.
        if turn_light_on.device_action.miss_hazard(TURN_LIGHT_ON) {
            return Err(Error::new(
                ErrorKind::Light,
                "No fire hazard for the `turn_light_on` route",
            ));
        }

        Ok(Light {
            device: self.device,
            turn_light_on: MandatoryAction::init(turn_light_on.device_action),
            turn_light_off: self.turn_light_off,
            allowed_hazards: ALLOWED_HAZARDS,
        })
    }
}

impl<S> Light<u8, (), S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Adds a turn light off action for a [`Light`].
    ///
    ///
    /// **This method is mandatory, if not called, a compilation
    /// error is raised.**.
    pub fn turn_light_off(
        self,
        turn_light_off: impl FnOnce(S) -> MandatoryAction<()>,
    ) -> Light<u8, u8, S> {
        let turn_light_off = turn_light_off(self.device.state.clone());

        Light {
            device: self.device,
            turn_light_on: self.turn_light_on,
            turn_light_off: MandatoryAction::init(turn_light_off.device_action),
            allowed_hazards: ALLOWED_HAZARDS,
        }
    }
}

impl<S> Light<u8, u8, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Sets a new main route.
    #[inline]
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.device = self.device.main_route(main_route);
        self
    }

    /// Adds an additional action for a [`Light`].
    pub fn add_action(mut self, light_action: impl FnOnce(S) -> DeviceAction) -> Result<Self> {
        let light_action = light_action(self.device.state.clone());

        // Return an error if action hazards are not a subset of allowed hazards.
        for hazard in light_action.hazards().iter() {
            if !self.allowed_hazards.contains(hazard) {
                return Err(Error::new(
                    ErrorKind::Light,
                    format!("{hazard} hazard is not allowed for light"),
                ));
            }
        }

        self.device = self.device.add_device_action(light_action);

        Ok(self)
    }

    /// Converts a [`Light`] into a [`Device`].
    pub fn into_device(self) -> Device<S> {
        self.device
            .add_device_action(self.turn_light_on.device_action)
            .add_device_action(self.turn_light_off.device_action)
    }
}

#[cfg(test)]
mod tests {

    use ascot_library::hazards::Hazard;
    use ascot_library::input::Input;
    use ascot_library::route::{Route, RouteHazards};

    use axum::extract::{Json, State};

    use serde::{Deserialize, Serialize};

    use crate::actions::empty::{
        empty_stateful, empty_stateless, mandatory_empty_stateful, mandatory_empty_stateless,
        EmptyPayload,
    };
    use crate::actions::serial::{
        mandatory_serial_stateful, mandatory_serial_stateless, serial_stateful, serial_stateless,
        SerialPayload,
    };
    use crate::actions::ActionError;

    use super::Light;

    #[derive(Clone)]
    struct LightState;

    #[derive(Deserialize)]
    struct Inputs {
        brightness: f64,
        #[serde(alias = "save-energy")]
        save_energy: bool,
    }

    #[derive(Serialize)]
    struct LightOnResponse {
        brightness: f64,
        #[serde(rename = "save-energy")]
        save_energy: bool,
    }

    async fn turn_light_on(
        State(_state): State<LightState>,
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialPayload<LightOnResponse>, ActionError> {
        Ok(SerialPayload::new(LightOnResponse {
            brightness: inputs.brightness,
            save_energy: inputs.save_energy,
        }))
    }

    async fn turn_light_on_stateless(
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialPayload<LightOnResponse>, ActionError> {
        Ok(SerialPayload::new(LightOnResponse {
            brightness: inputs.brightness,
            save_energy: inputs.save_energy,
        }))
    }

    async fn turn_light_off(State(_state): State<LightState>) -> Result<EmptyPayload, ActionError> {
        Ok(EmptyPayload::new("Turn light off worked perfectly"))
    }

    async fn turn_light_off_stateless() -> Result<EmptyPayload, ActionError> {
        Ok(EmptyPayload::new("Turn light off worked perfectly"))
    }

    async fn toggle(State(_state): State<LightState>) -> Result<EmptyPayload, ActionError> {
        Ok(EmptyPayload::new("Toggle worked perfectly"))
    }

    async fn toggle_stateless() -> Result<EmptyPayload, ActionError> {
        Ok(EmptyPayload::new("Toggle worked perfectly"))
    }

    struct Routes {
        light_on_route: RouteHazards,
        light_on_post_route: RouteHazards,
        light_off_route: RouteHazards,
        toggle_route: RouteHazards,
    }

    #[inline]
    fn create_routes() -> Routes {
        Routes {
            light_on_route: RouteHazards::single_hazard(
                Route::put("/on").description("Turn light on.").inputs([
                    Input::rangef64("brightness", (0., 20., 0.1, 0.)),
                    Input::boolean("save-energy", false),
                ]),
                Hazard::FireHazard,
            ),
            light_on_post_route: RouteHazards::no_hazards(
                Route::post("/on").description("Turn light on.").inputs([
                    Input::rangef64("brightness", (0., 20., 0.1, 0.)),
                    Input::boolean("save-energy", false),
                ]),
            ),
            light_off_route: RouteHazards::no_hazards(
                Route::put("/off").description("Turn light off."),
            ),
            toggle_route: RouteHazards::no_hazards(
                Route::put("/toggle").description("Toggle a light."),
            ),
        }
    }

    #[test]
    fn complete_with_state() {
        let routes = create_routes();

        Light::with_state(LightState {})
            .turn_light_on(mandatory_serial_stateful(
                routes.light_on_route,
                turn_light_on,
            ))
            .unwrap()
            .turn_light_off(mandatory_empty_stateful(
                routes.light_off_route,
                turn_light_off,
            ))
            .add_action(serial_stateful(routes.light_on_post_route, turn_light_on))
            .unwrap()
            .add_action(empty_stateful(routes.toggle_route, toggle))
            .unwrap()
            .into_device();

        assert!(true);
    }

    #[test]
    fn without_action_with_state() {
        let routes = create_routes();

        Light::with_state(LightState {})
            .turn_light_on(mandatory_serial_stateful(
                routes.light_on_route,
                turn_light_on,
            ))
            .unwrap()
            .turn_light_off(mandatory_empty_stateful(
                routes.light_off_route,
                turn_light_off,
            ))
            .into_device();

        assert!(true);
    }

    #[test]
    fn stateless_action_with_state() {
        let routes = create_routes();

        Light::with_state(LightState {})
            .turn_light_on(mandatory_serial_stateful(
                routes.light_on_route,
                turn_light_on,
            ))
            .unwrap()
            .turn_light_off(mandatory_empty_stateful(
                routes.light_off_route,
                turn_light_off,
            ))
            .add_action(serial_stateful(routes.light_on_post_route, turn_light_on))
            .unwrap()
            .add_action(empty_stateless(routes.toggle_route, toggle_stateless))
            .unwrap()
            .into_device();

        assert!(true);
    }

    #[test]
    fn complete_without_state() {
        let routes = create_routes();

        Light::new()
            .turn_light_on(mandatory_serial_stateless(
                routes.light_on_route,
                turn_light_on_stateless,
            ))
            .unwrap()
            .turn_light_off(mandatory_empty_stateless(
                routes.light_off_route,
                turn_light_off_stateless,
            ))
            .add_action(serial_stateless(
                routes.light_on_post_route,
                turn_light_on_stateless,
            ))
            .unwrap()
            .add_action(empty_stateless(routes.toggle_route, toggle_stateless))
            .unwrap()
            .into_device();

        assert!(true);
    }

    #[test]
    fn without_action_and_state() {
        let routes = create_routes();

        Light::new()
            .turn_light_on(mandatory_serial_stateless(
                routes.light_on_route,
                turn_light_on_stateless,
            ))
            .unwrap()
            .turn_light_off(mandatory_empty_stateless(
                routes.light_off_route,
                turn_light_off_stateless,
            ))
            .into_device();

        assert!(true);
    }
}
