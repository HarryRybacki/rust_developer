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

- [ ] Introduce production-ready libraries for key functionalities, such as:
  - log (with some backend) or tracing (with tracing-subscriber) for logging.
  - rayon for data parallelism, if applicable.
  - itertools for advanced iterator operations, if applicable.

### 4. Crates Exploration:

- [ ] Dive into resources such as [crates.io](https://crates.io/), [lib.rs](https://lib.rs/), or [rust-unofficial/awesome-rust](https://github.com/rust-unofficial/awesome-rust) on GitHub to discover crates that could simplify or enhance your chat application.
- [ ] Look for crates that offer robust, tested solutions to common problems or that can add new functionality to your application, if you want. Keep in mind that we will be rewriting the application to be asynchronous soon

### 5. Documentation and Comments:

- [ ] Update your README.md to document how to use the new crates and any significant changes you've made to the application structure.
- [ ] Add comments throughout your code to explain your reasoning and provide guidance on how the code works.

### 6. Refactoring:

- [ ] Refactor your existing codebase to make use of the new crates and shared library, ensuring that everything is cleanly integrated and operates smoothly.

### Usage:

1. Launch the Server from terminal 1: `cargo run -p server <server ip> <server port>`
2. Launch a Client to message through the Server from terminal 2: `cargo run -p client <server ip> <server port>`

Note: The server can be called without specifiying an IP/Port. In that case, a default address of localhost:11111 will be used.
Note: Multiple Clients must be active with the Server in order to transmit messages, images, or files.

### Questions:

### Class Notes:
