use std::{
    collections::HashMap,
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
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
    common::deseralize_msg(&buffer)
}
