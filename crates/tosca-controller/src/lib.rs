//! The `tosca-controller` library crate offers a set of APIs to manage,
//! orchestrate, and interact with all `tosca`-compliant devices within a
//! network.
//!
//! A device is compliant with the `tosca` architecture if its firmware is built
//! using the `tosca` APIs designed for the relative microcontroller.
//!
//! The core functionalities of this crate include:
//!
//! - Discovering all devices within the network that are compliant with the
//!   `tosca` architecture
//! - Constructing and sending _REST_ requests to `tosca` devices to trigger
//!   one or more of their operations
//! - Defining security and privacy policies to allow or block requests
//! - Intercepting device events by subscribing to the brokers where
//!   they are published
//!
//! To optimize system resource usage, `tosca-controller` leverages `tokio` as
//! an asynchronous executor. This improves performance by allowing concurrent
//! execution of independent tasks. If the underlying machine is multi-threaded,
//! the performance boost is further amplified, as tasks are distributed across
//! multiple threads too.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// A controller for interacting with `tosca` devices.
pub mod controller;
/// A device definition along with its operations.
pub mod device;
/// A mechanism for discovering all `tosca` devices in a network.
pub mod discovery;
/// Error management.
pub mod error;
/// Events data.
pub mod events;
/// A privacy and security policy manager that determines whether `REST`
/// requests can be sent or blocked.
pub mod policy;
/// Request data and its methods.
pub mod request;
/// All supported methods and data for device responses.
pub mod response;

#[cfg(test)]
mod tests;
