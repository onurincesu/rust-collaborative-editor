use crate::event::{AppEvent, ServerCommand};
use editor_protocol::*;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::mpsc::Sender;

pub fn start_server_listener_thread(
    stream_reader: TcpStream, // Klonlanmış ve sadece okuma için olan stream
    event_tx: Sender<AppEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stream_reader);
        loop {
            let mut line_from_server = String::new();
            match reader.read_line(&mut line_from_server) {
                Ok(0) => { // Bağlantı kapandı
                    let _ = event_tx.send(AppEvent::ServerMessage(ServerCommand::Error(
                        "Sunucu bağlantısı kesildi.".to_string(),
                    )));
                    break;
                }
                Ok(_) => {
                    let server_message = line_from_server.trim();
                    if server_message.is_empty() { continue; }

                    let parts: Vec<&str> = server_message.splitn(2, ' ').collect();
                    let command = parts[0];
                    let payload = if parts.len() > 1 { parts[1].to_string() } else { String::new() };

                    let app_event_payload = match command {
                        DOCUMENTS_LIST_MSG => {
                            let docs = if payload.is_empty() { Vec::new() } else { payload.split(',').map(String::from).collect() };
                            ServerCommand::UpdateDocumentList(docs)
                        }
                        DOCUMENT_CONTENT_MSG => {
                            let doc_name = payload.clone(); // payload'u sonra kullanacağız
                            let mut content_buffer = String::new();
                            loop {
                                let mut content_line = String::new();
                                match reader.read_line(&mut content_line) {
                                    Ok(0) => break, // Beklenmedik bağlantı kesilmesi
                                    Ok(_) => {
                                        if content_line.trim() == END_OF_MESSAGE_DELIMITER {
                                            break;
                                        }
                                        content_buffer.push_str(&content_line);
                                    }
                                    Err(e) => {
                                        let _ = event_tx.send(AppEvent::ServerMessage(ServerCommand::Error(format!("İçerik okunurken hata: {}", e))));
                                        return; // Thread'i sonlandır
                                    }
                                }
                            }
                            if content_buffer.ends_with('\n') { content_buffer.pop(); } // Son newline karakterini kaldır
                            ServerCommand::ReceiveDocumentContent { name: doc_name, content: content_buffer }
                        }
                        DOCUMENT_UPDATED_MSG => {
                            let doc_name = payload.clone();
                            let mut content_buffer = String::new();
                             loop {
                                let mut content_line = String::new();
                                match reader.read_line(&mut content_line) {
                                    Ok(0) => break,
                                    Ok(_) => {
                                        if content_line.trim() == END_OF_MESSAGE_DELIMITER {
                                            break;
                                        }
                                        content_buffer.push_str(&content_line);
                                    }
                                    Err(e) => {
                                         let _ = event_tx.send(AppEvent::ServerMessage(ServerCommand::Error(format!("Güncelleme okunurken hata: {}", e))));
                                        return;
                                    }
                                }
                            }
                            if content_buffer.ends_with('\n') { content_buffer.pop(); }
                            ServerCommand::UpdateDocumentContent{ name: doc_name, content: content_buffer }
                        }
                        USER_JOINED_MSG => ServerCommand::UserJoined(payload),
                        USER_LEFT_MSG => ServerCommand::UserLeft(payload),
                        NEW_DOCUMENT_AVAILABLE_MSG => ServerCommand::NewDocumentAvailable(payload),
                        DOCUMENT_SWITCHED_MSG => ServerCommand::SwitchedToDocument { name: payload },
                        CONNECTED_OK_MSG => ServerCommand::Status("Sunucuya başarıyla bağlanıldı!".to_string()),
                        DOCUMENT_CREATED_OK_MSG => ServerCommand::Status(format!("'{}' belgesi sunucuda oluşturuldu.", payload)),
                        DOCUMENT_CREATED_FAIL_MSG => ServerCommand::Error(format!("Belge oluşturma hatası: {}", payload)),
                        _ => ServerCommand::Status(format!("[SUNUCU BİLİNMEYEN]: {}", server_message)),
                    };

                    if event_tx.send(AppEvent::ServerMessage(app_event_payload)).is_err() {
                        // Ana thread muhtemelen kapandı, bu thread'i de sonlandır
                        break;
                    }
                }
                Err(e) => {
                    let _ = event_tx.send(AppEvent::ServerMessage(ServerCommand::Error(format!(
                        "Sunucudan okuma hatası: {}", e
                    ))));
                    break; // Hata durumunda döngüden çık
                }
            }
        }
    })
}