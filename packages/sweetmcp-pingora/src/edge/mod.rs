//! Edge service module decomposition
//!
//! This module provides the decomposed EdgeService functionality split into
//! logical modules for better maintainability and adherence to the 300-line limit.

pub mod core;
pub mod auth;
pub mod routing;

// Re-export key types and functions for backward compatibility
pub use core::{
    EdgeService, EdgeServiceError, ServiceStats, EdgeServiceBuilder
};
pub use auth::{
    AuthHandler, AuthResult, UserClaims, AuthContext, AuthMethod, AuthConfig
};
pub use routing::{
    RoutingHandler, RoutingStrategy, RoutingContext
};