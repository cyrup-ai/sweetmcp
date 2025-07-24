//! Installer module decomposition
//!
//! This module provides the decomposed installer functionality split into
//! logical modules for better maintainability and adherence to the 300-line limit.

pub mod core;
pub mod config;
pub mod uninstall;

// Re-export key types and functions for backward compatibility
pub use core::{
    AsyncTask, InstallProgress, CertificateConfig, ServiceConfig, InstallContext
};

pub use config::{
    install_sweetmcp_daemon, validate_configuration, create_default_configuration,
    remove_sweetmcp_host_entries
};

pub use uninstall::{
    uninstall_sweetmcp_daemon, backup_configuration, restore_configuration
};