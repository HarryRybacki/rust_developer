use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::{error::Error, io};
use thiserror::Error;
use tokio::{
    self,
    io::{AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

/// Represents a user.
///
/// This struct holds the ID and name of a user.
///
/// # Example
/// ```
/// let user = User { id: 1, name: "Alice".to_string() };
/// ```
#[derive(Clone, Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
}

/// Represents internal messages, including user ID updates.
///
/// This enum is used for internal communication within the server to handle user ID updates.
///
/// # Example
/// ```
/// let update = InternalMessage::UserIdUpdate(42);
/// ```
pub enum InternalMessage {
    UserIdUpdate(i64),
}

/// Represents a message consisting of text, an image, or a file.
///
/// This enum is used to handle different types of messages that can be sent or received.
///
/// # Example
/// ```
/// let text_message = MessageType::Text(Some("Alice".to_string()), "Hello, World!".to_string());
/// let image_message = MessageType::Image(Some("Alice".to_string()), vec![1, 2, 3]);
/// let file_message = MessageType::File(Some("Alice".to_string()), "file.txt".to_string(), vec![1, 2, 3]);
/// let register_message = MessageType::Register("Alice".to_string());
/// ```
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum MessageType {
    Text(Option<String>, String),          // (username, message)
    Image(Option<String>, Vec<u8>),        // (username, contents)
    File(Option<String>, String, Vec<u8>), // (username, filepath, contents)
    Register(String),                      // (username)
}

impl MessageType {
    /// Returns a String representing a serialized `MessageType`.
    ///
    /// This function serializes the message into a JSON string.
    ///
    /// # Example
    /// ```
    /// let message = MessageType::Text(Some("Alice".to_string()), "Hello, World!".to_string());
    /// let serialized = message.serialize_msg();
    /// println!("{}", serialized);
    /// ```
    pub fn serialize_msg(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Returns a `MessageType` from a deserialized byte array.
    ///
    /// This function deserializes the byte array into a `MessageType`.
    ///
    /// # Example
    /// ```
    /// let bytes = b"{\"Text\":[\"Alice\",\"Hello, World!\"]}";
    /// let message = MessageType::Text(None, String::new()).deserialize_msg(bytes);
    /// println!("{:?}", message);
    /// ```
    pub fn deserialize_msg(input: &[u8]) -> MessageType {
        serde_json::from_slice(input).unwrap()
    }

    /// Sends a serialized `MessageType` to a remote stream.
    ///
    /// This function serializes the message, sends its length, and then sends the serialized message.
    ///
    /// # Example
    /// ```
    /// let message = MessageType::Text(Some("Alice".to_string()), "Hello, World!".to_string());
    /// message.send(&mut stream).await?;
    /// ```
    ///
    /// # Errors
    /// This function returns an error if it fails to write to the stream.
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

    /// Receives a `MessageType` from a remote stream.
    ///
    /// This function reads the length of the incoming message, reads the message, and then deserializes it.
    ///
    /// # Example
    /// ```
    /// let message = MessageType::recv(&mut stream).await?;
    /// println!("{:?}", message);
    /// ```
    ///
    /// # Errors
    /// This function returns an error if it fails to read from the stream or deserialize the message.
    pub async fn recv<T: AsyncReadExt + Unpin>(stream: &mut T) -> Result<Self> {
        log::trace!("Entering MessageType::recv()");

        let mut length_bytes = [0; 4];

        // Determine the length, in bytes, of the incoming message
        stream
            .read_exact(&mut length_bytes)
            .await
            .context("Failed to read msg length.")?;
        let msg_len = u32::from_be_bytes(length_bytes) as usize;
        let mut buffer = vec![0u8; msg_len];

        // Read the incoming message from the stream buffer
        stream
            .read_exact(&mut buffer)
            .await
            .context("Failed to read stream")?;

        // Deserialize message from buffer and return it
        let msg = MessageType::deserialize_msg(&buffer);
        log::debug!("Successfully received message.");

        log::trace!("Exiting MessageType::recv()");
        Ok(msg)
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Text(Some(username), text) => write!(f, "[{}] {}", username, text),
            MessageType::Text(None, text) => write!(f, "[anonymous] {}", text),
            MessageType::Image(Some(username), _) => {
                write!(f, "[{}] <MessageType::Image>", username)
            }
            MessageType::Image(None, _) => write!(f, "[anonymous] <MessageType::Image>"),
            MessageType::File(Some(username), name, _) => {
                write!(f, "[{}] <MessageType::File>: {}", username, name)
            }
            MessageType::File(None, name, _) => {
                write!(f, "[anonymous] <MessageType::File>: {}", name)
            }
            MessageType::Register(account) => {
                write!(f, "<Registering user '{}' with the server>", account)
            }
        }
    }
}

/// Represents application errors.
///
/// This enum defines various errors that can occur in the application.
///
/// # Example
/// ```
/// let io_error = AppError::Io(io::Error::new(io::ErrorKind::Other, "an error"));
/// let serialization_error = AppError::Serialization(serde_json::Error::custom("an error"));
/// let message_error = AppError::Message("an error".to_string());
/// let disconnected_error = AppError::Disconnected;
/// let would_block_error = AppError::WouldBlock;
/// let unknown_error = AppError::Unknown("an error".to_string());
/// ```
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

/// Retrieves a message of specified length from a remote stream and attempts to construct and return a valid `MessageType`.
///
/// This function reads a message from the stream and deserializes it.
///
/// # Example
/// ```
/// let message = receive_msg(&mut stream, 128).await?;
/// println!("{:?}", message);
/// ```
///
/// # Errors
/// This function returns an error if it fails to read from the stream or deserialize the message.
pub async fn receive_msg(stream: &mut OwnedReadHalf, msg_len: usize) -> Result<MessageType> {
    let mut buffer = vec![0u8; msg_len];

    stream
        .read_exact(&mut buffer)
        .await
        .context("Failed to read stream")?;

    // Deseralize message from buffer and return it
    let msg = MessageType::deserialize_msg(&buffer);

    Ok(msg)
}

/// Generates a formatted hostname by parsing the arguments.
///
/// This function constructs a server address from command-line arguments or assigns default values.
///
/// # Example
/// ```
/// let args = vec!["program".to_string(), "localhost".to_string(), "8080".to_string()];
/// let address = get_hostname(args);
/// println!("{}", address); // Outputs: localhost:8080
/// ```
///
/// This function does not return any errors.
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

/// Represents a command that can be issued by the user.
///
/// This enum defines various commands that the user can issue.
///
/// # Example
/// ```
/// let command = Command::from_str(".help")?;
/// match command {
///     Command::Help => println!("Help command issued"),
///     _ => println!("Other command issued"),
/// }
/// ```
///
/// # Errors
/// This function returns an error if it fails to parse the command.
#[derive(Debug)]
pub enum Command {
    File,
    Help,
    Image,
    Register,
    Text,
    Quit,
}

impl std::str::FromStr for Command {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ".file" => Ok(Command::File),
            ".help" => Ok(Command::Help),
            ".image" => Ok(Command::Image),
            ".register" => Ok(Command::Register),
            ".quit" => Ok(Command::Quit),
            _ => Ok(Command::Text),
        }
    }
}

/// Represents an error that occurs when parsing a command.
///
/// This struct defines the error that occurs when a command cannot be parsed.
///
/// # Example
/// ```
/// let error = CommandParseError {};
/// println!("{}", error);
/// ```
#[derive(Debug)]
pub struct CommandParseError {}

impl Error for CommandParseError {}

impl std::fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem parsing command input.")
    }
}
