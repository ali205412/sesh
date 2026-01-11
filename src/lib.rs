//! sesh - A fully-featured TUI manager for GNU Screen
//!
//! This library provides the core functionality for managing GNU Screen sessions
//! through a terminal user interface.

pub mod app;
pub mod config;
pub mod event;
pub mod integrations;
pub mod screen;
pub mod ui;

// Re-export commonly used types
pub use app::App;
pub use config::Settings;
pub use screen::types::{Session, SessionStatus, Window, WindowActivity};
