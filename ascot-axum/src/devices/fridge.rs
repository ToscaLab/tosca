use ascot_library::device::DeviceKind;
use ascot_library::hazards::Hazard;
use ascot_library::route::Route;

use axum::{handler::Handler, Router};

use heapless::FnvIndexSet;

use crate::device::{Device, DeviceAction};
use crate::error::{Error, ErrorKind, Result};
use crate::MAXIMUM_ELEMENTS;

// The default main route for a fridge.
const FRIDGE_MAIN_ROUTE: &str = "/fridge";

// Mandatory fridge actions.
#[derive(Debug, PartialEq, Eq, Hash)]
enum Actions {
    IncreaseTemperature,
    DecreaseTemperature,
}

/// A smart home fridge.
///
/// The default server main route for a fridge is `fridge`.
///
/// If a smart home needs more fridges, each fridge **MUST** provide a
/// **different** main route in order to be registered.
pub struct Fridge<'a, S>
where
    S: Clone + Send + Sync + 'static,
{
    // Main server route for fridge routes.
    main_route: &'static str,
    // All fridge routes.
    routes: FnvIndexSet<Route<'a>, MAXIMUM_ELEMENTS>,
    // Router.
    router: Router,
    // Fridge state.
    state: Option<S>,
    // Allowed fridge hazards.
    allowed_hazards: FnvIndexSet<Hazard, MAXIMUM_ELEMENTS>,
    // Mandatory device actions.
    mandatory_actions: FnvIndexSet<Actions, MAXIMUM_ELEMENTS>,
}

impl<'a, S> Fridge<'a, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new [`Fridge`] instance.
    pub fn new() -> Self {
        let mut allowed_hazards = FnvIndexSet::new();
        let _ = allowed_hazards.insert(Hazard::FireHazard);

        let mut mandatory_actions = FnvIndexSet::new();
        let _ = mandatory_actions.insert(Actions::IncreaseTemperature);
        let _ = mandatory_actions.insert(Actions::DecreaseTemperature);

        Self {
            allowed_hazards,
            mandatory_actions,
            main_route: FRIDGE_MAIN_ROUTE,
            routes: FnvIndexSet::new(),
            router: Router::new(),
            state: None,
        }
    }

    /// Sets a new main route.
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds increase_temperature action.
    pub fn increase_temperature<H, T>(
        mut self,
        increase_temperature: DeviceAction<'a, H, T>,
    ) -> Result<Self>
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        // Raise an error whether increase_temperature does not contain
        // fire hazard and electric energy consumption hazards.
        if increase_temperature
            .miss_hazards(&[Hazard::ElectricEnergyConsumption, Hazard::FireHazard])
        {
            return Err(Error::new(
                ErrorKind::Fridge,
                "No fire and/or electric energy consumption hazards for the `increase_temperature` route",
            ));
        }

        self.router = self.router.merge(Device::<S>::build_router(
            increase_temperature.route.route,
            increase_temperature.route.config.rest_kind,
            increase_temperature.handler,
        ));

        let _ = self.routes.insert(increase_temperature.route);

        // Remove decrease_temperature action from the list of actions to set.
        self.mandatory_actions.remove(&Actions::IncreaseTemperature);

        Ok(self)
    }

    /// Adds decrease_temperature action.
    pub fn decrease_temperature<H, T>(
        mut self,
        decrease_temperature: DeviceAction<'a, H, T>,
    ) -> Result<Self>
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        // Raise an error whether decrease_temperature does not contain a
        // electric energy consumption hazard.
        if decrease_temperature.miss_hazard(Hazard::ElectricEnergyConsumption) {
            return Err(Error::new(
                ErrorKind::Fridge,
                "No electric energy consumption hazard for the `decrease_temperature` route",
            ));
        }

        self.router = self.router.merge(Device::<S>::build_router(
            decrease_temperature.route.route,
            decrease_temperature.route.config.rest_kind,
            decrease_temperature.handler,
        ));

        let _ = self.routes.insert(decrease_temperature.route);

        // Remove decrease_temperature action from the list of actions to set.
        self.mandatory_actions.remove(&Actions::DecreaseTemperature);

        Ok(self)
    }

    /// Adds an additional action for a [`Fridge`].
    pub fn add_action<H, T>(mut self, fridge_chainer: DeviceAction<'a, H, T>) -> Result<Self>
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        // Return an error if fridge_chainer hazards is not a subset of allowed hazards.
        if !fridge_chainer
            .route
            .hazards
            .is_subset(&self.allowed_hazards)
        {
            return Err(Error::new(
                ErrorKind::Fridge,
                "fridge_chainer has hazards that are not allowed for fridge",
            ));
        }

        self.router = self.router.merge(Device::<S>::build_router(
            fridge_chainer.route.route,
            fridge_chainer.route.config.rest_kind,
            fridge_chainer.handler,
        ));

        let _ = self.routes.insert(fridge_chainer.route);
        
        Ok(self)
    }

    /// Sets a state for a [`Fridge`].
    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    /// Builds a new [`Device`].
    pub fn build(self) -> Result<Device<'a, S>> {
        // Return an error if not all mandatory actions are set.
        if !self.mandatory_actions.is_empty() {
            return Err(Error::new(
                ErrorKind::Fridge,
                format!(
                    "The following mandatory actions are not set: {:?}",
                    self.mandatory_actions
                ),
            ));
        };

        let mut device = Device::new(DeviceKind::Fridge).main_route(self.main_route);

        device.routes = self.routes;
        device.router = self.router;

        Ok(device)
    }
}
