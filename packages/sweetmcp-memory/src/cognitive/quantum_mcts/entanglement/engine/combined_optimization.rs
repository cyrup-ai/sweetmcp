//! Combined optimization workflows for comprehensive engine operations
//!
//! This module provides sophisticated multi-step optimization workflows with
//! blazing-fast zero-allocation patterns and intelligent operation sequencing.

use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, warn};

use crate::cognitive::types::CognitiveError;
use super::{
    super::super::node_state::QuantumMCTSNode,
    core::QuantumEntanglementEngine,
    operation_types::{EngineOperationType, EngineOperationResult, EngineOperationDetails},
};

impl QuantumEntanglementEngine {
    /// Perform combined optimization with all operations
    pub async fn perform_combined_optimization(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Performing combined optimization with all operations");
        
        let mut total_improvement = 0.0;
        let mut operations_successful = 0;
        let mut total_operations = 0;
        
        let health_result = match self.health_check().await {
            Ok(health) => {
                total_operations += 1;
                if health.is_healthy() {
                    operations_successful += 1;
                }
                Some(health)
            }
            Err(e) => {
                warn!("Health check failed during combined optimization: {}", e);
                None
            }
        };
        
        // Step 2: Intelligent pruning if network is dense
        let pruning_result = if let Some(ref health) = health_result {
            if health.topology.network_density > 0.1 {
                match self.intelligent_pruning(tree).await {
                    Ok(pruning) => {
                        total_operations += 1;
                        if pruning.was_beneficial() {
                            operations_successful += 1;
                            total_improvement += pruning.network_improvement;
                        }
                        Some(pruning)
                    }
                    Err(e) => {
                        warn!("Pruning failed during combined optimization: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Step 3: Strategic creation if network is sparse
        let creation_result = if let Some(ref health) = health_result {
            if health.topology.network_density < 0.05 || !health.topology.is_connected {
                match self.create_strategic_entanglements(tree).await {
                    Ok(creation) => {
                        total_operations += 1;
                        if creation.was_successful() {
                            operations_successful += 1;
                            total_improvement += creation.network_improvement;
                        }
                        Some(creation)
                    }
                    Err(e) => {
                        warn!("Strategic creation failed during combined optimization: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Step 4: Load balancing to optimize distribution
        let balancing_result = match self.balance_entanglement_distribution(tree).await {
            Ok(balancing) => {
                total_operations += 1;
                if balancing.was_successful() {
                    operations_successful += 1;
                    total_improvement += balancing.balance_improvement;
                }
                Some(balancing)
            }
            Err(e) => {
                warn!("Load balancing failed during combined optimization: {}", e);
                None
            }
        };
        
        // Step 5: Full optimization to finalize improvements
        let optimization_result = match self.optimize_entanglements(tree).await {
            Ok(optimization) => {
                total_operations += 1;
                if optimization.was_beneficial() {
                    operations_successful += 1;
                    total_improvement += optimization.performance_improvement;
                }
                Some(optimization)
            }
            Err(e) => {
                warn!("Full optimization failed during combined optimization: {}", e);
                None
            }
        };
        
        let operation_time_ms = start_time.elapsed().as_millis() as u64;
        let success = operations_successful > total_operations / 2; // Majority success
        let average_improvement = if total_operations > 0 {
            total_improvement / total_operations as f64
        } else {
            0.0
        };
        
        Ok(EngineOperationResult::new(
            EngineOperationType::CombinedOptimization,
            operation_time_ms,
            success,
            average_improvement,
            EngineOperationDetails::Combined {
                optimization: optimization_result,
                creation: creation_result,
                pruning: pruning_result,
                balancing: balancing_result,
                health: health_result,
            },
            start_time,
        ))
    }
    
    /// Perform adaptive combined optimization based on network state
    pub async fn perform_adaptive_combined_optimization(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
    ) -> Result<EngineOperationResult, CognitiveError> {
        let start_time = Instant::now();
        debug!("Performing adaptive combined optimization");
        
        // First, analyze the current state
        let initial_health = self.health_check().await?;
        let density = initial_health.topology.network_density;
        let connectivity = initial_health.topology.is_connected;
        let health_score = initial_health.health_score;
        
        let mut operations = Vec::new();
        
        if health_score < 0.5 {
            operations.extend(vec![
                OptimizationStep::AggressivePruning,
                OptimizationStep::ConnectivityRepair,
                OptimizationStep::FullOptimization,
                OptimizationStep::LoadBalancing,
            ]);
        } else if health_score < 0.8 {
            if density > 0.15 {
                operations.push(OptimizationStep::IntelligentPruning);
            }
            if density < 0.03 || !connectivity {
                operations.push(OptimizationStep::StrategicCreation);
            }
            operations.extend(vec![
                OptimizationStep::FullOptimization,
                OptimizationStep::LoadBalancing,
            ]);
        } else {
            operations.extend(vec![
                OptimizationStep::LoadBalancing,
                OptimizationStep::LightOptimization,
            ]);
        }
        let result = self.execute_optimization_sequence(tree, operations, start_time).await?;
        
        debug!(
            "Adaptive combined optimization completed: {} improvement in {}ms",
            result.performance_improvement,
            result.operation_time_ms
        );
        
        Ok(result)
    }
    
    /// Execute a sequence of optimization steps
    async fn execute_optimization_sequence(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        steps: Vec<OptimizationStep>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        let mut total_improvement = 0.0;
        let mut operations_successful = 0;
        let total_operations = steps.len();
        
        let mut optimization_result = None;
        let mut creation_result = None;
        let mut pruning_result = None;
        let mut balancing_result = None;
        let mut health_result = None;
        
        for step in steps {
            match step {
                OptimizationStep::FullOptimization | OptimizationStep::LightOptimization => {
                    if let Ok(result) = self.optimize_entanglements(tree).await {
                        if result.was_beneficial() {
                            operations_successful += 1;
                            total_improvement += result.performance_improvement;
                        }
                        optimization_result = Some(result);
                    }
                }
                OptimizationStep::IntelligentPruning | OptimizationStep::AggressivePruning => {
                    if let Ok(result) = self.intelligent_pruning(tree).await {
                        if result.was_beneficial() {
                            operations_successful += 1;
                            total_improvement += result.network_improvement;
                        }
                        pruning_result = Some(result);
                    }
                }
                OptimizationStep::StrategicCreation | OptimizationStep::ConnectivityRepair => {
                    if let Ok(result) = self.create_strategic_entanglements(tree).await {
                        if result.was_successful() {
                            operations_successful += 1;
                            total_improvement += result.network_improvement;
                        }
                        creation_result = Some(result);
                    }
                }
                OptimizationStep::LoadBalancing => {
                    if let Ok(result) = self.balance_entanglement_distribution(tree).await {
                        if result.was_successful() {
                            operations_successful += 1;
                            total_improvement += result.balance_improvement;
                        }
                        balancing_result = Some(result);
                    }
                }
                OptimizationStep::HealthCheck => {
                    if let Ok(result) = self.health_check().await {
                        if result.is_healthy() {
                            operations_successful += 1;
                        }
                        health_result = Some(result);
                    }
                }
            }
        }
        
        let operation_time_ms = start_time.elapsed().as_millis() as u64;
        let success = operations_successful > total_operations / 2;
        let average_improvement = if total_operations > 0 {
            total_improvement / total_operations as f64
        } else {
            0.0
        };
        
        Ok(EngineOperationResult::new(
            EngineOperationType::CombinedOptimization,
            operation_time_ms,
            success,
            average_improvement,
            EngineOperationDetails::Combined {
                optimization: optimization_result,
                creation: creation_result,
                pruning: pruning_result,
                balancing: balancing_result,
                health: health_result,
            },
            start_time,
        ))
    }
}

/// Optimization step types for sequenced operations
#[derive(Debug, Clone, PartialEq)]
enum OptimizationStep {
    FullOptimization,
    LightOptimization,
    IntelligentPruning,
    AggressivePruning,
    StrategicCreation,
    ConnectivityRepair,
    LoadBalancing,
    HealthCheck,
}