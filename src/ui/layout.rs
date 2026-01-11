//! Main layout and rendering
//!
//! Orchestrates the overall TUI layout and delegates to widgets.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, InputMode, View};

use super::{help, input, preview, sessions, templates, theme::Theme, windows};

/// Main draw function
pub fn draw(frame: &mut Frame, app: &App) {
    let theme = Theme::dark();
    let size = frame.size();

    // Main layout: header, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(5),    // Content
            Constraint::Length(2), // Footer
        ])
        .split(size);

    // Draw header
    draw_header(frame, app, &theme, chunks[0]);

    // Draw main content based on view
    match app.view {
        View::Sessions => {
            draw_sessions_view(frame, app, &theme, chunks[1]);
        }
        View::Windows => {
            draw_windows_view(frame, app, &theme, chunks[1]);
        }
        View::Templates => {
            draw_templates_view(frame, app, &theme, chunks[1]);
        }
        View::Help => {
            // Help is drawn as overlay
        }
    }

    // Draw footer
    draw_footer(frame, app, &theme, chunks[2]);

    // Draw overlays
    if app.show_help {
        help::draw(frame, app, &theme);
    }

    // Draw input dialogs
    match &app.input_mode {
        InputMode::Input { prompt, .. } => {
            input::draw_input_dialog(frame, app, &theme, prompt);
        }
        InputMode::Confirm { message, .. } => {
            input::draw_confirm_dialog(frame, app, &theme, message);
        }
        InputMode::Search => {
            input::draw_search_bar(frame, app, &theme);
        }
        InputMode::Normal => {}
    }

    // Draw status/error messages
    if let Some(ref msg) = app.error_message {
        draw_message(frame, &theme, msg, true);
    } else if let Some(ref msg) = app.status_message {
        draw_message(frame, &theme, msg, false);
    }
}

/// Draw header
fn draw_header(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let title = match app.view {
        View::Sessions => {
            let local_count = app.sessions.iter().filter(|s| s.host.is_none()).count();
            let remote_count = app.sessions.iter().filter(|s| s.host.is_some()).count();
            let count_str = if remote_count > 0 {
                format!("{} local, {} remote", local_count, remote_count)
            } else {
                format!("{}", local_count)
            };
            format!(" sesh - Sessions ({}) ", count_str)
        }
        View::Windows => {
            if let Some(ref session) = app.selected_session {
                format!(" sesh - {} ({} windows) ", session, app.windows.len())
            } else {
                " sesh - Windows ".to_string()
            }
        }
        View::Templates => {
            format!(" sesh - Templates ({}) ", app.templates.len())
        }
        View::Help => " sesh - Help ".to_string(),
    };

    let help_hint = "[?] Help  [q] Quit";

    let title_len = title.len();
    let help_len = help_hint.len();
    let padding = area.width as usize - title_len - help_len;

    let header = Line::from(vec![
        Span::styled(title, theme.title()),
        Span::raw(" ".repeat(padding.max(1))),
        Span::styled(help_hint, theme.muted()),
    ]);

    frame.render_widget(Paragraph::new(header), area);
}

/// Draw sessions view (list + preview)
fn draw_sessions_view(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    if app.show_preview {
        // Split into list and preview
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        sessions::draw(frame, app, theme, chunks[0]);
        preview::draw(frame, app, theme, chunks[1]);
    } else {
        sessions::draw(frame, app, theme, area);
    }
}

/// Draw windows view
fn draw_windows_view(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    if app.show_preview {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        windows::draw(frame, app, theme, chunks[0]);
        preview::draw(frame, app, theme, chunks[1]);
    } else {
        windows::draw(frame, app, theme, area);
    }
}

/// Draw templates view
fn draw_templates_view(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    templates::draw(frame, app, theme, area);
}

/// Draw footer with key hints
fn draw_footer(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let keys = match app.view {
        View::Sessions => vec![
            ("[Enter]", "Attach"),
            ("[n]", "New"),
            ("[d]", "Detach"),
            ("[x]", "Kill"),
            ("[w]", "Windows"),
            ("[t]", "Templates"),
            ("[/]", "Search"),
            ("[r]", "Refresh"),
        ],
        View::Windows => vec![
            ("[Enter]", "Select"),
            ("[n]", "New"),
            ("[r]", "Rename"),
            ("[x]", "Kill"),
            ("[a]", "Attach"),
            ("[Esc]", "Back"),
        ],
        View::Templates => vec![("[Enter]", "Create"), ("[Esc]", "Back")],
        View::Help => vec![("[Esc]", "Close")],
    };

    let mut spans = Vec::new();
    for (i, (key, desc)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(*key, theme.key()));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*desc, theme.key_desc()));
    }

    let footer = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(theme.border()),
    );

    frame.render_widget(footer, area);
}

/// Draw status/error message
fn draw_message(frame: &mut Frame, theme: &Theme, message: &str, is_error: bool) {
    let area = frame.size();

    // Position at bottom of screen
    let msg_area = Rect {
        x: 1,
        y: area.height.saturating_sub(4),
        width: area.width.saturating_sub(2),
        height: 1,
    };

    let style = if is_error {
        theme.error()
    } else {
        theme.success()
    };

    let msg = Paragraph::new(message).style(style);

    frame.render_widget(msg, msg_area);
}

/// Center a rect within another
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Create a fixed-size centered rect
pub fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let x = (r.width.saturating_sub(width)) / 2;
    let y = (r.height.saturating_sub(height)) / 2;

    Rect {
        x: r.x + x,
        y: r.y + y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}
