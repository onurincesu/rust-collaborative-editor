use crate::app_state::{AppState, ActiveWindow};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw_ui(frame: &mut Frame, app_state: &mut AppState) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), 
            Constraint::Percentage(50), 
            Constraint::Percentage(25), 
        ])
        .split(frame.size());

    draw_documents_panel(frame, app_state, main_chunks[0]);

    let middle_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    
            Constraint::Length(3), 
            Constraint::Length(5), 
        ])
        .split(main_chunks[1]);

    draw_document_content_panel(frame, app_state, middle_chunks[0]);
    draw_command_input_panel(frame, app_state, middle_chunks[1]);
    draw_event_log_panel(frame, app_state, middle_chunks[2]);

    draw_users_panel(frame, app_state, main_chunks[2]);
}

fn draw_documents_panel(frame: &mut Frame, app_state: &mut AppState, area: Rect) {
    let items: Vec<ListItem> = app_state
        .available_documents
        .iter()
        .map(|doc_name| ListItem::new(Span::raw(doc_name.clone())))
        .collect();

    let border_style = if app_state.active_window == ActiveWindow::DocumentList {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Belgeler (TAB ile geçiş)")
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Gray)
                .fg(Color::Black),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut app_state.documents_list_state);
}

fn draw_users_panel(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let items: Vec<ListItem> = app_state
        .active_users
        .iter()
        .map(|user_name| ListItem::new(Span::raw(user_name.clone())))
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Aktif Kullanıcılar")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(list, area);
}

fn draw_document_content_panel(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let title = match &app_state.current_document_name {
        Some(name) => format!("İçerik: {} ", name),
        None => "İçerik (Belge Seçilmedi) ".to_string(),
    };
    let text: Vec<Line> = app_state
        .current_document_content
        .iter()
        .map(|line| Line::from(Span::raw(line.clone())))
        .collect();

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false }); 
    frame.render_widget(paragraph, area);
}

fn draw_command_input_panel(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let border_style = if app_state.active_window == ActiveWindow::CommandInput {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Komut giriş alanının iç kenarlarını hesaba katmak için x ve y koordinatlarını ayarla
    // ve imleç pozisyonunu buna göre düzelt.
    // Block kenarları için genellikle 1 karakter her yönden gider.
    let input_display_text = format!("> {}", app_state.command_input);

    let paragraph = Paragraph::new(Span::raw(input_display_text.clone())).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Komut (TAB ile geçiş)")
            .border_style(border_style),
    );
    frame.render_widget(paragraph, area);

    if app_state.active_window == ActiveWindow::CommandInput {
        // İmleç pozisyonu: Block'un sol kenarı (x) + "> " için 2 karakter + metin uzunluğu
        frame.set_cursor(
            area.x + 1 + 2 + app_state.command_input.chars().count() as u16, // +1 for left border
            area.y + 1, // +1 for top border
        );
    }
}

fn draw_event_log_panel(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let messages: Vec<Line> = app_state
        .event_log
        .iter()
        .map(|msg| Line::from(Span::raw(msg.clone())))
        .collect();

    let paragraph = Paragraph::new(messages)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Olaylar / Durum")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}