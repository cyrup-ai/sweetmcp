use anyhow::{Context, Result};
use elevated_daemon_installer::{InstallerBuilder, install_daemon, uninstall_daemon};
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
    
    // Build the installer configuration
    let installer = InstallerBuilder::new("cyrupd", exe_path)
        .description("Cyrup Service Manager")
        .arg("run")
        .arg("--foreground")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .env("RUST_LOG", "info")
        .auto_restart(true)
        .network(true);
    
    // On Linux, try to use cyops group if it exists
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
    
    // Install the daemon with GUI authorization
    match install_daemon(installer) {
        Ok(()) => {
            info!("Daemon installed successfully");
            
            // macOS codesign after installation if requested
            if cfg!(target_os = "macos") && sign {
                let installed_path = Path::new("/usr/local/bin/cyrupd");
                let id = identity.unwrap_or_else(|| "-".to_string());
                info!("Codesigning {} with identity {}", installed_path.display(), id);
                
                let status = std::process::Command::new("codesign")
                    .args(["--timestamp", "--options", "runtime", "--force", "--sign", &id])
                    .arg(installed_path)
                    .status()?;
                    
                if !status.success() {
                    return Err(anyhow::anyhow!("codesign failed"));
                }
            }
            
            Ok(())
        }
        Err(elevated_daemon_installer::InstallerError::Cancelled) => {
            Err(anyhow::anyhow!("Installation cancelled by user"))
        }
        Err(elevated_daemon_installer::InstallerError::PermissionDenied) => {
            Err(anyhow::anyhow!("Permission denied. Please provide administrator credentials."))
        }
        Err(e) => Err(e.into())
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
        Err(elevated_daemon_installer::InstallerError::Cancelled) => {
            Err(anyhow::anyhow!("Uninstallation cancelled by user"))
        }
        Err(elevated_daemon_installer::InstallerError::PermissionDenied) => {
            Err(anyhow::anyhow!("Permission denied. Please provide administrator credentials."))
        }
        Err(e) => Err(e.into())
    }
}