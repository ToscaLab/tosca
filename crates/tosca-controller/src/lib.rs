//! The `tosca-controller` library crate provides APIs for managing,
//! orchestrating, and interacting with devices within the same network,
//! all running firmware based on the `tosca` architecture.
//!
//! The core functionalities of this crate include:
//!
//! - Discovering all devices within the network that are compatible with the
//!   `tosca` architecture
//! - Constructing and sending _REST_ requests to `tosca` devices to trigger
//!   one or more of their operations
//! - Defining security and privacy policies to allow or block requests
//! - Intercepting device events by subscribing to the brokers where
//!   they are published
//!
//! This crate uses the `tokio` asynchronous executor to split its
//! functionalities into independent tasks, enhancing concurrency and enabling
//! more efficient use of systems resources.

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
