# Homework 5

## Description:

Diving deeper into Rust's capabilities, this assignment will have you explore the world of networked applications. Your task is to create a client-server messaging system, demonstrating Rust's strengths in handling networking and file operations.

### 1. Removing Text Transformation Functionality:

  [X] Before delving into this assignment, ensure you've removed the text transformation functionality from your previous homework. This task will focus solely on networking.

### 2. Wire Format:

  [] For the format in which data is sent and received over the network, consider using one of the following:
    * serde_cbor
    * bincode
    * postcard
  These crates can help serialize and deserialize data efficiently for network transfer.

### 3. Server Creation:

  [] Design the server to receive messages from multiple clients.
  [] Accept port and hostname as parameters. If none are provided, default to localhost:11111.
  [] Setting the hostname to 0.0.0.0 will allow connections from any IP.

### 4. Client Creation:

  [] Clients should connect to the server to send messages.
  [] They too should accept port and hostname parameters, defaulting to localhost:11111 if not given.

### 5. Message Types:

  [] Clients should read input from stdin and recognize three distinct message types:
    [] .file <path>: Sends a file to the server.
    [] image <path>: Sends an image (assumed or required to be .png).
    [] Any other text: Considered a standard text message.
  [] The .quit command should terminate the client.

### 6. Client-side File Handling:

  [] When the client receives images, save them in the images/ directory, naming them <timestamp>.png.
  [] Other received files should be stored in the files/ directory.
  [] Display a notification like Receiving image... or Receiving <filename> for incoming files.
  [] For incoming text messages, display them directly in stdout.

### 7. Bonus Challenge - Image Conversion:

  [] For an extra point, design the client to automatically convert any received image to .png format. This could necessitate some exploration and potentially the addition of other crates.

### Questions:

1. 
