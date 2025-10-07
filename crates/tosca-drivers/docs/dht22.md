# DHT22 - Temperature and Humidity Sensor

The **DHT22** is a digital temperature and humidity sensor
supported by the `tosca-drivers` crate via the `dht22` feature.

## Wiring

The following diagram shows how to connect a DHT22 sensor to an ESP32-C3 board.

![DHT22 wiring](./wiring/dht22.png)

| DHT22 Pin  | ESP32-C3 Pin |
|------------|--------------|
| VCC        | 3.3/5V       |
| GND        | GND          |
| DATA       | Any GPIO     |

## Usage

Enable the DHT22 driver in your `Cargo.toml`:

```toml
[dependencies]
tosca-drivers = { version = "0.1.0", features = ["dht22"] }
