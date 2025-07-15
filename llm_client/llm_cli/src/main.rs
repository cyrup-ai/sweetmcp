mod commands;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colorful::Colorful;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "llm")]
#[command(author, version, about = "CLI for interacting with LLMs", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE", global = true)]
    config: Option<PathBuf>,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Interactive chat with an LLM
    Chat(commands::chat::ChatArgs),
    
    /// Single completion request
    Completion(commands::completion::CompletionArgs),
    
    /// List available models
    Models {
        /// Filter by provider
        #[arg(short, long)]
        provider: Option<String>,
    },
    
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        key: String,
        value: String,
    },
    /// Initialize configuration file
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info", 
        2 => "debug",
        _ => "trace",
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(format!("llm_cli={},llm_client={}", log_level, log_level))
        .init();
    
    // Load configuration
    let config = config::Config::load(cli.config.as_deref()).await?;
    
    // Execute command
    match cli.command {
        Commands::Chat(args) => commands::chat::execute(args, config).await?,
        Commands::Completion(args) => commands::completion::execute(args, config).await?,
        Commands::Models { provider } => commands::list_models(provider).await?,
        Commands::Config { action } => handle_config_action(action, cli.config).await?,
    }
    
    Ok(())
}

async fn handle_config_action(action: ConfigAction, config_path: Option<PathBuf>) -> Result<()> {
    match action {
        ConfigAction::Show => {
            let config = config::Config::load(config_path.as_deref()).await?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigAction::Set { key, value } => {
            let mut config = config::Config::load(config_path.as_deref()).await?;
            config.set(&key, &value)?;
            config.save().await?;
            println!("{} {} = {}", "✓".green(), key, value);
        }
        ConfigAction::Init => {
            let path = config::Config::init_config_file(config_path.as_deref()).await?;
            println!("{} Created configuration file at: {}", "✓".green(), path.display());
        }
    }
    Ok(())
}

