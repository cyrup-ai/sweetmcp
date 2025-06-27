// This module handles UI-specific plugin commands like init, check, upgrade.
// Build logic has been moved to src/plugin/build/

mod check;
mod init;
mod upgrade;

pub use check::{CheckArgs, check_plugins};
use clap::Subcommand;
pub use init::{InitArgs, init_plugin};
pub use upgrade::{UpgradeArgs, upgrade_plugins};

/// Commands related to plugin management (excluding build).
#[derive(Subcommand, Debug)]
pub enum PluginCommands {
    /// Check that plugins are properly configured and optionally build them.
    #[command(about = "Check plugin configuration and optionally build WASM")]
    Check(CheckArgs),

    /// Initialize a new Rust WASM plugin from a template.
    #[command(about = "Initialize a new Rust WASM plugin from template")]
    Init(InitArgs),

    /// Upgrade common dependencies (like extism-pdk) in Rust plugins.
    #[command(about = "Upgrade common dependencies in Rust plugins")]
    Upgrade(UpgradeArgs),
    // NOTE: The 'Build' command is removed.
}

/// Handles the plugin subcommands passed from the main CLI.
pub fn handle_plugin_command(cmd: &PluginCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        PluginCommands::Check(args) => check::check_plugins(args)?,
        PluginCommands::Init(args) => init::init_plugin(args)?,
        PluginCommands::Upgrade(args) => upgrade::upgrade_plugins(args)?,
    }
    Ok(())
}
