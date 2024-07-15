use anyhow::{Context, Result};
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::{get_hostname, receive_msg, MessageType};
use std::{env, net::SocketAddr};
use sqlx::{migrate::MigrateDatabase, Pool, Row, Sqlite, SqlitePool};
use tokio::{
    self,
    io::AsyncReadExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
    sync,
};

// Using as lightweight a DB as possible
const DB_URL: &str = "sqlite://sqlite.db";

#[tokio::main]
async fn main() -> Result<()> {
    // Establish our logger
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();

    // Create sqlite DB if it's not already present
    let db = setup_db().await?;


    // Process parameters to determine hostname and whatnot for Server
    let args: Vec<String> = env::args().collect();
    let address = get_hostname(args);
    log::info!("Launching server on address: {}", address);

    // Create tokio listener to establish client connections
    let listener = TcpListener::bind(address)
        .await
        .context("Failed to bind to socket.")?;

    // Create broadcast channel to share messages between client connections
    let (br_send, _br_recv) = sync::broadcast::channel(1024);

    // Initiate accept loop for server
    loop {
        // Capture the incoming socket and address; continue looping if connection fails
        let Ok((stream, addr)) = listener.accept().await else {
            log::error!("Failed to connect to client socket.");
            continue;
        };

        log::debug!("New client connection: {}", &addr);

        // Clone the send and create a subscriber. Pass these to the task managing writing to this client's tcp stream. This is the heart of the routing mechanism for these messages
        let sender = br_send.clone();
        let receiver = sender.subscribe();
        let db_clone_rdr = db.clone();
        let db_clone_wtr = db.clone();
        // Split stream into separate reader and writer; we want independent mut refs to pass to separate tokio tasks
        let (mut stream_rdr, mut stream_wtr) = stream.into_split();

        // Spawn tokio task to manage reading from the client
        tokio::spawn(async move {
            process_client_rdr(&sender, stream_rdr, addr, &db_clone_rdr)
                .await
                .context("Server error handling the client reader")
                .unwrap();
        });

        // Spawn tokio task to manage writing to the client
        tokio::spawn(async move {
            process_client_wtr(receiver, &mut stream_wtr, addr, &db_clone_wtr)
                .await
                .context("Server error handling the client writer")
                .unwrap();
        });
    }

    Ok(())
}

async fn process_client_rdr(
    tx: &sync::broadcast::Sender<(MessageType, SocketAddr)>,
    mut client_stream: OwnedReadHalf,
    addr: SocketAddr,
    db: &Pool<Sqlite>,
) -> Result<()> {
    log::trace!("Starting process: Client Reader for: {}", &addr);
    let mut length_bytes = [0; 4];

    // TODO: Add client to DB

    loop {
        // TODO: read_exact is blocking IIRC, should this task calling this function be `is_blocking` or something?
        match client_stream
            .read_exact(&mut length_bytes)
            .await
            .context("Failed to read length")
        {
            Ok(_) => {
                let msg_len = u32::from_be_bytes(length_bytes) as usize;

                log::debug!(
                    "Attempting to retrieve a {}-byte message from {}:",
                    msg_len.to_string(),
                    addr
                );
                let msg = receive_msg(&mut client_stream, msg_len)
                    .await
                    .context("Failed to read message")?;

                // "Wake up" the the writer task and have it handle messaging the clients
                if tx.send((msg.clone(), addr)).is_err() {
                    log::error!(
                        "Something when wrong sending the message down the broadast channel..."
                    );
                }

                continue;
            }
            Err(e) => {
                // TODO: Handle the `early eof` errors caused by clients dropping
                log::error!(
                    "Error reading from {}: {:?}\nLikely a client disconnect. Dropping client.",
                    addr,
                    e
                );
                // FIXME: Handle client disconnects.
                break;
            }
        }
    }

    // Drop client from DB
    Ok(())
}

async fn process_message(msg: MessageType) -> Result<()> {
    todo!();
}

async fn process_client_wtr(
    mut rx: sync::broadcast::Receiver<(MessageType, SocketAddr)>,
    stream: &mut OwnedWriteHalf,
    addr: SocketAddr,
    db: &Pool<Sqlite>,
) -> Result<()> {
    log::trace!("Starting process: Client Writer for: {}", &addr);

    while let Ok((msg, other_addr)) = rx.recv().await {
        // If this is the task responsible for sending to the same client the msg came from, ignore
        if other_addr == addr {
            log::debug!(
                "Will not broadcast message from: {} to {}. Same client.",
                other_addr,
                addr
            );
            continue;
        }

        // Otherwise send it to their resepctive TCP Stream
        match msg.send(stream).await {
            Ok(_) => {
                log::debug!("Server successfully sent message to: {}", addr);
            }
            Err(e) => {
                log::error!("Error sending msg to {} tcp stream: {:?}", &addr, e);
                log::info!("Server killing client writer task for: {}", addr);
                break;
            }
        }
        continue;
    }

    Ok(())
}

async fn setup_db() -> Result<Pool<Sqlite>> {

    // Create DB if it doesn't already exist
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        log::trace!("Entering setup_db");
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => {
                log::info!("New sqlite DB established: {}", DB_URL);
            },
            Err(error) => {
                log::error!("Error creating new DB: {}", &error);
                // TODO: How do we want to handle the error state?
            },
        }
    } 

    // Establish connection to DB
    let db = SqlitePool::connect(DB_URL)
        .await
        .context("Failed to connect to SQLite DB")?;

    // Execute migrations
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").context("Failed to determine CARGO_MANIFEST_DIR")?;
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");
    match sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(&db)
        .await {
            Ok(results) => log::info!("Migration success: {:?}", results),
            Err(error) => {
                // FIXME: Implement failure logic
                log::error!("{:?}", error);
                todo!();
        }
    }

    // START TESTING
    let result = sqlx::query(
        "SELECT name
         FROM sqlite_schema
         WHERE type ='table' 
         AND name NOT LIKE 'sqlite_%';",
    )
        .fetch_all(&db)
        .await
        .unwrap();
    for (idx, row) in result.iter().enumerate() {
        log::info!("[{}]: {:?}", idx, row.get::<String, &str>("name"));
    }
    
    // TODO: Turn into add_user fn
    let result = sqlx::query("INSERT INTO users (name) VALUES (?)")
        .bind("Tim Tom")
        .execute(&db)
        .await
        .unwrap();

    println!("Query result: {:?}", result);

    // TOOD: Turn into get_users fn
    let user_results = sqlx::query_as::<_, User>("SELECT id, name FROM users")
        .fetch_all(&db)
        .await
        .unwrap();

    for user in user_results {
        println!("[{}] name: {}", user.id, &user.name);
    }

    // TODO: Turn into delete_use fn
    let delete_result = sqlx::query("DELETE FROM users WHERE name=$1")
        .bind("Tim Tom")
        .execute(&db)
        .await
        .unwrap();
    println!("Delete result: {:?}", delete_result);

    // END TESTING

    Ok(db)
}
