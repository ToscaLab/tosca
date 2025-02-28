# `ascot-esp32c3`

[![LICENSE][license badge]][license]

A Rust library crate to create `Ascot` firmware for `ESP32-C3` boards.

It provides some APIs to connect the board to a router through Wi-Fi,
configure and run an HTTP server with some of the implemented discovery service.
For now, only a `mDNS-SD` service has been developed.

Some APIs have been implemented with the goal of defining well-known devices.
As now, only a `light` has been implemented in the [src/devices](./src/devices)
directory. These APIs are specifically thought to guide a developer in defining
a correct and a safe device, such as the addition of mandatory and optional 
actions which are possible on that device.

The implemented devices can be found inside the [examples](./examples)
directory.

## Building Prerequisites

Follow the [Prerequisites](https://github.com/esp-rs/esp-idf-template#prerequisites)
section contained in the `esp-idf-template` crate.

## Building

To build this crate with a `debug` profile run:

```console
cargo build
```

To build this crate with a `release` profile which enables all time and
memory optimizations run:

```console
cargo build --release
```

## Usage Prerequisites

Below some prerequisites for those projects interested in using this crate:

- The [sdkconfig.defaults](./sdkconfig.defaults) configuration file will
probably be different from firmware to firmware, so copy this file into your
project, and then change its values according to your needs.
For example, the stack size might be increased or some of these file options
might be added or removed.

## Building complete firmware devices

The directory [examples](./examples) contains firmware implemented with
the `ascot-esp32c3` crate. Each firmware is independent from another and it can
be moved in a separate repository.

Before any kind of build, run `cargo clean` to remove old builds configurations,
and then run `cargo update` to update all dependencies.

To build a firmware run:

```console
cd examples/[firmware directory name]
cargo build
```

It is necessary to enter the `examples/[firmware directory name]` to use the
`sdkconfig.defaults` file specific for that firmware.

To flash and run the firmware on an `ESP32-C3` board:

```console
cd examples/[firmware directory name]
cargo run [--release]
```

The optional `--release` parameter enables all optimizations and makes the
final firmware smaller.

<!-- Links -->
[license]: https://github.com/SoftengPoliTo/ascot/blob/master/LICENSE-MIT

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
