//! Templates widget
//!
//! Renders the template selector view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;

use super::theme::Theme;

/// Draw the templates view
pub fn draw(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    // Split into list and preview
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_template_list(frame, app, theme, chunks[0]);
    draw_template_preview(frame, app, theme, chunks[1]);
}

/// Draw the template list
fn draw_template_list(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused())
        .title(format!(" Templates ({}) ", app.templates.len()));

    if app.templates.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(Span::styled("  No templates found.", theme.muted())),
            Line::from(Span::raw("")),
            Line::from(Span::styled(
                "  Templates should be placed in:",
                theme.muted(),
            )),
            Line::from(Span::styled("  ~/.config/sesh/templates/", theme.accent())),
        ])
        .block(block);

        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .templates
        .iter()
        .map(|template| {
            let window_count = format!("{} windows", template.windows.len());
            let desc = template.description.as_deref().unwrap_or("-");

            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{:<16}", truncate_str(&template.name, 16)),
                    theme.accent(),
                ),
                Span::raw("  "),
                Span::styled(format!("{:<12}", window_count), theme.muted()),
                Span::raw("  "),
                Span::styled(truncate_str(desc, 40), theme.muted()),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(theme.selected())
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.template_index));

    frame.render_stateful_widget(list, area, &mut state);
}

/// Draw template preview/details
fn draw_template_preview(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border())
        .title(" Template Details ");

    if let Some(template) = app.templates.get(app.template_index) {
        let mut lines = vec![Line::from(vec![
            Span::styled("Name: ", theme.muted()),
            Span::styled(&template.name, theme.accent()),
        ])];

        if let Some(ref desc) = template.description {
            lines.push(Line::from(vec![
                Span::styled("Description: ", theme.muted()),
                Span::styled(desc, theme.normal()),
            ]));
        }

        if let Some(ref root) = template.root {
            lines.push(Line::from(vec![
                Span::styled("Root: ", theme.muted()),
                Span::styled(root, theme.normal()),
            ]));
        }

        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled("Windows:", theme.muted())));

        for (i, window) in template.windows.iter().enumerate() {
            let cmd = window.command.as_deref().unwrap_or("<default shell>");

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{}. ", i + 1), theme.muted()),
                Span::styled(&window.name, theme.accent()),
                Span::styled(": ", theme.muted()),
                Span::styled(cmd, theme.normal()),
            ]));
        }

        if !template.variables.is_empty() {
            lines.push(Line::from(Span::raw("")));
            lines.push(Line::from(Span::styled("Variables:", theme.muted())));

            for (name, var) in &template.variables {
                let default_val = var.default.as_deref().unwrap_or("");

                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("${}", name), theme.accent()),
                    Span::styled(" = ", theme.muted()),
                    Span::styled(default_val, theme.normal()),
                ]));
            }
        }

        let preview = Paragraph::new(lines).block(block);
        frame.render_widget(preview, area);
    } else {
        let empty =
            Paragraph::new(Span::styled("No template selected", theme.muted())).block(block);
        frame.render_widget(empty, area);
    }
}

/// Truncate a string to max length
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
