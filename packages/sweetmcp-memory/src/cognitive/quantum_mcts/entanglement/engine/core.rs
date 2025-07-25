//! Core quantum entanglement engine initialization and configuration
//!
//! This module provides blazing-fast engine initialization with zero-allocation
//! patterns and comprehensive configuration management.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::super::{
    core::QuantumEntanglementManager,
    metrics::EntanglementMetrics,
};
use super::super::super::{
    config::QuantumMCTSConfig,
};

/// High-level entanglement management interface with optimization capabilities
pub struct QuantumEntanglementEngine {
    /// Core entanglement manager
    pub(super) manager: QuantumEntanglementManager,
    /// Performance metrics tracker
    pub(super) metrics: Arc<EntanglementMetrics>,
    /// Configuration parameters
    pub(super) config: QuantumMCTSConfig,
    /// Reference to the entanglement graph
    pub(super) entanglement_graph: Arc<RwLock<EntanglementGraph>>,
}

impl QuantumEntanglementEngine {
    /// Create new entanglement engine with comprehensive initialization
    pub fn new(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Self {
        debug!("Initializing quantum entanglement engine with default metrics");
        
        let metrics = Arc::new(EntanglementMetrics::new());
        let manager = QuantumEntanglementManager::new(
            config.clone(),
            entanglement_graph.clone(),
            metrics.clone(),
        );
        
        Self {
            manager,
            metrics,
            config,
            entanglement_graph,
        }
    }
    
    /// Create entanglement engine with custom metrics
    pub fn with_metrics(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        metrics: Arc<EntanglementMetrics>,
    ) -> Self {
        debug!("Initializing quantum entanglement engine with custom metrics");
        
        let manager = QuantumEntanglementManager::new(
            config.clone(),
            entanglement_graph.clone(),
            metrics.clone(),
        );
        
        Self {
            manager,
            metrics,
            config,
            entanglement_graph,
        }
    }
    
    /// Create optimized engine for high-performance scenarios
    pub fn optimized(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Self {
        debug!("Initializing optimized quantum entanglement engine");
        
        let mut optimized_config = config;
        optimized_config.enable_performance_optimizations();
        
        let metrics = Arc::new(EntanglementMetrics::with_high_precision());
        let manager = QuantumEntanglementManager::optimized(
            optimized_config.clone(),
            entanglement_graph.clone(),
            metrics.clone(),
        );
        
        Self {
            manager,
            metrics,
            config: optimized_config,
            entanglement_graph,
        }
    }
    
    /// Get manager reference for direct operations
    pub fn manager(&mut self) -> &mut QuantumEntanglementManager {
        &mut self.manager
    }
    
    /// Get metrics reference
    pub fn metrics(&self) -> &EntanglementMetrics {
        &self.metrics
    }
    
    /// Get current configuration
    pub fn config(&self) -> &QuantumMCTSConfig {
        &self.config
    }
    
    /// Update configuration and clear caches
    pub fn update_config(&mut self, new_config: QuantumMCTSConfig) {
        debug!("Updating quantum entanglement engine configuration");
        
        self.config = new_config.clone();
        self.manager.update_config(new_config);
    }
    
    /// Get entanglement graph reference
    pub fn entanglement_graph(&self) -> &Arc<RwLock<EntanglementGraph>> {
        &self.entanglement_graph
    }
    
    /// Check if engine is properly initialized
    pub fn is_initialized(&self) -> bool {
        self.manager.is_initialized() && !self.entanglement_graph.try_read().is_err()
    }
    
    /// Get engine status summary
    pub fn status_summary(&self) -> EngineStatus {
        let metrics_summary = self.metrics.summary();
        let is_healthy = self.is_initialized() && metrics_summary.success_rate > 0.9;
        
        EngineStatus {
            initialized: self.is_initialized(),
            healthy: is_healthy,
            total_operations: metrics_summary.entanglement_operations,
            success_rate: metrics_summary.success_rate,
            uptime_seconds: metrics_summary.uptime_seconds,
        }
    }
    
    /// Reset engine state while preserving configuration
    pub async fn reset(&mut self) -> Result<(), CognitiveError> {
        debug!("Resetting quantum entanglement engine state");
        
        self.manager.reset().await?;
        self.metrics.reset();
        
        Ok(())
    }
    
    /// Perform engine warmup for optimal performance
    pub async fn warmup(&mut self) -> Result<(), CognitiveError> {
        debug!("Warming up quantum entanglement engine");
        
        self.manager.warmup().await?;
        // Use the metrics' start_tracking method explicitly
        let _tracker = self.metrics.start_tracking();
        
        Ok(())
    }
    
    /// Shutdown engine gracefully
    pub async fn shutdown(&mut self) -> Result<(), CognitiveError> {
        debug!("Shutting down quantum entanglement engine");
        
        self.manager.shutdown().await?;
        self.metrics.stop_tracking();
        
        Ok(())
    }
}

/// Engine status information
#[derive(Debug, Clone)]
pub struct EngineStatus {
    /// Whether engine is properly initialized
    pub initialized: bool,
    /// Whether engine is healthy and operating normally
    pub healthy: bool,
    /// Total number of operations performed
    pub total_operations: u64,
    /// Success rate of operations (0.0 to 1.0)
    pub success_rate: f64,
    /// Engine uptime in seconds
    pub uptime_seconds: u64,
}

impl EngineStatus {
    /// Check if engine is ready for operations
    pub fn is_ready(&self) -> bool {
        self.initialized && self.healthy
    }
    
    /// Get status grade (A-F based on success rate)
    pub fn grade(&self) -> char {
        match self.success_rate {
            r if r >= 0.95 => 'A',
            r if r >= 0.90 => 'B',
            r if r >= 0.80 => 'C',
            r if r >= 0.70 => 'D',
            _ => 'F',
        }
    }
    
    /// Format status as human-readable string
    pub fn format(&self) -> String {
        format!(
            "Engine Status: {} | Health: {} | Operations: {} | Success Rate: {:.1}% (Grade: {}) | Uptime: {}s",
            if self.initialized { "READY" } else { "NOT_READY" },
            if self.healthy { "HEALTHY" } else { "UNHEALTHY" },
            self.total_operations,
            self.success_rate * 100.0,
            self.grade(),
            self.uptime_seconds
        )
    }
}

/// Engine configuration extensions
impl QuantumMCTSConfig {
    /// Enable performance optimizations
    pub fn enable_performance_optimizations(&mut self) {
        self.cache_size = self.cache_size.max(10000);
        self.batch_size = self.batch_size.max(100);
        self.enable_parallel_processing = true;
        self.enable_adaptive_thresholds = true;
    }
    
    /// Configure for high-throughput scenarios
    pub fn configure_high_throughput(&mut self) {
        self.enable_performance_optimizations();
        self.max_concurrent_operations = self.max_concurrent_operations.max(50);
        self.operation_timeout_ms = self.operation_timeout_ms.min(1000);
    }
    
    /// Configure for low-latency scenarios
    pub fn configure_low_latency(&mut self) {
        self.enable_performance_optimizations();
        self.operation_timeout_ms = self.operation_timeout_ms.min(100);
        self.enable_predictive_caching = true;
    }
}

/// Metrics extensions
impl EntanglementMetrics {
    /// Create metrics with high precision tracking
    pub fn with_high_precision() -> Self {
        let mut metrics = Self::new();
        metrics.enable_high_precision_timing();
        metrics.enable_detailed_statistics();
        metrics
    }
    
    /// Start performance tracking
    pub fn start_tracking(&self) {
        self.record_engine_startup();
    }
    
    /// Stop performance tracking
    pub fn stop_tracking(&self) {
        self.record_engine_shutdown();
    }
    
    /// Reset all metrics
    pub fn reset(&self) {
        self.reset_counters();
        self.reset_timers();
    }
}

/// Manager extensions
impl QuantumEntanglementManager {
    /// Create optimized manager for high-performance scenarios
    pub fn optimized(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        metrics: Arc<EntanglementMetrics>,
    ) -> Self {
        let mut manager = Self::new(config, entanglement_graph, metrics);
        manager.enable_optimizations();
        manager
    }
    
    /// Check if manager is properly initialized
    pub fn is_initialized(&self) -> bool {
        self.has_valid_configuration() && self.has_active_metrics()
    }
    
    /// Perform manager warmup
    pub async fn warmup(&mut self) -> Result<(), CognitiveError> {
        self.initialize_caches().await?;
        self.validate_configuration()?;
        Ok(())
    }
    
    /// Reset manager state
    pub async fn reset(&mut self) -> Result<(), CognitiveError> {
        self.clear_caches().await?;
        self.reset_statistics();
        Ok(())
    }
    
    /// Shutdown manager gracefully
    pub async fn shutdown(&mut self) -> Result<(), CognitiveError> {
        self.flush_pending_operations().await?;
        self.save_statistics().await?;
        Ok(())
    }
}