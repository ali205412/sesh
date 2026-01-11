//! UI theme and styling
//!
//! Defines colors, styles, and visual elements for the TUI.

use ratatui::style::{Color, Modifier, Style};

/// Theme configuration
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary foreground color
    pub fg: Color,
    /// Primary background color
    pub bg: Color,
    /// Accent color for highlights
    pub accent: Color,
    /// Secondary accent color
    pub accent_secondary: Color,
    /// Success/positive color
    pub success: Color,
    /// Warning color
    pub warning: Color,
    /// Error color
    pub error: Color,
    /// Muted/dimmed color
    pub muted: Color,
    /// Border color
    pub border: Color,
    /// Selected item background
    pub selected_bg: Color,
    /// Selected item foreground
    pub selected_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Dark theme (default)
    pub fn dark() -> Self {
        Self {
            fg: Color::White,
            bg: Color::Reset,
            accent: Color::Cyan,
            accent_secondary: Color::Blue,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::DarkGray,
            border: Color::DarkGray,
            selected_bg: Color::Rgb(40, 44, 52),
            selected_fg: Color::Cyan,
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            fg: Color::Black,
            bg: Color::White,
            accent: Color::Blue,
            accent_secondary: Color::Cyan,
            success: Color::Green,
            warning: Color::Rgb(200, 150, 0),
            error: Color::Red,
            muted: Color::Gray,
            border: Color::Gray,
            selected_bg: Color::Rgb(230, 240, 255),
            selected_fg: Color::Blue,
        }
    }

    // Style methods

    /// Normal text style
    pub fn normal(&self) -> Style {
        Style::default().fg(self.fg)
    }

    /// Muted/dimmed text
    pub fn muted(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Accent colored text
    pub fn accent(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Bold text
    pub fn bold(&self) -> Style {
        Style::default().fg(self.fg).add_modifier(Modifier::BOLD)
    }

    /// Title style
    pub fn title(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Border style
    pub fn border(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Border style when focused
    pub fn border_focused(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Selected item style
    pub fn selected(&self) -> Style {
        Style::default()
            .fg(self.selected_fg)
            .bg(self.selected_bg)
            .add_modifier(Modifier::BOLD)
    }

    /// Unselected item style
    pub fn unselected(&self) -> Style {
        Style::default().fg(self.fg)
    }

    /// Success/positive style
    pub fn success(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Warning style
    pub fn warning(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Error style
    pub fn error(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Status: attached
    pub fn status_attached(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Status: detached
    pub fn status_detached(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Status: multi
    pub fn status_multi(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Key hint style
    pub fn key(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Key description style
    pub fn key_desc(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Header/section style
    pub fn header(&self) -> Style {
        Style::default().fg(self.muted).add_modifier(Modifier::BOLD)
    }

    /// Git clean status
    pub fn git_clean(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Git dirty status
    pub fn git_dirty(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Preview line numbers
    pub fn line_number(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Preview content
    pub fn preview_content(&self) -> Style {
        Style::default().fg(self.fg)
    }

    /// Input field style
    pub fn input(&self) -> Style {
        Style::default().fg(self.fg).bg(Color::Rgb(30, 30, 30))
    }

    /// Input cursor style
    pub fn input_cursor(&self) -> Style {
        Style::default().fg(self.bg).bg(self.fg)
    }

    /// Prompt text style
    pub fn prompt(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    /// Help overlay style
    pub fn help_overlay(&self) -> Style {
        Style::default().fg(self.fg).bg(Color::Rgb(20, 20, 20))
    }
}

/// Symbols used in the UI
pub struct Symbols;

impl Symbols {
    /// Attached session indicator
    pub const ATTACHED: &'static str = "\u{25CF}"; // ●
    /// Detached session indicator
    pub const DETACHED: &'static str = "\u{25CB}"; // ○
    /// Multi-user session indicator
    pub const MULTI: &'static str = "\u{25C9}"; // ◉
    /// Selection cursor
    pub const CURSOR: &'static str = ">";
    /// Git clean indicator
    pub const GIT_CLEAN: &'static str = "\u{2713}"; // ✓
    /// Git dirty indicator
    pub const GIT_DIRTY: &'static str = "\u{2717}"; // ✗
    /// Arrow right
    pub const ARROW_RIGHT: &'static str = "\u{2192}"; // →
    /// Separator
    pub const SEPARATOR: &'static str = "\u{2502}"; // │
    /// Box drawing characters
    pub const BOX_TOP_LEFT: &'static str = "\u{250C}"; // ┌
    pub const BOX_TOP_RIGHT: &'static str = "\u{2510}"; // ┐
    pub const BOX_BOTTOM_LEFT: &'static str = "\u{2514}"; // └
    pub const BOX_BOTTOM_RIGHT: &'static str = "\u{2518}"; // ┘
    pub const BOX_HORIZONTAL: &'static str = "\u{2500}"; // ─
    pub const BOX_VERTICAL: &'static str = "\u{2502}"; // │
}

/// Format session count string
pub fn format_session_count(local: usize, remote: usize) -> String {
    if remote > 0 {
        format!("{} local, {} remote", local, remote)
    } else {
        format!("{}", local)
    }
}

/// Format window count string
pub fn format_window_count(count: usize) -> String {
    if count == 1 {
        "1 win".to_string()
    } else {
        format!("{} win", count)
    }
}

/// Format time ago string
pub fn format_time_ago(age: &str) -> String {
    age.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_session_count() {
        assert_eq!(format_session_count(3, 0), "3");
        assert_eq!(format_session_count(3, 2), "3 local, 2 remote");
    }

    #[test]
    fn test_format_window_count() {
        assert_eq!(format_window_count(1), "1 win");
        assert_eq!(format_window_count(5), "5 win");
    }
}
