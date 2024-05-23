use std::{error::Error, net::TcpStream, str::FromStr};

use common::{send_message, MessageType};

// Stub client code to just send something to the server for testing
pub fn run_client(server_address: &str) -> Result<(), Box<dyn Error>> {
    println!("In client::main()");

    let mut stream = TcpStream::connect(&server_address)?;

    println!("Entering client::main() loop on inut");
    // Read input from stdin
    loop {
        client_usage();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        // Identify command and possible message type
        let trimmed_input = input.trim();
        let parts: Vec<&str> = trimmed_input.splitn(2, ' ').collect();
        let command = Command::from_str(parts[0])?;
        dbg!("{:?}", &command);

        // Prepare user message
        let message: MessageType = match command {
            Command::Quit => break,
            Command::Help => continue,
            Command::File => todo!(),
            Command::Image => todo!(),
            Command::Text => MessageType::Text(parts.join(" ")),
        };

        // Send the message
        send_message(&mut stream, message)?;
    }
    println!("Exiting client::main() loop on input");
    println!("Leaving client::main()");

    Ok(())
}

fn client_usage() {
    println!(
        "------------------------------ \n\
    Usage: client <server ip> <server port>\n\
    ------------------------------ \n\
    Message broadcast options: \n\
    \t- <message> \n\
    \t- .file <path> \n\
    \t- .image <path> \n\
    \t- .help \n\
    \t- .quit \n\
    ------------------------------"
    );
}

#[derive(Debug)]
pub enum Command {
    File,
    Image,
    Text,
    Help,
    Quit,
}

impl std::str::FromStr for Command {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ".file" => Ok(Command::File),
            ".image" => Ok(Command::Image),
            ".help" => Ok(Command::Help),
            ".quit" => Ok(Command::Quit),
            _ => Ok(Command::Text),
        }
    }
}

#[derive(Debug)]
pub struct CommandParseError {}

impl Error for CommandParseError {}

impl std::fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Problem parsing command input.")
    }
}