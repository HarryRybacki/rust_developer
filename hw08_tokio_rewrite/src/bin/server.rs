use anyhow::{Context, Result};
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::{get_hostname, recieve_message, MessageType};
use std::{env, net::SocketAddr};
use tokio::{
    self,
    io::{self, AsyncReadExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
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

    // Create tokio listener
    let listener = TcpListener::bind(address)
        .await
        .context("Failed to bind to socket.")?;

    // Initiate accept loop for server
    loop {
        // Capture the incoming socket and address; continue looping if connection fails
        let Ok((stream, addr)) = listener.accept().await else {
            log::error!("Failed to connect to client socket.");
            continue;
        };

        log::trace!("New client connection: {}", &addr);

        // Split stream into separate reader and writer; we want independent mut refs to pass to separate tokio tasks
        let (stream_rdr, stream_wtr) = stream.into_split();

        // Spawn tokio task to manage reading from the client
        tokio::spawn(async move {
            process_client_rdr(stream_rdr, addr).await;
        });

        // Spawn tokio task to manage writing to the client
        tokio::spawn(async move {
            process_client_wtr(stream_wtr, addr).await;
        });
    }

    Ok(())
}

// FIXME: Should return a Result
async fn process_client_rdr(mut stream: OwnedReadHalf, addr: SocketAddr) {
    log::info!("Starting to process client reader for {}", &addr);
    //let mut length_buffer = vec![0; 4];
    let mut length_bytes = [0; 4];

    // Add client to DB

    loop {
        // TODO: read_exact is blocking IIRC, should this task calling this function be `is_blocking` or something?
        match stream
            .read_exact(&mut length_bytes)
            .await
            .context("Failed to read length")
        {
            Ok(_) => {
                let msg_len = u32::from_be_bytes(length_bytes) as usize;

                log::info!(
                    "Attempting to retrieve a {}-byte message from {}:",
                    msg_len.to_string(),
                    addr
                );
                let msg = recieve_message(&mut stream, msg_len)
                    .await
                    .context("Failed to read message")
                    .unwrap(); // FIXME: Replace with Try operator
                log::info!("{:?}", msg);
                let test = process_message(msg)
                    .await
                    .context("Failed to process the incoming message...")
                    .unwrap(); // FIXME: Replace with Try operator
                               // TODO: do something with the message... like send it other clients or something
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
}

async fn process_message(msg: MessageType) -> Result<()> {
    todo!();
}

// FIXME: Should return a Result
async fn process_client_wtr(mut stream: OwnedWriteHalf, addr: SocketAddr) {
    log::info!("Starting to process client writer for {}", &addr);

    // Keep the writer task alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
