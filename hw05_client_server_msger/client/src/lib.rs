use std::{error::Error, net::TcpStream};

use common::{send_message, MessageType};

// Stub client code to just send something to the server for testing
pub fn run_client(server_address: &str) -> Result<(), Box<dyn Error>> {
    println!("In client::main()");

    // Create a message
    let message = MessageType::Text(String::from("test message for server"));

    let mut stream = TcpStream::connect(&server_address)?;

    // Send the message

    let _result = send_message(&mut stream, message)?;

    // catch a response?

    println!("Leaving client::main()");

    Ok(())
}

fn client_usage() {
    println!(
        "------------------------------ \n\
    Usage: client <server ip> <server port>\n\
    ------------------------------ \n\
    Message broadcast options: \n\
    \t- <message> \n\
    \t- .file <path> \n\
    \t- .image <path> \n\
    \t- .help \n\
    \t- .quit \n\
    ------------------------------"
    );
}
