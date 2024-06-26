use anyhow::{Context, Result};
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::{get_hostname, send_message, Command, MessageType};
use std::{
    env,
    error::Error,
    str::{Bytes, FromStr},
};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::mpsc,
};

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
    let (tx, mut rx) = mpsc::channel::<String>(1024);

    // Spawn tokio task to manage capturing terminal inputs
    let tx_clone = tx.clone();
    let stdin_task = tokio::spawn(async move {
        if let Err(e) = process_stdin(tx_clone).await {
            log::error!("Something went wrong handling client terminal reader process: {}\nShutting down...", e);
            // TODO: Add cancellation signal to shutdown any open tasks.
        };
    });

    // Split stream into separate reader and writer; we want independant mut refs to pass to separate tokio tasks
    let (mut reader, mut writer) = stream.into_split();

    // Spawn tokio task to manage reading from server stream
    let rdr_task = tokio::spawn(async move {
        if let Err(e) = process_server_rdr(reader).await {
            log::error!(
                "Something went wrong handling server writer process: {}\nShutting down...",
                e
            );
            // TODO: Add cancellation signal to shutdown any open tasks.
        };
    });

    // Spawn tokio task to manage writing to server stream
    let wtr_task = tokio::spawn(async move {
        if let Err(e) = process_server_wtr(writer, &mut rx).await {
            log::error!(
                "Something went wrong handling server writer process: {}\nShutting down...",
                e
            );
            // TODO: Add cancellation signal to shutdown any open tasks.
        };
    });

    // TODO: Reflect, should this be as is or should I have a select! circling between them?
    tokio::join!(stdin_task, rdr_task, wtr_task);

    Ok(())
}

/// Manages user input
async fn process_stdin(tx: mpsc::Sender<String>) -> Result<()> {
    log::info!("Starting process stdin consumer.");
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
                // FIXME: Impliment a cancellation mechanism for the client and plug in here
                todo!();
                break;
            }
            Command::Help => {
                client_usage();
            }
            Command::File | Command::Image | Command::Text => {
                let msg = generate_message(command, parts).await?;
                tx.send(msg.serialize_msg())
                    .await
                    .context("Failed to send message to the writer task")?;
            }
        }
    }

    Ok(())
}

/// Manages incomming mesages from the server
async fn process_server_rdr(mut stream: OwnedReadHalf) -> Result<()> {
    log::info!("Starting process server reader.");

    Ok(())
}

/// Manages sending messages to the server
async fn process_server_wtr(
    mut stream: OwnedWriteHalf,
    rx: &mut mpsc::Receiver<String>,
) -> Result<()> {
    log::info!("Starting process server writer.");

    while let Some(message) = rx.recv().await {
        log::info!("Sending new message to server");
        send_message(&mut stream, message).await?;
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