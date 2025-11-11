use tosca::device::DeviceKind;
use tosca::hazards::Hazard;
use tosca::route::{LightOffRoute, LightOnRoute, Route};

use crate::device::Device;
use crate::error::{Error, Result};
use crate::responses::{BaseResponse, MandatoryResponse};

// The default main route for a light.
const LIGHT_MAIN_ROUTE: &str = "/light";

// Allowed hazards.
const ALLOWED_HAZARDS: &[Hazard] = &[Hazard::FireHazard, Hazard::ElectricEnergyConsumption];

/// A smart home light.
///
/// The default server main route for a light is `light`.
///
/// If a smart home needs more lights, each light **MUST** provide a
/// **different** main route in order to be registered.
pub struct Light<const M1: bool, const M2: bool, S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    // Internal device.
    device: Device<S>,
    // Turn light on response.
    turn_light_on: MandatoryResponse<M1>,
    // Turn light off response.
    turn_light_off: MandatoryResponse<M2>,
    // Allowed light hazards.
    allowed_hazards: &'static [Hazard],
}

impl Default for Light<false, false, ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Light<false, false, ()> {
    /// Creates a [`Light`] instance without a state.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl<S> Light<false, false, S>
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
            turn_light_on: MandatoryResponse::empty(),
            turn_light_off: MandatoryResponse::empty(),
            allowed_hazards: ALLOWED_HAZARDS,
        }
    }

    /// Turns on a light using a mandatory handler.
    ///
    /// **If not called, this method results in a compilation error.**
    pub fn turn_light_on(
        self,
        route: LightOnRoute,
        turn_light_on: impl FnOnce(Route, S) -> MandatoryResponse<false>,
    ) -> Light<true, false, S> {
        let turn_light_on = turn_light_on(route.into_route(), self.device.state.clone());

        Light {
            device: self.device,
            turn_light_on: MandatoryResponse::init(turn_light_on.base_response),
            turn_light_off: self.turn_light_off,
            allowed_hazards: ALLOWED_HAZARDS,
        }
    }
}

impl<S> Light<true, false, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Turns off a light using a mandatory handler.
    ///
    /// **If not called, this method results in a compilation error.**.
    pub fn turn_light_off(
        self,
        route: LightOffRoute,
        turn_light_off: impl FnOnce(Route, S) -> MandatoryResponse<false>,
    ) -> Light<true, true, S> {
        let turn_light_off = turn_light_off(route.into_route(), self.device.state.clone());

        Light {
            device: self.device,
            turn_light_on: self.turn_light_on,
            turn_light_off: MandatoryResponse::init(turn_light_off.base_response),
            allowed_hazards: ALLOWED_HAZARDS,
        }
    }
}

impl<S> Light<true, true, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Sets a new main route.
    #[must_use]
    #[inline]
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.device = self.device.main_route(main_route);
        self
    }

    /// Adds an additional [`BaseResponse`] to the [`Light`].
    ///
    /// # Errors
    ///
    /// Returns an error if one or more hazards are not allowed for
    /// the [`Light`] device.
    pub fn add_response(mut self, light_response: impl FnOnce(S) -> BaseResponse) -> Result<Self> {
        let base_response = light_response(self.device.state.clone());

        // Throws an error if any base response hazards are not part of
        // the allowed hazards for the Light.
        for hazard in base_response.hazards() {
            if !self.allowed_hazards.contains(hazard) {
                return Err(Error::device(
                    DeviceKind::Light,
                    format!("{hazard} hazard is not allowed for light"),
                ));
            }
        }

        self.device = self.device.add_base_response(base_response);

        Ok(self)
    }

    /// Adds an informational response to the [`Light`].
    #[must_use]
    pub fn add_info_response(
        mut self,
        light_info_response: impl FnOnce(S, ()) -> BaseResponse,
    ) -> Self {
        let base_response = light_info_response(self.device.state.clone(), ());

        self.device = self.device.add_base_response(base_response);

        self
    }

    /// Converts a [`Light`] into a [`Device`].
    pub fn into_device(self) -> Device<S> {
        self.device.add_mandatory_responses([
            self.turn_light_on.base_response,
            self.turn_light_off.base_response,
        ])
    }
}

#[cfg(test)]
mod tests {

    use tosca::hazards::Hazard;
    use tosca::parameters::Parameters;
    use tosca::route::Route;

    use axum::extract::{Json, State};

    use serde::{Deserialize, Serialize};

    use crate::devices::light::{LightOffRoute, LightOnRoute};
    use crate::responses::error::ErrorResponse;
    use crate::responses::ok::{
        OkResponse, mandatory_ok_stateful, mandatory_ok_stateless, ok_stateful, ok_stateless,
    };
    use crate::responses::serial::{
        SerialResponse, mandatory_serial_stateful, mandatory_serial_stateless, serial_stateful,
        serial_stateless,
    };

    use super::Light;

    #[derive(Clone)]
    struct LightState;

    #[derive(Deserialize)]
    struct Inputs {
        brightness: f64,
        #[serde(alias = "save-energy")]
        save_energy: bool,
    }

    #[derive(Serialize, Deserialize)]
    struct LightOnResponse {
        brightness: f64,
        #[serde(rename = "save-energy")]
        save_energy: bool,
    }

    async fn turn_light_on(
        State(_state): State<LightState>,
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialResponse<LightOnResponse>, ErrorResponse> {
        Ok(SerialResponse::new(LightOnResponse {
            brightness: inputs.brightness,
            save_energy: inputs.save_energy,
        }))
    }

    async fn turn_light_on_stateless(
        Json(inputs): Json<Inputs>,
    ) -> Result<SerialResponse<LightOnResponse>, ErrorResponse> {
        Ok(SerialResponse::new(LightOnResponse {
            brightness: inputs.brightness,
            save_energy: inputs.save_energy,
        }))
    }

    async fn turn_light_off(State(_state): State<LightState>) -> Result<OkResponse, ErrorResponse> {
        Ok(OkResponse::ok())
    }

    async fn turn_light_off_stateless() -> Result<OkResponse, ErrorResponse> {
        Ok(OkResponse::ok())
    }

    async fn toggle(State(_state): State<LightState>) -> Result<OkResponse, ErrorResponse> {
        Ok(OkResponse::ok())
    }

    async fn toggle_stateless() -> Result<OkResponse, ErrorResponse> {
        Ok(OkResponse::ok())
    }

    struct Routes {
        light_on: LightOnRoute,
        light_on_post: Route,
        light_off: LightOffRoute,
        toggle: Route,
    }

    #[inline]
    fn create_routes() -> Routes {
        Routes {
            light_on: LightOnRoute::put("On")
                .description("Turn light on.")
                .with_hazard(Hazard::ElectricEnergyConsumption)
                .with_parameters(
                    Parameters::new()
                        .rangef64("brightness", (0., 20., 0.1))
                        .bool("save-energy", false),
                ),

            light_on_post: Route::post("On Post", "/on-post")
                .description("Turn light on.")
                .with_hazard(Hazard::ElectricEnergyConsumption)
                .with_parameters(
                    Parameters::new()
                        .rangef64("brightness", (0., 20., 0.1))
                        .bool("save-energy", false),
                ),

            light_off: LightOffRoute::put("Off").description("Turn light off."),

            toggle: Route::put("Toggle", "/toggle")
                .description("Toggle a light.")
                .with_hazard(Hazard::ElectricEnergyConsumption),
        }
    }

    #[test]
    fn complete_with_state() {
        let routes = create_routes();

        Light::with_state(LightState {})
            .turn_light_on(routes.light_on, mandatory_serial_stateful(turn_light_on))
            .turn_light_off(routes.light_off, mandatory_ok_stateful(turn_light_off))
            .add_response(serial_stateful(routes.light_on_post, turn_light_on))
            .unwrap()
            .add_response(ok_stateful(routes.toggle, toggle))
            .unwrap()
            .into_device();
    }

    #[test]
    fn without_response_with_state() {
        let routes = create_routes();

        Light::with_state(LightState {})
            .turn_light_on(routes.light_on, mandatory_serial_stateful(turn_light_on))
            .turn_light_off(routes.light_off, mandatory_ok_stateful(turn_light_off))
            .into_device();
    }

    #[test]
    fn stateless_response_with_state() {
        let routes = create_routes();

        Light::with_state(LightState {})
            .turn_light_on(routes.light_on, mandatory_serial_stateful(turn_light_on))
            .turn_light_off(routes.light_off, mandatory_ok_stateful(turn_light_off))
            .add_response(serial_stateful(routes.light_on_post, turn_light_on))
            .unwrap()
            .add_response(ok_stateless(routes.toggle, toggle_stateless))
            .unwrap()
            .into_device();
    }

    #[test]
    fn complete_without_state() {
        let routes = create_routes();

        Light::new()
            .turn_light_on(
                routes.light_on,
                mandatory_serial_stateless(turn_light_on_stateless),
            )
            .turn_light_off(
                routes.light_off,
                mandatory_ok_stateless(turn_light_off_stateless),
            )
            .add_response(serial_stateless(
                routes.light_on_post,
                turn_light_on_stateless,
            ))
            .unwrap()
            .add_response(ok_stateless(routes.toggle, toggle_stateless))
            .unwrap()
            .into_device();
    }

    #[test]
    fn without_response_and_state() {
        let routes = create_routes();

        Light::new()
            .turn_light_on(
                routes.light_on,
                mandatory_serial_stateless(turn_light_on_stateless),
            )
            .turn_light_off(
                routes.light_off,
                mandatory_ok_stateless(turn_light_off_stateless),
            )
            .into_device();
    }
}
