use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::device::DeviceInfo;
use crate::strings::ShortString;

/// Payload kinds for an action response.
#[derive(Serialize, Deserialize)]
pub enum PayloadKind {
    /// A short message to notify the receiver that an action terminated
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
