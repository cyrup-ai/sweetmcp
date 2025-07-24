//! Edge service core module
//!
//! This module provides comprehensive EdgeService functionality including
//! service initialization, operations, and builder pattern with zero allocation
//! patterns and blazing-fast performance.

pub mod service;
pub mod operations;
pub mod builder;

// Re-export key types and functions for ergonomic usage
pub use service::{
    EdgeService, ServiceStatus, ServiceMetrics, EdgeServiceError, ErrorSeverity,
};

pub use operations::{
    HealthStatus, ServiceStatistics,
};

pub use builder::{
    EdgeServiceBuilder, BuilderStatus, BuilderPreset,
};

/// Create a new EdgeService with default configuration
pub fn edge_service(
    cfg: std::sync::Arc<crate::config::Config>,
    bridge_tx: tokio::sync::mpsc::Sender<crate::mcp_bridge::BridgeMsg>,
    peer_registry: crate::peer_discovery::PeerRegistry,
) -> EdgeService {
    EdgeService::new(cfg, bridge_tx, peer_registry)
}

/// Create a new EdgeServiceBuilder
pub fn edge_service_builder() -> EdgeServiceBuilder {
    EdgeServiceBuilder::new()
}

/// Create EdgeService with builder pattern
pub fn build_edge_service() -> EdgeServiceBuilder {
    EdgeServiceBuilder::new()
}

/// Create EdgeService for development
pub fn development_edge_service() -> EdgeServiceBuilder {
    EdgeServiceBuilder::development()
}

/// Create EdgeService for production
pub fn production_edge_service() -> EdgeServiceBuilder {
    EdgeServiceBuilder::production()
}

/// Create EdgeService for testing
pub fn testing_edge_service() -> EdgeServiceBuilder {
    EdgeServiceBuilder::testing()
}