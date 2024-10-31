use serde::{Deserialize, Serialize};

use crate::device::DeviceInfo;
use crate::strings::MiniString;

/// Payload kinds.
#[derive(Serialize, Deserialize)]
pub enum PayloadKind {
    /// No data.
    Empty,
    /// Serial data (i.e. JSON).
    Serial,
    /// Informative data on a device (i.e. JSON).
    Info,
    /// Stream of data (bytes).
    Stream,
}

/// Empty payload with only a description.
#[derive(Serialize, Deserialize)]
pub struct EmptyPayload {
    // Empty payload description (maximum 32 byte-long).
    description: MiniString,
}

impl EmptyPayload {
    /// Creates an [`EmptyPayload`].
    #[inline(always)]
    pub fn new(description: &str) -> Self {
        Self {
            description: MiniString::new(description).unwrap_or(MiniString::empty()),
        }
    }
}

/// Serial payload.
#[derive(Serialize, Deserialize)]
pub struct SerialPayload<S: Serialize> {
    // Serializable data.
    #[serde(flatten)]
    data: S,
}

impl<S: Serialize> SerialPayload<S> {
    /// Creates a [`SerialPayload`].
    pub const fn new(data: S) -> Self {
        Self { data }
    }
}

/// Informative payload.
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

/// Stream payload.
pub struct StreamPayload<'a> {
    // Stream type.
    #[allow(dead_code)]
    stream_type: (&'a str, &'a str),
    // Stream headers.
    #[allow(dead_code)]
    headers: Option<&'a [(&'a str, &'a str)]>,
}

impl<'a> StreamPayload<'a> {
    /// Creates a [`StreamPayload`].
    pub const fn new(stream_type: (&'a str, &'a str)) -> Self {
        Self {
            stream_type,
            headers: None,
        }
    }
}
