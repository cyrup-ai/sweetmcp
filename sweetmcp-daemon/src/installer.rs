use crate::install::{install_daemon, uninstall_daemon, InstallerBuilder, InstallerError};
use anyhow::{Context, Result};
use log::{info, warn};
use std::fs;
use std::path::Path;
use std::process::Command;

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
            
            // Initialize TLS manager and import CA certificate
            if let Err(e) = initialize_and_import_ca_certificate() {
                warn!("Failed to initialize TLS and import CA certificate: {}", e);
                // Don't fail installation if certificate import fails
            }
            
            // Add host entry for mcp.cyrup.dev pointing to 1.1.1.1
            if let Err(e) = add_mcp_host_entry() {
                warn!("Failed to add mcp.cyrup.dev host entry: {}", e);
                // Don't fail installation if host entry fails
            }

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

/// Initialize TLS manager and import the generated CA certificate to system trust store
fn initialize_and_import_ca_certificate() -> Result<()> {
    use std::path::PathBuf;
    
    info!("Initializing TLS manager and importing CA certificate");
    
    // Use sweetmcp TLS manager to generate/load certificates
    let cert_dir = PathBuf::from("/etc/cyrupd/tls");
    
    // Create cert directory
    fs::create_dir_all(&cert_dir)
        .context("Failed to create certificate directory")?;
    
    // For now, we'll generate a simple self-signed CA certificate
    // The full TLS manager requires async context, so we'll do a simpler approach
    let ca_cert_path = cert_dir.join("ca.crt");
    
    if !ca_cert_path.exists() {
        info!("Generating self-signed CA certificate");
        generate_simple_ca_certificate(&ca_cert_path)?;
    }
    
    // Import CA certificate to system trust store
    import_ca_certificate_to_system(&ca_cert_path)
}

/// Generate a simple self-signed CA certificate using openssl
fn generate_simple_ca_certificate(ca_cert_path: &Path) -> Result<()> {
    let ca_key_path = ca_cert_path.with_extension("key");
    
    // Generate private key
    let key_output = Command::new("openssl")
        .args(&[
            "genpkey",
            "-algorithm", "RSA",
            "-out", &ca_key_path.to_string_lossy(),
            "-pkcs8",
            "-aes256",
            "-pass", "pass:sweetmcp"
        ])
        .output()
        .context("Failed to generate CA private key")?;
    
    if !key_output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to generate CA key: {}",
            String::from_utf8_lossy(&key_output.stderr)
        ));
    }
    
    // Generate self-signed certificate
    let cert_output = Command::new("openssl")
        .args(&[
            "req",
            "-new",
            "-x509",
            "-key", &ca_key_path.to_string_lossy(),
            "-out", &ca_cert_path.to_string_lossy(),
            "-days", "3650",
            "-passin", "pass:sweetmcp",
            "-subj", "/CN=SweetMCP CA/O=Cyrup/C=US"
        ])
        .output()
        .context("Failed to generate CA certificate")?;
    
    if !cert_output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to generate CA certificate: {}",
            String::from_utf8_lossy(&cert_output.stderr)
        ));
    }
    
    info!("Generated self-signed CA certificate");
    Ok(())
}

/// Import CA certificate to system trust store
fn import_ca_certificate_to_system(ca_cert_path: &Path) -> Result<()> {
    let ca_cert_str = ca_cert_path.to_string_lossy();
    
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            import_ca_certificate_macos(&ca_cert_str)
        } else if #[cfg(target_os = "linux")] {
            import_ca_certificate_linux(&ca_cert_str)
        } else {
            warn!("CA certificate import not supported on this platform");
            Ok(())
        }
    }
}

/// Import CA certificate on macOS using security command
#[cfg(target_os = "macos")]
fn import_ca_certificate_macos(ca_cert_path: &str) -> Result<()> {
    info!("Importing CA certificate to macOS System keychain");
    
    // Import to System keychain with trust settings for SSL
    let import_output = Command::new("security")
        .args(&[
            "add-trusted-cert",
            "-d",              // Add to admin cert store
            "-r", "trustRoot", // Trust as root certificate
            "-p", "ssl",       // Trust for SSL
            "-p", "smime",     // Trust for S/MIME
            "-k", "/Library/Keychains/System.keychain",
            ca_cert_path
        ])
        .output()
        .context("Failed to execute security add-trusted-cert")?;
    
    if !import_output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to import CA certificate: {}",
            String::from_utf8_lossy(&import_output.stderr)
        ));
    }
    
    info!("CA certificate imported to macOS System keychain");
    Ok(())
}

/// Import CA certificate on Linux
#[cfg(target_os = "linux")]
fn import_ca_certificate_linux(ca_cert_path: &str) -> Result<()> {
    info!("Importing CA certificate to Linux certificate store");
    
    // Copy to system certificate directory
    let cert_dest = "/usr/local/share/ca-certificates/sweetmcp-ca.crt";
    
    // Ensure directory exists
    Command::new("mkdir")
        .args(&["-p", "/usr/local/share/ca-certificates"])
        .output()
        .context("Failed to create certificate directory")?;
    
    // Copy certificate
    fs::copy(ca_cert_path, cert_dest)
        .context("Failed to copy CA certificate to system directory")?;
    
    // Update certificate store
    let update_output = Command::new("update-ca-certificates")
        .output()
        .context("Failed to execute update-ca-certificates")?;
    
    if !update_output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to update certificate store: {}",
            String::from_utf8_lossy(&update_output.stderr)
        ));
    }
    
    info!("CA certificate imported to Linux certificate store");
    Ok(())
}

/// Add host entry for mcp.cyrup.dev pointing to 1.1.1.1
fn add_mcp_host_entry() -> Result<()> {
    info!("Adding host entry: 1.1.1.1 mcp.cyrup.dev");
    
    let hosts_file = if cfg!(target_os = "windows") {
        "C:\\Windows\\System32\\drivers\\etc\\hosts"
    } else {
        "/etc/hosts"
    };
    
    // Read current hosts file
    let current_hosts = fs::read_to_string(hosts_file)
        .context("Failed to read hosts file")?;
    
    // Check if entry already exists
    if current_hosts.contains("mcp.cyrup.dev") {
        info!("mcp.cyrup.dev entry already exists in hosts file");
        return Ok(());
    }
    
    // Append new entry
    let new_entry = "\n# SweetMCP\n1.1.1.1 mcp.cyrup.dev\n";
    let updated_hosts = format!("{}{}", current_hosts, new_entry);
    
    // Write updated hosts file
    fs::write(hosts_file, updated_hosts)
        .context("Failed to write hosts file")?;
    
    info!("Successfully added mcp.cyrup.dev host entry");
    Ok(())
}
