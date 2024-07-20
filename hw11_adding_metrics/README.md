# [Homework 11](https://robot-dreams-rust.mag.wiki/18-metrics-in-rust/index.html#homework)

## Description

In this assignment, you will add monitoring capabilities to the server part of your chat application using Prometheus. Monitoring is a crucial aspect of maintaining and understanding the health and performance of applications, especially in production environments.

### Integrate Prometheus:
- [X] Add Prometheus to your chat application's server.
- [X] Ensure that Prometheus is set up correctly to gather metrics from your server.

### Metrics Implementation:
- [X] Implement at least one metric using Prometheus. At a minimum, add a counter to track the number of messages sent through your server.
- [ ] Optionally, consider adding a gauge to monitor the number of active connections to your server. This can provide insights into user engagement and server load.

### Metrics Endpoint:
- [X] Set up an endpoint within your server application to expose these metrics to Prometheus. This typically involves creating a `/metrics` endpoint.
- [X] Ensure that the endpoint correctly exposes the metrics in a format that Prometheus can scrape.

> [!NOTE] 
> Typically, this means using the TextEncoder: https://docs.rs/prometheus/0.13.3/prometheus/struct.TextEncoder.html
> You can refer to the Hyper example: https://github.com/tikv/rust-prometheus/blob/master/examples/example_hyper.rs

### Documentation and Testing:
- [X] Document the new metrics feature in your README.md, including how to access the metrics endpoint and interpret the exposed data.
- [X] Test to make sure that the metrics are accurately recorded and exposed. Verify that Prometheus can successfully scrape these metrics from your server.

## Server and Client Usage:

### Server
> [!WARNING]
> Ensure you have sqlite installed on your local machine. It is required by the server.

Launching the server is quite simple, from the packages root directory:
    `RUST_LOG=<log level> cargo run --bin server <listening ip> <listening port>`
e.g.:
    `RUST_LOG=debug cargo run --bin server 127.0.0.1 8080`

### Prometheus
> [!WARNING]
> Ensure you have prometheus installed on your local machine before attempting to use it.

The server collects metrics on the amount of messages sent across it from clients. Those metrics are served via hyper on port 8081 and can be aggregated by Prometheus using the config file provided in-tree.

Launching prometheus:
    `prometheus --config.file=prometheus.yml`
Accessing the dashboard (from a local browser):
    `http://<server ip>:<server port>/graph?g0.expr=messages_sent_total`
e.g.:
    `http://127.0.0.1:9090/graph?g0.expr=messages_sent_total`


### Client
Launching the client is just as simple:
    `cargo run --bin client <server ip> <server port>`
e.g.
    `cargo run --bin client 127.0.0.1 8080`

For client usage, invoke `.usage` after launching.

### Questions:
n/a

### Class Notes:
n/a

### Reflections for Lukáš and self:

This assignment turned out to be one of the simplest ones to implement. At least with the minimal example I have established here; one metric to display the count of messages clients have sent across the server.

### Questions:
n/a

### Class Notes:
n/a
