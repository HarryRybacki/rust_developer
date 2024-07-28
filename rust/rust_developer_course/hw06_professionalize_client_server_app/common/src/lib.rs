use serde::{Deserialize, Serialize};
use std::{error::Error, io, io::Read, io::Write, net::TcpStream};

/// Represents a Message consisteng of: Text, an Image, or a File.
#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Text(String),
    Image(Vec<u8>),
    File(String, Vec<u8>),
}

impl Clone for MessageType {
    fn clone(&self) -> MessageType {
        match self {
            MessageType::Text(text) => MessageType::Text(text.clone()),
            MessageType::Image(image) => MessageType::Image(image.clone()),
            MessageType::File(filename, content) => {
                MessageType::File(filename.clone(), content.clone())
            }
        }
    }
}

/// Retuns a String representing a serialized MessageType.
pub fn serialize_msg(message: MessageType) -> String {
    // Serde Serialize trait on the MessageType makes this seamless
    serde_json::to_string(&message).unwrap()
}

/// Retuns a MessageType from a deserialized Byte Array.
pub fn deseralize_msg(input: &[u8]) -> MessageType {
    // Serde Deserialize trait on the MessageType makes this seamless
    serde_json::from_slice(input).unwrap()
}

/// Sends a MessageType to a specified TcpStream. Uses a 'Length First'
/// protocol by enapsulating the serialized Messagetype's length in a u32
/// and sendig it to the Server before the actual message.
///
/// # Errors
/// Functinon will propogate up any errors encountered serializing a MessageType
/// or while writing to the TcpStream.
pub fn send_message(stream: &mut TcpStream, message: MessageType) -> Result<(), Box<dyn Error>> {
    log::trace!("Entering common::send_message()");
    // Serialize the message for tx
    let serialized_msg = serialize_msg(message);

    // Send length of serialized message (as 4-byte value)
    let len = serialized_msg.len() as u32;
    stream.write_all(&len.to_be_bytes())?;

    // Send the serialized message
    stream.write_all(serialized_msg.as_bytes())?;

    log::trace!("Exiting send_message()\n sent: {}", &serialized_msg);
    Ok(())
}

/// Receives a MessageType to a specified TcpStream. Uses a 'Length First'
/// protocol by retrieving the serialized Messagetype's length in a u32 first
/// then reading and attempting to deserialize the actual message.
///
/// # Errors
/// Functinon will propogate up any errors encountered:
/// - set_read_timeout may result in a WouldBlock error
/// - Reading from the stream can fail and error
/// - Deserializing the message may fail and error
pub fn receive_message(stream: &mut TcpStream) -> Result<MessageType, Box<dyn Error>> {
    log::trace!("Entering common::recieve_messsage()");

    // get length of message
    let mut len_bytes = [0u8; 4];

    // Attempt to read from the stream, raise Error if needed
    // Set read timeout to avoid blocking indefinitely
    stream.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;
    match stream.read_exact(&mut len_bytes) {
        Ok(_) => {
            let len = u32::from_be_bytes(len_bytes) as usize;

            // fetch message from buffer
            let mut buffer = vec![0u8; len];
            stream.read_exact(&mut buffer)?;

            log::trace!("Exiting common::receieve_message() [IN OKAY MATCH]");
            // Deseralize and return message from buffer
            Ok(deseralize_msg(&buffer))
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            // If no data is available, return an error indicating the would block condition
            Err(Box::new(io::Error::new(
                io::ErrorKind::WouldBlock,
                "No data available",
            )))
        }
        Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
            // A client has disconnected
            Err(Box::new(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Client disconnected",
            )))
        }
        Err(e) => {
            log::trace!("Exiting common::receieve_message() [IN UNKOWN ERROR MATCH]");
            Err(From::from(e))
        }
    }
}

/// Generates a formatted String hostname by parsing the args.
pub fn get_hostname(args: Vec<String>) -> String {
    let server_hostname: String;
    let server_port: String;

    match args.len() {
        3 => {
            server_hostname = args[1].clone();
            server_port = args[2].clone();
        }
        _ => {
            server_hostname = String::from("localhost");
            server_port = String::from("11111");
        }
    }

    // Generate the address from params or assign default
    format!("{}:{}", server_hostname, server_port)
}
