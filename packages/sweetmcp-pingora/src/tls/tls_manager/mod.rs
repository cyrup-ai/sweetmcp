//! TLS Manager module decomposition
//!
//! This module provides the decomposed TLS manager functionality split into
//! logical modules for better maintainability and adherence to the 300-line limit.

pub mod core;
pub mod certificate;
pub mod ocsp;
pub mod security;
pub mod config;
pub mod metrics;
pub mod error;

// Re-export key types and functions for backward compatibility
pub use core::{
    TlsManager, TlsError, ParsedCertificate, CertificateUsage, CrlCache
};
pub use certificate::{
    CertificateManager, CertificateValidator, CertificateUtils
};
pub use ocsp::{
    OcspManager, OcspUtils
};
pub use security::{
    SecurityManager, SecurityPolicy
};
pub use config::{
    ConfigManager, ConfigUtils
};
pub use metrics::{
    MetricsManager, CertificateGenerator, PerformanceMonitor
};
pub use error::{
    ErrorHandler, ErrorRecovery
};