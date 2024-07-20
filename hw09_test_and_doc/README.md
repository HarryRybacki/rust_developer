# [Homework 9](https://robot-dreams-rust.mag.wiki/16-testing-and-documentation/index.html#homework)

## Description:

For this week's task, we'll focus on enhancing your chat application with some essential practices in software development: adding documentation and tests. This assignment is more open-ended, allowing you to decide the extent and depth of documentation and testing based on your discretion and the application's needs.

### Add doc-comments to key functions and modules in your client and server code.
- [X] Focus on providing clear, concise descriptions that explain what each function or module does.
- [X] You don't need to document every function, but aim to cover the main ones, especially those that are complex or not immediately obvious.

### Basic Testing:
- [X] Write a few tests for parts of your application. You can choose which aspects to test based on what you think is most crucial or interesting.
- [X] Consider including a couple of unit tests for individual functions or components and maybe an integration test if applicable.
- [X] Use Rust's built-in testing framework to add these tests to your project.

### Flexibility in Testing:
- [X] There's no requirement for comprehensive test coverage for this assignment. Just a few tests to demonstrate your understanding of testing principles in Rust will suffice.
- [X] Feel free to explore and test parts of the application you're most curious about or consider most critical.

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

I've noticed a few bugs and duplicate code while reviewing the codebase and writing out doc comments.
- The serialize/deserialize/recv lib code were largely moved into the MessageType as a part of the async re-write. Some of that needs to go. I took care of most of it in line here with the exception of the recv code. Both the client and the server leverage that function review it more before completley gutting it.
- The MessageType::send function is leveraging the types `to_string` method for logging outgoing messages to the client's stdout. However, the client presently is unaware of its own username after registering and so it shows up as an anon messages still. This will require a little refactor to fix.
- I will definitely consider structuring my code differently in future projects to allow for easier testing. Perhaps even proper TDD would be well suited for Rust.

### Questions:
n/a

### Class Notes:
n/a
