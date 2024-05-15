use std::{env, process, sync::mpsc, thread};

use hw02_string_manip_cli::run;

fn main() {
    // Grab transmutation type from args or print error to stderr and exit
    let transmutation = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("expected 1 argument, but got none");
        process::exit(1);
    });

    // Create channel for threads to communicate within
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to handle input from stdin
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let mut input = String::new();
        // TODO convert this to loop to continually read input rather than closing
        if let Ok(_) = std::io::stdin().read_line(&mut input) {
            match tx_clone.send(input.clone()) {
                Ok(_) => println!("Sent {input}"),
                Err(_) => println!("Failed to send {input}"),
            }
        } else {
            println!("Failed to read from stdin.");
        }
    });

    // Spawn thread to receive and process info from the channel
    let transmutation_clone = transmutation.clone();
    thread::spawn(move || {
        for request in rx {
            println!("We have received {request}")
        }
    });

    // Perform transmutation and print resulting str or gracefully handle error and exit
    match run(&transmutation) {
        Err(e) => {
            eprintln!("problem running application: {}", e);
            process::exit(1);
        }
        Ok(transmuted_str) => println!("Transmutation result: \n{}", transmuted_str),
    };
}
