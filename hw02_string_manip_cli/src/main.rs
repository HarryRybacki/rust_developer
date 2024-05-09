use std::{env, process};

use hw02_string_manip_cli::run;

fn main() {
    // Grab transmutation type from args or print error to stderr and exit
    let transmutation = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Expected 1 argument, but got none");
        process::exit(1);
    });

    // Execute transmutation or print error to stderr and exit
    if let Err(e) = run(&transmutation) {
        eprintln!("Problem running application: {}", e);
        process::exit(1);
    }
}
