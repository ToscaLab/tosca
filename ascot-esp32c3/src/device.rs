use core::result::Result;

use ascot_library::device::{DeviceData, DeviceKind, DeviceSerializer};
use ascot_library::hazards::Hazard;
use ascot_library::route::{Route, RouteConfigs};

use esp_idf_svc::http::server::{EspHttpConnection, Request, Response};
use esp_idf_svc::io::EspIOError;

// Default main route for a device.
const DEFAULT_MAIN_ROUTE: &str = "/device";

// An internal module to avoid declaring the trait as public.
mod internal {
    use super::{EspHttpConnection, EspIOError, Request, Response};
    // A trait to avoid writing over and over the same definition across
    // functions.
    pub trait ResponseTrait:
        for<'a, 'r> Fn(
            Request<&'a mut EspHttpConnection<'r>>,
        ) -> Result<Response<&'a mut EspHttpConnection<'r>>, EspIOError>
        + Send
        + 'static
    {
    }
}

impl<T> internal::ResponseTrait for T where
    T: for<'a, 'r> Fn(
            Request<&'a mut EspHttpConnection<'r>>,
        ) -> Result<Response<&'a mut EspHttpConnection<'r>>, EspIOError>
        + Send
        + 'static
{
}

/// Constructs a response which the server returns whenever the associated
/// action is being invoked.
pub struct ResponseBuilder<R: internal::ResponseTrait>(
    /// The closure called by a server whenever an action is called.
    /// It is responsible for showing the response content.
    pub R,
    /// Response content.
    pub &'static str,
);

/// A device action connects a server route with a device handler and,
/// optionally, with every possible hazards associated with the handler.
#[allow(clippy::module_name_repetitions)]
pub struct DeviceAction {
    // Route and hazards.
    pub(crate) route: Route,
    // Body.
    pub(crate) body: Option<Box<dyn Fn() -> Result<(), EspIOError> + Send + 'static>>,
    // Response-
    pub(crate) response: Box<dyn internal::ResponseTrait>,
    // Response content.
    pub(crate) content: &'static str,
}

impl DeviceAction {
    /// Creates a new [`DeviceAction`].
    #[must_use]
    #[inline]
    pub fn new<R: internal::ResponseTrait>(route: Route, response: ResponseBuilder<R>) -> Self {
        Self {
            route,
            body: None,
            response: Box::new(response.0),
            content: response.1,
        }
    }

    /// Checks whether a [`DeviceAction`] misses a specific [`Hazard`].
    #[must_use]
    #[inline]
    pub fn miss_hazard(&self, hazard: Hazard) -> bool {
        !self.route.hazards().contains(hazard)
    }

    /// Checks whether a [`DeviceAction`] misses the given [`Hazard`]s.
    #[must_use]
    #[inline]
    pub fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool {
        !hazards
            .iter()
            .all(|hazard| self.route.hazards().contains(*hazard))
    }

    /// Adds the body necessary to construct the response of an action.
    #[must_use]
    #[inline]
    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Fn() -> Result<(), EspIOError> + Send + 'static,
    {
        self.body = Some(Box::new(body));
        self
    }
}

// Build a device from a precise device.
pub(crate) trait DeviceBuilder {
    fn into_device(self) -> Device;
}

/// A general smart home device.
pub struct Device {
    // Kind.
    kind: DeviceKind,
    // Main device route.
    pub(crate) main_route: &'static str,
    // All device routes with their hazards and handlers.
    pub(crate) routes_data: Vec<DeviceAction>,
}

impl DeviceSerializer for Device {
    fn serialize_data(&self) -> DeviceData {
        let mut route_configs = RouteConfigs::empty();
        for route_data in &self.routes_data {
            route_configs.add(route_data.route.serialize_data());
        }

        DeviceData {
            kind: self.kind,
            main_route: self.main_route,
            route_configs,
        }
    }
}

impl Device {
    /// Creates a new [`Device`] instance.
    #[must_use]
    pub const fn new(kind: DeviceKind) -> Self {
        Self {
            kind,
            main_route: DEFAULT_MAIN_ROUTE,
            routes_data: Vec::new(),
        }
    }

    /// Sets a new main route.
    #[must_use]
    pub const fn main_route(mut self, main_route: &'static str) -> Self {
        self.main_route = main_route;
        self
    }

    /// Adds a [`DeviceAction`].
    #[must_use]
    pub fn add_action(mut self, device_action: DeviceAction) -> Self {
        self.routes_data.push(device_action);
        self
    }
}
