use core::future::Future;

use ascot_library::payloads::EmptyPayload as AscotEmptyPayload;
use ascot_library::route::RouteHazards;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{Deserialize, Serialize};

use super::{ActionError, DeviceAction, MandatoryAction};

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
    pub trait EmptyTypeName<Args> {}
}

impl<F, Fut> private::EmptyTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<EmptyPayload, ActionError>> + Send,
{
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
            }
    };
}
super::all_the_tuples!(impl_empty_type_name);

/// Creates a mandatory stateful [`DeviceAction`] with an [`EmptyPayload`].
#[inline(always)]
pub fn mandatory_empty_stateful<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, S> + private::EmptyTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| MandatoryAction::new(DeviceAction::stateful(route_hazards, handler, state))
}

/// Creates a stateful [`DeviceAction`] with an [`EmptyPayload`].
pub fn empty_stateful<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, S> + private::EmptyTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| DeviceAction::stateful(route_hazards, handler, state)
}

/// Creates a mandatory stateless [`DeviceAction`] with an [`EmptyPayload`].
#[inline(always)]
pub fn mandatory_empty_stateless<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, ()> + private::EmptyTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| MandatoryAction::new(DeviceAction::stateless(route_hazards, handler))
}

/// Creates a stateless [`DeviceAction`] with an [`EmptyPayload`].
pub fn empty_stateless<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, ()> + private::EmptyTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| DeviceAction::stateless(route_hazards, handler)
}
