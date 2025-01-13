#![allow(clippy::module_name_repetitions)]

use core::future::Future;

use ascot_library::response::SerialResponse as AscotSerialResponse;
use ascot_library::route::Route;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{error::ErrorResponse, DeviceAction, MandatoryAction};

/// Serial response.
///
/// This response provides more detailed information about an action.
#[derive(Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct SerialResponse<T: DeserializeOwned>(AscotSerialResponse<T>);

impl<T: Serialize + DeserializeOwned> SerialResponse<T> {
    /// Creates a new [`SerialResponse`].
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self(AscotSerialResponse::new(data))
    }
}

impl<T: Serialize + DeserializeOwned> IntoResponse for SerialResponse<T> {
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
    Fut: Future<Output = Result<SerialResponse<T>, ErrorResponse>> + Send,
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
            Fut: Future<Output = Result<SerialResponse<T>, ErrorResponse>> + Send,
            {
            }
    };
}

super::all_the_tuples!(impl_serial_type_name);

/// Creates a mandatory stateful [`DeviceAction`] with a [`SerialResponse`].
#[inline]
pub fn mandatory_serial_stateful<H, T, S>(
    route: Route,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, S> + private::SerialTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| MandatoryAction::new(DeviceAction::stateful(route, handler, state))
}

/// Creates a stateful [`DeviceAction`] with a [`SerialResponse`].
#[inline]
pub fn serial_stateful<H, T, S>(route: Route, handler: H) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, S> + private::SerialTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| DeviceAction::stateful(route, handler, state)
}

/// Creates a mandatory stateless [`DeviceAction`] with a [`SerialResponse`].
#[inline]
pub fn mandatory_serial_stateless<H, T, S>(
    route: Route,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, ()> + private::SerialTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| MandatoryAction::new(DeviceAction::stateless(route, handler))
}

/// Creates a stateless [`DeviceAction`] with a [`SerialResponse`].
#[inline]
pub fn serial_stateless<H, T, S>(route: Route, handler: H) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, ()> + private::SerialTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| DeviceAction::stateless(route, handler)
}
