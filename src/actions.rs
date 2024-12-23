use serde::{Deserialize, Serialize};

// REMINDER:
// 1. Parse an action response to verify whether it is an action error
// 2. Parse an action response according to the description contained in the
// definition of a route. If an error occurs during parsing, raise a
// parsing error.

/// Kinds of errors for an action executed on a device.
#[derive(Serialize, Deserialize)]
pub enum ActionErrorKind {
    /// Data needed for an action is not correct because deemed invalid or
    /// malformed.
    InvalidData,
    /// An internal error occurred on the device during the execution of an
    /// action.
    Internal,
}

/// An action error data.
#[derive(Deserialize)]
pub struct ActionError<'a> {
    /// Action error kind.
    pub kind: ActionErrorKind,
    /// Error description.
    pub description: &'a str,
    /// Information about the error.
    pub info: Option<&'a str>,
}
