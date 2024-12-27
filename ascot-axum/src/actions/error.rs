use ascot_library::actions::ActionError;
use ascot_library::payloads::ErrorPayload as AscotErrorPayload;

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// A payload containing information about an error occurred within an action.
///
/// It describes the kind of error, the cause, and optional information.
pub struct ErrorPayload(AscotErrorPayload);

impl ErrorPayload {
    /// Creates an [`ErrorPayload`] with a specific [`ActionError`] and
    /// a description.
    ///
    /// If an error occurs, an empty description is returned.
    #[inline]
    pub fn with_description(error: ActionError, description: &'static str) -> Self {
        Self(AscotErrorPayload::with_description(error, description))
    }

    /// Creates an [`ErrorPayload`] with a specific [`ActionError`], an
    /// error description, and additional information about the error.
    ///
    /// If this method fails for some internal reasons, empty description and
    /// information are returned.
    #[inline]
    pub fn with_description_error(
        error: ActionError,
        description: &'static str,
        info: impl std::error::Error,
    ) -> Self {
        Self(AscotErrorPayload::with_description_error(
            error,
            description,
            &info.to_string(),
        ))
    }

    /// Creates an [`ErrorPayload`] for invalid data with a description.
    ///
    /// If this method fails for some internal reasons, an empty description
    /// is returned.
    #[inline]
    pub fn invalid_data(description: &'static str) -> Self {
        Self::with_description(ActionError::InvalidData, description)
    }

    /// Creates an [`ErrorPayload`] for invalid data with a description and
    /// additional information about the error.
    ///
    /// If this method fails for some internal reasons, empty description and
    /// information are returned.
    #[inline]
    pub fn invalid_data_with_error(
        description: &'static str,
        error: impl std::error::Error,
    ) -> Self {
        Self::with_description_error(ActionError::InvalidData, description, error)
    }

    /// Creates an [`ErrorPayload`] for an internal error with a description.
    ///
    /// If this method fails for some internal reasons, an empty description
    /// is returned.
    #[inline]
    pub fn internal(description: &'static str) -> Self {
        Self::with_description(ActionError::Internal, description)
    }

    /// Creates an [`ErrorPayload`] for an internal error with a description and
    /// additional information about the error.
    ///
    /// If this method fails for some internal reasons, empty description and
    /// information are returned.
    #[inline(always)]
    pub fn internal_with_error(description: &'static str, error: impl std::error::Error) -> Self {
        Self::with_description_error(ActionError::Internal, description, error)
    }
}

impl IntoResponse for ErrorPayload {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self.0)).into_response()
    }
}
