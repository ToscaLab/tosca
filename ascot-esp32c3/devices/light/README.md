# Light Firmware

A light firmware to turn on and off the built-in LED of an `ESP32-C3` board.

It implements an HTTP server which replies to `light/on` and
`light/off` **REST** requests, in charge of modifying built-in LED state.
The server constructs a response containing the status of the requested action.

The board can de discovered in a trusted network through a `mDNS-SD` service.

## Building Prerequisites

Follow the [Prerequisites](https://github.com/esp-rs/esp-idf-template#prerequisites)
section contained in the `esp-idf-template` crate.

## Building

To build the code with the `debug` profile run:

```console
cargo build
```

To build the code with the `release` profile in order to enable some time and
size optimizations:

```console
cargo build --release
```

## Running

To flash and run the firmware on an `ESP32-C3` board:

```console
cargo run [--release]
```

The optional `--release` parameter enables all optimizations and makes the
final firmware smaller.

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
- Connect the board to a laptop through a serial connection to visualize logs
- Update the `ESP_IDF_VERSION` environment variable in the `.cargo/config.toml`
file if any problems arise
- Pin the `nightly` version to be more stable
