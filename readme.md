# Rust Collaborative TUI Text Editor

A real-time, multi-user text editor with a Terminal User Interface (TUI), built with Rust. This project demonstrates client-server architecture, TCP communication, and concurrent state management.

## Features

* **Real-Time Collaboration**: Multiple users can connect to a server and edit text documents simultaneously.
* **TUI Client**: A terminal-based user interface built with `ratatui` and `crossterm`.
* **Document Management**: Users can create, list, and switch between different text documents.
* **Centralized Server**: Manages document state and broadcasts changes to all connected clients.
* **Simple Protocol**: Uses a plain-text, line-based protocol for client-server communication.

## Architecture

The project is divided into three main crates:

1.  **`editor_server`**: The central server that handles client connections, manages document state, and synchronizes changes.
2.  **`editor_client`**: The TUI application that users run to connect to the server, view, and edit documents.
3.  **`editor_protocol`**: A shared library crate that defines the communication protocol constants and helper functions used by both the client and server.

### Communication Flow

1.  A client connects to the server and sends a `CONNECT` command with a username.
2.  The server acknowledges the connection and sends back a list of available documents.
3.  The client can request to `CREATE`, `LIST`, or `SWITCH` documents.
4.  When a user edits a document, the client sends an `UPDATE_DOCUMENT` command to the server with the new content.
5.  The server updates the document's state and broadcasts the `DOCUMENT_UPDATED` message to all other clients editing the same document.

## Getting Started

### Prerequisites

* Rust programming language and Cargo package manager. You can install them from [rust-lang.org](https://www.rust-lang.org/tools/install).

### Building and Running

1.  **Clone the repository**:
    ```bash
    git clone <repository-url>
    cd <repository-name>
    ```

2.  **Start the server**:
    In a terminal window, run:
    ```bash
    cargo run --bin editor_server
    ```
    The server will start and listen for connections on `0.0.0.0:12345`.

3.  **Run the client**:
    In a separate terminal window, run:
    ```bash
    cargo run --bin editor_client
    ```
    You will be prompted to enter a username. After that, the TUI will launch, and you can start interacting with the server.

### Usage

The client interface is divided into several panels:

* **Documents**: Lists all available documents on the server.
* **Content**: Displays the content of the currently selected document.
* **Users**: Shows a list of all currently connected users.
* **Command Input**: Where you type commands.
* **Events/Status**: A log of recent events and status messages from the server.

**Available Commands**:

* `CREATE <doc_name>`: Creates a new document.
* `LIST`: Refreshes the document list.
* `SWITCH <doc_name>`: Switches to view and edit a different document.
* `EDIT <content>`: Sends a line of text to be added to the current document.
* `QUIT`: Disconnects from the server and exits the client.

**Navigation**:

* Use `TAB` to switch between the **Command Input** and **Documents** panels.
* Use the `Up` and `Down` arrow keys to navigate the document list.
* Press `Enter` on a selected document to switch to it.
