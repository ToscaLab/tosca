//! A Rust library crate providing architecture-agnostic drivers for various sensors and devices.
//!
//! All drivers are implemented using only the [`embedded-hal`] and [`embedded-hal-async`] traits,
//! making them compatible with any platform that supports these abstractions.
//!
//! [`embedded-hal`]: https://crates.io/crates/embedded-hal
//! [`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![no_std]

#[cfg(feature = "am312")]
pub mod am312;

#[cfg(feature = "bh1750")]
pub mod bh1750;

#[cfg(feature = "dht22")]
pub mod dht22;

#[cfg(feature = "ds18b20")]
pub mod ds18b20;
