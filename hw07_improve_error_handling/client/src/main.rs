use env_logger::{Builder, Env};
use std::env;
use anyhow::Result;

/// Establishes a Client to send and receive messages (text, images, and
/// files) from other clients connected to a Remote server.
fn main() -> Result<()> {
    // Establish our logger
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();

    // Determine server address
    let args: Vec<String> = env::args().collect();
    let server_address = common::get_hostname(args);
    log::info!("Remote server address: {}", server_address);

    match client::run_client(&server_address) {
        Ok(()) => log::info!("Client run successful. \nExiting..."),
        Err(e) => {
            log::error!("Client encountered error while running: {}\nExiting...", e)
        }
    }

    Ok(())
}
