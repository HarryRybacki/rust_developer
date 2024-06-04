use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    io,
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
pub fn listen_and_accept(address: &str) -> Result<()> {
    log::trace!("Entering listen_and_accept()");

    // Establish TcpListener to capture incoming streams
    let listener = TcpListener::bind(address).context("Failed to bind to listening address.")?;

    // Create a threadsafe HashMap to track clients connected to the server
    let clients = Arc::new(Mutex::new(HashMap::<SocketAddr, TcpStream>::new()));

    // Iterate over incoming streams and handle connections
    for stream in listener.incoming() {
        log::info!("Server is opening a new stream");

        // Unwrap stream, note peer address, and clone Clients for the thread
        let stream = stream.context("Failed to open stream.")?;
        let peer_addr = stream
            .peer_addr()
            .context("Failed to grab client's peer address.")?;
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

    log::trace!("Exiting listen_and_accept()");
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
) -> Result<()> {
    log::trace!("Entering handle_client()");

    // Clone clients HashMap and add the stream
    let inner_clients = Arc::clone(clients);
    log::debug!("A new clone of clients has been made.");
    let client_addr = stream
        .peer_addr()
        .context("Failed to grab client's peer address.")?;

    // Add new client to the Servers HashMap
    let _ = add_client(
        &inner_clients,
        &client_addr,
        stream
            .try_clone()
            .context("Failed to clone client stream.")?,
    );

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
    log::trace!("Exiting handle_client()");

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
) -> Result<()> {
    {
        // Wrap in expression so the guard is returned immediatly after completing its insert
        log::debug!("Attempting to lock clients to insert new client");
        let mut clients_guard = client_map.lock().unwrap();
        let stream_clone = stream.try_clone().context("Failed cloning client stream.");
        clients_guard.insert(*client, stream_clone.unwrap());
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
) -> Result<()> {
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
) -> Result<()> {
    log::trace!("Entering broadcast_message()");
    log::debug!("Attempting to lock clients to broadcast to clients");
    let clients_guard = clients.lock().unwrap();

    for (addr, client_stream) in clients_guard.iter() {
        if *addr != sender_addr {
            let mut stream = client_stream
                .try_clone()
                .context("Failed to clone client's stream.")?;
            send_message(&mut stream, message.clone())?;
        }
    }

    log::trace!("Exiting broadcast_message()");
    Ok(())
}
