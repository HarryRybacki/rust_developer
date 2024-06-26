# [Homework 7](https://robot-dreams-rust.mag.wiki/13-error-handling-custom-types/index.html#homework)
In this assignment, you will be enhancing the robustness of your client-server chat application by introducing comprehensive error handling. By leveraging the anyhow and thiserror crates, you'll simplify the process and ensure more accurate, user-friendly error reporting.

## [Description:](https://robot-dreams-rust.mag.wiki/13-error-handling-custom-types/index.html#description)

### Integrate Anyhow and Thiserror:

- [X] Introduce the anyhow crate to manage errors in a straightforward, flexible way. This crate is especially useful for handling errors that don't need much context or are unexpected.
- [X] Utilize the thiserror crate to create custom, meaningful error types for your application. This is particularly beneficial for errors where you need more context and structured data.

### Error Handling in the Server:

- [X] Ensure that your server accurately reports errors to the client in a strongly-typed manner. Any operation that can fail should communicate its failure reason clearly and specifically.

### Client-Side Error Management:

- [X] Modify the client to handle and display error messages received from the server appropriately. Ensure that these messages are user-friendly and informative.

### Refactoring for Error Handling:

- [X] Review your existing codebase for both the client and server. Identify areas where error handling can be improved and implement changes using anyhow and thiserror.
- [X] Pay special attention to operations that involve network communication, file handling, and data parsing, as these are common sources of errors.

### Documentation and Testing:

- [X] Test various failure scenarios to ensure that errors are handled gracefully and the error messages are clear and helpful.

### Questions:
n/a

### Class Notes:
n/a

### Reflections for Lukáš:
I wish that I had more time this week to play around with this. Refactoring the crunchy error handling in the common lib went a long way to making the code more readable but I still think there is a lot of room for improvement there. 
