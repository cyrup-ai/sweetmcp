//! Cross-platform privileged daemon installer.
//! 
//! This crate provides a unified interface for installing system daemons/services across
//! Linux (systemd), macOS (launchd), and Windows (Service Control Manager) with proper
//! GUI privilege escalation.
//! 
//! # Examples
//! 
//! ```no_run
//! use elevated_daemon_installer::{InstallerBuilder, install_daemon};
//! 
//! let installer = InstallerBuilder::new("my-daemon", "/usr/local/bin/my-daemon")
//!     .arg("--config")
//!     .arg("/etc/my-daemon/config.toml")
//!     .env("RUST_LOG", "info")
//!     .user("daemon-user")
//!     .group("daemon-group")
//!     .description("My Awesome Daemon Service")
//!     .auto_restart(true)
//!     .network(true);
//! 
//! // Synchronous installation
//! install_daemon(installer)?;
//! 
//! // Uninstall
//! uninstall_daemon("my-daemon")?;
//! # Ok::<(), elevated_daemon_installer::InstallerError>(())
//! ```
//! 
//! # Platform Behavior
//! 
//! - **Linux**: Uses PolicyKit (`pkexec`) for GUI authorization, creates systemd units
//! - **macOS**: Uses `osascript` for authorization, creates launchd plists
//! - **Windows**: Uses UAC elevation, registers Windows services
//! 
//! All platforms provide proper user prompts instead of failing with permission errors.

#![deny(rust_2018_idioms, unused_must_use, clippy::all)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod builder;
mod error;

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        use linux::PlatformExecutor as Executor;
    } else if #[cfg(target_os = "macos")] {
        mod macos;
        use macos::PlatformExecutor as Executor;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        use windows::PlatformExecutor as Executor;
    } else {
        compile_error!("Unsupported platform for elevated_daemon_installer");
    }
}

pub use builder::{CommandBuilder, InstallerBuilder};
pub use error::InstallerError;

/// Result type alias for installer operations
pub type Result<T> = std::result::Result<T, InstallerError>;

/// Synchronous daemon installation.
/// 
/// This will prompt the user for administrative privileges using the platform's
/// standard GUI authorization mechanism.
/// 
/// # Errors
/// 
/// Returns `InstallerError::Cancelled` if the user cancels the authorization prompt,
/// `InstallerError::PermissionDenied` if authorization fails, or other errors for
/// system-level failures.
pub fn install_daemon(builder: InstallerBuilder) -> Result<()> {
    Executor::install(builder)
}

/// Synchronous daemon uninstallation.
/// 
/// Removes the daemon/service from the system's service manager and cleans up
/// associated files.
pub fn uninstall_daemon(label: &str) -> Result<()> {
    Executor::uninstall(label)
}

/// Asynchronous daemon installation (requires `runtime` feature).
/// 
/// Same as [`install_daemon`] but runs asynchronously using tokio.
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
pub async fn install_daemon_async(builder: InstallerBuilder) -> Result<()> {
    Executor::install_async(builder).await
}

/// Asynchronous daemon uninstallation (requires `runtime` feature).
/// 
/// Same as [`uninstall_daemon`] but runs asynchronously using tokio.
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
pub async fn uninstall_daemon_async(label: &str) -> Result<()> {
    Executor::uninstall_async(label).await
}