use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use editor_protocol::DOCUMENTS_DIR;

/// Loads all documents from the `DOCUMENTS_DIR` directory.
pub fn load_all_documents(docs_map: &mut HashMap<String, String>) {
    let doc_dir_path = Path::new(DOCUMENTS_DIR);
    if !doc_dir_path.exists() {
        if let Err(e) = fs::create_dir_all(doc_dir_path) {
            eprintln!("Could not create document directory {}: {}", DOCUMENTS_DIR, e);
            return;
        }
        println!("Document directory created: {}", DOCUMENTS_DIR);
    }

    match fs::read_dir(doc_dir_path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
                        if let Some(doc_name) = path.file_name().and_then(|name| name.to_str()) {
                            match fs::read_to_string(&path) {
                                Ok(content) => {
                                    docs_map.insert(doc_name.to_string(), content);
                                    println!("Loaded document: {}", doc_name);
                                }
                                Err(e) => {
                                    eprintln!("Error reading document {}: {}", doc_name, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading directory {}: {}", DOCUMENTS_DIR, e);
        }
    }
}

/// Saves a document to the disk.
pub fn save_document(doc_name: &str, content: &str) -> Result<(), std::io::Error> {
    let path_str = format!("{}{}", DOCUMENTS_DIR, doc_name);
    let path = Path::new(&path_str);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    println!("Document saved: {}", doc_name);
    Ok(())
}