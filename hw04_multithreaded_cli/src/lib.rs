use comfy_table::{self, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use csv::ReaderBuilder;
use slug::slugify;
use std::{error::Error, fs::File, io::Read, path, str::FromStr, sync::mpsc};

pub fn run(command: Command, input_str: String) -> Result<String, Box<dyn Error>> {
    // Transmute target string
    let result = match command {
        Command::Lowercase => lowercase_str(&input_str),
        Command::Uppercase => uppercase_str(&input_str),
        Command::NoSpaces => no_spaces_str(&input_str),
        Command::Trim => trim_str(&input_str),
        Command::Double => double_str(&input_str),
        Command::Slugify => slugify_str(&input_str),
        Command::Csv => csv_str(&input_str),
    };

    // Return transmuted string or hand Err up the cal stack
    match result {
        Ok(output) => Ok(output),
        Err(e) => Err(e),
    }
}

pub fn process_input(
    tx: mpsc::Sender<(Command, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();

    loop {
        input.clear(); // sanitize input before reading the next line
        println!("Please choose your transmutation and input: <command> <input>");
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
                eprintln!("invalid input -- expected: <command> <input>");
            }
        }
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

fn csv_str(file_path: &str) -> Result<String, Box<dyn Error>> {
    // Convert csv at `filepath` to String or return error up the stack
    let csv_str = read_csv_file(file_path)?;

    if csv_str.is_empty() || csv_str == "\n" {
        Err(From::from("input csv is empty"))
    } else {
        // Create a Table to store our data
        let mut table = comfy_table::Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS);

        // Create a Reader
        let mut rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(csv_str.as_bytes());

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

fn read_csv_file(path: &str) -> Result<String, Box<dyn Error>> {
    let path = path::Path::new(path);

    // Open the file or return specific error up the stack
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => return Err(From::from("Error: CSV file not found")),
            std::io::ErrorKind::PermissionDenied => {
                return Err(From::from("Error: Permission denied"))
            }
            _ => return Err(From::from("Unable to process CSV file")),
        },
    };

    // Grab the contents and store them as a String to be processed
    let mut csv_str = String::new();
    file.read_to_string(&mut csv_str)?;

    Ok(csv_str)
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
            _ => Err(CommandParseError {
                invalid_command: s.to_string(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct CommandParseError {
    invalid_command: String,
}

impl Error for CommandParseError {}

impl std::fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid command provided: '{}' -- valid commands are: 'lowercase', 'uppercase', 'no-spaces', 'trim', 'double', 'slugify', and 'csv'",
            self.invalid_command
        )
    }
}

// TODO implement the Debug trait for CommandParseError
