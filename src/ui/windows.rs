//! Windows list widget
//!
//! Renders the window list for a selected session.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::App;
use crate::screen::{Window, WindowActivity};

use super::theme::Theme;

/// Draw the windows list
pub fn draw(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let title = if let Some(ref session) = app.selected_session {
        format!(" Windows - {} ", session)
    } else {
        " Windows ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused())
        .title(title);

    if app.windows.is_empty() {
        let empty_msg = ListItem::new(Line::from(vec![Span::styled(
            "  No windows found.",
            theme.muted(),
        )]));

        let list = List::new(vec![empty_msg]).block(block);

        frame.render_widget(list, area);
        return;
    }

    let items: Vec<ListItem> = app
        .windows
        .iter()
        .map(|window| window_to_list_item(window, theme, area.width))
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(theme.selected())
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.window_index));

    frame.render_stateful_widget(list, area, &mut state);
}

/// Convert a window to a list item
fn window_to_list_item<'a>(window: &'a Window, theme: &Theme, _width: u16) -> ListItem<'a> {
    let number_str = format!("{:>2}", window.number);
    let number = Span::styled(number_str, theme.muted());

    let name = Span::styled(
        format!("{:<20}", truncate_str(&window.name, 20)),
        if window.active {
            theme.accent()
        } else {
            theme.normal()
        },
    );

    let command = if let Some(ref cmd) = window.command {
        Span::styled(truncate_str(cmd, 40), theme.muted())
    } else {
        Span::styled("-", theme.muted())
    };

    let activity = match window.activity {
        WindowActivity::Active => Span::styled("active", theme.success()),
        WindowActivity::Bell => Span::styled("bell", theme.warning()),
        WindowActivity::Running => Span::styled("running", theme.accent()),
        WindowActivity::Idle => Span::styled("idle", theme.muted()),
    };

    let flags = if window.active {
        Span::styled("*", theme.accent())
    } else if window.flags.contains('-') {
        Span::styled("-", theme.muted())
    } else {
        Span::raw(" ")
    };

    ListItem::new(Line::from(vec![
        Span::raw("  "),
        number,
        Span::raw(": "),
        name,
        Span::raw("  "),
        command,
        Span::raw("  "),
        flags,
        Span::raw("  "),
        activity,
    ]))
}

/// Truncate a string to max length
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
