# Light Ascot Firmware

[![LICENSE][license badge]][license]

A light `Ascot` firmware to turn on and off the built-in LED of
an `ESP32-C3` board.

It implements an HTTP server which replies to `light/on` and
`light/off` **REST** requests responsible for changing the state of the
board built-in LED.
For each request, the server response with the status of the ongoing action.

The board can de discovered in a network through a `mDNS-SD` service.

## Building Prerequisites

Follow the [Prerequisites](https://github.com/esp-rs/esp-idf-template#prerequisites)
section contained in the `esp-idf-template` crate.

## Building

Before any kind of build, run `cargo clean` to remove old builds configurations,
and then run `cargo update` to update all dependencies.

To build this firmware with the `debug` profile run:

```console
cargo build
```

To build this firmware with a `release` profile which enables all time and
memory optimizations run:

```console
cargo build --release
```

## Running

To flash and run the firmware on an `ESP32-C3` board:

```console
cargo run [--release]
```

The optional `--release` parameter is recommended since it enables all
optimizations and makes the final firmware smaller.

## Board usage on WSL

Support for connecting `USB` devices is not natively available on [Windows
Subsystem for Linux (WSL)](https://learn.microsoft.com/en-us/windows/wsl/).

In order to use the `ESP32-C3` board with `WSL`, follow this
[guide](https://learn.microsoft.com/en-us/windows/wsl/connect-usb) and manually
connect the `USB` port used by the board to `WSL`.

## Usage Prerequisites

- The [sdkconfig.defaults](./sdkconfig.defaults) configuration file will
probably needs changes whenever the code is modified.
For example, the stack size might be increased or a specific option might be
added or removed.
- Rename `cfg.toml.example` to `cfg.toml` and fill it with your
Wi-Fi credentials: `SSID` and `PASSWORD`
- Connect the board to a laptop through a serial connection to visualize
the log
- Update the `ESP_IDF_VERSION` environment variable in the `.cargo/config.toml`
file if any problems arise
- Pin to a specific `nightly` version if more stability is requested
- For an over-the-air (OTA) update, it could be needed to change the size of the
partitions contained in the `partitions.csv` file. The offsets of each
partition have been automatically computed by the `espflash` command, invoked
during a `cargo run` instance. Their values have been later copied into the
`partitions.csv` file in order to show them explicitly. Before running
`cargo run` though, launch the `espflash erase-flash` command in order
to delete old partitions configurations which might be present on the
`ESP32-C3` board.

<!-- Links -->
[license]: https://github.com/SoftengPoliTo/ascot-firmware/blob/master/LICENSE-MIT

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
