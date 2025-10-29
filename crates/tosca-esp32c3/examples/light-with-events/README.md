# Light Firmware with Event Manager

[![LICENSE][license badge]][license]

A `Tosca` light firmware for an `ESP32-C3` board that turns the built-in LED
on and off, sending notifications to an external process in response to changes
to internal sensors.

An event represents a change that can occur at any moment or at predefined
intervals.

The firmware hosts an `HTTP` server that manages the state of the board's
built-in LED via `REST` requests:

- `light/on` route turns the LED on
- `light/off` route turns the LED off
- `light/toggle` route toggles the LED

For each request, the server responds with the _final_ status of the operation
triggered by the request.

The board can be discovered by another node on the same network via
an `mDNS-SD` service using the default domain `tosca`.

## Build Process

To build the firmware:

```console
cargo build --release
```

To flash and run the firmware on an `ESP32-C3` board:

```console
cargo run --release
```

> [!IMPORTANT]
> Always use the release profile [--release] when building esp-hal crate.
  The dev profile can potentially be one or more orders of magnitude
  slower than release profile, and may cause issues with timing-senstive
  peripherals and/or devices.

## Board usage on WSL

Support for connecting `USB` devices is not natively available on [Windows
Subsystem for Linux (WSL)](https://learn.microsoft.com/en-us/windows/wsl/).

In order to use the `ESP32-C3` board with `WSL`, follow this
[guide](https://learn.microsoft.com/en-us/windows/wsl/connect-usb) and manually
connect the `USB` port used by the board to `WSL`.

## Usage Prerequisites

- Rename `cfg.toml.example` to `cfg.toml` and update it with your
Wi-Fi credentials (`SSID` and `PASSWORD`) as well as the broker configuration
(`ADDRESS` and `PORT`)
- Connect the board to a laptop via a `USB-C` cable to view the logs
- Pin the project to a specific `nightly` version for more stability, if needed

<!-- Links -->
[license]: https://github.com/ToscaLab/tosca/blob/master/LICENSE

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
