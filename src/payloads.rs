use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::actions::ActionError;
use crate::device::DeviceInfo;
use crate::strings::ShortString;

/// Payload kinds for an action response.
#[derive(Serialize, Deserialize)]
pub enum PayloadKind {
    /// A short message to notify a receiver that an action terminated
    /// correctly.
    Ok,
    /// Serial data (i.e. JSON).
    ///
    /// This payload adds further information to an action response.
    Serial,
    /// Informative data to describe a device (i.e. JSON).
    ///
    /// This payload contains additional information on a device.
    Info,
    /// Stream of data expressed as a sequence of bytes.
    Stream,
}

/// An `Ok` payload notifies a receiver with a short message that a device
/// action has terminated correctly.
#[derive(Serialize, Deserialize)]
pub struct OkPayload(ShortString);

impl OkPayload {
    /// Creates an [`OkPayload`].
    #[inline(always)]
    pub fn ok() -> Self {
        // Ok payload message (64 byte-long).
        Self(ShortString::new("The action terminated correctly.").unwrap_or(ShortString::empty()))
    }
}

/// Serial payload.
///
/// This payload adds further information to an action response.
#[derive(Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct SerialPayload<T: DeserializeOwned> {
    // Serializable data.
    #[serde(flatten)]
    data: T,
}

impl<T: Serialize + DeserializeOwned> SerialPayload<T> {
    /// Creates a [`SerialPayload`].
    pub const fn new(data: T) -> Self {
        Self { data }
    }
}

/// Informative payload.
///
/// This payload contains additional information on a device.
#[derive(Serialize, Deserialize)]
pub struct InfoPayload {
    // Serializable data.
    #[serde(flatten)]
    data: DeviceInfo,
}

impl InfoPayload {
    /// Creates a [`InfoPayload`].
    pub const fn new(data: DeviceInfo) -> Self {
        Self { data }
    }
}

/// A payload containing information about an error occurred within an action.
///
/// It describes the kind of error, the cause, and optional information.
#[derive(Deserialize)]
pub struct ErrorPayload {
    /// Action error type.
    pub error: ActionError,
    /// Error description.
    pub description: ShortString,
    /// Information about an error.
    pub info: Option<ShortString>,
}

impl ErrorPayload {
    /// Creates an [`ErrorPayload`] with a specific [`ActionError`]
    /// and a description.
    #[inline]
    pub fn with_description(error: ActionError, description: &'static str) -> Self {
        Self {
            error,
            description: ShortString::infallible(description),
            info: None,
        }
    }

    /// Creates an [`ErrorPayload`] with a specific [`ActionError`], a
    /// description, and the effective error.
    #[inline]
    pub fn with_description_error(
        error: ActionError,
        description: &'static str,
        info: &str,
    ) -> Self {
        Self {
            error,
            description: ShortString::infallible(description),
            info: Some(ShortString::infallible(info)),
        }
    }

    /// Creates an [`ErrorPayload`] for invalid data with a description.
    #[inline]
    pub fn invalid_data(description: &'static str) -> Self {
        Self::with_description(ActionError::InvalidData, description)
    }

    /// Creates an [`ErrorPayload`] for invalid data with a description and
    /// the effective error.
    #[inline]
    pub fn invalid_data_with_error(description: &'static str, info: &str) -> Self {
        Self::with_description_error(ActionError::InvalidData, description, info)
    }

    /// Creates an [`ErrorPayload`] for an internal error with a description.
    #[inline]
    pub fn internal(description: &'static str) -> Self {
        Self::with_description(ActionError::Internal, description)
    }

    /// Creates an [`ErrorPayload`] for an internal error with a description and
    /// the effective error.
    #[inline(always)]
    pub fn internal_with_error(description: &'static str, info: &str) -> Self {
        Self::with_description_error(ActionError::Internal, description, info)
    }
}
