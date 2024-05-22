use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    net::TcpStream,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Text(String),
    Image(Vec<u8>),
    File(String, Vec<u8>),
}

pub fn serialize_msg(message: MessageType) -> String {
    // Serde Serialize trait on the MessageType makes this seamless
    serde_json::to_string(&message).unwrap()
}

pub fn deseralize_msg(input: &[u8]) -> MessageType {
    // Serde Deserialize trait on the MessageType makes this seamless
    serde_json::from_slice(input).unwrap()
}

pub fn send_message(stream: &mut TcpStream, message: MessageType) {
    println!("Entering send_message()");
    // Serialize the message for tx
    let serialized_msg = serialize_msg(message);

    // Open a stream to the server
    //let mut stream = TcpStream::connect(address).unwrap();

    // Send length of serialized message (as 4-byte value)
    let len = serialized_msg.len() as u32;
    // QUESTION: why <u32>.to_be_bytes() -> write, not write_all?
    stream.write(&len.to_be_bytes()).unwrap();

    // Send the serialized message
    // QUESTION: why <String>.as_bytes() -> write_all, not write?
    stream.write_all(&serialized_msg.as_bytes()).unwrap();
    println!("Exiting send_message()");
}

pub fn get_hostname(args: Vec<String>) -> String {
    let mut server_hostname = String::new();
    let mut server_port = String::new();

    match args.len() {
        3 => {
            dbg!("{}", args.clone());
            server_hostname = args[1].clone();
            server_port = args[2].clone();
        },
        _ => { 
            server_hostname = String::from("localhost");
            server_port = String::from("11111");
        }
    }

    // Generate the address from params or assign default
    format!("{}:{}", server_hostname, server_port)
}