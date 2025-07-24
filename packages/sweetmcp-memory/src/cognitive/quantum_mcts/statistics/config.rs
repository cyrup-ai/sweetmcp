//! Quantum MCTS configuration types for statistics
//!
//! This module provides comprehensive configuration definitions with zero allocation
//! optimizations and blazing-fast performance for quantum MCTS statistics.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Quantum MCTS configuration for statistics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumMCTSConfig {
    /// General MCTS configuration
    pub mcts: MCTSConfig,
    /// Quantum-specific configuration
    pub quantum: QuantumConfig,
    /// Statistics configuration
    pub statistics: StatisticsConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
}

impl QuantumMCTSConfig {
    /// Create new quantum MCTS configuration with defaults
    pub fn new() -> Self {
        Self {
            mcts: MCTSConfig::default(),
            quantum: QuantumConfig::default(),
            statistics: StatisticsConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }

    /// Create configuration optimized for performance
    pub fn performance_optimized() -> Self {
        let mut config = Self::new();
        config.performance.enable_fast_path = true;
        config.performance.cache_size = 10000;
        config.statistics.collection_interval = Duration::from_millis(100);
        config
    }

    /// Create configuration optimized for accuracy
    pub fn accuracy_optimized() -> Self {
        let mut config = Self::new();
        config.mcts.exploration_constant = 1.414;
        config.quantum.coherence_threshold = 0.99;
        config.statistics.detailed_tracking = true;
        config
    }
}

impl Default for QuantumMCTSConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// MCTS-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCTSConfig {
    /// Maximum number of iterations
    pub max_iterations: u64,
    /// Exploration constant (UCB1)
    pub exploration_constant: f64,
    /// Maximum tree depth
    pub max_depth: usize,
    /// Simulation timeout
    pub simulation_timeout: Duration,
    /// Enable progressive widening
    pub progressive_widening: bool,
}

impl Default for MCTSConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10000,
            exploration_constant: 1.0,
            max_depth: 100,
            simulation_timeout: Duration::from_secs(1),
            progressive_widening: true,
        }
    }
}

/// Quantum-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumConfig {
    /// Quantum coherence threshold
    pub coherence_threshold: f64,
    /// Entanglement strength factor
    pub entanglement_factor: f64,
    /// Superposition decay rate
    pub superposition_decay: f64,
    /// Quantum measurement frequency
    pub measurement_frequency: u64,
    /// Enable quantum interference
    pub enable_interference: bool,
}

impl Default for QuantumConfig {
    fn default() -> Self {
        Self {
            coherence_threshold: 0.8,
            entanglement_factor: 0.5,
            superposition_decay: 0.01,
            measurement_frequency: 100,
            enable_interference: true,
        }
    }
}

/// Statistics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsConfig {
    /// Enable statistics collection
    pub enabled: bool,
    /// Collection interval
    pub collection_interval: Duration,
    /// Maximum history size
    pub max_history_size: usize,
    /// Enable detailed tracking
    pub detailed_tracking: bool,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Statistics output format
    pub output_format: StatisticsFormat,
}

impl Default for StatisticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_millis(500),
            max_history_size: 1000,
            detailed_tracking: false,
            enable_profiling: true,
            output_format: StatisticsFormat::Json,
        }
    }
}

/// Statistics output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatisticsFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Binary format
    Binary,
    /// Human-readable format
    Human,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable fast path optimizations
    pub enable_fast_path: bool,
    /// Cache size for performance optimization
    pub cache_size: usize,
    /// Number of worker threads
    pub worker_threads: usize,
    /// Memory limit in bytes
    pub memory_limit: usize,
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Batch processing size
    pub batch_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_fast_path: false,
            cache_size: 1000,
            worker_threads: num_cpus::get(),
            memory_limit: 1024 * 1024 * 1024, // 1GB
            enable_parallel: true,
            batch_size: 100,
        }
    }
}

/// Configuration validation and utilities
impl QuantumMCTSConfig {
    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.mcts.max_iterations == 0 {
            return Err(ConfigValidationError::InvalidParameter("max_iterations must be > 0".to_string()));
        }

        if self.mcts.exploration_constant < 0.0 {
            return Err(ConfigValidationError::InvalidParameter("exploration_constant must be >= 0".to_string()));
        }

        if self.quantum.coherence_threshold < 0.0 || self.quantum.coherence_threshold > 1.0 {
            return Err(ConfigValidationError::InvalidParameter("coherence_threshold must be between 0 and 1".to_string()));
        }

        if self.performance.worker_threads == 0 {
            return Err(ConfigValidationError::InvalidParameter("worker_threads must be > 0".to_string()));
        }

        Ok(())
    }

    /// Get effective cache size based on memory limit
    pub fn effective_cache_size(&self) -> usize {
        let max_cache_size = self.performance.memory_limit / 1024; // Rough estimate
        self.performance.cache_size.min(max_cache_size)
    }

    /// Check if parallel processing should be enabled
    pub fn should_use_parallel(&self) -> bool {
        self.performance.enable_parallel && self.performance.worker_threads > 1
    }
}

/// Configuration validation errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigValidationError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("Configuration conflict: {0}")]
    ConfigurationConflict(String),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}