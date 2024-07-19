# [Homework 8](https://robot-dreams-rust.mag.wiki/13-error-handling-custom-types/index.html#homework)

## Server and Client Usage:

### Server

> [!WARNING]
> Ensure you have sqlite installed on your local machine. It is required by the server.

Launching the server is quite simple, from the packages root directory: 
    
    `RUST_LOG=<log level> cargo run --bin server <listening ip> <listening port>`
e.g.
    `RUST_LOG=debug cargo run --bin server 127.0.0.1 8080`

### Client
Launching the client is just as simple:
    `cargo run --bin client <server ip> <server port>`
e.g.
    `cargo run --bin client 127.0.0.1 8080`

For client usage, invoke `.usage` after launching.

### Questions:
n/a

### Class Notes:
n/a

### Reflections for Lukáš and self:

#### Async-Re-write
**This re-write took a lot longer anticipated.**

Understanding the flow of asynchronous programming was one hurdle. After attempting to refactor my application twice, I ended up opting for a full re-write (leveraging 
old code where it made sense to.)

The Server and Client both ended up in their own binary files. 

The Server loops over new client connections. For each new connection it:
- Creates a channel to communicate between the Tasks handling the client read/write connections
- Creates a channel for passing updated user ids when a client opts to register themselves
- Spawns a Task for reading from and a Task for witing to the client connection.

The Client spawns three Tasks:
- One to listening and handling input from stdin
- One for writing to the server's stream
- And one for handling incoming messages from the server's stream
Additionally, the client leverages a CancellationToken for gracefully shutting down each task if the user
decides it's time to `.quit` or if the server drops unexpectedly for some reason.

#### Database Additions
The database addition is pretty simple. 

I leveraged sqlite with two tables, one for messages and another for users. They are linked to one another via the user_id. Message text is stored, as are filenames for files. Images merely not that the associated user sent an image.

The migrations seem to work for you, but not for me. To resolve this, I have the setup_db function manually creating the table (assuming the migration didn't work).

#### Security Considerations and Users
Due to the time constraints, I did not implement a proper user authentication model (or, unfortunatly, get a chance to explore crates that would have helped out here). 

To that end, this is now an 'anonymous chat client with an opt-in self-identification system.' What does that mean?

By default, all users are expected to be anonymous. If they wish, they can identify themselves with a chosen handle. All messages they send will be associated with that handle. All messages not associated with a handle will be associated with the 'first' anonymous user.

Can users pretend to be other users by assuming their handles? Absolutely! It's up to the user to verify the sender by some other means external to this system ;) 

### Questions:
n/a

### Class Notes:
n/a

### Reflections for Lukáš and self:

#### Async-Re-write
**This re-write took a lot longer anticipated.**

Understanding the flow of asynchronous programming was one hurdle. After attempting to refactor my application twice, I ended up opting for a full re-write (leveraging 
old code where it made sense to.)

The Server and Client both ended up in their own binary files. 

The Server loops over new client connections. For each new connection it:
- Creates a channel to communicate between the Tasks handling the client read/write connections
- Creates a channel for passing updated user ids when a client opts to register themselves
- Spawns a Task for reading from and a Task for witing to the client connection.

The Client spawns three Tasks:
- One to listening and handling input from stdin
- One for writing to the server's stream
- And one for handling incoming messages from the server's stream
Additionally, the client leverages a CancellationToken for gracefully shutting down each task if the user
decides it's time to `.quit` or if the server drops unexpectedly for some reason.

#### Database addition
The database addition is pretty simple. 

I leveraged sqlite with two tables, one for messages and another for users. They are linked to one another via the user_id.

The migrations seem to work for you, but not for me. To resolve this, I have the setup_db function manually creating the table (assuming the migration didn't work).
ouys 

## Description:

This assignment takes your client-server chat application to the next level by rewriting it to use the asynchronous paradigm with Tokio. Additionally, you'll start integrating a database to store chat and user data, marking a significant advancement in your application's complexity and functionality.

### Asynchronous Rewriting Using Tokio:

- [X] Refactor both the client and server components of your application to work asynchronously, using Tokio as the foundation.
    - Where we are:
        - Server 
            - DONE: 
                - Establishes a listener
                - Render messages coming in from client connection
                - Loop on listener and spawn thread for each client connecting
                - Receiving MessageTypes correctly
                - Broadcasts messages received back out to clients other than the original sender
            - NEXT: 
                - Resolve remaining FIXME(s)
        - Client 
            - DONE: 
                - Connects to server 
                - Starts three tasks for: handling stdin, handling strings from server, sending strings to server
                - Stdin and send are stubbed; they work based on Strings
                - Refactor String messages to be old MessageTypes
                - Handle receiving MessageTypes from the server
                - Process MessageTypes based on their type after rececving
                - Implement cancellation signal
                - Notify server when we know we are exiting
            - NEXT:
                - Resolve remaining FIXME(s)
- [X] Ensure all I/O operations, network communications, and other latency-sensitive tasks are handled using Tokio's asynchronous capabilities.

### Database Integration:

- [X] Choose a database framework like **sqlx**, diesel, or any other of your preference to integrate into the server for data persistence.

PICK UP HERE -- NEXT STEP IS TO LET A CLIENT REGISTER ITSELF WITH THE SERVER -- ADD A REGISTER COMMAND?

- [X] Design the database to store chat messages and user data effectively.

### User Identification:
- [X] Implement a mechanism for clients to identify themselves to the server. This can range from a simple identifier to a more secure authentication process, depending on your preference and the complexity you wish to introduce.
- [X] Ensure that the identification process is seamlessly integrated into the asynchronous workflow of the client-server communication.

### Security Considerations:
- [X] While focusing on the asynchronous model and database integration, keep in mind basic security practices for user identification and data storage.
- [X] Decide on the level of security you want to implement at this stage and ensure it is appropriately documented.

### Refactoring for Asynchronous and Database Functionality:

- [X] Thoroughly test all functionalities to ensure they work as expected in the new asynchronous setup.
- [X] Ensure the server's interactions with the database are efficient and error-handled correctly.

### Documentation and Comments:

- [X] Update your README.md to reflect the shift to asynchronous programming and the introduction of database functionality.
Document how to set up and run the modified application, especially any new requirements for the database setup.


### Async Notes
1. Futures leverage userspace threads
1. Async fns can be thought to run in zero to n 'chunks' where n is the number of .await(s)
1. .await(s) are called on a Future(s) within an async fn blocks. They yield execution of that function back up the stack, allowing the Executor to let other Futures progress. The original await() call will periodically check (bts) if the Future is complete. If at that time the future is complete, the async function block continues executing.
```rust
async {
    // let x = read_to_string("file").await;
    // await is essentially a loop that yields when it cannot continue for whatever reason 

    let fut = read_to_string("file");
    let x = loop {
        if let Some(result) = fut.try_check_complete() {
            break result;
        } else {
            fut.try_make_progress();
            yield;
        }
    }
}
```
1. Async is cooperative scheduling; if futures do not yield periodically things get fkd e.g.:
    - calling `std::fs::File` and `std::net::<stream>` (will simply block the thread)
   Equiv. async will yield allowing other async tasks to given cycles to progress.
1. If you must use something that will block such as heavy cpu action, leverage something like `tokio::task::[spawn_blocking|block_in_place]`. This way the scheduler expects this and will ensure other Futures can still progress.
1. Be careful with side-effects from Futures dropping when a select! exits early
1. where select! is good for branching control flow across Futures based on which is ready first, "joins" are tell you to wait for all futures (depth) to complete before continuing

### Tokio Notes
1. Runtime `tokio::runtime` (calling and working with Futures)
    - Futures often contain Futures e.g. an async fn which contains an async fn which calls an async stream
      - Tokio is only aware of the top level Future tasks, not their inner Futures
    - tokio::task::spawn will throw a Future into the queue to be scheduled and return a joinhandle which can be used or ignored
    - tokio::task::block_on will block until a specific Future is resolved
    - the Workers have a 'ready to work' and a 'not ready to work' queue, they only poll the 'ready to work queue'
      - poll'd tasks that return Pending get sent to 'not ready for work'
        - there is a `wake` method inside a `context` which is pasked down Resource chain. When this gets called by the I/O event loop, the scheduler knows it can be moved back to 'ready to work' 
    - the Scheduler uses a 'work stealing' algorithm so anticipate your Future(s) (tasks) being moved between threads
        - NOTE: The Send trait implies that a Future can be moved between threads. 
    - Blocking
        - AVOID HOLDING UP THREADS: Anything that can block the Worker's (OS level) thread is bad e.g. using std::io::stdin rather than tokio::io:stdin
        - `spawn_blocking` a thread or use `block_on` when you expect something to take > 100ms
            - and you have `block_in_place` when you need to something blocking but does not impliment the Send trait
    - LocalSet(s) are for sets of tasks (Futures) that must be run on the same thread (e.g. don't implement the Send Trait). But, LocalSet(s) can only be top level tasks in Tokio (or by spawning a new local thread at the OS level e.g. std::thread::spawn)
    - Tokio::mutex vs std::mutex:
        - tokio mutex lock methods are aync (can use .await) but is fairly inefficient
        - After you call .lock on a std::mutex DO NOT CALL AWAIT ON ANOTHER FUTURE WITHOUT or you can get into a race condition with the mutex being locked
    - tokio-tasks crate for visualizing the queues

1. Resources `tokio::io`(TcpStream, UdpStream, FS read/write, ...)
    - AsyncRead trait implies things can be read from in an async context via `poll_read`
    - Typically towards the bottom of a Future/task stack (like leafs of the tree)
    - tokio::fs `std::fs`
        - often just a wrapper on top of std::fs stuff to provide async features but is slow
        - dont be afraid to use `spawn_blocking` and then call std::fs if there is a big performance concern.
    - tokio::process `std::process`
        - when you `drop` handle to a child process, the process isn't killed. It continues executing
        - this is unlike a future, when you `drop` that it is not expected to continue
    - tokio::io `std::io`
        - AsyncReadExt and AsyncWriteExt (extensions) traits
            -  Proivide convience methods on top of things that impl AsyncWrite or AsyncRead e.g. `read_to_string`, `write_all`, ... 
                - Give Futures on top of AsyncRead or AsyncWrite so you can `await` them and stuff
        - AsyncBufReadExt and AsyncBufWriteExt traits
            - Similar, they also provide conveince methods e.g. `read_lines` 
        - If you need to share a resource e.g. a TcpStream, don't wrap it in a mutex. Use the "Actor Pattern" ## THIS WILL BE USEFUL IN HOMEWORK
            - Spawn a top level task which owns the TcpStream
            - It has a Channel which things can write to or read from 
```rust
#[tokio::main]
async fn main() {
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    let stream = tokio::net::TcpStream("127.0.0.1:8080").await.unwrap();
    tokio::spawn(async move {
        while let Some(bytes) = rx.next().await {
            stream.write_all(bytes).await.unwrap()
        }
    })

    # one way
    tx.send(vec![0, 1, 2, 3, 4])
    # other way
    {
        tx = tx.clone();
        tokio::spawn(async move {
            loop {
                tx.send(vec![0, 1, 2, 3, 4]).await;
            }
        });
    }
    {
        tx = tx.clone();
        tokio::spawn(async move {
            loop {
                tx.send(vec![0, 1, 2, 3, 4]).await;
            }
        });
    }

}
```

1. Utilities
    - tokio::sync 
        - help Futures communicate with one another
        - mpsc::channel is great similar to std but is more akin to a hammer
        - oneshot::channel -- like a channel but you can only send/rx once
        - broadcast::channel -- one tx, many rx but ensures everyone can get the rx
        - sync::watch -- like a chafnnel but only sees the most recent updated 
        - sync::notify -- great for doing something when something else changes (good for killing via .quit?)
    - select! (think of racing Futures and doing something when one finishes)
        - Common examples
            - "Wait for tcp packet or user pres ctr-c"
            - "Wait for input on tcp channel or std in"
            - "Wait for new emssage on this channel or write to complete"
            - "Wait for input on this channel or this Notify to complete to cancel early"
        - Cancellation concerns
            - When one Future in the select! finshes and it's arm is executed, all other Futures are dropped silently. Cancellation safety means the Future can be resumed 
            - To make things cancellation safe that are not inherintely, create the Future and `std::pin::pin` it OUTSIDE of the select! and then pass a mutable reference to the Future in the select
    -   
1. Common Complications
    - tokio::spawn
    - concurrency vs parallelism
    - mpsc fan-in
    - 