//! Code signing module for sweetmcp-daemon
//! 
//! Provides cross-platform code signing functionality using Tauri's bundler
//! infrastructure for production-ready signing on macOS, Windows, and Linux.

use std::path::{Path, PathBuf};
use std::env;
use anyhow::{Context, Result};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

pub mod config;

/// Signing configuration for the daemon
#[derive(Debug, Clone)]
pub struct SigningConfig {
    /// Path to the binary to sign
    pub binary_path: PathBuf,
    /// Output path for signed binary (may be same as input)
    pub output_path: PathBuf,
    /// Platform-specific settings
    pub platform: PlatformConfig,
}

/// Platform-specific signing configuration
#[derive(Debug, Clone)]
pub enum PlatformConfig {
    MacOS {
        /// Signing identity (certificate name or "-" for ad-hoc)
        identity: String,
        /// Team ID for notarization
        team_id: Option<String>,
        /// Apple ID for notarization
        apple_id: Option<String>,
        /// App-specific password for notarization
        apple_password: Option<String>,
        /// Entitlements file path
        entitlements: Option<PathBuf>,
    },
    Windows {
        /// Certificate thumbprint or path to .pfx file
        certificate: String,
        /// Certificate password (for .pfx files)
        password: Option<String>,
        /// Timestamp server URL
        timestamp_url: String,
        /// Digest algorithm (sha256 recommended)
        digest_algorithm: String,
    },
    Linux {
        /// GPG key ID for signing
        key_id: Option<String>,
        /// Create detached signature
        detached: bool,
    },
}

impl Default for SigningConfig {
    fn default() -> Self {
        let binary_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("sweetmcp-daemon"));
        
        Self {
            binary_path: binary_path.clone(),
            output_path: binary_path,
            platform: Self::default_platform_config(),
        }
    }
}

impl SigningConfig {
    /// Create a new signing configuration
    pub fn new(binary_path: PathBuf) -> Self {
        Self {
            binary_path: binary_path.clone(),
            output_path: binary_path,
            platform: Self::default_platform_config(),
        }
    }
    
    /// Load configuration from environment and config files
    pub fn load() -> Result<Self> {
        config::load_config()
    }
    
    /// Get default platform configuration
    fn default_platform_config() -> PlatformConfig {
        #[cfg(target_os = "macos")]
        {
            PlatformConfig::MacOS {
                identity: env::var("APPLE_SIGNING_IDENTITY")
                    .unwrap_or_else(|_| "-".to_string()),
                team_id: env::var("APPLE_TEAM_ID").ok(),
                apple_id: env::var("APPLE_ID").ok(),
                apple_password: env::var("APPLE_PASSWORD").ok(),
                entitlements: None,
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            PlatformConfig::Windows {
                certificate: env::var("WINDOWS_CERTIFICATE")
                    .or_else(|_| env::var("WINDOWS_CERTIFICATE_THUMBPRINT"))
                    .unwrap_or_default(),
                password: env::var("WINDOWS_CERTIFICATE_PASSWORD").ok(),
                timestamp_url: env::var("WINDOWS_TIMESTAMP_URL")
                    .unwrap_or_else(|_| "http://timestamp.digicert.com".to_string()),
                digest_algorithm: "sha256".to_string(),
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            PlatformConfig::Linux {
                key_id: env::var("GPG_KEY_ID").ok(),
                detached: true,
            }
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            panic!("Unsupported platform for signing");
        }
    }
}

/// Sign a binary with the given configuration
pub fn sign_binary(config: &SigningConfig) -> Result<()> {
    match &config.platform {
        #[cfg(target_os = "macos")]
        PlatformConfig::MacOS { .. } => macos::sign(config),
        
        #[cfg(target_os = "windows")]
        PlatformConfig::Windows { .. } => windows::sign(config),
        
        #[cfg(target_os = "linux")]
        PlatformConfig::Linux { .. } => linux::sign(config),
        
        #[allow(unreachable_patterns)]
        _ => Err(anyhow::anyhow!("Platform signing not implemented")),
    }
}

/// Verify a signed binary
pub fn verify_signature(binary_path: &Path) -> Result<bool> {
    #[cfg(target_os = "macos")]
    return macos::verify(binary_path);
    
    #[cfg(target_os = "windows")]
    return windows::verify(binary_path);
    
    #[cfg(target_os = "linux")]
    return linux::verify(binary_path);
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return Ok(false);
}

/// Sign the current running binary (self-signing)
pub fn sign_self() -> Result<()> {
    let config = SigningConfig::load()?;
    sign_binary(&config)
}

/// Check if code signing is available on this platform
pub fn is_signing_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Check if codesign is available
        which::which("codesign").is_ok()
    }
    
    #[cfg(target_os = "windows")]
    {
        // Check if signtool is available
        which::which("signtool").is_ok()
    }
    
    #[cfg(target_os = "linux")]
    {
        // Check if gpg is available
        which::which("gpg").is_ok() || which::which("gpg2").is_ok()
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = SigningConfig::default();
        assert!(!config.binary_path.as_os_str().is_empty());
    }
    
    #[test]
    fn test_signing_available() {
        // This should at least not panic
        let _ = is_signing_available();
    }
}