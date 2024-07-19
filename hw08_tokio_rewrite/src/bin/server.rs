use anyhow::{Context, Result};
use chrono::Utc;
use env_logger::{Builder, Env};
use hw08_tokio_rewrite::{get_hostname, receive_msg, MessageType};
use sqlx::{migrate::MigrateDatabase, Pool, Row, Sqlite, SqlitePool};
use std::{env, net::SocketAddr};
use tokio::{
    self,
    io::AsyncReadExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener,
    },
    sync::{self, mpsc},
};

// Using as lightweight a DB as possible
const DB_URL: &str = "sqlite://sqlite.db";

// Enum to represent messages, including user ID updates
enum InternalMessage {
    UserIdUpdate(i64),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Establish our logger
    let env = Env::default().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();

    // Create sqlite DB if it's not already present
    let db = setup_db().await?;

    // Determine the anonymous user's ID
    let anon_user_id = get_or_create_anon_user_id(&db).await?;

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
        let (stream_rdr, mut stream_wtr) = stream.into_split();

        // Channel to handle internal messages
        let (internal_tx, internal_rx) = mpsc::channel(32);
        let internal_tx_rdr = internal_tx.clone();

        // Spawn tokio task to manage reading from the client
        tokio::spawn(async move {
            process_client_rdr(
                &sender,
                stream_rdr,
                addr,
                &db_clone_rdr,
                internal_tx_rdr,
                anon_user_id,
            )
            .await
            .context("Server error handling the client reader")
            .unwrap();
        });

        // Spawn tokio task to manage writing to the client
        tokio::spawn(async move {
            process_client_wtr(receiver, &mut stream_wtr, addr, &db_clone_wtr, internal_rx)
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
    internal_tx: mpsc::Sender<InternalMessage>,
    mut user_id: i64,
) -> Result<()> {
    log::trace!("Starting process: Client Reader for: {}", &addr);
    let mut length_bytes = [0; 4];

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
                    "Attempting to retrieve a {}-byte message from {} at {}:",
                    msg_len.to_string(),
                    user_id,
                    addr
                );
                let msg = receive_msg(&mut client_stream, msg_len)
                    .await
                    .context("Failed to read message")?;

                // If msg type is a register, attempt to add the user to the DB
                let updated_msg = process_message(&msg, &mut user_id, db, &internal_tx)
                    .await
                    .context("Failed to process message")?;

                // "Wake up" the the writer task and have it handle messaging the clients
                if tx.send((updated_msg.clone(), addr)).is_err() {
                    log::error!(
                        "Something when wrong sending the message down the broadast channel..."
                    );
                }

                continue;
            }
            Err(e) => {
                // TODO: Handle the `early eof` errors caused by clients dropping
                log::error!(
                    "Error reading from user {} at {}: {:?}\nLikely a client disconnect. Dropping client.",
                    user_id,
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

/// Processes incoming messages and handles things like DB registrations as needed
async fn process_message(
    msg: &MessageType,
    user_id: &mut i64,
    db: &Pool<Sqlite>,
    internal_tx: &mpsc::Sender<InternalMessage>,
) -> Result<MessageType> {
    match msg {
        MessageType::Register(account) => {
            add_user_to_db(account, db)
                .await
                .context("Failed to register account and add to the user database")?;

            // Retrieve the new user ID and update the user_id mutable reference
            if let Some(new_user_id) = get_user_id_by_name(account, db).await? {
                *user_id = new_user_id;
                internal_tx
                    .send(InternalMessage::UserIdUpdate(new_user_id))
                    .await
                    .unwrap();
            }
            Ok(MessageType::Register(account.clone()))
        }
        _ => {
            let username = get_username_by_id(*user_id, db).await?.unwrap_or_else(|| "anonymous".to_string());
            let updated_msg = match msg {
                MessageType::Text(_, content) => MessageType::Text(Some(username), content.clone()),
                MessageType::File(_, file_name, data) => MessageType::File(Some(username), file_name.clone(), data.clone()),
                MessageType::Image(_, data) => MessageType::Image(Some(username), data.clone()),
                MessageType::Register(_) => unreachable!(),
            };

            store_message_in_db(&updated_msg, *user_id, db).await?;
            Ok(updated_msg)
        }
    }

}

async fn process_client_wtr(
    mut rx: sync::broadcast::Receiver<(MessageType, SocketAddr)>,
    stream: &mut OwnedWriteHalf,
    addr: SocketAddr,
    db: &Pool<Sqlite>,
    mut internal_rx: mpsc::Receiver<InternalMessage>,
) -> Result<()> {
    log::trace!("Starting process: Client Writer for: {}", &addr);

    // Store the current user ID
    let mut user_id: i64 = 1;

    loop {
        tokio::select! {
            // Handle broadcast messages
            Ok((msg, other_addr)) = rx.recv() => {
                // If this is the task responsible for sending to the same client the msg came from, ignore
                if other_addr == addr {
                    log::debug!(
                        "Will not broadcast message from: {} to {}. Same client.",
                        other_addr,
                        addr
                    );
                    continue;
                }

                // Otherwise send it to their respective TCP Stream
                match msg.send(stream).await {
                    Ok(_) => {
                        log::debug!("Server successfully sent message to: {} at {}", user_id, addr);
                    }
                    Err(e) => {
                        log::error!("Error sending msg to {} tcp stream: {:?}", &addr, e);
                        log::info!("Server killing client writer task for: {} at {}", user_id, addr);
                        break;
                    }
                }
            },
            // Handle internal messages
            Some(internal_msg) = internal_rx.recv() => {
                match internal_msg {
                    InternalMessage::UserIdUpdate(new_user_id) => {
                        user_id = new_user_id;
                        log::debug!("Updated user_id to: {}", user_id);
                    },
                }
            }
        }
    }

    Ok(())
}

/// Establishes the DB for server use
async fn setup_db() -> Result<Pool<Sqlite>> {
    // Create DB if it doesn't already exist
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        log::trace!("Entering setup_db");
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => {
                log::info!("New sqlite DB established: {}", DB_URL);
            }
            Err(error) => {
                log::error!("Error creating new DB: {}", &error);
                // Handle the error state as needed
            }
        }
    }

    // Establish connection to DB
    let db = SqlitePool::connect(DB_URL)
        .await
        .context("Failed to connect to SQLite DB")?;

    // Execute migrations
    let crate_dir =
        std::env::var("CARGO_MANIFEST_DIR").context("Failed to determine CARGO_MANIFEST_DIR")?;
    let migrations = std::path::Path::new(&crate_dir).join("migrations");
    let migrator = sqlx::migrate::Migrator::new(migrations).await.unwrap();
    match migrator.run(&db).await {
        Ok(_) => log::info!("Migration success"),
        Err(error) => {
            log::error!("Migration error: {:?}", error);
            return Err(error.into());
        }
    }

    // FIXME: Figure out wtf the messages table isn't being migrated
    // Manually create the messages table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            content TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        );",
    )
    .execute(&db)
    .await
    .context("Failed to create messages table")?;

    log::info!("Messages table created or already exists");

    Ok(db)
}

/// Adds a user to the database
async fn add_user_to_db(account: &str, db: &Pool<Sqlite>) -> Result<()> {
    sqlx::query("INSERT INTO users (name) VALUES (?)")
        .bind(account)
        .execute(db)
        .await
        .context("Failed to insert user into the database")?;
    log::debug!("User {} added to the database", account);

    Ok(())
}

/// Adds a message to the database associated with a specific user_id
async fn store_message_in_db(msg: &MessageType, user_id: i64, db: &Pool<Sqlite>) -> Result<()> {
    match msg {
        MessageType::Text(_, content) | MessageType::Text(None, content)=> {
            sqlx::query("INSERT INTO messages (content, user_id) VALUES (?, ?)")
                .bind(content)
                .bind(user_id)
                .execute(db)
                .await
                .context("Failed to insert text message into the database")?;
        }
        MessageType::File(_, name, _) | MessageType::File(None, name, _) => {
            sqlx::query("INSERT INTO messages (content, user_id) VALUES (?, ?)")
                .bind(name)
                .bind(user_id)
                .execute(db)
                .await
                .context("Failed to insert file message into the database")?;
        }
        MessageType::Image(_, _) | MessageType::Image(None, _) => {
            let timestamp = Utc::now().to_string();
            sqlx::query("INSERT INTO messages (content, user_id) VALUES (?, ?)")
                .bind(timestamp)
                .bind(user_id)
                .execute(db)
                .await
                .context("Failed to insert image message into the database")?;
        }
        MessageType::Register(_) => return Ok(()), // Should not be storing Register messages
    }

    log::debug!("Message stored in the database with user ID: {}", user_id);
    Ok(())
}

/// Fetches, or creates a new, user_id for the anon user
async fn get_or_create_anon_user_id(db: &Pool<Sqlite>) -> Result<i64> {
    // Check if the anonymous user exists
    let row = sqlx::query("SELECT id FROM users WHERE name = 'anonymous'")
        .fetch_optional(db)
        .await?;

    if let Some(row) = row {
        Ok(row.get("id"))
    } else {
        // Create the anonymous user if it does not exist
        sqlx::query("INSERT INTO users (name) VALUES ('anonymous')")
            .execute(db)
            .await?;
        let row = sqlx::query("SELECT id FROM users WHERE name = 'anonymous'")
            .fetch_one(db)
            .await?;

        let user_id = row.get("id");
        log::debug!(
            "Created and added 'anonymous' user to db, user_id: {}",
            &user_id
        );

        Ok(user_id)
    }
}

async fn get_user_id_by_name(account: &str, db: &Pool<Sqlite>) -> Result<Option<i64>> {
    let row = sqlx::query("SELECT id FROM users WHERE name = ?")
        .bind(account)
        .fetch_optional(db)
        .await?;
    if let Some(row) = row {
        return Ok(Some(row.get("id")));
    }
    Ok(None)
}

async fn get_username_by_id(user_id: i64, db: &Pool<Sqlite>) -> Result<Option<String>> {
    let row = sqlx::query("SELECT name FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await?;
    Ok(row.map(|r| r.get("name")))
}