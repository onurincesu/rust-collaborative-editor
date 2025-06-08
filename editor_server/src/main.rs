use editor_protocol::{PORT, SERVER_ADDRESS};
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::Write;

mod client_handler;
mod document_manager;

pub type ClientWriter = Arc<Mutex<Box<dyn Write + Send>>>;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", SERVER_ADDRESS, PORT))?;
    println!("Server started at {}:{}", SERVER_ADDRESS, PORT);

    let documents_arc = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    document_manager::load_all_documents(&mut documents_arc.lock().unwrap());

    let all_clients_writers_arc = Arc::new(Mutex::new(HashMap::<std::net::SocketAddr, ClientWriter>::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection accepted: {}", stream.peer_addr()?);
                let documents_clone = Arc::clone(&documents_arc);
                let all_clients_writers_clone = Arc::clone(&all_clients_writers_arc);
                thread::spawn(move || {
                    client_handler::handle_client(stream, documents_clone, all_clients_writers_clone);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}