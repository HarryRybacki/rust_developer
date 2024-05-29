use env_logger;
use std::env;

fn main() {
    // Establish our logger
    env_logger::init();

    // Process parameters to determine hostname and what not for server
    let args: Vec<String> = env::args().collect();
    let address = common::get_hostname(args);
    log::info!("Launching server on address: {}", address);

    // Create the server and begin routing
    match server::listen_and_accept(&address) {
        Err(e) => log::error!("Server error: {}", e),
        Ok(()) => log::info!("Server shutting down... goodnight."),
    }
}
