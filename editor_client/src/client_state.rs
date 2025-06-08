use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc; // Mutex için gerekli değilse de Arc için ekleyelim.
use std::sync::Mutex;


#[derive(Debug)]
pub struct ClientGuiState {
    pub available_documents: Vec<String>,
    pub active_users: Vec<String>,
    pub current_document_name: Option<String>,
    pub current_document_content: String,
    pub status_message: String,
    pub event_messages: Vec<String>, // Sunucudan gelen anlık olaylar için
    pub needs_redraw: AtomicBool,    // Ana döngüye yeniden çizim sinyali
}

impl ClientGuiState {
    pub fn new() -> Self {
        ClientGuiState {
            available_documents: Vec::new(),
            active_users: Vec::new(),
            current_document_name: None,
            current_document_content: String::new(),
            status_message: "Hazır.".to_string(),
            event_messages: Vec::new(),
            needs_redraw: AtomicBool::new(true), // Başlangıçta çizim yapsın
        }
    }

    pub fn add_event_message(&mut self, message: String) {
        self.event_messages.push(message);
        if self.event_messages.len() > 5 { // En fazla son 5 olayı tut
            self.event_messages.remove(0);
        }
        self.needs_redraw.store(true, Ordering::SeqCst);
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = message;
        self.needs_redraw.store(true, Ordering::SeqCst);
    }
}

// Paylaşılan durum için tip takma adı
pub type SharedClientGuiState = Arc<Mutex<ClientGuiState>>;