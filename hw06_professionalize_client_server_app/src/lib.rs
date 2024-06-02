/*
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    error::Error,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    sync::mpsc,
};

pub fn process_input(
    tx: mpsc::Sender<(Command, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();

    // If args present, assume non-interactive mode requested
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 | 3 => {
            let command = Command::from_str(&args[1])?;

            // Collect target string from user input
            println!("Please enter string to: '{}'", &args[1]);
            let mut input_str = String::new();

            let message = (command, input_str);
            tx.send(message)?;

            return Ok(());
        }
        _ => {
            // no args provided or something weird happend, enter interactive mode
            loop {
                input.clear();

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
    }
}

pub fn run(command: Command, input_str: String) -> Result<String, Box<dyn Error>> {
    // Transmute target string
    let result = match command {
        Command::Help => help(&input_str),
        Command::Quit => quit(&input_str),
    };

    // Return transmuted string or hand Err up the cal stack
    match result {
        Ok(output) => Ok(output),
        Err(e) => Err(e),
    }
}

fn help(target_str: &str) -> Result<String, Box<dyn Error>> {
    todo!();
    if target_str.is_empty() || target_str == "\n" {
        Err(From::from("input string is empty"))
    } else {
        let output = target_str.to_lowercase();
        Ok(output)
    }
}

fn quit(target_str: &str) -> Result<String, Box<dyn Error>> {
    todo!();
    if target_str.is_empty() || target_str == "\n" {
        Err(From::from("input string is empty"))
    } else {
        let output = target_str.to_uppercase();
        Ok(output)
    }
}

pub enum Command {
    Help,
    Quit,
}

impl std::str::FromStr for Command {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ".help" => Ok(Command::Help),
            ".quit" => Ok(Command::Quit),
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
            "invalid command provided: '{}'\nValid commands are: '.help', and '.quit'",
            self.invalid_command
        )
    }
}

// TODO implement the Debug trait for CommandParseError
 */