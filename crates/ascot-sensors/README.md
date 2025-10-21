# `ascot-sensors`

[![LICENSE][license badge]][license]

A Rust library crate providing architecture-agnostic drivers for various sensors.

This crate currently includes drivers for:

- [**AM312**](./docs/am312.md): PIR motion sensor.
- [**BH1750**](./docs/bh1750.md): ambient light sensor.
- [**DHT22**](./docs/dht22.md): temperature and humidity sensor.
- [**DS18B20**](./docs/ds18b20.md): temperature sensor.

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
