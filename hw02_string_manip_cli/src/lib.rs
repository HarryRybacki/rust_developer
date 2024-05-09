use comfy_table::{self, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use csv::ReaderBuilder;
use slug::slugify;
use std::{
    error::Error,
    io::{self, Read},
};

pub fn run(transmutation: &str) -> Result<String, Box<dyn Error>> {
    // Validate the chose transmutation or propogate the Err up to main()
    let valid_transmutation = validate_transmutation(transmutation)?;

    // Collect target string from user input
    println!("Please enter string to: '{}'", &valid_transmutation);
    let mut target_str = String::new();

    // Handle CSV case requiring multi-line input
    match transmutation {
        "csv" => io::stdin().read_to_string(&mut target_str)?,
        _ => io::stdin().read_line(&mut target_str)?, // valid_transmutation guarantees no bad inputs
    };

    // TODO CLEANUP THIS IS FOR TESTING CSV
    let data1 = "\
Name,Place,Id
Mark,Zurich,1
Ashley,Madrid,2
John,New York,3
";
    let data2 = "\
Language,Paradigm,Year,Creator,Rust_Inspiration
C,Imperative,1972,Dennis Ritchie,Medium
Java,Object-Oriented,1995,James Gosling,Low
Python,Multi-Paradigm,1991,Guido van Rossum,Medium
Rust,Multi-Paradigm,2010,Graydon Hoare,High
Haskell,Functional,1990,Lennart Augustsson, Medium
";
    let data3 = "\
Language,Paradigm,Year,Creator,Rust_Inspiration
C,Imperative,1972,Dennis Ritchie,Medium
Java,,1995,James Gosling,Low
Python,Multi-Paradigm,1991,Guido van Rossum,Medium
Rust,Multi-Paradigm,2010,Graydon Hoare,High
Haskell,Functional,1990,Lennart Augustsson,
";

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

    // Return transmuted string or hand error back up to main()
    match result {
        Ok(output) => Ok(output),
        Err(e) => Err(e),
    }
}

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
        // TODO Is this okay or should we hand the exception up?
        let output = table.to_string();
        Ok(output)
    }
}
