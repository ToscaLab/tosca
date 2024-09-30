# ascot-axum

This library crate provides a series of APIs to build an `axum` server which
represents the firmware of an IoT device.

Among its functionalities, it can interact send and receive data through
REST APIs.

The implemented devices can be found inside the [examples](./examples)
directory.

# Statically-linked firmware device

In order to build a statically-linked firmware, run the following command:

```bash
cargo build --package firmware_device [--release] --target=x86_64-unknown-linux-musl
```

where `firmware_device` is the name of the example to build, while `--release`
is an optional argument which enables all time and memory optimizations.

# Cross-compilation to aarch64 (ARM64) architecture

Install a binary named [cross](https://github.com/cross-rs/cross) which allow
to easily cross-compile Rust projects using Docker, without messing with
custom `Dockerfile`s.

```console
cargo install -f cross
```

In order to build a binary for `ARM64` architecture run:

```console
cross build --package firmware_device [--release] --target=aarch64-unknown-linux-musl
```

where `firmware_device` is the name of the example to build, while `--release`
is an optional argument which enables all time and memory optimizations.
