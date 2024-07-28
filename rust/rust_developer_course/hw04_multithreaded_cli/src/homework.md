# Homework 4

Expanding on the previous homework, we are going to complicate things once again by making the application interactive.

This assignment will transform your previous application into a multithreaded one

## Description:
You'll be tasked with implementing multi-threading in your Rust application. This will enhance the efficiency of your program by dividing tasks among separate threads.

### 1. Set up Concurrency:

  [X] Spin up two threads: one dedicated to receiving input and another for processing it.
  [X] Make use of channels to transfer data between the two threads. You can employ Rust's native std::mpsc::channel or explore the flume library for this.

### 2. Input-Receiving Thread:

  [X] This thread should continuously read from stdin and parse the received input in the format <command> <input>. Remember to avoid "stringly-typed APIs" - the command can be an enum. For an enum, you can implement the FromStr trait for idiomatic parsing.

### 3. Processing Thread:

  [X] Analyze the command received from the input thread and execute the appropriate operation.
  [X] If successful, print the output to stdout. If there's an error, print it to stderr.

### 4. CSV File Reading:

  [X] Instead of reading CSV from stdin, now adapt your application to read from a file using the read_to_string() function. Make sure you handle any potential errors gracefully.

### 5. Bonus Challenge - Oneshot Functionality:

  [X] If you're looking for an additional challenge, implement a mechanism where the program enters the interactive mode only when there are no CLI arguments provided. If arguments are given, the program should operate in the previous way, processing the command directly.

### Questions:

1. Q: Why do we clone our sender/tx half of the channel? 
   A: Because std channels support multiple senders but only one receiver.
2. Q: How do you ensure that after receiving input, the processing thread can handle 
      its transmutation and display the resulting string BEFORE the input thread 
      prompts the user for the next <command> <input>?