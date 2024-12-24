use core::future::Future;

use ascot_library::payloads::SerialPayload as AscotSerialPayload;
use ascot_library::route::RouteHazards;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{ActionError, DeviceAction, MandatoryAction};

/// Serial payload structure.
#[derive(Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct SerialPayload<T: DeserializeOwned>(AscotSerialPayload<T>);

impl<T: Serialize + DeserializeOwned> SerialPayload<T> {
    /// Creates a new [`SerialPayload`].
    pub const fn new(data: T) -> Self {
        Self(AscotSerialPayload::new(data))
    }
}

impl<T: Serialize + DeserializeOwned> IntoResponse for SerialPayload<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

mod private {
    pub trait SerialTypeName<Args> {}
}

impl<T, F, Fut> private::SerialTypeName<()> for F
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<SerialPayload<T>, ActionError>> + Send,
{
}

macro_rules! impl_serial_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, T, Fut, M, $($ty,)* $($last)?> private::SerialTypeName<(M, $($ty,)* $($last)?)> for F
        where
            T: Serialize + DeserializeOwned,
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<SerialPayload<T>, ActionError>> + Send,
            {
            }
    };
}

super::all_the_tuples!(impl_serial_type_name);

/// Creates a mandatory stateful [`DeviceAction`] with a [`SerialPayload`].
#[inline(always)]
pub fn mandatory_serial_stateful<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, S> + private::SerialTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| MandatoryAction::new(DeviceAction::stateful(route_hazards, handler, state))
}

/// Creates a stateful [`DeviceAction`] with a [`SerialPayload`].
#[inline(always)]
pub fn serial_stateful<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, S> + private::SerialTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| DeviceAction::stateful(route_hazards, handler, state)
}

/// Creates a mandatory stateless [`DeviceAction`] with a [`SerialPayload`].
#[inline(always)]
pub fn mandatory_serial_stateless<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, ()> + private::SerialTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| MandatoryAction::new(DeviceAction::stateless(route_hazards, handler))
}

/// Creates a stateless [`DeviceAction`] with a [`SerialPayload`].
#[inline(always)]
pub fn serial_stateless<H, T, S>(
    route_hazards: RouteHazards,
    handler: H,
) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, ()> + private::SerialTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| DeviceAction::stateless(route_hazards, handler)
}
