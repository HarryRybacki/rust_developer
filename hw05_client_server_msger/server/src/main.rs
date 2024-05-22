use std::env;

fn main() {
    println!("entering server::main()");

    // Process parameters to determine hostname and what not for server
    let args: Vec<String> = env::args().collect();
    let address = common::get_hostname(args);
    println!("Server address set to: {}", address);

    // Create the server and begin routing
    let _server = server::listen_and_accept(&address);

    println!("Leaving server::main()");
}
