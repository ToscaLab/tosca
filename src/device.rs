use serde::{Deserialize, Serialize};

use crate::route::RouteConfigs;
use crate::{MiniString, ShortString};

/// Payload kinds.
#[derive(Serialize, Deserialize)]
pub enum PayloadKind {
    /// No data.
    Empty,
    /// Serial data (i.e JSON).
    Serial,
    /// Stream of data (bytes).
    Stream,
}

/// Empty payload structure.
#[derive(Serialize, Deserialize)]
pub struct EmptyPayload {
    // Empty payload description.
    description: &'static str,
}

impl EmptyPayload {
    /// Creates a new [`EmptyPayload`].
    pub fn new(description: &'static str) -> Self {
        Self { description }
    }
}

/// A payload associated with a device response.
#[derive(Serialize, Deserialize)]
pub struct DevicePayload(PayloadKind);

impl DevicePayload {
    /// Creates an empty [`DevicePayload`].
    pub const fn empty() -> Self {
        Self(PayloadKind::Empty)
    }

    /// Creates a serial [`DevicePayload`].
    pub const fn serial() -> Self {
        Self(PayloadKind::Serial)
    }

    /// Creates a stream [`DevicePayload`].
    pub const fn stream() -> Self {
        Self(PayloadKind::Stream)
    }
}

/// Kinds of errors for a device response.
#[derive(Serialize, Deserialize)]
pub enum DeviceErrorKind {
    /// Data needed to build a response are not correct because invalid or
    /// malformed.
    InvalidData,
    /// An internal error occurred on the device.
    Internal,
}

/// A device error response.
#[derive(Serialize, Deserialize)]
pub struct DeviceError {
    /// Device response error kind.
    pub kind: DeviceErrorKind,
    /// Error description.
    pub description: MiniString,
    /// Information about the error.
    pub info: Option<ShortString>,
}

impl DeviceError {
    /// Creates a new [`DeviceError`] where the description of the error is
    /// passed as a string slice.
    #[inline(always)]
    pub fn from_str(kind: DeviceErrorKind, description: &str) -> Self {
        Self {
            kind,
            description: MiniString::new(description).unwrap_or(MiniString::empty()),
            info: None,
        }
    }

    /// Creates a new [`DeviceError`] of kind [`DeviceErrorKind::InvalidData`].
    #[inline(always)]
    pub fn invalid_data(description: &str) -> Self {
        Self::from_str(DeviceErrorKind::InvalidData, description)
    }

    /// Creates a new [`DeviceError`] of kind [`DeviceErrorKind::Internal`].
    #[inline(always)]
    pub fn internal(description: &str) -> Self {
        Self::from_str(DeviceErrorKind::Internal, description)
    }

    /// Adds information about the error.
    #[inline(always)]
    pub fn info(mut self, info: &str) -> Self {
        self.info = Some(ShortString::new(info).unwrap_or(ShortString::empty()));
        self
    }
}

/// Device data.
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceData<'a> {
    /// Device kind.
    pub kind: DeviceKind,
    /// Device main route.
    #[serde(rename = "main route")]
    pub main_route: &'a str,
    #[serde(borrow)]
    /// All device route configurations.
    pub route_configs: RouteConfigs<'a>,
}

/// A device kind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeviceKind {
    /// Unknown.
    Unknown,
    /// Light.
    Light,
    /// Fridge.
    Fridge,
    /// Camera.
    Camera,
}

/// A trait to serialize device data.
pub trait DeviceSerializer {
    /// Serializes device data.
    fn serialize_data(&self) -> DeviceData;
}
