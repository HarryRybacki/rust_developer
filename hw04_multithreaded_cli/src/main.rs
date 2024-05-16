use std::{sync::mpsc, thread};

use hw04_multithreaded_cli::{process_input, run};

fn main() {
    /* Commenting out this section, may come back into play later.
    // Grab transmutation type from args or print error to stderr and exit
    let transmutation = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("expected 1 argument, but got none");
        process::exit(1);
    });
     */

    // Create channel for threads to communicate within
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to process input from stdin
    let input_thread = thread::spawn(move || {
        if let Err(e) = process_input(tx) {
            eprintln!("Error handling input: {e}");
        }
    });

    // Spawn thread to receive and process info from the channel
    let processing_thread = thread::spawn(move || {
        for request in rx {
            let (command, input_str) = request;

            // TODO pickup here; reconnect to run()
            println!("We have received {input_str}");
        }
    });

    /* Commenting out this section as well while we work on the threads
    // Perform transmutation and print resulting str or gracefully handle error and exit
    match run(&transmutation) {
        Err(e) => {
            eprintln!("problem running application: {}", e);
            process::exit(1);
        }
        Ok(transmuted_str) => println!("Transmutation result: \n{}", transmuted_str),
    };
     */

    let _ = input_thread.join();
    let _ = processing_thread.join();
}
