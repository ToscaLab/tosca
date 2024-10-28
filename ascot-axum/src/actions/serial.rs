use core::future::Future;

use ascot_library::hazards::Hazards;
use ascot_library::payloads::SerialPayload as AscotSerialPayload;
use ascot_library::route::RouteHazards;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};

use serde::{Deserialize, Serialize};

use super::{private::Internal, Action, ActionError, DeviceAction};

/// Serial payload structure.
#[derive(Serialize, Deserialize)]
pub struct SerialPayload<S: Serialize>(AscotSerialPayload<S>);

impl<S: Serialize> SerialPayload<S> {
    /// Creates a new [`SerialPayload`].
    pub const fn new(data: S) -> Self {
        Self(AscotSerialPayload::new(data))
    }
}

impl<S: Serialize> IntoResponse for SerialPayload<S> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

mod private {
    pub trait SerialTypeName<Args> {}
}

impl<S, F, Fut> private::SerialTypeName<()> for F
where
    S: Serialize,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<SerialPayload<S>, ActionError>> + Send,
{
}

macro_rules! impl_serial_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, S, Fut, M, $($ty,)* $($last)?> private::SerialTypeName<(M, $($ty,)* $($last)?)> for F
        where
            S: Serialize,
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<SerialPayload<S>, ActionError>> + Send,
            {
            }
    };
}

super::all_the_tuples!(impl_serial_type_name);

/// An action with a serial payload.
pub struct SerialAction(DeviceAction);

impl Internal for SerialAction {
    #[inline(always)]
    fn internal_hazards(&self) -> &Hazards {
        &self.0.route_hazards.hazards
    }

    #[inline(always)]
    fn data(self) -> (Router, RouteHazards) {
        self.0.data()
    }
}

impl Action for SerialAction {}

impl SerialAction {
    /// Creates a new [`SerialAction`] without a state.
    #[inline]
    pub fn stateless<H, T>(route_hazards: RouteHazards, handler: H) -> Self
    where
        H: Handler<T, ()> + private::SerialTypeName<T>,
        T: 'static,
    {
        Self(DeviceAction::stateless(route_hazards, handler))
    }

    /// Creates a new [`SerialAction`] with a state.
    #[inline]
    pub fn stateful<H, T, S>(route_hazards: RouteHazards, handler: H, state: S) -> Self
    where
        H: Handler<T, S> + private::SerialTypeName<T>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        Self(DeviceAction::stateful(route_hazards, handler, state))
    }
}
