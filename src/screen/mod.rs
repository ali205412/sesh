//! Screen interaction module
//!
//! Provides functionality for interacting with GNU Screen sessions,
//! both locally and over SSH.

pub mod local;
pub mod parser;
pub mod remote;
pub mod types;

pub use types::{Preview, Session, SessionStatus, Window, WindowActivity};
