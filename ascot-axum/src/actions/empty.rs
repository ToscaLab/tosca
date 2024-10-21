use core::any::type_name;
use core::future::Future;

use ascot_library::hazards::{Hazard, Hazards};
use ascot_library::payloads::EmptyPayload as AscotEmptyPayload;
use ascot_library::route::{Route, RouteHazards};

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};

use serde::{Deserialize, Serialize};

use super::{private::Internal, Action, ActionError, DeviceAction};

/// An empty payload.
#[derive(Serialize, Deserialize)]
pub struct EmptyPayload(AscotEmptyPayload);

impl EmptyPayload {
    /// Creates a new [`EmptyPayload`].
    #[inline(always)]
    pub fn new(description: &str) -> Self {
        Self(AscotEmptyPayload::new(description))
    }
}

impl IntoResponse for EmptyPayload {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

mod private {
    pub trait EmptyTypeName<Args> {
        fn empty_type_name(&self) -> &'static str;
    }
}

impl<F, Fut> private::EmptyTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<EmptyPayload, ActionError>> + Send,
{
    fn empty_type_name(&self) -> &'static str {
        type_name::<Fut::Output>()
    }
}

macro_rules! impl_empty_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, Fut, M, $($ty,)* $($last)?> private::EmptyTypeName<(M, $($ty,)* $($last)?)> for F
        where
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<EmptyPayload, ActionError>> + Send,
            {
                fn empty_type_name(&self) -> &'static str {
                    type_name::<Fut::Output>()
                }
            }
    };
}
super::all_the_tuples!(impl_empty_type_name);

/// An action with an empty payload.
pub struct EmptyAction(DeviceAction);

impl Internal for EmptyAction {
    #[inline(always)]
    fn internal_hazards(&self) -> &Hazards {
        &self.0.hazards
    }
    #[inline(always)]
    fn data(self) -> (Router, RouteHazards) {
        self.0.data()
    }
}

impl Action for EmptyAction {}

impl EmptyAction {
    /// Creates a new [`EmptyAction`].
    #[inline]
    pub fn no_hazards<H, T>(route: Route, handler: H) -> Self
    where
        H: Handler<T, ()> + private::EmptyTypeName<T>,
        T: 'static,
    {
        Self(DeviceAction::init(route, handler, Hazards::empty()))
    }

    /// Creates a new [`EmptyAction`] with a single [`Hazard`].
    #[inline]
    pub fn with_hazard<H, T>(route: Route, handler: H, hazard: Hazard) -> Self
    where
        H: Handler<T, ()> + private::EmptyTypeName<T>,
        T: 'static,
    {
        Self(DeviceAction::hazard(route, handler, hazard))
    }

    /// Creates a new [`EmptyAction`] with [`Hazard`]s.
    #[inline]
    pub fn with_hazards<H, T>(route: Route, handler: H, hazards: &'static [Hazard]) -> Self
    where
        H: Handler<T, ()> + private::EmptyTypeName<T>,
        T: 'static,
    {
        Self(DeviceAction::hazards(route, handler, hazards))
    }
}
