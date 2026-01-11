//! SSH host configuration
//!
//! Defines SSH host entries for remote session management.

use serde::{Deserialize, Serialize};

/// SSH host configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    /// Host alias/name
    pub name: String,

    /// Hostname or IP address
    pub hostname: String,

    /// SSH username
    pub user: Option<String>,

    /// SSH port
    pub port: Option<u16>,

    /// Path to identity file
    pub identity_file: Option<String>,
}

impl HostConfig {
    /// Create a new host config
    pub fn new(name: &str, hostname: &str) -> Self {
        Self {
            name: name.to_string(),
            hostname: hostname.to_string(),
            user: None,
            port: None,
            identity_file: None,
        }
    }

    /// Set the username
    pub fn with_user(mut self, user: &str) -> Self {
        self.user = Some(user.to_string());
        self
    }

    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set the identity file
    pub fn with_identity_file(mut self, path: &str) -> Self {
        self.identity_file = Some(path.to_string());
        self
    }

    /// Get SSH connection string (user@hostname)
    pub fn connection_string(&self) -> String {
        if let Some(ref user) = self.user {
            format!("{}@{}", user, self.hostname)
        } else {
            self.hostname.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_config() {
        let host = HostConfig::new("prod", "example.com")
            .with_user("deploy")
            .with_port(22);

        assert_eq!(host.name, "prod");
        assert_eq!(host.connection_string(), "deploy@example.com");
    }
}
