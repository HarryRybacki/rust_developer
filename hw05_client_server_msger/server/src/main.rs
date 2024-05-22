fn main() {
    println!("entering server::main()");

    // TODO: Process parameters to determine hostname and what not for server
    let address = "127.0.0.1:8080";

    let _server = server::listen_and_accept(address);

    println!("Leaving server::main()");
}
