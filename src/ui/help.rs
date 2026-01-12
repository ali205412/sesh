//! Help overlay widget
//!
//! Renders the help overlay with keyboard shortcuts in two columns.

use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

use super::{layout::centered_rect, theme::Theme};

/// Draw the help overlay
pub fn draw(frame: &mut Frame, _app: &App, theme: &Theme) {
    let area = centered_rect(80, 70, frame.size());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused())
        .title(" Help - Press ? or Esc to close ")
        .style(theme.help_overlay());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into two columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Left column sections
    let left_sections = vec![
        (
            "Navigation",
            vec![
                ("j / ↓", "Move down"),
                ("k / ↑", "Move up"),
                ("g / Home", "Go to top"),
                ("G / End", "Go to bottom"),
                ("PgUp/PgDn", "Page up/down"),
                ("Tab", "Switch host"),
            ],
        ),
        (
            "Sessions",
            vec![
                ("Enter", "Attach"),
                ("A", "Attach (new terminal)"),
                ("n", "New session"),
                ("R", "Rename session"),
                ("d", "Detach"),
                ("x", "Kill"),
                ("w", "View windows"),
                ("r", "Refresh"),
            ],
        ),
        (
            "Windows",
            vec![
                ("Enter", "Select & attach"),
                ("n", "New window"),
                ("r", "Rename"),
                ("x", "Kill"),
            ],
        ),
    ];

    // Right column sections
    let right_sections = vec![
        (
            "Templates",
            vec![("t", "Open templates"), ("Enter", "Create from template")],
        ),
        (
            "Search",
            vec![
                ("/", "Start search"),
                ("Enter", "Confirm"),
                ("Esc", "Clear"),
            ],
        ),
        (
            "Settings",
            vec![
                ("S", "Open settings"),
                ("h / l", "Switch category"),
                ("Enter", "Toggle value"),
            ],
        ),
        (
            "General",
            vec![
                ("?", "Toggle help"),
                ("p", "Toggle preview"),
                ("Esc", "Back / Close"),
                ("q", "Quit"),
                ("Ctrl-c", "Force quit"),
            ],
        ),
    ];

    // Render left column
    let left_lines = build_section_lines(&left_sections, theme);
    let left_para = Paragraph::new(left_lines);
    frame.render_widget(left_para, columns[0]);

    // Render right column
    let right_lines = build_section_lines(&right_sections, theme);
    let right_para = Paragraph::new(right_lines);
    frame.render_widget(right_para, columns[1]);
}

fn build_section_lines<'a>(sections: &[(&str, Vec<(&str, &str)>)], theme: &Theme) -> Vec<Line<'a>> {
    let mut lines: Vec<Line> = Vec::new();

    for (section_name, keys) in sections {
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            format!(" {}", section_name),
            theme.header(),
        )));

        for (key, desc) in keys {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{:<12}", key), theme.key()),
                Span::styled(desc.to_string(), theme.key_desc()),
            ]));
        }
    }

    lines
}
