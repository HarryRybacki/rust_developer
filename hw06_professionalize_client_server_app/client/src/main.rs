use env_logger::{Env, Builder};
use std::env;

fn main() {
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
}
