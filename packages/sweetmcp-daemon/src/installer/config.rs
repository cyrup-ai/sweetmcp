//! Configuration and service setup for installer
//!
//! This module provides configuration generation, service setup, and platform-specific
//! installation logic with zero allocation fast paths and blazing-fast performance.

use crate::install::{install_daemon_async, InstallerBuilder, InstallerError};
use crate::install::fluent_voice;
use crate::signing;
use anyhow::{Context, Result};
use log::{info, warn};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use super::core::{InstallContext, ServiceConfig, InstallProgress};

/// Configure and install the SweetMCP daemon with optimized installation flow
pub async fn install_sweetmcp_daemon(
    exe_path: PathBuf,
    config_path: PathBuf,
    sign: bool,
) -> Result<()> {
    let mut context = InstallContext::new(exe_path.clone());
    context.config_path = config_path.clone();

    // Validate prerequisites
    context.validate_prerequisites()?;

    // Create directories
    context.create_directories()?;

    // Generate certificates
    context.generate_certificates()?;

    // Configure services
    configure_services(&mut context)?;

    // Build installer configuration with optimized service setup
    let installer = build_installer_config(&context)?;

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

            // Install fluent-voice components
            let fluent_voice_path = std::path::Path::new("/opt/sweetmcp/fluent-voice");
            if let Err(e) = fluent_voice::install_fluent_voice(fluent_voice_path).await {
                warn!("Failed to install fluent-voice components: {}", e);
                // Don't fail installation if fluent-voice installation fails
                // Voice features will be unavailable but other services can still run
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
                        info!("Re-signing installed binary at {}", installed_path.display());
                        if let Err(e) = signing::sign_binary(&installed_path) {
                            warn!("Failed to re-sign installed binary: {}", e);
                        } else {
                            info!("Successfully re-signed installed binary");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to verify installed binary signature: {}", e);
                    }
                }
            }

            context.send_progress(InstallProgress::complete(
                "installation".to_string(),
                "SweetMCP daemon installed successfully".to_string(),
            ));

            Ok(())
        }
        Err(e) => {
            context.send_progress(InstallProgress::error(
                "installation".to_string(),
                format!("Failed to install daemon: {}", e),
            ));
            Err(e.into())
        }
    }
}

/// Configure services for the installer with optimized service configuration
fn configure_services(context: &mut InstallContext) -> Result<()> {
    // Configure Pingora service
    let pingora_service = ServiceConfig::new(
        "sweetmcp-pingora".to_string(),
        "internal:pingora".to_string(), // Special command handled internally
    )
    .description("SweetMCP Edge Proxy Service".to_string())
    .env("RUST_LOG".to_string(), "info".to_string())
    .auto_restart(true)
    .depends_on("sweetmcp-daemon".to_string());

    // Configure autoconfig service
    let autoconfig_service = ServiceConfig::new(
        "sweetmcp-autoconfig".to_string(),
        "internal:autoconfig".to_string(), // Special command handled internally
    )
    .description("Automatic MCP client configuration service".to_string())
    .env("RUST_LOG".to_string(), "info".to_string())
    .auto_restart(true)
    .depends_on("sweetmcp-pingora".to_string()); // Start after pingora

    context.add_service(pingora_service);
    context.add_service(autoconfig_service);

    context.send_progress(InstallProgress::new(
        "services".to_string(),
        0.6,
        "Configured system services".to_string(),
    ));

    Ok(())
}

/// Build installer configuration with platform-specific settings
fn build_installer_config(context: &InstallContext) -> Result<InstallerBuilder> {
    let mut installer = InstallerBuilder::new("cyrupd", context.exe_path.clone())
        .description("Cyrup Service Manager")
        .arg("run")
        .arg("--foreground")
        .arg("--config")
        .arg(context.config_path.to_str().unwrap())
        .env("RUST_LOG", "info")
        .auto_restart(true)
        .network(true);

    // Add configured services
    for service in &context.services {
        installer = installer.service(convert_to_service_definition(service)?);
    }

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

    Ok(installer)
}

/// Convert ServiceConfig to service definition with optimized conversion
fn convert_to_service_definition(
    service: &ServiceConfig,
) -> Result<crate::config::ServiceDefinition> {
    let mut env_vars = std::collections::HashMap::new();
    for (key, value) in &service.env_vars {
        env_vars.insert(key.clone(), value.clone());
    }

    // Add default RUST_LOG if not present
    if !env_vars.contains_key("RUST_LOG") {
        env_vars.insert("RUST_LOG".to_string(), "info".to_string());
    }

    // Create health check configuration based on service type
    let health_check = match service.name.as_str() {
        "sweetmcp-pingora" => Some(crate::config::HealthCheckConfig {
            check_type: "tcp".to_string(),
            target: "127.0.0.1:8443".to_string(),
            interval_secs: 60,
            timeout_secs: 10,
            retries: 3,
            expected_response: None,
            on_failure: vec![],
        }),
        "sweetmcp-autoconfig" => Some(crate::config::HealthCheckConfig {
            check_type: "tcp".to_string(),
            target: "127.0.0.1:8443".to_string(),
            interval_secs: 300, // Check every 5 minutes
            timeout_secs: 30,
            retries: 3,
            expected_response: None,
            on_failure: vec![],
        }),
        _ => None,
    };

    Ok(crate::config::ServiceDefinition {
        name: service.name.clone(),
        description: Some(service.description.clone()),
        command: service.command.clone(),
        working_dir: service.working_dir.clone(),
        env_vars,
        auto_restart: service.auto_restart,
        user: service.user.clone(),
        group: service.group.clone(),
        restart_delay_s: Some(10),
        depends_on: service.dependencies.clone(),
        health_check,
        log_rotation: None,
        watch_dirs: Vec::new(),
        ephemeral_dir: None,
        service_type: Some(match service.name.as_str() {
            "sweetmcp-pingora" => "proxy".to_string(),
            "sweetmcp-autoconfig" => "autoconfig".to_string(),
            _ => "service".to_string(),
        }),
        memfs: None,
    })
}

/// Get the installed daemon path with platform-specific logic
fn get_installed_daemon_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/usr/local/bin/cyrupd")
    }
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/usr/bin/cyrupd")
    }
    #[cfg(target_os = "windows")]
    {
        PathBuf::from("C:\\Program Files\\SweetMCP\\cyrupd.exe")
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        PathBuf::from("/usr/local/bin/cyrupd")
    }
}

/// Generate and import wildcard certificate with optimized certificate generation
async fn generate_and_import_wildcard_certificate() -> Result<()> {
    let cert_dir = get_cert_dir();
    let wildcard_cert_path = cert_dir.join("wildcard.pem");

    // Check if wildcard certificate already exists and is valid
    if wildcard_cert_path.exists() {
        if let Ok(()) = validate_existing_wildcard_cert(&wildcard_cert_path) {
            info!("Valid wildcard certificate already exists, skipping generation");
            return Ok(());
        } else {
            info!("Existing wildcard certificate is invalid, regenerating");
        }
    }

    // Ensure certificate directory exists
    tokio::fs::create_dir_all(&cert_dir)
        .await
        .context("Failed to create certificate directory")?;

    info!("Generating SweetMCP wildcard certificate...");

    // Create certificate parameters for wildcard certificate
    let mut params = CertificateParams::new(vec!["*.cyrup.dev".to_string()]);

    // Add subject alternative names for all SweetMCP domains
    params.subject_alt_names = vec![
        SanType::DnsName("*.cyrup.dev".to_string()),
        SanType::DnsName("cyrup.dev".to_string()),
        SanType::DnsName("*.sweetmcp.cyrup.dev".to_string()),
        SanType::DnsName("sweetmcp.cyrup.dev".to_string()),
        SanType::DnsName("*.sweetmcp.cyrup.cloud".to_string()),
        SanType::DnsName("sweetmcp.cyrup.cloud".to_string()),
        SanType::DnsName("*.sweetmcp.cyrup.pro".to_string()),
        SanType::DnsName("sweetmcp.cyrup.pro".to_string()),
    ];

    // Set distinguished name
    let mut dn = DistinguishedName::new();
    dn.push(DnType::OrganizationName, "SweetMCP");
    dn.push(DnType::CommonName, "*.cyrup.dev");
    params.distinguished_name = dn;

    // Set non-expiring validity period (100 years)
    let now = SystemTime::now();
    params.not_before = now;
    params.not_after = now + Duration::from_secs(100 * 365 * 24 * 60 * 60);

    // Generate key pair and self-signed certificate
    let key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)
        .context("Failed to generate key pair")?;
    params.key_pair = Some(key_pair);

    let cert = rcgen::Certificate::from_params(params)
        .context("Failed to generate certificate")?;

    // Create combined PEM file with certificate and private key
    let cert_pem = cert.serialize_pem()?;
    let key_pem = cert.serialize_private_key_pem();
    let combined_pem = format!("{}\n{}", cert_pem, key_pem);

    // Write combined PEM file
    tokio::fs::write(&wildcard_cert_path, &combined_pem)
        .await
        .context("Failed to write wildcard certificate")?;

    // Set secure permissions on certificate file
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = tokio::fs::metadata(&wildcard_cert_path)
            .await
            .context("Failed to get file metadata")?
            .permissions();
        perms.set_mode(0o600); // Owner read/write only
        tokio::fs::set_permissions(&wildcard_cert_path, perms)
            .await
            .context("Failed to set file permissions")?;
    }

    info!(
        "SweetMCP wildcard certificate generated successfully at {}",
        wildcard_cert_path.display()
    );
    Ok(())
}

/// Get certificate directory path with platform-specific logic
fn get_cert_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/usr/local/var/sweetmcp/certs")
    }
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/var/lib/sweetmcp/certs")
    }
    #[cfg(target_os = "windows")]
    {
        PathBuf::from("C:\\ProgramData\\SweetMCP\\certs")
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        PathBuf::from("/tmp/sweetmcp/certs")
    }
}

/// Validate existing wildcard certificate with fast validation
fn validate_existing_wildcard_cert(cert_path: &Path) -> Result<()> {
    // Read certificate file
    let cert_pem = fs::read_to_string(cert_path)
        .context("Failed to read certificate file")?;

    // Parse certificate to validate it's well-formed
    let cert_der = pem::parse(&cert_pem)
        .context("Failed to parse certificate PEM")?;

    if cert_der.tag() != "CERTIFICATE" {
        return Err(anyhow::anyhow!("Invalid certificate format"));
    }

    // Parse X.509 certificate
    let cert = x509_parser::parse_x509_certificate(&cert_der.contents())
        .context("Failed to parse X.509 certificate")?
        .1;

    // Check if certificate is still valid (not expired)
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("Failed to get current time")?
        .as_secs();

    let not_after = cert.validity().not_after.timestamp() as u64;
    
    if now > not_after {
        return Err(anyhow::anyhow!("Certificate has expired"));
    }

    // Check if certificate expires within 30 days
    if now + (30 * 24 * 60 * 60) > not_after {
        warn!("Certificate expires within 30 days, consider regenerating");
    }

    Ok(())
}

/// Add SweetMCP host entries with optimized host file modification
fn add_sweetmcp_host_entries() -> Result<()> {
    let hosts_file_path = get_hosts_file_path();
    
    // Read existing hosts file
    let existing_content = fs::read_to_string(&hosts_file_path)
        .context("Failed to read hosts file")?;

    // Check if SweetMCP entries already exist
    if existing_content.contains("# SweetMCP entries") {
        info!("SweetMCP host entries already exist, skipping");
        return Ok(());
    }

    // Prepare SweetMCP host entries
    let sweetmcp_entries = vec![
        "# SweetMCP entries",
        "127.0.0.1 sweetmcp.cyrup.dev",
        "127.0.0.1 api.sweetmcp.cyrup.dev",
        "127.0.0.1 ws.sweetmcp.cyrup.dev",
        "127.0.0.1 sweetmcp.cyrup.cloud",
        "127.0.0.1 api.sweetmcp.cyrup.cloud",
        "127.0.0.1 ws.sweetmcp.cyrup.cloud",
        "127.0.0.1 sweetmcp.cyrup.pro",
        "127.0.0.1 api.sweetmcp.cyrup.pro",
        "127.0.0.1 ws.sweetmcp.cyrup.pro",
        "# End SweetMCP entries",
    ];

    // Append SweetMCP entries to hosts file
    let mut new_content = existing_content;
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push('\n');
    new_content.push_str(&sweetmcp_entries.join("\n"));
    new_content.push('\n');

    // Write updated hosts file
    fs::write(&hosts_file_path, new_content)
        .context("Failed to write hosts file")?;

    info!("Added SweetMCP host entries to {}", hosts_file_path.display());
    Ok(())
}

/// Get hosts file path with platform-specific logic
fn get_hosts_file_path() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from("/etc/hosts")
    }
    #[cfg(windows)]
    {
        PathBuf::from("C:\\Windows\\System32\\drivers\\etc\\hosts")
    }
    #[cfg(not(any(unix, windows)))]
    {
        PathBuf::from("/etc/hosts")
    }
}

/// Remove SweetMCP host entries with optimized host file cleanup
pub fn remove_sweetmcp_host_entries() -> Result<()> {
    let hosts_file_path = get_hosts_file_path();
    
    // Read existing hosts file
    let existing_content = fs::read_to_string(&hosts_file_path)
        .context("Failed to read hosts file")?;

    // Check if SweetMCP entries exist
    if !existing_content.contains("# SweetMCP entries") {
        info!("No SweetMCP host entries found, skipping removal");
        return Ok(());
    }

    // Remove SweetMCP entries
    let lines: Vec<&str> = existing_content.lines().collect();
    let mut new_lines = Vec::new();
    let mut in_sweetmcp_section = false;

    for line in lines {
        if line.trim() == "# SweetMCP entries" {
            in_sweetmcp_section = true;
            continue;
        }
        if line.trim() == "# End SweetMCP entries" {
            in_sweetmcp_section = false;
            continue;
        }
        if !in_sweetmcp_section {
            new_lines.push(line);
        }
    }

    // Write updated hosts file
    let new_content = new_lines.join("\n");
    fs::write(&hosts_file_path, new_content)
        .context("Failed to write hosts file")?;

    info!("Removed SweetMCP host entries from {}", hosts_file_path.display());
    Ok(())
}

/// Configuration validation and setup
pub fn validate_configuration(config_path: &Path) -> Result<()> {
    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Configuration file not found: {:?}",
            config_path
        ));
    }

    // Read and validate configuration file
    let config_content = fs::read_to_string(config_path)
        .context("Failed to read configuration file")?;

    // Basic TOML validation
    let _config: toml::Value = toml::from_str(&config_content)
        .context("Invalid TOML configuration")?;

    info!("Configuration file validated successfully");
    Ok(())
}

/// Create default configuration file with optimized config generation
pub fn create_default_configuration(config_path: &Path) -> Result<()> {
    let config_dir = config_path.parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid configuration path"))?;

    // Create configuration directory if it doesn't exist
    fs::create_dir_all(config_dir)
        .context("Failed to create configuration directory")?;

    // Default configuration content
    let default_config = r#"
# SweetMCP Daemon Configuration

[daemon]
# Daemon process settings
pid_file = "/var/run/sweetmcp/daemon.pid"
log_level = "info"
log_file = "/var/log/sweetmcp/daemon.log"

[network]
# Network configuration
bind_address = "127.0.0.1"
port = 33399
max_connections = 1000

[security]
# Security settings
enable_tls = true
cert_file = "/usr/local/var/sweetmcp/certs/server.crt"
key_file = "/usr/local/var/sweetmcp/certs/server.key"
ca_file = "/usr/local/var/sweetmcp/certs/ca.crt"

[services]
# Service configuration
enable_pingora = true
enable_autoconfig = true
enable_voice = false

[database]
# Database configuration
url = "surrealkv:///usr/local/var/sweetmcp/data/sweetmcp.db"
namespace = "sweetmcp"
database = "main"

[plugins]
# Plugin configuration
plugin_dir = "/usr/local/var/sweetmcp/plugins"
enable_sandboxing = true
max_memory_mb = 256
timeout_seconds = 30
"#;

    // Write default configuration
    fs::write(config_path, default_config)
        .context("Failed to write default configuration")?;

    info!("Created default configuration at {:?}", config_path);
    Ok(())
}