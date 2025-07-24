//! Operation execution logic for quantum entanglement engine
//!
//! This module provides the main operation execution logic with blazing-fast
//! zero-allocation patterns and comprehensive performance tracking.

use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::cognitive::types::CognitiveError;
use super::{
    super::super::node_state::QuantumMCTSNode,
    super::metrics::PerformanceTracker,
    core::QuantumEntanglementEngine,
    operation_types::{EngineOperationType, EngineOperationResult, EngineOperationDetails},
};

impl QuantumEntanglementEngine {
    /// Perform comprehensive engine operation with full optimization
    pub async fn perform_comprehensive_operation(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        operation_type: EngineOperationType,
    ) -> Result<EngineOperationResult, CognitiveError> {
        let start_time = Instant::now();
        let _tracker = PerformanceTracker::start();
        
        debug!("Starting comprehensive engine operation: {:?}", operation_type);
        
        let result = match operation_type {
            EngineOperationType::FullOptimization => {
                self.execute_full_optimization(tree, start_time).await?
            }
            
            EngineOperationType::StrategyCreation => {
                self.execute_strategy_creation(tree, start_time).await?
            }
            
            EngineOperationType::IntelligentPruning => {
                self.execute_intelligent_pruning(tree, start_time).await?
            }
            
            EngineOperationType::LoadBalancing => {
                self.execute_load_balancing(tree, start_time).await?
            }
            
            EngineOperationType::HealthCheck => {
                self.execute_health_check(start_time).await?
            }
            
            EngineOperationType::CombinedOptimization => {
                // This is handled in combined_optimization.rs
                return Err(CognitiveError::InvalidOperation(
                    "Combined optimization should be called through perform_combined_optimization".to_string()
                ));
            }
        };
        
        // Record operation metrics
        self.record_operation_metrics(&result);
        
        info!(
            "Engine operation {:?} completed in {}ms: {} (+{:.1}% improvement)",
            result.operation_type,
            result.operation_time_ms,
            if result.success { "SUCCESS" } else { "PARTIAL" },
            result.performance_improvement
        );
        
        Ok(result)
    }
    
    /// Execute full optimization operation
    async fn execute_full_optimization(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Executing full optimization operation");
        
        let optimization_result = self.optimize_entanglements(tree).await?;
        let success = optimization_result.was_beneficial();
        let performance_improvement = optimization_result.performance_improvement;
        
        Ok(EngineOperationResult::new(
            EngineOperationType::FullOptimization,
            start_time.elapsed().as_millis() as u64,
            success,
            performance_improvement,
            EngineOperationDetails::Optimization(optimization_result),
            start_time,
        ))
    }
    
    /// Execute strategic entanglement creation operation
    async fn execute_strategy_creation(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Executing strategic creation operation");
        
        let creation_result = self.create_strategic_entanglements(tree).await?;
        let success = creation_result.was_successful();
        let performance_improvement = creation_result.network_improvement;
        
        Ok(EngineOperationResult::new(
            EngineOperationType::StrategyCreation,
            start_time.elapsed().as_millis() as u64,
            success,
            performance_improvement,
            EngineOperationDetails::Creation(creation_result),
            start_time,
        ))
    }
    
    /// Execute intelligent pruning operation
    async fn execute_intelligent_pruning(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Executing intelligent pruning operation");
        
        let pruning_result = self.intelligent_pruning(tree).await?;
        let success = pruning_result.was_beneficial();
        let performance_improvement = pruning_result.network_improvement;
        
        Ok(EngineOperationResult::new(
            EngineOperationType::IntelligentPruning,
            start_time.elapsed().as_millis() as u64,
            success,
            performance_improvement,
            EngineOperationDetails::Pruning(pruning_result),
            start_time,
        ))
    }
    
    /// Execute load balancing operation
    async fn execute_load_balancing(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Executing load balancing operation");
        
        let balancing_result = self.balance_entanglement_distribution(tree).await?;
        let success = balancing_result.was_successful();
        let performance_improvement = balancing_result.balance_improvement;
        
        Ok(EngineOperationResult::new(
            EngineOperationType::LoadBalancing,
            start_time.elapsed().as_millis() as u64,
            success,
            performance_improvement,
            EngineOperationDetails::Balancing(balancing_result),
            start_time,
        ))
    }
    
    /// Execute health check operation
    async fn execute_health_check(
        &mut self,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Executing health check operation");
        
        let health_result = self.health_check().await?;
        let success = health_result.is_healthy();
        let performance_improvement = health_result.health_score * 100.0;
        
        Ok(EngineOperationResult::new(
            EngineOperationType::HealthCheck,
            start_time.elapsed().as_millis() as u64,
            success,
            performance_improvement,
            EngineOperationDetails::HealthCheck(health_result),
            start_time,
        ))
    }
    
    /// Perform automatic maintenance based on current network state
    pub async fn perform_automatic_maintenance(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
    ) -> Result<Vec<EngineOperationResult>, CognitiveError> {
        debug!("Performing automatic maintenance based on network analysis");
        
        let mut results = Vec::new();
        
        // Analyze current network health
        let health_check = self.perform_comprehensive_operation(tree, EngineOperationType::HealthCheck).await?;
        results.push(health_check);
        
        // Extract health report for decision making
        if let EngineOperationDetails::HealthCheck(ref health_report) = results[0].details {
            // Perform maintenance operations based on health analysis
            if health_report.health_score < 0.8 {
                // Network needs significant improvement - use combined optimization
                warn!("Network health score {:.2} requires combined optimization", health_report.health_score);
                
                // For now, perform individual optimizations
                if health_report.topology.network_density > 0.15 {
                    let pruning_result = self.perform_comprehensive_operation(tree, EngineOperationType::IntelligentPruning).await?;
                    results.push(pruning_result);
                }
                
                if health_report.topology.network_density < 0.03 {
                    let creation_result = self.perform_comprehensive_operation(tree, EngineOperationType::StrategyCreation).await?;
                    results.push(creation_result);
                }
                
                let optimization_result = self.perform_comprehensive_operation(tree, EngineOperationType::FullOptimization).await?;
                results.push(optimization_result);
                
            } else if health_report.topology.network_density > 0.15 {
                // Network is too dense - prune
                debug!("Network density {:.3} requires pruning", health_report.topology.network_density);
                let pruning_result = self.perform_comprehensive_operation(tree, EngineOperationType::IntelligentPruning).await?;
                results.push(pruning_result);
                
            } else if health_report.topology.network_density < 0.03 {
                // Network is too sparse - create connections
                debug!("Network density {:.3} requires connection creation", health_report.topology.network_density);
                let creation_result = self.perform_comprehensive_operation(tree, EngineOperationType::StrategyCreation).await?;
                results.push(creation_result);
                
            } else if !health_report.topology.is_connected {
                // Network connectivity issues
                debug!("Network connectivity issues detected, performing optimization");
                let optimization_result = self.perform_comprehensive_operation(tree, EngineOperationType::FullOptimization).await?;
                results.push(optimization_result);
                
            } else {
                // Network is healthy - perform light balancing
                debug!("Network is healthy, performing maintenance balancing");
                let balancing_result = self.perform_comprehensive_operation(tree, EngineOperationType::LoadBalancing).await?;
                results.push(balancing_result);
            }
        }
        
        info!("Automatic maintenance completed: {} operations performed", results.len());
        
        Ok(results)
    }
    
    /// Record operation metrics for performance tracking
    fn record_operation_metrics(&self, result: &EngineOperationResult) {
        self.metrics.record_engine_operation(
            &result.operation_type,
            result.operation_time_ms,
            result.success,
            result.performance_improvement,
        );
        
        match &result.details {
            EngineOperationDetails::Optimization(opt) => {
                self.metrics.record_optimization_metrics(
                    opt.performance_improvement,
                    opt.optimizations_applied.len(),
                    result.operation_time_ms,
                );
            }
            EngineOperationDetails::Creation(creation) => {
                self.metrics.record_creation_metrics(
                    creation.entanglements_created,
                    creation.network_improvement,
                    result.operation_time_ms,
                );
            }
            EngineOperationDetails::Pruning(pruning) => {
                self.metrics.record_pruning_metrics(
                    pruning.entanglements_removed,
                    pruning.network_improvement,
                    result.operation_time_ms,
                );
            }
            EngineOperationDetails::Balancing(balancing) => {
                self.metrics.record_balancing_metrics(
                    balancing.balance_improvement,
                    balancing.nodes_rebalanced,
                    result.operation_time_ms,
                );
            }
            EngineOperationDetails::HealthCheck(health) => {
                self.metrics.record_health_metrics(
                    health.health_score,
                    health.issues.len(),
                    result.operation_time_ms,
                );
            }
            EngineOperationDetails::Combined { .. } => {
                self.metrics.record_combined_operation_metrics(
                    result.performance_improvement,
                    result.operation_time_ms,
                );
            }
        }
    }
}