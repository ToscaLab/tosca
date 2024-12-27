use serde::{Deserialize, Serialize};

use crate::strings::ShortString;

// REMINDER:
// 1. Parse an action response to verify whether it is an action error.
// 2. If it is not an error, evaluate the respective payload.

/// All possible kind of errors for an action executed on a device.
#[derive(Serialize, Deserialize)]
pub enum ErrorKind {
    /// Data needed for an action is not correct because invalid or malformed.
    InvalidData,
    /// An internal error occurred on a device during the execution of an
    /// action.
    Internal,
}

/// An action error.
///
/// It identifies a failed operation within an action describing the kind of
/// error, the cause, and optional information.
#[derive(Deserialize)]
pub struct ActionError {
    /// Action error kind.
    pub kind: ErrorKind,
    /// Error description.
    pub description: ShortString,
    /// Information about an error.
    pub info: Option<ShortString>,
}
