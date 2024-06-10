use anyhow::{anyhow, Context, Result};
use chrono::Local;
use common::{send_message, AppError, MessageType};
use std::{
    error::Error,
    fs,
    io::Write,
    net::TcpStream,
    path, process,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

/// Runner function for clients.
///
/// Maintains a TcpStream with a remote Server. Spawns a listening thread to
/// handle receiving messages from the Server. Loops on stdin to process
/// User request to send messages or disconnect from the Server.
///
/// # Errors
/// Function will propogate up Errors.
/// - Connecting to the Server's TcpStream
/// - Handling incoming messages from Server
/// - Processing User requests to send messages
pub fn run_client(server_address: &str) -> Result<()> {
    log::trace!("Entering client::run_client()");

    // Connect to the Server
    let mut stream = TcpStream::connect(server_address)?;

    // Launch listener thread to handle messages from the server
    let listner_stream = stream.try_clone()?;
    let should_listen = Arc::new(AtomicBool::new(true));
    let should_listen_clone = Arc::clone(&should_listen);

    thread::spawn(move || {
        while should_listen_clone.load(Ordering::SeqCst) {
            log::trace!("Client listener thread is running...");
            match client_listener(
                listner_stream.try_clone().unwrap(),
                Arc::clone(&should_listen_clone),
            ) {
                Ok(()) => log::debug!("client listener succesfully returned thread"),
                Err(e) => log::error!("client listener encountered error within thread: {}", e),
            }
        }
        log::trace!("Client listener thread has halted...");
    });

    // Display client usage
    client_usage();
    log::trace!("run_client() beginning loop on stdin");
    // Process User requests from stdin
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        // Identify command and possible message type
        let trimmed_input = input.trim();
        let parts: Vec<&str> = trimmed_input.splitn(2, ' ').collect();
        let command = Command::from_str(parts[0])?;

        // Prepare user message to send
        let message: MessageType = match command {
            Command::Quit => break,
            Command::Help => {
                client_usage();
                continue;
            }
            Command::File => match fs::read(parts[1]) {
                Ok(data) => {
                    let file_name = path::Path::new(parts[1])
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap();
                    log::info!("[SENDING FILE] {}", &file_name);
                    MessageType::File(String::from(file_name), data)
                }
                Err(e) => {
                    log::error!("Error reading file: {}", e);
                    return Err(anyhow!("Error: {}", e));
                }
            },
            Command::Image => match fs::read(parts[1]) {
                Ok(data) => {
                    log::info!("[SENDING IMAGE] {}", &parts[1]);
                    MessageType::Image(data)
                }
                Err(e) => {
                    log::error!("Error reading image file: {}", e);
                    return Err(anyhow!("Error: {}", e));
                }
            },
            Command::Text => {
                let message = parts.join(" ");
                log::info!("[SENT] {}", &message);
                MessageType::Text(message)
            }
        };

        // Send the message
        send_message(&mut stream, message)?;
    }
    log::debug!("run_client() ended loop on stdin");

    // Tell the listener thread to halt
    should_listen.store(false, Ordering::SeqCst);

    log::trace!("Exiting client::main()");
    Ok(())
}

/// Loops over a TcpStream and handles incoming messsages from the server.
/// a client's. Will halt when `should_listen` is set to False.
///
/// # Errors
/// Function will propogate up Errors returned from receive_message.
/// It is expected that receive_message() will return a WouldBlock periodically
/// to avoid IO blocking. In this case, we just restart the main loop.
fn client_listener(mut stream: TcpStream, should_listen: Arc<AtomicBool>) -> Result<()> {
    while should_listen.load(Ordering::SeqCst) {
        // Use a non-blocking read with a timeout to avoid IO blocking
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;

        match common::receive_message(&mut stream) {
            Ok(msg) => {
                log::trace!("Client received message from server");
                match msg {
                    MessageType::File(filename, data) => save_file(filename, data)?,
                    MessageType::Image(image) => save_image(image)?,
                    MessageType::Text(message) => save_text(message)?,
                }
            }
            Err(AppError::WouldBlock) => continue,
            Err(AppError::Disconnected) => {
                log::info!("The server disconnected, shutting down the client...");
                println!("Server has shutdown. \nExiting...");
                process::exit(0)
            }
            Err(e) => {
                log::error!("Error in client_listener: {:?} [IN UNKNOWN ERROR MATCH]", e);
                break;
            }
        };
    }

    Ok(())
}

/// Displays text from a recieved message.
///
/// Returns Result of Ok or Error.
fn save_text(message: String) -> Result<()> {
    log::info!("[RECEIVED] {}", message);
    Ok(())
}

/// Saves a byte array as an image locally.
///
/// Assumes filetype is `.png` and storing in `./images/` dir.
///
/// Returns Result of Ok or Error.
fn save_image(image: Vec<u8>) -> Result<()> {
    let file_name = generate_file_name();

    let mut file = fs::File::create(&file_name)
        .with_context(|| format!("Failed to open file: {}", &file_name))?;
    file.write_all(&image)
        .context("Failed to save image locally")?;

    log::info!("[RECEIVED IMAGE] Saving to..: {}", file_name);
    Ok(())
}

/// Saves a byte array as a file locally.
///
/// Assumes filename includes extension and storing in `./files/` dir.
///
/// Returns Result of Ok or Error.
fn save_file(file_name: String, data: Vec<u8>) -> Result<()> {
    // Attempt to create the path
    let path = std::path::Path::new("./files");
    fs::create_dir_all(path).context("Failed to create directory.")?;

    // Create and save the file
    let file_path = path.join(file_name);
    let file_path_str = &file_path
        .to_str()
        .context("Failed to convert file path to string")?;
    let mut file = fs::File::create(&file_path)
        .with_context(|| format!("Failed to create file: {}", &file_path_str))?;
    file.write_all(&data)
        .context("Failed to write file to local storage.")?;

    log::info!("[RECEIVED FILE] Saving to..: {}", file_path_str);
    Ok(())
}

/// Creates String representing a file's name.
///
/// Assumes filetype is `.png` and storing in `./images/` dir.
///
/// Returns String or Error.
fn generate_file_name() -> String {
    // Attempt to create the file path
    let path = path::Path::new("./images");
    fs::create_dir_all(path).unwrap();
    let now = Local::now();

    format!("./images/{}.png", now.format("%Y%m%d%H%M%S"))
}

/// Displays Client usage helper text.
fn client_usage() {
    log::info!(
        "
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
