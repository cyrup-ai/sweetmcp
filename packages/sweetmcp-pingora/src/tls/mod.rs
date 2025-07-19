//! TLS module organization

pub mod ocsp;
mod tls_manager;

// Re-export all public types from tls_manager
pub use tls_manager::*;
