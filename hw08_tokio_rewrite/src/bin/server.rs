use std::env;
use anyhow::{Context, Result};
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::{get_hostname};

use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // Establish our logger
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();

    // Process parameters to determine hostname and what not for Server
    let args: Vec<String> = env::args().collect();
    let address = get_hostname(args);
    log::info!("Launching server on address: {}", address);

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .context("Failed to bind to socket.")?;

    loop {
        let Ok((stream, addr)) = listener.accept().await else {
            log::error!("Failed to accept client connection");
            continue;
        };

        
    }


    Ok(())
}
