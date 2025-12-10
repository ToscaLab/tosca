# `events-receiver-with-sse`

[![LICENSE][license badge]][license]

A receiver for events produced by `tosca` devices. It scans the network for
`tosca` devices, subscribes to their brokers to receive events, and displays
their data in real-time on a web page.

To send real-time updates from the server to the browser over a single `HTTP`
connection, it utilizes
[Server-Sent Events (SSE)](https://en.wikipedia.org/wiki/Server-sent_events)
technology.

This example demonstrates how to build a **Web Application** using the
`tosca-controller` APIs to manage events.

## Usage

To build the example:

```console
cargo build
```

To run the example:

```console
cargo run
```

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca/blob/master/LICENSE

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
