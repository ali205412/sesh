//! fzf integration
//!
//! Provides fuzzy finding using fzf when available.

use anyhow::Result;
use std::io::Write;
use std::process::{Command, Stdio};

/// Check if fzf is available
pub fn is_available() -> bool {
    Command::new("which")
        .arg("fzf")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Run fzf with a list of items and return the selected item
pub fn select<T: AsRef<str>>(items: &[T], prompt: Option<&str>) -> Result<Option<String>> {
    if !is_available() {
        anyhow::bail!("fzf is not installed");
    }

    let mut cmd = Command::new("fzf");

    if let Some(p) = prompt {
        cmd.arg("--prompt").arg(p);
    }

    cmd.arg("--height=40%")
        .arg("--layout=reverse")
        .arg("--border")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn()?;

    // Write items to fzf's stdin
    if let Some(mut stdin) = child.stdin.take() {
        for item in items {
            writeln!(stdin, "{}", item.as_ref())?;
        }
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Some(selected))
    } else {
        // User cancelled or no selection
        Ok(None)
    }
}

/// Run fzf with multi-select mode
pub fn select_multiple<T: AsRef<str>>(items: &[T], prompt: Option<&str>) -> Result<Vec<String>> {
    if !is_available() {
        anyhow::bail!("fzf is not installed");
    }

    let mut cmd = Command::new("fzf");

    if let Some(p) = prompt {
        cmd.arg("--prompt").arg(p);
    }

    cmd.arg("--multi")
        .arg("--height=40%")
        .arg("--layout=reverse")
        .arg("--border")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn()?;

    // Write items to fzf's stdin
    if let Some(mut stdin) = child.stdin.take() {
        for item in items {
            writeln!(stdin, "{}", item.as_ref())?;
        }
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        let selected: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();
        Ok(selected)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available() {
        // Just check it doesn't panic
        let _ = is_available();
    }
}
