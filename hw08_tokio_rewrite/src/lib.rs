use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{error::Error, io};
use thiserror::Error;
use tokio::{self, net::TcpStream};

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
    pub fn deseralize_msg(input: &[u8]) -> MessageType {
        serde_json::from_slice(input).unwrap()
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "This is a MessageType: {} ", self)
    }
}

pub async fn generate_message(command: Command, parts: Vec<&str>) -> Result<MessageType> {
    let msg = match command {
        Command::File => {
            let path_str = parts.get(1).context("Missing file path.")?;
            let data = tokio::fs::read(path_str)
                .await
                .context("Failed to read file.")?;
            let file_name = std::path::Path::new(path_str)
                .file_name()
                .and_then(|name| name.to_str())
                .context("Failed to get file name")?;
            log::info!("[SENDING FILE] {}", &file_name);
            MessageType::File(String::from(file_name), data)
        }
        Command::Image => {
            let path_str = parts.get(1).context("Missing image path.")?;
            let data = tokio::fs::read(path_str)
                .await
                .context("Failed to read image.")?;
            log::info!("[SENDING IMAGE] {}", &path_str);
            MessageType::Image(data)
        }
        Command::Text => {
            let message = parts.join(" ");
            log::info!("[SENT] {}", &message);
            MessageType::Text(message)
        }
        // FIXME: This should return an error, these Command types do not support conversion
        Command::Help | Command::Quit => unreachable!(),
    };
    Ok(msg)
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
