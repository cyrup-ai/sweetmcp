use crate::install::{install_daemon_async, uninstall_daemon, InstallerBuilder, InstallerError};
use crate::signing;
use anyhow::{Context, Result};
use log::{info, warn};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

/// Install the daemon using elevated_daemon_installer with GUI authorization.
pub async fn install(dry: bool, sign: bool, identity: Option<String>) -> Result<()> {
    install_async_impl(dry, sign, identity).await
}

/// Synchronous installation fallback for non-async contexts
pub fn install_sync(dry: bool, _sign: bool, _identity: Option<String>) -> Result<()> {
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

    // Get the current executable path and validate it exists
    let exe_path = std::env::current_exe().context("current_exe()")?;
    if !exe_path.exists() {
        return Err(crate::install::InstallerError::MissingExecutable(
            exe_path.to_string_lossy().to_string()
        ).into());
    }

    if dry {
        info!("[dry run] Would install daemon:");
        info!("  - Binary: {}", exe_path.display());
        info!("  - Config: {}", config_path.display());
        info!("  - Service: cyrupd");
        return Ok(());
    }

    // Build the installer configuration using both arg and args methods
    let installer = InstallerBuilder::new("cyrupd", exe_path)
        .description("Cyrup Service Manager")
        .args(["run", "--foreground", "--config", config_path.to_str().unwrap()])
        .env("RUST_LOG", "info")
        .auto_restart(true)
        .network(true);

    // Install using synchronous installer
    match crate::install::install_daemon(installer) {
        Ok(()) => {
            info!("Daemon installed successfully (sync mode)");
            Ok(())
        }
        Err(crate::install::InstallerError::Cancelled) => Err(anyhow::anyhow!("Installation cancelled by user")),
        Err(crate::install::InstallerError::PermissionDenied) => Err(anyhow::anyhow!(
            "Permission denied. Please provide administrator credentials."
        )),
        Err(e) => Err(e.into()),
    }
}

/// Async implementation of installation
async fn install_async_impl(dry: bool, sign: bool, identity: Option<String>) -> Result<()> {
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

    // Sign the binary BEFORE installation if requested
    if sign && !dry {
        info!("Signing daemon binary before installation...");

        let mut sign_config = signing::SigningConfig::new(exe_path.clone());

        // Override identity if provided
        if let Some(id) = identity.clone() {
            match &mut sign_config.platform {
                #[cfg(target_os = "macos")]
                signing::PlatformConfig::MacOS { identity, .. } => *identity = id,
                #[cfg(target_os = "windows")]
                signing::PlatformConfig::Windows { certificate, .. } => *certificate = id,
                #[cfg(target_os = "linux")]
                signing::PlatformConfig::Linux { key_id, .. } => *key_id = Some(id),
                _ => {}
            }
        }

        // Sign the binary
        signing::sign_binary(&sign_config).context("Failed to sign daemon binary")?;

        info!("Successfully signed daemon binary");
    }

    if dry {
        info!("[dry run] Would install daemon:");
        info!("  - Binary: {}", exe_path.display());
        info!("  - Config: {}", config_path.display());
        info!("  - Service: cyrupd");
        if sign {
            #[cfg(target_os = "macos")]
            info!(
                "  - macOS Codesign: {}",
                identity.as_deref().unwrap_or("Developer ID or ad-hoc")
            );
            #[cfg(target_os = "windows")]
            info!(
                "  - Windows Authenticode: {}",
                identity.as_deref().unwrap_or("Auto-select certificate")
            );
            #[cfg(target_os = "linux")]
            info!(
                "  - Linux GPG Sign: {}",
                identity.as_deref().unwrap_or("Default GPG key")
            );
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
            env.insert(
                "SWEETMCP_UDS_PATH".to_string(),
                "/run/sugora.sock".to_string(),
            );
            env.insert(
                "SWEETMCP_METRICS_BIND".to_string(),
                "127.0.0.1:9090".to_string(),
            );
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
        service_type: None,
        memfs: None,
    };

    // Create the autoconfig service definition
    let autoconfig_service = crate::config::ServiceDefinition {
        name: "sweetmcp-autoconfig".to_string(),
        description: Some("Automatic MCP client configuration service".to_string()),
        command: "internal:autoconfig".to_string(), // Special command handled internally
        working_dir: None,
        env_vars: {
            let mut env = std::collections::HashMap::new();
            env.insert("RUST_LOG".to_string(), "info".to_string());
            env
        },
        auto_restart: true,
        user: None, // Run as same user as daemon
        group: None,
        restart_delay_s: Some(10),
        depends_on: vec!["sweetmcp-pingora".to_string()], // Start after pingora
        health_check: Some(crate::config::HealthCheckConfig {
            check_type: "tcp".to_string(),
            target: "127.0.0.1:8443".to_string(),
            interval_secs: 300, // Check every 5 minutes
            timeout_secs: 30,
            retries: 3,
            expected_response: None,
            on_failure: vec![],
        }),
        log_rotation: None,
        watch_dirs: Vec::new(),
        ephemeral_dir: None,
        service_type: Some("autoconfig".to_string()),
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
        .service(pingora_service)
        .service(autoconfig_service);

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
    let installer = installer.user("root").group("wheel");

    // Install the daemon with GUI authorization
    match install_daemon_async(installer).await {
        Ok(()) => {
            info!("Daemon installed successfully");

            // Generate wildcard certificate and import to trust store
            if let Err(e) = generate_and_import_wildcard_certificate().await {
                warn!("Failed to generate wildcard certificate and import: {}", e);
                // Don't fail installation if certificate import fails
            }

            // Add host entries for all SweetMCP domains pointing to 127.0.0.1
            if let Err(e) = add_sweetmcp_host_entries() {
                warn!("Failed to add SweetMCP host entries: {}", e);
                // Don't fail installation if host entries fail
            }

            // Verify the installed binary is still signed
            if sign {
                let installed_path = get_installed_daemon_path();
                match signing::verify_signature(&installed_path) {
                    Ok(true) => info!(
                        "Installed binary signature verified on {}",
                        installed_path.display()
                    ),
                    Ok(false) => {
                        warn!("Installed binary lost its signature during installation");
                        // Re-sign the installed binary
                        info!("Re-signing installed binary...");
                        let mut resign_config = signing::SigningConfig::new(installed_path.clone());
                        if let Some(id) = identity {
                            match &mut resign_config.platform {
                                #[cfg(target_os = "macos")]
                                signing::PlatformConfig::MacOS { identity, .. } => *identity = id,
                                #[cfg(target_os = "windows")]
                                signing::PlatformConfig::Windows { certificate, .. } => {
                                    *certificate = id
                                }
                                #[cfg(target_os = "linux")]
                                signing::PlatformConfig::Linux { key_id, .. } => *key_id = Some(id),
                                _ => {}
                            }
                        }
                        if let Err(e) = signing::sign_binary(&resign_config) {
                            warn!("Failed to re-sign installed binary: {}", e);
                        } else {
                            info!("Successfully re-signed installed binary");
                        }
                    }
                    Err(e) => warn!("Failed to verify installed binary signature: {}", e),
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

/// Async uninstallation for contexts requiring async operations
pub async fn uninstall_async(dry: bool) -> Result<()> {
    if dry {
        info!("[dry run] Would uninstall daemon: cyrupd (async)");
        return Ok(());
    }

    match crate::install::uninstall_daemon_async("cyrupd").await {
        Ok(()) => {
            info!("Daemon uninstalled successfully (async mode)");
            Ok(())
        }
        Err(InstallerError::Cancelled) => Err(anyhow::anyhow!("Uninstallation cancelled by user")),
        Err(InstallerError::PermissionDenied) => Err(anyhow::anyhow!(
            "Permission denied. Please provide administrator credentials."
        )),
        Err(e) => Err(e.into()),
    }
}

/// Generate wildcard certificate and import to system trust store
async fn generate_and_import_wildcard_certificate() -> Result<()> {
    info!("Generating wildcard certificate and importing to system trust store");

    // Get XDG config directory
    let xdg_config_home = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine XDG config directory"))?;

    // Generate wildcard certificate
    generate_sweetmcp_wildcard_certificate(&xdg_config_home)
        .await
        .context("Failed to generate wildcard certificate")?;

    // Import certificate to system trust store
    let wildcard_cert_path = xdg_config_home.join("sweetmcp").join("wildcard.cyrup.pem");
    import_wildcard_certificate_to_system(&wildcard_cert_path)
}

/// Generate SweetMCP wildcard certificate with multiple domain SAN entries
async fn generate_sweetmcp_wildcard_certificate(xdg_config_home: &Path) -> Result<()> {
    let cert_dir = xdg_config_home.join("sweetmcp");

    // Create cert directory if it doesn't exist
    #[cfg(feature = "runtime")]
    tokio::fs::create_dir_all(&cert_dir)
        .await
        .context("Failed to create certificate directory")?;

    #[cfg(not(feature = "runtime"))]
    fs::create_dir_all(&cert_dir).context("Failed to create certificate directory")?;

    let wildcard_cert_path = cert_dir.join("wildcard.cyrup.pem");

    // Check if certificate already exists and is valid
    if wildcard_cert_path.exists() {
        if validate_existing_wildcard_cert(&wildcard_cert_path).is_ok() {
            info!(
                "Valid wildcard certificate already exists at {}",
                wildcard_cert_path.display()
            );
            return Ok(());
        }
        info!("Existing wildcard certificate is invalid, regenerating...");
    }

    info!("Generating new SweetMCP wildcard certificate with multiple SAN entries");

    let mut params =
        CertificateParams::new(Vec::default()).context("Failed to create certificate params")?;

    // Set as non-CA certificate
    params.is_ca = rcgen::IsCa::NoCa;

    // SweetMCP domains with wildcards for all domains
    params.subject_alt_names = vec![
        // Wildcard entries
        SanType::DnsName(
            "*.cyrup.dev"
                .try_into()
                .context("Invalid DNS name: *.cyrup.dev")?,
        ),
        SanType::DnsName(
            "*.cyrup.ai"
                .try_into()
                .context("Invalid DNS name: *.cyrup.ai")?,
        ),
        SanType::DnsName(
            "*.cyrup.cloud"
                .try_into()
                .context("Invalid DNS name: *.cyrup.cloud")?,
        ),
        SanType::DnsName(
            "*.cyrup.pro"
                .try_into()
                .context("Invalid DNS name: *.cyrup.pro")?,
        ),
        // Base domains
        SanType::DnsName(
            "cyrup.dev"
                .try_into()
                .context("Invalid DNS name: cyrup.dev")?,
        ),
        SanType::DnsName(
            "cyrup.ai"
                .try_into()
                .context("Invalid DNS name: cyrup.ai")?,
        ),
        SanType::DnsName(
            "cyrup.cloud"
                .try_into()
                .context("Invalid DNS name: cyrup.cloud")?,
        ),
        SanType::DnsName(
            "cyrup.pro"
                .try_into()
                .context("Invalid DNS name: cyrup.pro")?,
        ),
        // Specific SweetMCP entries (covered by wildcards but explicit for clarity)
        SanType::DnsName(
            "sweetmcp.cyrup.dev"
                .try_into()
                .context("Invalid DNS name: sweetmcp.cyrup.dev")?,
        ),
        SanType::DnsName(
            "sweetmcp.cyrup.ai"
                .try_into()
                .context("Invalid DNS name: sweetmcp.cyrup.ai")?,
        ),
        SanType::DnsName(
            "sweetmcp.cyrup.cloud"
                .try_into()
                .context("Invalid DNS name: sweetmcp.cyrup.cloud")?,
        ),
        SanType::DnsName(
            "sweetmcp.cyrup.pro"
                .try_into()
                .context("Invalid DNS name: sweetmcp.cyrup.pro")?,
        ),
    ];

    // Set distinguished name
    let mut dn = DistinguishedName::new();
    dn.push(DnType::OrganizationName, "SweetMCP");
    dn.push(DnType::CommonName, "*.cyrup.dev");
    params.distinguished_name = dn;

    // Set non-expiring validity period (100 years)
    let now = SystemTime::now();
    params.not_before = now.into();
    params.not_after = (now + Duration::from_secs(100 * 365 * 24 * 60 * 60)).into();

    // Generate key pair and self-signed certificate
    let key_pair = KeyPair::generate().context("Failed to generate key pair")?;

    let cert = params
        .self_signed(&key_pair)
        .context("Failed to generate certificate")?;

    // Create combined PEM file with certificate and private key
    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();
    let combined_pem = format!("{}\n{}", cert_pem, key_pem);

    // Write combined PEM file
    #[cfg(feature = "runtime")]
    tokio::fs::write(&wildcard_cert_path, &combined_pem)
        .await
        .context("Failed to write wildcard certificate")?;

    #[cfg(not(feature = "runtime"))]
    fs::write(&wildcard_cert_path, &combined_pem)
        .context("Failed to write wildcard certificate")?;

    // Set secure permissions on certificate file
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        #[cfg(feature = "runtime")]
        {
            let mut perms = tokio::fs::metadata(&wildcard_cert_path)
                .await
                .context("Failed to get file metadata")?
                .permissions();
            perms.set_mode(0o600); // Owner read/write only
            tokio::fs::set_permissions(&wildcard_cert_path, perms)
                .await
                .context("Failed to set file permissions")?;
        }

        #[cfg(not(feature = "runtime"))]
        {
            let mut perms = fs::metadata(&wildcard_cert_path)
                .context("Failed to get file metadata")?
                .permissions();
            perms.set_mode(0o600); // Owner read/write only
            fs::set_permissions(&wildcard_cert_path, perms)
                .context("Failed to set file permissions")?;
        }
    }

    info!(
        "SweetMCP wildcard certificate generated successfully at {}",
        wildcard_cert_path.display()
    );
    Ok(())
}

/// Validate existing wildcard certificate
fn validate_existing_wildcard_cert(cert_path: &Path) -> Result<()> {
    let cert_content = fs::read_to_string(cert_path).context("Failed to read certificate file")?;

    // Basic validation - check if it contains the expected domains
    let required_domains = [
        "sweetmcp.cyrup.dev",
        "sweetmcp.cyrup.ai",
        "sweetmcp.cyrup.cloud",
        "sweetmcp.cyrup.pro",
    ];

    for domain in &required_domains {
        if !cert_content.contains(domain) {
            return Err(anyhow::anyhow!("Missing required domain: {}", domain));
        }
    }

    // Check if it has both certificate and private key
    if !cert_content.contains("-----BEGIN CERTIFICATE-----")
        || !cert_content.contains("-----BEGIN PRIVATE KEY-----")
    {
        return Err(anyhow::anyhow!(
            "Invalid certificate format - missing certificate or private key"
        ));
    }

    Ok(())
}

/// Import wildcard certificate to system trust store
fn import_wildcard_certificate_to_system(cert_path: &Path) -> Result<()> {
    let cert_str = cert_path.to_string_lossy();

    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            import_wildcard_certificate_macos(&cert_str)
        } else if #[cfg(target_os = "linux")] {
            import_wildcard_certificate_linux(&cert_str)
        } else {
            warn!("Wildcard certificate import not supported on this platform");
            Ok(())
        }
    }
}

/// Import wildcard certificate on macOS using security command
#[cfg(target_os = "macos")]
fn import_wildcard_certificate_macos(cert_path: &str) -> Result<()> {
    info!("Importing SweetMCP wildcard certificate to macOS System keychain");

    // Extract just the certificate part from the combined PEM file
    let cert_content =
        std::fs::read_to_string(cert_path).context("Failed to read certificate file")?;

    // Find the certificate part (everything before the private key)
    let cert_part = if let Some(key_start) = cert_content.find("-----BEGIN PRIVATE KEY-----") {
        &cert_content[..key_start]
    } else {
        &cert_content
    };

    // Write certificate part to temporary file
    let temp_cert_path = format!("{}.cert", cert_path);
    std::fs::write(&temp_cert_path, cert_part.trim())
        .context("Failed to write temporary certificate file")?;

    // Import to System keychain with trust settings for SSL
    let import_output = Command::new("security")
        .args(&[
            "add-trusted-cert",
            "-d", // Add to admin cert store
            "-r",
            "trustRoot", // Trust as root certificate
            "-k",
            "/Library/Keychains/System.keychain",
            &temp_cert_path,
        ])
        .output()
        .context("Failed to execute security add-trusted-cert")?;

    // Clean up temporary file
    let _ = std::fs::remove_file(&temp_cert_path);

    if !import_output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to import wildcard certificate: {}",
            String::from_utf8_lossy(&import_output.stderr)
        ));
    }

    info!("SweetMCP wildcard certificate imported to macOS System keychain");
    Ok(())
}

/// Import wildcard certificate on Linux
#[cfg(target_os = "linux")]
fn import_wildcard_certificate_linux(cert_path: &str) -> Result<()> {
    info!("Importing SweetMCP wildcard certificate to Linux certificate store");

    // Extract just the certificate part from the combined PEM file
    let cert_content =
        std::fs::read_to_string(cert_path).context("Failed to read certificate file")?;

    // Find the certificate part (everything before the private key)
    let cert_part = if let Some(key_start) = cert_content.find("-----BEGIN PRIVATE KEY-----") {
        &cert_content[..key_start]
    } else {
        &cert_content
    };

    // Copy to system certificate directory
    let cert_dest = "/usr/local/share/ca-certificates/sweetmcp-wildcard.crt";

    // Ensure directory exists
    Command::new("mkdir")
        .args(&["-p", "/usr/local/share/ca-certificates"])
        .output()
        .context("Failed to create certificate directory")?;

    // Write certificate part to system directory
    std::fs::write(cert_dest, cert_part.trim())
        .context("Failed to write wildcard certificate to system directory")?;

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

    info!("SweetMCP wildcard certificate imported to Linux certificate store");
    Ok(())
}

/// Add host entries for all SweetMCP domains pointing to 127.0.0.1
fn add_sweetmcp_host_entries() -> Result<()> {
    info!("Adding SweetMCP host entries pointing to 127.0.0.1");

    let hosts_file = if cfg!(target_os = "windows") {
        "C:\\Windows\\System32\\drivers\\etc\\hosts"
    } else {
        "/etc/hosts"
    };

    // Read current hosts file
    let current_hosts = fs::read_to_string(hosts_file).context("Failed to read hosts file")?;

    let sweetmcp_domains = [
        "sweetmcp.cyrup.dev",
        "sweetmcp.cyrup.ai",
        "sweetmcp.cyrup.cloud",
        "sweetmcp.cyrup.pro",
    ];

    let mut new_entries = Vec::new();
    let mut entries_added = false;

    // Check which entries need to be added
    for domain in &sweetmcp_domains {
        if !current_hosts.contains(domain) {
            new_entries.push(format!("127.0.0.1 {}", domain));
            entries_added = true;
        } else {
            info!("{} entry already exists in hosts file", domain);
        }
    }

    if !entries_added {
        info!("All SweetMCP host entries already exist");
        return Ok(());
    }

    // Append new entries
    let mut updated_hosts = current_hosts;
    if !updated_hosts.ends_with('\n') {
        updated_hosts.push('\n');
    }
    updated_hosts.push_str("\n# SweetMCP Auto-Integration\n");
    for entry in &new_entries {
        updated_hosts.push_str(&format!("{}\n", entry));
    }

    // Write updated hosts file
    fs::write(hosts_file, updated_hosts).context("Failed to write hosts file")?;

    info!(
        "Successfully added {} SweetMCP host entries",
        new_entries.len()
    );
    Ok(())
}

/// Get the installed daemon path for the current platform
fn get_installed_daemon_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Windows installs to Program Files or System32
        PathBuf::from("C:\\Program Files\\Cyrupd\\cyrupd.exe")
    }

    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/usr/local/bin/cyrupd")
    }

    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/usr/local/bin/cyrupd")
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        PathBuf::from("/usr/local/bin/cyrupd")
    }
}
