use hw05_client_server_msger::listen_and_accept;

fn main() {
    println!("entering main::main()");

    // TODO: Process parameters to determine hostname and what not for server
    let address = "127.0.0.1:8080";

    let server = listen_and_accept(address);

    println!("Leaving main::main()");

    /*
    // Create channel for threads to communicate within
    let (tx, rx) = mpsc::channel();


    // Spawn a thread to process input from stdin
    let input_thread = thread::spawn(move || {
        if let Err(e) = process_input(tx) {
            eprintln!("Input Error: {e}");
        }
    });


    // Spawn thread to receive and process info from the channel
    let processing_thread = thread::spawn(move || {
        for request in rx {
            let (command, input_str) = request;

            match run(command, input_str) {
                Err(e) => eprintln!("Procesing Error: {}", e),
                Ok(transmuted_str) => println!("Transmutation result: \n{}", transmuted_str),
            };
        }
    });

    // Keep main() running until threads close
    let _ = input_thread.join();
    let _ = processing_thread.join();
      */
}
