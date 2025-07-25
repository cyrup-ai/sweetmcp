//! Quantum entanglement engine module integration
//!
//! This module provides ergonomic re-exports and integration for all quantum
//! entanglement components with zero allocation patterns and blazing-fast performance.

// Core engine modules
pub mod engine_core;
pub mod engine_operations;
pub mod engine_optimization;
pub mod engine_analysis;
pub mod engine_health;
pub mod engine_health_types;
pub mod engine_issue_types;
pub mod engine_issue_collection;

// Decomposed integration modules
pub mod entanglement_types;
pub mod entanglement_optimization_utils;
pub mod entanglement_health_utils;
pub mod entanglement_integration;

// Decomposed submodules - ensure proper directory structure
pub mod engine;
pub mod analysis;
pub mod metrics;

// Add missing core module with proper exports
pub mod core;

// Re-export core types for backward compatibility
pub use core::QuantumEntanglementManager;

// Re-export core types and functionality
pub use engine_core::QuantumEntanglementEngine;
pub use engine_operations::OptimizationResult;
pub use engine_optimization::OptimizationPrediction;
pub use engine_health::{EngineHealthReport, NetworkAnalysisReport, OptimizationPriority};
pub use engine_health::NetworkPerformanceMetrics;
pub use engine_health_types::{CriticalNode, CriticalityType, HealthStatus};
pub use engine_issue_types::{NetworkIssue, IssueSeverity, IssueCategory};
pub use engine_issue_collection::{IssueCollection, IssueSummaryStats};

// Re-export from decomposed submodules
pub use engine::*;
pub use analysis::*;
pub use metrics::*;
pub use core::*;

// Re-export entanglement types
pub use entanglement_types::{
    OptimizationStrategy, OptimizationUrgency, ComprehensiveHealthReport,
    HealthTrend, OptimizationContext,
};

// Re-export optimization utilities
pub use entanglement_optimization_utils::{
    calculate_composite_health_score, calculate_topology_health_score,
    calculate_metrics_health_score, recommend_optimization_strategy,
    calculate_optimization_urgency, create_optimization_context,
    analyze_health_trends, calculate_optimization_priority,
    identify_optimization_bottlenecks, OptimizationBottleneck,
    BottleneckType, BottleneckSeverity,
};

// Re-export health utilities
pub use entanglement_health_utils::{
    create_comprehensive_health_report, analyze_detailed_health_trends,
    generate_health_summary, detect_health_anomalies,
    DetailedHealthTrend, HealthSummaryStats, HealthAnomaly,
    AnomalyType, AnomalySeverity,
};

// Re-export analysis types from local modules
pub use self::analysis::NetworkTopology;
pub use self::metrics::EntanglementMetrics;

/// Convenience constructor for quantum entanglement engine
pub fn create_quantum_entanglement_engine(
    manager: std::sync::Arc<self::core::QuantumEntanglementManager>,
    analyzer: std::sync::Arc<self::analysis::NetworkTopologyAnalyzer>,
    config: crate::cognitive::quantum_mcts::config::QuantumMCTSConfig,
) -> QuantumEntanglementEngine {
    QuantumEntanglementEngine::new(manager, analyzer, config)
}

/// Create engine with default configuration
pub fn create_default_quantum_entanglement_engine(
    manager: std::sync::Arc<self::core::QuantumEntanglementManager>,
    analyzer: std::sync::Arc<self::analysis::NetworkTopologyAnalyzer>,
) -> QuantumEntanglementEngine {
    let config = crate::cognitive::quantum_mcts::config::QuantumMCTSConfig::default();
    QuantumEntanglementEngine::new(manager, analyzer, config)
}

/// Create optimized engine configuration for high-performance scenarios
pub fn create_high_performance_engine(
    manager: std::sync::Arc<self::core::QuantumEntanglementManager>,
    analyzer: std::sync::Arc<self::analysis::NetworkTopologyAnalyzer>,
) -> QuantumEntanglementEngine {
    let mut config = crate::cognitive::quantum_mcts::config::QuantumMCTSConfig::default();
    
    // Optimize for performance
    config.optimization_frequency = std::time::Duration::from_secs(300); // 5 minutes
    config.max_optimization_iterations = 1000;
    config.enable_parallel_processing = true;
    config.cache_analysis_results = true;
    
    QuantumEntanglementEngine::new(manager, analyzer, config)
}

/// Create memory-optimized engine configuration for resource-constrained scenarios
pub fn create_memory_optimized_engine(
    manager: std::sync::Arc<self::core::QuantumEntanglementManager>,
    analyzer: std::sync::Arc<self::analysis::NetworkTopologyAnalyzer>,
) -> QuantumEntanglementEngine {
    let mut config = crate::cognitive::quantum_mcts::config::QuantumMCTSConfig::default();
    
    // Optimize for memory usage
    config.max_cached_reports = 10;
    config.enable_compression = true;
    config.lazy_evaluation = true;
    config.gc_frequency = std::time::Duration::from_secs(60);
    
    QuantumEntanglementEngine::new(manager, analyzer, config)
}

/// Engine factory for different use cases
pub struct EngineFactory;

impl EngineFactory {
    /// Create engine optimized for real-time applications
    pub fn real_time_engine(
        manager: std::sync::Arc<self::core::QuantumEntanglementManager>,
        analyzer: std::sync::Arc<self::analysis::NetworkTopologyAnalyzer>,
    ) -> QuantumEntanglementEngine {
        let mut config = crate::cognitive::quantum_mcts::config::QuantumMCTSConfig::default();
        
        config.optimization_frequency = std::time::Duration::from_millis(100);
        config.max_optimization_time = std::time::Duration::from_millis(50);
        config.enable_fast_path_optimization = true;
        config.priority_based_scheduling = true;
        
        QuantumEntanglementEngine::new(manager, analyzer, config)
    }

    /// Create engine optimized for batch processing
    pub fn batch_processing_engine(
        manager: std::sync::Arc<self::core::QuantumEntanglementManager>,
        analyzer: std::sync::Arc<self::analysis::NetworkTopologyAnalyzer>,
    ) -> QuantumEntanglementEngine {
        let mut config = crate::cognitive::quantum_mcts::config::QuantumMCTSConfig::default();
        
        config.optimization_frequency = std::time::Duration::from_secs(3600); // 1 hour
        config.max_optimization_iterations = 10000;
        config.enable_deep_analysis = true;
        config.batch_size = 1000;
        
        QuantumEntanglementEngine::new(manager, analyzer, config)
    }

    /// Create engine optimized for research and analysis
    pub fn research_engine(
        manager: std::sync::Arc<self::core::QuantumEntanglementManager>,
        analyzer: std::sync::Arc<self::analysis::NetworkTopologyAnalyzer>,
    ) -> QuantumEntanglementEngine {
        let mut config = crate::cognitive::quantum_mcts::config::QuantumMCTSConfig::default();
        
        config.enable_detailed_logging = true;
        config.collect_performance_metrics = true;
        config.enable_experimental_features = true;
        config.max_analysis_depth = 100;
        
        QuantumEntanglementEngine::new(manager, analyzer, config)
    }
}

/// Quick health check utilities
pub mod quick_health {
    use super::*;
    
    /// Perform quick health assessment
    pub fn quick_health_check(
        topology: &NetworkTopology,
        metrics: &EntanglementMetrics,
        issues: &IssueCollection,
    ) -> QuickHealthResult {
        let composite_score = calculate_composite_health_score(topology, metrics, issues);
        let strategy = recommend_optimization_strategy(topology, metrics, issues);
        let urgency = calculate_optimization_urgency(topology, issues);
        
        QuickHealthResult {
            health_score: composite_score,
            health_grade: grade_from_score(composite_score),
            strategy,
            urgency,
            requires_attention: composite_score < 0.7 || urgency.is_critical(),
        }
    }
    
    /// Convert health score to letter grade
    fn grade_from_score(score: f64) -> char {
        if score >= 0.9 {
            'A'
        } else if score >= 0.8 {
            'B'
        } else if score >= 0.7 {
            'C'
        } else if score >= 0.6 {
            'D'
        } else {
            'F'
        }
    }
}

/// Quick health check result
#[derive(Debug, Clone)]
pub struct QuickHealthResult {
    pub health_score: f64,
    pub health_grade: char,
    pub strategy: OptimizationStrategy,
    pub urgency: OptimizationUrgency,
    pub requires_attention: bool,
}

impl QuickHealthResult {
    /// Get summary description
    pub fn summary(&self) -> String {
        format!(
            "Health: {} ({:.1}%), Strategy: {:?}, Urgency: {:?}{}",
            self.health_grade,
            self.health_score * 100.0,
            self.strategy,
            self.urgency,
            if self.requires_attention { " - ATTENTION REQUIRED" } else { "" }
        )
    }
    
    /// Check if system is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_score >= 0.7 && !self.requires_attention
    }
}