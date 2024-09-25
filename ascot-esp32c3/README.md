# Ascot library for `ESP32-C3` firmware

An `Ascot` library to create a firmware for an `ESP32-C3` board.

It contains APIs which allow to connect the board to a router through Wi-Fi,
configure and run an HTTP server with a `mDNS-SD` service.

This library also provides a thin-layer to the [ascot-library](../README) to
manage routes, errors and hazards.

## Building Prerequisites

Follow the [Prerequisites](https://github.com/esp-rs/esp-idf-template#prerequisites)
section contained in the `esp-idf-template` crate.

## Building

To build this library in `debug` mode:

```console
cargo build
```

To build this library in `release` mode, so that all optimizations are enabled:

```console
cargo build --release
```

## Usage Prerequisites

Some prerequisites for projects which are going to use this library:

- The [sdkconfig.defaults](./sdkconfig.defaults) configuration file will
probably be different from firmware to firmware, so copy this file into your
project and then change its values according to your needs.
For example, the stack size might be increased or a specific option might be
added or removed.

## Building devices

The directory [devices](../devices) contains firmware implemented with
the `ascot-esp32c3` crate. Each firmware is independent from another and it can
be moved in a separate repository.

To build a firmware run:

```console
cd devices/[firmware directory name]
cargo build
```

It is necessary to enter the `devices/[firmware directory name]` to use the
`sdkconfig.defaults` file specific for that firmware.

To flash and run the firmware on an `ESP32-C3` board:

```console
cd devices/[firmware directory name]
cargo run [--release]
```

The optional `--release` parameter enables all optimizations and makes the
final firmware smaller.
