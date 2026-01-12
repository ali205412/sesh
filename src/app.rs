//! Application state and logic
//!
//! Manages the TUI application state, navigation, and operations.

use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashMap;

use crate::config::{templates::Template, Settings};
use crate::event::{key_to_action, Action, AppEvent, EventConfig, EventHandler, Terminal};
use crate::screen::{self, Preview, Session, Window};
use crate::ui;

/// Current view/mode of the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Main session list view
    Sessions,
    /// Window list for a selected session
    Windows,
    /// Template selector
    Templates,
    /// Help overlay
    Help,
    /// Settings/configuration
    Settings,
}

/// Input mode state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    /// Normal navigation mode
    Normal,
    /// Search/filter mode
    Search,
    /// Text input for creating/renaming
    Input {
        prompt: String,
        purpose: InputPurpose,
    },
    /// Confirmation dialog
    Confirm {
        message: String,
        action: ConfirmAction,
    },
}

/// Purpose of text input
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputPurpose {
    NewSession,
    RenameSession,
    NewWindow,
    RenameWindow,
    TemplateVariable(String),
    AddHostName,
    AddHostHostname,
    AddHostUser,
    AddHostPort,
    AddHostIdentityFile,
}

/// Action to confirm
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    KillSession(String),
    KillWindow(String, usize),
}

/// Application state
pub struct App {
    /// Configuration
    pub config: Settings,

    /// Current view
    pub view: View,

    /// Input mode
    pub input_mode: InputMode,

    /// Text input buffer
    pub input_buffer: String,

    /// Input cursor position
    pub input_cursor: usize,

    /// Search query
    pub search_query: String,

    /// All sessions (grouped by host)
    pub sessions: Vec<Session>,

    /// Filtered sessions (after search)
    pub filtered_sessions: Vec<usize>,

    /// Currently selected host (None = local)
    pub selected_host: Option<String>,

    /// Current session index
    pub session_index: usize,

    /// Currently selected session (for windows view)
    pub selected_session: Option<String>,

    /// Windows for selected session
    pub windows: Vec<Window>,

    /// Current window index
    pub window_index: usize,

    /// Templates
    pub templates: Vec<Template>,

    /// Current template index
    pub template_index: usize,

    /// Preview content
    pub preview: Preview,

    /// Whether preview is enabled
    pub show_preview: bool,

    /// Whether help is shown
    pub show_help: bool,

    /// Status message
    pub status_message: Option<String>,

    /// Error message
    pub error_message: Option<String>,

    /// Whether app should quit
    pub should_quit: bool,

    /// Available hosts (including local)
    pub hosts: Vec<Option<String>>,

    /// Current host index
    pub host_index: usize,

    /// Settings category index
    pub settings_category_index: usize,

    /// Settings item index within category
    pub settings_item_index: usize,

    /// Theme for rendering
    pub theme: crate::ui::theme::Theme,

    /// Fuzzy matcher
    matcher: SkimMatcherV2,

    /// New host being added (name, hostname, user, port, identity_file)
    pub new_host: Option<(String, String, String, String, String)>,
}

impl App {
    /// Create a new application
    pub fn new(config: Settings, initial_host: Option<String>) -> Result<Self> {
        // Build host list: local (None) + configured hosts
        let mut hosts: Vec<Option<String>> = vec![None];
        for host in &config.hosts {
            hosts.push(Some(host.name.clone()));
        }

        let show_preview = config.ui.show_preview;

        let theme = crate::ui::theme::Theme::dark();

        let app = Self {
            config,
            view: View::Sessions,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            input_cursor: 0,
            search_query: String::new(),
            sessions: Vec::new(),
            filtered_sessions: Vec::new(),
            selected_host: initial_host,
            session_index: 0,
            selected_session: None,
            windows: Vec::new(),
            window_index: 0,
            templates: Vec::new(),
            template_index: 0,
            preview: Preview::new(),
            show_preview,
            show_help: false,
            status_message: None,
            error_message: None,
            should_quit: false,
            hosts,
            host_index: 0,
            settings_category_index: 0,
            settings_item_index: 0,
            theme,
            matcher: SkimMatcherV2::default(),
            new_host: None,
        };

        Ok(app)
    }

    /// Run the application main loop
    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = Terminal::new()?;
        let mut events = EventHandler::new(EventConfig {
            tick_rate_ms: self.config.ui.refresh_interval_ms as u64,
            mouse_enabled: self.config.navigation.mouse_enabled,
        });

        // Initial data load
        self.refresh_sessions().await;
        self.load_templates();

        loop {
            // Draw UI
            terminal.backend.draw(|frame| {
                ui::draw(frame, self);
            })?;

            // Handle events
            if let Some(event) = events.next().await {
                match event {
                    AppEvent::Key(key) => {
                        let in_input = matches!(self.input_mode, InputMode::Input { .. });
                        let in_search = matches!(self.input_mode, InputMode::Search);
                        let action = key_to_action(key, in_input, in_search);
                        self.handle_action(action).await;
                    }
                    AppEvent::Mouse(mouse) => {
                        // Handle mouse events
                        // TODO: Implement mouse handling with bounds from UI
                    }
                    AppEvent::Resize(_, _) => {
                        // Terminal will redraw automatically
                    }
                    AppEvent::Tick => {
                        // Update preview if needed
                        if self.show_preview && !self.sessions.is_empty() {
                            self.update_preview().await;
                        }
                    }
                    AppEvent::Error(e) => {
                        self.error_message = Some(e);
                    }
                    AppEvent::Quit => {
                        self.should_quit = true;
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        terminal.restore()?;
        Ok(())
    }

    /// Handle an action
    async fn handle_action(&mut self, action: Action) {
        // Clear messages
        self.status_message = None;

        match action {
            Action::Quit => {
                if matches!(self.input_mode, InputMode::Normal) && self.view == View::Sessions {
                    self.should_quit = true;
                } else {
                    self.go_back();
                }
            }
            Action::ForceQuit => {
                self.should_quit = true;
            }
            Action::Back => self.go_back(),
            Action::Up => self.move_up(),
            Action::Down => self.move_down(),
            Action::Top => self.move_to_top(),
            Action::Bottom => self.move_to_bottom(),
            Action::PageUp => self.page_up(),
            Action::PageDown => self.page_down(),
            Action::Select => self.select().await,
            Action::NewSession => self.start_new_session(),
            Action::RenameSession => self.start_rename_session(),
            Action::KillSession => self.confirm_kill_session(),
            Action::DetachSession => {
                // In Settings view with Hosts, 'd' deletes a host instead of detaching
                if self.view == View::Settings && self.is_hosts_category() {
                    self.delete_host();
                } else {
                    self.detach_session().await;
                }
            }
            Action::AttachSession => {
                // In Settings view with Hosts, 'a' adds a host instead of attaching
                if self.view == View::Settings && self.is_hosts_category() {
                    self.start_add_host();
                } else {
                    self.attach_session(false).await;
                }
            }
            Action::AttachSpawn => self.attach_session(true).await,
            Action::AddHost => self.start_add_host(),
            Action::EditHost => {} // TODO
            Action::DeleteHost => self.delete_host(),
            Action::ViewWindows => self.view_windows().await,
            Action::ViewTemplates => self.view = View::Templates,
            Action::ViewSettings => {
                self.view = View::Settings;
                self.settings_category_index = 0;
                self.settings_item_index = 0;
            }
            Action::Refresh => self.refresh_sessions().await,
            Action::StartSearch => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }
            Action::ClearSearch => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
                self.apply_filter();
            }
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
            }
            Action::TogglePreview => {
                self.show_preview = !self.show_preview;
            }
            Action::SwitchHost => self.switch_host(),
            Action::InputChar(c) => self.input_char(c),
            Action::InputBackspace => self.input_backspace(),
            Action::InputDelete => self.input_delete(),
            Action::InputConfirm => self.input_confirm().await,
            Action::InputCancel => self.input_cancel(),
            Action::Left => {
                if self.view == View::Settings && self.settings_category_index > 0 {
                    self.settings_category_index -= 1;
                    self.settings_item_index = 0;
                }
            }
            Action::Right => {
                if self.view == View::Settings {
                    use crate::ui::settings::SettingsCategory;
                    let max = SettingsCategory::all().len().saturating_sub(1);
                    if self.settings_category_index < max {
                        self.settings_category_index += 1;
                        self.settings_item_index = 0;
                    }
                }
            }
            Action::None => {}
        }
    }

    /// Go back to previous view/mode
    fn go_back(&mut self) {
        // First check if help overlay is shown
        if self.show_help {
            self.show_help = false;
            return;
        }

        match &self.input_mode {
            InputMode::Normal => match self.view {
                View::Windows => {
                    self.view = View::Sessions;
                    self.selected_session = None;
                    self.windows.clear();
                }
                View::Templates => {
                    self.view = View::Sessions;
                }
                View::Help => {
                    self.show_help = false;
                }
                View::Settings => {
                    // Save settings when closing
                    let _ = self.config.save(None);
                    self.view = View::Sessions;
                }
                View::Sessions => {}
            },
            InputMode::Search => {
                self.input_mode = InputMode::Normal;
                self.search_query.clear();
                self.apply_filter();
            }
            InputMode::Input { .. } | InputMode::Confirm { .. } => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
        }
    }

    /// Move selection up
    fn move_up(&mut self) {
        match self.view {
            View::Sessions => {
                if self.session_index > 0 {
                    self.session_index -= 1;
                }
            }
            View::Windows => {
                if self.window_index > 0 {
                    self.window_index -= 1;
                }
            }
            View::Templates => {
                if self.template_index > 0 {
                    self.template_index -= 1;
                }
            }
            View::Settings => {
                if self.settings_item_index > 0 {
                    self.settings_item_index -= 1;
                }
            }
            View::Help => {}
        }
    }

    /// Move selection down
    fn move_down(&mut self) {
        match self.view {
            View::Sessions => {
                let max = self.filtered_sessions.len().saturating_sub(1);
                if self.session_index < max {
                    self.session_index += 1;
                }
            }
            View::Windows => {
                let max = self.windows.len().saturating_sub(1);
                if self.window_index < max {
                    self.window_index += 1;
                }
            }
            View::Templates => {
                let max = self.templates.len().saturating_sub(1);
                if self.template_index < max {
                    self.template_index += 1;
                }
            }
            View::Settings => {
                let max = self.get_settings_item_count().saturating_sub(1);
                if self.settings_item_index < max {
                    self.settings_item_index += 1;
                }
            }
            View::Help => {}
        }
    }

    /// Move to top
    fn move_to_top(&mut self) {
        match self.view {
            View::Sessions => self.session_index = 0,
            View::Windows => self.window_index = 0,
            View::Templates => self.template_index = 0,
            View::Settings => self.settings_item_index = 0,
            View::Help => {}
        }
    }

    /// Move to bottom
    fn move_to_bottom(&mut self) {
        match self.view {
            View::Sessions => {
                self.session_index = self.filtered_sessions.len().saturating_sub(1);
            }
            View::Windows => {
                self.window_index = self.windows.len().saturating_sub(1);
            }
            View::Templates => {
                self.template_index = self.templates.len().saturating_sub(1);
            }
            View::Settings => {
                self.settings_item_index = self.get_settings_item_count().saturating_sub(1);
            }
            View::Help => {}
        }
    }

    /// Page up
    fn page_up(&mut self) {
        match self.view {
            View::Sessions => {
                self.session_index = self.session_index.saturating_sub(10);
            }
            View::Windows => {
                self.window_index = self.window_index.saturating_sub(10);
            }
            View::Templates => {
                self.template_index = self.template_index.saturating_sub(10);
            }
            View::Settings => {
                self.settings_item_index = self.settings_item_index.saturating_sub(10);
            }
            View::Help => {}
        }
    }

    /// Page down
    fn page_down(&mut self) {
        match self.view {
            View::Sessions => {
                let max = self.filtered_sessions.len().saturating_sub(1);
                self.session_index = (self.session_index + 10).min(max);
            }
            View::Windows => {
                let max = self.windows.len().saturating_sub(1);
                self.window_index = (self.window_index + 10).min(max);
            }
            View::Templates => {
                let max = self.templates.len().saturating_sub(1);
                self.template_index = (self.template_index + 10).min(max);
            }
            View::Settings => {
                let max = self.get_settings_item_count().saturating_sub(1);
                self.settings_item_index = (self.settings_item_index + 10).min(max);
            }
            View::Help => {}
        }
    }

    /// Get the number of items in current settings category
    fn get_settings_item_count(&self) -> usize {
        use crate::ui::settings::{get_settings_for_category, SettingsCategory};
        let categories = SettingsCategory::all();
        if let Some(cat) = categories.get(self.settings_category_index) {
            if *cat == SettingsCategory::Hosts {
                self.config.hosts.len().max(1) // At least 1 for "add host" option
            } else {
                get_settings_for_category(&self.config, *cat).len()
            }
        } else {
            0
        }
    }

    /// Handle select action
    async fn select(&mut self) {
        match &self.input_mode {
            InputMode::Confirm { .. } => {
                self.execute_confirm_action().await;
            }
            InputMode::Normal => {
                match self.view {
                    View::Sessions => {
                        // Attach to selected session
                        self.attach_session(false).await;
                    }
                    View::Windows => {
                        // Select window and attach
                        if let Some(window) = self.windows.get(self.window_index) {
                            if let Some(ref session) = self.selected_session {
                                let _ = screen::local::select_window(session, window.number).await;
                            }
                        }
                        self.attach_session(false).await;
                    }
                    View::Templates => {
                        self.create_from_template().await;
                    }
                    View::Settings => {
                        self.toggle_setting();
                    }
                    View::Help => {
                        self.show_help = false;
                    }
                }
            }
            _ => {}
        }
    }

    /// Get currently selected session
    fn get_selected_session(&self) -> Option<&Session> {
        if self.filtered_sessions.is_empty() {
            return None;
        }
        let idx = self.filtered_sessions.get(self.session_index)?;
        self.sessions.get(*idx)
    }

    /// Start creating a new session
    fn start_new_session(&mut self) {
        self.input_mode = InputMode::Input {
            prompt: "Session name:".to_string(),
            purpose: InputPurpose::NewSession,
        };
        self.input_buffer.clear();
        self.input_cursor = 0;
    }

    /// Start renaming a session
    fn start_rename_session(&mut self) {
        if let Some(session) = self.get_selected_session().cloned() {
            self.input_mode = InputMode::Input {
                prompt: format!("Rename '{}' to:", session.name),
                purpose: InputPurpose::RenameSession,
            };
            self.input_buffer = session.name.clone();
            self.input_cursor = self.input_buffer.len();
        }
    }

    /// Toggle a setting value
    fn toggle_setting(&mut self) {
        use crate::ui::settings::{apply_setting, get_settings_for_category, SettingsCategory};

        let categories = SettingsCategory::all();
        if let Some(cat) = categories.get(self.settings_category_index) {
            let mut settings_items = get_settings_for_category(&self.config, *cat);
            if let Some(item) = settings_items.get_mut(self.settings_item_index) {
                item.value.toggle();
                apply_setting(&mut self.config, &item.key, &item.value);
                // Save immediately
                let _ = self.config.save(None);
            }
        }
    }

    /// Confirm killing a session
    fn confirm_kill_session(&mut self) {
        if let Some(session) = self.get_selected_session() {
            self.input_mode = InputMode::Confirm {
                message: format!("Kill session '{}'?", session.name),
                action: ConfirmAction::KillSession(session.id.clone()),
            };
        }
    }

    /// Execute confirmed action
    async fn execute_confirm_action(&mut self) {
        let action = match &self.input_mode {
            InputMode::Confirm { action, .. } => action.clone(),
            _ => return,
        };

        self.input_mode = InputMode::Normal;

        match action {
            ConfirmAction::KillSession(id) => match screen::local::kill_session(&id).await {
                Ok(_) => {
                    self.status_message = Some("Killed session".to_string());
                    self.refresh_sessions().await;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to kill session: {}", e));
                }
            },
            ConfirmAction::KillWindow(session, number) => {
                match screen::local::kill_window(&session, number).await {
                    Ok(_) => {
                        self.status_message = Some(format!("Killed window {}", number));
                        self.refresh_windows().await;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to kill window: {}", e));
                    }
                }
            }
        }
    }

    /// Detach selected session
    async fn detach_session(&mut self) {
        if let Some(session) = self.get_selected_session() {
            let id = session.id.clone();
            let host = session.host.clone();

            let result = if let Some(ref host) = host {
                screen::remote::detach_session(&self.config, host, &id).await
            } else {
                screen::local::detach_session(&id).await
            };

            match result {
                Ok(_) => {
                    self.status_message = Some("Session detached".to_string());
                    self.refresh_sessions().await;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to detach: {}", e));
                }
            }
        }
    }

    /// Attach to selected session
    async fn attach_session(&mut self, spawn: bool) {
        if let Some(session) = self.get_selected_session() {
            let id = session.id.clone();
            let host = session.host.clone();

            if spawn {
                match screen::local::attach_spawn(&self.config, &id, host.as_deref()).await {
                    Ok(_) => {
                        self.status_message = Some("Opened in new terminal".to_string());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to spawn: {}", e));
                    }
                }
            } else {
                // This will exec and replace the current process
                let _ = screen::local::attach_exec(&id, host.as_deref()).await;
                // If we get here, attach failed
                self.error_message = Some("Failed to attach".to_string());
            }
        }
    }

    /// View windows for selected session
    async fn view_windows(&mut self) {
        if let Some(session) = self.get_selected_session() {
            self.selected_session = Some(session.id.clone());
            self.view = View::Windows;
            self.window_index = 0;
            self.refresh_windows().await;
        }
    }

    /// Refresh windows list
    async fn refresh_windows(&mut self) {
        if let Some(ref session) = self.selected_session {
            match screen::local::list_windows(session).await {
                Ok(windows) => {
                    self.windows = windows;
                    if self.window_index >= self.windows.len() {
                        self.window_index = self.windows.len().saturating_sub(1);
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to list windows: {}", e));
                }
            }
        }
    }

    /// Switch to next host
    fn switch_host(&mut self) {
        self.host_index = (self.host_index + 1) % self.hosts.len();
        self.selected_host = self.hosts[self.host_index].clone();
        // Refresh will be triggered by tick or manually
    }

    /// Refresh session list
    pub async fn refresh_sessions(&mut self) {
        let mut all_sessions = Vec::new();

        // Get local sessions
        match screen::local::list_sessions().await {
            Ok(sessions) => {
                all_sessions.extend(sessions);
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to list local sessions: {}", e));
            }
        }

        // Get remote sessions for configured hosts
        for host in &self.config.hosts {
            match screen::remote::list_sessions(&self.config, &host.name).await {
                Ok(sessions) => {
                    all_sessions.extend(sessions);
                }
                Err(_) => {
                    // Silently ignore remote errors
                }
            }
        }

        self.sessions = all_sessions;
        self.apply_filter();

        // Reset selection if out of bounds
        if self.session_index >= self.filtered_sessions.len() {
            self.session_index = self.filtered_sessions.len().saturating_sub(1);
        }
    }

    /// Load templates
    fn load_templates(&mut self) {
        match crate::config::templates::list_templates(&self.config) {
            Ok(templates) => {
                self.templates = templates;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load templates: {}", e));
            }
        }
    }

    /// Apply search filter
    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_sessions = (0..self.sessions.len()).collect();
        } else {
            self.filtered_sessions = self
                .sessions
                .iter()
                .enumerate()
                .filter_map(|(i, session)| {
                    let score = self.matcher.fuzzy_match(&session.name, &self.search_query);
                    score.map(|_| i)
                })
                .collect();
        }

        // Reset selection if out of bounds
        if self.session_index >= self.filtered_sessions.len() {
            self.session_index = 0;
        }
    }

    /// Update preview content
    async fn update_preview(&mut self) {
        if let Some(session) = self.get_selected_session() {
            let session_id = session.id.clone();
            let host = session.host.clone();

            let result = if let Some(ref host) = host {
                screen::remote::get_preview(&self.config, host, &session_id, None).await
            } else {
                screen::local::get_preview(&session_id, None).await
            };

            if let Ok(preview) = result {
                self.preview = preview;
            }
        }
    }

    /// Create session from template
    async fn create_from_template(&mut self) {
        if let Some(template) = self.templates.get(self.template_index).cloned() {
            // If template has variables, prompt for them
            if !template.variables.is_empty() {
                // For now, use defaults
                // TODO: Implement variable prompting
            }

            let variables = HashMap::new();
            let session_name = &template.name;

            match crate::config::templates::create_from_template(
                &self.config,
                &template,
                session_name,
                &variables,
            )
            .await
            {
                Ok(_) => {
                    self.status_message =
                        Some(format!("Created session from template '{}'", template.name));
                    self.view = View::Sessions;
                    self.refresh_sessions().await;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to create session: {}", e));
                }
            }
        }
    }

    /// Input character
    fn input_char(&mut self, c: char) {
        match &self.input_mode {
            InputMode::Search => {
                self.search_query.push(c);
                self.apply_filter();
            }
            InputMode::Input { .. } => {
                self.input_buffer.insert(self.input_cursor, c);
                self.input_cursor += 1;
            }
            _ => {}
        }
    }

    /// Handle backspace
    fn input_backspace(&mut self) {
        match &self.input_mode {
            InputMode::Search => {
                self.search_query.pop();
                self.apply_filter();
            }
            InputMode::Input { .. } => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                    self.input_buffer.remove(self.input_cursor);
                }
            }
            _ => {}
        }
    }

    /// Handle delete
    fn input_delete(&mut self) {
        if let InputMode::Input { .. } = &self.input_mode {
            if self.input_cursor < self.input_buffer.len() {
                self.input_buffer.remove(self.input_cursor);
            }
        }
    }

    /// Confirm input
    async fn input_confirm(&mut self) {
        match &self.input_mode {
            InputMode::Search => {
                // Keep search active, just stay in filter mode
                self.input_mode = InputMode::Normal;
            }
            InputMode::Input { purpose, .. } => {
                let value = self.input_buffer.clone();
                let purpose = purpose.clone();
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();

                if value.is_empty() {
                    return;
                }

                match purpose {
                    InputPurpose::NewSession => {
                        match screen::local::create_session(
                            &value,
                            None,
                            self.config.screen.default_shell.as_deref(),
                        )
                        .await
                        {
                            Ok(_) => {
                                self.status_message = Some(format!("Created session '{}'", value));
                                self.refresh_sessions().await;
                            }
                            Err(e) => {
                                self.error_message =
                                    Some(format!("Failed to create session: {}", e));
                            }
                        }
                    }
                    InputPurpose::RenameSession => {
                        if let Some(session) = self.get_selected_session().cloned() {
                            match screen::local::rename_session(&session.id, &value).await {
                                Ok(_) => {
                                    self.status_message =
                                        Some(format!("Renamed '{}' -> '{}'", session.name, value));
                                    self.refresh_sessions().await;
                                }
                                Err(e) => {
                                    self.error_message =
                                        Some(format!("Failed to rename session: {}", e));
                                }
                            }
                        }
                    }
                    InputPurpose::NewWindow => {
                        if let Some(ref session) = self.selected_session {
                            match screen::local::create_window(session, Some(&value)).await {
                                Ok(_) => {
                                    self.status_message =
                                        Some(format!("Created window '{}'", value));
                                    self.refresh_windows().await;
                                }
                                Err(e) => {
                                    self.error_message =
                                        Some(format!("Failed to create window: {}", e));
                                }
                            }
                        }
                    }
                    InputPurpose::RenameWindow => {
                        // TODO: Implement window rename
                    }
                    InputPurpose::TemplateVariable(_) => {
                        // TODO: Implement template variable handling
                    }
                    InputPurpose::AddHostName => {
                        if let Some(ref mut host) = self.new_host {
                            host.0 = value;
                        }
                        self.input_mode = InputMode::Input {
                            prompt: "Hostname (e.g., example.com):".to_string(),
                            purpose: InputPurpose::AddHostHostname,
                        };
                        self.input_buffer.clear();
                        self.input_cursor = 0;
                        return;
                    }
                    InputPurpose::AddHostHostname => {
                        if let Some(ref mut host) = self.new_host {
                            host.1 = value;
                        }
                        self.input_mode = InputMode::Input {
                            prompt: "Username:".to_string(),
                            purpose: InputPurpose::AddHostUser,
                        };
                        self.input_buffer.clear();
                        self.input_cursor = 0;
                        return;
                    }
                    InputPurpose::AddHostUser => {
                        if let Some(ref mut host) = self.new_host {
                            host.2 = value;
                        }
                        self.input_mode = InputMode::Input {
                            prompt: "Port (default 22):".to_string(),
                            purpose: InputPurpose::AddHostPort,
                        };
                        self.input_buffer = "22".to_string();
                        self.input_cursor = 2;
                        return;
                    }
                    InputPurpose::AddHostPort => {
                        if let Some(ref mut host) = self.new_host {
                            host.3 = if value.is_empty() { "22".to_string() } else { value };
                        }
                        self.input_mode = InputMode::Input {
                            prompt: "Identity file (e.g., ~/.ssh/id_ed25519):".to_string(),
                            purpose: InputPurpose::AddHostIdentityFile,
                        };
                        self.input_buffer.clear();
                        self.input_cursor = 0;
                        return;
                    }
                    InputPurpose::AddHostIdentityFile => {
                        if let Some(ref mut host) = self.new_host {
                            host.4 = value;
                        }
                        // Now save the host
                        self.finish_add_host();
                    }
                }
            }
            InputMode::Confirm { .. } => {
                self.execute_confirm_action().await;
            }
            InputMode::Normal => {}
        }
    }

    /// Cancel input
    fn input_cancel(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.input_cursor = 0;
        self.new_host = None;
    }

    /// Check if currently viewing Hosts category in Settings
    fn is_hosts_category(&self) -> bool {
        use crate::ui::settings::SettingsCategory;
        let categories = SettingsCategory::all();
        categories.get(self.settings_category_index) == Some(&SettingsCategory::Hosts)
    }

    /// Start adding a new host
    fn start_add_host(&mut self) {
        self.new_host = Some((String::new(), String::new(), String::new(), "22".to_string(), String::new()));
        self.input_mode = InputMode::Input {
            prompt: "Host name (alias):".to_string(),
            purpose: InputPurpose::AddHostName,
        };
        self.input_buffer.clear();
        self.input_cursor = 0;
    }

    /// Delete selected host
    fn delete_host(&mut self) {
        if !self.is_hosts_category() || self.config.hosts.is_empty() {
            return;
        }
        let idx = self.settings_item_index.min(self.config.hosts.len().saturating_sub(1));
        if idx < self.config.hosts.len() {
            self.config.hosts.remove(idx);
            let _ = self.config.save(None);
            // Update hosts list
            self.hosts = vec![None];
            for host in &self.config.hosts {
                self.hosts.push(Some(host.name.clone()));
            }
            self.status_message = Some("Host deleted".to_string());
        }
    }

    /// Finish adding a new host
    fn finish_add_host(&mut self) {
        use crate::config::hosts::HostConfig;

        if let Some((name, hostname, user, port, identity_file)) = self.new_host.take() {
            if name.is_empty() || hostname.is_empty() {
                self.error_message = Some("Host name and hostname are required".to_string());
                return;
            }

            let port_num = port.parse::<u16>().ok();

            let new_host = HostConfig {
                name: name.clone(),
                hostname,
                user: if user.is_empty() { None } else { Some(user) },
                port: port_num,
                identity_file: if identity_file.is_empty() { None } else { Some(identity_file) },
            };

            self.config.hosts.push(new_host);
            let _ = self.config.save(None);

            // Update hosts list
            self.hosts = vec![None];
            for host in &self.config.hosts {
                self.hosts.push(Some(host.name.clone()));
            }

            self.status_message = Some(format!("Added host '{}'", name));
        }
    }

    /// Get sessions for current host filter
    pub fn get_visible_sessions(&self) -> Vec<&Session> {
        self.filtered_sessions
            .iter()
            .filter_map(|&idx| self.sessions.get(idx))
            .filter(|s| {
                match (&self.selected_host, &s.host) {
                    (None, None) => true,           // Local only
                    (Some(h), Some(sh)) => h == sh, // Matching host
                    (None, Some(_)) => true,        // Show all when local selected
                    _ => false,
                }
            })
            .collect()
    }
}
