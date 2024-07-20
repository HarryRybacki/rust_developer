use anyhow::{Context, Result};
use env_logger::{Builder, Env};
use hw11_rust_metrics::{get_hostname, receive_msg, Command, MessageType};
use std::{env, str::FromStr};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, ErrorKind},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    select,
    sync::mpsc,
};
use tokio_util::sync;

/// Entry point for the client application.
///
/// This function initializes logging, processes command-line arguments to determine the server address, and manages
/// the connection to the server. It spawns separate tasks for handling terminal input, reading from the server, and
/// writing to the server.
///
/// # Example
/// ```
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // Setup and start client application
///     main().await?;
///     Ok(())
/// }
/// ```
///
/// # Errors
/// This function returns an error if it fails to connect to the server or if there are issues with any of the spawned tasks.
#[tokio::main]
async fn main() -> Result<()> {
    // Establish our logger
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();

    // Process parameters to determine hostname and what not for Server
    let args: Vec<String> = env::args().collect();
    let address = get_hostname(args);

    log::info!("Connecting to server: {}", address);
    // Establish network and stdin readers
    let stream = TcpStream::connect(&address).await.map_err(|e| {
        log::error!("Client failed to connect to server at {}: {}", &address, e);
        e
    })?;

    // Create a mpsc channel to send stdin from the terminal task to server writer task
    let (tx, mut rx) = mpsc::channel::<MessageType>(1024);

    // Create and clone shutdown token to handle managing graceful shutdowns across tasks
    let shutdown_token = sync::CancellationToken::new();
    let stdin_shutdown = shutdown_token.clone();
    let rdr_shutdown = shutdown_token.clone();
    let wtr_shutdown = shutdown_token.clone();

    // Spawn tokio task to manage capturing terminal inputs
    let stdin_task = tokio::spawn(async move {
        // Wait for cancellation or handle stdin from user
        select! {
            _ = stdin_shutdown.cancelled() => log::debug!("Cancel signal initiated, stdin_task shutting down..."),
            res = process_stdin(tx, stdin_shutdown.clone()) => {
                match res {
                    Ok(_) => log::debug!("stdin reader exiting task successfully.\nShutting down..."),
                    Err(e) => log::error!("stdin reader encountered an error: {:?}\nShutting down...", e),
                }
            }
        }
    });

    // Split stream into separate reader and writer; we want independant mut refs to pass to separate tokio tasks
    let (mut reader, mut writer) = stream.into_split();

    // Spawn tokio task to manage reading from server stream
    let rdr_task = tokio::spawn(async move {
        // Wait for cancellation or handle stdin from user
        select! {
            _ = rdr_shutdown.cancelled() => log::debug!("Cancel signal initiated, stdin_task shutting down..."),
            res = process_server_rdr(reader, rdr_shutdown.clone()) => {
                match res {
                    Ok(_) => log::debug!("Server reader exitign task successfully.\nShutting down..."),
                    Err(e) => log::error!("Server reader encountered an error: {:?}\nShutting down...", e),
                }
            }
        }
    });

    // Spawn tokio task to manage writing to server stream
    let wtr_task = tokio::spawn(async move {
        select! {
            _ = wtr_shutdown.cancelled() => {
                log::debug!("Cancel signal initiated, stdin_task shutting down...");
                // FIXME: Server shut be notifed client is disconnecting
            },
            res = process_server_wtr(writer, &mut rx, wtr_shutdown.clone()) => {
                match res {
                    Ok(_) => log::debug!("Server reader exitign task successfully.\nShutting down..."),
                    Err(e) => log::error!("Server writer encountered an error: {:?}\nShutting down...", e),
                }
            }
        }
    });

    tokio::join!(stdin_task, rdr_task, wtr_task);

    Ok(())
}

/// Handles user input from stdin and sends messages to the server.
///
/// This function reads user input, determines the command type, and sends the appropriate message to the server through
/// a channel.
///
/// # Example
/// ```
/// let (tx, mut rx) = mpsc::channel::<MessageType>(1024);
/// let shutdown_token = sync::CancellationToken::new();
/// process_stdin(tx, shutdown_token.clone()).await?;
/// ```
///
/// # Errors
/// This function returns an error if it fails to read from stdin or send messages.
async fn process_stdin(
    tx: mpsc::Sender<MessageType>,
    shutdown: sync::CancellationToken,
) -> Result<()> {
    log::trace!("Starting process: stdin Consumer.");
    let stdin = tokio::io::stdin();
    let mut stdin_rdr = BufReader::new(stdin).lines();

    // Wait and process user input from stdin
    client_usage();
    while let Some(line) = stdin_rdr
        .next_line()
        .await
        .context("Failed to read from stdin.")?
    {
        // Determine user intent
        let trimmed_input = line.trim();
        let parts: Vec<&str> = trimmed_input.splitn(2, ' ').collect();
        let command = Command::from_str(parts[0])?;

        // Handle requests to exit gracefully or display usage
        match command {
            Command::Quit => {
                log::debug!("User requested quit. Client initiating shutdown..");
                log::info!("Shutdown the client...");
                shutdown.cancel();
                break;
            }
            Command::Help => {
                client_usage();
            }
            Command::Text => {
                if parts[0].is_empty() {
                    log::debug!("User attempting to send an empty String. Ignoring...");
                    continue;
                } else {
                    let msg = generate_message(command, parts).await?;
                    tx.send(msg)
                        .await
                        .context("Failed to send message to the writer task")?;
                }
            }
            Command::File | Command::Image => {
                let msg = generate_message(command, parts).await?;
                tx.send(msg)
                    .await
                    .context("Failed to send message to the writer task")?;
            }
            Command::Register => {
                if parts[0].is_empty() {
                    log::debug!("User attempting to register without an account. Ignoring...");
                    continue;
                } else {
                    let msg = generate_message(command, parts).await?;
                    tx.send(msg)
                        .await
                        .context("Failed to send message to the writer task")?;
                }
            }
        }
    }

    Ok(())
}

/// Reads and processes incoming messages from the server.
///
/// This function continuously reads messages from the server, processes them, and performs appropriate actions such as
/// logging and saving files.
///
/// # Example
/// ```
/// let (mut reader, mut writer) = stream.into_split();
/// let shutdown_token = sync::CancellationToken::new();
/// process_server_rdr(reader, shutdown_token.clone()).await?;
/// ```
///
/// # Errors
/// This function returns an error if it fails to read from the stream or process messages.
async fn process_server_rdr(
    mut stream: OwnedReadHalf,
    shutdown: sync::CancellationToken,
) -> Result<()> {
    log::trace!("Starting process: Server Reader.");
    let mut length_bytes = [0; 4];

    loop {
        match stream
            .read_exact(&mut length_bytes)
            .await
            .context("Failed to read length")
        {
            Ok(_) => {
                let msg_len = u32::from_be_bytes(length_bytes) as usize;

                log::debug!(
                    "Attempting to retrieve a {}-byte message from the server.",
                    msg_len.to_string()
                );
                let msg = receive_msg(&mut stream, msg_len)
                    .await
                    .context("Failed to read message")?;
                log::debug!("{:?}", msg);

                match msg {
                    MessageType::File(Some(username), name, data) => {
                        log::info!("[RECEIVED FILE from {}] Saving to..: {}", username, name);
                        save_file(name, data).await?
                    }
                    MessageType::File(None, name, data) => {
                        log::info!("[RECEIVED FILE from anonymous] Saving to..: {}", name);
                        save_file(name, data).await?
                    }
                    MessageType::Image(Some(username), data) => {
                        log::info!("[RECEIVED IMAGE from {}]", username);
                        save_image(data).await?
                    }
                    MessageType::Image(None, data) => {
                        log::info!("[RECEIVED IMAGE from Anonymous]");
                        save_image(data).await?
                    }
                    MessageType::Text(Some(username), text) => {
                        log::info!("[{}] {}", username, text)
                    }
                    MessageType::Text(None, text) => {
                        log::info!("[Anonymous] {}", text)
                    }
                    MessageType::Register(account) => {
                        log::info!("[NEW USER LOGGED IN] {}", account)
                    }
                }
            }
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    match io_err.kind() {
                        ErrorKind::UnexpectedEof => {
                            log::info!("Server disconnected. Shutting down...");
                            let _ = shutdown.cancel();
                            break;
                        }
                        ErrorKind::ConnectionReset => {
                            log::info!("Server at connection reset. Shutting down...");
                            let _ = shutdown.cancel();
                            break;
                        }
                        ErrorKind::BrokenPipe => {
                            log::info!("Server connection had broken pipe. Shutting down...");
                            let _ = shutdown.cancel();
                            break;
                        }
                        _ => {
                            log::info!("Unexpected error reading from server: {:?}", e);
                            let _ = shutdown.cancel();
                            break;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Saves a byte array as a file locally.
///
/// This function creates a file in the `./files/` directory with the given name and writes the provided data to it.
///
/// # Example
/// ```
/// save_file("example.txt".to_string(), vec![104, 101, 108, 108, 111]).await?;
/// ```
///
/// # Errors
/// This function returns an error if it fails to create the directory, convert the file path to a string, or write the
/// data to the file.
async fn save_file(file_name: String, data: Vec<u8>) -> Result<()> {
    // Attempt to create the path
    let path = std::path::Path::new("./files");
    tokio::fs::create_dir_all(path)
        .await
        .context("Failed to create directory.")?;

    // Create and save the file
    let file_path = path.join(file_name);
    let file_path_str = file_path
        .to_str()
        .context("Failed to convert file path to string")?;

    let mut file = tokio::fs::File::create(&file_path)
        .await
        .with_context(|| format!("Failed to create file: {}", &file_path_str))?;

    file.write_all(&data)
        .await
        .context("Failed to write file to local storage.")?;

    log::info!("[RECEIVED FILE] Saving to..: {}", file_path_str);

    Ok(())
}

/// Saves a byte array as an image locally.
///
/// This function creates a file in the `./images/` directory with a generated name based on the current datetime and
/// writes the provided image data to it.
///
/// # Example
/// ```
/// save_image(vec![137, 80, 78, 71]).await?;
/// ```
///
/// # Errors
/// This function returns an error if it fails to create the directory, generate the file name, or write the data to
/// the file.
async fn save_image(data: Vec<u8>) -> Result<()> {
    let file_name = generate_file_name()
        .await
        .context("Failed to generate file name for image")?;

    let mut file = tokio::fs::File::create(&file_name)
        .await
        .with_context(|| format!("Failed to open file: {}", &file_name))?;
    file.write_all(&data)
        .await
        .context("Failed to write image to local disk.")?;
    log::info!("[RECEIVED IMAGE] Saving to..: {}", file_name);

    Ok(())
}

/// Generates a file name based on the current datetime.
///
/// This function creates a directory `./images/` if it doesn't exist and generates a filename in the format
/// `YYYYMMDDHHMMSS.png`.
///
/// # Example
/// ```
/// let file_name = generate_file_name().await?;
/// println!("Generated file name: {}", file_name);
/// ```
///
/// # Errors
/// This function returns an error if it fails to create the directory or generate the file name.
async fn generate_file_name() -> Result<String> {
    let path = std::path::Path::new("./images");
    tokio::fs::create_dir_all(path)
        .await
        .context("Failed to create directory")?;

    let now = chrono::Local::now();

    Ok(format!("./images/{}.png", now.format("%Y%m%d%H%M%S")))
}

/// Manages sending messages to the server.
///
/// This function listens for messages from the stdin task and sends them to the server through the provided stream.
///
/// # Example
/// ```
/// let (tx, mut rx) = mpsc::channel::<MessageType>(1024);
/// let (mut reader, mut writer) = stream.into_split();
/// let shutdown_token = sync::CancellationToken::new();
/// process_server_wtr(writer, &mut rx, shutdown_token.clone()).await?;
/// ```
///
/// # Errors
/// This function returns an error if it fails to send messages over the stream.
async fn process_server_wtr(
    mut stream: OwnedWriteHalf,
    rx: &mut mpsc::Receiver<MessageType>,
    shutdown: sync::CancellationToken,
) -> Result<()> {
    log::trace!("Starting process: Server Writer.");

    // Wait for messages coming from stdin and process
    loop {
        tokio::select! {
            Some(message) = rx.recv() => {
                message
                    .send(&mut stream)
                    .await
                    .context("Failed to send message over stream to server.")?;
            }
            _ = shutdown.cancelled() => {
                log::debug!("Shutdown signal received in writer, exiting...");
                // FIXME: Server shut be notifed client is disconnecting
                break;
            }
        }
    }

    Ok(())
}

/// Displays client usage helper text.
///
/// This function logs the available commands and their usage for the client.
///
/// # Example
/// ```
/// client_usage();
/// ```
///
/// This function does not return any errors.
fn client_usage() {
    log::info!(
        "
------------------------------ \n\
Message broadcast options: \n\
\t- <message> \n\
\t- .file <path> \n\
\t- .image <path> \n\
\t- .register <account name> \n\
\t- .help \n\
\t- .quit \n\
------------------------------"
    );
}

/// Creates a `MessageType` based on user CLI input.
///
/// This function takes a command and a vector of message parts, and generates the corresponding `MessageType`.
///
/// # Example
/// ```
/// let parts = vec!["this", "is", "a", "test"];
/// let command = Command::Text;
/// let message = generate_message(command, parts).await?;
/// ```
///
/// # Errors
/// This function returns an error if it fails to process the message parts.
async fn generate_message(command: Command, parts: Vec<&str>) -> Result<MessageType> {
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
            log::debug!("[GENERATING MessageType::File] from {}", &file_name);
            MessageType::File(None, String::from(file_name), data)
        }
        Command::Register => {
            let account = parts
                .iter()
                .skip(1)
                .map(|s| *s)
                .collect::<Vec<&str>>()
                .join(" ");
            log::debug!("[GENERATING MessageType::Register] {}", &account);
            MessageType::Register(account)
        }
        Command::Image => {
            let path_str = parts.get(1).context("Missing image path.")?;
            let data = tokio::fs::read(path_str)
                .await
                .context("Failed to read image.")?;
            log::debug!("[GENERATING MessageType::Image] from {}", &path_str);
            MessageType::Image(None, data)
        }
        Command::Text => {
            let message = parts.join(" ");
            log::debug!("[GENERATING MessageType::Text] {}", &message);
            MessageType::Text(None, message)
        }
        Command::Help | Command::Quit => unreachable!(),
    };
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[tokio::test]
    async fn text_command_makes_text_messagetype() {
        let test_parts = vec!["this", "is", "a", "test"];
        let test_cmd = Command::Text;
        let base_msg = MessageType::Text(None, "this is a test".to_string());

        let generated_msg = generate_message(test_cmd, test_parts).await.unwrap();

        assert_eq!(base_msg, generated_msg);
    }

    #[tokio::test]
    async fn text_command_does_not_make_register_messagetype() {
        let test_parts = vec![".register", "Timothy"];
        let test_cmd = Command::Text;
        let base_msg = MessageType::Register("Timothy".to_string());

        let generated_msg = generate_message(test_cmd, test_parts).await.unwrap();

        assert_ne!(base_msg, generated_msg);
    }

    #[tokio::test]
    async fn register_command_makes_register_messagetype() {
        let test_parts = vec![".register", "Timothy"];
        let test_cmd = Command::Register;
        let base_msg = MessageType::Register("Timothy".to_string());

        let generated_msg = generate_message(test_cmd, test_parts).await.unwrap();

        assert_eq!(base_msg, generated_msg);
    }

    #[tokio::test]
    async fn register_command_does_not_make_text_messagetype() {
        let test_parts = vec![".register", "Timothy"];
        let test_cmd = Command::Register;
        let base_msg = MessageType::Text(None, "Timothy".to_string());

        let generated_msg = generate_message(test_cmd, test_parts).await.unwrap();

        assert_ne!(base_msg, generated_msg);
    }
}
