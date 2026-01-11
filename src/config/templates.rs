//! Session templates
//!
//! YAML-based templates for creating pre-configured sessions.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::Settings;
use crate::screen;

/// Session template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Root directory for the session
    pub root: Option<String>,

    /// Commands to run on creation
    #[serde(default)]
    pub on_create: Vec<String>,

    /// Windows to create
    #[serde(default)]
    pub windows: Vec<TemplateWindow>,

    /// Variables for substitution
    #[serde(default)]
    pub variables: HashMap<String, TemplateVariable>,
}

/// Window definition in a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateWindow {
    /// Window name
    pub name: String,

    /// Command to run
    pub command: Option<String>,

    /// Working directory (relative to root or absolute)
    pub dir: Option<String>,

    /// Split panes
    #[serde(default)]
    pub splits: Vec<TemplateSplit>,
}

/// Split pane definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSplit {
    /// Command to run
    pub command: Option<String>,

    /// Size percentage
    pub size: Option<String>,

    /// Direction (horizontal or vertical)
    pub direction: Option<String>,
}

/// Template variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Prompt to show when asking for value
    pub prompt: Option<String>,

    /// Default value
    pub default: Option<String>,
}

/// List all available templates
pub fn list_templates(config: &Settings) -> Result<Vec<Template>> {
    let templates_dir = config.templates_dir();

    if !templates_dir.exists() {
        return Ok(Vec::new());
    }

    let mut templates = Vec::new();

    for entry in std::fs::read_dir(&templates_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path
            .extension()
            .map(|e| e == "yaml" || e == "yml")
            .unwrap_or(false)
        {
            if let Ok(template) = load_template_from_path(&path) {
                templates.push(template);
            }
        }
    }

    // Sort by name
    templates.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(templates)
}

/// Load a template by name
pub fn load_template(config: &Settings, name: &str) -> Result<Template> {
    let templates_dir = config.templates_dir();

    // Try .yaml extension first
    let yaml_path = templates_dir.join(format!("{}.yaml", name));
    if yaml_path.exists() {
        return load_template_from_path(&yaml_path);
    }

    // Try .yml extension
    let yml_path = templates_dir.join(format!("{}.yml", name));
    if yml_path.exists() {
        return load_template_from_path(&yml_path);
    }

    anyhow::bail!("Template '{}' not found", name)
}

/// Load a template from a specific path
fn load_template_from_path(path: &PathBuf) -> Result<Template> {
    let content =
        std::fs::read_to_string(path).context(format!("Failed to read template: {:?}", path))?;

    let template: Template =
        serde_yaml::from_str(&content).context(format!("Failed to parse template: {:?}", path))?;

    Ok(template)
}

/// Create a session from a template
pub async fn create_from_template(
    config: &Settings,
    template: &Template,
    session_name: &str,
    variables: &HashMap<String, String>,
) -> Result<()> {
    // Expand variables in root directory
    let root = template
        .root
        .as_ref()
        .map(|r| expand_variables(r, variables));

    // Create the main session
    screen::local::create_session(
        session_name,
        root.as_deref(),
        config.screen.default_shell.as_deref(),
    )
    .await?;

    // Give it a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create windows
    for (i, window) in template.windows.iter().enumerate() {
        if i == 0 {
            // Rename first window instead of creating
            screen::local::rename_window(session_name, 0, &window.name).await?;
        } else {
            // Create new window
            screen::local::create_window(session_name, Some(&window.name)).await?;
        }

        // Run command if specified
        if let Some(ref cmd) = window.command {
            let expanded_cmd = expand_variables(cmd, variables);
            let cmd_with_newline = format!("{}\n", expanded_cmd);
            screen::local::send_keys(session_name, &cmd_with_newline).await?;
        }

        // Small delay between windows
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Run on_create commands
    for cmd in &template.on_create {
        let expanded_cmd = expand_variables(cmd, variables);
        screen::local::send_keys(session_name, &format!("{}\n", expanded_cmd)).await?;
    }

    Ok(())
}

/// Expand variables in a string
fn expand_variables(s: &str, variables: &HashMap<String, String>) -> String {
    let mut result = s.to_string();

    for (key, value) in variables {
        result = result.replace(&format!("${{{}}}", key), value);
        result = result.replace(&format!("${}", key), value);
    }

    // Expand tilde
    shellexpand::tilde(&result).into_owned()
}

/// Generate an example template
pub fn generate_example_template() -> String {
    let template = Template {
        name: "example".to_string(),
        description: Some("Example development environment".to_string()),
        root: Some("~/projects/${PROJECT_NAME}".to_string()),
        on_create: vec![],
        windows: vec![
            TemplateWindow {
                name: "editor".to_string(),
                command: Some("nvim .".to_string()),
                dir: None,
                splits: vec![],
            },
            TemplateWindow {
                name: "server".to_string(),
                command: Some("npm run dev".to_string()),
                dir: None,
                splits: vec![],
            },
            TemplateWindow {
                name: "shell".to_string(),
                command: None,
                dir: None,
                splits: vec![],
            },
        ],
        variables: {
            let mut vars = HashMap::new();
            vars.insert(
                "PROJECT_NAME".to_string(),
                TemplateVariable {
                    prompt: Some("Project name:".to_string()),
                    default: Some("myproject".to_string()),
                },
            );
            vars
        },
    };

    serde_yaml::to_string(&template).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_variables() {
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "myproject".to_string());

        let result = expand_variables("~/projects/${NAME}", &vars);
        assert!(result.contains("myproject"));
    }

    #[test]
    fn test_parse_template() {
        let yaml = r#"
name: test
description: Test template
root: ~/projects
windows:
  - name: editor
    command: nvim .
  - name: shell
"#;
        let template: Result<Template, _> = serde_yaml::from_str(yaml);
        assert!(template.is_ok());
        let t = template.unwrap();
        assert_eq!(t.name, "test");
        assert_eq!(t.windows.len(), 2);
    }
}
