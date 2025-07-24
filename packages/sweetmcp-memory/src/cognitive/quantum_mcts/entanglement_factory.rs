//! Factory functions for creating entanglement coordinators with different configurations
//!
//! This module provides convenient factory functions to create EntanglementCoordinator
//! instances optimized for different use cases with zero-allocation patterns and
//! blazing-fast performance.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cognitive::quantum::EntanglementGraph;
use super::{
    config::QuantumMCTSConfig,
    entanglement_coordinator::EntanglementCoordinator,
};

/// High-performance coordinator optimized for speed
pub fn create_high_performance_coordinator(
    entanglement_graph: Arc<RwLock<EntanglementGraph>>,
) -> EntanglementCoordinator {
    let config = QuantumMCTSConfig {
        decoherence_threshold: 0.3,
        amplitude_threshold: 0.2,
        entanglement_strength: 0.8,
        max_entanglements_per_node: 20,
        pruning_threshold: 0.15,
        batch_size: 50,
        ..Default::default()
    };
    
    EntanglementCoordinator::with_reporting_interval(
        config, 
        entanglement_graph, 
        std::time::Duration::from_secs(30)
    )
}

/// Balanced coordinator with moderate settings
pub fn create_balanced_coordinator(
    entanglement_graph: Arc<RwLock<EntanglementGraph>>,
) -> EntanglementCoordinator {
    EntanglementCoordinator::new(QuantumMCTSConfig::default(), entanglement_graph)
}

/// Conservative coordinator optimized for quality
pub fn create_conservative_coordinator(
    entanglement_graph: Arc<RwLock<EntanglementGraph>>,
) -> EntanglementCoordinator {
    let config = QuantumMCTSConfig {
        decoherence_threshold: 0.1,
        amplitude_threshold: 0.5,
        entanglement_strength: 0.6,
        max_entanglements_per_node: 8,
        pruning_threshold: 0.3,
        batch_size: 10,
        ..Default::default()
    };
    
    EntanglementCoordinator::with_reporting_interval(
        config, 
        entanglement_graph, 
        std::time::Duration::from_secs(120)
    )
}

/// Memory-optimized coordinator for resource-constrained environments
pub fn create_memory_optimized_coordinator(
    entanglement_graph: Arc<RwLock<EntanglementGraph>>,
) -> EntanglementCoordinator {
    let config = QuantumMCTSConfig {
        decoherence_threshold: 0.2,
        amplitude_threshold: 0.3,
        entanglement_strength: 0.5,
        max_entanglements_per_node: 5,
        pruning_threshold: 0.4,
        batch_size: 5,
        ..Default::default()
    };
    
    EntanglementCoordinator::with_reporting_interval(
        config, 
        entanglement_graph, 
        std::time::Duration::from_secs(300)
    )
}

/// Research coordinator with extensive monitoring and analysis
pub fn create_research_coordinator(
    entanglement_graph: Arc<RwLock<EntanglementGraph>>,
) -> EntanglementCoordinator {
    let config = QuantumMCTSConfig {
        decoherence_threshold: 0.05,
        amplitude_threshold: 0.7,
        entanglement_strength: 0.9,
        max_entanglements_per_node: 100,
        pruning_threshold: 0.05,
        batch_size: 100,
        ..Default::default()
    };
    
    EntanglementCoordinator::with_reporting_interval(
        config, 
        entanglement_graph, 
        std::time::Duration::from_secs(10)
    )
}

/// Custom coordinator with user-specified configuration
pub fn create_custom_coordinator(
    entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    config: QuantumMCTSConfig,
    reporting_interval: std::time::Duration,
) -> EntanglementCoordinator {
    EntanglementCoordinator::with_reporting_interval(
        config, 
        entanglement_graph, 
        reporting_interval
    )
}

/// Coordinator preset configurations
pub mod presets {
    use super::*;
    
    /// Get high-performance configuration
    pub fn high_performance_config() -> QuantumMCTSConfig {
        QuantumMCTSConfig {
            decoherence_threshold: 0.3,
            amplitude_threshold: 0.2,
            entanglement_strength: 0.8,
            max_entanglements_per_node: 20,
            pruning_threshold: 0.15,
            batch_size: 50,
            ..Default::default()
        }
    }
    
    /// Get balanced configuration
    pub fn balanced_config() -> QuantumMCTSConfig {
        QuantumMCTSConfig::default()
    }
    
    /// Get conservative configuration
    pub fn conservative_config() -> QuantumMCTSConfig {
        QuantumMCTSConfig {
            decoherence_threshold: 0.1,
            amplitude_threshold: 0.5,
            entanglement_strength: 0.6,
            max_entanglements_per_node: 8,
            pruning_threshold: 0.3,
            batch_size: 10,
            ..Default::default()
        }
    }
    
    /// Get memory-optimized configuration
    pub fn memory_optimized_config() -> QuantumMCTSConfig {
        QuantumMCTSConfig {
            decoherence_threshold: 0.2,
            amplitude_threshold: 0.3,
            entanglement_strength: 0.5,
            max_entanglements_per_node: 5,
            pruning_threshold: 0.4,
            batch_size: 5,
            ..Default::default()
        }
    }
    
    /// Get research configuration
    pub fn research_config() -> QuantumMCTSConfig {
        QuantumMCTSConfig {
            decoherence_threshold: 0.05,
            amplitude_threshold: 0.7,
            entanglement_strength: 0.9,
            max_entanglements_per_node: 100,
            pruning_threshold: 0.05,
            batch_size: 100,
            ..Default::default()
        }
    }
}

/// Configuration validation utilities
pub mod validation {
    use super::*;
    
    /// Validate configuration parameters
    pub fn validate_config(config: &QuantumMCTSConfig) -> Result<(), String> {
        if config.decoherence_threshold < 0.0 || config.decoherence_threshold > 1.0 {
            return Err("Decoherence threshold must be between 0.0 and 1.0".to_string());
        }
        
        if config.amplitude_threshold < 0.0 || config.amplitude_threshold > 1.0 {
            return Err("Amplitude threshold must be between 0.0 and 1.0".to_string());
        }
        
        if config.entanglement_strength < 0.0 || config.entanglement_strength > 1.0 {
            return Err("Entanglement strength must be between 0.0 and 1.0".to_string());
        }
        
        if config.max_entanglements_per_node == 0 {
            return Err("Max entanglements per node must be greater than 0".to_string());
        }
        
        if config.pruning_threshold < 0.0 || config.pruning_threshold > 1.0 {
            return Err("Pruning threshold must be between 0.0 and 1.0".to_string());
        }
        
        if config.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }
        
        Ok(())
    }
    
    /// Get configuration recommendations based on use case
    pub fn recommend_config(use_case: &str) -> Option<QuantumMCTSConfig> {
        match use_case.to_lowercase().as_str() {
            "high_performance" | "speed" | "fast" => Some(presets::high_performance_config()),
            "balanced" | "default" | "moderate" => Some(presets::balanced_config()),
            "conservative" | "quality" | "accurate" => Some(presets::conservative_config()),
            "memory_optimized" | "low_memory" | "constrained" => Some(presets::memory_optimized_config()),
            "research" | "analysis" | "detailed" => Some(presets::research_config()),
            _ => None,
        }
    }
    
    /// Optimize configuration for specific constraints
    pub fn optimize_for_constraints(
        base_config: QuantumMCTSConfig,
        max_memory_mb: Option<usize>,
        max_latency_ms: Option<u64>,
        min_quality: Option<f64>,
    ) -> QuantumMCTSConfig {
        let mut config = base_config;
        
        // Memory constraints
        if let Some(max_mem) = max_memory_mb {
            if max_mem < 100 {
                config.max_entanglements_per_node = config.max_entanglements_per_node.min(5);
                config.batch_size = config.batch_size.min(5);
            } else if max_mem < 500 {
                config.max_entanglements_per_node = config.max_entanglements_per_node.min(10);
                config.batch_size = config.batch_size.min(20);
            }
        }
        
        // Latency constraints
        if let Some(max_lat) = max_latency_ms {
            if max_lat < 10 {
                config.batch_size = config.batch_size.min(10);
                config.pruning_threshold = config.pruning_threshold.max(0.3);
            } else if max_lat < 50 {
                config.batch_size = config.batch_size.min(25);
                config.pruning_threshold = config.pruning_threshold.max(0.2);
            }
        }
        
        // Quality constraints
        if let Some(min_qual) = min_quality {
            if min_qual > 0.8 {
                config.decoherence_threshold = config.decoherence_threshold.min(0.1);
                config.amplitude_threshold = config.amplitude_threshold.max(0.5);
                config.entanglement_strength = config.entanglement_strength.max(0.7);
                config.pruning_threshold = config.pruning_threshold.min(0.2);
            }
        }
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_factory_functions() {
        let entanglement_graph = Arc::new(RwLock::new(EntanglementGraph::new()));
        
        let high_perf = create_high_performance_coordinator(entanglement_graph.clone());
        assert!(high_perf.get_config().batch_size >= 50);
        
        let balanced = create_balanced_coordinator(entanglement_graph.clone());
        assert_eq!(balanced.get_config().decoherence_threshold, QuantumMCTSConfig::default().decoherence_threshold);
        
        let conservative = create_conservative_coordinator(entanglement_graph.clone());
        assert!(conservative.get_config().decoherence_threshold <= 0.1);
        
        let memory_opt = create_memory_optimized_coordinator(entanglement_graph.clone());
        assert!(memory_opt.get_config().max_entanglements_per_node <= 5);
        
        let research = create_research_coordinator(entanglement_graph);
        assert!(research.get_config().max_entanglements_per_node >= 100);
    }
    
    #[test]
    fn test_config_validation() {
        let valid_config = presets::balanced_config();
        assert!(validation::validate_config(&valid_config).is_ok());
        
        let mut invalid_config = valid_config;
        invalid_config.decoherence_threshold = 1.5;
        assert!(validation::validate_config(&invalid_config).is_err());
    }
    
    #[test]
    fn test_config_recommendations() {
        assert!(validation::recommend_config("high_performance").is_some());
        assert!(validation::recommend_config("balanced").is_some());
        assert!(validation::recommend_config("conservative").is_some());
        assert!(validation::recommend_config("invalid_use_case").is_none());
    }
}
