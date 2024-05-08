use std::{env, io, process};

use hw02_string_manip_cli::run;

fn main() {
    // report error call process exit if unable to extract transmutation from env args
    let transmutation = env::args().nth(1).unwrap_or_else(|| {
        // TODO MOVE TO EPRINTLN
        println!("Problem parsing arguments: transmutation not found.");
        process::exit(1);
    });

    // TODO move this into run()?
    // Collect target string from user input
    println!("Please enter string to: '{}'", transmutation);
    let mut target_str = String::new();
    io::stdin()
        .read_line(&mut target_str)
        .expect("Failed to read input");

    if let Err(e) = run(&transmutation, &target_str) {
        // TODO MOVE TO EPRINTLN
        println!("Problem running application: {}", e);
        process::exit(1);
    }
}
