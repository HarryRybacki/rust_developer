use std::{
    collections::HashMap,
    fmt, io,
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
        let stream = stream?;
        let peer_addr = stream.peer_addr()?;
        let inner_clients = Arc::clone(&clients);

        // Create new thread to manage handle_client
        thread::spawn(move || match handle_client(stream, &inner_clients) {
            Ok(()) => println!("client handled succesfully within thread"),
            Err(e) => eprintln!("encountered server error within thread: {}", e),
        });

        // Stream is about to close, attempt to drop the client now
        // TODO: Is this the right location? Is this the right way to handle?
        drop_client(&clients, &peer_addr);
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
    let inner_clients = Arc::clone(clients);
    println!("\tA new clone of clients has been made.");
    let client_addr = stream.peer_addr()?;
    add_client(&inner_clients, &client_addr, stream.try_clone().unwrap());

    loop {
        let msg = match common::receive_message(&mut stream) {
            Ok(msg) => {
                //println!("returned from common::receive_message() [IN OK MATCH]");
                msg
            }
            Err(ref e) => {
                // black magic to let server loop
                if let Some(io_err) = e.downcast_ref::<io::Error>() {
                    // Note: The receive_message() will return WouldBlock periodically
                    //       to make sure the stream hasn't been closed. We want the
                    //       server to continue listening in this case, but break if not
                    if io_err.kind() == io::ErrorKind::WouldBlock {
                        //println!("No data available, continuing...");
                        continue;
                    }
                }
                if let Some(io_err) = e.downcast_ref::<io::Error>() {
                    if io_err.kind() == io::ErrorKind::UnexpectedEof {
                        println!("A client probably disconnected, continuing...");
                        break;
                    }
                }
                eprintln!("Error in client_listener: {:?} [IN UKNOWN ERROR MATCH", e);
                break;
            }
        };

        println!(
            "Server rx'd message from client {}: {:?}\n\tPreparing broadcast...",
            &client_addr, msg
        );

        // Broadcast message out to everyone but the original sender
        broadcast_message(msg, Arc::clone(clients), client_addr)?;
    }

    Ok(())
}

fn add_client(
    client_map: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    client: &SocketAddr,
    stream: TcpStream,
) {
    println!("\tattempting to lock clients to insert new client");

    let mut clients_guard = client_map.lock().unwrap();
    // QUESTION: Is it smells bad to deref the SocketAddr like this?
    clients_guard.insert(*client, stream.try_clone().unwrap());
    println!("\tServer added client connection: {}", &client);
    //dbg!("{:?}", &clients_guard);
}

fn drop_client(client_map: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>, client: &SocketAddr) {
    println!("\tattempting to lock clients to drop old client");

    let mut clients_guard = client_map.lock().unwrap();
    clients_guard.remove(client);
    println!("\tServer dropped client: {}", &client);
    //dbg!("{:?}", &clients_guard);
}

fn broadcast_message(
    message: MessageType,
    clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    sender_addr: SocketAddr,
) -> Result<(), ServerError> {
    //println!("Entering server::broadcast_message()");
    println!("\tattempting to lock clients to broadcast to clients");
    let clients_guard = clients.lock().unwrap();

    for (addr, client_stream) in clients_guard.iter() {
        if *addr != sender_addr {
            let mut stream = client_stream.try_clone()?;
            let _ = send_message(&mut stream, message.clone());
        }
    }

    //println!("Exiting server::broadcast_message()");
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
