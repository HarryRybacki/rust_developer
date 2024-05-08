use std::{error::Error, fmt, io};

use slug::slugify;

pub fn run(transmutation: &str) -> Result<(), Box<dyn Error>> {
    // Validate the chose transmutation or propogate the Err up to main()
    let valid_transmutation = validate_transmutation(transmutation)?;

    // Collect target string from user input
    println!("Please enter string to: '{}'", &valid_transmutation);
    let mut target_str = String::new();
    io::stdin()
        .read_line(&mut target_str)
        .expect("Failed to read input");

    // Transmute target string
    let result = match valid_transmutation.as_ref() {
        "lowercase" => lowercase_str(&target_str),
        "uppercase" => uppercase_str(&target_str),
        "no-spaces" => no_spaces_str(&target_str),
        "trim" => trim_str(&target_str),
        "double" => double_str(&target_str),
        "slugify" => slugify_str(&target_str),
        "csv" => csv(&target_str),
        _ => unreachable!(), // valid_transmutation guarantees this arm is unreachable
    };

    // Print results or hand error back up to main()
    match result {
        Ok(output) => {
            println!("Transmutation result: {}", output);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

// TODO: Is a custom error type needed?
#[derive(Debug)]
struct MyError {
    message: String,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MyError {}

fn validate_transmutation(transmutation: &str) -> Result<String, Box<dyn Error>> {
    // Validate transmutation type
    let transmutations = vec![
        "lowercase",
        "uppercase",
        "no-spaces",
        "trim",
        "double",
        "slugify",
    ];

    if transmutations.contains(&transmutation) {
        Ok(transmutation.to_string())
    } else {
        Err(Box::new(MyError {
            message: format!("{} not a valid transmutation. Try again.", transmutation),
        }))
    }
}

fn lowercase_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(Box::new(MyError {
            message: format!("Input string is empty."),
        }))
    } else {
        let output = target_str.to_lowercase();
        Ok(output)
    }
}

fn uppercase_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(Box::new(MyError {
            message: format!("Input string is empty."),
        }))
    } else {
        let output = target_str.to_uppercase();
        Ok(output)
    }
}

fn no_spaces_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(Box::new(MyError {
            message: format!("Input string is empty."),
        }))
    } else {
        let output = target_str.trim().replace(" ", "");
        Ok(output)
    }
}

fn trim_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(Box::new(MyError {
            message: format!("Input string is empty."),
        }))
    } else {
        let output = target_str.trim().to_string();
        Ok(output)
    }
}
fn double_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(Box::new(MyError {
            message: format!("Input string is empty."),
        }))
    } else {
        let mut output = String::new();
        output.push_str(target_str);
        output.push_str(target_str);
        Ok(output)
    }
}

fn slugify_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(Box::new(MyError {
            message: format!("Input string is empty."),
        }))
    } else {
        let output = slugify(target_str);
        Ok(output)
    }
}

fn csv(_target_str: &str) -> Result<String, Box<dyn Error>> {
    todo!()
}
