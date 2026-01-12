//! Settings TUI
//!
//! Provides a full-featured settings interface for configuring sesh.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::config::Settings;

use super::theme::Theme;

/// Settings categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    General,
    Screen,
    Navigation,
    Hosts,
    Keybindings,
}

impl SettingsCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::General,
            Self::Screen,
            Self::Navigation,
            Self::Hosts,
            Self::Keybindings,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Screen => "Screen",
            Self::Navigation => "Navigation",
            Self::Hosts => "SSH Hosts",
            Self::Keybindings => "Keybindings",
        }
    }
}

/// A single editable setting
#[derive(Debug, Clone)]
pub struct SettingItem {
    pub key: String,
    pub label: String,
    pub value: SettingValue,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum SettingValue {
    Bool(bool),
    String(String),
    Number(i64),
    Choice(String, Vec<String>), // current, options
}

impl SettingValue {
    pub fn display(&self) -> String {
        match self {
            Self::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
            Self::String(s) => {
                if s.is_empty() {
                    "(not set)".to_string()
                } else {
                    s.clone()
                }
            }
            Self::Number(n) => n.to_string(),
            Self::Choice(current, _) => current.clone(),
        }
    }

    pub fn toggle(&mut self) {
        match self {
            Self::Bool(b) => *b = !*b,
            Self::Choice(current, options) => {
                if let Some(idx) = options.iter().position(|o| o == current) {
                    let next = (idx + 1) % options.len();
                    *current = options[next].clone();
                }
            }
            _ => {}
        }
    }
}

/// Get settings items for a category
pub fn get_settings_for_category(
    settings: &Settings,
    category: SettingsCategory,
) -> Vec<SettingItem> {
    match category {
        SettingsCategory::General => vec![
            SettingItem {
                key: "ui.theme".to_string(),
                label: "Theme".to_string(),
                value: SettingValue::Choice(
                    settings.ui.theme.clone(),
                    vec!["dark".to_string(), "light".to_string()],
                ),
                description: "Color theme for the interface".to_string(),
            },
            SettingItem {
                key: "ui.show_preview".to_string(),
                label: "Show Preview".to_string(),
                value: SettingValue::Bool(settings.ui.show_preview),
                description: "Display terminal preview panel".to_string(),
            },
            SettingItem {
                key: "ui.preview_lines".to_string(),
                label: "Preview Lines".to_string(),
                value: SettingValue::Number(settings.ui.preview_lines as i64),
                description: "Number of lines in preview panel".to_string(),
            },
            SettingItem {
                key: "ui.refresh_interval_ms".to_string(),
                label: "Refresh Interval".to_string(),
                value: SettingValue::Number(settings.ui.refresh_interval_ms as i64),
                description: "Auto-refresh interval in milliseconds".to_string(),
            },
        ],
        SettingsCategory::Screen => vec![
            SettingItem {
                key: "screen.attach_mode".to_string(),
                label: "Attach Mode".to_string(),
                value: SettingValue::Choice(
                    settings.screen.attach_mode.clone(),
                    vec!["exec".to_string(), "spawn".to_string()],
                ),
                description: "exec: replace process, spawn: new terminal".to_string(),
            },
            SettingItem {
                key: "screen.spawn_terminal".to_string(),
                label: "Terminal".to_string(),
                value: SettingValue::Choice(
                    settings.screen.spawn_terminal.clone().unwrap_or_default(),
                    vec![
                        "alacritty".to_string(),
                        "kitty".to_string(),
                        "wezterm".to_string(),
                        "foot".to_string(),
                        "gnome-terminal".to_string(),
                        "konsole".to_string(),
                    ],
                ),
                description: "Terminal emulator for spawn mode".to_string(),
            },
            SettingItem {
                key: "screen.default_shell".to_string(),
                label: "Default Shell".to_string(),
                value: SettingValue::String(
                    settings.screen.default_shell.clone().unwrap_or_default(),
                ),
                description: "Shell for new sessions (empty = system default)".to_string(),
            },
            SettingItem {
                key: "screen.socket_dir".to_string(),
                label: "Socket Directory".to_string(),
                value: SettingValue::String(settings.screen.socket_dir.clone().unwrap_or_default()),
                description: "Custom screen socket directory".to_string(),
            },
        ],
        SettingsCategory::Navigation => vec![
            SettingItem {
                key: "navigation.vim_keys".to_string(),
                label: "Vim Keys".to_string(),
                value: SettingValue::Bool(settings.navigation.vim_keys),
                description: "Enable hjkl navigation".to_string(),
            },
            SettingItem {
                key: "navigation.arrow_keys".to_string(),
                label: "Arrow Keys".to_string(),
                value: SettingValue::Bool(settings.navigation.arrow_keys),
                description: "Enable arrow key navigation".to_string(),
            },
            SettingItem {
                key: "navigation.mouse_enabled".to_string(),
                label: "Mouse Support".to_string(),
                value: SettingValue::Bool(settings.navigation.mouse_enabled),
                description: "Enable mouse click support".to_string(),
            },
        ],
        SettingsCategory::Hosts => vec![], // Handled separately
        SettingsCategory::Keybindings => vec![
            SettingItem {
                key: "keybindings.quit".to_string(),
                label: "Quit".to_string(),
                value: SettingValue::String(settings.keybindings.quit.join(", ")),
                description: "Keys to quit sesh".to_string(),
            },
            SettingItem {
                key: "keybindings.select".to_string(),
                label: "Select/Enter".to_string(),
                value: SettingValue::String(settings.keybindings.select.join(", ")),
                description: "Keys to select/attach".to_string(),
            },
            SettingItem {
                key: "keybindings.new_session".to_string(),
                label: "New Session".to_string(),
                value: SettingValue::String(settings.keybindings.new_session.join(", ")),
                description: "Keys to create new session".to_string(),
            },
            SettingItem {
                key: "keybindings.kill_session".to_string(),
                label: "Kill Session".to_string(),
                value: SettingValue::String(settings.keybindings.kill_session.join(", ")),
                description: "Keys to kill session".to_string(),
            },
        ],
    }
}

/// Draw the settings screen
pub fn draw(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    // Clear background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_focused())
        .title(" Settings [S] - Press Esc to close ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout: categories on left, settings on right
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(40)])
        .split(inner);

    draw_categories(frame, app, theme, chunks[0]);

    let category = SettingsCategory::all()[app.settings_category_index];
    if category == SettingsCategory::Hosts {
        draw_hosts(frame, app, theme, chunks[1]);
    } else {
        draw_settings_list(frame, app, theme, chunks[1], category);
    }
}

fn draw_categories(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let categories = SettingsCategory::all();
    let items: Vec<ListItem> = categories
        .iter()
        .map(|cat| ListItem::new(Line::from(vec![Span::raw("  "), Span::raw(cat.name())])))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(theme.border()),
        )
        .highlight_style(theme.selected())
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.settings_category_index));

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_settings_list(
    frame: &mut Frame,
    app: &App,
    theme: &Theme,
    area: Rect,
    category: SettingsCategory,
) {
    let settings_items = get_settings_for_category(&app.config, category);

    if settings_items.is_empty() {
        let msg = Paragraph::new("No settings in this category").style(theme.muted());
        frame.render_widget(msg, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(4)])
        .split(area);

    // Settings list
    let items: Vec<ListItem> = settings_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == app.settings_item_index;
            let value_style = if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                theme.accent()
            };

            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{:<20}", item.label), theme.normal()),
                Span::styled(item.value.display(), value_style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(theme.selected())
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.settings_item_index));

    frame.render_stateful_widget(list, chunks[0], &mut state);

    // Description at bottom
    if let Some(item) = settings_items.get(app.settings_item_index) {
        let help_text = match &item.value {
            SettingValue::Bool(_) => "Press Enter or Space to toggle",
            SettingValue::Choice(_, _) => "Press Enter or Space to cycle options",
            SettingValue::String(_) => "Press Enter to edit",
            SettingValue::Number(_) => "Press Enter to edit, +/- to adjust",
        };

        let desc = Paragraph::new(vec![
            Line::from(vec![Span::styled(&item.description, theme.muted())]),
            Line::from(""),
            Line::from(vec![Span::styled(help_text, theme.key())]),
        ])
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(theme.border()),
        )
        .wrap(Wrap { trim: true });

        frame.render_widget(desc, chunks[1]);
    }
}

fn draw_hosts(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(6)])
        .split(area);

    if app.config.hosts.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "  No SSH hosts configured",
                theme.muted(),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press ", theme.muted()),
                Span::styled("[a]", theme.key()),
                Span::styled(" to add a new host", theme.muted()),
            ]),
        ]);
        frame.render_widget(msg, chunks[0]);
    } else {
        let items: Vec<ListItem> = app
            .config
            .hosts
            .iter()
            .enumerate()
            .map(|(i, host)| {
                let conn = format!(
                    "{}@{}:{}",
                    host.user.as_deref().unwrap_or("(default)"),
                    host.hostname,
                    host.port.unwrap_or(22)
                );
                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("{:<15}", host.name), theme.normal()),
                    Span::styled(conn, theme.muted()),
                ]))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(theme.selected())
            .highlight_symbol("> ");

        let mut state = ListState::default();
        state.select(Some(
            app.settings_item_index
                .min(app.config.hosts.len().saturating_sub(1)),
        ));

        frame.render_stateful_widget(list, chunks[0], &mut state);
    }

    // Help text
    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("[a]", theme.key()),
            Span::styled(" Add host  ", theme.muted()),
            Span::styled("[e]", theme.key()),
            Span::styled(" Edit  ", theme.muted()),
            Span::styled("[d]", theme.key()),
            Span::styled(" Delete  ", theme.muted()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Hosts are used for SSH session management",
            theme.muted(),
        )]),
    ])
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(theme.border()),
    );

    frame.render_widget(help, chunks[1]);
}

/// Apply a setting change to the config
pub fn apply_setting(settings: &mut Settings, key: &str, value: &SettingValue) {
    match key {
        "ui.theme" => {
            if let SettingValue::Choice(v, _) | SettingValue::String(v) = value {
                settings.ui.theme = v.clone();
            }
        }
        "ui.show_preview" => {
            if let SettingValue::Bool(v) = value {
                settings.ui.show_preview = *v;
            }
        }
        "ui.preview_lines" => {
            if let SettingValue::Number(v) = value {
                settings.ui.preview_lines = *v as usize;
            }
        }
        "ui.refresh_interval_ms" => {
            if let SettingValue::Number(v) = value {
                settings.ui.refresh_interval_ms = *v as u32;
            }
        }
        "screen.attach_mode" => {
            if let SettingValue::Choice(v, _) | SettingValue::String(v) = value {
                settings.screen.attach_mode = v.clone();
            }
        }
        "screen.spawn_terminal" => {
            if let SettingValue::Choice(v, _) | SettingValue::String(v) = value {
                settings.screen.spawn_terminal = if v.is_empty() { None } else { Some(v.clone()) };
            }
        }
        "screen.default_shell" => {
            if let SettingValue::String(v) = value {
                settings.screen.default_shell = if v.is_empty() { None } else { Some(v.clone()) };
            }
        }
        "screen.socket_dir" => {
            if let SettingValue::String(v) = value {
                settings.screen.socket_dir = if v.is_empty() { None } else { Some(v.clone()) };
            }
        }
        "navigation.vim_keys" => {
            if let SettingValue::Bool(v) = value {
                settings.navigation.vim_keys = *v;
            }
        }
        "navigation.arrow_keys" => {
            if let SettingValue::Bool(v) = value {
                settings.navigation.arrow_keys = *v;
            }
        }
        "navigation.mouse_enabled" => {
            if let SettingValue::Bool(v) = value {
                settings.navigation.mouse_enabled = *v;
            }
        }
        _ => {}
    }
}
