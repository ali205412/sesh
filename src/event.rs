//! Event handling for the TUI
//!
//! Manages keyboard, mouse, and terminal events.

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    MouseEvent, MouseEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use std::io::{stdout, Stdout};
use std::time::Duration;
use tokio::sync::mpsc;

/// Application events
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Key press event
    Key(KeyEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
    /// Tick event for periodic updates
    Tick,
    /// Request to quit the application
    Quit,
    /// Error event
    Error(String),
}

/// Keyboard action that the app should handle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    // Navigation
    Up,
    Down,
    Left,
    Right,
    Top,
    Bottom,
    PageUp,
    PageDown,

    // Selection
    Select,
    Back,

    // Actions
    NewSession,
    RenameSession,
    KillSession,
    DetachSession,
    AttachSession,
    AttachSpawn,
    ViewWindows,
    ViewTemplates,
    ViewSettings,
    Refresh,
    RefreshAll, // Include remote sessions

    // Settings actions
    AddHost,
    EditHost,
    DeleteHost,

    // Search
    StartSearch,
    ClearSearch,

    // UI
    ToggleHelp,
    TogglePreview,
    SwitchHost,

    // Input mode
    InputChar(char),
    InputBackspace,
    InputDelete,
    InputConfirm,
    InputCancel,

    // System
    Quit,
    ForceQuit,

    // No action
    None,
}

/// Event handler configuration
pub struct EventConfig {
    /// Tick rate in milliseconds
    pub tick_rate_ms: u64,
    /// Whether mouse is enabled
    pub mouse_enabled: bool,
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: 250,
            mouse_enabled: true,
        }
    }
}

/// Event handler that runs in a separate task
pub struct EventHandler {
    /// Channel receiver for events
    rx: mpsc::UnboundedReceiver<AppEvent>,
    /// Handle to the event task
    _task: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(config: EventConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let tick_rate = Duration::from_millis(config.tick_rate_ms);

        let task = tokio::spawn(async move {
            loop {
                // Poll for events with timeout
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            if tx.send(AppEvent::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(Event::Mouse(mouse)) => {
                            if tx.send(AppEvent::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Ok(Event::Resize(w, h)) => {
                            if tx.send(AppEvent::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            let _ = tx.send(AppEvent::Error(e.to_string()));
                        }
                    }
                } else {
                    // Tick event
                    if tx.send(AppEvent::Tick).is_err() {
                        break;
                    }
                }
            }
        });

        Self { rx, _task: task }
    }

    /// Get the next event
    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}

/// Parse a key event into an action based on current mode
pub fn key_to_action(key: KeyEvent, in_input_mode: bool, in_search_mode: bool) -> Action {
    if in_input_mode {
        return input_mode_action(key);
    }

    if in_search_mode {
        return search_mode_action(key);
    }

    normal_mode_action(key)
}

/// Actions in normal (navigation) mode
fn normal_mode_action(key: KeyEvent) -> Action {
    match key.code {
        // Quit
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::ForceQuit,

        // Navigation - vim style
        KeyCode::Char('j') | KeyCode::Down => Action::Down,
        KeyCode::Char('k') | KeyCode::Up => Action::Up,
        KeyCode::Char('h') | KeyCode::Left => Action::Left,
        KeyCode::Char('l') | KeyCode::Right => Action::Right,
        KeyCode::Char('g') => Action::Top,
        KeyCode::Char('G') => Action::Bottom,
        KeyCode::Home => Action::Top,
        KeyCode::End => Action::Bottom,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,

        // Selection
        KeyCode::Enter => Action::Select,
        KeyCode::Esc => Action::Back,
        KeyCode::Backspace => Action::Back,

        // Actions
        KeyCode::Char('n') => Action::NewSession,
        KeyCode::Char('R') => Action::RenameSession,
        KeyCode::Char('x') => Action::KillSession,
        KeyCode::Char('d') => Action::DetachSession,
        KeyCode::Char('a') => Action::AttachSession,
        KeyCode::Char('A') => Action::AttachSpawn,
        KeyCode::Char('w') => Action::ViewWindows,
        KeyCode::Char('t') => Action::ViewTemplates,
        KeyCode::Char('S') => Action::ViewSettings,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('F') => Action::RefreshAll, // F for full refresh including remotes

        // Search
        KeyCode::Char('/') => Action::StartSearch,

        // UI toggles
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('p') => Action::TogglePreview,
        KeyCode::Tab => Action::SwitchHost,

        _ => Action::None,
    }
}

/// Actions in search/filter mode
fn search_mode_action(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::ClearSearch,
        KeyCode::Enter => Action::InputConfirm,
        KeyCode::Backspace => Action::InputBackspace,
        KeyCode::Delete => Action::InputDelete,
        KeyCode::Char(c) => Action::InputChar(c),

        // Allow navigation while searching
        KeyCode::Up => Action::Up,
        KeyCode::Down => Action::Down,

        _ => Action::None,
    }
}

/// Actions in text input mode (e.g., naming a session)
fn input_mode_action(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::InputCancel,
        KeyCode::Enter => Action::InputConfirm,
        KeyCode::Backspace => Action::InputBackspace,
        KeyCode::Delete => Action::InputDelete,
        KeyCode::Char(c) => Action::InputChar(c),
        KeyCode::Left => Action::Left,
        KeyCode::Right => Action::Right,
        KeyCode::Home => Action::Top,
        KeyCode::End => Action::Bottom,
        _ => Action::None,
    }
}

/// Parse mouse event into an action
pub fn mouse_to_action(
    mouse: MouseEvent,
    list_bounds: (u16, u16, u16, u16),
) -> Option<(Action, usize)> {
    let (x, y, width, height) = list_bounds;

    // Check if mouse is within list bounds
    if mouse.column < x || mouse.column >= x + width || mouse.row < y || mouse.row >= y + height {
        return None;
    }

    let row_index = (mouse.row - y) as usize;

    match mouse.kind {
        MouseEventKind::Down(event::MouseButton::Left) => Some((Action::Select, row_index)),
        MouseEventKind::ScrollUp => Some((Action::Up, row_index)),
        MouseEventKind::ScrollDown => Some((Action::Down, row_index)),
        _ => None,
    }
}

/// Terminal management
pub struct Terminal {
    /// The terminal backend
    pub backend: ratatui::Terminal<ratatui::backend::CrosstermBackend<Stdout>>,
}

impl Terminal {
    /// Create and initialize a new terminal
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(EnableMouseCapture)?;

        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;

        Ok(Self { backend: terminal })
    }

    /// Restore terminal to original state
    pub fn restore(&mut self) -> Result<()> {
        disable_raw_mode()?;
        self.backend.backend_mut().execute(LeaveAlternateScreen)?;
        self.backend.backend_mut().execute(DisableMouseCapture)?;
        self.backend.show_cursor()?;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_mode_navigation() {
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(key_to_action(key, false, false), Action::Down);

        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(key_to_action(key, false, false), Action::Up);

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(key_to_action(key, false, false), Action::Quit);
    }

    #[test]
    fn test_input_mode() {
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(key_to_action(key, true, false), Action::InputChar('a'));

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(key_to_action(key, true, false), Action::InputCancel);
    }

    #[test]
    fn test_search_mode() {
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(key_to_action(key, false, true), Action::InputChar('x'));

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(key_to_action(key, false, true), Action::ClearSearch);
    }
}
