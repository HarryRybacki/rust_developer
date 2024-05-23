use std::env;

fn main() {
    println!("entering client::main()");

    // Determine server address
    let args: Vec<String> = env::args().collect();
    let server_address = common::get_hostname(args);
    println!("Remote server address: {}", server_address);

    match client::run_client(&server_address) {
        Ok(()) => println!("Client run successful. Exiting..."),
        Err(e) => {
            eprintln!("Encountered error while running client: {}", e)
        }
    }

    println!("leaving client::main()");
}
