//! Input dialogs
//!
//! Renders input dialogs, search bars, and confirmation prompts.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

use super::{layout::centered_rect_fixed, theme::Theme};

/// Draw a text input dialog
pub fn draw_input_dialog(frame: &mut Frame, app: &App, theme: &Theme, prompt: &str) {
    let area = centered_rect_fixed(50, 5, frame.size());

    // Clear background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused())
        .title(format!(" {} ", prompt));

    // Input line with cursor
    let input_text = &app.input_buffer;
    let cursor_pos = app.input_cursor;

    let (before, after) = input_text.split_at(cursor_pos.min(input_text.len()));
    let cursor_char = after.chars().next().unwrap_or(' ');
    let after_cursor = if after.len() > 1 { &after[1..] } else { "" };

    let input_line = Line::from(vec![
        Span::styled(before, theme.normal()),
        Span::styled(cursor_char.to_string(), theme.input_cursor()),
        Span::styled(after_cursor, theme.normal()),
    ]);

    let help_line = Line::from(vec![
        Span::styled("[Enter] ", theme.key()),
        Span::styled("Confirm  ", theme.key_desc()),
        Span::styled("[Esc] ", theme.key()),
        Span::styled("Cancel", theme.key_desc()),
    ]);

    let content = Paragraph::new(vec![
        Line::from(Span::raw("")),
        input_line,
        Line::from(Span::raw("")),
        help_line,
    ])
    .block(block);

    frame.render_widget(content, area);
}

/// Draw a confirmation dialog
pub fn draw_confirm_dialog(frame: &mut Frame, _app: &App, theme: &Theme, message: &str) {
    let area = centered_rect_fixed(50, 6, frame.size());

    // Clear background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused())
        .title(" Confirm ");

    let content = Paragraph::new(vec![
        Line::from(Span::raw("")),
        Line::from(Span::styled(message, theme.warning())),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("[Enter] ", theme.key()),
            Span::styled("Yes  ", theme.key_desc()),
            Span::styled("[Esc] ", theme.key()),
            Span::styled("No", theme.key_desc()),
        ]),
    ])
    .block(block);

    frame.render_widget(content, area);
}

/// Draw the search bar
pub fn draw_search_bar(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = frame.size();

    // Position at bottom, above footer
    let search_area = Rect {
        x: area.x + 1,
        y: area.height.saturating_sub(5),
        width: area.width.saturating_sub(2),
        height: 1,
    };

    let search_line = Line::from(vec![
        Span::styled(" / ", theme.prompt()),
        Span::styled(&app.search_query, theme.normal()),
        Span::styled("_", theme.input_cursor()),
    ]);

    frame.render_widget(Paragraph::new(search_line), search_area);
}
