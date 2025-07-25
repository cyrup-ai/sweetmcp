//! Engine factory for creating optimized quantum entanglement engines
//!
//! This module provides factory patterns for creating quantum entanglement engines
//! optimized for different scenarios with blazing-fast zero-allocation initialization.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::{
    super::super::{
        config::QuantumMCTSConfig,
    },
    super::{
        metrics::EntanglementMetrics,
        analysis::NetworkTopologyAnalyzer,
    },
    core::QuantumEntanglementEngine,
};

/// Engine factory for creating optimized instances with specialized configurations
pub struct QuantumEntanglementEngineFactory;

impl QuantumEntanglementEngineFactory {
    /// Create engine optimized for high-performance scenarios
    pub fn create_high_performance(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating high-performance quantum entanglement engine");
        
        let mut optimized_config = config;
        optimized_config.configure_high_throughput();
        
        // High-performance optimizations
        optimized_config.max_concurrent_operations = 200;
        optimized_config.batch_operation_size = 100;
        optimized_config.enable_vectorized_operations = true;
        optimized_config.cache_size_multiplier = 4.0;
        optimized_config.prefetch_depth = 3;
        optimized_config.parallel_execution_threshold = 10;
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }
    
    /// Create engine optimized for low-latency scenarios
    pub fn create_low_latency(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating low-latency quantum entanglement engine");
        
        let mut optimized_config = config;
        optimized_config.configure_low_latency();
        
        // Low-latency optimizations
        optimized_config.max_concurrent_operations = 50;
        optimized_config.batch_operation_size = 10;
        optimized_config.operation_timeout_ms = 100;
        optimized_config.enable_preemptive_optimization = true;
        optimized_config.cache_warmup_enabled = true;
        optimized_config.priority_scheduling = true;
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }
    
    /// Create engine optimized for memory efficiency
    pub fn create_memory_efficient(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating memory-efficient quantum entanglement engine");
        
        let mut optimized_config = config;
        
        // Memory efficiency optimizations
        optimized_config.max_concurrent_operations = 20;
        optimized_config.batch_operation_size = 5;
        optimized_config.cache_size_multiplier = 0.5;
        optimized_config.enable_memory_pooling = true;
        optimized_config.garbage_collection_frequency = 1000;
        optimized_config.enable_compression = true;
        optimized_config.lazy_initialization = true;
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }
    
    /// Create engine with custom metrics and configuration
    pub fn create_custom(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        metrics: Arc<EntanglementMetrics>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating custom quantum entanglement engine");
        
        QuantumEntanglementEngine::with_metrics(config, entanglement_graph, metrics)
    }
    
    /// Create engine with automatic configuration based on network size
    pub async fn create_adaptive(
        base_config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Result<QuantumEntanglementEngine, CognitiveError> {
        debug!("Creating adaptive quantum entanglement engine");
        
        // Analyze current network to determine optimal configuration
        let topology = NetworkTopologyAnalyzer::analyze_network_topology(&entanglement_graph).await?;
        let mut adaptive_config = base_config;
        
        // Adapt configuration based on network characteristics
        if topology.total_nodes > 1000 {
            adaptive_config.configure_high_throughput();
            adaptive_config.max_concurrent_operations = 100;
            adaptive_config.batch_operation_size = 50;
            adaptive_config.enable_parallel_processing = true;
        } else if topology.total_nodes < 100 {
            adaptive_config.configure_low_latency();
            adaptive_config.max_concurrent_operations = 20;
            adaptive_config.batch_operation_size = 5;
            adaptive_config.enable_preemptive_optimization = true;
        } else {
            adaptive_config.enable_performance_optimizations();
            adaptive_config.max_concurrent_operations = 50;
            adaptive_config.batch_operation_size = 20;
            adaptive_config.enable_adaptive_batching = true;
        }
        
        // Adjust based on network density
        if topology.network_density > 0.1 {
            adaptive_config.entanglement_strength_threshold *= 1.5;
            adaptive_config.pruning_frequency_ms /= 2;
            adaptive_config.aggressive_pruning = true;
        } else if topology.network_density < 0.03 {
            adaptive_config.entanglement_strength_threshold *= 0.7;
            adaptive_config.creation_batch_size *= 2;
            adaptive_config.enable_creation_acceleration = true;
        }
        
        // Adjust based on network connectivity
        if !topology.is_connected {
            adaptive_config.connectivity_repair_priority = true;
            adaptive_config.bridge_creation_enabled = true;
            adaptive_config.isolation_detection_threshold = 0.1;
        }
        
        // Adjust based on average degree
        if topology.average_degree > 10.0 {
            adaptive_config.hub_node_optimization = true;
            adaptive_config.load_balancing_frequency_ms = 5000;
        } else if topology.average_degree < 2.0 {
            adaptive_config.sparse_network_optimization = true;
            adaptive_config.minimum_connectivity_threshold = 2.0;
        }
        
        Ok(QuantumEntanglementEngine::optimized(adaptive_config, entanglement_graph))
    }
    
    /// Create engine optimized for specific workload patterns
    pub fn create_for_workload(
        base_config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        workload_type: WorkloadType,
    ) -> QuantumEntanglementEngine {
        debug!("Creating engine optimized for {:?} workload", workload_type);
        
        let mut optimized_config = base_config;
        
        match workload_type {
            WorkloadType::ComputeIntensive => {
                optimized_config.max_quantum_parallel = num_cpus::get() * 2;
                optimized_config.quantum_exploration = 1.5;
                optimized_config.recursive_iterations = 5;
                optimized_config.enable_error_correction = false;
            }
            WorkloadType::MemoryIntensive => {
                optimized_config.max_tree_size = optimized_config.max_tree_size.saturating_mul(2);
                // Memory optimizations are handled internally by the engine
                optimized_config.quantum_exploration = 1.8; // More exploration for memory-intensive
                optimized_config.entanglement_strength = 0.8; // Stronger connections
            }
            WorkloadType::NetworkIntensive => {
                // Network optimizations are handled at a higher level
                optimized_config.quantum_exploration = 2.2; // More exploration
                optimized_config.decoherence_threshold = 0.2; // More tolerant to decoherence
            }
            WorkloadType::Balanced => {
                // Use default balanced settings
                optimized_config.quantum_exploration = 2.0;
                optimized_config.entanglement_strength = 0.7;
                optimized_config.decoherence_threshold = 0.1;
            }
        }
        
        QuantumEntanglementEngine::optimized(optimized_config, entanglement_graph)
    }
}

/// Workload type for specialized engine optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadType {
    /// CPU-intensive operations requiring computational optimization
    ComputeIntensive,
    /// Memory-intensive operations requiring memory optimization
    MemoryIntensive,
    /// Network-intensive operations requiring I/O optimization
    NetworkIntensive,
    /// Balanced workload requiring general optimization
    Balanced,
}

impl WorkloadType {
    /// Get description of workload type
    pub fn description(&self) -> &'static str {
        match self {
            WorkloadType::ComputeIntensive => "CPU-intensive computational workload",
            WorkloadType::MemoryIntensive => "Memory-intensive data processing workload",
            WorkloadType::NetworkIntensive => "Network-intensive I/O workload",
            WorkloadType::Balanced => "Balanced mixed workload",
        }
    }
    
    /// Get recommended configuration multipliers
    pub fn get_multipliers(&self) -> (f64, f64, f64) {
        match self {
            WorkloadType::ComputeIntensive => (2.0, 1.0, 1.0), // CPU, Memory, Network
            WorkloadType::MemoryIntensive => (1.0, 2.0, 1.0),
            WorkloadType::NetworkIntensive => (1.0, 1.0, 2.0),
            WorkloadType::Balanced => (1.2, 1.2, 1.2),
        }
    }
}