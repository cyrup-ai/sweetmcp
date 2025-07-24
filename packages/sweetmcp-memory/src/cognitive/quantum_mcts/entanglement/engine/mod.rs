//! Quantum entanglement engine coordination module
//!
//! This module provides comprehensive coordination of all quantum entanglement
//! engine submodules with blazing-fast zero-allocation patterns and elegant
//! ergonomic interfaces.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::{
    core::QuantumEntanglementManager,
    analysis::{NetworkTopology, NetworkTopologyAnalyzer},
    metrics::{EntanglementMetrics, PerformanceTracker},
};
use super::super::{
    config::QuantumMCTSConfig,
    node_state::QuantumMCTSNode,
};

// Declare all engine submodules
pub mod operation_types;
pub mod engine_factory;
pub mod operation_executor;
pub mod combined_optimization;
pub mod maintenance_statistics;
pub mod maintenance_assessment;
pub mod performance_grading;

// Import submodule functionality (removed duplicate imports - using pub use instead)

// Re-export all types for ergonomic usage
pub use self::{
    operation_types::{EngineOperationType, EngineOperationResult, EngineOperationDetails},
    engine_factory::{QuantumEntanglementEngineFactory, WorkloadType},
    maintenance_statistics::{EngineStatistics, MaintenanceAssessment, MaintenancePriority, MaintenanceAction},
    maintenance_assessment::{MaintenancePlan, ResourceRequirements},
    performance_grading::{PerformanceGrades, EnginePerformanceReport, TrendDirection},
    balancing::{BalancingResult, NodeBalance, BalancingStrategy, NetworkBalanceAnalysis, DistributionStatistics},
    core::{QuantumEntanglementEngine, EngineStatus},
    optimization::{OptimizationResult, CreationResult},
    pruning::{PruningResult, PruningStrategy, StrengthStatistics, RecentPruningStatistics},
    health::{NetworkHealthReport, HealthCheckConfig, HealthIssue, IssueSeverity},
};

// Import engine core components
mod core;
mod optimization;
mod pruning;
mod balancing;
mod health;

/// Engine coordination facade for simplified high-level operations
pub struct EngineCoordinator {
    /// Engine factory for creating optimized instances
    factory: QuantumEntanglementEngineFactory,
    /// Performance tracking
    performance_tracker: Arc<PerformanceTracker>,
}

impl EngineCoordinator {
    /// Create new engine coordinator
    pub fn new() -> Self {
        Self {
            factory: QuantumEntanglementEngineFactory,
            performance_tracker: Arc::new(PerformanceTracker::new()),
        }
    }
    
    /// Create high-performance engine instance
    pub fn create_high_performance_engine(
        &self,
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> QuantumEntanglementEngine {
        debug!("Creating high-performance engine through coordinator");
        QuantumEntanglementEngineFactory::create_high_performance(config, entanglement_graph)
    }
    
    /// Create adaptive engine instance
    pub async fn create_adaptive_engine(
        &self,
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Result<QuantumEntanglementEngine, CognitiveError> {
        debug!("Creating adaptive engine through coordinator");
        QuantumEntanglementEngineFactory::create_adaptive(config, entanglement_graph).await
    }
    
    /// Create engine optimized for specific workload
    pub fn create_workload_optimized_engine(
        &self,
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        workload_type: WorkloadType,
    ) -> QuantumEntanglementEngine {
        debug!("Creating workload-optimized engine: {:?}", workload_type);
        QuantumEntanglementEngineFactory::create_for_workload(config, entanglement_graph, workload_type)
    }
    
    /// Perform comprehensive engine assessment
    pub async fn assess_engine_performance(
        &self,
        engine: &QuantumEntanglementEngine,
    ) -> Result<EngineAssessment, CognitiveError> {
        debug!("Performing comprehensive engine assessment");
        
        let performance_report = engine.create_performance_report().await?;
        let maintenance_assessment = engine.assess_maintenance_needs().await?;
        let statistics = engine.get_comprehensive_statistics().await?;
        
        Ok(EngineAssessment {
            performance_report,
            maintenance_assessment,
            statistics,
            overall_health_score: Self::calculate_overall_health(&performance_report, &statistics),
            recommendations: Self::generate_comprehensive_recommendations(&performance_report, &maintenance_assessment),
        })
    }
    
    /// Calculate overall health score
    fn calculate_overall_health(
        report: &EnginePerformanceReport,
        statistics: &EngineStatistics,
    ) -> f64 {
        let performance_score = report.performance_grades.grade_point_average() / 4.0;
        let health_score = statistics.health_score;
        let efficiency_score = statistics.efficiency_score();
        
        (performance_score * 0.4 + health_score * 0.4 + efficiency_score * 0.2).min(1.0)
    }
    
    /// Generate comprehensive recommendations
    fn generate_comprehensive_recommendations(
        performance_report: &EnginePerformanceReport,
        maintenance_assessment: &MaintenanceAssessment,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Add performance recommendations
        recommendations.extend(performance_report.get_improvement_recommendations());
        
        // Add maintenance recommendations
        if maintenance_assessment.is_urgent() {
            recommendations.push("URGENT: Perform maintenance operations immediately".to_string());
        }
        
        for action in &maintenance_assessment.required_actions {
            recommendations.push(format!("Maintenance: {}", action.description()));
        }
        
        // Add optimization recommendations based on grades
        if performance_report.performance_grades.overall < 'C' {
            recommendations.push("Consider engine reconfiguration or replacement".to_string());
        } else if performance_report.performance_grades.overall < 'B' {
            recommendations.push("Schedule comprehensive optimization session".to_string());
        }
        
        recommendations
    }
}

impl Default for EngineCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive engine assessment result
#[derive(Debug, Clone)]
pub struct EngineAssessment {
    /// Performance report
    pub performance_report: EnginePerformanceReport,
    /// Maintenance assessment
    pub maintenance_assessment: MaintenanceAssessment,
    /// Engine statistics
    pub statistics: EngineStatistics,
    /// Overall health score (0.0 to 1.0)
    pub overall_health_score: f64,
    /// Comprehensive recommendations
    pub recommendations: Vec<String>,
}

impl EngineAssessment {
    /// Check if engine is performing optimally
    pub fn is_optimal(&self) -> bool {
        self.overall_health_score > 0.9 && 
        self.performance_report.performance_grades.is_excellent() &&
        !self.maintenance_assessment.is_urgent()
    }
    
    /// Check if engine needs immediate attention
    pub fn needs_immediate_attention(&self) -> bool {
        self.overall_health_score < 0.5 ||
        self.performance_report.performance_grades.overall <= 'D' ||
        self.maintenance_assessment.is_urgent()
    }
    
    /// Get assessment summary
    pub fn summary(&self) -> String {
        format!(
            "Engine Assessment: Health {:.1}/10, Grade {}, {} recommendations",
            self.overall_health_score * 10.0,
            self.performance_report.performance_grades.overall,
            self.recommendations.len()
        )
    }
    
    /// Format detailed assessment report
    pub fn format_detailed_report(&self) -> String {
        format!(
            "=== Comprehensive Engine Assessment ===\n\
            Overall Health Score: {:.2}/10\n\
            {}\n\
            {}\n\
            {}\n\
            \n\
            === Recommendations ===\n\
            {}",
            self.overall_health_score * 10.0,
            self.performance_report.performance_summary(),
            self.maintenance_assessment.summary(),
            self.statistics.performance_summary(),
            self.recommendations.join("\n")
        )
    }
}

/// Engine operation utilities for common tasks
pub struct EngineOperationUtils;

impl EngineOperationUtils {
    /// Execute engine operation with automatic error recovery
    pub async fn execute_with_recovery(
        engine: &mut QuantumEntanglementEngine,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        operation_type: EngineOperationType,
        max_retries: u32,
    ) -> Result<EngineOperationResult, CognitiveError> {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match engine.perform_comprehensive_operation(tree, operation_type.clone()).await {
                Ok(result) => {
                    if attempt > 0 {
                        info!("Engine operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        debug!("Engine operation failed, retrying: attempt {}/{}", attempt + 1, max_retries + 1);
                        // Simple backoff strategy
                        tokio::time::sleep(tokio::time::Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| CognitiveError::OperationFailed("Maximum retries exceeded".to_string())))
    }
    
    /// Batch execute multiple operations
    pub async fn batch_execute(
        engine: &mut QuantumEntanglementEngine,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        operations: Vec<EngineOperationType>,
    ) -> Vec<Result<EngineOperationResult, CognitiveError>> {
        let mut results = Vec::with_capacity(operations.len());
        
        for operation in operations {
            let result = engine.perform_comprehensive_operation(tree, operation).await;
            results.push(result);
        }
        
        results
    }
}