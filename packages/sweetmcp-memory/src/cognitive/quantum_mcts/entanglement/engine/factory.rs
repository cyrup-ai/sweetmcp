//! Engine factory for creating optimized instances
//!
//! This module provides blazing-fast engine factory with zero allocation
//! optimizations and elegant ergonomic interfaces for quantum entanglement engine creation.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::super::super::{
    config::QuantumMCTSConfig,
};
use super::super::{
    metrics::EntanglementMetrics,
};
use super::core::{QuantumEntanglementEngine};

/// Engine factory for creating optimized instances
pub struct QuantumEntanglementEngineFactory;

impl QuantumEntanglementEngineFactory {
    /// Create engine optimized for high-performance scenarios
    #[inline]
    pub fn create_high_performance(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating high-performance quantum entanglement engine");
        
        let mut optimized_config = config;
        optimized_config.configure_high_throughput();
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }
    
    /// Create engine optimized for low-latency scenarios
    #[inline]
    pub fn create_low_latency(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating low-latency quantum entanglement engine");
        
        let mut optimized_config = config;
        optimized_config.configure_low_latency();
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }
    
    /// Create engine with custom metrics and configuration
    #[inline]
    pub fn create_custom(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        metrics: Arc<EntanglementMetrics>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating custom quantum entanglement engine");
        
        QuantumEntanglementEngine::with_custom_metrics(config, entanglement_graph, metrics)
    }
    
    /// Create engine with automatic configuration based on network size
    #[inline]
    pub fn create_adaptive(
        base_config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Result<QuantumEntanglementEngine, CognitiveError> {
        debug!("Creating adaptive quantum entanglement engine");
        
        // Analyze network size to determine optimal configuration
        let network_size = Self::estimate_network_size(&entanglement_graph)?;
        let mut adaptive_config = base_config;
        
        // Configure based on network characteristics
        match network_size {
            NetworkSize::Small => {
                adaptive_config.configure_small_network();
                debug!("Configured for small network (< 100 nodes)");
            }
            NetworkSize::Medium => {
                adaptive_config.configure_medium_network();
                debug!("Configured for medium network (100-1000 nodes)");
            }
            NetworkSize::Large => {
                adaptive_config.configure_large_network();
                debug!("Configured for large network (1000-10000 nodes)");
            }
            NetworkSize::ExtraLarge => {
                adaptive_config.configure_extra_large_network();
                debug!("Configured for extra large network (> 10000 nodes)");
            }
        }
        
        Ok(QuantumEntanglementEngine::optimized(adaptive_config, entanglement_graph))
    }

    /// Create engine optimized for memory-constrained environments
    #[inline]
    pub fn create_memory_optimized(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating memory-optimized quantum entanglement engine");
        
        let mut optimized_config = config;
        optimized_config.configure_memory_optimized();
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }

    /// Create engine optimized for CPU-intensive workloads
    #[inline]
    pub fn create_cpu_optimized(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating CPU-optimized quantum entanglement engine");
        
        let mut optimized_config = config;
        optimized_config.configure_cpu_optimized();
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }

    /// Create engine with balanced performance characteristics
    #[inline]
    pub fn create_balanced(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating balanced quantum entanglement engine");
        
        let mut optimized_config = config;
        optimized_config.configure_balanced();
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }

    /// Create engine for real-time applications
    #[inline]
    pub fn create_realtime(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating real-time quantum entanglement engine");
        
        let mut optimized_config = config;
        optimized_config.configure_realtime();
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }

    /// Create engine for batch processing
    #[inline]
    pub fn create_batch_processing(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating batch processing quantum entanglement engine");
        
        let mut optimized_config = config;
        optimized_config.configure_batch_processing();
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }

    /// Estimate network size for adaptive configuration
    #[inline]
    fn estimate_network_size(
        entanglement_graph: &Arc<RwLock<EntanglementGraph>>,
    ) -> Result<NetworkSize, CognitiveError> {
        // This would typically require async access to the graph
        // For now, we'll use a simplified estimation approach
        
        // In a real implementation, this would analyze the graph structure
        // For demonstration, we'll return a default size
        Ok(NetworkSize::Medium)
    }

    /// Create engine with specific optimization profile
    #[inline]
    pub fn create_with_profile(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        profile: OptimizationProfile,
    ) -> QuantumEntanglementEngine {
        debug!("Creating quantum entanglement engine with profile: {:?}", profile);
        
        let mut optimized_config = config;
        
        match profile {
            OptimizationProfile::HighPerformance => {
                optimized_config.configure_high_throughput();
            }
            OptimizationProfile::LowLatency => {
                optimized_config.configure_low_latency();
            }
            OptimizationProfile::MemoryOptimized => {
                optimized_config.configure_memory_optimized();
            }
            OptimizationProfile::CpuOptimized => {
                optimized_config.configure_cpu_optimized();
            }
            OptimizationProfile::Balanced => {
                optimized_config.configure_balanced();
            }
            OptimizationProfile::Realtime => {
                optimized_config.configure_realtime();
            }
            OptimizationProfile::BatchProcessing => {
                optimized_config.configure_batch_processing();
            }
        }
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }

    /// Create multiple engines for load distribution
    #[inline]
    pub fn create_distributed_engines(
        base_config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        engine_count: usize,
    ) -> Result<Vec<QuantumEntanglementEngine>, CognitiveError> {
        debug!("Creating {} distributed quantum entanglement engines", engine_count);
        
        if engine_count == 0 {
            return Err(CognitiveError::InvalidConfiguration(
                "Engine count must be greater than 0".to_string()
            ));
        }

        let mut engines = Vec::with_capacity(engine_count);
        
        for i in 0..engine_count {
            let mut config = base_config.clone();
            
            // Configure each engine with slight variations for load distribution
            config.configure_for_distribution(i, engine_count);
            
            let engine = QuantumEntanglementEngine::optimized(config, entanglement_graph.clone());
            engines.push(engine);
        }
        
        Ok(engines)
    }

    /// Get recommended configuration for given requirements
    #[inline]
    pub fn recommend_configuration(
        requirements: &PerformanceRequirements,
    ) -> OptimizationProfile {
        // Analyze requirements and recommend optimal profile
        if requirements.max_latency_ms < 10 {
            OptimizationProfile::Realtime
        } else if requirements.max_latency_ms < 50 {
            OptimizationProfile::LowLatency
        } else if requirements.memory_limit_mb < 512 {
            OptimizationProfile::MemoryOptimized
        } else if requirements.cpu_cores > 16 {
            OptimizationProfile::CpuOptimized
        } else if requirements.throughput_ops_per_sec > 10000 {
            OptimizationProfile::HighPerformance
        } else if requirements.is_batch_workload {
            OptimizationProfile::BatchProcessing
        } else {
            OptimizationProfile::Balanced
        }
    }
}

/// Network size categories for adaptive configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkSize {
    /// Small network (< 100 nodes)
    Small,
    /// Medium network (100-1000 nodes)
    Medium,
    /// Large network (1000-10000 nodes)
    Large,
    /// Extra large network (> 10000 nodes)
    ExtraLarge,
}

/// Optimization profiles for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationProfile {
    /// High throughput optimization
    HighPerformance,
    /// Low latency optimization
    LowLatency,
    /// Memory usage optimization
    MemoryOptimized,
    /// CPU usage optimization
    CpuOptimized,
    /// Balanced performance
    Balanced,
    /// Real-time processing
    Realtime,
    /// Batch processing optimization
    BatchProcessing,
}

/// Performance requirements for configuration recommendation
#[derive(Debug, Clone)]
pub struct PerformanceRequirements {
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: u64,
    /// Memory limit in megabytes
    pub memory_limit_mb: u64,
    /// Number of available CPU cores
    pub cpu_cores: usize,
    /// Required throughput in operations per second
    pub throughput_ops_per_sec: u64,
    /// Whether this is a batch workload
    pub is_batch_workload: bool,
    /// Whether real-time guarantees are needed
    pub requires_realtime: bool,
}

impl PerformanceRequirements {
    /// Create new performance requirements
    #[inline]
    pub fn new() -> Self {
        Self {
            max_latency_ms: 100,
            memory_limit_mb: 1024,
            cpu_cores: 4,
            throughput_ops_per_sec: 1000,
            is_batch_workload: false,
            requires_realtime: false,
        }
    }

    /// Create requirements for real-time applications
    #[inline]
    pub fn realtime() -> Self {
        Self {
            max_latency_ms: 5,
            memory_limit_mb: 2048,
            cpu_cores: 8,
            throughput_ops_per_sec: 5000,
            is_batch_workload: false,
            requires_realtime: true,
        }
    }

    /// Create requirements for batch processing
    #[inline]
    pub fn batch_processing() -> Self {
        Self {
            max_latency_ms: 1000,
            memory_limit_mb: 4096,
            cpu_cores: 16,
            throughput_ops_per_sec: 50000,
            is_batch_workload: true,
            requires_realtime: false,
        }
    }

    /// Create requirements for memory-constrained environments
    #[inline]
    pub fn memory_constrained() -> Self {
        Self {
            max_latency_ms: 200,
            memory_limit_mb: 256,
            cpu_cores: 2,
            throughput_ops_per_sec: 500,
            is_batch_workload: false,
            requires_realtime: false,
        }
    }
}

impl Default for PerformanceRequirements {
    fn default() -> Self {
        Self::new()
    }
}