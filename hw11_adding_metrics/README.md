# [Homework 11](https://robot-dreams-rust.mag.wiki/18-metrics-in-rust/index.html#homework)

## Description

In this assignment, you will add monitoring capabilities to the server part of your chat application using Prometheus. Monitoring is a crucial aspect of maintaining and understanding the health and performance of applications, especially in production environments.

### Integrate Prometheus:
- [ ] Add Prometheus to your chat application's server.
- [ ] Ensure that Prometheus is set up correctly to gather metrics from your server.

### Metrics Implementation:
- [ ] Implement at least one metric using Prometheus. At a minimum, add a counter to track the number of messages sent through your server.
- [ ] Optionally, consider adding a gauge to monitor the number of active connections to your server. This can provide insights into user engagement and server load.

### Metrics Endpoint:
- [ ] Set up an endpoint within your server application to expose these metrics to Prometheus. This typically involves creating a `/metrics` endpoint.
- [ ] Ensure that the endpoint correctly exposes the metrics in a format that Prometheus can scrape.

> [!NOTE] 
> Typically, this means using the TextEncoder: https://docs.rs/prometheus/0.13.3/prometheus/struct.TextEncoder.html
> You can refer to the Hyper example: https://github.com/tikv/rust-prometheus/blob/master/examples/example_hyper.rs

### Documentation and Testing:
- [ ] Document the new metrics feature in your README.md, including how to access the metrics endpoint and interpret the exposed data.
- [ ] Test to make sure that the metrics are accurately recorded and exposed. Verify that Prometheus can successfully scrape these metrics from your server.

## Server and Client Usage:

### Server

> [!WARNING]
> Ensure you have sqlite installed on your local machine. It is required by the server.

Launching the server is quite simple, from the packages root directory: 
    
    `RUST_LOG=<log level> cargo run --bin server <listening ip> <listening port>`
e.g.
    `RUST_LOG=debug cargo run --bin server 127.0.0.1 8080`

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

### Questions:
n/a

### Class Notes:
n/a
