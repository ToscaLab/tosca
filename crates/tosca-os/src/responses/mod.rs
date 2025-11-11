/// A response providing details about an error encountered during a
/// device operation.
pub mod error;
/// A response containing energy and economy data for a device.
pub mod info;
/// A response notifying the controller that
/// an operation completed successfully.
pub mod ok;
/// A response containing the data produced during a device operation.
pub mod serial;
/// Response to handle a stream of data as a sequence of bytes.
#[cfg(feature = "stream")]
pub mod stream;

use tosca::hazards::{Hazard, Hazards};
use tosca::parameters::ParametersData;
use tosca::response::ResponseKind;
use tosca::route::{RestKind, Route, RouteConfig};

use axum::{Router, handler::Handler};

use tracing::{error, info};

use std::fmt::Write;

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], );
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
    };
}

pub(super) use all_the_tuples;

fn build_get_route(route: &str, parameters: &ParametersData) -> String {
    let mut route = String::from(route);
    for (name, _) in parameters {
        // TODO: Consider returning `Option<String>`
        if let Err(e) = write!(route, "/{{{name}}}") {
            error!("Error in adding a path to a route : {e}");
            break;
        }
    }
    info!("Build GET route: {}", route);
    route
}

#[derive(Debug)]
/// A base response for a [`crate::device::Device`].
///
/// Any other response can be converted into a base response.
///
/// Designed to provide methods for checking the correctness of [`Hazard`]s.
pub struct BaseResponse {
    // Router.
    pub(crate) router: Router,
    // Route configuration.
    pub(crate) route_config: RouteConfig,
}

impl BaseResponse {
    /// Checks if the response does not contain the given [`Hazard`].
    #[must_use]
    #[inline]
    pub fn miss_hazard(&self, hazard: Hazard) -> bool {
        !self.route_config.data.hazards.contains(&hazard)
    }

    /// Checks if the response does not contain the given [`Hazard`]s.
    #[must_use]
    #[inline]
    pub fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool {
        !hazards
            .iter()
            .all(|hazard| self.route_config.data.hazards.contains(hazard))
    }

    /// Returns the [`Hazards`] associated with this response.
    #[must_use]
    #[inline]
    pub fn hazards(&self) -> &Hazards {
        &self.route_config.data.hazards
    }

    pub(crate) fn stateless<H, T>(route: Route, response_kind: ResponseKind, handler: H) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        Self::init(route, response_kind, handler, ())
    }

    pub(crate) fn stateful<H, T, S>(
        route: Route,
        response_kind: ResponseKind,
        handler: H,
        state: S,
    ) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        Self::init(route, response_kind, handler, state)
    }

    fn init<H, T, S>(route: Route, response_kind: ResponseKind, handler: H, state: S) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        let mut route_config = route.serialize_data();
        route_config.response_kind = response_kind;

        // Create the GET route for the axum architecture.
        let route = if matches!(route_config.rest_kind, RestKind::Get)
            && !route_config.data.parameters.is_empty()
        {
            &build_get_route(&route_config.data.path, &route_config.data.parameters)
        } else {
            route_config.data.path.as_ref()
        };

        let router = Router::new()
            .route(
                route,
                match route_config.rest_kind {
                    RestKind::Get => axum::routing::get(handler),
                    RestKind::Put => axum::routing::put(handler),
                    RestKind::Post => axum::routing::post(handler),
                    RestKind::Delete => axum::routing::delete(handler),
                },
            )
            .with_state(state);

        Self {
            router,
            route_config,
        }
    }
}

/// A mandatory [`BaseResponse`].
///
/// Marks a [`BaseResponse`] as mandatory for a device, meaning it must be
/// included.
pub struct MandatoryResponse<const SET: bool> {
    pub(crate) base_response: BaseResponse,
}

impl MandatoryResponse<false> {
    pub(crate) fn empty() -> Self {
        Self {
            base_response: BaseResponse {
                router: Router::new(),
                route_config: Route::get("", "").serialize_data(),
            },
        }
    }

    pub(super) const fn new(base_response: BaseResponse) -> Self {
        Self { base_response }
    }
}

impl MandatoryResponse<true> {
    /// Returns a reference to [`BaseResponse`].
    #[must_use]
    pub const fn base_response_as_ref(&self) -> &BaseResponse {
        &self.base_response
    }

    pub(crate) const fn init(base_response: BaseResponse) -> Self {
        Self { base_response }
    }
}

#[cfg(test)]
mod tests {
    use tosca::parameters::Parameters;

    use super::{Route, build_get_route};

    #[test]
    fn test_build_get_route() {
        let route = Route::get("Route", "/route")
            .description("A GET route.")
            .with_parameters(
                Parameters::new()
                    .rangeu64_with_default("rangeu64", (0, 20, 1), 5)
                    .rangef64("rangef64", (0., 20., 0.1)),
            )
            .serialize_data();

        assert_eq!(
            &build_get_route(&route.data.path, &route.data.parameters),
            "/route/{rangeu64}/{rangef64}"
        );
    }
}
