use ascot_library::device::{DeviceData, DeviceKind, DeviceSerializer};
use ascot_library::route::{RouteConfigs, RoutesHazards};

use axum::Router;

use tracing::info;

use crate::actions::DeviceAction;

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

/// A general device.
#[derive(Debug)]
pub struct Device<S = ()>
where
    S: Clone + Send + Sync + 'static,
{
    // Main device route.
    pub(crate) main_route: &'static str,
    // Router.
    pub(crate) router: Router,
    // State.
    pub(crate) state: S,
    // Kind.
    kind: DeviceKind,
    // All device routes and their hazards.
    routes_hazards: RoutesHazards,
}

impl<S> DeviceSerializer for Device<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn serialize_data(&self) -> DeviceData {
        let mut route_configs = RouteConfigs::empty();
        for route_hazards in self.routes_hazards.iter() {
            info!(
                "Device route: [{}, \"{}{}\"]",
                route_hazards.route.kind(),
                self.main_route,
                route_hazards.route.route()
            );

            route_configs.add(route_hazards.serialize_data());
        }

        DeviceData {
            kind: self.kind,
            main_route: self.main_route,
            route_configs,
        }
    }
}

impl Default for Device<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Device<()> {
    /// Creates an unknown [`Device`] without a state.
    #[inline(always)]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl<S> Device<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates an unknown [`Device`] with state.
    #[inline]
    pub fn with_state(state: S) -> Self {
        Self::init(DeviceKind::Unknown, state)
    }

    /// Sets a new main route.
    pub const fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds an action to the [`Device`].
    #[inline]
    pub fn add_action(self, device_action: impl FnOnce(S) -> DeviceAction) -> Self {
        let device_action = device_action(self.state.clone());
        self.add_device_action(device_action)
    }

    #[inline]
    pub(crate) fn init(kind: DeviceKind, state: S) -> Self {
        Self {
            main_route: DEFAULT_MAIN_ROUTE,
            router: Router::new(),
            kind,
            routes_hazards: RoutesHazards::empty(),
            state,
        }
    }

    #[inline]
    pub(crate) fn add_device_action(mut self, device_action: DeviceAction) -> Self {
        self.router = self.router.merge(device_action.router);
        self.routes_hazards.add(device_action.route_hazards);
        self
    }
}
}
