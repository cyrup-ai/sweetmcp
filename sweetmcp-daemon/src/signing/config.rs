//! Signing configuration management

use super::{PlatformConfig, SigningConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Signing configuration file structure
#[derive(Debug, Serialize, Deserialize)]
pub struct SigningConfigFile {
    /// macOS signing configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macos: Option<MacOSConfig>,

    /// Windows signing configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<WindowsConfig>,

    /// Linux signing configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linux: Option<LinuxConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MacOSConfig {
    /// Signing identity (certificate name)
    pub identity: Option<String>,
    /// Team ID for notarization
    pub team_id: Option<String>,
    /// Path to entitlements file
    pub entitlements: Option<PathBuf>,
    /// Certificate file path (for CI/CD)
    pub certificate_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsConfig {
    /// Certificate thumbprint or path
    pub certificate: Option<String>,
    /// Timestamp server URL
    pub timestamp_url: Option<String>,
    /// Digest algorithm
    pub digest_algorithm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinuxConfig {
    /// GPG key ID
    pub key_id: Option<String>,
    /// Create detached signatures
    pub detached: Option<bool>,
}

/// Load signing configuration from file and environment
pub fn load_config() -> Result<SigningConfig> {
    let mut config = SigningConfig::default();

    // Try to load from config file
    if let Ok(file_config) = load_config_file() {
        merge_config(&mut config, file_config);
    }

    // Override with environment variables
    override_from_env(&mut config);

    Ok(config)
}

/// Find and load the signing configuration file
fn load_config_file() -> Result<SigningConfigFile> {
    // Search paths in order of priority
    let search_paths = vec![
        // Current directory
        PathBuf::from("signing.toml"),
        // Project root
        PathBuf::from("sweetmcp-daemon/signing.toml"),
        // User config directory
        dirs::config_dir()
            .map(|p| p.join("sweetmcp/signing.toml"))
            .unwrap_or_default(),
        // System config
        PathBuf::from("/etc/sweetmcp/signing.toml"),
    ];

    for path in search_paths {
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;

            let config: SigningConfigFile = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

            return Ok(config);
        }
    }

    // No config file found, return empty config
    Ok(SigningConfigFile {
        macos: None,
        windows: None,
        linux: None,
    })
}

/// Merge file configuration into the main config
fn merge_config(config: &mut SigningConfig, file_config: SigningConfigFile) {
    match &mut config.platform {
        #[cfg(target_os = "macos")]
        PlatformConfig::MacOS {
            identity,
            team_id,
            entitlements,
            ..
        } => {
            if let Some(mac_config) = file_config.macos {
                if let Some(id) = mac_config.identity {
                    *identity = id;
                }
                if let Some(tid) = mac_config.team_id {
                    *team_id = Some(tid);
                }
                if let Some(ent) = mac_config.entitlements {
                    *entitlements = Some(ent);
                }
            }
        }

        #[cfg(target_os = "windows")]
        PlatformConfig::Windows {
            certificate,
            timestamp_url,
            digest_algorithm,
            ..
        } => {
            if let Some(win_config) = file_config.windows {
                if let Some(cert) = win_config.certificate {
                    *certificate = cert;
                }
                if let Some(ts_url) = win_config.timestamp_url {
                    *timestamp_url = ts_url;
                }
                if let Some(digest) = win_config.digest_algorithm {
                    *digest_algorithm = digest;
                }
            }
        }

        #[cfg(target_os = "linux")]
        PlatformConfig::Linux { key_id, detached } => {
            if let Some(linux_config) = file_config.linux {
                if let Some(kid) = linux_config.key_id {
                    *key_id = Some(kid);
                }
                if let Some(det) = linux_config.detached {
                    *detached = det;
                }
            }
        }

        #[allow(unreachable_patterns)]
        _ => {}
    }
}

/// Override configuration from environment variables
fn override_from_env(config: &mut SigningConfig) {
    // Binary paths from environment
    if let Ok(bin_path) = env::var("SWEETMCP_BINARY_PATH") {
        config.binary_path = PathBuf::from(bin_path);
    }
    if let Ok(out_path) = env::var("SWEETMCP_OUTPUT_PATH") {
        config.output_path = PathBuf::from(out_path);
    }

    // Platform-specific overrides are already handled in SigningConfig::default_platform_config()
    // This is where we'd add any additional environment overrides
}

/// Create a sample configuration file
pub fn create_sample_config() -> Result<String> {
    let sample = SigningConfigFile {
        macos: Some(MacOSConfig {
            identity: Some("Developer ID Application: Your Name (TEAMID)".to_string()),
            team_id: Some("TEAMID".to_string()),
            entitlements: Some(PathBuf::from("entitlements.plist")),
            certificate_path: Some(PathBuf::from("/Users/username/.ssh/development.cer")),
        }),
        windows: Some(WindowsConfig {
            certificate: Some("thumbprint_or_path_to_pfx".to_string()),
            timestamp_url: Some("http://timestamp.digicert.com".to_string()),
            digest_algorithm: Some("sha256".to_string()),
        }),
        linux: Some(LinuxConfig {
            key_id: Some("YOUR_GPG_KEY_ID".to_string()),
            detached: Some(true),
        }),
    };

    toml::to_string_pretty(&sample).context("Failed to serialize sample config")
}
