use ascot_library::device::{DeviceData, DeviceKind, DeviceSerializer};
use ascot_library::route::{RouteConfigs, RoutesHazards};

use axum::Router;

use crate::actions::Action;

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

/// A general smart home device.
#[derive(Debug)]
pub struct Device {
    // Main device route.
    pub(crate) main_route: &'static str,
    // Router.
    pub(crate) router: Router,
    // Kind.
    kind: DeviceKind,
    // All device routes and their hazards.
    routes_hazards: RoutesHazards,
}

impl DeviceSerializer for Device {
    fn serialize_data(&self) -> DeviceData {
        let mut route_configs = RouteConfigs::empty();
        for route_hazards in self.routes_hazards.iter() {
            route_configs.add(route_hazards.serialize_data());
        }

        DeviceData {
            kind: self.kind,
            main_route: self.main_route,
            route_configs,
        }
    }
}

impl Device {
    /// Creates an unknown [`Device`].
    #[inline]
    pub fn unknown() -> Self {
        Self::new(DeviceKind::Unknown)
    }

    /// Sets a new main route.
    pub const fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds an [`Action`] to the device.
    #[inline]
    pub fn add_action(mut self, device_chainer: impl Action) -> Self {
        let (router, route_hazards) = device_chainer.data();
        self.router = self.router.merge(router);
        self.routes_hazards.add(route_hazards);
        self
    }

    // Creates a new instance defining the DeviceKind.
    #[inline]
    pub(crate) fn new(kind: DeviceKind) -> Self {
        Self {
            main_route: DEFAULT_MAIN_ROUTE,
            router: Router::new(),
            kind,
            routes_hazards: RoutesHazards::empty(),
        }
    }
}
