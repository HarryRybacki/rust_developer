use chrono::Local;
use common::{send_message, MessageType};
use std::{
    error::Error,
    fs,
    io::{self, Write},
    net::TcpStream,
    path,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

pub fn run_client(server_address: &str) -> Result<(), Box<dyn Error>> {
    //println!("Entering client::run_client()");

    // Connect to the server
    let mut stream = TcpStream::connect(server_address)?;

    // Launch listener thread to handle messages from the server
    let listner_stream = stream.try_clone()?;
    let should_listen = Arc::new(AtomicBool::new(true));
    let should_listen_clone = Arc::clone(&should_listen);

    thread::spawn(move || {
        while should_listen_clone.load(Ordering::SeqCst) {
            //println!("Client listener thread is running...");
            //dbg!("{:?}", should_listen_clone.load(Ordering::SeqCst));
            match client_listener(
                listner_stream.try_clone().unwrap(),
                Arc::clone(&should_listen_clone),
            ) {
                Ok(()) => println!("client listener succesfully returned thread"),
                Err(e) => eprintln!("client listener encountered error within thread: {}", e),
            }
        }
        //println!("Client listener thread is stopping...");
    });

    // Display client usage
    client_usage();
    // Read input from stdin
    //println!("run_client() beginning loop on stdin");
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
                    println!("[SENDING FILE] {}", &file_name);
                    MessageType::File(String::from(file_name), data)
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    return Err(Box::new(e));
                }
            },
            Command::Image => match fs::read(parts[1]) {
                Ok(data) => {
                    println!("[SENDING IMAGE] {}", &parts[1]);
                    MessageType::Image(data)
                }
                Err(e) => {
                    eprintln!("Error reading image file: {}", e);
                    return Err(Box::new(e));
                }
            },
            Command::Text => {
                let message = parts.join(" ");
                println!("[SENT] {}", &message);
                MessageType::Text(message)
            }
        };

        // Send the message
        send_message(&mut stream, message)?;
    }
    //println!("run_client() ended loop on stdin");

    // Tell the listener thread to halt
    should_listen.store(false, Ordering::SeqCst);

    //println!("Exiting client::main()");
    Ok(())
}

fn client_listener(
    mut stream: TcpStream,
    should_listen: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    while should_listen.load(Ordering::SeqCst) {
        // Use a non-blocking read with a timeout
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;

        let _ = match common::receive_message(&mut stream) {
            Ok(msg) => {
                //println!("Client received message from server");
                match msg {
                    MessageType::File(filename, data) => save_file(filename, data),
                    MessageType::Image(image) => save_image(image),
                    MessageType::Text(message) => save_text(message),
                }
            }
            Err(ref e) => {
                // black magic to let server loop
                if let Some(io_err) = e.downcast_ref::<io::Error>() {
                    // Note: The receive_message() will return WouldBlock periodically
                    //       to make sure the stream hasn't been closed. We want the
                    //       listener to continue listening in this case, but break if not
                    if io_err.kind() == io::ErrorKind::WouldBlock {
                        //println!("No data available, continuing...");
                        continue;
                    }
                }
                eprintln!("Error in client_listener: {:?}", e);
                break;
            }
        };
    }

    Ok(())
}

fn save_text(message: String) -> Result<(), Box<dyn Error>> {
    println!("[RECEIVED] {}", message);
    Ok(())
}

fn save_image(image: Vec<u8>) -> Result<(), Box<dyn Error>> {
    // Saves a byte array as an image locally
    // Assumes filetype is `.png` and storing in `./images/` dir
    // Returns Result of Ok or Error

    let file_name = generate_file_name();

    let mut file = fs::File::create(&file_name)?;
    file.write_all(&image)?;

    println!("[RECEIVED IMAGE] Saving to..: {}", file_name);
    Ok(())
}

fn save_file(file_name: String, data: Vec<u8>) -> Result<(), Box<dyn Error>> {
    // Saves a byte array as a file locally
    // Assumes filename includes extension and storing in `./files/` dir
    // Returns Result of Ok or Error

    // Attempt to create the path
    let path = std::path::Path::new("./files");
    fs::create_dir_all(path)?;

    // Create the file
    let file_path = path.join(&file_name);
    let mut file = fs::File::create(&file_path)?;
    file.write_all(&data)?;

    // Save the file
    let file_path_str = file_path
        .to_str()
        .expect("Error encountered after saving file locally...");

    println!("[RECEIVED FILE] Saving to..: {}", file_path_str);
    Ok(())
}

fn generate_file_name() -> String {
    // Creates String representing a file's name
    // Assumes filetype is `.png` and storing in `./images/` dir
    // Returns String or Error

    // Attempt to create the file path
    let path = path::Path::new("./images");
    fs::create_dir_all(path).unwrap();
    let now = Local::now();

    format!("./images/{}.png", now.format("%Y%m%d%H%M%S"))
}

fn client_usage() {
    println!(
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
