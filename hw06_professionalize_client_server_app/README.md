# [Homework 6](https://robot-dreams-rust.mag.wiki/11-rust-ecosystem/index.html#homework)
Your next challenge is to professionalize your client-server chat application by organizing it into Cargo crates and incorporating production-ready libraries. This assignment will also give you the opportunity to clean up your project structure and prepare it for real-world applications.

## [Description:](https://robot-dreams-rust.mag.wiki/11-rust-ecosystem/index.html#description)

### 1. Cargo Crates Conversion:

- [X] If you have not already, transform both the client and server parts of your chat application into separate Cargo crates.

- [X] Structure your project directory to clearly separate the two parts of the application.

### 2. Shared Functionality:

- [X] Identify any shared functionality between the client and server.
- [X] Consider abstracting this shared code into a third "library" crate that both the client and server can utilize.

### 3. Production-Ready Libraries:

- [X] Introduce production-ready libraries for key functionalities, such as:
  - [X] log (with some backend) or tracing (with tracing-subscriber) for logging.
  - rayon for data parallelism, if applicable.
  - itertools for advanced iterator operations, if applicable.

### 4. Crates Exploration:

- [X] Dive into resources such as [crates.io](https://crates.io/), [lib.rs](https://lib.rs/), or [rust-unofficial/awesome-rust](https://github.com/rust-unofficial/awesome-rust) on GitHub to discover crates that could simplify or enhance your chat application.
- [X] Look for crates that offer robust, tested solutions to common problems or that can add new functionality to your application, if you want. Keep in mind that we will be rewriting the application to be asynchronous soon

### 5. Documentation and Comments:

- [X] Update your README.md to document how to use the new crates and any significant changes you've made to the application structure.
- [X] Add comments throughout your code to explain your reasoning and provide guidance on how the code works.

### 6. Refactoring:

- [X] Refactor your existing codebase to make use of the new crates and shared library, ensuring that everything is cleanly integrated and operates smoothly.

### Usage:

#### Invoking the server:
Note: The server can be called without specifiying an IP/Port. In that case, a default address of localhost:11111 will be used.

1. Launch the Server from terminal 1: `cargo run -p server <server ip> <server port>`

#### Invoking a client:
Note: Multiple Clients must be active with the Server in order to transmit messages, images, or files.

1. Launch a Client to message through the Server from terminal 2: `cargo run -p client <server ip> <server port>`

#### Enabling logging:
Note: INFO level logging is enabled by default.

1. Prepend your client or server launch command with the log level you wish to see (info, warn, error, debug): `RUST_LOG=<level> <launch command>`

### Questions:

No new questions came up this week related to the homework. 

### Class Notes:

N/A

### Reflections for Lukáš:

This week was busy (outside of class). So not so much time to play around here. 

Fortunately, I already had a leg up on this week's homework having refactored my application last week; separating client and server into their own crates as well as integrating a common library for shared functions e.g. `send_message`. 

The biggest refactor of the week was moving away from the println/eprintln macros and integrating log/env_logger combo throughout the codebase. This cleaned up the output from both the client and the server a lot. I was surprised at how easy the logger was to establish. But, I'm still uncetain how to handle logging output in the shared library. I'll open a GH issue and ping you in it.

I did plan on adding custom error type for the Client code but opted to hold off until playing with `anyhow` and `thiserror` next week first.
