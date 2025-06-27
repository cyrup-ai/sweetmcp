use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::{config::Config, plugin::PluginManager, router};

// --- Submodule Declarations ---
pub mod health;
pub mod plugin;

// --- Re-exports for CLI structure ---
pub use health::{HealthCommands, handle_health_command};
pub use plugin::{PluginCommands, handle_plugin_command};

// --- CLI Definition ---

/// CYRUP MCP API Server and CLI Tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the configuration file (YAML, JSON, or TOML).
    #[arg(short, long, value_name = "FILE", default_value = "config.yaml")]
    pub config: PathBuf,

    /// Optional path to write logs to a file instead of stderr.
    #[arg(long)]
    pub log_path: Option<String>,

    /// Log level (error, warn, info, debug, trace).
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Skip signature verification for OCI plugins (INSECURE - use with caution).
    #[arg(long)]
    pub insecure_skip_signature: bool,

    /// The command to execute.
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    /// Start the MCP JSON-RPC server, listening on stdin/stdout.
    Serve(ServeArgs),
    /// Manage plugins (check, init, upgrade).
    #[command(subcommand)]
    Plugin(PluginCommands),
    /// Perform health checks on server components.
    #[command(subcommand)]
    Health(HealthCommands),
    /// Validate the configuration file format and content.
    Validate,
}

#[derive(Parser, Debug)]
pub struct ServeArgs {
    /// Run as a system daemon (detached from terminal)
    #[arg(long)]
    pub daemon: bool,

    /// PID file path for daemon mode
    #[arg(long, default_value = "/var/run/cyrup-mcp.pid")]
    pub pid_file: PathBuf,

    /// Socket path for daemon mode (instead of stdin/stdout)
    #[arg(long, default_value = "/var/run/cyrup-mcp.sock")]
    pub socket_path: PathBuf,

    /// User to run daemon as (requires root to switch)
    #[arg(long)]
    pub user: Option<String>,

    /// Group to run daemon as (requires root to switch)  
    #[arg(long)]
    pub group: Option<String>,

    /// Working directory for daemon
    #[arg(long, default_value = "/")]
    pub working_dir: PathBuf,

    /// Enable systemd integration
    #[arg(long)]
    pub systemd: bool,
}

/// Parse CLI arguments only (no side effects)
pub fn parse_cli_args() -> Cli {
    Cli::parse()
}

/// Main entry point for the UI/CLI layer, called by `src/main.rs`.
/// This function only dispatches commands, all setup is done in main.rs.
pub async fn run_ui_with_state(
    cli: Cli,
    config: Config,
    plugin_manager: PluginManager,
) -> Result<()> {
    match cli.command {
        Commands::Serve(serve_args) => router::run_server(config, plugin_manager, serve_args).await,
        Commands::Plugin(plugin_cmd) => {
            // Convert the boxed error to anyhow::Error
            handle_plugin_command(&plugin_cmd).map_err(|e| anyhow::anyhow!("{}", e))
        }
        Commands::Health(health_cmd) => {
            // Remove Debug print to fix E0277
            handle_health_command(&health_cmd)
        }
        Commands::Validate => {
            println!("✅ Configuration file '{}' is valid.", cli.config.display());
            Ok(())
        }
    }
}
