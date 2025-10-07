# `ascot-sensors`

[![LICENSE][license badge]][license]

A Rust library crate providing architecture-agnostic drivers for various sensors.

This crate currently includes drivers for:

- **AM312**: PIR motion sensor.
- **BH1750**: ambient light sensor.
- **DHT22**: temperature and humidity sensor.

All drivers are implemented using only the [`embedded-hal`] and
[`embedded-hal-async`] traits, making them compatible with any platform that
supports these abstractions.

## Features

You can enable only the sensors you need using Cargo features:

```toml
[dependencies]
ascot-sensors.version = "0.1.0"
ascot-sensors.default-features = false
ascot-sensors.features = ["bh1750", "dht22"] # only include needed drivers
```

<!-- Links -->
[license]: https://github.com/SoftengPoliTo/ascot/blob/master/LICENSE
[`embedded-hal`]: https://crates.io/crates/embedded-hal
[`embedded-hal-async`]: https://crates.io/crates/embedded-hal-async

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
