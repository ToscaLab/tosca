# `tosca`

[![Actions][actions badge]][actions]
[![Codecov][codecov badge]][codecov]
[![LICENSE][license badge]][license]

This repository is organized as a Cargo workspace that includes several crates.
The main crate is [tosca](./crates/tosca), a Rust library that provides a set of
APIs and interfaces to:

- Create and manage HTTP `Rest` routes
- Define a device and its methods
- Associate some hazards to a device

It can even run on an embedded system since it is a `no_std` library.

The [tosca-os](./crates/tosca-os) and [tosca-esp32c3](./crates/tosca-esp32c3)
are two Rust libraries which make use of the `tosca` library as dependency to
define the APIs for their respective architectures.

The `tosca-os` library is thought for firmware which run on operating systems.
In the [tosca-os/examples](./crates/tosca-os/examples) directory, a simple
device firmware has been implemented as examples: a [light](./crates/tosca-os/examples/light).

The `tosca-esp32c3` library is thought for firmware which run on a `ESP32-C3`
board.
In the [tosca-esp32c3/examples](./crates/tosca-esp32c3/examples) directory,
various device firmware have been implemented.

The `tosca-drivers` library provides architecture-agnostic drivers
for various sensors and devices.
All drivers are built on top of [`embedded-hal`] and [`embedded-hal-async`],
making them compatible with any supported platform.

## Building

To build the entire workspace with the `debug` profile, run the following
command from the root of the repository:

```console
cargo build
```

To build this workspace with a `release` profile, which enables all time and
memory optimizations, run:

```console
cargo build --release
```

To build only a specific crate, navigate to its corresponding subdirectory
inside [crates](./crates) and run the same build commands described above.

> [!NOTE]
> The `tosca-esp32c3` crate is not part of the workspace. It needs to be built
separately because it targets a specific architecture
(`riscv32imc-unknown-none-elf`), which necessitates a specialized build process.
The [per-package-target](https://doc.rust-lang.org/cargo/reference/unstable.html#per-package-target)
feature in Cargo is unstable, therefore only available on nightly toolchain.

<!-- Links -->
[actions]: https://github.com/ToscaLabs/tosca/actions
[codecov]: https://codecov.io/gh/ToscaLabs/tosca
[license]: https://github.com/ToscaLabs/tosca/blob/master/LICENSE
[`embedded-hal`]: https://crates.io/crates/embedded-hal
[`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async

<!-- Badges -->
[actions badge]: https://github.com/ToscaLabs/tosca/workflows/ci/badge.svg
[codecov badge]: https://codecov.io/gh/ToscaLabs/tosca/branch/master/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
