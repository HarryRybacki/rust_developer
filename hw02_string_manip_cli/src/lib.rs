use std::{error::Error, fmt, io};

use comfy_table;
use csv::ReaderBuilder;
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

    // TODO CLEANUP THIS IS FOR TESTING CSV
    let data = "\
Name,Place,Id
Mark,Zurich,1
Ashley,Madrid,2
John,New York,3
";

    // Transmute target string
    let result = match valid_transmutation.as_ref() {
        "lowercase" => lowercase_str(&target_str),
        "uppercase" => uppercase_str(&target_str),
        "no-spaces" => no_spaces_str(&target_str),
        "trim" => trim_str(&target_str),
        "double" => double_str(&target_str),
        "slugify" => slugify_str(&target_str),
        "csv" => csv_str(&data), // TODO clean up from testing w/ data
        _ => unreachable!(),     // valid_transmutation guarantees this arm is unreachable
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
        "csv",
    ];

    if transmutations.contains(&transmutation) {
        Ok(transmutation.to_string())
    } else {
        // PICK UP HERE -- REMOVE MYERROR AND REPLACE WITH THIS SYNTAX. ASK CHATGPT WTF FROM::FROM IS ABOUT
        return Err(From::from("received invalid transmutation type provided"));
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

fn csv_str(target_str: &str) -> Result<String, Box<dyn Error>> {
    if target_str.is_empty() || target_str == "\n" {
        Err(Box::new(MyError {
            message: format!("Input string is empty."),
        }))
    } else {
        // Create a Table to store our data
        let mut table = comfy_table::Table::new();

        // Create a Reader
        let mut rdr = ReaderBuilder::new().from_reader(target_str.as_bytes());

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
        // TODO Is this okay or should we hand the exception up?
        let output = table.to_string();
        Ok(output)
    }
}
