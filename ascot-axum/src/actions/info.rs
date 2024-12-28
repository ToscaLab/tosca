use core::future::Future;

use ascot_library::device::DeviceInfo;
use ascot_library::payloads::InfoPayload as LibraryInfoPayload;
use ascot_library::route::Route;

use axum::{
    extract::Json,
    handler::Handler,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{Deserialize, Serialize};

use super::{error::ErrorPayload, DeviceAction};

/// An informative payload on a device.
#[derive(Serialize, Deserialize)]
pub struct InfoPayload(LibraryInfoPayload);

impl InfoPayload {
    /// Creates a [`InfoPayload`].
    pub const fn new(info: DeviceInfo) -> Self {
        Self(LibraryInfoPayload::new(info))
    }
}

impl IntoResponse for InfoPayload {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

mod private {
    pub trait InfoTypeName<Args> {}
}

impl<F, Fut> private::InfoTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<InfoPayload, ErrorPayload>> + Send,
{
}

macro_rules! impl_info_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, Fut, M, $($ty,)* $($last)?> private::InfoTypeName<(M, $($ty,)* $($last)?)> for F
        where
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<InfoPayload, ErrorPayload>> + Send,
            {
            }
    };
}
super::all_the_tuples!(impl_info_type_name);

/// Creates a stateful [`DeviceAction`] with a [`InfoPayload`].
pub fn info_stateful<H, T, S, I>(route: Route, handler: H) -> impl FnOnce(S, I) -> DeviceAction
where
    H: Handler<T, S> + private::InfoTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
    I: 'static,
{
    move |state: S, _: I| DeviceAction::stateful(route, handler, state)
}

/// Creates a stateless [`DeviceAction`] with a [`InfoPayload`].
pub fn info_stateless<H, T, S, I>(route: Route, handler: H) -> impl FnOnce(S, I) -> DeviceAction
where
    H: Handler<T, ()> + private::InfoTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
    I: 'static,
{
    move |_state: S, _: I| DeviceAction::stateless(route, handler)
}
