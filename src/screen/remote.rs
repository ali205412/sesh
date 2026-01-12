//! Remote screen command execution via SSH
//!
//! Handles all screen operations for remote sessions over SSH.

use anyhow::{bail, Context, Result};
use std::process::Stdio;
use tokio::process::Command;

use super::parser;
use super::types::Session;
use crate::config::Settings;

/// Get SSH connection arguments for a host
fn get_ssh_args(config: &Settings, host_name: &str) -> Result<(String, Vec<String>)> {
    let host_config = config
        .hosts
        .iter()
        .find(|h| h.name == host_name)
        .ok_or_else(|| anyhow::anyhow!("Host '{}' not found in configuration", host_name))?;

    let mut args = Vec::new();

    // Add identity file if specified
    if let Some(ref key) = host_config.identity_file {
        let expanded = shellexpand::tilde(key);
        args.push("-i".to_string());
        args.push(expanded.into_owned());
    }

    // Add port if non-standard
    if let Some(port) = host_config.port {
        if port != 22 {
            args.push("-p".to_string());
            args.push(port.to_string());
        }
    }

    // Build connection string
    let connection = if let Some(ref user) = host_config.user {
        format!("{}@{}", user, host_config.hostname)
    } else {
        host_config.hostname.clone()
    };

    Ok((connection, args))
}

/// Run an SSH command and return output
async fn run_ssh_command(
    config: &Settings,
    host_name: &str,
    remote_cmd: &[&str],
) -> Result<String> {
    let (connection, mut args) = get_ssh_args(config, host_name)?;

    // Add SSH options for faster connection
    args.push("-o".to_string());
    args.push("ConnectTimeout=3".to_string());
    args.push("-o".to_string());
    args.push("BatchMode=yes".to_string());
    args.push("-o".to_string());
    args.push("StrictHostKeyChecking=accept-new".to_string());

    // Add connection
    args.push(connection);

    // Add remote command
    args.extend(remote_cmd.iter().map(|s| s.to_string()));

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        Command::new("ssh")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    )
    .await
    .context("SSH command timed out")?
    .context("Failed to run SSH command")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // SSH might fail but screen -ls returns exit 1 normally
    if !output.status.success() && !stdout.contains("screen") && !stderr.contains("screen") {
        bail!("SSH command failed: {}", stderr);
    }

    Ok(format!("{}{}", stdout, stderr))
}

/// List screen sessions on a remote host
pub async fn list_sessions(config: &Settings, host_name: &str) -> Result<Vec<Session>> {
    let output = run_ssh_command(config, host_name, &["screen", "-ls"]).await?;

    if parser::is_no_sessions(&output) {
        return Ok(Vec::new());
    }

    parser::parse_session_list(&output, Some(host_name))
}

/// Create a new session on a remote host
pub async fn create_session(
    config: &Settings,
    host_name: &str,
    name: &str,
    dir: Option<&str>,
) -> Result<()> {
    let cmd = ["screen", "-dmS", name];

    // If directory specified, cd to it first
    let full_cmd = if let Some(dir) = dir {
        format!("cd {} && screen -dmS {}", dir, name)
    } else {
        format!("screen -dmS {}", name)
    };

    let output = run_ssh_command(config, host_name, &["sh", "-c", &full_cmd]).await?;

    if output.to_lowercase().contains("error") {
        bail!("Failed to create remote session: {}", output);
    }

    Ok(())
}

/// Detach a session on a remote host
pub async fn detach_session(config: &Settings, host_name: &str, session: &str) -> Result<()> {
    let output = run_ssh_command(config, host_name, &["screen", "-d", session]).await?;

    if output.to_lowercase().contains("error") && !output.contains("No screen session") {
        bail!("Failed to detach remote session: {}", output);
    }

    Ok(())
}

/// Kill a session on a remote host
pub async fn kill_session(config: &Settings, host_name: &str, session: &str) -> Result<()> {
    let output =
        run_ssh_command(config, host_name, &["screen", "-X", "-S", session, "quit"]).await?;

    if output.to_lowercase().contains("error") {
        bail!("Failed to kill remote session: {}", output);
    }

    Ok(())
}

/// List windows in a remote session
pub async fn list_windows(
    config: &Settings,
    host_name: &str,
    session: &str,
) -> Result<Vec<super::types::Window>> {
    let output = run_ssh_command(
        config,
        host_name,
        &["screen", "-S", session, "-Q", "windows"],
    )
    .await?;

    parser::parse_window_list(&output)
}

/// Get preview content from a remote session
pub async fn get_preview(
    config: &Settings,
    host_name: &str,
    session: &str,
    window: Option<usize>,
) -> Result<super::types::Preview> {
    let temp_file = format!("/tmp/sesh-preview-{}", std::process::id());

    // Build the command sequence
    let mut commands = Vec::new();

    // Select window if specified
    if let Some(win) = window {
        commands.push(format!("screen -S {} -X select {}", session, win));
    }

    // Capture content
    commands.push(format!(
        "screen -S {} -X hardcopy -h {}",
        session, temp_file
    ));

    // Wait a bit then cat and cleanup
    commands.push("sleep 0.1".to_string());
    commands.push(format!("cat {} 2>/dev/null", temp_file));
    commands.push(format!("rm -f {}", temp_file));

    let full_cmd = commands.join(" && ");
    let output = run_ssh_command(config, host_name, &["sh", "-c", &full_cmd]).await?;

    let lines = parser::parse_hardcopy(&output);

    Ok(super::types::Preview {
        lines: lines.clone(),
        total_lines: lines.len(),
        scroll_offset: 0,
        updating: false,
    })
}

/// Send a command to a remote session
pub async fn send_command(
    config: &Settings,
    host_name: &str,
    session: &str,
    command: &str,
) -> Result<()> {
    let output =
        run_ssh_command(config, host_name, &["screen", "-S", session, "-X", command]).await?;

    if output.to_lowercase().contains("error") {
        bail!("Failed to send command to remote session: {}", output);
    }

    Ok(())
}

/// Check if a host is reachable via SSH
pub async fn check_host_reachable(config: &Settings, host_name: &str) -> Result<bool> {
    let result = run_ssh_command(config, host_name, &["echo", "ok"]).await;
    Ok(result.map(|o| o.trim() == "ok").unwrap_or(false))
}

/// Get screen version on remote host
pub async fn get_screen_version(config: &Settings, host_name: &str) -> Result<String> {
    let output = run_ssh_command(config, host_name, &["screen", "--version"]).await?;
    Ok(output.trim().to_string())
}

#[cfg(test)]
mod tests {
    // Remote tests require actual SSH setup, so we skip them in unit tests
}
