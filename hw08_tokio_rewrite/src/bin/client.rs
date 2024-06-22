use std::env;
use anyhow::Result;
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::get_hostname;
use tokio::{
    io::{self, AsyncBufReadExt, BufReader},
    net::TcpStream,
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
    let network_stream = TcpStream::connect(&address)
        .await
        .map_err(|e| {
            log::error!("Client failed to connect to server at {}: {}", address, e);
            e
        })?;
    let network_rdr = BufReader::new(network_stream);
    let mut network_lines = network_rdr.lines();

    let terminal_in = io::stdin();
    let terminal_rdr = BufReader::new(terminal_in);
    let mut terminal_lines = terminal_rdr.lines();

    // loop over input Futures and handle accordingly
    log::info!("Enterin client loop...");
    loop {
        tokio::select! {
            stream_rx = network_lines.next_line() => {
                // handle stream input
                continue;
            }
            terminal_line = terminal_lines.next_line() => {
                // handle terminal input
                log::info!("Reading terminal input we hope");
                match terminal_line {
                    Ok(Some(terminal_line)) => {
                        println!("User input: {}", terminal_line);
                    }
                    Ok(None) => {
                        println!("no input, we done?");
                        break;
                    }
                    Err(e) => {
                        log::error!("Client error reading from stdin: {}", e);
                        break
                    }
                }
            }
        }
    }

    Ok(())
}
