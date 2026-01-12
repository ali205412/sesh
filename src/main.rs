//! sesh - A fully-featured TUI manager for GNU Screen
//!
//! Provides an intuitive terminal user interface for managing screen sessions,
//! windows, and templates without memorizing cryptic screen commands.

mod app;
mod config;
mod event;
mod integrations;
mod screen;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// A fully-featured TUI manager for GNU Screen
#[derive(Parser, Debug)]
#[command(name = "sesh")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Custom config file path
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<String>,

    /// Connect to a remote host
    #[arg(short = 'H', long, value_name = "HOST")]
    pub host: Option<String>,

    /// Enable debug logging
    #[arg(short, long)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all screen sessions
    List {
        /// Show sessions from all configured hosts
        #[arg(short, long)]
        all: bool,
    },

    /// Create a new screen session
    New {
        /// Session name
        name: String,

        /// Working directory for the session
        #[arg(short, long)]
        dir: Option<String>,
    },

    /// Attach to an existing session
    Attach {
        /// Session name or ID
        session: String,

        /// Open in a new terminal window instead of replacing current process
        #[arg(short, long)]
        spawn: bool,
    },

    /// Detach a session (remote detach)
    Detach {
        /// Session name or ID
        session: String,
    },

    /// Kill a screen session
    Kill {
        /// Session name or ID
        session: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Create a session from a template
    Start {
        /// Template name
        template: String,

        /// Override session name
        #[arg(short, long)]
        name: Option<String>,

        /// Template variables (key=value)
        #[arg(short, long, value_name = "KEY=VALUE")]
        var: Vec<String>,
    },

    /// List available templates
    Templates,

    /// Show windows in a session
    Windows {
        /// Session name or ID
        session: String,
    },

    /// Rename a screen session
    Rename {
        /// Current session name or ID
        session: String,

        /// New session name
        #[arg(value_name = "NEW_NAME")]
        new_name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.debug {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new("sesh=debug"))
            .with(tracing_subscriber::fmt::layer().with_target(false))
            .init();
    }

    // Load configuration
    let config = config::Settings::load(cli.config.as_deref())?;

    match cli.command {
        Some(Commands::List { all }) => cmd_list(&config, all, cli.host.as_deref()).await,
        Some(Commands::New { name, dir }) => cmd_new(&config, &name, dir.as_deref()).await,
        Some(Commands::Attach { session, spawn }) => {
            cmd_attach(&config, &session, spawn, cli.host.as_deref()).await
        }
        Some(Commands::Detach { session }) => {
            cmd_detach(&config, &session, cli.host.as_deref()).await
        }
        Some(Commands::Kill { session, force }) => {
            cmd_kill(&config, &session, force, cli.host.as_deref()).await
        }
        Some(Commands::Start {
            template,
            name,
            var,
        }) => cmd_start(&config, &template, name.as_deref(), &var).await,
        Some(Commands::Templates) => cmd_templates(&config).await,
        Some(Commands::Windows { session }) => {
            cmd_windows(&config, &session, cli.host.as_deref()).await
        }
        Some(Commands::Rename { session, new_name }) => {
            cmd_rename(&session, &new_name, cli.host.as_deref()).await
        }
        None => {
            // Launch TUI
            run_tui(config, cli.host).await
        }
    }
}

/// Run the interactive TUI
async fn run_tui(config: config::Settings, host: Option<String>) -> Result<()> {
    let mut app = app::App::new(config, host)?;
    app.run().await
}

/// List sessions command
async fn cmd_list(config: &config::Settings, all: bool, host: Option<&str>) -> Result<()> {
    let sessions = if let Some(host) = host {
        screen::remote::list_sessions(config, host).await?
    } else {
        let mut sessions = screen::local::list_sessions().await?;
        if all {
            for host_config in &config.hosts {
                if let Ok(remote_sessions) =
                    screen::remote::list_sessions(config, &host_config.name).await
                {
                    sessions.extend(remote_sessions);
                }
            }
        }
        sessions
    };

    if sessions.is_empty() {
        println!("No screen sessions found.");
    } else {
        println!(
            "{:<20} {:<10} {:<12} {:<30}",
            "NAME", "WINDOWS", "STATUS", "CREATED"
        );
        println!("{}", "-".repeat(75));
        for session in sessions {
            println!(
                "{:<20} {:<10} {:<12} {:<30}",
                session.name,
                session.window_count,
                session.status,
                session.created.format("%Y-%m-%d %H:%M:%S")
            );
        }
    }
    Ok(())
}

/// Create new session command
async fn cmd_new(config: &config::Settings, name: &str, dir: Option<&str>) -> Result<()> {
    screen::local::create_session(name, dir, config.screen.default_shell.as_deref()).await?;
    println!("Created session: {}", name);
    Ok(())
}

/// Attach to session command
async fn cmd_attach(
    config: &config::Settings,
    session: &str,
    spawn: bool,
    host: Option<&str>,
) -> Result<()> {
    if spawn {
        screen::local::attach_spawn(config, session, host).await
    } else {
        screen::local::attach_exec(session, host).await
    }
}

/// Detach session command
async fn cmd_detach(config: &config::Settings, session: &str, host: Option<&str>) -> Result<()> {
    if let Some(host) = host {
        screen::remote::detach_session(config, host, session).await?;
    } else {
        screen::local::detach_session(session).await?;
    }
    println!("Detached session: {}", session);
    Ok(())
}

/// Kill session command
async fn cmd_kill(
    _config: &config::Settings,
    session: &str,
    force: bool,
    host: Option<&str>,
) -> Result<()> {
    if !force {
        print!("Kill session '{}'? [y/N] ", session);
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    if host.is_some() {
        // TODO: Implement remote kill
        anyhow::bail!("Remote kill not yet implemented");
    } else {
        screen::local::kill_session(session).await?;
    }
    println!("Killed session: {}", session);
    Ok(())
}

/// Start from template command
async fn cmd_start(
    config: &config::Settings,
    template: &str,
    name: Option<&str>,
    vars: &[String],
) -> Result<()> {
    let tmpl = config::templates::load_template(config, template)?;
    let session_name = name.unwrap_or(&tmpl.name);

    // Parse variables
    let mut variables: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for var in vars {
        if let Some((key, value)) = var.split_once('=') {
            variables.insert(key.to_string(), value.to_string());
        }
    }

    config::templates::create_from_template(config, &tmpl, session_name, &variables).await?;
    println!(
        "Created session '{}' from template '{}'",
        session_name, template
    );
    Ok(())
}

/// List templates command
async fn cmd_templates(config: &config::Settings) -> Result<()> {
    let templates = config::templates::list_templates(config)?;

    if templates.is_empty() {
        println!("No templates found.");
        println!("Templates should be placed in: ~/.config/sesh/templates/");
    } else {
        println!("{:<20} {:<10} {:<50}", "NAME", "WINDOWS", "DESCRIPTION");
        println!("{}", "-".repeat(80));
        for tmpl in templates {
            println!(
                "{:<20} {:<10} {:<50}",
                tmpl.name,
                tmpl.windows.len(),
                tmpl.description.as_deref().unwrap_or("-")
            );
        }
    }
    Ok(())
}

/// Show windows command
async fn cmd_windows(_config: &config::Settings, session: &str, host: Option<&str>) -> Result<()> {
    let windows = if host.is_some() {
        // TODO: Implement remote windows
        anyhow::bail!("Remote windows not yet implemented");
    } else {
        screen::local::list_windows(session).await?
    };

    if windows.is_empty() {
        println!("No windows found in session '{}'.", session);
    } else {
        println!("{:<5} {:<20} {:<50}", "NUM", "NAME", "COMMAND");
        println!("{}", "-".repeat(75));
        for window in windows {
            println!(
                "{:<5} {:<20} {:<50}",
                window.number,
                window.name,
                window.command.as_deref().unwrap_or("-")
            );
        }
    }
    Ok(())
}

/// Rename session command
async fn cmd_rename(session: &str, new_name: &str, host: Option<&str>) -> Result<()> {
    if host.is_some() {
        anyhow::bail!("Remote rename not yet implemented");
    } else {
        screen::local::rename_session(session, new_name).await?;
    }
    println!("Renamed '{}' -> '{}'", session, new_name);
    Ok(())
}
