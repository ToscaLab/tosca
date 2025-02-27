# Fridge

[![LICENSE][license badge]][license]

## Building

Run the command

```console
cargo build
```

## Running the server

The server runs on `localhost` and listens to port `3000`. To make it run:

```console
cargo run
```

## REST API examples

Through `curl` or a web browser, it is possible to call the APIs which perform
some actions on a device.

**Increase temperature**

```console
curl -X PUT 127.0.0.1:3000/fridge/increase-temperature/1.0
```

```console
curl -X POST 127.0.0.1:3000/fridge/increase-temperature/1.0
```

**Decrease temperature**

```console
curl -X PUT 127.0.0.1:3000/fridge/decrease-temperature/1.0
```

At server startup, an initial message signalling its effective execution
is printed.

```
Starting server...
```

<!-- Links -->
[license]: https://github.com/SoftengPoliTo/ascot/blob/master/LICENSE-MIT

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
