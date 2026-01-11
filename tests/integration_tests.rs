//! Integration tests for sesh
//!
//! Tests the interaction between modules and async operations.

use sesh::config::Settings;
use sesh::screen::types::{Session, SessionStatus, Window, WindowActivity};

mod screen_operations {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "12345.test".to_string(),
            "test".to_string(),
            12345,
            SessionStatus::Detached,
        );

        assert_eq!(session.name, "test");
        assert_eq!(session.pid, 12345);
        assert!(session.is_local());
        assert!(!session.is_attached());
    }

    #[test]
    fn test_session_remote() {
        let mut session = Session::new(
            "12345.remote".to_string(),
            "remote".to_string(),
            12345,
            SessionStatus::Attached,
        );
        session.host = Some("server.example.com".to_string());

        assert!(!session.is_local());
        assert!(session.is_attached());
        assert_eq!(session.display_name(), "remote@server.example.com");
    }

    #[test]
    fn test_window_creation() {
        let window = Window::new(0, "bash".to_string());

        assert_eq!(window.number, 0);
        assert_eq!(window.name, "bash");
        assert!(!window.active);
        assert_eq!(window.activity, WindowActivity::Idle);
    }

    #[test]
    fn test_session_status_variants() {
        let statuses = vec![
            (SessionStatus::Detached, false),
            (SessionStatus::Attached, true),
            (SessionStatus::Multi, true),
            (SessionStatus::Unknown, false),
        ];

        for (status, expected_attached) in statuses {
            let session = Session::new("1.test".to_string(), "test".to_string(), 1, status);
            assert_eq!(
                session.is_attached(),
                expected_attached,
                "Status {:?} should return {} for is_attached",
                status,
                expected_attached
            );
        }
    }
}

mod config_operations {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_settings_roundtrip() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");
        let path_str = config_path.to_string_lossy().to_string();

        // Create custom settings
        let mut settings = Settings::default();
        settings.ui.theme = "test_theme".to_string();
        settings.ui.preview_lines = 15;
        settings.screen.attach_mode = "spawn".to_string();
        settings.screen.spawn_terminal = Some("alacritty".to_string());
        settings.integrations.git_status = false;

        // Save
        settings.save(Some(&path_str)).unwrap();

        // Load and verify
        let loaded = Settings::load(Some(&path_str)).unwrap();
        assert_eq!(loaded.ui.theme, "test_theme");
        assert_eq!(loaded.ui.preview_lines, 15);
        assert_eq!(loaded.screen.attach_mode, "spawn");
        assert_eq!(loaded.screen.spawn_terminal, Some("alacritty".to_string()));
        assert!(!loaded.integrations.git_status);
    }

    #[test]
    fn test_partial_config_merge() {
        // Test that partial config merges with defaults
        let partial_toml = r#"
[ui]
theme = "custom"
"#;
        let settings: Settings = toml::from_str(partial_toml).unwrap();

        // Custom value
        assert_eq!(settings.ui.theme, "custom");
        // Default values preserved
        assert!(settings.ui.show_preview);
        assert!(settings.navigation.vim_keys);
    }
}

mod async_operations {
    use super::*;

    #[tokio::test]
    async fn test_async_settings_load() {
        // Test that settings can be loaded in async context
        let settings = tokio::task::spawn_blocking(Settings::default).await;
        assert!(settings.is_ok());
        let settings = settings.unwrap();
        assert!(settings.navigation.vim_keys);
    }
}

mod shell_integration {
    use sesh::integrations::shell;

    #[test]
    fn test_bash_integration_not_empty() {
        let script = shell::bash_integration();
        assert!(!script.is_empty());
        assert!(script.contains("sesh"));
        assert!(script.contains("_sesh_completions"));
    }

    #[test]
    fn test_zsh_integration_not_empty() {
        let script = shell::zsh_integration();
        assert!(!script.is_empty());
        assert!(script.contains("sesh"));
        assert!(script.contains("compdef"));
    }

    #[test]
    fn test_fish_integration_not_empty() {
        let script = shell::fish_integration();
        assert!(!script.is_empty());
        assert!(script.contains("sesh"));
        assert!(script.contains("abbr"));
        assert!(script.contains("complete"));
    }

    #[test]
    fn test_fish_integration_has_abbreviations() {
        let script = shell::fish_integration();
        // Check for fish abbreviations
        assert!(script.contains("abbr --add ss"));
        assert!(script.contains("abbr --add sl"));
        assert!(script.contains("abbr --add sn"));
    }

    #[test]
    fn test_fish_integration_has_functions() {
        let script = shell::fish_integration();
        // Check for fish functions
        assert!(script.contains("function sesh-attach"));
        assert!(script.contains("function sesh-new"));
        assert!(script.contains("function sesh-here"));
    }

    #[test]
    fn test_fish_integration_has_keybinding() {
        let script = shell::fish_integration();
        // Check for Ctrl+S keybinding
        assert!(script.contains("bind \\cs"));
    }
}

mod preview_tests {
    use sesh::screen::types::Preview;

    #[test]
    fn test_preview_empty() {
        let preview = Preview::new();
        assert!(preview.lines.is_empty());
        assert_eq!(preview.total_lines, 0);
        assert_eq!(preview.scroll_offset, 0);
    }

    #[test]
    fn test_preview_visible_lines() {
        let preview = Preview {
            lines: vec![
                "Line 1".to_string(),
                "Line 2".to_string(),
                "Line 3".to_string(),
                "Line 4".to_string(),
                "Line 5".to_string(),
            ],
            total_lines: 5,
            scroll_offset: 0,
            updating: false,
        };

        let visible = preview.visible_lines(3);
        assert_eq!(visible.len(), 3);
        assert_eq!(visible[0], (1, "Line 1"));
        assert_eq!(visible[1], (2, "Line 2"));
        assert_eq!(visible[2], (3, "Line 3"));
    }

    #[test]
    fn test_preview_scroll() {
        let mut preview = Preview {
            lines: (1..=20).map(|i| format!("Line {}", i)).collect(),
            total_lines: 20,
            scroll_offset: 0,
            updating: false,
        };

        // Scroll down
        preview.scroll_down(5, 10);
        assert_eq!(preview.scroll_offset, 5);

        // Scroll up
        preview.scroll_up(2);
        assert_eq!(preview.scroll_offset, 3);

        // Check visible lines after scroll
        let visible = preview.visible_lines(5);
        assert_eq!(visible[0], (4, "Line 4"));
    }
}
