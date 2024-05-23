use std::env;

fn main() {
    // Process parameters to determine hostname and what not for server
    let args: Vec<String> = env::args().collect();
    let address = common::get_hostname(args);
    println!("Launching server on address: {}", address);

    // Create the server and begin routing
    match server::listen_and_accept(&address) {
        Err(e) => eprintln!("Server error: {}", e),
        Ok(()) => println!("Server shutting down... goodnight."),
    }
}
