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

    let clients = Arc::new(Mutex::new(HashMap::<SocketAddr, TcpStream>::new()));

    // Monitor stream and handle incoming connections
    for stream in listener.incoming() {
        println!("Server is opening a new stream");

        let mut stream = stream?;
        let clients = Arc::clone(&clients);

        thread::spawn(move || match handle_client(stream, clients) {
            Ok(()) => println!("client handled succesfully within thread"),
            Err(e) => {
                eprintln!("encountered server error within thread: {}", e)
            }
        });
    }

    println!("Exiting server::listen_and_accept()");
    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
) -> Result<(), ServerError> {
    println!("Entering server::handle_client()");

    // Attempt to store the client in the clients HashMap
    let addr = stream.peer_addr()?;
    // TODO fix the unwrap
    let mut clients_guard = clients.lock().unwrap();
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

                /* TODO: Figure out how and where to gracefully drop clients from the server's tracker
                let mut clients_guard = clients.lock().unwrap();
                clients_guard.remove(&addr);
                println!("Server dropped client: {}", &addr);
                */
                eprintln!(
                    "Server error encountered reading message from stream: {:?}",
                    e
                );

                break;
            }
        };

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
