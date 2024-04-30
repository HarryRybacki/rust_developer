use std::env;
use std::io;

use slug::slugify;

fn main() {
    // Store command line arguments and verify input provided
    let args: Vec<String> = env::args().collect();

    // Confirm transmutation was specified
    if args.len() < 2 {
        panic!("No transmutation type provided. Please pass desired transmutation through as an argument.")
    }

    // Determine and validate requested transmutation
    let transmutation: &str = &args[1];

    let transmutations: [&str; 6] = [
        "lowercase",
        "uppercase",
        "no-spaces",
        "trim",
        "double",
        "slugify",
    ];

    if !transmutations.contains(&transmutation) {
        panic!("Invalid transmutation type selected. Please try again.");
    }

    // Collect target string from user input
    println!("Please enter string to: '{}'", transmutation);
    let mut target_str = String::new();
    io::stdin()
        .read_line(&mut target_str)
        .expect("Failed to read input");

    // Transmute user input
    let mut transmuted_str = String::new();

    if transmutation == "lowercase" {
        transmuted_str.push_str(&target_str.to_lowercase());
    } else if transmutation == "uppercase" {
        transmuted_str.push_str(&target_str.to_uppercase());
    } else if transmutation == "no-spaces" {
        transmuted_str.push_str(&target_str.replace(" ", ""))
    } else if transmutation == "trim" {
        transmuted_str.push_str(&target_str.trim());
    } else if transmutation == "double" {
        transmuted_str.push_str(&target_str);
        transmuted_str.push_str(&target_str);
    } else if transmutation == "slugify" {
        let slugged_str = slugify(&target_str);
        transmuted_str.push_str(&slugged_str)
    }

    // Display results to stdout
    println!(
        "Input String: {}\nTransmuted String: {}",
        target_str, transmuted_str
    );
}
