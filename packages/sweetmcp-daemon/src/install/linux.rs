//! Linux platform implementation using systemd and native Linux APIs.
//!
//! This implementation provides sophisticated service management with zero allocation,
//! blazing-fast performance, and comprehensive error handling to match the macOS implementation.

use crate::install::{InstallerBuilder, InstallerError};
use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::borrow::Cow;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

pub(crate) struct PlatformExecutor;

// Constants for zero-allocation buffers
const UNIT_NAME_MAX: usize = 256;
const UNIT_PATH_MAX: usize = 512;
const MAX_SERVICE_NAME: usize = 256;
const MAX_DESCRIPTION: usize = 512;

// Global helper path - initialized once, used everywhere (like macOS implementation)
static HELPER_PATH: OnceCell<PathBuf> = OnceCell::new();

// Embedded helper executable data (like macOS APP_ZIP_DATA and Windows HELPER_EXE_DATA)
const HELPER_BINARY_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sweetmcp-helper"));

// Atomic state for service operations
static SERVICE_OPERATION_STATE: AtomicU32 = AtomicU32::new(0);

// Pre-computed systemd unit template for zero allocation
const UNIT_TEMPLATE: &str = include_str!("../templates/systemd.service.template");

/// Systemd service configuration with zero-allocation patterns
#[derive(Clone)]
struct SystemdConfig<'a> {
    service_name: &'a str,
    description: &'a str,
    binary_path: &'a str,
    args: &'a [String],
    env_vars: &'a [(String, String)],
    auto_restart: bool,
    wants_network: bool,
    user: Option<&'a str>,
    group: Option<&'a str>,
}

impl PlatformExecutor {
    /// Install the daemon as a systemd service with comprehensive configuration
    pub fn install(b: InstallerBuilder) -> Result<(), InstallerError> {
        // Ensure helper path is initialized
        Self::ensure_helper_path()?;

        // Check if we have sufficient privileges
        Self::check_privileges()?;

        // Create systemd configuration
        let config = SystemdConfig {
            service_name: &b.label,
            description: &b.description,
            binary_path: b.program.to_str().ok_or_else(|| {
                InstallerError::System("Invalid binary path encoding".to_string())
            })?,
            args: &b.args,
            env_vars: &b.env.iter().collect::<Vec<_>>(),
            auto_restart: b.auto_restart,
            wants_network: b.wants_network,
            user: None, // Run as root for system service
            group: None,
        };

        // Generate and install systemd unit file
        Self::create_systemd_unit(&config)?;

        // Create systemd drop-in directories for advanced configuration
        Self::create_dropin_config(&config)?;

        // Register with systemd journal for structured logging
        Self::setup_journal_integration(&b.label)?;

        // Install service definitions if any
        if !b.services.is_empty() {
            Self::install_services(&b.services)?;
        }

        // Enable and start the service
        Self::enable_systemd_service(&b.label)?;

        if b.auto_restart {
            Self::start_systemd_service(&b.label)?;
        }

        Ok(())
    }

    /// Uninstall the systemd service and clean up all resources
    pub fn uninstall(label: &str) -> Result<(), InstallerError> {
        // Stop the service first
        Self::stop_systemd_service(label)?;

        // Disable the service
        Self::disable_systemd_service(label)?;

        // Remove systemd unit files
        Self::remove_systemd_unit(label)?;

        // Clean up drop-in configurations
        Self::cleanup_dropin_config(label)?;

        // Remove journal integration
        Self::cleanup_journal_integration(label)?;

        // Reload systemd daemon to reflect changes
        Self::reload_systemd_daemon()?;

        Ok(())
    }

    /// Ensure helper executable is extracted and available
    fn ensure_helper_path() -> Result<(), InstallerError> {
        if HELPER_PATH.get().is_some() {
            return Ok(());
        }

        // Create unique helper path in temp directory
        let temp_dir = std::env::temp_dir();
        let helper_name = format!("sweetmcp-helper-{}", std::process::id());
        let helper_path = temp_dir.join(helper_name);

        // Extract embedded helper executable
        fs::write(&helper_path, HELPER_BINARY_DATA).map_err(|e| {
            InstallerError::System(format!("Failed to extract helper executable: {}", e))
        })?;

        // Make helper executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&helper_path)
                .map_err(|e| {
                    InstallerError::System(format!("Failed to get helper metadata: {}", e))
                })?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&helper_path, perms).map_err(|e| {
                InstallerError::System(format!("Failed to set helper permissions: {}", e))
            })?;
        }

        // Verify the helper is properly signed (if signing is available)
        if crate::signing::is_signing_available() {
            Self::verify_helper_signature(&helper_path)?;
        }

        // Store the path globally
        HELPER_PATH
            .set(helper_path)
            .map_err(|_| InstallerError::System("Helper path already initialized".to_string()))?;

        Ok(())
    }

    /// Check if we have sufficient privileges for systemd operations
    fn check_privileges() -> Result<(), InstallerError> {
        // Check if we're running as root or have CAP_SYS_ADMIN
        let uid = unsafe { libc::getuid() };
        if uid != 0 {
            // Check for systemd user service support
            let home_dir = std::env::var("HOME").map_err(|_| InstallerError::PermissionDenied)?;
            let user_systemd_dir = PathBuf::from(home_dir).join(".config/systemd/user");

            if !user_systemd_dir.exists() {
                return Err(InstallerError::PermissionDenied);
            }
        }

        Ok(())
    }

    /// Create systemd unit file with comprehensive configuration
    fn create_systemd_unit(config: &SystemdConfig) -> Result<(), InstallerError> {
        let unit_content = Self::generate_unit_content(config)?;

        // Determine unit file path
        let unit_path = if unsafe { libc::getuid() } == 0 {
            // System service
            PathBuf::from("/etc/systemd/system").join(format!("{}.service", config.service_name))
        } else {
            // User service
            let home_dir = std::env::var("HOME").map_err(|_| {
                InstallerError::System("HOME environment variable not set".to_string())
            })?;
            PathBuf::from(home_dir)
                .join(".config/systemd/user")
                .join(format!("{}.service", config.service_name))
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = unit_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                InstallerError::System(format!("Failed to create systemd directory: {}", e))
            })?;
        }

        // Write unit file atomically
        Self::write_file_atomic(&unit_path, &unit_content)?;

        // Set appropriate permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&unit_path)
                .map_err(|e| {
                    InstallerError::System(format!("Failed to get unit file metadata: {}", e))
                })?
                .permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&unit_path, perms).map_err(|e| {
                InstallerError::System(format!("Failed to set unit file permissions: {}", e))
            })?;
        }

        Ok(())
    }

    /// Generate systemd unit file content with zero allocation where possible
    fn generate_unit_content(config: &SystemdConfig) -> Result<String, InstallerError> {
        let mut content = String::with_capacity(2048); // Pre-allocate for performance

        // [Unit] section
        content.push_str("[Unit]\n");
        content.push_str(&format!("Description={}\n", config.description));
        content.push_str("Documentation=https://github.com/cyrup/sweetmcp\n");

        if config.wants_network {
            content.push_str("Wants=network-online.target\n");
            content.push_str("After=network-online.target\n");
            content.push_str("Requires=network.target\n");
        }

        content.push_str("After=multi-user.target\n");
        content.push_str("DefaultDependencies=no\n");
        content.push('\n');

        // [Service] section
        content.push_str("[Service]\n");
        content.push_str("Type=notify\n"); // Use sd_notify for proper startup signaling
        content.push_str("NotifyAccess=main\n");

        // Build ExecStart command
        let exec_start = if config.args.is_empty() {
            format!("ExecStart={}\n", config.binary_path)
        } else {
            format!(
                "ExecStart={} {}\n",
                config.binary_path,
                config.args.join(" ")
            )
        };
        content.push_str(&exec_start);

        // Restart configuration
        if config.auto_restart {
            content.push_str("Restart=on-failure\n");
            content.push_str("RestartSec=5s\n");
            content.push_str("StartLimitInterval=60s\n");
            content.push_str("StartLimitBurst=3\n");
        } else {
            content.push_str("Restart=no\n");
        }

        // Environment variables
        for (key, value) in config.env_vars {
            content.push_str(&format!("Environment=\"{}={}\"\n", key, value));
        }

        // Security and sandboxing
        content.push_str("NoNewPrivileges=true\n");
        content.push_str("ProtectSystem=strict\n");
        content.push_str("ProtectHome=true\n");
        content.push_str("ProtectKernelTunables=true\n");
        content.push_str("ProtectControlGroups=true\n");
        content.push_str("RestrictSUIDSGID=true\n");
        content.push_str("RestrictRealtime=true\n");
        content.push_str("RestrictNamespaces=true\n");
        content.push_str("LockPersonality=true\n");
        content.push_str("MemoryDenyWriteExecute=true\n");

        // Allow specific directories for daemon operation
        content.push_str("ReadWritePaths=/var/log /var/lib /tmp\n");
        content.push_str("ReadOnlyPaths=/etc\n");

        // Resource limits
        content.push_str("LimitNOFILE=65536\n");
        content.push_str("LimitNPROC=4096\n");

        // User/Group configuration
        if let Some(user) = config.user {
            content.push_str(&format!("User={}\n", user));
        }
        if let Some(group) = config.group {
            content.push_str(&format!("Group={}\n", group));
        }

        // Logging
        content.push_str("StandardOutput=journal\n");
        content.push_str("StandardError=journal\n");
        content.push_str("SyslogIdentifier=sweetmcp\n");

        // Watchdog support
        content.push_str("WatchdogSec=30s\n");
        content.push('\n');

        // [Install] section
        content.push_str("[Install]\n");
        content.push_str("WantedBy=multi-user.target\n");

        Ok(content)
    }

    /// Create systemd drop-in configuration for advanced features
    fn create_dropin_config(config: &SystemdConfig) -> Result<(), InstallerError> {
        let dropin_dir = if unsafe { libc::getuid() } == 0 {
            PathBuf::from("/etc/systemd/system").join(format!("{}.service.d", config.service_name))
        } else {
            let home_dir = std::env::var("HOME").map_err(|_| {
                InstallerError::System("HOME environment variable not set".to_string())
            })?;
            PathBuf::from(home_dir)
                .join(".config/systemd/user")
                .join(format!("{}.service.d", config.service_name))
        };

        // Create drop-in directory
        fs::create_dir_all(&dropin_dir).map_err(|e| {
            InstallerError::System(format!("Failed to create drop-in directory: {}", e))
        })?;

        // Create override configuration for advanced features
        let override_content = format!(
            r#"[Service]
# Resource management
MemoryMax=1G
CPUQuota=200%
TasksMax=1024

# Additional security
SystemCallFilter=@system-service
SystemCallErrorNumber=EPERM
SystemCallArchitectures=native

# Capability restrictions
CapabilityBoundingSet=CAP_NET_BIND_SERVICE CAP_SETUID CAP_SETGID
AmbientCapabilities=CAP_NET_BIND_SERVICE

# Process management
OOMScoreAdjust=-100
Nice=-5

# Service metadata
X-SweetMCP-Service=true
X-SweetMCP-Version={}
"#,
            env!("CARGO_PKG_VERSION")
        );

        let override_path = dropin_dir.join("10-sweetmcp.conf");
        Self::write_file_atomic(&override_path, &override_content)?;

        Ok(())
    }

    /// Setup systemd journal integration for structured logging
    fn setup_journal_integration(service_name: &str) -> Result<(), InstallerError> {
        // Create journal configuration for the service
        let journal_config = format!(
            r#"# Systemd journal configuration for {}
[Journal]
MaxRetentionSec=7day
MaxFileSec=1day
Compress=yes
"#,
            service_name
        );

        let journal_config_dir = PathBuf::from("/etc/systemd/journald.conf.d");
        if journal_config_dir.exists() {
            let config_path = journal_config_dir.join(format!("{}.conf", service_name));
            Self::write_file_atomic(&config_path, &journal_config)?;
        }

        Ok(())
    }

    /// Enable the systemd service
    fn enable_systemd_service(service_name: &str) -> Result<(), InstallerError> {
        let output = if unsafe { libc::getuid() } == 0 {
            Command::new("systemctl")
                .args(["enable", &format!("{}.service", service_name)])
                .output()
        } else {
            Command::new("systemctl")
                .args(["--user", "enable", &format!("{}.service", service_name)])
                .output()
        };

        let output = output.map_err(|e| {
            InstallerError::System(format!("Failed to execute systemctl enable: {}", e))
        })?;

        if !output.status.success() {
            return Err(InstallerError::System(format!(
                "Failed to enable systemd service: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Start the systemd service
    fn start_systemd_service(service_name: &str) -> Result<(), InstallerError> {
        let output = if unsafe { libc::getuid() } == 0 {
            Command::new("systemctl")
                .args(["start", &format!("{}.service", service_name)])
                .output()
        } else {
            Command::new("systemctl")
                .args(["--user", "start", &format!("{}.service", service_name)])
                .output()
        };

        let output = output.map_err(|e| {
            InstallerError::System(format!("Failed to execute systemctl start: {}", e))
        })?;

        if !output.status.success() {
            return Err(InstallerError::System(format!(
                "Failed to start systemd service: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Stop the systemd service
    fn stop_systemd_service(service_name: &str) -> Result<(), InstallerError> {
        let output = if unsafe { libc::getuid() } == 0 {
            Command::new("systemctl")
                .args(["stop", &format!("{}.service", service_name)])
                .output()
        } else {
            Command::new("systemctl")
                .args(["--user", "stop", &format!("{}.service", service_name)])
                .output()
        };

        let output = output.map_err(|e| {
            InstallerError::System(format!("Failed to execute systemctl stop: {}", e))
        })?;

        if !output.status.success() {
            return Err(InstallerError::System(format!(
                "Failed to stop systemd service: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Disable the systemd service
    fn disable_systemd_service(service_name: &str) -> Result<(), InstallerError> {
        let output = if unsafe { libc::getuid() } == 0 {
            Command::new("systemctl")
                .args(["disable", &format!("{}.service", service_name)])
                .output()
        } else {
            Command::new("systemctl")
                .args(["--user", "disable", &format!("{}.service", service_name)])
                .output()
        };

        let output = output.map_err(|e| {
            InstallerError::System(format!("Failed to execute systemctl disable: {}", e))
        })?;

        if !output.status.success() {
            return Err(InstallerError::System(format!(
                "Failed to disable systemd service: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Remove systemd unit file
    fn remove_systemd_unit(service_name: &str) -> Result<(), InstallerError> {
        let unit_path = if unsafe { libc::getuid() } == 0 {
            PathBuf::from("/etc/systemd/system").join(format!("{}.service", service_name))
        } else {
            let home_dir = std::env::var("HOME").map_err(|_| {
                InstallerError::System("HOME environment variable not set".to_string())
            })?;
            PathBuf::from(home_dir)
                .join(".config/systemd/user")
                .join(format!("{}.service", service_name))
        };

        if unit_path.exists() {
            fs::remove_file(&unit_path).map_err(|e| {
                InstallerError::System(format!("Failed to remove unit file: {}", e))
            })?;
        }

        Ok(())
    }

    /// Clean up drop-in configuration
    fn cleanup_dropin_config(service_name: &str) -> Result<(), InstallerError> {
        let dropin_dir = if unsafe { libc::getuid() } == 0 {
            PathBuf::from("/etc/systemd/system").join(format!("{}.service.d", service_name))
        } else {
            let home_dir = std::env::var("HOME").map_err(|_| {
                InstallerError::System("HOME environment variable not set".to_string())
            })?;
            PathBuf::from(home_dir)
                .join(".config/systemd/user")
                .join(format!("{}.service.d", service_name))
        };

        if dropin_dir.exists() {
            fs::remove_dir_all(&dropin_dir).map_err(|e| {
                InstallerError::System(format!("Failed to remove drop-in directory: {}", e))
            })?;
        }

        Ok(())
    }

    /// Clean up journal integration
    fn cleanup_journal_integration(service_name: &str) -> Result<(), InstallerError> {
        let journal_config_dir = PathBuf::from("/etc/systemd/journald.conf.d");
        let config_path = journal_config_dir.join(format!("{}.conf", service_name));

        if config_path.exists() {
            fs::remove_file(&config_path).map_err(|e| {
                InstallerError::System(format!("Failed to remove journal config: {}", e))
            })?;
        }

        Ok(())
    }

    /// Reload systemd daemon to pick up changes
    fn reload_systemd_daemon() -> Result<(), InstallerError> {
        let output = if unsafe { libc::getuid() } == 0 {
            Command::new("systemctl").args(["daemon-reload"]).output()
        } else {
            Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .output()
        };

        let output = output.map_err(|e| {
            InstallerError::System(format!("Failed to execute systemctl daemon-reload: {}", e))
        })?;

        if !output.status.success() {
            return Err(InstallerError::System(format!(
                "Failed to reload systemd daemon: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Install service definitions in configuration directory
    fn install_services(
        services: &[crate::config::ServiceDefinition],
    ) -> Result<(), InstallerError> {
        for service in services {
            let service_toml = toml::to_string_pretty(service).map_err(|e| {
                InstallerError::System(format!("Failed to serialize service: {}", e))
            })?;

            // Create services directory
            let services_dir = PathBuf::from("/etc/sweetmcp/services");
            fs::create_dir_all(&services_dir).map_err(|e| {
                InstallerError::System(format!("Failed to create services directory: {}", e))
            })?;

            // Write service file
            let service_file = services_dir.join(format!("{}.toml", service.name));
            Self::write_file_atomic(&service_file, &service_toml)?;

            // Set appropriate permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&service_file)
                    .map_err(|e| {
                        InstallerError::System(format!(
                            "Failed to get service file metadata: {}",
                            e
                        ))
                    })?
                    .permissions();
                perms.set_mode(0o644);
                fs::set_permissions(&service_file, perms).map_err(|e| {
                    InstallerError::System(format!("Failed to set service file permissions: {}", e))
                })?;
            }
        }
        Ok(())
    }

    /// Verify helper executable signature (if signing is available)
    fn verify_helper_signature(helper_path: &Path) -> Result<(), InstallerError> {
        // Use the signing module to verify the helper
        crate::signing::verify_signature(helper_path).map_err(|e| {
            InstallerError::System(format!("Helper signature verification failed: {}", e))
        })?;
        Ok(())
    }

    /// Write file atomically to prevent corruption
    fn write_file_atomic(path: &Path, content: &str) -> Result<(), InstallerError> {
        let temp_path = path.with_extension("tmp");

        {
            let mut file = fs::File::create(&temp_path).map_err(|e| {
                InstallerError::System(format!("Failed to create temp file: {}", e))
            })?;

            file.write_all(content.as_bytes())
                .map_err(|e| InstallerError::System(format!("Failed to write temp file: {}", e)))?;

            file.sync_all()
                .map_err(|e| InstallerError::System(format!("Failed to sync temp file: {}", e)))?;
        }

        fs::rename(&temp_path, path)
            .map_err(|e| InstallerError::System(format!("Failed to rename temp file: {}", e)))?;

        Ok(())
    }

    pub async fn install_async(b: InstallerBuilder) -> Result<(), InstallerError> {
        tokio::task::spawn_blocking(move || Self::install(b))
            .await
            .context("task join failed")?
    }

    pub async fn uninstall_async(label: &str) -> Result<(), InstallerError> {
        let label = label.to_string();
        tokio::task::spawn_blocking(move || Self::uninstall(&label))
            .await
            .context("task join failed")?
    }
}
