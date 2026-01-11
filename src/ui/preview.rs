//! Preview widget
//!
//! Renders a preview of terminal content from the selected session.

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

use super::theme::Theme;

/// Draw the preview panel
pub fn draw(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let title = if let Some(session) = get_preview_session_name(app) {
        format!(" Preview - {} ", session)
    } else {
        " Preview ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border())
        .title(title);

    let inner_height = area.height.saturating_sub(2) as usize;

    if app.preview.lines.is_empty() {
        let empty = Paragraph::new(vec![Line::from(Span::styled(
            "No preview available",
            theme.muted(),
        ))])
        .block(block);

        frame.render_widget(empty, area);
        return;
    }

    // Get visible lines with line numbers
    let lines: Vec<Line> = app
        .preview
        .visible_lines(inner_height)
        .iter()
        .map(|(num, content)| {
            Line::from(vec![
                Span::styled(format!("{:>4}", num), theme.line_number()),
                Span::styled("\u{2502} ", theme.muted()),
                Span::styled((*content).to_string(), theme.preview_content()),
            ])
        })
        .collect();

    let preview = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(preview, area);
}

/// Get the name of the session being previewed
fn get_preview_session_name(app: &App) -> Option<String> {
    match app.view {
        crate::app::View::Sessions => app
            .filtered_sessions
            .get(app.session_index)
            .and_then(|&idx| app.sessions.get(idx))
            .map(|s| s.name.clone()),
        crate::app::View::Windows => app.selected_session.clone(),
        _ => None,
    }
}
