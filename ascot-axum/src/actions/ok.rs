use core::future::Future;

use ascot_library::payloads::OkPayload as AscotOkPayload;
use ascot_library::route::RouteHazards;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{Deserialize, Serialize};

use super::{ActionError, DeviceAction, MandatoryAction};

/// An `Ok` payload.
#[derive(Serialize, Deserialize)]
pub struct OkPayload(AscotOkPayload);

impl OkPayload {
    /// Creates an [`OkPayload`].
    #[inline(always)]
    pub fn ok() -> Self {
        Self(AscotOkPayload::ok())
    }
}

impl IntoResponse for OkPayload {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

mod private {
    pub trait OkTypeName<Args> {}
}

impl<F, Fut> private::OkTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<OkPayload, ActionError>> + Send,
{
}

macro_rules! impl_ok_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, Fut, M, $($ty,)* $($last)?> private::OkTypeName<(M, $($ty,)* $($last)?)> for F
        where
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<OkPayload, ActionError>> + Send,
            {
            }
    };
}
super::all_the_tuples!(impl_ok_type_name);

/// Creates a mandatory stateful [`DeviceAction`] with an [`OkPayload`].
#[inline(always)]
pub fn mandatory_ok_stateful<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, S> + private::OkTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| MandatoryAction::new(DeviceAction::stateful(route_hazards, handler, state))
}

/// Creates a stateful [`DeviceAction`] with an [`OkPayload`].
pub fn ok_stateful<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, S> + private::OkTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| DeviceAction::stateful(route_hazards, handler, state)
}

/// Creates a mandatory stateless [`DeviceAction`] with an [`OkPayload`].
#[inline(always)]
pub fn mandatory_ok_stateless<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, ()> + private::OkTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| MandatoryAction::new(DeviceAction::stateless(route_hazards, handler))
}

/// Creates a stateless [`DeviceAction`] with an [`OkPayload`].
pub fn ok_stateless<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, ()> + private::OkTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| DeviceAction::stateless(route_hazards, handler)
}
