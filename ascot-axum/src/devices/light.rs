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
    }
}
