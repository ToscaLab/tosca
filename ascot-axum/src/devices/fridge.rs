use ascot_library::device::DeviceKind;
use ascot_library::hazards::Hazard;

use crate::actions::{DeviceAction, MandatoryAction};
use crate::device::Device;
use crate::error::{Error, ErrorKind, Result};

// The default main route for a fridge.
const FRIDGE_MAIN_ROUTE: &str = "/fridge";

// Mandatory actions hazards.
const INCREASE_TEMPERATURE: &[Hazard] = &[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood];
const DECREASE_TEMPERATURE: Hazard = Hazard::ElectricEnergyConsumption;

// Allowed hazards.
const ALLOWED_HAZARDS: &[Hazard] = &[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood];

/// A smart home fridge.
///
/// The default server main route for a fridge is `fridge`.
///
/// If a smart home needs more fridges, each fridge **MUST** provide a
/// **different** main route in order to be registered.
pub struct Fridge<M1 = (), M2 = (), S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    // Device.
    device: Device<S>,
    // Increase temperature action.
    increase_temperature: MandatoryAction<M1>,
    // Decrease temperature action.
    decrease_temperature: MandatoryAction<M2>,
    // Allowed fridge hazards.
    allowed_hazards: &'static [Hazard],
}

impl Default for Fridge<(), (), ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Fridge<(), (), ()> {
    /// Creates a [`Fridge`] instance without a state.
    #[inline(always)]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl<S> Fridge<(), (), S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a [`Fridge`] instance with a state.
    #[inline]
    pub fn with_state(state: S) -> Self {
        // Create a new device.
        let device = Device::init(DeviceKind::Fridge, state).main_route(FRIDGE_MAIN_ROUTE);

        Self {
            device,
            increase_temperature: MandatoryAction::empty(),
            decrease_temperature: MandatoryAction::empty(),
            allowed_hazards: ALLOWED_HAZARDS,
        }
    }

    /// Adds an increase temperature action for a [`Fridge`].
    ///
    /// **This method is mandatory, if not called, a compilation
    /// error is raised.**.
    pub fn increase_temperature(
        self,
        increase_temperature: impl FnOnce(S) -> MandatoryAction<()>,
    ) -> Result<Fridge<u8, (), S>> {
        let increase_temperature = increase_temperature(self.device.state.clone());

        // Raise an error whether increase_temperature does not contain
        // electric energy consumption or spoiled food hazards.
        if increase_temperature
            .device_action
            .miss_hazards(INCREASE_TEMPERATURE)
        {
            return Err(Error::new(
                ErrorKind::Fridge,
                "No electric energy consumption or spoiled food hazards for the `increase_temperature` route",
            ));
        }

        Ok(Fridge {
            device: self.device,
            increase_temperature: MandatoryAction::init(increase_temperature.device_action),
            decrease_temperature: self.decrease_temperature,
            allowed_hazards: ALLOWED_HAZARDS,
        })
    }
}

impl<S> Fridge<u8, (), S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Adds a decrease temperature action for a [`Fridge`].
    ///
    /// **This method is mandatory, if not called, a compilation
    /// error is raised.**.
    pub fn decrease_temperature(
        self,
        decrease_temperature: impl FnOnce(S) -> MandatoryAction<()>,
    ) -> Result<Fridge<u8, u8, S>> {
        let decrease_temperature = decrease_temperature(self.device.state.clone());

        // Raise an error whether decrease_temperature does not contain
        // electric energy consumption hazard.
        if decrease_temperature
            .device_action
            .miss_hazard(DECREASE_TEMPERATURE)
        {
            return Err(Error::new(
                ErrorKind::Fridge,
                "No electric energy consumption hazard for the `decrease_temperature` route",
            ));
        }

        Ok(Fridge {
            device: self.device,
            increase_temperature: self.increase_temperature,
            decrease_temperature: MandatoryAction::init(decrease_temperature.device_action),
            allowed_hazards: ALLOWED_HAZARDS,
        })
    }
}

impl<S> Fridge<u8, u8, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Sets a new main route.
    #[inline]
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.device = self.device.main_route(main_route);
        self
    }

    /// Adds an additional action for a [`Fridge`].
    pub fn add_action(mut self, fridge_action: impl FnOnce(S) -> DeviceAction) -> Result<Self> {
        let fridge_action = fridge_action(self.device.state.clone());

        // Return an error if action hazards are not a subset of allowed hazards.
        for hazard in fridge_action.hazards().iter() {
            if !self.allowed_hazards.contains(hazard) {
                return Err(Error::new(
                    ErrorKind::Fridge,
                    format!("{hazard} hazard is not allowed for fridge"),
                ));
            }
        }

        self.device = self.device.add_device_action(fridge_action);

        Ok(self)
    }

    /// Adds an informative action for [`Fridge`].
    pub fn add_info_action(
        mut self,
        fridge_info_action: impl FnOnce(S, ()) -> DeviceAction,
    ) -> Self {
        let fridge_info_action = fridge_info_action(self.device.state.clone(), ());

        self.device = self.device.add_device_action(fridge_info_action);

        self
    }

    /// Converts a [`Fridge`] into a [`Device`].
    pub fn into_device(self) -> Device<S> {
        self.device
            .add_device_action(self.increase_temperature.device_action)
            .add_device_action(self.decrease_temperature.device_action)
    }
}

#[cfg(test)]
mod tests {

    use ascot_library::hazards::Hazard;
    use ascot_library::input::Input;
    use ascot_library::route::{Route, RouteHazards};

    use axum::extract::{Json, State};

    use serde::{Deserialize, Serialize};

    use crate::actions::serial::{
        mandatory_serial_stateful, mandatory_serial_stateless, serial_stateful, serial_stateless,
        SerialPayload,
    };
    use crate::actions::ActionError;

    use super::Fridge;

    #[derive(Clone)]
    struct FridgeState;

    #[derive(Deserialize)]
    struct IncreaseTemperature {
        increment: f64,
    }

    #[derive(Serialize)]
    struct ChangeTempResponse {
        temperature: f64,
    }

    async fn increase_temperature(
        State(_state): State<FridgeState>,
        Json(inputs): Json<IncreaseTemperature>,
    ) -> Result<SerialPayload<ChangeTempResponse>, ActionError> {
        Ok(SerialPayload::new(ChangeTempResponse {
            temperature: inputs.increment,
        }))
    }

    async fn increase_temperature_without_state(
        Json(inputs): Json<IncreaseTemperature>,
    ) -> Result<SerialPayload<ChangeTempResponse>, ActionError> {
        Ok(SerialPayload::new(ChangeTempResponse {
            temperature: inputs.increment,
        }))
    }

    #[derive(Deserialize)]
    struct DecreaseTemperature {
        decrement: f64,
    }

    async fn decrease_temperature(
        State(_state): State<FridgeState>,
        Json(inputs): Json<DecreaseTemperature>,
    ) -> Result<SerialPayload<ChangeTempResponse>, ActionError> {
        Ok(SerialPayload::new(ChangeTempResponse {
            temperature: inputs.decrement,
        }))
    }

    async fn decrease_temperature_without_state(
        Json(inputs): Json<DecreaseTemperature>,
    ) -> Result<SerialPayload<ChangeTempResponse>, ActionError> {
        Ok(SerialPayload::new(ChangeTempResponse {
            temperature: inputs.decrement,
        }))
    }

    struct Routes {
        increase_temp_route: RouteHazards,
        decrease_temp_route: RouteHazards,
        increase_temp_post_route: RouteHazards,
    }

    #[inline]
    fn create_routes() -> Routes {
        Routes {
            increase_temp_route: RouteHazards::with_hazards(
                Route::put("/increase-temperature")
                    .description("Increase temperature.")
                    .input(Input::rangef64("increment", (1., 4., 0.1, 2.))),
                &[Hazard::ElectricEnergyConsumption, Hazard::SpoiledFood],
            ),

            decrease_temp_route: RouteHazards::single_hazard(
                Route::put("/decrease-temperature")
                    .description("Decrease temperature.")
                    .input(Input::rangef64("decrement", (1., 4., 0.1, 2.))),
                Hazard::ElectricEnergyConsumption,
            ),

            increase_temp_post_route: RouteHazards::no_hazards(
                Route::post("/increase-temperature")
                    .description("Increase temperature.")
                    .input(Input::rangef64("increment", (1., 4., 0.1, 2.))),
            ),
        }
    }

    #[test]
    fn complete_with_state() {
        let routes = create_routes();

        Fridge::with_state(FridgeState {})
            .increase_temperature(mandatory_serial_stateful(
                routes.increase_temp_route,
                increase_temperature,
            ))
            .unwrap()
            .decrease_temperature(mandatory_serial_stateful(
                routes.decrease_temp_route,
                decrease_temperature,
            ))
            .unwrap()
            .add_action(serial_stateful(
                routes.increase_temp_post_route,
                increase_temperature,
            ))
            .unwrap()
            .into_device();

        assert!(true);
    }

    #[test]
    fn without_action_with_state() {
        let routes = create_routes();

        Fridge::with_state(FridgeState {})
            .increase_temperature(mandatory_serial_stateful(
                routes.increase_temp_route,
                increase_temperature,
            ))
            .unwrap()
            .decrease_temperature(mandatory_serial_stateful(
                routes.decrease_temp_route,
                decrease_temperature,
            ))
            .unwrap()
            .into_device();

        assert!(true);
    }

    #[test]
    fn stateless_action_with_state() {
        let routes = create_routes();

        Fridge::with_state(FridgeState {})
            .increase_temperature(mandatory_serial_stateful(
                routes.increase_temp_route,
                increase_temperature,
            ))
            .unwrap()
            .decrease_temperature(mandatory_serial_stateful(
                routes.decrease_temp_route,
                decrease_temperature,
            ))
            .unwrap()
            .add_action(serial_stateless(
                routes.increase_temp_post_route,
                increase_temperature_without_state,
            ))
            .unwrap()
            .into_device();

        assert!(true);
    }

    #[test]
    fn complete_without_state() {
        let routes = create_routes();

        Fridge::new()
            .increase_temperature(mandatory_serial_stateless(
                routes.increase_temp_route,
                increase_temperature_without_state,
            ))
            .unwrap()
            .decrease_temperature(mandatory_serial_stateless(
                routes.decrease_temp_route,
                decrease_temperature_without_state,
            ))
            .unwrap()
            .add_action(serial_stateless(
                routes.increase_temp_post_route,
                increase_temperature_without_state,
            ))
            .unwrap()
            .into_device();

        assert!(true);
    }

    #[test]
    fn without_action_and_state() {
        let routes = create_routes();

        Fridge::new()
            .increase_temperature(mandatory_serial_stateless(
                routes.increase_temp_route,
                increase_temperature_without_state,
            ))
            .unwrap()
            .decrease_temperature(mandatory_serial_stateless(
                routes.decrease_temp_route,
                decrease_temperature_without_state,
            ))
            .unwrap()
            .into_device();

        assert!(true);
    }
}
