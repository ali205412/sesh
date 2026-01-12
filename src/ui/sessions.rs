//! Session list widget
//!
//! Renders the main session list with status and info.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::App;
use crate::screen::{Session, SessionStatus};

use super::theme::{Symbols, Theme};

/// Draw the session list
pub fn draw(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused())
        .title(" Sessions ");

    // Group sessions by host
    let mut local_sessions: Vec<&Session> = Vec::new();
    let mut remote_sessions: std::collections::HashMap<String, Vec<&Session>> =
        std::collections::HashMap::new();

    for idx in &app.filtered_sessions {
        if let Some(session) = app.sessions.get(*idx) {
            if let Some(ref host) = session.host {
                remote_sessions
                    .entry(host.clone())
                    .or_default()
                    .push(session);
            } else {
                local_sessions.push(session);
            }
        }
    }

    // Build list items
    let mut items: Vec<ListItem> = Vec::new();
    let mut item_to_session_idx: Vec<usize> = Vec::new();

    // Local sessions header
    if !local_sessions.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(" LOCAL ", theme.header()),
            Span::styled(
                "\u{2500}".repeat(area.width.saturating_sub(10) as usize),
                theme.muted(),
            ),
        ])));
        item_to_session_idx.push(usize::MAX); // Header marker

        for (i, session) in local_sessions.iter().enumerate() {
            items.push(session_to_list_item(session, app, theme, area.width));
            // Find the actual index
            if let Some(pos) = app.sessions.iter().position(|s| s.id == session.id) {
                item_to_session_idx.push(pos);
            }
        }
    }

    // Remote sessions by host
    for (host, sessions) in &remote_sessions {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!(" REMOTE ({}) ", host), theme.header()),
            Span::styled(
                "\u{2500}".repeat(area.width.saturating_sub(host.len() as u16 + 14) as usize),
                theme.muted(),
            ),
        ])));
        item_to_session_idx.push(usize::MAX); // Header marker

        for session in sessions {
            items.push(session_to_list_item(session, app, theme, area.width));
            if let Some(pos) = app.sessions.iter().position(|s| s.id == session.id) {
                item_to_session_idx.push(pos);
            }
        }
    }

    // Handle empty state
    if items.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  No sessions found. Press ", theme.muted()),
            Span::styled("[n]", theme.key()),
            Span::styled(" to create one.", theme.muted()),
        ])));
    }

    // Calculate selected index in flat list
    let mut selected = None;
    if !app.filtered_sessions.is_empty() {
        if let Some(session_idx) = app.filtered_sessions.get(app.session_index) {
            selected = item_to_session_idx.iter().position(|&i| i == *session_idx);
        }
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(theme.selected())
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(selected);

    frame.render_stateful_widget(list, area, &mut state);
}

/// Convert a session to a list item
fn session_to_list_item<'a>(
    session: &'a Session,
    _app: &App,
    theme: &Theme,
    width: u16,
) -> ListItem<'a> {
    let status_symbol = match session.status {
        SessionStatus::Attached => Span::styled(Symbols::ATTACHED, theme.status_attached()),
        SessionStatus::Detached => Span::styled(Symbols::DETACHED, theme.status_detached()),
        SessionStatus::Multi => Span::styled(Symbols::MULTI, theme.status_multi()),
        SessionStatus::Unknown => Span::styled("?", theme.muted()),
    };

    let status_text = match session.status {
        SessionStatus::Attached => Span::styled("attached", theme.status_attached()),
        SessionStatus::Detached => Span::styled("detached", theme.status_detached()),
        SessionStatus::Multi => Span::styled("multi", theme.status_multi()),
        SessionStatus::Unknown => Span::styled("unknown", theme.muted()),
    };

    let window_count = Span::styled(format!("{} win", session.window_count), theme.muted());

    let age = Span::styled(session.age_string(), theme.muted());

    // Calculate dynamic name width based on terminal width
    // Reserve space for: "  " + symbol + " " + "  " + window_count + "  " + status + "  " + age
    // Approx: 2 + 1 + 1 + 2 + 6 + 2 + 8 + 2 + 4 = 28 chars fixed overhead
    let fixed_overhead: u16 = 32;
    let available = width.saturating_sub(fixed_overhead);
    // Use at least 12 chars, at most 50, scale with available space
    let name_width = (available / 2).clamp(12, 50) as usize;

    // Git info if available
    let git_info = if let Some(ref branch) = session.git_branch {
        let git_symbol = if session.git_clean.unwrap_or(true) {
            Span::styled(Symbols::GIT_CLEAN, theme.git_clean())
        } else {
            Span::styled(Symbols::GIT_DIRTY, theme.git_dirty())
        };
        vec![
            Span::raw("  "),
            Span::styled(truncate_str(branch, 20), theme.accent()),
            Span::raw(" "),
            git_symbol,
        ]
    } else {
        vec![]
    };

    // Working directory - use remaining space
    let dir_width = if width > 100 {
        30
    } else if width > 80 {
        20
    } else {
        0
    };
    let dir_info = if dir_width > 0 {
        if let Some(ref dir) = session.working_dir {
            vec![
                Span::raw("  "),
                Span::styled(truncate_str(dir, dir_width), theme.muted()),
            ]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Build the line - session name uses full calculated width
    let mut spans = vec![
        Span::raw("  "),
        status_symbol,
        Span::raw(" "),
        Span::styled(session.name.clone(), theme.normal()),
    ];

    // Add padding to align columns only if name is shorter than width
    let name_len = session.name.len();
    if name_len < name_width {
        spans.push(Span::raw(" ".repeat(name_width - name_len)));
    }

    spans.push(Span::raw("  "));
    spans.push(window_count);
    spans.push(Span::raw("  "));
    spans.push(status_text);

    spans.extend(dir_info);
    spans.extend(git_info);

    spans.push(Span::raw("  "));
    spans.push(age);

    ListItem::new(Line::from(spans))
}

/// Truncate a string to max length
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
