# [Homework 8](https://robot-dreams-rust.mag.wiki/13-error-handling-custom-types/index.html#homework)

## Description:

This assignment takes your client-server chat application to the next level by rewriting it to use the asynchronous paradigm with Tokio. Additionally, you'll start integrating a database to store chat and user data, marking a significant advancement in your application's complexity and functionality.

### Asynchronous Rewriting Using Tokio:

- [ ] Refactor both the client and server components of your application to work asynchronously, using Tokio as the foundation.
    - Where we are:
        - Server 
            - DONE: 
                - Establishes a listener
            - NEXT: 
                - Render messages coming in from client connection
                - Loop on listener and spawn thread for each client connecting
        - Client 
            - DONE:
                - Connects to server and loops on printing output from the server as well as reacting to its stdin
            - NEXT: 
                - Upon receving input from stdin, it should send it to the Server
- [ ] Ensure all I/O operations, network communications, and other latency-sensitive tasks are handled using Tokio's asynchronous capabilities.

### Database Integration:

- [ ] Choose a database framework like sqlx, diesel, or any other of your preference to integrate into the server for data persistence.
- [ ] Design the database to store chat messages and user data effectively.

### User Identification:
- [ ] Implement a mechanism for clients to identify themselves to the server. This can range from a simple identifier to a more secure authentication process, depending on your preference and the complexity you wish to introduce.
- [ ] Ensure that the identification process is seamlessly integrated into the asynchronous workflow of the client-server communication.

### Security Considerations:
- [ ] While focusing on the asynchronous model and database integration, keep in mind basic security practices for user identification and data storage.
- [ ] Decide on the level of security you want to implement at this stage and ensure it is appropriately documented.

### Refactoring for Asynchronous and Database Functionality:

- [ ] Thoroughly test all functionalities to ensure they work as expected in the new asynchronous setup.
- [ ] Ensure the server's interactions with the database are efficient and error-handled correctly.

### Documentation and Comments:

- [ ] Update your README.md to reflect the shift to asynchronous programming and the introduction of database functionality.
Document how to set up and run the modified application, especially any new requirements for the database setup.

### Questions:
n/a

### Class Notes:
n/a

### Reflections for Lukáš and self:

#### Async
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
1. Async is cooperative scheduling; if futures do not yield periodically things get fk'd e.g.:
    - calling `std::fs::File` and `std::net::<stream>` (will simply block the thread)
   Equiv. async will yield allowing other async tasks to given cycles to progress.
1. If you must use something that will block such as heavy cpu action, leverage something like `tokio::task::[spawn_blocking|block_in_place]`. This way the scheduler expects this and will ensure other Futures can still progress.
1. Be careful with side-effects from Futures dropping when a select! exits early

#### Tokio
1. 