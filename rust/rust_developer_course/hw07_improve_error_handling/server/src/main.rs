use anyhow::Result;
use env_logger::{Builder, Env};
use std::env;

/// Establishes a Server. This server maintains a list of TCP Clients, listens
/// to their connections, and broadcasts messages (text, images, and files)
/// sent from one Client to all other Clients connected to the Server.
fn main() -> Result<()> {
    // Establish our logger
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();

    // Process parameters to determine hostname and what not for Server
    let args: Vec<String> = env::args().collect();
    let address = common::get_hostname(args);
    log::info!("Launching server on address: {}", address);

    // Create the Server and begin listening
    match server::listen_and_accept(&address) {
        Err(e) => log::error!("Server error: {}", e),
        Ok(()) => log::info!("Server shutting down... goodnight."),
    }

    Ok(())
}
