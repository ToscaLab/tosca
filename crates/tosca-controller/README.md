# `tosca-controller`

[![LICENSE][license badge]][license]

The `tosca-controller` library crate provides APIs for managing, orchestrating,
and interacting with devices within the same network, all running firmware
based on the `tosca` architecture.

The core functionalities of this crate include:

- Discovering all devices within the network that are compatible with the
  `tosca` architecture
- Constructing and sending _REST_ requests to `tosca` devices to trigger
  one or more of their operations
- Defining security and privacy policies to allow or block requests
- Intercepting device events by subscribing to the brokers where
  they are published

This crate uses the `tokio` asynchronous executor to split its functionalities
into independent tasks, enhancing concurrency and enabling more efficient 
use of systems resources.



<!-- Links -->
[license]: https://github.com/ToscaLab/tosca/blob/master/LICENSE

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
