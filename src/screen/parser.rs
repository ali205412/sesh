//! Parser for screen command output
//!
//! Parses output from `screen -ls` and other screen commands.

use anyhow::Result;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use regex::Regex;
use std::sync::OnceLock;

use super::types::{Session, SessionStatus, Window, WindowActivity};

/// Regex for parsing screen -ls output lines
/// Example: "  12345.session-name  (01/15/2024 10:30:45 AM)  (Detached)"
static SESSION_REGEX: OnceLock<Regex> = OnceLock::new();

/// Regex for parsing window list output
/// Example: "  0 bash  1 vim  2-$ editor"
static WINDOW_REGEX: OnceLock<Regex> = OnceLock::new();

fn session_regex() -> &'static Regex {
    SESSION_REGEX.get_or_init(|| {
        Regex::new(r"^\s*(\d+)\.(\S+)\s+\(([^)]+)\)\s+\((\w+)\)").expect("Invalid session regex")
    })
}

fn window_regex() -> &'static Regex {
    WINDOW_REGEX
        .get_or_init(|| Regex::new(r"(\d+)([-*$#!@+]?)\s+(\S+)").expect("Invalid window regex"))
}

/// Parse `screen -ls` output into a list of sessions
pub fn parse_session_list(output: &str, host: Option<&str>) -> Result<Vec<Session>> {
    let mut sessions = Vec::new();
    let regex = session_regex();

    for line in output.lines() {
        if let Some(caps) = regex.captures(line) {
            let pid: u32 = caps
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);

            let name = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let date_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            let status_str = caps.get(4).map(|m| m.as_str()).unwrap_or("");

            let id = format!("{}.{}", pid, name);
            let status = parse_status(status_str);
            let created = parse_datetime(date_str).unwrap_or_else(Local::now);

            let mut session = Session::new(id, name, pid, status);
            session.created = created;
            session.host = host.map(String::from);

            sessions.push(session);
        }
    }

    Ok(sessions)
}

/// Parse session status string
fn parse_status(status: &str) -> SessionStatus {
    match status.to_lowercase().as_str() {
        "detached" => SessionStatus::Detached,
        "attached" => SessionStatus::Attached,
        "multi" => SessionStatus::Multi,
        _ => SessionStatus::Unknown,
    }
}

/// Parse datetime from screen output
/// Handles formats like "01/15/2024 10:30:45 AM" or "01/15/24 10:30:45"
fn parse_datetime(s: &str) -> Option<DateTime<Local>> {
    // Try various formats
    let formats = [
        "%m/%d/%Y %I:%M:%S %p", // 01/15/2024 10:30:45 AM
        "%m/%d/%y %I:%M:%S %p", // 01/15/24 10:30:45 AM
        "%m/%d/%Y %H:%M:%S",    // 01/15/2024 10:30:45
        "%m/%d/%y %H:%M:%S",    // 01/15/24 10:30:45
        "%Y-%m-%d %H:%M:%S",    // 2024-01-15 10:30:45
    ];

    for fmt in &formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(s.trim(), fmt) {
            return Local.from_local_datetime(&naive).single();
        }
    }

    None
}

/// Parse window list output from `screen -Q windows` or `screen -X windows`
/// Output format varies but typically: "0 bash  1 vim  2$ editor"
pub fn parse_window_list(output: &str) -> Result<Vec<Window>> {
    let mut windows = Vec::new();
    let regex = window_regex();

    // The output might be a single line with all windows or multiple lines
    for line in output.lines() {
        for caps in regex.captures_iter(line) {
            let number: usize = caps
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or("0")
                .parse()
                .unwrap_or(0);

            let flags = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let name = caps
                .get(3)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            let mut window = Window::new(number, name);
            window.flags = flags.clone();
            window.active = flags.contains('$') || flags.contains('*');
            window.activity = parse_window_flags(&flags);

            windows.push(window);
        }
    }

    // Sort by window number
    windows.sort_by_key(|w| w.number);

    Ok(windows)
}

/// Parse window flags to determine activity
fn parse_window_flags(flags: &str) -> WindowActivity {
    if flags.contains('@') {
        WindowActivity::Bell
    } else if flags.contains('$') || flags.contains('*') {
        WindowActivity::Active
    } else if flags.contains('+') {
        WindowActivity::Running
    } else {
        WindowActivity::Idle
    }
}

/// Parse hardcopy output (terminal content capture)
pub fn parse_hardcopy(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| {
            // Remove trailing whitespace but preserve leading spaces
            line.trim_end().to_string()
        })
        .collect()
}

/// Parse the output of `screen -Q select` or similar query
pub fn parse_query_response(output: &str) -> Option<String> {
    let trimmed = output.trim();
    if trimmed.is_empty() || trimmed == "-1" {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Check if screen -ls output indicates no sessions
pub fn is_no_sessions(output: &str) -> bool {
    output.contains("No Sockets found") || output.contains("No sessions")
}

/// Extract socket directory from screen -ls output
pub fn parse_socket_dir(output: &str) -> Option<String> {
    // Look for lines like "There is a screen on:" or "There are screens on:"
    // followed by the socket directory path
    for line in output.lines() {
        if line.contains("Socket") || line.contains("socket") {
            // The directory is often in a line like "No Sockets found in /run/screen/S-user"
            if let Some(idx) = line.find("/") {
                let path = &line[idx..];
                // Remove trailing punctuation
                let path = path.trim_end_matches(|c: char| {
                    !c.is_alphanumeric() && c != '/' && c != '-' && c != '_'
                });
                return Some(path.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn test_parse_session_list() {
        let output = r#"There are screens on:
	12345.dev-server	(01/15/2024 10:30:45 AM)	(Attached)
	67890.build	(01/14/2024 09:00:00 AM)	(Detached)
2 Sockets in /run/screen/S-user.
"#;

        let sessions = parse_session_list(output, None).unwrap();
        assert_eq!(sessions.len(), 2);

        assert_eq!(sessions[0].name, "dev-server");
        assert_eq!(sessions[0].pid, 12345);
        assert_eq!(sessions[0].status, SessionStatus::Attached);

        assert_eq!(sessions[1].name, "build");
        assert_eq!(sessions[1].pid, 67890);
        assert_eq!(sessions[1].status, SessionStatus::Detached);
    }

    #[test]
    fn test_parse_session_list_with_host() {
        let output = r#"There is a screen on:
	11111.remote-session	(01/10/2024 08:00:00 AM)	(Detached)
1 Socket in /run/screen/S-deploy.
"#;

        let sessions = parse_session_list(output, Some("server.example.com")).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].host, Some("server.example.com".to_string()));
    }

    #[test]
    fn test_parse_session_list_empty() {
        let output = "No Sockets found in /run/screen/S-user.\n";
        let sessions = parse_session_list(output, None).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_parse_session_list_multi_status() {
        let output = r#"There is a screen on:
	99999.shared-session	(01/20/2024 12:00:00 PM)	(Multi)
1 Socket in /run/screen/S-user.
"#;
        let sessions = parse_session_list(output, None).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].status, SessionStatus::Multi);
    }

    #[test]
    fn test_parse_window_list() {
        // Standard screen window format: number[flags] name
        let output = "0 bash  1 vim  2$ editor  3 shell";

        let windows = parse_window_list(output).unwrap();
        assert_eq!(windows.len(), 4);

        assert_eq!(windows[0].number, 0);
        assert_eq!(windows[0].name, "bash");
        assert!(!windows[0].active);

        assert_eq!(windows[2].number, 2);
        assert_eq!(windows[2].name, "editor");
        assert!(windows[2].active);
    }

    #[test]
    fn test_parse_window_list_with_flags() {
        // Window list with various flags
        let output = "0* bash  1- vim  2$ editor";
        let windows = parse_window_list(output).unwrap();
        assert_eq!(windows.len(), 3);
        assert!(windows[0].active); // * = current
        assert_eq!(windows[1].flags, "-"); // - = previous
        assert!(windows[2].active); // $ = current
    }

    #[test]
    fn test_parse_window_list_multiline() {
        let output = "0 bash\n1 vim\n2$ editor\n3 shell";
        let windows = parse_window_list(output).unwrap();
        assert_eq!(windows.len(), 4);
        assert_eq!(windows[0].name, "bash");
        assert_eq!(windows[3].name, "shell");
    }

    #[test]
    fn test_parse_window_list_with_bell() {
        let output = "0@ alert  1 normal";
        let windows = parse_window_list(output).unwrap();
        assert_eq!(windows[0].activity, WindowActivity::Bell);
        assert_eq!(windows[1].activity, WindowActivity::Idle);
    }

    #[test]
    fn test_parse_window_list_empty() {
        let output = "";
        let windows = parse_window_list(output).unwrap();
        assert!(windows.is_empty());
    }

    #[test]
    fn test_parse_status() {
        assert_eq!(parse_status("Detached"), SessionStatus::Detached);
        assert_eq!(parse_status("Attached"), SessionStatus::Attached);
        assert_eq!(parse_status("Multi"), SessionStatus::Multi);
        assert_eq!(parse_status("detached"), SessionStatus::Detached);
        assert_eq!(parse_status("ATTACHED"), SessionStatus::Attached);
        assert_eq!(parse_status("unknown"), SessionStatus::Unknown);
    }

    #[test]
    fn test_is_no_sessions() {
        assert!(is_no_sessions("No Sockets found in /run/screen/S-user."));
        assert!(is_no_sessions("No sessions found."));
        assert!(!is_no_sessions("There is a screen on:"));
        assert!(!is_no_sessions("There are screens on:"));
    }

    #[test]
    fn test_parse_hardcopy() {
        let content = "Line 1   \nLine 2\n  Line 3  \n";
        let lines = parse_hardcopy(content);

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 1");
        assert_eq!(lines[1], "Line 2");
        assert_eq!(lines[2], "  Line 3"); // Leading space preserved
    }

    #[test]
    fn test_parse_hardcopy_empty() {
        let content = "";
        let lines = parse_hardcopy(content);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_parse_query_response() {
        assert_eq!(
            parse_query_response("  result  "),
            Some("result".to_string())
        );
        assert_eq!(parse_query_response("-1"), None);
        assert_eq!(parse_query_response(""), None);
        assert_eq!(parse_query_response("   "), None);
    }

    #[test]
    fn test_parse_socket_dir() {
        let output = "No Sockets found in /run/screen/S-testuser.";
        assert_eq!(
            parse_socket_dir(output),
            Some("/run/screen/S-testuser".to_string())
        );

        let output2 = "There is a screen on:\n\t12345.test\nSocket in /run/screen/S-user.";
        assert_eq!(
            parse_socket_dir(output2),
            Some("/run/screen/S-user".to_string())
        );
    }

    #[test]
    fn test_window_flags_parsing() {
        assert_eq!(parse_window_flags("*$"), WindowActivity::Active);
        assert_eq!(parse_window_flags("+"), WindowActivity::Running);
        assert_eq!(parse_window_flags("@"), WindowActivity::Bell);
        assert_eq!(parse_window_flags("-"), WindowActivity::Idle);
        assert_eq!(parse_window_flags(""), WindowActivity::Idle);
    }

    // Snapshot tests for complex parsing scenarios
    #[test]
    fn test_snapshot_session_names() {
        // Test various session name formats
        let names = vec![
            ("simple", "dev-server"),
            ("with_underscore", "my_project"),
            ("with_numbers", "project123"),
            ("dotted", "app.v2"),
        ];
        assert_yaml_snapshot!(names);
    }

    #[test]
    fn test_snapshot_window_activities() {
        let activities = vec![
            ("active", WindowActivity::Active),
            ("bell", WindowActivity::Bell),
            ("running", WindowActivity::Running),
            ("idle", WindowActivity::Idle),
        ];
        assert_yaml_snapshot!(activities);
    }
}
