use crate::install::{install_daemon, uninstall_daemon, InstallerBuilder, InstallerError};
use anyhow::{Context, Result};
use log::info;
use std::fs;
use std::path::Path;

/// Install the daemon using elevated_daemon_installer with GUI authorization.
pub fn install(dry: bool, sign: bool, identity: Option<String>) -> Result<()> {
    // Create config directory and file in user-specific location
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("cyrupd");

    let config_path = config_dir.join("cyrupd.toml");

    // Create config if it doesn't exist
    if !config_path.exists() {
        info!("Creating default config at {}", config_path.display());
        if !dry {
            fs::create_dir_all(&config_dir)?;
            let def = crate::config::ServiceConfig::default();
            fs::write(&config_path, toml::to_string_pretty(&def)?)?;
        }
    }

    // Get the current executable path
    let exe_path = std::env::current_exe().context("current_exe()")?;

    if dry {
        info!("[dry run] Would install daemon:");
        info!("  - Binary: {}", exe_path.display());
        info!("  - Config: {}", config_path.display());
        info!("  - Service: cyrupd");
        if sign && cfg!(target_os = "macos") {
            info!("  - Codesign: {}", identity.as_deref().unwrap_or("-"));
        }
        return Ok(());
    }

    // Build the pingora server first
    info!("Building SweetMCP Pingora server...");
    let build_status = std::process::Command::new("cargo")
        .args(&["build", "--package", "sweetmcp-pingora"])
        .current_dir(std::env::current_dir()?)
        .status()?;
    
    if !build_status.success() {
        return Err(anyhow::anyhow!("Failed to build sweetmcp-pingora"));
    }

    // Create the pingora service definition
    let pingora_binary = exe_path.parent().unwrap().join("sweetmcp_server");
    let pingora_service = crate::config::ServiceDefinition {
        name: "sweetmcp-pingora".to_string(),
        description: Some("SweetMCP Pingora Gateway Server".to_string()),
        command: pingora_binary.to_string_lossy().to_string(),
        working_dir: Some(std::env::current_dir()?.to_string_lossy().to_string()),
        env_vars: {
            let mut env = std::collections::HashMap::new();
            env.insert("RUST_LOG".to_string(), "info".to_string());
            env.insert("SWEETMCP_TCP_BIND".to_string(), "0.0.0.0:8443".to_string());
            env.insert("SWEETMCP_UDS_PATH".to_string(), "/run/sugora.sock".to_string());
            env.insert("SWEETMCP_METRICS_BIND".to_string(), "127.0.0.1:9090".to_string());
            env.insert("SWEETMCP_DEV_MODE".to_string(), "true".to_string());
            env
        },
        auto_restart: true,
        user: Some("root".to_string()),
        group: Some("wheel".to_string()),
        restart_delay_s: Some(5),
        depends_on: Vec::new(),
        health_check: None,
        log_rotation: None,
        watch_dirs: Vec::new(),
        ephemeral_dir: None,
        memfs: None,
    };

    // Build the installer configuration
    let installer = InstallerBuilder::new("cyrupd", exe_path)
        .description("Cyrup Service Manager")
        .arg("run")
        .arg("--foreground")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .env("RUST_LOG", "info")
        .auto_restart(true)
        .network(true)
        .service(pingora_service);

    // Platform-specific user/group settings
    #[cfg(target_os = "linux")]
    let installer = {
        if let Ok(group) = nix::unistd::Group::from_name("cyops")? {
            if group.is_some() {
                installer.group("cyops")
            } else {
                installer
            }
        } else {
            installer
        }
    };

    // On macOS, run as root with wheel group for system daemon privileges
    #[cfg(target_os = "macos")]
    let installer = installer
        .user("root")
        .group("wheel");

    // Install the daemon with GUI authorization
    match install_daemon(installer) {
        Ok(()) => {
            info!("Daemon installed successfully");

            // macOS codesign after installation if requested
            if cfg!(target_os = "macos") && sign {
                let installed_path = Path::new("/usr/local/bin/cyrupd");
                let id = identity.unwrap_or_else(|| "-".to_string());
                info!(
                    "Codesigning {} with identity {}",
                    installed_path.display(),
                    id
                );

                let status = std::process::Command::new("codesign")
                    .args([
                        "--timestamp",
                        "--options",
                        "runtime",
                        "--force",
                        "--sign",
                        &id,
                    ])
                    .arg(installed_path)
                    .status()?;

                if !status.success() {
                    return Err(anyhow::anyhow!("codesign failed"));
                }
            }

            Ok(())
        }
        Err(InstallerError::Cancelled) => Err(anyhow::anyhow!("Installation cancelled by user")),
        Err(InstallerError::PermissionDenied) => Err(anyhow::anyhow!(
            "Permission denied. Please provide administrator credentials."
        )),
        Err(e) => Err(e.into()),
    }
}

/// Uninstall the daemon using elevated_daemon_installer
pub fn uninstall(dry: bool) -> Result<()> {
    if dry {
        info!("[dry run] Would uninstall daemon: cyrupd");
        return Ok(());
    }

    match uninstall_daemon("cyrupd") {
        Ok(()) => {
            info!("Daemon uninstalled successfully");
            Ok(())
        }
        Err(InstallerError::Cancelled) => Err(anyhow::anyhow!("Uninstallation cancelled by user")),
        Err(InstallerError::PermissionDenied) => Err(anyhow::anyhow!(
            "Permission denied. Please provide administrator credentials."
        )),
        Err(e) => Err(e.into()),
    }
}
