# template_cli_tokio

A template for building asynchronous CLI applications with Tokio runtime.

## License

This work is released under Unlicense or CC0 (waiving all rights, as much as legally possible in Japan). However, I personally request that you **delete the Git history before using this template**.


## Overview

This project serves as a starting template for developing interactive command-line applications that combine async REPL input with concurrent processing tasks (e.g., network communication).

## Features

- **Async REPL**: Asynchronous command-line input using `rustyline-async`
- **Concurrent Processing**: Efficient async task management with Tokio
- **Graceful Shutdown**: Proper cleanup mechanism for multiple concurrent tasks
- **Colored Output**: Terminal color support for better readability
- **Shell Word Parsing**: Command-line argument parsing capabilities

## Dependencies

- `tokio`: Async runtime
- `rustyline-async`: Async readline implementation
- `clap`: Command-line argument parser
- `shell-words`: Shell-style string parsing

## Project Structure

```
src/
├── main.rs    # Main logic and sample implementation (TCP communication)
└── cli.rs     # CLI framework implementation
```

### cli.rs Components

- `Cli`: CLI initialization and lifecycle management
- `Printer`: stdout/stderr helper with color support
- `CliEvent`: Command events (Input/Exit)
- `KillReceiver`: Task termination signal receiver

### main.rs Sample Implementation

Includes a sample TCP echo server communication example:
- Async CLI input handling
- TCP stream send loop
- TCP stream receive loop

## How to Run Example

### 1. Start Echo Server

```sh
ncat -l 5555 -k -c cat
```

### 2. Run Application

```sh
cargo run
```

### 3. Enter Commands

```sh
> foo
[Send] foo
[Recv] size: 4, foo

> q
```

## Usage

To use this template:

1. Reference the sample implementation in `main.rs` for your own logic
2. Initialize CLI with `Cli::new()` and obtain the event receiver
3. Implement command handling in the event loop by receiving `CliEvent`
4. Spawn concurrent tasks as needed using `tokio::spawn`
5. Call `cli.kill()` for graceful shutdown when exiting

## Exit Commands

The following commands will exit the application:

- `exit`
- `quit`
- `q`
- Ctrl+C
