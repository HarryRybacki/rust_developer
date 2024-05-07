use std:: {
    env,
    io,
    process,
};

use slug::slugify;

fn main() {
    // report error call process exit if build() fails and unwrap explodes unexpectedly
    let transmutation = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Problem parsing arguments: transmutation not found.");
        process::exit(1);
    });

    // HACK -- how to better handle converting this to a &str?
    let transmutation: &str = &transmutation;

    let transmutations = [
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
