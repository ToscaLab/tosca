//! A crate library for building Tosca devices firmware running on
//! operating systems.
//!
//! This crate has been conceived for those devices which require a great amount
//! of resources for their execution, in terms of computing times, memory size,
//! and connected components.
//!
//! As of now, only devices for `x86_64` and `ARM` hardware architectures have
//! been tested.
//!
//! All devices firmware are modelled on real use cases. Each device is
//! composed of a description and a set of operations with their own
//! hazards associated.
//!
//! A description is defined as a sequence of fields, such as
//! the device name, kind, and other information to set a secure
//! communication channel.
//!
//! When a controller makes a `REST` request to a server invoking a specific
//! route, one or more associated operations are performed on the device.
//!
//! Each route may have zero or more associated hazards.
//! If a route has no hazards, it could still pose unknown risks to the device.
//! In such cases, it is the responsibility of the controller to evaluate
//! whether the request should be blocked based on the potential hazards for the
//! device.
//!
//! This crate cannot determine the outcome of device operations at compile
//! time, as they are dependant on the runtime environment. As such, hazards are
//! informational only, aiding the controller in deciding about whether to allow
//! or block a request based on privacy policies.
//!
//! An `std` environment is mandatory to make a full usage of the provided
//! functionalities.

#![deny(unsafe_code)]
#![deny(missing_docs)]

/// All device kinds implementable in a firmware.
pub mod devices;

/// Methods for defining a device and its associated operations.
pub mod device;
/// Error handling.
pub mod error;
/// All responses kinds and their payloads.
pub mod responses;
/// Methods to define and run the server which represents the firmware.
pub mod server;
/// Methods to define and run the discovery service necessary to detect a
/// device in a network.
pub mod service {
    pub use super::services::{ServiceConfig, TransportProtocol};
}

/// Methods to parse requests and construct responses.
pub mod extract {
    pub use axum::extract::{FromRef, Json, Path, State};
    pub use axum::http::header;
}

mod mac;
mod services;
