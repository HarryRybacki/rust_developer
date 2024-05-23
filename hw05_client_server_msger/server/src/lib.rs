use std::{
    collections::HashMap,
    fmt,
    io::{self, Read},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use common::{send_message, MessageType};

/// Establishes a server to listen and route messages
///
/// Functionally, this establishes a TcpListener which will process
/// incoming streams. New clients are stored in a HashMap. Any message
/// received by one client will be forwarded to any other client the server
/// has a current connection with
///
/// TODO: How do we know when to halt the server?
pub fn listen_and_accept(address: &str) -> Result<(), ServerError> {
    println!("Entering listen_and_accept()");

    // Establish TcpListener to capture incoming streams
    let listener = TcpListener::bind(address)?;

    let clients = Arc::new(Mutex::new(HashMap::<SocketAddr, TcpStream>::new()));

    // Monitor stream and handle incoming connections
    for stream in listener.incoming() {
        println!("New stream from the listener");

        let mut stream = stream?;
        let clients = Arc::clone(&clients);

        thread::spawn(move || match handle_client(stream, clients) {
            Ok(()) => println!("client handled succesfully within thread"),
            Err(e) => eprintln!("encountered server error within thread: {}", e),
        });
    }

    println!("Exiting listen_and_accept()");
    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
) -> Result<(), ServerError> {
    println!("New connection and call to handle_client()");

    // Attempt to store the client in the clients HashMap
    let addr = stream.peer_addr()?;
    // TODO fix the unwrap
    let mut clients_guard = clients.lock().unwrap();
    clients_guard.insert(addr, stream.try_clone()?);

    loop {
        let msg = match read_message(&mut stream) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!(
                    "Server error encountered reading message from stream: {:?}",
                    e
                );
                break;
            }
        };

        println!("Received message from {}: {:?}", addr, msg);

        let response = MessageType::Text(String::from("Message received..."));
        if let Err(e) = send_message(&mut stream, response) {
            println!("Error sending response: {:?}", e);
            break;
        }
    }

    {
        let mut clients_guard = clients.lock().unwrap();
        clients_guard.remove(&addr);
    }
    Ok(())
}

fn read_message(mut stream: &mut TcpStream) -> Result<MessageType, ServerError> {
    println!("Entering server::read_messsage()");

    // get length of message
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    // fetch message from buffer
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;

    // deseralize message
    Ok(common::deseralize_msg(&buffer))
}
#[derive(Debug)]
pub enum ServerError {
    Io(io::Error),
    SerdeJson(serde_json::Error),
    Other(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::Io(err) => write!(f, "IO error: {}", err),
            ServerError::SerdeJson(err) => write!(f, "Serialization error: {}", err),
            ServerError::Other(err) => write!(f, "Other error: {}", err),
        }
    }
}

impl std::error::Error for ServerError {}

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> Self {
        ServerError::Io(err)
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        ServerError::SerdeJson(err)
    }
}
