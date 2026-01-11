//! UI module
//!
//! Provides all TUI rendering functionality.

mod help;
mod input;
mod layout;
mod preview;
mod sessions;
mod templates;
pub mod theme;
mod windows;

use ratatui::Frame;

use crate::app::App;

/// Main draw function
pub fn draw(frame: &mut Frame, app: &App) {
    layout::draw(frame, app);
}
