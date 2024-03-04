use core::marker::PhantomData;

use ascot_library::device::{DeviceData, DeviceKind, DeviceSerializer};
use ascot_library::hazards::Hazard;
use ascot_library::route::{RestKind, Route, RouteConfig};

use axum::{
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};

use heapless::FnvIndexSet;

use crate::MAXIMUM_ELEMENTS;

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

/// A [`Response`] error for a device.
pub struct ResponseError {
    /// Information about the error.
    pub info: String,
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        let body = format!("Error: {}", self.info);
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

/// A device action connects a server route with a device handler and,
/// optionally, with every possible hazards associated with the handler.
pub struct DeviceAction<'a, H, T>
where
    H: Handler<T, ()>,
    T: 'static,
{
    // Route.
    pub(crate) route: Route<'a>,
    // Handler.
    pub(crate) handler: H,
    // Handler type.
    handler_type: PhantomData<T>,
}

impl<'a, H, T> DeviceAction<'a, H, T>
where
    H: Handler<T, ()>,
    T: 'static,
{
    /// Creates a new [`DeviceAction`].
    pub fn no_hazards(route: Route<'a>, handler: H) -> Self {
        Self {
            route,
            handler,
            handler_type: PhantomData,
        }
    }

    /// Creates a new [`DeviceAction`] with a single [`Hazard`].
    pub fn with_hazard(mut route: Route<'a>, handler: H, hazard: Hazard) -> Self {
        let _ = route.hazards.insert(hazard);

        Self {
            route,
            handler,
            handler_type: PhantomData,
        }
    }

    /// Creates a new [`DeviceAction`] with [`Hazard`]s.
    pub fn with_hazards(mut route: Route<'a>, handler: H, hazards: &'static [Hazard]) -> Self {
        hazards.into_iter().for_each(|hazard| {
            let _ = route.hazards.insert(*hazard);
        });

        Self {
            route,
            handler,
            handler_type: PhantomData,
        }
    }

    /// Checks whether a [`DeviceAction`] misses a specific [`Hazard`].
    pub fn miss_hazard(&self, hazard: Hazard) -> bool {
        !self.route.hazards.contains(&hazard)
    }

    /// Checks whether a [`DeviceAction`] misses the given [`Hazard`]s.
    pub fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool {
        !hazards
            .iter()
            .all(|hazard| self.route.hazards.contains(hazard))
    }
}

/// A general smart home device.
#[derive(Debug)]
pub struct Device<'a, S>
where
    S: Clone + Send + Sync + 'static,
{
    // Kind.
    kind: DeviceKind,
    // Main device route.
    main_route: &'static str,
    // All device routes.
    pub(crate) routes: FnvIndexSet<Route<'a>, MAXIMUM_ELEMENTS>,
    // Router.
    pub(crate) router: Router,
    // Device state.
    pub(crate) state: Option<S>,
}

impl<'a, S> DeviceSerializer<'a> for Device<'a, S>
where
    S: Clone + Send + Sync + 'static,
{
    fn serialize_data(&self) -> DeviceData<'a> {
        DeviceData {
            kind: self.kind,
            main_route: self.main_route.into(),
            routes_configs: self
                .routes
                .into_iter()
                .map(|route| RouteConfig {
                    data: route.config.data.serialize_data(),
                    rest_kind: route.config.rest_kind,
                    hazards: route
                        .hazards
                        .iter()
                        .map(|hazard| hazard.serialize_data())
                        .collect(),
                })
                .collect(),
        }
    }
}

impl<'a, S> Device<'a, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new [`Device`] instance.
    pub fn new(kind: DeviceKind) -> Self {
        Self {
            kind,
            main_route: DEFAULT_MAIN_ROUTE,
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

    /// Adds a [`DeviceAction`].
    pub fn add_action<H, T>(mut self, device_chainer: DeviceAction<'a, H, T>) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        self.router = self.router.merge(Self::build_router(
            device_chainer.route.route,
            device_chainer.route.config.rest_kind,
            device_chainer.handler,
        ));
        let _ = self.routes.insert(device_chainer.route);
        self
    }

    /// Sets a device state.
    pub fn state(mut self, state: S) -> Self {
        self.state = Some(state);
        self
    }

    // Build a new router.
    pub(crate) fn build_router<H, T>(
        route_name: &'static str,
        rest_kind: RestKind,
        handler: H,
    ) -> Router
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        Router::new().route(
            route_name,
            match rest_kind {
                RestKind::Get => axum::routing::get(handler),
                RestKind::Put => axum::routing::put(handler),
                RestKind::Post => axum::routing::post(handler),
            },
        )
    }
}
