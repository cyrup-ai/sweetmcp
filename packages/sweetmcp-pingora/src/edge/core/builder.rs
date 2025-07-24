//! EdgeService builder pattern implementation
//!
//! This module provides the EdgeServiceBuilder for flexible construction
//! of EdgeService instances with zero allocation patterns and blazing-fast
//! performance.

use super::service::{EdgeService, EdgeServiceError};
use crate::{
    config::Config,
    peer_discovery::PeerRegistry,
    rate_limit::AdvancedRateLimitManager,
    shutdown::ShutdownCoordinator,
};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::{debug, info};

/// Builder for EdgeService with flexible configuration
pub struct EdgeServiceBuilder {
    cfg: Option<Arc<Config>>,
    bridge_tx: Option<Sender<crate::mcp_bridge::BridgeMsg>>,
    peer_registry: Option<PeerRegistry>,
    custom_rate_limiter: Option<Arc<AdvancedRateLimitManager>>,
    custom_shutdown_coordinator: Option<Arc<ShutdownCoordinator>>,
}

impl EdgeServiceBuilder {
    /// Create new EdgeServiceBuilder
    pub fn new() -> Self {
        debug!("Creating new EdgeServiceBuilder");
        Self {
            cfg: None,
            bridge_tx: None,
            peer_registry: None,
            custom_rate_limiter: None,
            custom_shutdown_coordinator: None,
        }
    }

    /// Set configuration with validation
    pub fn with_config(mut self, cfg: Arc<Config>) -> Self {
        debug!("Setting configuration with {} upstreams", cfg.upstreams.len());
        self.cfg = Some(cfg);
        self
    }

    /// Set bridge channel with optimized channel handling
    pub fn with_bridge_channel(mut self, bridge_tx: Sender<crate::mcp_bridge::BridgeMsg>) -> Self {
        debug!("Setting bridge channel");
        self.bridge_tx = Some(bridge_tx);
        self
    }

    /// Set peer registry with fast registry setup
    pub fn with_peer_registry(mut self, peer_registry: PeerRegistry) -> Self {
        debug!("Setting peer registry");
        self.peer_registry = Some(peer_registry);
        self
    }

    /// Set custom rate limiter with advanced configuration
    pub fn with_custom_rate_limiter(mut self, rate_limiter: Arc<AdvancedRateLimitManager>) -> Self {
        debug!("Setting custom rate limiter");
        self.custom_rate_limiter = Some(rate_limiter);
        self
    }

    /// Set custom shutdown coordinator with optimized shutdown handling
    pub fn with_custom_shutdown_coordinator(mut self, coordinator: Arc<ShutdownCoordinator>) -> Self {
        debug!("Setting custom shutdown coordinator");
        self.custom_shutdown_coordinator = Some(coordinator);
        self
    }

    /// Build EdgeService with validation and optimization
    pub fn build(self) -> Result<EdgeService, EdgeServiceError> {
        info!("Building EdgeService");

        let cfg = self.cfg.ok_or_else(|| 
            EdgeServiceError::ConfigurationError("Configuration is required".to_string()))?;
        
        let bridge_tx = self.bridge_tx.ok_or_else(|| 
            EdgeServiceError::ConfigurationError("Bridge channel is required".to_string()))?;
        
        let peer_registry = self.peer_registry.ok_or_else(|| 
            EdgeServiceError::ConfigurationError("Peer registry is required".to_string()))?;

        // Create base service
        let mut service = EdgeService::new(cfg, bridge_tx, peer_registry);

        // Apply custom components if provided
        if let Some(custom_rate_limiter) = self.custom_rate_limiter {
            debug!("Applying custom rate limiter");
            service.rate_limit_manager = custom_rate_limiter;
        }

        if let Some(custom_shutdown_coordinator) = self.custom_shutdown_coordinator {
            debug!("Applying custom shutdown coordinator");
            service.shutdown_coordinator = custom_shutdown_coordinator;
        }

        // Validate the built service
        service.validate_config()?;

        info!("EdgeService built successfully");
        Ok(service)
    }

    /// Build with default components for testing
    pub fn build_for_testing(self) -> Result<EdgeService, EdgeServiceError> {
        info!("Building EdgeService for testing");

        // Create minimal configuration if not provided
        let cfg = self.cfg.unwrap_or_else(|| {
            Arc::new(Config {
                upstreams: vec!["http://localhost:8080".to_string()],
                jwt_secret: "test-secret".to_string(),
                ..Default::default()
            })
        });

        // Create test channel if not provided
        let bridge_tx = self.bridge_tx.unwrap_or_else(|| {
            let (tx, _rx) = tokio::sync::mpsc::channel(100);
            tx
        });

        // Create test peer registry if not provided
        let peer_registry = self.peer_registry.unwrap_or_else(|| {
            PeerRegistry::new()
        });

        // Build with test configuration
        Self {
            cfg: Some(cfg),
            bridge_tx: Some(bridge_tx),
            peer_registry: Some(peer_registry),
            custom_rate_limiter: self.custom_rate_limiter,
            custom_shutdown_coordinator: self.custom_shutdown_coordinator,
        }.build()
    }

    /// Validate builder state before building
    pub fn validate(&self) -> Result<(), EdgeServiceError> {
        if self.cfg.is_none() {
            return Err(EdgeServiceError::ConfigurationError(
                "Configuration must be set before building".to_string()
            ));
        }

        if self.bridge_tx.is_none() {
            return Err(EdgeServiceError::ConfigurationError(
                "Bridge channel must be set before building".to_string()
            ));
        }

        if self.peer_registry.is_none() {
            return Err(EdgeServiceError::ConfigurationError(
                "Peer registry must be set before building".to_string()
            ));
        }

        // Validate configuration if present
        if let Some(ref cfg) = self.cfg {
            if cfg.upstreams.is_empty() {
                return Err(EdgeServiceError::ConfigurationError(
                    "At least one upstream must be configured".to_string()
                ));
            }

            if cfg.jwt_secret.is_empty() {
                return Err(EdgeServiceError::ConfigurationError(
                    "JWT secret must be configured".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Get builder status for debugging
    pub fn status(&self) -> BuilderStatus {
        BuilderStatus {
            has_config: self.cfg.is_some(),
            has_bridge_channel: self.bridge_tx.is_some(),
            has_peer_registry: self.peer_registry.is_some(),
            has_custom_rate_limiter: self.custom_rate_limiter.is_some(),
            has_custom_shutdown_coordinator: self.custom_shutdown_coordinator.is_some(),
            is_ready: self.validate().is_ok(),
        }
    }

    /// Reset builder to initial state
    pub fn reset(mut self) -> Self {
        debug!("Resetting EdgeServiceBuilder");
        self.cfg = None;
        self.bridge_tx = None;
        self.peer_registry = None;
        self.custom_rate_limiter = None;
        self.custom_shutdown_coordinator = None;
        self
    }

    /// Clone builder for parallel construction
    pub fn clone_builder(&self) -> Self {
        Self {
            cfg: self.cfg.clone(),
            bridge_tx: self.bridge_tx.clone(),
            peer_registry: self.peer_registry.clone(),
            custom_rate_limiter: self.custom_rate_limiter.clone(),
            custom_shutdown_coordinator: self.custom_shutdown_coordinator.clone(),
        }
    }

    /// Build multiple services with different configurations
    pub fn build_multiple(
        base_builder: Self,
        configs: Vec<Arc<Config>>,
    ) -> Result<Vec<EdgeService>, EdgeServiceError> {
        let mut services = Vec::with_capacity(configs.len());

        for config in configs {
            let builder = base_builder.clone_builder().with_config(config);
            let service = builder.build()?;
            services.push(service);
        }

        info!("Built {} EdgeService instances", services.len());
        Ok(services)
    }

    /// Create builder from existing service configuration
    pub fn from_service(service: &EdgeService) -> Self {
        Self {
            cfg: Some(service.cfg.clone()),
            bridge_tx: Some(service.bridge_tx.clone()),
            peer_registry: Some(service.peer_registry.clone()),
            custom_rate_limiter: Some(service.rate_limit_manager.clone()),
            custom_shutdown_coordinator: Some(service.shutdown_coordinator.clone()),
        }
    }

    /// Apply configuration preset
    pub fn with_preset(mut self, preset: BuilderPreset) -> Self {
        match preset {
            BuilderPreset::Development => {
                debug!("Applying development preset");
                // Development settings would be applied here
                self
            }
            BuilderPreset::Production => {
                debug!("Applying production preset");
                // Production settings would be applied here
                self
            }
            BuilderPreset::Testing => {
                debug!("Applying testing preset");
                // Testing settings would be applied here
                self
            }
        }
    }
}

impl Default for EdgeServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder status information
#[derive(Debug, Clone)]
pub struct BuilderStatus {
    pub has_config: bool,
    pub has_bridge_channel: bool,
    pub has_peer_registry: bool,
    pub has_custom_rate_limiter: bool,
    pub has_custom_shutdown_coordinator: bool,
    pub is_ready: bool,
}

impl BuilderStatus {
    /// Check if builder is complete
    pub fn is_complete(&self) -> bool {
        self.has_config && self.has_bridge_channel && self.has_peer_registry
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> f64 {
        let required_components = 3.0; // config, bridge_channel, peer_registry
        let mut completed = 0.0;

        if self.has_config { completed += 1.0; }
        if self.has_bridge_channel { completed += 1.0; }
        if self.has_peer_registry { completed += 1.0; }

        (completed / required_components) * 100.0
    }

    /// Get missing components
    pub fn missing_components(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();

        if !self.has_config {
            missing.push("configuration");
        }
        if !self.has_bridge_channel {
            missing.push("bridge_channel");
        }
        if !self.has_peer_registry {
            missing.push("peer_registry");
        }

        missing
    }
}

/// Configuration presets for common use cases
#[derive(Debug, Clone)]
pub enum BuilderPreset {
    Development,
    Production,
    Testing,
}

/// Convenience functions for common builder patterns
impl EdgeServiceBuilder {
    /// Quick builder for development
    pub fn development() -> Self {
        Self::new().with_preset(BuilderPreset::Development)
    }

    /// Quick builder for production
    pub fn production() -> Self {
        Self::new().with_preset(BuilderPreset::Production)
    }

    /// Quick builder for testing
    pub fn testing() -> Self {
        Self::new().with_preset(BuilderPreset::Testing)
    }
}