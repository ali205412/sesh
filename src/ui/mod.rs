//! UI module
//!
//! Provides all TUI rendering functionality.

mod help;
mod input;
mod layout;
mod preview;
mod sessions;
pub mod settings;
mod templates;
pub mod theme;
mod windows;

use ratatui::Frame;

use crate::app::{App, View};

/// Main draw function
pub fn draw(frame: &mut Frame, app: &App) {
    match app.view {
        View::Settings => {
            // Draw main layout first, then overlay settings
            layout::draw(frame, app);
            let area = centered_rect(80, 80, frame.size());
            settings::draw(frame, app, &app.theme, area);
        }
        _ => layout::draw(frame, app),
    }
}

/// Create a centered rectangle
fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    use ratatui::layout::{Constraint, Direction, Layout};

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
