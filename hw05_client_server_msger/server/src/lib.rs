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
    println!("Entering server::listen_and_accept()");

    // Establish TcpListener to capture incoming streams
    let listener = TcpListener::bind(address)?;

    // Create a threadsafe HashMap to track clients connected to the server
    let clients = Arc::new(Mutex::new(HashMap::<SocketAddr, TcpStream>::new()));

    // Iterate over incoming streams and handle connections
    for stream in listener.incoming() {
        println!("Server is opening a new stream");

        // Unwrap stream, note peer address, and clone Clients for the thread
        let mut stream = stream?;
        let peer_addr = stream.peer_addr()?;
        let inner_clients = Arc::clone(&clients);

        // Create new thread to manage handle_client
        let handle = thread::spawn(move || {
            match handle_client(stream, &inner_clients) {
                Ok(()) => println!("client handled succesfully within thread"),
                Err(e) => eprintln!("encountered server error within thread: {}", e)
            }
        });

        // Stream is about to close, attempt to drop the client now
        // Ensure thread closes so we are safe to lock and drop the client
        let _ = handle.join();
        // TODO: Is this the right location? Is this the right way to handle?
        let mut clients_guard = clients.lock().unwrap();
        clients_guard.remove(&peer_addr);
        println!("Server dropped client: {}", &peer_addr);
    }

    println!("Exiting server::listen_and_accept()");
    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    clients: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
) -> Result<(), ServerError> {
    println!("Entering server::handle_client()");

    // Clone clients HashMap and add the stream
    let inner_clients = Arc::clone(&clients);
    println!("\tA new clone of clients has been made.");
    dbg!("{:?}", &clients);

    let addr = stream.peer_addr()?;

    println!("\tattempting to lock clients to insert new client");
    // TODO fix the unwrap
    let mut clients_guard = inner_clients.lock().unwrap();
    clients_guard.insert(addr, stream.try_clone()?);
    println!("Server added client connection: {}", &addr);

    loop {
        let msg = match common::receive_message(&mut stream) {
            Ok(msg) => {
                println!("returned from common::receive_message() [IN OK MATCH]");
                msg
            }
            Err(e) => {
                println!("returned from common::receive_message() [IN ERROR MATCH]");
                // TODO: Drop client from the servers map?
                eprintln!(
                    "Server error encountered reading message from stream: {:?}",
                    e
                );
                break;
            }
        };

        println!(
            "Server rx'd message from client {}: {:?}\n\tPreparing broadcast...",
            addr, msg
        );
        // TODO: Finish broadcast_message
        //broadcast_message(msg, &clients, addr)?;

        println!("Server rx'd message from client {}: {:?}", addr, msg);
        println!("Server preparing confirmation of receipt");
        let response = format!("Server confirms message receipt from you: '{}'", &addr);

        let response_for_client = MessageType::Text(String::from(response));
        if let Err(e) = send_message(&mut stream, response_for_client) {
            println!("Error sending response: {:?}", e);
            break;
        }
    }

    Ok(())
}

fn broadcast_message(
    message: MessageType,
    clients: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    sender_addr: SocketAddr,
) -> Result<(), ServerError> {
    println!("Entering server::broadcast_message()");

    // try send a the message to every client in the server is tracking
    todo!();

    println!("Exiting server::broadcast_message()");
    Ok(())
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
