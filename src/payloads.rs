use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::device::DeviceInfo;
use crate::strings::MiniString;

/// Payload kinds for an action response.
#[derive(Serialize, Deserialize)]
pub enum PayloadKind {
    /// No data in an action response.
    ///
    /// This payload identifies an action which terminated in the correct way.
    Empty,
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
