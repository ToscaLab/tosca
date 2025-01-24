# Ip-Camera Server

An Ip-Camera server usable from different operating systems for taking
screenshots and sending a video stream of the surrounding environment.

# Building

To build the camera binary, run the command:

```console
cargo build [--release]
```

where `--release` is an option which enables all time and memory optimizations
needed for having a binary usable in production. If the option is not inserted,
the binary will be built with all debug symbols inside.

## Cross-compilation to aarch64 (ARM64) architecture

Install a binary named [cross](https://github.com/cross-rs/cross) which allow
to easily cross-compile Rust projects using Docker, without messing with
custom `Dockerfile`s.

```console
cargo install -f cross
```

To build a binary for an `ARM64` architecture run:

```console
cross build [--release] --target=aarch64-unknown-linux-musl
```

where `--release` is an option which enables all time and memory optimizations
needed for having a binary usable in production. If the option is not inserted,
the binary will be built with all debug symbols inside.
