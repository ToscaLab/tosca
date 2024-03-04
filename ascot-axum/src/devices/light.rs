use ascot_library::device::DeviceKind;
use ascot_library::hazards::Hazard;
use ascot_library::route::Route;

use axum::{handler::Handler, Router};

use heapless::FnvIndexSet;

use crate::device::{Device, DeviceAction};
use crate::error::{Error, ErrorKind, Result};
use crate::MAXIMUM_ELEMENTS;

// The default main route for a light.
const LIGHT_MAIN_ROUTE: &str = "/light";

/// A smart home light.
///
/// The default server main route for a light is `light`.
///
/// If a smart home needs more lights, each light **MUST** provide a
/// **different** main route in order to be registered.
pub struct Light<'a, S>
where
    S: Clone + Send + Sync + 'static,
{
    // Main server route for light routes.
    main_route: &'static str,
    // All light routes.
    routes: FnvIndexSet<Route<'a>, MAXIMUM_ELEMENTS>,
    // Router.
    router: Router,
    // Light state.
    state: Option<S>,
}

impl<'a, S> Light<'a, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new [`Light`] instance.
    pub fn new<H, T, H1, T1>(
        turn_light_on: DeviceAction<'a, H, T>,
        turn_light_off: DeviceAction<'a, H1, T1>,
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

        let router = Router::new()
            .merge(Device::<S>::build_router(
                turn_light_on.route.route,
                turn_light_on.route.config.rest_kind,
                turn_light_on.handler,
            ))
            .merge(Device::<S>::build_router(
                turn_light_off.route.route,
                turn_light_off.route.config.rest_kind,
                turn_light_off.handler,
            ));

        let mut routes = FnvIndexSet::new();
        let _ = routes.insert(turn_light_on.route);
        let _ = routes.insert(turn_light_off.route);

        Ok(Self {
            main_route: LIGHT_MAIN_ROUTE,
            routes,
            router,
            state: None,
        })
    }

    /// Sets a new main route.
    pub fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds an additional action for a [`Light`].
    pub fn add_action<H, T>(mut self, light_chainer: DeviceAction<'a, H, T>) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        self.router = self.router.merge(Device::<S>::build_router(
            light_chainer.route.route,
            light_chainer.route.config.rest_kind,
            light_chainer.handler,
        ));

        let _ = self.routes.insert(light_chainer.route);
        self
    }

    /// Sets a state for a [`Light`].
    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    /// Builds a new [`Device`].
    pub fn build(self) -> Device<'a, S> {
        let mut device = Device::new(DeviceKind::Light).main_route(self.main_route);

        device.routes = self.routes;
        device.router = self.router;

        device
    }
}
