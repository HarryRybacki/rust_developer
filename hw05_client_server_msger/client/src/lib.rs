use std::net::TcpStream;

use common::{send_message, MessageType};

// Stub client code to just send something to the server for testing
pub fn run_client() {
    println!("In client::main()");

    // Create a message
    let message = MessageType::Text(String::from("test message for server"));

    // Create a tcp connection
    let address = "127.0.0.1:8080";

    let mut stream = TcpStream::connect(address).unwrap();

    let _result = send_message(&mut stream, message);

    // catch a response?

    println!("Leaving client::main()");
}
