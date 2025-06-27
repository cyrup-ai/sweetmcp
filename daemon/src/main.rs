mod cli;
mod config;
mod daemon;
mod installer;
mod ipc;
mod lifecycle;
mod manager;
mod service;
mod state_machine;

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use log::{error, info};
use manager::ServiceManager;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    if let Err(e) = real_main() {
        error!("{e:#}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<()> {
    let args = cli::Args::parse();
    
    match args.sub.unwrap_or(cli::Cmd::Run { foreground: false, config: None, system: false }) {
        cli::Cmd::Run { foreground, config, system } => run_daemon(foreground, config, system),
        cli::Cmd::Install { dry_run, sign, identity } =>
            installer::install(dry_run, sign, identity),
        cli::Cmd::Uninstall { dry_run } =>
            installer::uninstall(dry_run),
    }
}

fn run_daemon(force_foreground: bool, config_path: Option<String>, use_system: bool) -> Result<()> {
    let should_stay_foreground = force_foreground || daemon::need_foreground();
    
    if !should_stay_foreground {
        daemon::daemonise(Path::new("/var/run/cyrupd.pid"))?;
    }
    
    // Determine config path based on CLI arguments
    let cfg_path = if let Some(path) = config_path {
        // User specified an explicit config path
        PathBuf::from(path)
    } else if use_system {
        // User wants system-wide config
        PathBuf::from("/etc/cyrupd/cyrupd.toml")
    } else {
        // Default to user config directory
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("cyrupd");
        config_dir.join("cyrupd.toml")
    };
    
    // Load or create default config
    let cfg_str = fs::read_to_string(&cfg_path)
        .or_else(|_| {
            info!("Config not found at {}, using defaults", cfg_path.display());
            Ok::<String, anyhow::Error>(toml::to_string_pretty(&config::ServiceConfig::default())?)
        })?;
    let cfg: config::ServiceConfig = toml::from_str(&cfg_str)?;
    
    info!("Using config from: {}", cfg_path.display());
    
    manager::install_signal_handlers();
    let mgr = ServiceManager::new(&cfg)?;
    daemon::systemd_ready();  // tell systemd we are ready
    info!("Cyrup daemon started (pid {})", std::process::id());
    mgr.run()?;
    info!("Cyrup daemon exiting");
    Ok(())
}