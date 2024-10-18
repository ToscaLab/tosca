#![no_std]

// REMINDERS:
//
// - The `heapless` dependency can consume a lot of stack. Reduce the number
//   of elements in the stack structures if some issues arises.

// Maximum number of elements on stack for a data structure.
const MAXIMUM_ELEMENTS: usize = 8;

pub mod device;
pub mod error;
pub mod hazards;
pub mod input;
pub mod payloads;
pub mod route;
pub mod strings;

pub use error::{Error, ErrorKind};
