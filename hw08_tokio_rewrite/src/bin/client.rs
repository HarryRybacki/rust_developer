use anyhow::Result;
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::{get_hostname, MessageType};
use std::env;
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
    let (tx, mut rx) = mpsc::channel::<String>(1024); // FIXME: wtf we have to specify the type? probs should be a &[u8] if anything

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

async fn process_stdin(tx: mpsc::Sender<String>) -> Result<()> {
    log::info!("Starting process stdin consumer.");
    let stdin = tokio::io::stdin();
    let mut stdin_rdr = BufReader::new(stdin).lines();

    // Wait and process user input from stdin
    while let Some(line) = stdin_rdr.next_line().await.unwrap() {
        // TODO: Plug in old MessageType creation code here.
        if line.starts_with("send ") {
            let message = line.strip_prefix("send ").unwrap().to_string();
            println!("will send: {}", message);
            if tx.send(message).await.is_err() {
                eprintln!("Failed to send message to the writer task");
            }
        } else {
            println!("Unknown command: {}", line);
        }
    }

    Ok(())
}
async fn process_server_rdr(mut stream: OwnedReadHalf) -> Result<()> {
    log::info!("Starting process server reader.");

    Ok(())
}

async fn process_server_wtr(
    mut stream: OwnedWriteHalf,
    rx: &mut mpsc::Receiver<String>,
) -> Result<()> {
    log::info!("Starting process server writer.");

    while let Some(message) = rx.recv().await {
        println!("Should send '{}' to server...", &message);
        stream.write_all(message.as_bytes()).await?;
    }

    Ok(())
}

/*
async fn send_message(stream: &mut TcpListener, msg: MessageType) {
    todo!();
}
*/
