use anyhow::{Context, Result};
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::{generate_message, get_hostname, Command, MessageType};
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
    // FIXME: wtf we have to specify the type? probs should be a &[u8] if anything
    //        Should not be String or MessageType probably... we're sending the 'serialized' version to the writer task
    let (tx, mut rx) = mpsc::channel::<MessageType>(1024);

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

async fn process_stdin(tx: mpsc::Sender<MessageType>) -> Result<()> {
    log::info!("Starting process stdin consumer.");
    let stdin = tokio::io::stdin();
    let mut stdin_rdr = BufReader::new(stdin).lines();

    // Wait and process user input from stdin
    client_usage();
    while let Some(line) = stdin_rdr.next_line().await.unwrap() {
        // Determine user intent
        let trimmed_input = line.trim();
        let parts: Vec<&str> = trimmed_input.splitn(2, ' ').collect();
        let command = Command::from_str(parts[0])?;

        // Handle requests to exit gracefully or display usage
        match command {
            Command::Quit => {
                // FIXME: Impliment a cancellation mechanism for the client and plug in here
                println!("quit...");
                break;
            }
            Command::Help => {
                println!("help...");
                client_usage();
            }
            Command::File | Command::Image | Command::Text => {
                let msg = generate_message(command, parts).await?;
                tx.send(msg)
                    .await
                    .context("Failed to send message to the writer task")?;
            }
        }
    }

    Ok(())
}

async fn process_server_rdr(mut stream: OwnedReadHalf) -> Result<()> {
    log::info!("Starting process server reader.");

    Ok(())
}

// FIXME: reciever should probably be a &[u8] as the stdin reader is sending
//        the serialized message here, not the raw type.
async fn process_server_wtr(
    mut stream: OwnedWriteHalf,
    rx: &mut mpsc::Receiver<MessageType>,
) -> Result<()> {
    log::info!("Starting process server writer.");

    while let Some(message) = rx.recv().await {
        println!("Should send '{}' to server...", "pew");
        send_message(&mut stream, message).await?;
        //stream.write_all(message.as_bytes()).await?;
    }

    Ok(())
}

pub async fn send_message(stream: &mut OwnedWriteHalf, message: MessageType) -> Result<()> {
    log::trace!("Entering common::send_message()");
    // Serialize the message for tx
    let serialized_msg = message.serialize_msg();

    // Send length of serialized message (as 4-byte value)
    let len = serialized_msg.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;

    // Send the serialized message
    stream.write_all(serialized_msg.as_bytes()).await?;

    log::trace!("Exiting send_message()\n sent: {}", &serialized_msg);
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
