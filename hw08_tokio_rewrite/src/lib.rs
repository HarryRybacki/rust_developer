use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;
use tokio::net::TcpStream;

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
