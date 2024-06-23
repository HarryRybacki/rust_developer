use anyhow::Result;
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::{get_hostname, MessageType};
use std::env;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::{TcpListener, TcpStream},
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

    log::info!("Connecting to server...");
    // Establish network and stdin readers
    let stream = TcpStream::connect(&address).await.map_err(|e| {
        log::error!("Client failed to connect to server at {}: {}", address, e);
        e
    })?;

    // Split stream into separate reader and writer; we want independant mut refs to pass to separate tokio tasks
    let (mut reader, mut write) = stream.into_split();

    // Create a mpsc channel to send stdin from the terminal task to server writer task

    // Spawn tokio task to manage capturing terminal inputs

    // Spawn tokio task to manage reading from server stream

    // Spawn tokio task to manage writing to server stream

    Ok(())
}

async fn send_message(stream: &mut TcpListener, msg: MessageType) {
    todo!();
}
