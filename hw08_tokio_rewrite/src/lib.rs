use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{error::Error, io};
use thiserror::Error;
use tokio::{
    self,
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

/// Represents a Message consisteng of: Text, an Image, or a File.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum MessageType {
    Text(String),
    Image(Vec<u8>),
    File(String, Vec<u8>),
}

impl MessageType {
    /// Retuns a String representing a serialized MessageType.
    pub fn serialize_msg(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Retuns a MessageType from a deserialized Byte Array.
    pub fn deseralize_msg(&self, input: &[u8]) -> MessageType {
        serde_json::from_slice(input).unwrap()
    }

    pub async fn send<T: AsyncWriteExt + Unpin>(&self, stream: &mut T) -> Result<()> {
        log::trace!("Entering MessageType::send()");

        // Serialize the msssage before transmitting
        let serialized = self.serialize_msg();

        // Send length of serialized message (as 4-byte value)
        let len = serialized.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;

        // Send the serialized message
        stream.write_all(serialized.as_bytes()).await?;
        log::info!("[SENT] {}", self.to_string());

        log::trace!("Exiting MessageType::send()");

        Ok(())
    }

    pub async fn recv<T: AsyncReadExt + Unpin>(stream: &mut T) -> Result<Self> {
        log::trace!("Entering MessageType::recv()");

        let mut length_bytes = [0; 4];

        // Determine the length, in bytes, of the incomming message
        let msg_len = stream
            .read_exact(&mut length_bytes)
            .await
            .context("Failed to read msg length.")?;
        let mut buffer = vec![0u8; msg_len];

        // Read the incomming message from the stream buffer
        stream
            .read_exact(&mut buffer)
            .await
            .context("Failed to read stream")?;

        // Deseralize message from buffer and return it
        let msg = deserialize_msg(&buffer)
            .await
            .context("Failed to deserialze bytes into MessageType")?;
        log::debug!("Succesfully received message.");

        log::trace!("Exiting MessageType::recv()");
        Ok(msg)
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Text(text) => write!(f, "{} ", text),
            MessageType::Image(_) => write!(f, "<MessageType::Image>"),
            MessageType::File(name, _) => write!(f, "<MessageType::File>: {}", name),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization Error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Message Error: {0}")]
    Message(String),

    #[error("Client or Server disconnected")]
    Disconnected,

    #[error("Would Block")]
    WouldBlock,

    #[error("Unknown Error: {0}")]
    Unknown(String),
}

/// Sends a serialized MessageType to remote stream
pub async fn send_message(stream: &mut OwnedWriteHalf, message: String) -> Result<()> {
    log::trace!("Entering lib::send_message()");
    // Send length of serialized message (as 4-byte value)
    let len = message.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;

    // Send the serialized message
    stream.write_all(message.as_bytes()).await?;

    log::trace!("Exiting lib::send_message()\n sent: {}", &message);
    Ok(())
}

/// Returns a MessageType from a deserialized Byte Array.
async fn deserialize_msg(input: &[u8]) -> Result<MessageType, serde_json::Error> {
    serde_json::from_slice(input)
}

/// Retrieves a message of length `msg_len` from a remote stream and attempts
/// to construct and return a valid MessageType
pub async fn receive_msg(stream: &mut OwnedReadHalf, msg_len: usize) -> Result<MessageType> {
    let mut buffer = vec![0u8; msg_len];

    stream
        .read_exact(&mut buffer)
        .await
        .context("Failed to read stream");

    // Deseralize message from buffer and return it
    let msg = deserialize_msg(&buffer)
        .await
        .context("Failed to deserialze bytes into MessageType")?;
    Ok(msg)
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
#[derive(Debug)]
pub enum Command {
    File,
    Image,
    Text,
    Help,
    Quit,
}

impl std::str::FromStr for Command {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ".file" => Ok(Command::File),
            ".image" => Ok(Command::Image),
            ".help" => Ok(Command::Help),
            ".quit" => Ok(Command::Quit),
            _ => Ok(Command::Text),
        }
    }
}

#[derive(Debug)]
pub struct CommandParseError {}

impl Error for CommandParseError {}

impl std::fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem parsing command input.")
    }
}
