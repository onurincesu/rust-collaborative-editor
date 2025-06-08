use ratatui::widgets::ListState;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct AppState {
    pub username: String,
    pub available_documents: Vec<String>,
    pub documents_list_state: ListState, // Belge listesindeki seçimi takip etmek için
    pub active_users: Vec<String>,
    // pub users_list_state: ListState, // Kullanıcı listesi için de gerekirse eklenebilir
    pub current_document_name: Option<String>,
    pub current_document_content: Vec<String>, // İçeriği satır satır tutalım
    pub command_input: String,                 // Kullanıcının girdiği komut
    pub event_log: Vec<String>, // Sunucu olayları ve durum mesajları için
    pub active_window: ActiveWindow, // Hangi pencerenin aktif olduğunu belirtir
    pub should_quit: bool,           // Uygulamadan çıkış yapılmalı mı?
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ActiveWindow {
    #[default]
    CommandInput,
    DocumentList,
    // ContentView, // Eğer içerik alanı da doğrudan düzenlenebilir olacaksa
}

impl AppState {
    pub fn new(username: String) -> Self {
        let mut app_state = AppState {
            username,
            event_log: vec!["Connecting...".to_string()],
            ..Default::default()
        };
        app_state.documents_list_state.select(None); // Başlangıçta hiçbir belge seçili değil
        app_state
    }

    pub fn add_event_log(&mut self, message: String) {
        self.event_log.push(message);
        if self.event_log.len() > 5 { // Son 5 olayı göster
            self.event_log.remove(0);
        }
    }

    pub fn select_next_document(&mut self) {
        if self.available_documents.is_empty() {
            self.documents_list_state.select(None);
            return;
        }
        let i = match self.documents_list_state.selected() {
            Some(i) => {
                if i >= self.available_documents.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.documents_list_state.select(Some(i));
    }

    pub fn select_previous_document(&mut self) {
        if self.available_documents.is_empty() {
            self.documents_list_state.select(None);
            return;
        }
        let i = match self.documents_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.available_documents.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.documents_list_state.select(Some(i));
    }
}

pub type SharedAppState = Arc<Mutex<AppState>>;