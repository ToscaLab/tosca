use std::error::Error;
use std::future::Future;

use ascot::response::ResponseKind;
use ascot::route::Route;

use axum::{
    body::{Body, Bytes},
    handler::Handler,
    http::header::HeaderName,
    response::{IntoResponse, Response},
};

use futures_core::TryStream;

use tokio::io::AsyncRead;
use tokio_util::io::ReaderStream;

use super::{error::ErrorResponse, DeviceAction, MandatoryAction};

/// A stream response.
///
/// Stream of data expressed as a sequence of bytes.
pub struct StreamResponse(Response);

impl StreamResponse {
    /// Creates a [`StreamResponse`] from headers and stream.
    #[inline]
    pub fn from_headers_stream<const N: usize, S>(
        headers: [(HeaderName, &str); N],
        stream: S,
    ) -> Self
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<Box<dyn Error + Sync + Send>>,
    {
        Self((headers, Body::from_stream(stream)).into_response())
    }

    /// Creates a [`StreamResponse`] from a stream.
    #[inline]
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<Box<dyn Error + Sync + Send>>,
    {
        Self(Body::from_stream(stream).into_response())
    }

    /// Creates a [`StreamResponse`] from headers and reader.
    #[inline]
    pub fn from_headers_reader<const N: usize, R>(
        headers: [(HeaderName, &str); N],
        reader: R,
    ) -> Self
    where
        R: AsyncRead + Send + 'static,
    {
        let stream = ReaderStream::new(reader);
        Self((headers, Body::from_stream(stream)).into_response())
    }

    /// Creates a [`StreamResponse`] from a reader.
    #[inline]
    pub fn from_reader<R>(reader: R) -> Self
    where
        R: AsyncRead + Send + 'static,
    {
        let stream = ReaderStream::new(reader);
        Self(Body::from_stream(stream).into_response())
    }
}

impl IntoResponse for StreamResponse {
    fn into_response(self) -> Response {
        self.0
    }
}

mod private {
    pub trait StreamTypeName<Args> {}
}

impl<F, Fut> private::StreamTypeName<()> for F
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<StreamResponse, ErrorResponse>> + Send,
{
}

macro_rules! impl_empty_type_name {
    (
        [$($ty:ident),*], $($last:ident)?
    ) => {
        impl<F, Fut, M, $($ty,)* $($last)?> private::StreamTypeName<(M, $($ty,)* $($last)?)> for F
        where
            F: FnOnce($($ty,)* $($last)?) -> Fut,
            Fut: Future<Output = Result<StreamResponse, ErrorResponse>> + Send,
            {
            }
    };
}
super::all_the_tuples!(impl_empty_type_name);

/// Creates a mandatory stateful [`DeviceAction`] with a [`StreamResponse`].
#[inline]
pub fn mandatory_stream_stateful<H, T, S>(
    route: Route,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, S> + private::StreamTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| {
        MandatoryAction::new(DeviceAction::stateful(
            route,
            ResponseKind::Stream,
            handler,
            state,
        ))
    }
}

/// Creates a stateful [`DeviceAction`] with a [`StreamResponse`].
#[inline]
pub fn stream_stateful<H, T, S>(route: Route, handler: H) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, S> + private::StreamTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |state: S| DeviceAction::stateful(route, ResponseKind::Stream, handler, state)
}

/// Creates a mandatory stateless [`DeviceAction`] with a [`StreamResponse`].
#[inline]
pub fn mandatory_stream_stateless<H, T, S>(
    route: Route,
    handler: H,
) -> impl FnOnce(S) -> MandatoryAction<false>
where
    H: Handler<T, ()> + private::StreamTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| {
        MandatoryAction::new(DeviceAction::stateless(
            route,
            ResponseKind::Stream,
            handler,
        ))
    }
}

/// Creates a stateless [`DeviceAction`] with a [`StreamResponse`].
#[inline]
pub fn stream_stateless<H, T, S>(route: Route, handler: H) -> impl FnOnce(S) -> DeviceAction
where
    H: Handler<T, ()> + private::StreamTypeName<T>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    move |_state: S| DeviceAction::stateless(route, ResponseKind::Stream, handler)
}
