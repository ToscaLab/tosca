use serde::{Deserialize, Serialize};

use crate::strings::{MiniString, ShortString};

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
#[derive(Serialize, Deserialize)]
pub struct ActionError {
    /// Action error kind.
    pub kind: ActionErrorKind,
    /// Error description.
    pub description: MiniString,
    /// Information about the error.
    pub info: Option<ShortString>,
}

impl ActionError {
    /// Creates a new [`ActionError`] where the description of the error is
    /// passed as a string slice.
    #[inline(always)]
    pub fn from_str(kind: ActionErrorKind, description: &str) -> Self {
        Self {
            kind,
            description: MiniString::new(description).unwrap_or(MiniString::empty()),
            info: None,
        }
    }

    /// Creates a new [`ActionError`] of kind [`ActionErrorKind::InvalidData`].
    #[inline(always)]
    pub fn invalid_data(description: &str) -> Self {
        Self::from_str(ActionErrorKind::InvalidData, description)
    }

    /// Creates a new [`ActionError`] of kind [`ActionErrorKind::Internal`].
    #[inline(always)]
    pub fn internal(description: &str) -> Self {
        Self::from_str(ActionErrorKind::Internal, description)
    }

    /// Adds information about the error.
    #[inline(always)]
    pub fn info(mut self, info: &str) -> Self {
        self.info = Some(ShortString::new(info).unwrap_or(ShortString::empty()));
        self
    }
}
