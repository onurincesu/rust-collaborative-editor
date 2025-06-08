use editor_protocol::*;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use crate::document_manager;

// Sunucunun paylaşılan durumunu (belgeler ve aktif istemciler) temsil eder.
// Bu yapı, main.rs içinde tanımlanıp Arc<Mutex<>> ile sarmalanacak.
pub struct ServerSharedState {
    pub documents: HashMap<String, String>, // belge_adı -> içerik
    pub clients: Vec<ClientInfo>, // Aktif istemcilerin bilgileri
}

// Her bir bağlı istemcinin bilgisini tutar.
#[derive(Clone)]
pub struct ClientInfo {
    pub username: String,
    // İstemciye mesaj göndermek için TcpStream'in yazma kısmını klonlamak yerine,
    // her istemciye özgü bir ID veya stream'i doğrudan tutabiliriz.
    // Ancak yayın yaparken tüm client'ların stream'lerine erişmemiz gerekecek.
    // Şimdilik, her client handler kendi stream'ini yönetecek ve yayınlar ana döngüden yapılacak.
    // Bu yüzden burada stream'i tutmayalım, bunun yerine ClientHandler'a stream'i doğrudan verelim.
    pub current_document_name: Option<String>,
    // İstemciye mesaj göndermek için bir kanal (mpsc) veya stream'in bir kopyası (Arc<Mutex<TcpStream>>) kullanılabilir.
    // Basitlik adına, broadcast işlemleri için tüm client stream'lerinin bir listesi ana sunucu yapısında tutulacak.
    // Bu struct şimdilik sadece username ve aktif belgeyi tutsun.
}


// İstemci için paylaşılan mesaj gönderme yeteneği
type ClientWriter = Arc<Mutex<Box<dyn Write + Send>>>;

// Her bir istemci bağlantısını yönetir.
pub fn handle_client(
    stream: TcpStream,
    server_documents_arc: Arc<Mutex<HashMap<String, String>>>,
    // Tüm istemcilere yayın yapmak için bir istemci listesi (yazıcıları ile birlikte)
    // Her client için bir ID ve ona ait bir writer tutulabilir.
    // Veya daha basitçe, her client_handler kendi stream'inin bir klonunu (yazma kısmı) tutar
    // ve global bir client listesi de bu yazıcıları tutar.
    // Şimdilik, yayınlar için tüm client_handler'ların writer'larına bir şekilde erişilmesi gerekecek.
    // client_writers: Arc<Mutex<Vec<ClientWriter>>>,
    // Her client için ayrı bir ID ve ona ait bir writer tutmak daha yönetilebilir olabilir.
    // (peer_addr -> writer) şeklinde bir HashMap kullanılabilir.
    all_clients_writers_arc: Arc<Mutex<HashMap<std::net::SocketAddr, ClientWriter>>>
) {
    let peer_addr = stream.peer_addr().expect("Bağlı istemcinin adresi alınamadı.");
    println!("Yeni istemci bağlandı: {}", peer_addr);

    let reader_stream = stream.try_clone().expect("Stream klonlanamadı (okuma).");
    let writer_stream = stream; // Orijinal stream yazma için kullanılır.

    let mut reader = BufReader::new(reader_stream);
    let mut writer: ClientWriter = Arc::new(Mutex::new(Box::new(writer_stream)));
    
    // Bu istemcinin yazıcısını global listeye ekle
    all_clients_writers_arc.lock().unwrap().insert(peer_addr, writer.clone());


    let mut current_username: Option<String> = None;
    let mut current_document_name_for_client: Option<String> = None;

    loop {
        let mut command_line = String::new();
        match reader.read_line(&mut command_line) {
            Ok(0) => { // Bağlantı kapandı
                println!("İstemci {} bağlantıyı kesti (EOF).", peer_addr);
                break;
            }
            Ok(_) => {
                let command_line = command_line.trim();
                if command_line.is_empty() { continue; }

                let parts: Vec<&str> = command_line.splitn(2, ' ').collect();
                let command = parts[0];
                let argument = if parts.len() > 1 { parts[1] } else { "" };

                println!("{}'dan alındı: {}", current_username.as_deref().unwrap_or("Bilinmeyen"), command_line);

                match command {
                    CONNECT_CMD => {
                        if !argument.is_empty() {
                            current_username = Some(argument.to_string());
                            send_message(&writer, CONNECTED_OK_MSG);
                            println!("Kullanıcı {} bağlandı.", argument);
                            // Kullanıcıya mevcut belge listesini gönder
                            send_available_documents(&writer, &server_documents_arc.lock().unwrap());
                            // Diğerlerine haber ver
                            broadcast_message_to_others(
                                &all_clients_writers_arc.lock().unwrap(),
                                peer_addr,
                                &format_command_with_arg(USER_JOINED_MSG, argument)
                            );
                        } else {
                            send_message(&writer, "HATA Kullanıcı adı sağlanmadı");
                        }
                    }
                    LIST_DOCUMENTS_CMD => {
                        send_available_documents(&writer, &server_documents_arc.lock().unwrap());
                    }
                    CREATE_DOCUMENT_CMD => {
                        if !argument.is_empty() {
                            let mut doc_name = argument.to_string();
                            if !doc_name.ends_with(".txt") {
                                doc_name.push_str(".txt");
                            }
                            let mut docs = server_documents_arc.lock().unwrap();
                            if !docs.contains_key(&doc_name) {
                                docs.insert(doc_name.clone(), String::new());
                                if document_manager::save_document(&doc_name, "").is_ok() {
                                    send_message(&writer, &format_command_with_arg(DOCUMENT_CREATED_OK_MSG, &doc_name));
                                    // Diğer istemcilere bildir
                                     broadcast_message_to_all(
                                        &all_clients_writers_arc.lock().unwrap(),
                                        &format_command_with_arg(NEW_DOCUMENT_AVAILABLE_MSG, &doc_name)
                                    );
                                } else {
                                    send_message(&writer, &format!("{} Belge diske kaydedilemedi.", DOCUMENT_CREATED_FAIL_MSG));
                                    docs.remove(&doc_name); // Başarısız olursa geri al
                                }
                            } else {
                                send_message(&writer, &format!("{} Belge zaten var.", DOCUMENT_CREATED_FAIL_MSG));
                            }
                        } else {
                            send_message(&writer, "HATA Belge adı sağlanmadı");
                        }
                    }
                    SWITCH_DOCUMENT_CMD => {
                        if !argument.is_empty() {
                            let doc_name_to_switch = argument.to_string();
                            let docs = server_documents_arc.lock().unwrap();
                            if let Some(content) = docs.get(&doc_name_to_switch) {
                                current_document_name_for_client = Some(doc_name_to_switch.clone());
                                send_message(&writer, &format_command_with_arg(DOCUMENT_SWITCHED_MSG, &doc_name_to_switch));
                                send_full_document_content(&writer, &doc_name_to_switch, content);
                                if let Some(ref uname) = current_username {
                                     broadcast_message_to_others( // Belki farklı bir mesaj ("USER_SWITCHED_DOC")
                                        &all_clients_writers_arc.lock().unwrap(),
                                        peer_addr,
                                        &format!("{} {} {} belgesine geçti.", USER_JOINED_MSG, uname, doc_name_to_switch)
                                    );
                                }
                            } else {
                                send_message(&writer, &format!("HATA Belge bulunamadı: {}", doc_name_to_switch));
                            }
                        } else {
                            send_message(&writer, "HATA Geçiş yapılacak belge adı sağlanmadı");
                        }
                    }
                     GET_DOCUMENT_CMD => { // SWITCH_DOCUMENT ile benzer, ama belki sadece içeriği gönderir.
                        if !argument.is_empty() {
                            let doc_name_to_get = argument.to_string();
                            let docs = server_documents_arc.lock().unwrap();
                            if let Some(content) = docs.get(&doc_name_to_get) {
                                // İstemcinin aktif belgesini değiştirmeden sadece içeriği gönder.
                                // Veya SWITCH gibi davranabilir. Java kodunda GET_DOCUMENT sonrası currentDocumentName ayarlanıyor.
                                current_document_name_for_client = Some(doc_name_to_get.clone());
                                send_full_document_content(&writer, &doc_name_to_get, content);
                            } else {
                                send_message(&writer, &format!("HATA Belge bulunamadı: {}", doc_name_to_get));
                            }
                        } else {
                             send_message(&writer, "HATA Belge adı GET_DOCUMENT için sağlanmadı");
                        }
                    }
                    UPDATE_DOCUMENT_CMD => {
                        if !argument.is_empty() {
                            let doc_to_update = argument.to_string();
                             // İstemcinin aktif olarak düzenlediği belgeyi güncellemesine izin ver
                            if Some(doc_to_update.clone()) == current_document_name_for_client {
                                let mut new_content_buffer = String::new();
                                loop {
                                    let mut line = String::new();
                                    match reader.read_line(&mut line) {
                                        Ok(0) => break, // Bağlantı koptu
                                        Ok(_) => {
                                            if line.trim() == END_OF_MESSAGE_DELIMITER {
                                                break;
                                            }
                                            new_content_buffer.push_str(&line);
                                        }
                                        Err(_) => break, // Hata
                                    }
                                }
                                // Son \n'i kaldır (Java'daki gibi)
                                if new_content_buffer.ends_with('\n') {
                                    new_content_buffer.pop();
                                }

                                let mut docs = server_documents_arc.lock().unwrap();
                                if docs.contains_key(&doc_to_update) {
                                    docs.insert(doc_to_update.clone(), new_content_buffer.clone());
                                    if document_manager::save_document(&doc_to_update, &new_content_buffer).is_ok() {
                                        // Diğer istemcilere (aynı belgeyi düzenleyenlere) bildir
                                        let update_msg = format_document_message(DOCUMENT_UPDATED_MSG, &doc_to_update, &new_content_buffer);
                                        let all_writers = all_clients_writers_arc.lock().unwrap();
                                        for (client_addr, client_writer_arc) in all_writers.iter() {
                                            // TODO: Sadece aynı belgeyi düzenleyenlere göndermek için
                                            // her istemcinin aktif belgesini de bilmemiz gerek.
                                            // Şimdilik herkese (güncelleyen hariç) gönderelim, ama bu ideal değil.
                                            // Daha iyisi: `current_document_name_for_client` bilgisini global client listesinde tutmak.
                                            if *client_addr != peer_addr { // Kendisine gönderme
                                                 // İdealde: if client_is_editing(doc_to_update) ...
                                                send_message(client_writer_arc, &update_msg);
                                            }
                                        }
                                        // Geri bildirim (Java'da yoktu ama faydalı olabilir)
                                        // send_message(&writer, "UPDATE_ACKNOWLEDGED");
                                    } else {
                                        send_message(&writer, "HATA Sunucuda belge kaydedilemedi.");
                                    }
                                } else {
                                     send_message(&writer, "HATA Güncellenecek belge sunucuda bulunamadı.");
                                }
                            } else {
                                send_message(&writer, &format!("HATA: {} belgesini düzenleme yetkiniz yok. Önce geçiş yapın.", doc_to_update));
                            }
                        } else {
                            send_message(&writer, "HATA Belge adı UPDATE_DOCUMENT için sağlanmadı");
                        }
                    }
                    DISCONNECT_CMD => {
                        println!("İstemci {} bağlantıyı sonlandırma isteği gönderdi.", peer_addr);
                        break;
                    }
                    _ => {
                        send_message(&writer, &format!("HATA Bilinmeyen komut: {}", command));
                    }
                }
            }
            Err(e) => {
                eprintln!("İstemci {}'dan okuma hatası: {}", peer_addr, e);
                break;
            }
        }
    }

    // Temizlik
    all_clients_writers_arc.lock().unwrap().remove(&peer_addr);
    if let Some(username) = current_username {
        println!("Kullanıcı {} ({}) bağlantısı kesildi.", username, peer_addr);
        broadcast_message_to_all(
            &all_clients_writers_arc.lock().unwrap(),
            &format_command_with_arg(USER_LEFT_MSG, &username)
        );
    } else {
        println!("İstemci {} bağlantısı kesildi (kullanıcı adı yok).", peer_addr);
    }
}

fn send_message(writer_arc: &ClientWriter, message: &str) {
    let mut writer_guard = writer_arc.lock().unwrap();
    if let Err(e) = writeln!(writer_guard, "{}", message) {
        // eprintln!("Mesaj gönderilemedi: {}", e); // Çok fazla log olabilir
    }
    if let Err(e) = writer_guard.flush() {
        // eprintln!("Flush hatası: {}", e);
    }
}

fn send_available_documents(writer: &ClientWriter, docs: &HashMap<String, String>) {
    let doc_names: Vec<String> = docs.keys().cloned().collect();
    let doc_list_string = doc_names.join(",");
    send_message(writer, &format_command_with_arg(DOCUMENTS_LIST_MSG, &doc_list_string));
}

fn send_full_document_content(writer: &ClientWriter, doc_name: &str, content: &str) {
    send_message(writer, &format_document_message(DOCUMENT_CONTENT_MSG, doc_name, content));
}

// Belirli bir istemci hariç diğer tüm istemcilere mesaj yayınlar.
fn broadcast_message_to_others(
    client_writers: &HashMap<std::net::SocketAddr, ClientWriter>,
    exclude_addr: std::net::SocketAddr,
    message: &str
) {
    for (addr, writer_arc) in client_writers.iter() {
        if *addr != exclude_addr {
            send_message(writer_arc, message);
        }
    }
}
// Tüm istemcilere mesaj yayınlar.
fn broadcast_message_to_all(
    client_writers: &HashMap<std::net::SocketAddr, ClientWriter>,
    message: &str
) {
    for writer_arc in client_writers.values() {
        send_message(writer_arc, message);
    }
}