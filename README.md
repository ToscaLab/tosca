# `ascot`

[![LICENSE][license badge]][license]

This Rust library contains a series of APIs and interfaces to:

- Create and manage HTTP `Rest` routes
- Define a device and its methods
- Associate some hazards to a device

It can even run on an embedded system since it is a `no_std` library.

The [ascot-os](./ascot-os) and [ascot-esp32c3](./ascot-esp32c3) are two Rust
libraries which make use of the `ascot-library` as dependency to define the
APIs for their respective architectures.

The `ascot-os` library is thought for firmware which run on operating systems.
In the [ascot-os/examples](./ascot-os/examples) directory, two different
device firmware have been implemented as examples: a
[light](./ascot-os/examples/light) and a [fridge](./ascot-os/examples/fridge).

The `ascot-esp32c3` library is thought for firmware which run on a `ESP32-C3`
board.
In the [ascot-esp32c3/devices](./ascot-esp32c3/devices) directory, a simple
device firmware has been implemented as example: a [light](./ascot-esp32c3/devices/light).

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

<!-- Links -->
[license]: https://github.com/SoftengPoliTo/ascot/blob/master/LICENSE

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
