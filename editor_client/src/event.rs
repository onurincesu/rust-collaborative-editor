use crossterm::event::KeyEvent;
// use std::sync::mpsc; // Bu satırı kaldırın veya yorum satırı yapın

// Uygulama içinde dolaşacak olay türleri
#[derive(Debug, Clone)]
pub enum AppEvent {
    Input(KeyEvent),              // Kullanıcıdan klavye girişi
    ServerMessage(ServerCommand), // Sunucudan gelen işlenmiş komut/mesaj
}

// Sunucudan gelen mesajların daha yapısal hali
#[derive(Debug, Clone)]
pub enum ServerCommand {
    UpdateDocumentList(Vec<String>),
    NewDocumentAvailable(String),
    UserJoined(String),
    UserLeft(String),
    ReceiveDocumentContent { name: String, content: String },
    UpdateDocumentContent { name: String, content: String },
    SwitchedToDocument { name: String }, 
    Status(String),
    Error(String),
}