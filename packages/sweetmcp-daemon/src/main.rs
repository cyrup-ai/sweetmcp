mod cli;
mod config;
mod daemon;
mod install;
mod installer;
mod ipc;
mod lifecycle;
mod manager;
mod service;
mod signing;
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

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    if let Err(e) = rt.block_on(real_main()) {
        error!("{e:#}");
        std::process::exit(1);
    }
}

async fn real_main() -> Result<()> {
    let args = cli::Args::parse();

    match args.sub.unwrap_or(cli::Cmd::Run {
        foreground: false,
        config: None,
        system: false,
    }) {
        cli::Cmd::Run {
            foreground,
            config,
            system,
        } => run_daemon(foreground, config, system).await,
        cli::Cmd::Install {
            dry_run,
            sign,
            identity,
        } => installer::install(dry_run, sign, identity).await,
        cli::Cmd::Uninstall { dry_run } => installer::uninstall_async(dry_run).await,
        cli::Cmd::Sign {
            binary,
            identity,
            verify,
            show_config,
            self_sign,
        } => handle_sign_command(binary, identity, verify, show_config, self_sign).await,
    }
}

async fn run_daemon(
    force_foreground: bool,
    config_path: Option<String>,
    use_system: bool,
) -> Result<()> {
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
    let cfg_str = fs::read_to_string(&cfg_path).or_else(|_| {
        info!("Config not found at {}, using defaults", cfg_path.display());
        Ok::<String, anyhow::Error>(toml::to_string_pretty(&config::ServiceConfig::default())?)
    })?;
    let cfg: config::ServiceConfig = toml::from_str(&cfg_str)?;

    info!("Using config from: {}", cfg_path.display());

    manager::install_signal_handlers();
    let mut mgr = ServiceManager::new(&cfg)?;

    // Start SSE server if enabled
    mgr.start_sse_server(&cfg).await?;

    daemon::systemd_ready(); // tell systemd we are ready
    info!("Cyrup daemon started (pid {})", std::process::id());
    mgr.run()?;
    info!("Cyrup daemon exiting");
    Ok(())
}

async fn handle_sign_command(
    binary: Option<String>,
    identity: Option<String>,
    verify: bool,
    show_config: bool,
    self_sign: bool,
) -> Result<()> {
    // Check if signing is available on this platform
    if !signing::is_signing_available() {
        eprintln!("Code signing is not available on this platform");
        return Ok(());
    }

    if show_config {
        let sample = signing::config::create_sample_config()?;
        println!("Sample signing configuration:\n\n{}", sample);
        return Ok(());
    }

    // Handle self-signing
    if self_sign {
        println!("Self-signing current binary...");
        match signing::sign_self() {
            Ok(_) => {
                println!("✓ Successfully self-signed");
                return Ok(());
            }
            Err(e) => {
                eprintln!("✗ Failed to self-sign: {}", e);
                std::process::exit(1);
            }
        }
    }

    let binary_path = if let Some(path) = binary {
        PathBuf::from(path)
    } else {
        std::env::current_exe()?
    };

    if verify {
        // Verify signature
        match signing::verify_signature(&binary_path) {
            Ok(true) => {
                println!("✓ {} is properly signed", binary_path.display());
                Ok(())
            }
            Ok(false) => {
                eprintln!(
                    "✗ {} is not signed or signature is invalid",
                    binary_path.display()
                );
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("✗ Failed to verify signature: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Sign the binary
        let mut config = signing::SigningConfig::load()?;
        config.binary_path = binary_path;
        config.output_path = config.binary_path.clone();

        // Override identity if provided
        if let Some(id) = identity {
            match &mut config.platform {
                #[cfg(target_os = "macos")]
                signing::PlatformConfig::MacOS { identity, .. } => *identity = id,
                #[cfg(target_os = "windows")]
                signing::PlatformConfig::Windows { certificate, .. } => *certificate = id,
                #[cfg(target_os = "linux")]
                signing::PlatformConfig::Linux { key_id, .. } => *key_id = Some(id),
                _ => {}
            }
        }

        println!("Signing {}...", config.binary_path.display());

        match signing::sign_binary(&config) {
            Ok(_) => {
                println!("✓ Successfully signed {}", config.binary_path.display());

                // Verify the signature
                if signing::verify_signature(&config.output_path)? {
                    println!("✓ Signature verified");
                } else {
                    eprintln!("⚠️  Warning: Signature verification failed");
                }

                Ok(())
            }
            Err(e) => {
                eprintln!("✗ Failed to sign binary: {}", e);
                std::process::exit(1);
            }
        }
    }
}
