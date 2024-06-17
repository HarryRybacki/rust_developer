# [Homework 8](https://robot-dreams-rust.mag.wiki/13-error-handling-custom-types/index.html#homework)

## Description:

This assignment takes your client-server chat application to the next level by rewriting it to use the asynchronous paradigm with Tokio. Additionally, you'll start integrating a database to store chat and user data, marking a significant advancement in your application's complexity and functionality.

### Asynchronous Rewriting Using Tokio:

- [ ] Refactor both the client and server components of your application to work asynchronously, using Tokio as the foundation.
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


### Class Notes:


### Reflections for Lukáš:
