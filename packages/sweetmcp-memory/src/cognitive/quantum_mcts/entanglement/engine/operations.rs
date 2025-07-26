//! Engine operations for comprehensive quantum entanglement management
//!
//! This module provides blazing-fast engine operations with zero allocation
//! optimizations and elegant ergonomic interfaces for quantum entanglement operations.

use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::cognitive::types::CognitiveError;
use super::super::super::node_state::QuantumMCTSNode;
use super::{
    core::{QuantumEntanglementEngine},
    operation_types::{
        EngineOperationResult, EngineOperationType, EngineOperationDetails,
    },
    core_types::{
        EngineStatistics, EnginePerformanceReport, PerformanceGrades,
    },
    OptimizationResult, CreationResult, PruningResult, BalancingResult,
    health::NetworkHealthReport,
};

impl QuantumEntanglementEngine {
    /// Perform comprehensive engine operation with full optimization
    #[inline]
    pub async fn perform_comprehensive_operation(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        operation_type: EngineOperationType,
    ) -> Result<EngineOperationResult, CognitiveError> {
        let start_time = Instant::now();
        debug!("Starting comprehensive operation: {:?}", operation_type);

        let result = match operation_type {
            EngineOperationType::FullOptimization => {
                self.perform_full_optimization(tree, start_time).await
            }
            EngineOperationType::StrategyCreation => {
                self.perform_strategy_creation(tree, start_time).await
            }
            EngineOperationType::IntelligentPruning => {
                self.perform_intelligent_pruning(tree, start_time).await
            }
            EngineOperationType::LoadBalancing => {
                self.perform_load_balancing(tree, start_time).await
            }
            EngineOperationType::HealthCheck => {
                self.perform_health_check(tree, start_time).await
            }
            EngineOperationType::CombinedOptimization => {
                self.perform_combined_optimization(tree, start_time).await
            }
        };

        match &result {
            Ok(op_result) => {
                info!(
                    "Comprehensive operation completed: {:?} in {}ms with {:.1}% improvement",
                    operation_type,
                    op_result.operation_time_ms,
                    op_result.performance_improvement
                );
            }
            Err(error) => {
                warn!("Comprehensive operation failed: {:?} - {}", operation_type, error);
            }
        }

        result
    }

    /// Perform full optimization operation
    #[inline]
    async fn perform_full_optimization(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Performing full optimization");

        // Execute optimization through the optimization component
        let optimization_result = self.optimization.optimize_entanglements(tree).await?;

        let operation_time_ms = start_time.elapsed().as_millis() as u64;
        let success = optimization_result.success;
        let performance_improvement = optimization_result.performance_improvement;

        Ok(EngineOperationResult::new(
            EngineOperationType::FullOptimization,
            operation_time_ms,
            success,
            performance_improvement,
            EngineOperationDetails::Optimization(optimization_result),
        ))
    }

    /// Perform strategy creation operation
    #[inline]
    async fn perform_strategy_creation(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Performing strategy creation");

        // Execute creation through the optimization component
        let creation_result = self.optimization.create_strategic_entanglements(tree).await?;

        let operation_time_ms = start_time.elapsed().as_millis() as u64;
        let success = creation_result.success;
        let performance_improvement = creation_result.impact_score();

        Ok(EngineOperationResult::new(
            EngineOperationType::StrategyCreation,
            operation_time_ms,
            success,
            performance_improvement,
            EngineOperationDetails::Creation(creation_result),
        ))
    }

    /// Perform intelligent pruning operation
    #[inline]
    async fn perform_intelligent_pruning(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Performing intelligent pruning");

        // Execute pruning through the pruning component
        let pruning_result = self.pruning.prune_weak_entanglements(tree).await?;

        let operation_time_ms = start_time.elapsed().as_millis() as u64;
        let success = pruning_result.success;
        let performance_improvement = pruning_result.efficiency_improvement;

        Ok(EngineOperationResult::new(
            EngineOperationType::IntelligentPruning,
            operation_time_ms,
            success,
            performance_improvement,
            EngineOperationDetails::Pruning(pruning_result),
        ))
    }

    /// Perform load balancing operation
    #[inline]
    async fn perform_load_balancing(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Performing load balancing");

        // Execute balancing through the balancing component
        let balancing_result = self.balancing.balance_network_load(tree).await?;

        let operation_time_ms = start_time.elapsed().as_millis() as u64;
        let success = balancing_result.success;
        let performance_improvement = balancing_result.improvement_score;

        Ok(EngineOperationResult::new(
            EngineOperationType::LoadBalancing,
            operation_time_ms,
            success,
            performance_improvement,
            EngineOperationDetails::Balancing(balancing_result),
        ))
    }

    /// Perform health check operation
    #[inline]
    async fn perform_health_check(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Performing health check");

        // Execute health check through the health component
        let health_report = self.health.check_network_health(tree).await?;

        let operation_time_ms = start_time.elapsed().as_millis() as u64;
        let success = health_report.is_healthy();
        let performance_improvement = health_report.overall_score();

        Ok(EngineOperationResult::new(
            EngineOperationType::HealthCheck,
            operation_time_ms,
            success,
            performance_improvement,
            EngineOperationDetails::HealthCheck(health_report),
        ))
    }

    /// Perform combined optimization with all operations
    #[inline]
    pub async fn perform_combined_optimization(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        start_time: Instant,
    ) -> Result<EngineOperationResult, CognitiveError> {
        debug!("Performing combined optimization with all operations");

        let mut optimization_result = None;
        let mut creation_result = None;
        let mut pruning_result = None;
        let mut balancing_result = None;
        let mut health_result = None;

        let mut overall_success = true;
        let mut total_improvement = 0.0;
        let mut operation_count = 0;

        // Phase 1: Health Check (always first)
        match self.health.check_network_health(tree).await {
            Ok(health) => {
                let improvement = health.overall_score();
                total_improvement += improvement;
                operation_count += 1;
                health_result = Some(health);
                debug!("Health check completed with score: {:.2}", improvement);
            }
            Err(error) => {
                warn!("Health check failed: {}", error);
                overall_success = false;
            }
        }

        // Phase 2: Optimization (if health is acceptable)
        if health_result.as_ref().map_or(false, |h| h.is_healthy()) {
            match self.optimization.optimize_entanglements(tree).await {
                Ok(opt) => {
                    let improvement = opt.performance_improvement;
                    total_improvement += improvement;
                    operation_count += 1;
                    optimization_result = Some(opt);
                    debug!("Optimization completed with improvement: {:.2}%", improvement);
                }
                Err(error) => {
                    warn!("Optimization failed: {}", error);
                    overall_success = false;
                }
            }
        }

        // Phase 3: Strategic Creation (if optimization succeeded)
        if optimization_result.as_ref().map_or(false, |o| o.success) {
            match self.optimization.create_strategic_entanglements(tree).await {
                Ok(creation) => {
                    let improvement = creation.impact_score();
                    total_improvement += improvement;
                    operation_count += 1;
                    creation_result = Some(creation);
                    debug!("Strategic creation completed with impact: {:.2}", improvement);
                }
                Err(error) => {
                    warn!("Strategic creation failed: {}", error);
                    overall_success = false;
                }
            }
        }

        // Phase 4: Intelligent Pruning (always beneficial)
        match self.pruning.prune_weak_entanglements(tree).await {
            Ok(pruning) => {
                let improvement = pruning.efficiency_improvement;
                total_improvement += improvement;
                operation_count += 1;
                pruning_result = Some(pruning);
                debug!("Pruning completed with efficiency improvement: {:.2}%", improvement);
            }
            Err(error) => {
                warn!("Pruning failed: {}", error);
                overall_success = false;
            }
        }

        // Phase 5: Load Balancing (final optimization)
        match self.balancing.balance_network_load(tree).await {
            Ok(balancing) => {
                let improvement = balancing.improvement_score;
                total_improvement += improvement;
                operation_count += 1;
                balancing_result = Some(balancing);
                debug!("Load balancing completed with improvement: {:.2}", improvement);
            }
            Err(error) => {
                warn!("Load balancing failed: {}", error);
                overall_success = false;
            }
        }

        let operation_time_ms = start_time.elapsed().as_millis() as u64;
        let average_improvement = if operation_count > 0 {
            total_improvement / operation_count as f64
        } else {
            0.0
        };

        info!(
            "Combined optimization completed: {} operations, {:.2}% average improvement, {}ms total",
            operation_count, average_improvement, operation_time_ms
        );

        Ok(EngineOperationResult::new(
            EngineOperationType::CombinedOptimization,
            operation_time_ms,
            overall_success,
            average_improvement,
            EngineOperationDetails::Combined {
                optimization: optimization_result,
                creation: creation_result,
                pruning: pruning_result,
                balancing: balancing_result,
                health: health_result,
            },
        ))
    }

    /// Perform automatic maintenance based on current network state
    #[inline]
    pub async fn perform_automatic_maintenance(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
    ) -> Result<Vec<EngineOperationResult>, CognitiveError> {
        debug!("Performing automatic maintenance");

        let mut maintenance_results = Vec::new();

        // First, check network health to determine what maintenance is needed
        let health_result = self.perform_health_check(tree, Instant::now()).await?;
        maintenance_results.push(health_result.clone());

        // Analyze health report to determine required maintenance operations
        if let EngineOperationDetails::HealthCheck(health_report) = &health_result.details {
            // If network is unhealthy, perform comprehensive maintenance
            if !health_report.is_healthy() {
                debug!("Network health issues detected, performing comprehensive maintenance");

                // Perform pruning to remove problematic entanglements
                let pruning_result = self.perform_intelligent_pruning(tree, Instant::now()).await?;
                maintenance_results.push(pruning_result);

                // Rebalance the network after pruning
                let balancing_result = self.perform_load_balancing(tree, Instant::now()).await?;
                maintenance_results.push(balancing_result);

                // Optimize remaining entanglements
                let optimization_result = self.perform_full_optimization(tree, Instant::now()).await?;
                maintenance_results.push(optimization_result);
            } else {
                debug!("Network is healthy, performing light maintenance");

                // Light maintenance: just optimization and balancing
                let optimization_result = self.perform_full_optimization(tree, Instant::now()).await?;
                maintenance_results.push(optimization_result);

                let balancing_result = self.perform_load_balancing(tree, Instant::now()).await?;
                maintenance_results.push(balancing_result);
            }
        }

        info!("Automatic maintenance completed with {} operations", maintenance_results.len());
        Ok(maintenance_results)
    }

    /// Get comprehensive engine statistics
    #[inline]
    pub fn get_comprehensive_statistics(&self) -> Result<EngineStatistics, CognitiveError> {
        debug!("Gathering comprehensive engine statistics");

        // Collect statistics from all components
        let optimization_stats = self.optimization.get_statistics()?;
        let pruning_stats = self.pruning.get_statistics()?;
        let balancing_stats = self.balancing.get_statistics()?;
        let health_stats = self.health.get_statistics()?;

        // Aggregate statistics
        let mut stats = EngineStatistics::new();

        // Calculate weighted averages and totals
        stats.health_score = (optimization_stats.health_score + 
                             pruning_stats.health_score + 
                             balancing_stats.health_score + 
                             health_stats.health_score) / 4.0;

        stats.success_rate = (optimization_stats.success_rate + 
                             pruning_stats.success_rate + 
                             balancing_stats.success_rate + 
                             health_stats.success_rate) / 4.0;

        stats.average_latency_us = (optimization_stats.average_latency_us + 
                                   pruning_stats.average_latency_us + 
                                   balancing_stats.average_latency_us + 
                                   health_stats.average_latency_us) / 4.0;

        stats.cache_efficiency = (optimization_stats.cache_efficiency + 
                                 pruning_stats.cache_efficiency + 
                                 balancing_stats.cache_efficiency + 
                                 health_stats.cache_efficiency) / 4.0;

        stats.throughput_ops_per_sec = optimization_stats.throughput_ops_per_sec + 
                                      pruning_stats.throughput_ops_per_sec + 
                                      balancing_stats.throughput_ops_per_sec + 
                                      health_stats.throughput_ops_per_sec;

        stats.total_operations = optimization_stats.total_operations + 
                                pruning_stats.total_operations + 
                                balancing_stats.total_operations + 
                                health_stats.total_operations;

        stats.failed_operations = optimization_stats.failed_operations + 
                                  pruning_stats.failed_operations + 
                                  balancing_stats.failed_operations + 
                                  health_stats.failed_operations;

        debug!("Comprehensive statistics gathered: {}", stats.performance_summary());
        Ok(stats)
    }

    /// Create engine performance report
    #[inline]
    pub fn create_performance_report(&self) -> Result<EnginePerformanceReport, CognitiveError> {
        debug!("Creating engine performance report");

        let statistics = self.get_comprehensive_statistics()?;
        let health_report = self.health.get_latest_health_report()?;

        // Calculate performance grades
        let latency_grade = Self::calculate_latency_grade(statistics.average_latency_us);
        let throughput_grade = Self::calculate_throughput_grade(statistics.throughput_ops_per_sec);
        let reliability_grade = Self::calculate_reliability_grade(statistics.success_rate);
        let efficiency_grade = Self::calculate_efficiency_grade(statistics.cache_efficiency);

        // Calculate overall grade (weighted average)
        let overall_score = (latency_grade as u8 + throughput_grade as u8 + 
                           reliability_grade as u8 + efficiency_grade as u8) as f64 / 4.0;
        let overall_grade = match overall_score as u8 {
            score if score >= b'A' => 'A',
            score if score >= b'B' => 'B',
            score if score >= b'C' => 'C',
            score if score >= b'D' => 'D',
            _ => 'F',
        };

        let performance_grades = PerformanceGrades::new(
            overall_grade,
            latency_grade,
            throughput_grade,
            reliability_grade,
            efficiency_grade,
        );

        let report = EnginePerformanceReport::new(
            performance_grades,
            statistics,
            health_report,
        );

        debug!("Performance report created: {}", report.performance_summary());
        Ok(report)
    }

    /// Calculate performance grade for latency
    #[inline]
    fn calculate_latency_grade(latency_us: f64) -> char {
        match latency_us {
            l if l < 100.0 => 'A',
            l if l < 500.0 => 'B',
            l if l < 1000.0 => 'C',
            l if l < 5000.0 => 'D',
            _ => 'F',
        }
    }

    /// Calculate performance grade for throughput
    #[inline]
    fn calculate_throughput_grade(ops_per_sec: f64) -> char {
        match ops_per_sec {
            t if t > 10000.0 => 'A',
            t if t > 5000.0 => 'B',
            t if t > 1000.0 => 'C',
            t if t > 100.0 => 'D',
            _ => 'F',
        }
    }

    /// Calculate performance grade for reliability
    #[inline]
    fn calculate_reliability_grade(success_rate: f64) -> char {
        match success_rate {
            r if r > 0.99 => 'A',
            r if r > 0.95 => 'B',
            r if r > 0.90 => 'C',
            r if r > 0.80 => 'D',
            _ => 'F',
        }
    }

    /// Calculate performance grade for efficiency
    #[inline]
    fn calculate_efficiency_grade(cache_hit_rate: f64) -> char {
        match cache_hit_rate {
            e if e > 0.95 => 'A',
            e if e > 0.90 => 'B',
            e if e > 0.80 => 'C',
            e if e > 0.70 => 'D',
            _ => 'F',
        }
    }
}