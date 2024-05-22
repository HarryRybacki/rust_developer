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

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Text(String),
    Image(Vec<u8>),
    File(String, Vec<u8>),
}

pub fn serialize_msg(message: MessageType) -> String {
    // Serde Serialize trait on the MessageType makes this seamless
    serde_json::to_string(&message).unwrap()
}

pub fn deseralize_msg(input: &[u8]) -> MessageType {
    // Serde Deserialize trait on the MessageType makes this seamless
    serde_json::from_slice(input).unwrap()
}

pub fn send_message(stream: &mut TcpStream, message: MessageType) {
    println!("Entering send_message()");
    // Serialize the message for tx
    let serialized_msg = serialize_msg(message);

    // Open a stream to the server
    //let mut stream = TcpStream::connect(address).unwrap();

    // Send length of serialized message (as 4-byte value)
    let len = serialized_msg.len() as u32;
    // QUESTION: why <u32>.to_be_bytes() -> write, not write_all?
    stream.write(&len.to_be_bytes()).unwrap();

    // Send the serialized message
    // QUESTION: why <String>.as_bytes() -> write_all, not write?
    stream.write_all(&serialized_msg.as_bytes()).unwrap();
    println!("Exiting send_message()");
}

/// Establishes a server to listen and route messages
///
/// Functionally, this establishes a TcpListener which will process
/// incoming streams. New clients are stored in a HashMap. Any message
/// received by one client will be forwarded to any other client the server
/// has a current connection with
///
/// TODO: How do we know when to halt the server?
pub fn listen_and_accept(address: &str) {
    println!("Entering listen_and_accept()");

    // Establish TcpListener to capture incoming streams
    let listener = TcpListener::bind(address).unwrap();

    let mut clients: HashMap<SocketAddr, TcpStream> = HashMap::new();

    for stream in listener.incoming() {
        println!("New stream from the listener");

        // Store client in HashMap
        let mut stream = stream.unwrap();
        let stream_clone = stream.try_clone().unwrap();
        let addr = stream.peer_addr().unwrap();
        let addr_clone = addr.clone();
        clients.insert(addr_clone, stream_clone);

        let msg = handle_client(clients.get(&addr).unwrap().try_clone().unwrap());

        // send response ?
        let response = MessageType::Text(String::from("Message received..."));
        send_message(&mut stream, response);

        println!("responded in listen_and_accept()");

        // print msg
        println!("{:?}", msg);
    }

    println!("Exiting listen_and_accept()");
}

fn handle_client(mut stream: TcpStream) -> MessageType {
    println!("Entering handle_client()");
    // get length of message
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).unwrap();
    let len = u32::from_be_bytes(len_bytes) as usize;

    // fetch message from buffer
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer).unwrap();

    // deseralize message
    deseralize_msg(&buffer)
}

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
