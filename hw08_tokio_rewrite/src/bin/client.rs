use anyhow::{Context, Result};
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::{get_hostname, receive_msg, Command, MessageType};
use std::{env, str::FromStr};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    select,
    sync::mpsc,
};
use tokio_util::sync;

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
    // NOTE: No need to clone the Sender as there will only ever be one (the client reader)
    //       Is there a better struct for this sort of 1:1 like a rendezvous or something?
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
                // FIXME: Server shut be notified that client is about to disconnect
            },
            res = process_server_wtr(writer, &mut rx, wtr_shutdown.clone()) => {
                match res {
                    Ok(_) => log::debug!("Server reader exitign task successfully.\nShutting down..."),
                    Err(e) => log::error!("Server writer encountered an error: {:?}\nShutting down...", e),
                }
            }
        }
    });

    // TODO: Reflect, should this be as is or should I have a select! circling between them?
    tokio::join!(stdin_task, rdr_task, wtr_task);

    Ok(())
}

/// Manages user input
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
        }
    }

    Ok(())
}

/// Manages incomming mesages from the server
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
                    "Attempting to retreive a {}-byte message from the server.",
                    msg_len.to_string()
                );
                let msg = receive_msg(&mut stream, msg_len)
                    .await
                    .context("Failed to read message")?;
                log::debug!("{:?}", msg);

                match msg {
                    MessageType::File(name, data) => save_file(name, data).await?,
                    MessageType::Image(data) => save_image(data).await?,
                    MessageType::Text(text) => log::info!("[RECEIVED] {}", text),
                }
            }
            Err(e) => {
                // TODO: Handle the `early eof` errors caused by clients dropping
                log::error!("Error reading from server: {:?}", e);
                let _ = shutdown.cancel();
                break;
            }
        }
    }

    Ok(())
}

/// Saves a byte array as a file locally.
///
/// Assumes filename includes extension and storing in `./files/` dir.
///
/// Returns Result of Ok or Error.
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
/// Assumes filetype is `.png` and storing in `./images/` dir.
///
/// Returns Result of Ok or Error.
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

/// Creates String representing a file's name based on the current datetime.
///
/// Assumes filetype is `.png` and storing in `./images/` dir.
///
/// Returns String or Error.
async fn generate_file_name() -> Result<String> {
    let path = std::path::Path::new("./images");
    tokio::fs::create_dir_all(path)
        .await
        .context("Failed to create directory")?;

    let now = chrono::Local::now();

    Ok(format!("./images/{}.png", now.format("%Y%m%d%H%M%S")))
}

/// Manages sending messages to the server
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
                // FIXME: Server shut be notifed we are disconnecting
                break;
            }
        }
    }

    Ok(())
}

/// Displays Client usage helper text.
fn client_usage() {
    log::info!(
        "
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

/// Creates a MessageType based on User cli input
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
            MessageType::File(String::from(file_name), data)
        }
        Command::Image => {
            let path_str = parts.get(1).context("Missing image path.")?;
            let data = tokio::fs::read(path_str)
                .await
                .context("Failed to read image.")?;
            log::debug!("[GENERATING MessageType::Image] from {}", &path_str);
            MessageType::Image(data)
        }
        Command::Text => {
            let message = parts.join(" ");
            log::debug!("[GENERATING MessageType::Text] {}", &message);
            MessageType::Text(message)
        }
        // FIXME: This should return an error, these Command types do not support conversion
        Command::Help | Command::Quit => unreachable!(),
    };
    Ok(msg)
}
