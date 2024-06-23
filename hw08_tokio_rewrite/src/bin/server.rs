use anyhow::{Context, Result};
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::get_hostname;
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

async fn process_client_rdr(mut stream: OwnedReadHalf, addr: SocketAddr) {
    log::info!("Starting to process client reader for {}", &addr);
    let mut buffer = vec![0; 1024];

    // Add client to DB

    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                // Connection was closed
                log::info!("Client {} disconnected", addr);
                break;
            }
            Ok(n) => {
                // Print received message
                log::info!(
                    "Received from {}: {}",
                    addr,
                    String::from_utf8_lossy(&buffer[..n])
                );
            }
            Err(e) => {
                // Handle read error
                log::error!("Error reading from {}: {:?}", addr, e);
                break;
            }
        }
    }

    // Drop client from DB
}

async fn process_client_wtr(mut stream: OwnedWriteHalf, addr: SocketAddr) {
    log::info!("Starting to process client writer for {}", &addr);

    // Keep the writer task alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
