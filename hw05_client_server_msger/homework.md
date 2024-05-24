# Homework 5

## Description:

Diving deeper into Rust's capabilities, this assignment will have you explore the world of networked applications. Your task is to create a client-server messaging system, demonstrating Rust's strengths in handling networking and file operations.

### 1. Removing Text Transformation Functionality:

  [X] Before delving into this assignment, ensure you've removed the text transformation functionality from your previous homework. This task will focus solely on networking.

### 2. Wire Format:

  [X] For the format in which data is sent and received over the network, consider using one of the following:
    * serde_cbor -- we will try this one.
    * bincode
    * postcard
  These crates can help serialize and deserialize data efficiently for network transfer.

### 3. Server Creation:

  [X] Design the server to receive messages from multiple clients.
  [X] Accept port and hostname as parameters. If none are provided, default to localhost:11111.
  [] Setting the hostname to 0.0.0.0 will allow connections from any IP.

### 4. Client Creation:

  [X] Clients should connect to the server to send messages.
  [X] They too should accept port and hostname parameters, defaulting to localhost:11111 if not given.

### 5. Message Types:

  [] Clients should read input from stdin and recognize three distinct message types:
    [] .file <path>: Sends a file to the server.
    [] .image <path>: Sends an image (assumed or required to be .png).
    [X] Any other text: Considered a standard text message.
  [X] The .quit command should terminate the client.

### 6. Client-side File Handling:

  [] When the client receives images, save them in the images/ directory, naming them <timestamp>.png.
  [] Other received files should be stored in the files/ directory.
  [] Display a notification like Receiving image... or Receiving <filename> for incoming files.
  [] For incoming text messages, display them directly in stdout.

### 7. Bonus Challenge - Image Conversion:

  [] For an extra point, design the client to automatically convert any received image to .png format. This could necessitate some exploration and potentially the addition of other crates.

### Usage:

1. Launch the server from terminal 1: `cargo run --bin server -- <server ip> <server port>`
2. Call client to pass message to server from terminal 2: `cargo run --bin client`
3. Confirm message received in stdout of terminal 1.

Note: The server can be called without specifiying an IP/Port. In that case, a default address of localhost:11111 will be used.

### Questions:

1. Q: How and where should the server be dropping it's client from the HashMap? I attempted to do this when my call to `handle_connection` fails or finishes but my threads kept getting stuck when doign so...
2. Q: When working with threads, when does it make sense to explicitly call join() on a handle vs just calling thread::spawn(...) and letting it fly?

### Class Notes:


structure 

src/
  bin/ 
    client.rs
  main.rs
  lib.rs

Running client

  cargo run -- hw05

Running server

  cargo run --bin client

Remember to do the 'prefix length' for sending/rxing