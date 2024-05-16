use comfy_table::{self, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use csv::ReaderBuilder;
use slug::slugify;
use std::{
    error::Error,
    io::{self, Read},
    str::FromStr,
    sync::mpsc,
};

pub fn run(transmutation: &str) -> Result<String, Box<dyn Error>> {
    // Validate the chosen transmutation or hand Err up the call stack
    let valid_transmutation = validate_transmutation(transmutation)?;

    // Collect target string from user input
    println!("Please enter string to: '{}'", &valid_transmutation);
    let mut target_str = String::new();

    // Handle CSV case requiring multi-line input
    match transmutation {
        "csv" => io::stdin().read_to_string(&mut target_str)?,
        _ => io::stdin().read_line(&mut target_str)?, // valid_transmutation() guarantees no bad inputs
    };

    // Transmute target string
    let result = match valid_transmutation.as_ref() {
        "lowercase" => lowercase_str(&target_str),
        "uppercase" => uppercase_str(&target_str),
        "no-spaces" => no_spaces_str(&target_str),
        "trim" => trim_str(&target_str),
        "double" => double_str(&target_str),
        "slugify" => slugify_str(&target_str),
        "csv" => csv_str(&target_str), // TODO clean up from testing w/ data
        _ => unreachable!(),           // valid_transmutation guarantees this arm is unreachable
    };

    // Return transmuted string or hand Err up the cal stack
    match result {
        Ok(output) => Ok(output),
        Err(e) => Err(e),
    }
}

fn validate_transmutation(transmutation: &str) -> Result<String, Box<dyn Error>> {
    // Validate transmutation type or hand Err up the call stack
    let transmutations = vec![
        "lowercase",
        "uppercase",
        "no-spaces",
        "trim",
        "double",
        "slugify",
        "csv",
    ];

    if transmutations.contains(&transmutation) {
        Ok(transmutation.to_string())
    } else {
        return Err(From::from("received invalid transmutation type provided"));
    }
}

fn lowercase_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(From::from("input string is empty"))
    } else {
        let output = target_str.to_lowercase();
        Ok(output)
    }
}

fn uppercase_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(From::from("input string is empty"))
    } else {
        let output = target_str.to_uppercase();
        Ok(output)
    }
}

fn no_spaces_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(From::from("input string is empty"))
    } else {
        let output = target_str.trim().replace(" ", "");
        Ok(output)
    }
}

fn trim_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(From::from("input string is empty"))
    } else {
        let output = target_str.trim().to_string();
        Ok(output)
    }
}
fn double_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(From::from("input string is empty"))
    } else {
        let mut output = String::new();
        output.push_str(target_str);
        output.push_str(target_str);
        Ok(output)
    }
}

fn slugify_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(From::from("input string is empty"))
    } else {
        let output = slugify(target_str);
        Ok(output)
    }
}

fn csv_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(From::from("input string is empty"))
    } else {
        // Create a Table to store our data
        let mut table = comfy_table::Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS);

        // Create a Reader
        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(target_str.as_bytes());

        // Grab the headers
        let headers = rdr.headers()?.clone();

        // Convert headers into an interator so a new Row can be generated from it
        let headers = comfy_table::Row::from(headers.iter());

        // Set the headers of the table
        table.set_header(headers);

        // Iterate over the records to create a set of rows
        for result in rdr.records() {
            // Get the record out of the result or hand up the error
            let record = result?;
            // Convert StringRecord into a row
            let row = comfy_table::Row::from(record.iter());
            table.add_row(row);
        }

        // Generate String from table and return
        // to_string() should be infallible because comfy_table::Table impls `Display`
        let output = table.to_string();
        Ok(output)
    }
}

pub enum Command {
    Lowercase,
    Uppercase,
    NoSpaces,
    Trim,
    Double,
    Slugify,
    Csv,
}

impl std::str::FromStr for Command {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lowercase" => Ok(Command::Lowercase),
            "uppercase" => Ok(Command::Uppercase),
            "no-spaces" => Ok(Command::NoSpaces),
            "trim" => Ok(Command::Trim),
            "double" => Ok(Command::Double),
            "slugify" => Ok(Command::Slugify),
            "csv" => Ok(Command::Csv),
            _ => Err(CommandParseError),
        }
    }
}

#[derive(Debug)]
pub struct CommandParseError;

impl Error for CommandParseError {}

impl std::fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid command entered")
    }
}

pub fn process_input(
    tx: mpsc::Sender<(Command, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();

    loop {
        input.clear();
        std::io::stdin().read_line(&mut input)?;

        let trimmed_input = input.trim();
        if !trimmed_input.is_empty() {
            let parts: Vec<&str> = trimmed_input.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let command_str = parts[0];
                let input_str = parts[1];
                let command = Command::from_str(command_str)?;

                let message = (command, input_str.to_string());
                tx.send(message)?;
            } else {
                eprintln!("Invalid input format. Expected <command> <input>");
            }
        }
    }
}
/*
    let mut input = String::new();

    loop {
            std::io::stdin().read_line(&mut input)?;

            let trimmed_input = input.trim();

            if !trimmed_input.is_empty() {

                // break input string into parts
                let mut parts = trimmed_input.splitn(2, ' ');
                let command_str = parts.next().unwrap();
                let input_str = parts.next().unwrap();

                // create Command enum and pass to rx thread
                let command = Command::from_str(command_str)?;

                tx.send(command, input_str);
            } else {
                println!("Failed to read from stdin.");
            }
    }

}
*/
