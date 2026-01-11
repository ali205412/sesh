//! Core data types for screen sessions and windows

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Status of a screen session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SessionStatus {
    /// Session has no attached displays
    Detached,
    /// Session has one or more attached displays
    Attached,
    /// Session is in multiuser mode with multiple attachments
    Multi,
    /// Session status is unknown
    #[default]
    Unknown,
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionStatus::Detached => write!(f, "detached"),
            SessionStatus::Attached => write!(f, "attached"),
            SessionStatus::Multi => write!(f, "multi"),
            SessionStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Represents a screen session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session name (the part after the PID, e.g., "dev-server" in "12345.dev-server")
    pub name: String,

    /// Full session identifier (PID.name format)
    pub id: String,

    /// Process ID of the screen backend
    pub pid: u32,

    /// Current status
    pub status: SessionStatus,

    /// Number of windows in the session
    pub window_count: usize,

    /// When the session was created
    pub created: DateTime<Local>,

    /// Host this session is on (None for local)
    pub host: Option<String>,

    /// Working directory of the session (if known)
    pub working_dir: Option<String>,

    /// Git branch (if in a git repo)
    pub git_branch: Option<String>,

    /// Git status (clean = true, dirty = false)
    pub git_clean: Option<bool>,

    /// List of attached users (for multiuser sessions)
    pub attached_users: Vec<String>,
}

impl Session {
    /// Create a new session with minimal info
    pub fn new(id: String, name: String, pid: u32, status: SessionStatus) -> Self {
        Self {
            name,
            id,
            pid,
            status,
            window_count: 0,
            created: Local::now(),
            host: None,
            working_dir: None,
            git_branch: None,
            git_clean: None,
            attached_users: Vec::new(),
        }
    }

    /// Get display name (includes host if remote)
    pub fn display_name(&self) -> String {
        if let Some(host) = &self.host {
            format!("{}@{}", self.name, host)
        } else {
            self.name.clone()
        }
    }

    /// Check if this is a local session
    pub fn is_local(&self) -> bool {
        self.host.is_none()
    }

    /// Check if session is attached
    pub fn is_attached(&self) -> bool {
        matches!(self.status, SessionStatus::Attached | SessionStatus::Multi)
    }

    /// Get age as human-readable string
    pub fn age_string(&self) -> String {
        let now = Local::now();
        let duration = now.signed_duration_since(self.created);

        if duration.num_days() > 0 {
            format!("{}d", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{}h", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{}m", duration.num_minutes())
        } else {
            "now".to_string()
        }
    }
}

/// Represents a window within a screen session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    /// Window number (0-based index)
    pub number: usize,

    /// Window name/title
    pub name: String,

    /// Command running in the window (if known)
    pub command: Option<String>,

    /// Whether this is the currently active window
    pub active: bool,

    /// Window flags from screen (e.g., "$" for current, "-" for previous)
    pub flags: String,

    /// Activity status
    pub activity: WindowActivity,
}

impl Window {
    /// Create a new window
    pub fn new(number: usize, name: String) -> Self {
        Self {
            number,
            name,
            command: None,
            active: false,
            flags: String::new(),
            activity: WindowActivity::Idle,
        }
    }
}

/// Activity state of a window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WindowActivity {
    /// Window is idle
    #[default]
    Idle,
    /// Window has recent activity
    Active,
    /// Window has output (bell/activity monitoring)
    Bell,
    /// Process is running
    Running,
}

impl fmt::Display for WindowActivity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowActivity::Idle => write!(f, "idle"),
            WindowActivity::Active => write!(f, "active"),
            WindowActivity::Bell => write!(f, "bell"),
            WindowActivity::Running => write!(f, "running"),
        }
    }
}

/// Terminal preview content
#[derive(Debug, Clone, Default)]
pub struct Preview {
    /// Lines of terminal content
    pub lines: Vec<String>,

    /// Total number of lines in scrollback
    pub total_lines: usize,

    /// Current scroll offset
    pub scroll_offset: usize,

    /// Whether preview is being updated
    pub updating: bool,
}

impl Preview {
    /// Create a new empty preview
    pub fn new() -> Self {
        Self::default()
    }

    /// Get visible lines with line numbers
    pub fn visible_lines(&self, height: usize) -> Vec<(usize, &str)> {
        self.lines
            .iter()
            .skip(self.scroll_offset)
            .take(height)
            .enumerate()
            .map(|(i, line)| (self.scroll_offset + i + 1, line.as_str()))
            .collect()
    }

    /// Scroll up by n lines
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    /// Scroll down by n lines
    pub fn scroll_down(&mut self, n: usize, height: usize) {
        let max_offset = self.lines.len().saturating_sub(height);
        self.scroll_offset = (self.scroll_offset + n).min(max_offset);
    }
}

/// Grouped sessions by host
#[derive(Debug, Clone)]
pub struct SessionGroup {
    /// Host name (None for local)
    pub host: Option<String>,

    /// Sessions in this group
    pub sessions: Vec<Session>,
}

impl SessionGroup {
    /// Create a new session group
    pub fn new(host: Option<String>) -> Self {
        Self {
            host,
            sessions: Vec::new(),
        }
    }

    /// Get display name for the group
    pub fn display_name(&self) -> &str {
        self.host.as_deref().unwrap_or("LOCAL")
    }

    /// Check if this is the local group
    pub fn is_local(&self) -> bool {
        self.host.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_display_name() {
        let mut session = Session::new(
            "12345.dev".to_string(),
            "dev".to_string(),
            12345,
            SessionStatus::Detached,
        );

        assert_eq!(session.display_name(), "dev");

        session.host = Some("server.example.com".to_string());
        assert_eq!(session.display_name(), "dev@server.example.com");
    }

    #[test]
    fn test_session_status_display() {
        assert_eq!(format!("{}", SessionStatus::Detached), "detached");
        assert_eq!(format!("{}", SessionStatus::Attached), "attached");
        assert_eq!(format!("{}", SessionStatus::Multi), "multi");
    }

    #[test]
    fn test_preview_scrolling() {
        let mut preview = Preview {
            lines: (0..100).map(|i| format!("Line {}", i)).collect(),
            total_lines: 100,
            scroll_offset: 0,
            updating: false,
        };

        assert_eq!(preview.scroll_offset, 0);

        preview.scroll_down(10, 20);
        assert_eq!(preview.scroll_offset, 10);

        preview.scroll_up(5);
        assert_eq!(preview.scroll_offset, 5);

        // Can't scroll past beginning
        preview.scroll_up(100);
        assert_eq!(preview.scroll_offset, 0);

        // Can't scroll past end
        preview.scroll_down(200, 20);
        assert_eq!(preview.scroll_offset, 80); // 100 - 20
    }
}
