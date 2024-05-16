use std::{process, sync::mpsc, thread};

use hw04_multithreaded_cli::{process_input, run};

fn main() {
    // Create channel for threads to communicate within
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to process input from stdin
    let input_thread = thread::spawn(move || {
        if let Err(e) = process_input(tx) {
            eprintln!("Error handling input: {e}");
            process::exit(1);
        }
    });

    // Spawn thread to receive and process info from the channel
    let processing_thread = thread::spawn(move || {
        for request in rx {
            let (command, input_str) = request;

            match run(command, input_str) {
                Err(e) => {
                    eprintln!("problem running application: {}", e);
                    // TODO do we still want to halt the program or let failures hit stderr and continue?
                    // process::exit(1);
                }
                Ok(transmuted_str) => println!("Transmutation result: \n{}", transmuted_str),
            };
        }
    });

    // keep main() running until threads close
    let _ = input_thread.join();
    let _ = processing_thread.join();
}
