//! A library crate designed to manage, orchestrate, and interact with all
//! `tosca` devices within a network.
//!
//! Key functionalities include:
//!
//! - Discovering all `tosca` devices on the network
//! - Constructing and sending `REST` requests to issue commands to the
//!   discovered devices
//! - Enforcing security and privacy policies to permit or block requests
//!
//! Some APIs trigger tasks to perform their operations. This design breaks
//! large operations into smaller and independent tasks, making it easier
//! to manage system resources more efficiently.

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
