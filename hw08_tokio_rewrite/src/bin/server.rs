use anyhow::{Context, Result};
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::{get_hostname, receive_msg, MessageType};
use std::{env, net::SocketAddr, ptr::addr_eq};
use tokio::{
    self,
    io::{self, AsyncReadExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
    sync::{self, broadcast},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Establish our logger
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();

    // Process parameters to determine hostname and whatnot for Server
    let args: Vec<String> = env::args().collect();
    let address = get_hostname(args);
    log::info!("Launching server on address: {}", address);

    // Create tokio listener to establish client connections
    let listener = TcpListener::bind(address)
        .await
        .context("Failed to bind to socket.")?;

    // Create broadcast channel to share messages between client connections
    let (br_send, _br_recv) = broadcast::channel(1024);

    // Initiate accept loop for server
    loop {
        // Capture the incoming socket and address; continue looping if connection fails
        let Ok((stream, addr)) = listener.accept().await else {
            log::error!("Failed to connect to client socket.");
            continue;
        };

        log::trace!("New client connection: {}", &addr);

        // Clone the send and create a subscriber. Pass these to the task managing writing to this client's tcp stream. This is the heart of the routing mechanism for these messages
        let sender = br_send.clone();
        let receiver = sender.subscribe();
        // Split stream into separate reader and writer; we want independent mut refs to pass to separate tokio tasks
        let (mut stream_rdr, mut stream_wtr) = stream.into_split();

        // Spawn tokio task to manage reading from the client
        tokio::spawn(async move {
            process_client_rdr(&sender, stream_rdr, addr)
                .await
                .context("Server error handling the client reader")
                .unwrap();
        });

        // Spawn tokio task to manage writing to the client
        tokio::spawn(async move {
            process_client_wtr(receiver, &mut stream_wtr, addr)
                .await
                .context("Server error handling the client writer")
                .unwrap();
        });
    }

    Ok(())
}

// FIXME: Should return a Result
async fn process_client_rdr(
    tx: &broadcast::Sender<(MessageType, SocketAddr)>,
    mut client_stream: OwnedReadHalf,
    addr: SocketAddr,
) -> Result<()> {
    log::info!("Starting process: Client Reader for: {}", &addr);
    let mut length_bytes = [0; 4];

    // Add client to DB

    loop {
        // TODO: read_exact is blocking IIRC, should this task calling this function be `is_blocking` or something?
        match client_stream
            .read_exact(&mut length_bytes)
            .await
            .context("Failed to read length")
        {
            Ok(_) => {
                let msg_len = u32::from_be_bytes(length_bytes) as usize;

                log::debug!(
                    "Attempting to retrieve a {}-byte message from {}:",
                    msg_len.to_string(),
                    addr
                );
                let msg = receive_msg(&mut client_stream, msg_len)
                    .await
                    .context("Failed to read message")?;
                //log::debug!("{:?}", msg);

                // "Wake up" the the writer task and have it handle messaging the clients
                if tx.send((msg.clone(), addr)).is_err() {
                    log::error!(
                        "Something when wrong sending the message down the broadast channel..."
                    );
                }

                continue;
            }
            Err(e) => {
                // TODO: Handle the `early eof` errors caused by clients dropping
                log::error!("Error reading from {}: {:?}", addr, e);
                break;
            }
        }
    }

    // Drop client from DB
    Ok(())
}

async fn process_message(msg: MessageType) -> Result<()> {
    todo!();
}

async fn process_client_wtr(
    mut rx: broadcast::Receiver<(MessageType, SocketAddr)>,
    stream: &mut OwnedWriteHalf,
    addr: SocketAddr,
) -> Result<()> {
    log::info!("Starting process: Client Writer for: {}", &addr);

    while let Ok((msg, other_addr)) = rx.recv().await {
        // If this is the task responsible for sending to the same client the msg came from, ignore
        if other_addr == addr {
            log::debug!(
                "Will not broadcast message from: {} to {}. Same client.",
                other_addr,
                addr
            );
            continue;
        }

        // Otherwise send it to their resepctive TCP Stream
        match msg.send(stream).await {
            Ok(_) => {
                log::debug!("Server successfully sent message to: {}", addr);
            }
            Err(e) => {
                log::error!("Error sending msg to {} tcp stream: {:?}", &addr, e);
                log::info!("Server killing client writer task for: {}", addr);
                break;
            }
        }
        continue;
    }

    Ok(())
}
