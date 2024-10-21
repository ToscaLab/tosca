use core::any::type_name;
use core::future::Future;

use ascot_library::hazards::{Hazard, Hazards};
use ascot_library::payloads::SerialPayload as AscotSerialPayload;
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
    pub trait SerialTypeName<Args> {
        fn serial_type_name(&self) -> &'static str;
    }
}

impl<S, F, Fut> private::SerialTypeName<()> for F
where
    S: Serialize,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<SerialPayload<S>, ActionError>> + Send,
{
    fn serial_type_name(&self) -> &'static str {
        type_name::<Fut::Output>()
    }
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
                fn serial_type_name(&self) -> &'static str {
                    type_name::<Fut::Output>()
                }
            }
    };
}

super::all_the_tuples!(impl_serial_type_name);

/// An action with a serial payload.
pub struct SerialAction(DeviceAction);

impl Internal for SerialAction {
    #[inline(always)]
    fn internal_hazards(&self) -> &Hazards {
        &self.0.hazards
    }

    #[inline(always)]
    fn data(self) -> (Router, RouteHazards) {
        self.0.data()
    }
}

impl Action for SerialAction {}

impl SerialAction {
    /// Creates a new [`SerialAction`].
    #[inline]
    pub fn no_hazards<H, T>(route: Route, handler: H) -> Self
    where
        H: Handler<T, ()> + private::SerialTypeName<T>,
        T: 'static,
    {
        Self(DeviceAction::init(route, handler, Hazards::empty()))
    }

    /// Creates a new [`SerialAction`] with a single [`Hazard`].
    #[inline]
    pub fn with_hazard<H, T>(route: Route, handler: H, hazard: Hazard) -> Self
    where
        H: Handler<T, ()> + private::SerialTypeName<T>,
        T: 'static,
    {
        Self(DeviceAction::hazard(route, handler, hazard))
    }

    /// Creates a new [`SerialAction`] with [`Hazard`]s.
    #[inline]
    pub fn with_hazards<H, T>(route: Route, handler: H, hazards: &'static [Hazard]) -> Self
    where
        H: Handler<T, ()> + private::SerialTypeName<T>,
        T: 'static,
    {
        Self(DeviceAction::hazards(route, handler, hazards))
    }
}
