//! Engine operation types and result structures
//!
//! This module provides comprehensive type definitions for quantum entanglement
//! engine operations with blazing-fast zero-allocation patterns and elegant
//! ergonomic interfaces.

use std::time::Instant;
use super::{
    core::EngineStatus,
    optimization::OptimizationResult,
    pruning::PruningResult,
    balancing::BalancingResult,
    health::NetworkHealthReport,
    super::super::{
        analysis::NetworkTopology,
        metrics::EntanglementMetricsSummary,
    },
};

/// Comprehensive engine operation result with performance metrics
#[derive(Debug, Clone)]
pub struct EngineOperationResult {
    /// Operation type performed
    pub operation_type: EngineOperationType,
    /// Time taken for operation in milliseconds
    pub operation_time_ms: u64,
    /// Whether operation was successful
    pub success: bool,
    /// Performance improvement achieved
    pub performance_improvement: f64,
    /// Detailed operation results
    pub details: EngineOperationDetails,
    /// Timestamp of operation
    pub timestamp: Instant,
}

impl EngineOperationResult {
    /// Create new operation result
    pub fn new(
        operation_type: EngineOperationType,
        operation_time_ms: u64,
        success: bool,
        performance_improvement: f64,
        details: EngineOperationDetails,
        timestamp: Instant,
    ) -> Self {
        Self {
            operation_type,
            operation_time_ms,
            success,
            performance_improvement,
            details,
            timestamp,
        }
    }

    /// Check if operation was highly successful
    pub fn was_highly_successful(&self) -> bool {
        self.success && self.performance_improvement > 10.0
    }
    
    /// Get operation summary
    pub fn summary(&self) -> String {
        format!(
            "{:?}: {} in {}ms (+{:.1}% improvement)",
            self.operation_type,
            if self.success { "SUCCESS" } else { "PARTIAL" },
            self.operation_time_ms,
            self.performance_improvement
        )
    }
    
    /// Get detailed report
    pub fn detailed_report(&self) -> String {
        let mut report = format!(
            "=== Engine Operation Report ===\n\
            Operation: {:?}\n\
            Duration: {}ms\n\
            Success: {}\n\
            Performance Improvement: {:.1}%\n\
            Timestamp: {:?}\n\n",
            self.operation_type,
            self.operation_time_ms,
            self.success,
            self.performance_improvement,
            self.timestamp
        );
        
        match &self.details {
            EngineOperationDetails::Optimization(opt) => {
                report.push_str(&opt.detailed_report());
            }
            EngineOperationDetails::Creation(creation) => {
                report.push_str(&creation.summary());
            }
            EngineOperationDetails::Pruning(pruning) => {
                report.push_str(&pruning.detailed_report());
            }
            EngineOperationDetails::Balancing(balancing) => {
                report.push_str(&balancing.detailed_report());
            }
            EngineOperationDetails::HealthCheck(health) => {
                report.push_str(&health.format_report());
            }
            EngineOperationDetails::Combined { optimization, creation, pruning, balancing, health } => {
                report.push_str("--- Combined Operation Results ---\n");
                if let Some(opt) = optimization {
                    report.push_str(&format!("Optimization: {}\n", opt.summary()));
                }
                if let Some(create) = creation {
                    report.push_str(&format!("Creation: {}\n", create.summary()));
                }
                if let Some(prune) = pruning {
                    report.push_str(&format!("Pruning: {}\n", prune.summary()));
                }
                if let Some(balance) = balancing {
                    report.push_str(&format!("Balancing: {}\n", balance.summary()));
                }
                if let Some(health_check) = health {
                    report.push_str(&format!("Health: {}\n", health_check.summary()));
                }
            }
        }
        
        report
    }
}

/// Types of engine operations with optimization focus
#[derive(Debug, Clone, PartialEq)]
pub enum EngineOperationType {
    /// Full optimization operation
    FullOptimization,
    /// Strategic entanglement creation
    StrategyCreation,
    /// Intelligent pruning
    IntelligentPruning,
    /// Load balancing
    LoadBalancing,
    /// Health check
    HealthCheck,
    /// Combined optimization
    CombinedOptimization,
}

impl EngineOperationType {
    /// Get operation description
    pub fn description(&self) -> &'static str {
        match self {
            EngineOperationType::FullOptimization => "Comprehensive entanglement network optimization",
            EngineOperationType::StrategyCreation => "Strategic entanglement creation for network growth",
            EngineOperationType::IntelligentPruning => "Intelligent pruning of weak entanglements",
            EngineOperationType::LoadBalancing => "Load balancing of entanglement distribution",
            EngineOperationType::HealthCheck => "Network health assessment and diagnostics",
            EngineOperationType::CombinedOptimization => "Combined multi-step optimization workflow",
        }
    }
    
    /// Get expected operation complexity (1-5 scale)
    pub fn complexity_level(&self) -> u8 {
        match self {
            EngineOperationType::HealthCheck => 1,
            EngineOperationType::IntelligentPruning => 2,
            EngineOperationType::LoadBalancing => 3,
            EngineOperationType::StrategyCreation => 3,
            EngineOperationType::FullOptimization => 4,
            EngineOperationType::CombinedOptimization => 5,
        }
    }
    
    /// Check if operation modifies network structure
    pub fn modifies_network(&self) -> bool {
        matches!(self, 
            EngineOperationType::FullOptimization |
            EngineOperationType::StrategyCreation |
            EngineOperationType::IntelligentPruning |
            EngineOperationType::CombinedOptimization
        )
    }
}

/// Detailed operation results with type-specific information
#[derive(Debug, Clone)]
pub enum EngineOperationDetails {
    /// Optimization operation details
    Optimization(OptimizationResult),
    /// Creation operation details
    Creation(super::optimization::CreationResult),
    /// Pruning operation details
    Pruning(PruningResult),
    /// Balancing operation details
    Balancing(BalancingResult),
    /// Health check details
    HealthCheck(NetworkHealthReport),
    /// Combined operation details
    Combined {
        optimization: Option<OptimizationResult>,
        creation: Option<super::optimization::CreationResult>,
        pruning: Option<PruningResult>,
        balancing: Option<BalancingResult>,
        health: Option<NetworkHealthReport>,
    },
}

impl EngineOperationDetails {
    /// Get summary of operation details
    pub fn summary(&self) -> String {
        match self {
            EngineOperationDetails::Optimization(opt) => opt.summary(),
            EngineOperationDetails::Creation(creation) => creation.summary(),
            EngineOperationDetails::Pruning(pruning) => pruning.summary(),
            EngineOperationDetails::Balancing(balancing) => balancing.summary(),
            EngineOperationDetails::HealthCheck(health) => health.summary(),
            EngineOperationDetails::Combined { optimization, creation, pruning, balancing, health } => {
                let mut parts = Vec::new();
                if let Some(opt) = optimization {
                    parts.push(format!("Opt: {}", opt.summary()));
                }
                if let Some(create) = creation {
                    parts.push(format!("Create: {}", create.summary()));
                }
                if let Some(prune) = pruning {
                    parts.push(format!("Prune: {}", prune.summary()));
                }
                if let Some(balance) = balancing {
                    parts.push(format!("Balance: {}", balance.summary()));
                }
                if let Some(health_check) = health {
                    parts.push(format!("Health: {}", health_check.summary()));
                }
                parts.join(", ")
            }
        }
    }
    
    /// Check if details indicate successful operation
    pub fn indicates_success(&self) -> bool {
        match self {
            EngineOperationDetails::Optimization(opt) => opt.was_beneficial(),
            EngineOperationDetails::Creation(creation) => creation.was_successful(),
            EngineOperationDetails::Pruning(pruning) => pruning.was_beneficial(),
            EngineOperationDetails::Balancing(balancing) => balancing.was_successful(),
            EngineOperationDetails::HealthCheck(health) => health.is_healthy(),
            EngineOperationDetails::Combined { optimization, creation, pruning, balancing, health } => {
                let mut success_count = 0;
                let mut total_count = 0;
                
                if let Some(opt) = optimization {
                    total_count += 1;
                    if opt.was_beneficial() { success_count += 1; }
                }
                if let Some(create) = creation {
                    total_count += 1;
                    if create.was_successful() { success_count += 1; }
                }
                if let Some(prune) = pruning {
                    total_count += 1;
                    if prune.was_beneficial() { success_count += 1; }
                }
                if let Some(balance) = balancing {
                    total_count += 1;
                    if balance.was_successful() { success_count += 1; }
                }
                if let Some(health_check) = health {
                    total_count += 1;
                    if health_check.is_healthy() { success_count += 1; }
                }
                
                if total_count == 0 { false } else { success_count > total_count / 2 }
            }
        }
    }
}