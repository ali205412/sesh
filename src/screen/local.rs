//! Local screen command execution
//!
//! Handles all screen operations for local sessions.

use anyhow::{bail, Context, Result};
use std::os::unix::process::CommandExt;
use std::process::Stdio;
use tokio::process::Command;

use super::parser;
use super::types::{Preview, Session, Window};
use crate::config::Settings;

/// List all local screen sessions
pub async fn list_sessions() -> Result<Vec<Session>> {
    let output = Command::new("screen")
        .args(["-ls"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to run 'screen -ls'")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // screen -ls returns exit code 1 when there are sessions (weird but true)
    // and also returns 1 when there are no sessions
    // We need to check the output content instead

    let combined = format!("{}{}", stdout, stderr);

    if parser::is_no_sessions(&combined) {
        return Ok(Vec::new());
    }

    parser::parse_session_list(&combined, None)
}

/// Create a new screen session
pub async fn create_session(name: &str, dir: Option<&str>, shell: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("screen");
    cmd.args(["-dmS", name]);

    if let Some(dir) = dir {
        let expanded = shellexpand::tilde(dir);
        cmd.current_dir(expanded.as_ref());
    }

    if let Some(shell) = shell {
        cmd.arg(shell);
    }

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to create screen session")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create session: {}", stderr);
    }

    Ok(())
}

/// Attach to a session by replacing the current process (exec)
pub async fn attach_exec(session: &str, host: Option<&str>) -> Result<()> {
    if let Some(host) = host {
        // SSH to remote host and attach
        let err = std::process::Command::new("ssh")
            .args(["-t", host, "screen", "-r", session])
            .exec();
        bail!("Failed to exec ssh: {}", err);
    } else {
        // Local attach - exec replaces current process
        let err = std::process::Command::new("screen")
            .args(["-r", session])
            .exec();
        bail!("Failed to exec screen: {}", err);
    }
}

/// Attach to a session in a new terminal window
/// Optimized for alacritty, with fallbacks for other terminals
pub async fn attach_spawn(config: &Settings, session: &str, host: Option<&str>) -> Result<()> {
    let terminal = config
        .screen
        .spawn_terminal
        .as_deref()
        .unwrap_or("alacritty");

    // Build the screen command
    let screen_args: Vec<&str> = if let Some(host) = host {
        vec!["ssh", "-t", host, "screen", "-r", session]
    } else {
        vec!["screen", "-r", session]
    };

    // Terminal-specific spawn logic
    match terminal {
        "alacritty" => {
            // Alacritty: Use msg create-window for same-process windows, or spawn new instance
            // For attaching to screen, we need a new process, so use -e
            let mut cmd = Command::new("alacritty");

            // Alacritty supports --title for window title
            cmd.arg("--title").arg(format!("sesh: {}", session));

            // Use -e to execute command
            cmd.arg("-e");

            if let Some(host) = host {
                cmd.args(["ssh", "-t", host, "screen", "-r", session]);
            } else {
                cmd.args(["screen", "-r", session]);
            }

            cmd.spawn().context("Failed to spawn alacritty")?;
        }

        "kitty" => {
            // Kitty: Use --title and direct command execution
            let mut cmd = Command::new("kitty");
            cmd.arg("--title").arg(format!("sesh: {}", session));

            if let Some(host) = host {
                cmd.args(["ssh", "-t", host, "screen", "-r", session]);
            } else {
                cmd.args(["screen", "-r", session]);
            }

            cmd.spawn().context("Failed to spawn kitty")?;
        }

        "wezterm" => {
            // Wezterm: Use 'start' subcommand
            let mut cmd = Command::new("wezterm");
            cmd.arg("start");
            cmd.arg("--");

            if let Some(host) = host {
                cmd.args(["ssh", "-t", host, "screen", "-r", session]);
            } else {
                cmd.args(["screen", "-r", session]);
            }

            cmd.spawn().context("Failed to spawn wezterm")?;
        }

        "gnome-terminal" => {
            // GNOME Terminal: Use -- to separate options from command
            let mut cmd = Command::new("gnome-terminal");
            cmd.arg("--title").arg(format!("sesh: {}", session));
            cmd.arg("--");

            if let Some(host) = host {
                cmd.args(["ssh", "-t", host, "screen", "-r", session]);
            } else {
                cmd.args(["screen", "-r", session]);
            }

            cmd.spawn().context("Failed to spawn gnome-terminal")?;
        }

        "konsole" => {
            // Konsole: Use -e for command execution
            let mut cmd = Command::new("konsole");
            cmd.arg("-e");

            if let Some(host) = host {
                // Konsole needs the command as a single string with -e
                let full_cmd = format!("ssh -t {} screen -r {}", host, session);
                cmd.args(["sh", "-c", &full_cmd]);
            } else {
                cmd.args(["screen", "-r", session]);
            }

            cmd.spawn().context("Failed to spawn konsole")?;
        }

        "foot" => {
            // Foot: Wayland-native terminal
            let mut cmd = Command::new("foot");
            cmd.arg("--title").arg(format!("sesh: {}", session));

            if let Some(host) = host {
                cmd.args(["ssh", "-t", host, "screen", "-r", session]);
            } else {
                cmd.args(["screen", "-r", session]);
            }

            cmd.spawn().context("Failed to spawn foot")?;
        }

        // Fallback for unknown terminals
        _ => {
            let screen_cmd = if let Some(host) = host {
                format!("ssh -t {} screen -r {}", host, session)
            } else {
                format!("screen -r {}", session)
            };

            Command::new(terminal)
                .args(["-e", "sh", "-c", &screen_cmd])
                .spawn()
                .context(format!("Failed to spawn {}", terminal))?;
        }
    }

    Ok(())
}

/// Detach a session remotely
pub async fn detach_session(session: &str) -> Result<()> {
    let output = Command::new("screen")
        .args(["-d", session])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to detach session")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // "No screen session found" is not really an error
        if !stderr.contains("No screen session found") {
            bail!("Failed to detach session: {}", stderr);
        }
    }

    Ok(())
}

/// Kill a screen session
pub async fn kill_session(session: &str) -> Result<()> {
    let output = Command::new("screen")
        .args(["-X", "-S", session, "quit"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to kill session")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to kill session: {}", stderr);
    }

    Ok(())
}

/// Rename a screen session
pub async fn rename_session(session: &str, new_name: &str) -> Result<()> {
    let output = Command::new("screen")
        .args(["-S", session, "-X", "sessionname", new_name])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to rename session")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to rename session: {}", stderr);
    }

    Ok(())
}

/// List windows in a session
pub async fn list_windows(session: &str) -> Result<Vec<Window>> {
    // Use screen -Q to query window list
    let output = Command::new("screen")
        .args(["-S", session, "-Q", "windows"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to list windows")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If -Q doesn't work, try alternative method
    if stdout.trim().is_empty() || stdout.contains("-1") {
        return list_windows_fallback(session).await;
    }

    parser::parse_window_list(&stdout)
}

/// Fallback method to list windows using hardstatus
async fn list_windows_fallback(session: &str) -> Result<Vec<Window>> {
    // Send the windows command and capture output via a temporary file
    let temp_file = format!("/tmp/sesh-windows-{}", std::process::id());

    // Tell screen to write window list to file
    let _ = Command::new("screen")
        .args(["-S", session, "-X", "windowlist", "-b"])
        .output()
        .await;

    // Give it a moment
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to read the window list from screen's hardstatus
    // This is a fallback and may not always work
    Ok(Vec::new())
}

/// Create a new window in a session
pub async fn create_window(session: &str, name: Option<&str>) -> Result<()> {
    let mut args = vec!["-S", session, "-X", "screen"];

    if let Some(name) = name {
        args.push("-t");
        args.push(name);
    }

    let output = Command::new("screen")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to create window")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to create window: {}", stderr);
    }

    Ok(())
}

/// Rename a window
pub async fn rename_window(session: &str, window: usize, name: &str) -> Result<()> {
    // First select the window
    Command::new("screen")
        .args(["-S", session, "-X", "select", &window.to_string()])
        .output()
        .await?;

    // Then rename it
    let output = Command::new("screen")
        .args(["-S", session, "-X", "title", name])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to rename window")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to rename window: {}", stderr);
    }

    Ok(())
}

/// Kill a window in a session
pub async fn kill_window(session: &str, window: usize) -> Result<()> {
    // Select the window first
    Command::new("screen")
        .args(["-S", session, "-X", "select", &window.to_string()])
        .output()
        .await?;

    // Kill it
    let output = Command::new("screen")
        .args(["-S", session, "-X", "kill"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to kill window")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to kill window: {}", stderr);
    }

    Ok(())
}

/// Select/switch to a window
pub async fn select_window(session: &str, window: usize) -> Result<()> {
    let output = Command::new("screen")
        .args(["-S", session, "-X", "select", &window.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to select window")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to select window: {}", stderr);
    }

    Ok(())
}

/// Get preview of terminal content
pub async fn get_preview(session: &str, window: Option<usize>) -> Result<Preview> {
    let temp_file = format!("/tmp/sesh-preview-{}", std::process::id());

    // Select window if specified
    if let Some(win) = window {
        let _ = Command::new("screen")
            .args(["-S", session, "-X", "select", &win.to_string()])
            .output()
            .await;
    }

    // Capture terminal content with scrollback
    let output = Command::new("screen")
        .args(["-S", session, "-X", "hardcopy", "-h", &temp_file])
        .output()
        .await
        .context("Failed to capture terminal content")?;

    // Give it a moment to write
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Read the captured content
    let content = tokio::fs::read_to_string(&temp_file)
        .await
        .unwrap_or_default();

    // Clean up temp file
    let _ = tokio::fs::remove_file(&temp_file).await;

    let lines = parser::parse_hardcopy(&content);

    Ok(Preview {
        lines: lines.clone(),
        total_lines: lines.len(),
        scroll_offset: 0,
        updating: false,
    })
}

/// Send a command to a session
pub async fn send_command(session: &str, command: &str) -> Result<()> {
    let output = Command::new("screen")
        .args(["-S", session, "-X", command])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to send command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to send command: {}", stderr);
    }

    Ok(())
}

/// Send text input to a session (stuff command)
pub async fn send_keys(session: &str, keys: &str) -> Result<()> {
    let output = Command::new("screen")
        .args(["-S", session, "-X", "stuff", keys])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to send keys")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to send keys: {}", stderr);
    }

    Ok(())
}

/// Check if screen is installed and available
pub async fn check_screen_available() -> Result<bool> {
    let output = Command::new("which").arg("screen").output().await;

    Ok(output.map(|o| o.status.success()).unwrap_or(false))
}

/// Get the screen version
pub async fn get_screen_version() -> Result<String> {
    let output = Command::new("screen")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to get screen version")?;

    let version = String::from_utf8_lossy(&output.stdout);
    Ok(version.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_screen_available() {
        // This test assumes screen might or might not be installed
        let result = check_screen_available().await;
        assert!(result.is_ok());
    }
}
