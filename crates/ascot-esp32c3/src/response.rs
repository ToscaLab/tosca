use alloc::borrow::Cow;
use alloc::format;
use alloc::vec::Vec;

use ascot::response::{
    ErrorKind, ErrorResponse as AscotErrorResponse, OkResponse as AscotOkResponse,
    SerialResponse as AscotSerialResponse,
};

use edge_http::io::server::Connection;
use edge_http::io::Error;

use embedded_io_async::{Read, Write};

use serde::{de::DeserializeOwned, Serialize};

/// Response headers.
pub struct Headers {
    status: u16,
    message: &'static str,
    content_type: &'static [(&'static str, &'static str)],
}

impl Headers {
    const fn not_found() -> Self {
        Self {
            status: 404,
            message: "Not Found",
            content_type: &[],
        }
    }

    const fn not_allowed() -> Self {
        Self {
            status: 405,
            message: "Method Not Allowed",
            content_type: &[],
        }
    }

    const fn json() -> Self {
        Self {
            status: 200,
            message: "Ok",
            content_type: &[("Content-Type", "application/json")],
        }
    }

    const fn json_error() -> Self {
        Self {
            status: 500,
            message: "Error",
            content_type: &[("Content-Type", "application/json")],
        }
    }
}

/// A response body.
pub struct Body(Cow<'static, [u8]>);

impl Body {
    const fn empty() -> Self {
        Self(Cow::Borrowed(&[]))
    }

    const fn static_ref(v: &'static [u8]) -> Self {
        Self(Cow::Borrowed(v))
    }

    #[inline]
    fn owned(v: Vec<u8>) -> Self {
        Self(Cow::Owned(v))
    }
}

#[inline]
fn json_to_vec<T: Serialize>(value: T) -> Vec<u8> {
    match serde_json::to_vec(&value) {
        Ok(value) => value,
        // TODO: A fallback response should be textual and intercepted by
        // the controller. Add a fallback response to Ascot.
        Err(e) => format!("{e:?}").as_bytes().into(),
    }
}

/// A server response.
pub struct Response {
    headers: Headers,
    body: Body,
}

impl From<Result<Response, Response>> for Response {
    #[inline]
    fn from(result: Result<Response, Response>) -> Response {
        match result {
            Ok(value) => value,
            Err(err) => err,
        }
    }
}

impl Response {
    /// Generates a [`Response`] with an `Ok` status and containing an
    /// [`ascot::response::OkResponse`].
    #[must_use]
    #[inline]
    pub fn ok() -> Response {
        let value = json_to_vec(AscotOkResponse::ok());
        Response::new(Headers::json(), Body::owned(value))
    }

    /// Generates a [`Response`] with an `Ok` status and containing a
    /// [`ascot::response::SerialResponse`].
    #[must_use]
    #[inline]
    pub fn serial<T: Serialize + DeserializeOwned>(value: T) -> Self {
        let value = json_to_vec(AscotSerialResponse::new(value));
        Response::new(Headers::json(), Body::owned(value))
    }

    /// Generates a [`Response`] with an `Ok` status and containing a
    /// [`ascot::response::SerialResponse`] derived from a given text input.
    #[must_use]
    #[inline]
    pub fn text(value: &str) -> Self {
        let value = Cow::Borrowed(value);
        let value = json_to_vec(AscotSerialResponse::new(value));
        Response::new(Headers::json(), Body::owned(value))
    }

    /// Generates a [`Response`] with an `Error` status and containing an
    /// [`ascot::response::ErrorResponse`].
    ///
    /// Requires specifying the [`ErrorKind`] kind and a general
    /// description.
    #[must_use]
    #[inline]
    pub fn error(error: ErrorKind, description: &str) -> Self {
        let value = json_to_vec(AscotErrorResponse::with_description(error, description));
        Response::new(Headers::json_error(), Body::owned(value))
    }

    /// Generates a [`Response`] with an `Error` status and containing a
    /// [`ascot::response::ErrorResponse`].
    ///
    /// Requires specifying the [`ErrorKind`] kind, a general error
    /// description, and optional information about the encountered error.
    #[must_use]
    #[inline]
    pub fn error_with_info(error: ErrorKind, description: &str, info: &str) -> Self {
        let value = json_to_vec(AscotErrorResponse::with_description_error(
            error,
            description,
            info,
        ));
        Response::new(Headers::json_error(), Body::owned(value))
    }

    /// An alias for the [`Self::error`] API, used to generate an invalid data
    /// [`Response`].
    ///
    /// Requires specifying a general error description.
    #[must_use]
    #[inline]
    pub fn invalid_data(description: &str) -> Self {
        Self::error(ErrorKind::InvalidData, description)
    }

    /// An alias for the [`Self::error`] API, used to generate an internal error
    /// [`Response`].
    ///
    /// Requires specifying a general error description.
    #[must_use]
    #[inline]
    pub fn internal(description: &str) -> Self {
        Self::error(ErrorKind::Internal, description)
    }

    /// An alias for the [`Self::error`] API, used to generate an internal error
    /// [`Response`] with some details about the error.
    ///
    /// Requires specifying a general error description and additional
    /// information about the encountered error.
    #[must_use]
    #[inline]
    pub fn internal_with_error(description: &str, info: &str) -> Self {
        Self::error_with_info(ErrorKind::Internal, description, info)
    }

    #[inline]
    pub(crate) fn json<T: Serialize>(value: &T) -> Self {
        Response::new(Headers::json(), Body::owned(json_to_vec(value)))
    }

    #[inline]
    pub(crate) async fn write<T, const N: usize>(
        self,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Error<T::Error>>
    where
        T: Read + Write,
    {
        self.write_from_ref(conn).await
    }

    #[inline]
    pub(crate) async fn write_from_ref<T, const N: usize>(
        &self,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Error<T::Error>>
    where
        T: Read + Write,
    {
        conn.initiate_response(
            self.headers.status,
            Some(self.headers.message),
            self.headers.content_type,
        )
        .await?;

        conn.write_all(&self.body.0).await
    }

    // TODO: Add this kind of response to the ascot crate. It is necessary that
    // a controller receives a response when a method is not correct or a route is
    // not found.

    pub(crate) const fn not_found() -> Self {
        Response::new(Headers::not_found(), Body::empty())
    }

    pub(crate) const fn not_allowed() -> Self {
        Response::new(
            Headers::not_allowed(),
            Body::static_ref("Method not allowed".as_bytes()),
        )
    }

    const fn new(headers: Headers, body: Body) -> Response {
        Self { headers, body }
    }
}
