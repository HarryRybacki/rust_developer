use std::{env, process};

use hw03_string_manip_cli_revised::run;

fn main() {
    // Grab transmutation type from args or print error to stderr and exit
    let transmutation = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("expected 1 argument, but got none");
        process::exit(1);
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
