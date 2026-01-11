//! Settings configuration
//!
//! TOML-based settings for sesh.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::hosts::HostConfig;

/// Main settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Settings {
    /// UI settings
    pub ui: UiSettings,

    /// Navigation settings
    pub navigation: NavigationSettings,

    /// Screen settings
    pub screen: ScreenSettings,

    /// Integration settings
    pub integrations: IntegrationSettings,

    /// Keybindings
    pub keybindings: KeyBindings,

    /// Configured SSH hosts
    #[serde(default)]
    pub hosts: Vec<HostConfig>,
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    /// Theme name
    pub theme: String,
    /// Show preview panel
    pub show_preview: bool,
    /// Number of preview lines
    pub preview_lines: usize,
    /// Refresh interval in milliseconds
    pub refresh_interval_ms: u32,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            show_preview: true,
            preview_lines: 8,
            refresh_interval_ms: 1000,
        }
    }
}

/// Navigation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NavigationSettings {
    /// Enable vim-style keys
    pub vim_keys: bool,
    /// Enable arrow keys
    pub arrow_keys: bool,
    /// Enable mouse support
    pub mouse_enabled: bool,
}

impl Default for NavigationSettings {
    fn default() -> Self {
        Self {
            vim_keys: true,
            arrow_keys: true,
            mouse_enabled: true,
        }
    }
}

/// Screen settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScreenSettings {
    /// Screen socket directory
    pub socket_dir: Option<String>,
    /// Default shell for new sessions
    pub default_shell: Option<String>,
    /// Attach mode: "exec" or "spawn"
    pub attach_mode: String,
    /// Terminal to use for spawn mode
    pub spawn_terminal: Option<String>,
}

impl Default for ScreenSettings {
    fn default() -> Self {
        Self {
            socket_dir: None,
            default_shell: None,
            attach_mode: "exec".to_string(),
            spawn_terminal: Some("xterm".to_string()),
        }
    }
}

/// Integration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IntegrationSettings {
    /// Show git status
    pub git_status: bool,
    /// Enable fzf integration
    pub fzf_enabled: bool,
    /// Enable shell hooks
    pub shell_hooks: bool,
}

impl Default for IntegrationSettings {
    fn default() -> Self {
        Self {
            git_status: true,
            fzf_enabled: true,
            shell_hooks: false,
        }
    }
}

/// Key bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyBindings {
    pub quit: Vec<String>,
    pub select: Vec<String>,
    pub back: Vec<String>,
    pub up: Vec<String>,
    pub down: Vec<String>,
    pub top: Vec<String>,
    pub bottom: Vec<String>,
    pub search: Vec<String>,
    pub new_session: Vec<String>,
    pub kill_session: Vec<String>,
    pub detach: Vec<String>,
    pub windows: Vec<String>,
    pub templates: Vec<String>,
    pub refresh: Vec<String>,
    pub help: Vec<String>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: vec!["q".to_string(), "Ctrl-c".to_string()],
            select: vec!["Enter".to_string(), "l".to_string()],
            back: vec!["Esc".to_string(), "h".to_string()],
            up: vec!["k".to_string(), "Up".to_string()],
            down: vec!["j".to_string(), "Down".to_string()],
            top: vec!["g".to_string(), "Home".to_string()],
            bottom: vec!["G".to_string(), "End".to_string()],
            search: vec!["/".to_string()],
            new_session: vec!["n".to_string()],
            kill_session: vec!["x".to_string()],
            detach: vec!["d".to_string()],
            windows: vec!["w".to_string()],
            templates: vec!["t".to_string()],
            refresh: vec!["r".to_string()],
            help: vec!["?".to_string()],
        }
    }
}

impl Settings {
    /// Load settings from file
    pub fn load(custom_path: Option<&str>) -> Result<Self> {
        let path = if let Some(p) = custom_path {
            PathBuf::from(shellexpand::tilde(p).as_ref())
        } else {
            Self::default_config_path()
        };

        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .context(format!("Failed to read config file: {:?}", path))?;
            let settings: Settings =
                toml::from_str(&content).context("Failed to parse config file")?;
            Ok(settings)
        } else {
            // Return defaults if no config exists
            Ok(Self::default())
        }
    }

    /// Get default config file path
    pub fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .map(|d| d.join("sesh").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("~/.config/sesh/config.toml"))
    }

    /// Get templates directory path
    pub fn templates_dir(&self) -> PathBuf {
        dirs::config_dir()
            .map(|d| d.join("sesh").join("templates"))
            .unwrap_or_else(|| PathBuf::from("~/.config/sesh/templates"))
    }

    /// Save settings to file
    pub fn save(&self, path: Option<&str>) -> Result<()> {
        let path = if let Some(p) = path {
            PathBuf::from(shellexpand::tilde(p).as_ref())
        } else {
            Self::default_config_path()
        };

        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    /// Generate default config file
    pub fn generate_default_config() -> String {
        let settings = Self::default();
        toml::to_string_pretty(&settings).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;
    use tempfile::tempdir;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(settings.ui.show_preview);
        assert!(settings.navigation.vim_keys);
        assert_eq!(settings.screen.attach_mode, "exec");
    }

    #[test]
    fn test_default_ui_settings() {
        let ui = UiSettings::default();
        assert_eq!(ui.theme, "dark");
        assert!(ui.show_preview);
        assert_eq!(ui.preview_lines, 8);
        assert_eq!(ui.refresh_interval_ms, 1000);
    }

    #[test]
    fn test_default_navigation_settings() {
        let nav = NavigationSettings::default();
        assert!(nav.vim_keys);
        assert!(nav.arrow_keys);
        assert!(nav.mouse_enabled);
    }

    #[test]
    fn test_default_screen_settings() {
        let screen = ScreenSettings::default();
        assert!(screen.socket_dir.is_none());
        assert!(screen.default_shell.is_none());
        assert_eq!(screen.attach_mode, "exec");
        assert!(screen.spawn_terminal.is_some());
    }

    #[test]
    fn test_default_integration_settings() {
        let integrations = IntegrationSettings::default();
        assert!(integrations.git_status);
        assert!(integrations.fzf_enabled);
        assert!(!integrations.shell_hooks);
    }

    #[test]
    fn test_default_keybindings() {
        let keys = KeyBindings::default();
        assert!(keys.quit.contains(&"q".to_string()));
        assert!(keys.up.contains(&"k".to_string()));
        assert!(keys.down.contains(&"j".to_string()));
        assert!(keys.search.contains(&"/".to_string()));
    }

    #[test]
    fn test_serialize_settings() {
        let settings = Settings::default();
        let toml = toml::to_string_pretty(&settings);
        assert!(toml.is_ok());
    }

    #[test]
    fn test_deserialize_settings() {
        let toml = r#"
[ui]
theme = "dark"
show_preview = true

[navigation]
vim_keys = true
"#;
        let settings: Result<Settings, _> = toml::from_str(toml);
        assert!(settings.is_ok());
    }

    #[test]
    fn test_deserialize_full_settings() {
        let toml = r#"
[ui]
theme = "light"
show_preview = false
preview_lines = 12
refresh_interval_ms = 500

[navigation]
vim_keys = false
arrow_keys = true
mouse_enabled = false

[screen]
socket_dir = "/custom/screen"
default_shell = "/bin/fish"
attach_mode = "spawn"
spawn_terminal = "alacritty"

[integrations]
git_status = false
fzf_enabled = false
shell_hooks = true

[keybindings]
quit = ["q", "Ctrl-q"]
"#;
        let settings: Settings = toml::from_str(toml).unwrap();
        assert_eq!(settings.ui.theme, "light");
        assert!(!settings.ui.show_preview);
        assert_eq!(settings.ui.preview_lines, 12);
        assert!(!settings.navigation.vim_keys);
        assert_eq!(settings.screen.attach_mode, "spawn");
        assert_eq!(
            settings.screen.spawn_terminal,
            Some("alacritty".to_string())
        );
        assert!(settings.integrations.shell_hooks);
    }

    #[test]
    fn test_save_and_load_settings() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let path_str = config_path.to_string_lossy().to_string();

        let mut settings = Settings::default();
        settings.ui.theme = "custom".to_string();
        settings.ui.preview_lines = 20;

        settings.save(Some(&path_str)).unwrap();
        assert!(config_path.exists());

        let loaded = Settings::load(Some(&path_str)).unwrap();
        assert_eq!(loaded.ui.theme, "custom");
        assert_eq!(loaded.ui.preview_lines, 20);
    }

    #[test]
    fn test_load_nonexistent_returns_defaults() {
        let settings = Settings::load(Some("/nonexistent/path/config.toml")).unwrap();
        assert_eq!(settings.ui.theme, "dark"); // Default value
    }

    #[test]
    fn test_generate_default_config() {
        let config = Settings::generate_default_config();
        assert!(!config.is_empty());
        assert!(config.contains("[ui]"));
        assert!(config.contains("[navigation]"));
        assert!(config.contains("[screen]"));
    }

    #[test]
    fn test_snapshot_default_keybindings() {
        let keys = KeyBindings::default();
        assert_yaml_snapshot!(keys);
    }
}
