use serde::{Deserialize, Serialize};

use crate::MiniString;

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

/// Empty payload.
#[derive(Serialize, Deserialize)]
pub struct EmptyPayload {
    // Empty payload description (maximum 32 byte-long).
    description: MiniString,
}

impl EmptyPayload {
    /// Creates a new [`EmptyPayload`].
    #[inline(always)]
    pub fn new(description: &str) -> Self {
        Self {
            description: MiniString::new(description).unwrap_or(MiniString::empty()),
        }
    }
}

/// Serial payload structure.
#[derive(Serialize, Deserialize)]
pub struct SerialPayload<S: Serialize> {
    // Serializable data.
    #[serde(flatten)]
    data: S,
}

impl<S: Serialize> SerialPayload<S> {
    /// Creates a new [`SerialPayload`].
    pub const fn new(data: S) -> Self {
        Self { data }
    }
}

/// Stream payload structure.
pub struct StreamPayload<'a> {
    // Stream type.
    #[allow(dead_code)]
    stream_type: (&'a str, &'a str),
    // Stream headers.
    #[allow(dead_code)]
    headers: Option<&'a [(&'a str, &'a str)]>,
}

impl<'a> StreamPayload<'a> {
    /// Creates a new [`StreamPayload`].
    pub const fn new(stream_type: (&'a str, &'a str)) -> Self {
        Self {
            stream_type,
            headers: None,
        }
    }
}
