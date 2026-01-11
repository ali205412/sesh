//! Help overlay widget
//!
//! Renders the help overlay with keyboard shortcuts.

use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

use super::{layout::centered_rect, theme::Theme};

/// Draw the help overlay
pub fn draw(frame: &mut Frame, _app: &App, theme: &Theme) {
    let area = centered_rect(60, 80, frame.size());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused())
        .title(" Help - Keyboard Shortcuts ")
        .style(theme.help_overlay());

    let sections = vec![
        (
            "Navigation",
            vec![
                ("j / Down", "Move down"),
                ("k / Up", "Move up"),
                ("g / Home", "Go to top"),
                ("G / End", "Go to bottom"),
                ("PageUp", "Page up"),
                ("PageDown", "Page down"),
                ("Tab", "Switch host"),
            ],
        ),
        (
            "Session Actions",
            vec![
                ("Enter", "Attach to session"),
                ("A", "Attach in new terminal"),
                ("n", "Create new session"),
                ("d", "Detach session"),
                ("x", "Kill session"),
                ("w", "View windows"),
                ("r", "Refresh list"),
            ],
        ),
        (
            "Window Actions",
            vec![
                ("Enter", "Select window and attach"),
                ("n", "Create new window"),
                ("r", "Rename window"),
                ("x", "Kill window"),
                ("Esc", "Go back"),
            ],
        ),
        (
            "Templates",
            vec![
                ("t", "Open template selector"),
                ("Enter", "Create from template"),
                ("Esc", "Cancel"),
            ],
        ),
        (
            "Search & Filter",
            vec![
                ("/", "Start search"),
                ("Esc", "Clear search"),
                ("Enter", "Confirm search"),
            ],
        ),
        (
            "UI Toggles",
            vec![
                ("?", "Toggle this help"),
                ("p", "Toggle preview"),
                ("q", "Quit sesh"),
                ("Ctrl-c", "Force quit"),
            ],
        ),
    ];

    let mut lines: Vec<Line> = Vec::new();

    for (section_name, keys) in sections {
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            format!(" {} ", section_name),
            theme.header(),
        )));
        lines.push(Line::from(Span::raw("")));

        for (key, desc) in keys {
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(format!("{:<12}", key), theme.key()),
                Span::styled(desc, theme.key_desc()),
            ]));
        }
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        " Press ? or Esc to close ",
        theme.muted(),
    )));

    let help = Paragraph::new(lines).block(block);

    frame.render_widget(help, area);
}
