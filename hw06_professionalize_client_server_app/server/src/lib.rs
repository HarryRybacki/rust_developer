use std::{
    collections::HashMap,
    fmt, io,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use common::{send_message, MessageType};

/// Establishes a TcpListener to receive connections from Client's then opens a new
/// thread for handling messages coming from those connections.
///
/// # Errors
/// Function will propogate up a ServerError if there is an issue processing the stream's
/// message.
pub fn listen_and_accept(address: &str) -> Result<(), ServerError> {
    log::trace!("Entering server::listen_and_accept()");

    // Establish TcpListener to capture incoming streams
    let listener = TcpListener::bind(address)?;

    // Create a threadsafe HashMap to track clients connected to the server
    let clients = Arc::new(Mutex::new(HashMap::<SocketAddr, TcpStream>::new()));

    // Iterate over incoming streams and handle connections
    for stream in listener.incoming() {
        log::info!("Server is opening a new stream");

        // Unwrap stream, note peer address, and clone Clients for the thread
        let stream = stream?;
        let peer_addr = stream.peer_addr()?;
        let inner_clients = Arc::clone(&clients);

        // Create new thread to manage handle_client
        thread::spawn(move || match handle_client(stream, &inner_clients) {
            Ok(()) => {
                let _ = drop_client(&inner_clients, &peer_addr);
                log::info!("Client handled succesfully within thread. Exiting...")
            }
            Err(e) => {
                let _ = drop_client(&inner_clients, &peer_addr);
                log::error!("Encountered server error within thread: {}", e)
            }
        });
    }

    log::trace!("Exiting server::listen_and_accept()");
    Ok(())
}

/// Loops over a TcpStream and handles incoming messsages from a client's
/// connection then broadcasts that message to every other client.
///
/// # Errors
/// Function will return a Server error if an issue is encountered receiving
/// the message or reading from the stream.
fn handle_client(
    mut stream: TcpStream,
    clients: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
) -> Result<(), ServerError> {
    log::trace!("Entering server::handle_client()");

    // Clone clients HashMap and add the stream
    let inner_clients = Arc::clone(clients);
    log::debug!("A new clone of clients has been made.");
    let client_addr = stream.peer_addr()?;

    // Add new client to the Servers HashMap
    let _ = add_client(&inner_clients, &client_addr, stream.try_clone()?);

    loop {
        let msg = match common::receive_message(&mut stream) {
            Ok(msg) => {
                log::debug!("returned from common::receive_message() [IN OK MATCH]");
                msg
            }
            Err(ref e) => {
                if let Some(io_err) = e.downcast_ref::<io::Error>() {
                    // Note: Receive_message() will return WouldBlock periodically
                    //       to make sure the stream hasn't been closed. We want the
                    //       server to continue listening in this case
                    if io_err.kind() == io::ErrorKind::WouldBlock {
                        continue;
                    }
                }
                if let Some(io_err) = e.downcast_ref::<io::Error>() {
                    if io_err.kind() == io::ErrorKind::UnexpectedEof {
                        log::info!("A client probably disconnected, continuing...");
                        break;
                    }
                }
                log::error!("Error in client_listener: {:?} [IN UKNOWN ERROR MATCH", e);
                break;
            }
        };

        // Broadcast message out to everyone but the original sender
        broadcast_message(msg, Arc::clone(clients), client_addr)?;
    }
    log::trace!("Exiting server::handle_client()");

    Ok(())
}

/// Attempts to add a Client's TcpStream and SocketAddr to the Server's HashMap
/// before informing all other connected Clients of the new one.
///
/// # Errors
/// Function will return a ServerError if an issue is encountered locking the
/// Clients HashMap or if there is an issue broadcasting the message.
fn add_client(
    client_map: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    client: &SocketAddr,
    stream: TcpStream,
) -> Result<(), ServerError> {
    {
        // Wrap in expression so the guard is returned immediatly after completing its insert
        log::debug!("Attempting to lock clients to insert new client");
        let mut clients_guard = client_map.lock().unwrap();
        clients_guard.insert(*client, stream.try_clone().unwrap());
        log::info!("Server added client connection: {}", &client);
    }

    // Inform other clients a new user has joined
    let msg = MessageType::Text(format!(
        "<SERVER MSG> A new user has joined the server: {}",
        client
    ));
    broadcast_message(msg, Arc::clone(client_map), *client)?;

    Ok(())
}
/// Attempts to remove a Client's TcpStream and SocketAddr to the Server's HashMap
/// before informing all other connected Clients they have left.
///
/// # Errors
/// Function will return a ServerError if an issue is encountered locking the
/// Clients HashMap or if there is an issue broadcasting the message.
fn drop_client(
    client_map: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    client: &SocketAddr,
) -> Result<(), ServerError> {
    {
        // Wrap in expression so the guard is returned immediatly after completing its remove
        log::debug!("Attempting to lock clients to drop old client");
        let mut clients_guard = client_map.lock().unwrap();
        clients_guard.remove(client);
        log::info!("Server dropped client: {}", &client);
    }

    // Inform other clients a user has left
    let msg = MessageType::Text(format!(
        "<SERVER MSG> User: {}, has left the server",
        client
    ));
    broadcast_message(msg, Arc::clone(client_map), *client)?;

    Ok(())
}

/// Broadcasts a MessageType to all clients other than the original sender.
///
/// # Errors
/// Function will return a ServerError if an issue is encountered locking the
/// Clients HashMap or while sending messages to clients.
fn broadcast_message(
    message: MessageType,
    clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    sender_addr: SocketAddr,
) -> Result<(), ServerError> {
    log::trace!("Entering server::broadcast_message()");
    log::debug!("Attempting to lock clients to broadcast to clients");
    let clients_guard = clients.lock().unwrap();

    for (addr, client_stream) in clients_guard.iter() {
        if *addr != sender_addr {
            let mut stream = client_stream.try_clone()?;
            let _ = send_message(&mut stream, message.clone());
        }
    }

    log::trace!("Exiting server::broadcast_message()");
    Ok(())
}

/// Leverage a custom error type in Server to keep things thread safe.
/// Generic Box<dyn Errors> were causing chaos with the compiler after
/// makin thigs multi-threaded.
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
