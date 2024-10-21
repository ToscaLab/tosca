mod empty;
mod serial;

use ascot_library::device::{DeviceError as AscotActionError, DeviceErrorKind};
use ascot_library::hazards::{Hazard, Hazards};

use ascot_library::route::{RestKind, Route, RouteHazards, RouteMode};

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};

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

pub struct ActionError(AscotActionError);

impl ActionError {
    /// Creates a new [`ActionError`] where the description of the error is
    /// passed as a string slice.
    #[inline(always)]
    pub fn from_str(kind: DeviceErrorKind, description: &str) -> Self {
        Self(AscotActionError::from_str(kind, description))
    }

    /// Creates an invalid data [`ActionError`].
    #[inline(always)]
    pub fn invalid_data(description: &str) -> Self {
        Self(AscotActionError::invalid_data(description))
    }

    /// Creates an internal error [`ActionError`].
    #[inline(always)]
    pub fn internal(description: &str) -> Self {
        Self(AscotActionError::internal(description))
    }

    /// Adds information about the error.
    #[inline(always)]
    pub fn info(self, info: &str) -> Self {
        Self(self.0.info(info))
    }
}

impl IntoResponse for ActionError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self.0)).into_response()
    }
}

mod private {
    pub trait Internal {
        fn internal_hazards(&self) -> &super::Hazards;
        fn data(self) -> (super::Router, super::RouteHazards);
    }
}

pub trait Action: private::Internal {
    /// Checks whether an [`Action`] misses a specific [`Hazard`].
    #[inline]
    fn miss_hazard(&self, hazard: Hazard) -> bool {
        !self.internal_hazards().contains(hazard)
    }

    /// Checks whether an [`Action`] misses the given [`Hazard`]s.
    #[inline]
    fn miss_hazards(&self, hazards: &'static [Hazard]) -> bool {
        !hazards
            .iter()
            .all(|hazard| self.internal_hazards().contains(*hazard))
    }

    /// Returns the [`Hazard`]s of an [`Action`].
    #[inline]
    fn hazards(&self) -> &Hazards {
        self.internal_hazards()
    }
}

#[derive(Debug)]
pub(crate) struct DeviceAction {
    // Route.
    pub(crate) route: Route,
    // Hazards.
    pub(crate) hazards: Hazards,
    // Router.
    pub(crate) router: Router,
}

impl DeviceAction {
    #[inline]
    pub(crate) fn hazard<H, T>(route: Route, handler: H, hazard: Hazard) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        DeviceAction::init(route, handler, Hazards::init(hazard))
    }

    #[inline]
    pub(crate) fn hazards<H, T>(route: Route, handler: H, hazards: &'static [Hazard]) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        DeviceAction::init(route, handler, Hazards::init_with_hazards(hazards))
    }

    #[inline(always)]
    pub(crate) fn data(self) -> (Router, RouteHazards) {
        (self.router, RouteHazards::new(self.route, self.hazards))
    }

    fn init<H, T>(mut route: Route, handler: H, hazards: Hazards) -> Self
    where
        H: Handler<T, ()>,
        T: 'static,
    {
        route.join_inputs(RouteMode::Linear, Some(":"));

        Self {
            hazards,
            router: Router::new().route(
                route.route(),
                match route.kind() {
                    RestKind::Get => axum::routing::get(handler),
                    RestKind::Put => axum::routing::put(handler),
                    RestKind::Post => axum::routing::post(handler),
                    RestKind::Delete => axum::routing::delete(handler),
                },
            ),
            route,
        }
    }
}

pub use empty::{EmptyAction, EmptyPayload};
pub use serial::{SerialAction, SerialPayload};
