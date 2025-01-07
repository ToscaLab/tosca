use serde::{Deserialize, Serialize};

// REMINDER:
// 1. Parse an action response to verify whether it is an action error.
// 2. If it is not an error, evaluate the respective payload.

/// All possible action errors which led a device action to fail.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ActionError {
    /// Data needed for an action is not correct because invalid or malformed.
    #[serde(rename = "Invalid Data")]
    InvalidData,
    /// An internal error occurred on a device during the execution of an
    /// action.
    #[serde(rename = "Internal")]
    Internal,
}
