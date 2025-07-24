//! Quantum entanglement management integration module
//!
//! This module provides ergonomic re-exports and integration for all entanglement
//! components with zero-allocation patterns and blazing-fast performance.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};

// Import and re-export submodules from entanglement directory
// Note: Import directly from the entanglement directory, not through the alias
// The entanglement directory is at the same level as this file
use super::entanglement::{
    core, analysis, metrics, engine
};

// Import sibling modules
use super::entanglement_coordinator;
use super::entanglement_analysis;
use super::entanglement_factory;

// Re-export for public API
pub use entanglement_coordinator::*;
pub use entanglement_analysis::*;
pub use entanglement_factory::*;

// Re-export core entanglement functionality
pub use self::core::QuantumEntanglementManager;
pub use self::analysis::{
    NetworkTopology, 
    EntanglementQuality, 
    NetworkInfluenceCalculator,
    NeighborhoodAnalysis,
    NetworkTopologyAnalyzer,
    NetworkBottleneck,
    BottleneckType,
};
pub use self::metrics::{
    EntanglementMetrics,
    EntanglementMetricsSummary,
    PerformanceTracker,
    MetricsCollector,
    EntanglementBenchmark,
    BenchmarkResults,
};
pub use self::engine::{
    QuantumEntanglementEngine,
    OptimizationResult,
    PruningResult,
    NetworkHealthReport,
};

// Re-export decomposed modules
pub use entanglement_coordinator::EntanglementCoordinator;
pub use entanglement_analysis::{ComprehensiveAnalysisReport, AnalysisExportData, SerializableAnalysisData};
pub use entanglement_factory as factory;

/// Quick access functions for common operations
pub mod quick {
    use super::*;
    
    /// Create a balanced entanglement coordinator
    pub fn create_coordinator(
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> EntanglementCoordinator {
        factory::create_balanced_coordinator(entanglement_graph)
    }
    
    /// Create a high-performance entanglement coordinator
    pub fn create_fast_coordinator(
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> EntanglementCoordinator {
        factory::create_high_performance_coordinator(entanglement_graph)
    }
    
    /// Perform quick network analysis
    pub async fn quick_analysis(
        coordinator: &EntanglementCoordinator,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<String, CognitiveError> {
        let report = coordinator.comprehensive_analysis(tree).await?;
        Ok(report.condensed_report())
    }
    
    /// Check if network needs attention
    pub async fn needs_attention(
        coordinator: &EntanglementCoordinator,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<bool, CognitiveError> {
        let report = coordinator.comprehensive_analysis(tree).await?;
        Ok(!report.is_performing_well() || !report.critical_issues().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_entanglement_coordinator_creation() {
        let entanglement_graph = Arc::new(RwLock::new(EntanglementGraph::new()));
        let coordinator = EntanglementCoordinator::new(
            QuantumMCTSConfig::default(),
            entanglement_graph,
        );
        
        assert_eq!(coordinator.get_metrics().entanglements_created(), 0);
        assert_eq!(coordinator.get_config().decoherence_threshold, QuantumMCTSConfig::default().decoherence_threshold);
    }
    
    #[tokio::test]
    async fn test_comprehensive_analysis() {
        let entanglement_graph = Arc::new(RwLock::new(EntanglementGraph::new()));
        let coordinator = EntanglementCoordinator::new(
            QuantumMCTSConfig::default(),
            entanglement_graph,
        );
        
        let tree = HashMap::new(); 
        let analysis = coordinator.comprehensive_analysis(&tree).await.unwrap();
        
        assert_eq!(analysis.node_count, 0);
        assert!(analysis.analysis_time_ms > 0);
        assert!(analysis.overall_score() >= 0.0);
        assert!(analysis.overall_score() <= 1.0);
    }
    
    #[test]
    fn test_factory_functions() {
        let entanglement_graph = Arc::new(RwLock::new(EntanglementGraph::new()));
        
        let high_perf = factory::create_high_performance_coordinator(entanglement_graph.clone());
        assert!(high_perf.get_config().batch_size >= 50);
        
        let balanced = factory::create_balanced_coordinator(entanglement_graph.clone());
        assert_eq!(balanced.get_config().decoherence_threshold, QuantumMCTSConfig::default().decoherence_threshold);
        
        let conservative = factory::create_conservative_coordinator(entanglement_graph);
        assert!(conservative.get_config().decoherence_threshold <= 0.1);
    }
    
    #[tokio::test]
    async fn test_quick_functions() {
        let entanglement_graph = Arc::new(RwLock::new(EntanglementGraph::new()));
        let coordinator = quick::create_coordinator(entanglement_graph.clone());
        
        assert_eq!(coordinator.get_config().decoherence_threshold, QuantumMCTSConfig::default().decoherence_threshold);
        
        let fast_coordinator = quick::create_fast_coordinator(entanglement_graph);
        assert!(fast_coordinator.get_config().batch_size >= 50);
        
        let tree = HashMap::new();
        let analysis = quick::quick_analysis(&coordinator, &tree).await.unwrap();
        assert!(!analysis.is_empty());
        
        let needs_attention = quick::needs_attention(&coordinator, &tree).await.unwrap();
        assert!(!needs_attention); // Empty network should not need attention
    }
}
