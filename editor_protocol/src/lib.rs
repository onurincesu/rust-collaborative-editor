pub const PORT: u16 = 12345;
pub const SERVER_ADDRESS: &str = "0.0.0.0"; // Listen on all available network interfaces
pub const CLIENT_CONNECT_ADDRESS: &str = "127.0.0.1"; // Client will connect to localhost
pub const DOCUMENTS_DIR: &str = "shared_documents/";

// Commands from Client to Server
pub const CONNECT_CMD: &str = "CONNECT";
pub const DISCONNECT_CMD: &str = "DISCONNECT";
pub const GET_DOCUMENT_CMD: &str = "GET_DOCUMENT";
pub const UPDATE_DOCUMENT_CMD: &str = "UPDATE_DOCUMENT";
pub const LIST_DOCUMENTS_CMD: &str = "LIST_DOCUMENTS";
pub const CREATE_DOCUMENT_CMD: &str = "CREATE_DOCUMENT";
pub const SWITCH_DOCUMENT_CMD: &str = "SWITCH_DOCUMENT";

// Messages from Server to Client
pub const CONNECTED_OK_MSG: &str = "CONNECTED_OK";
pub const USER_JOINED_MSG: &str = "USER_JOINED";
pub const USER_LEFT_MSG: &str = "USER_LEFT";
pub const DOCUMENT_CONTENT_MSG: &str = "DOCUMENT_CONTENT";
pub const DOCUMENT_UPDATED_MSG: &str = "DOCUMENT_UPDATED";
pub const DOCUMENTS_LIST_MSG: &str = "DOCUMENTS_LIST";
pub const DOCUMENT_CREATED_OK_MSG: &str = "DOCUMENT_CREATED_OK";
pub const DOCUMENT_CREATED_FAIL_MSG: &str = "DOCUMENT_CREATED_FAIL";
pub const DOCUMENT_SWITCHED_MSG: &str = "DOCUMENT_SWITCHED";
pub const NEW_DOCUMENT_AVAILABLE_MSG: &str = "NEW_DOCUMENT_AVAILABLE";

pub const END_OF_MESSAGE_DELIMITER: &str = "END_OF_MESSAGE";

/// Formats a message with a document name and content.
pub fn format_document_message(command: &str, doc_name: &str, content: &str) -> String {
    format!("{} {}\n{}\n{}", command, doc_name, content, END_OF_MESSAGE_DELIMITER)
}

/// Formats a command with a single argument.
pub fn format_command_with_arg(command: &str, argument: &str) -> String {
    format!("{} {}", command, argument)
}

/// Formats a simple command without arguments.
pub fn format_simple_command(command: &str) -> String {
    command.to_string()
}