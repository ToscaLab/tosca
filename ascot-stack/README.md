# `ascot-stack`

[![LICENSE][license badge]][license]

This Rust library contains a series of APIs and interfaces to interact with
stack-oriented devices. It is a `no_std` library.

Among its functionalities, it can:

- Create and manage HTTP `Rest` routes
- Define device data
- Define data collections
- Define route input parameters and hazards

## Building

To build with a `debug` profile run:

```console
cargo build
```

To build with `release` profile, which enables all time and memory
optimizations, run:

```console
cargo build --release
```

<!-- Links -->
[license]: https://github.com/SoftengPoliTo/ascot/blob/master/LICENSE-MIT

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
