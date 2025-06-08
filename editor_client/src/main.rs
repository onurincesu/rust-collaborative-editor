use editor_protocol::*;
use std::{
    error::Error,
    io::{self, Write},
    net::TcpStream,
    sync::mpsc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::{
    event::{self as CEvent, DisableMouseCapture, EnableMouseCapture, Event as TermEvent, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

mod app_state;
mod event;
mod server_listener;
mod ui;

use app_state::{AppState, ActiveWindow, SharedAppState};
use event::{AppEvent, ServerCommand};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Multi-User Text Editor Client (TUI)");
    print!("Enter your username: ");
    io::stdout().flush()?;
    let mut username_input = String::new();
    io::stdin().read_line(&mut username_input)?;
    let username = username_input.trim().to_string();

    if username.is_empty() {
        eprintln!("Username cannot be empty.");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_state_arc: SharedAppState = Arc::new(Mutex::new(AppState::new(username.clone())));
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>();

    let stream_to_server = match TcpStream::connect(format!("{}:{}", CLIENT_CONNECT_ADDRESS, PORT)) {
        Ok(stream) => stream,
        Err(e) => {
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
            eprintln!("Could not connect to server ({}:{}): {}", CLIENT_CONNECT_ADDRESS, PORT, e);
            return Ok(());
        }
    };
    let stream_writer_arc = Arc::new(Mutex::new(stream_to_server.try_clone()?));
    let stream_reader_clone = stream_to_server.try_clone()?;

    {
        let mut writer_guard = stream_writer_arc.lock().unwrap();
        writeln!(writer_guard, "{} {}", CONNECT_CMD, username)?;
        writer_guard.flush()?;
    }

    let server_listener_event_tx = event_tx.clone();
    server_listener::start_server_listener_thread(stream_reader_clone, server_listener_event_tx);

    let keyboard_event_tx = event_tx;
    thread::spawn(move || {
        let tick_rate = Duration::from_millis(200);
        loop {
            if CEvent::poll(tick_rate).unwrap_or(false) {
                if let Ok(TermEvent::Key(key_event)) = CEvent::read() {
                    if key_event.kind == KeyEventKind::Press {
                        if keyboard_event_tx.send(AppEvent::Input(key_event)).is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    loop {
        {
            let mut app = app_state_arc.lock().unwrap();
            terminal.draw(|f| ui::draw_ui(f, &mut app))?;
            if app.should_quit {
                break;
            }
        }

        if let Ok(app_event) = event_rx.recv() {
            let mut app = app_state_arc.lock().unwrap();
            handle_event(app_event, &mut app, &stream_writer_arc);
        } else {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_event(event: AppEvent, app: &mut AppState, stream_writer: &Arc<Mutex<TcpStream>>) {
    match event {
        AppEvent::Input(key_event) => handle_key_event(key_event, app, stream_writer),
        AppEvent::ServerMessage(server_cmd) => handle_server_command(server_cmd, app),
    }
}

fn handle_key_event(key_event: crossterm::event::KeyEvent, app: &mut AppState, stream_writer: &Arc<Mutex<TcpStream>>) {
    match app.active_window {
        ActiveWindow::CommandInput => {
            match key_event.code {
                KeyCode::Enter => {
                    let command_full = app.command_input.trim().to_string();
                    app.command_input.clear();
                    app.add_event_log(format!("Sending: {}", command_full));

                    let parts: Vec<&str> = command_full.splitn(2, ' ').collect();
                    let cmd_verb = parts[0].to_uppercase();
                    let cmd_arg = if parts.len() > 1 { parts[1] } else { "" };

                    let mut writer_guard = stream_writer.lock().unwrap();
                    match cmd_verb.as_str() {
                        "QUIT" => {
                            let _ = writeln!(writer_guard, "{}", DISCONNECT_CMD);
                            app.should_quit = true;
                        },
                        "LIST" => { let _ = writeln!(writer_guard, "{}", LIST_DOCUMENTS_CMD); },
                        "CREATE" if !cmd_arg.is_empty() => { let _ = writeln!(writer_guard, "{} {}", CREATE_DOCUMENT_CMD, cmd_arg); },
                        "SWITCH" if !cmd_arg.is_empty() => { let _ = writeln!(writer_guard, "{} {}", SWITCH_DOCUMENT_CMD, cmd_arg); },
                        "EDIT" => {
                            if let Some(ref doc_name) = app.current_document_name {
                                let msg = format_document_message(UPDATE_DOCUMENT_CMD, doc_name, cmd_arg);
                                let _ = writeln!(writer_guard, "{}", msg);
                            } else {
                                app.add_event_log("ERROR: No active document to edit.".to_string());
                            }
                        },
                        _ => app.add_event_log(format!("Unknown command or missing argument: {}", command_full)),
                    }
                    let _ = writer_guard.flush();
                },
                KeyCode::Char(c) => app.command_input.push(c),
                KeyCode::Backspace => { app.command_input.pop(); },
                KeyCode::Tab => app.active_window = ActiveWindow::DocumentList,
                _ => {},
            }
        },
        ActiveWindow::DocumentList => {
            match key_event.code {
                KeyCode::Enter => {
                    if let Some(selected_index) = app.documents_list_state.selected() {
                        if let Some(doc_name) = app.available_documents.get(selected_index).cloned() {
                            let mut writer_guard = stream_writer.lock().unwrap();
                            let _ = writeln!(writer_guard, "{} {}", SWITCH_DOCUMENT_CMD, &doc_name);
                            let _ = writer_guard.flush();
                            app.add_event_log(format!("Requesting to switch to '{}'.", doc_name));
                        }
                    }
                    app.active_window = ActiveWindow::CommandInput;
                },
                KeyCode::Up => app.select_previous_document(),
                KeyCode::Down => app.select_next_document(),
                KeyCode::Tab => app.active_window = ActiveWindow::CommandInput,
                _ => {},
            }
        },
    }
}

fn handle_server_command(server_cmd: ServerCommand, app: &mut AppState) {
    match server_cmd {
        ServerCommand::UpdateDocumentList(docs) => {
            app.available_documents = docs;
            if app.documents_list_state.selected().is_none() && !app.available_documents.is_empty() {
                app.documents_list_state.select(Some(0));
            }
            app.add_event_log("Document list updated.".to_string());
        },
        ServerCommand::NewDocumentAvailable(doc_name) => {
            if !app.available_documents.contains(&doc_name) {
                app.available_documents.push(doc_name.clone());
            }
            app.add_event_log(format!("New document available: {}", doc_name));
        },
        ServerCommand::UserJoined(username) => {
            if !app.active_users.contains(&username) {
                app.active_users.push(username.clone());
            }
            app.add_event_log(format!("{} joined.", username));
        },
        ServerCommand::UserLeft(username) => {
            app.active_users.retain(|u| u != &username);
            app.add_event_log(format!("{} left.", username));
        },
        ServerCommand::ReceiveDocumentContent { name, content } => {
            app.current_document_name = Some(name.clone());
            app.current_document_content = content.lines().map(String::from).collect();
            app.add_event_log(format!("Loaded document '{}'.", name));
        },
        ServerCommand::UpdateDocumentContent { name, content } => {
            if app.current_document_name.as_ref() == Some(&name) {
                app.current_document_content = content.lines().map(String::from).collect();
                app.add_event_log(format!("Active document '{}' updated.", name));
            } else {
                app.add_event_log(format!("Inactive document '{}' was updated.", name));
            }
        },
        ServerCommand::SwitchedToDocument { name } => {
            app.current_document_name = Some(name.clone());
            app.current_document_content.clear();
            app.add_event_log(format!("Switched to document '{}'.", name));
        },
        ServerCommand::Status(msg) => app.add_event_log(format!("[SERVER] {}", msg)),
        ServerCommand::Error(err_msg) => {
            app.add_event_log(format!("[ERROR] {}", err_msg));
            if err_msg.contains("Server connection closed") {
                app.should_quit = true;
            }
        },
    }
}