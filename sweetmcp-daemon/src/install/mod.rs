//! Cross-platform privileged daemon installer.
//!
//! This module provides a unified interface for installing system daemons/services across
//! Linux (systemd), macOS (launchd), and Windows (Service Control Manager) with proper
//! GUI privilege escalation.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod builder;
mod error;
pub mod fluent_voice;

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

pub use builder::InstallerBuilder;
pub use error::InstallerError;

/// Result type alias for installer operations
pub type Result<T> = std::result::Result<T, InstallerError>;

/// Synchronous daemon installation.
pub fn install_daemon(builder: InstallerBuilder) -> Result<()> {
    Executor::install(builder)
}

/// Synchronous daemon uninstallation.
pub fn uninstall_daemon(label: &str) -> Result<()> {
    Executor::uninstall(label)
}

/// Asynchronous daemon installation (requires `runtime` feature).
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
pub async fn install_daemon_async(builder: InstallerBuilder) -> Result<()> {
    Executor::install_async(builder).await
}

/// Asynchronous daemon uninstallation (requires `runtime` feature).
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
pub async fn uninstall_daemon_async(label: &str) -> Result<()> {
    Executor::uninstall_async(label).await
}
