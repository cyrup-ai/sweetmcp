//! Query execution optimization and filtering logic
//!
//! This module provides optimization strategies and filtering operations
//! with zero-allocation patterns and blazing-fast performance.

use smallvec::SmallVec;
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::utils::{Result, error::Error};
use super::core::{MemoryQuery, SortOrder};
use super::super::{
    memory_manager::MemoryManager,
    memory_node::MemoryNode,
};
use super::executor_core::{MemoryQueryExecutor, QueryExecutionStats};

impl MemoryQueryExecutor {
    /// Execute time-based filtering
    pub async fn execute_time_based_filtering(
        &self,
        results: &mut SmallVec<[MemoryNode; 32]>,
        query: &MemoryQuery,
    ) -> Result<()> {
        if let Some(time_range) = &query.filter.time_range {
            results.retain(|node| {
                let node_time = node.created_at();
                node_time >= time_range.start && node_time <= time_range.end
            });
        }
        Ok(())
    }

    /// Execute time-based filtering with statistics
    pub async fn execute_time_based_filtering_with_stats(
        &self,
        results: &mut SmallVec<[MemoryNode; 32]>,
        query: &MemoryQuery,
        stats: &mut QueryExecutionStats,
    ) -> Result<()> {
        if let Some(time_range) = &query.filter.time_range {
            let initial_count = results.len();
            results.retain(|node| {
                let node_time = node.created_at();
                node_time >= time_range.start && node_time <= time_range.end
            });
            let filtered_count = initial_count - results.len();
            stats.items_processed += filtered_count;
        }
        Ok(())
    }

    /// Execute metadata filtering
    pub async fn execute_metadata_filtering(
        &self,
        results: &mut SmallVec<[MemoryNode; 32]>,
        query: &MemoryQuery,
    ) -> Result<()> {
        if !query.filter.metadata_filters.is_empty() {
            results.retain(|node| {
                self.matches_metadata_filters(node, &query.filter.metadata_filters)
            });
        }
        Ok(())
    }

    /// Execute metadata filtering with statistics
    pub async fn execute_metadata_filtering_with_stats(
        &self,
        results: &mut SmallVec<[MemoryNode; 32]>,
        query: &MemoryQuery,
        stats: &mut QueryExecutionStats,
    ) -> Result<()> {
        if !query.filter.metadata_filters.is_empty() {
            let initial_count = results.len();
            results.retain(|node| {
                self.matches_metadata_filters(node, &query.filter.metadata_filters)
            });
            let filtered_count = initial_count - results.len();
            stats.items_processed += filtered_count;
        }
        Ok(())
    }

    /// Check if node matches metadata filters
    #[inline]
    fn matches_metadata_filters(
        &self,
        node: &MemoryNode,
        filters: &HashMap<String, serde_json::Value>,
    ) -> bool {
        for (key, expected_value) in filters {
            if let Some(node_value) = node.get_metadata(key) {
                if node_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Apply sorting to results
    #[inline]
    pub fn apply_sorting(&self, results: &mut SmallVec<[MemoryNode; 32]>, sort_order: &SortOrder) {
        match sort_order {
            SortOrder::CreatedAtAsc => {
                results.sort_by(|a, b| a.created_at().cmp(&b.created_at()));
            }
            SortOrder::CreatedAtDesc => {
                results.sort_by(|a, b| b.created_at().cmp(&a.created_at()));
            }
            SortOrder::UpdatedAtAsc => {
                results.sort_by(|a, b| a.updated_at().cmp(&b.updated_at()));
            }
            SortOrder::UpdatedAtDesc => {
                results.sort_by(|a, b| b.updated_at().cmp(&a.updated_at()));
            }
            SortOrder::AccessCountAsc => {
                results.sort_by(|a, b| a.access_count().cmp(&b.access_count()));
            }
            SortOrder::AccessCountDesc => {
                results.sort_by(|a, b| b.access_count().cmp(&a.access_count()));
            }
            SortOrder::RelevanceScore => {
                results.sort_by(|a, b| {
                    b.relevance_score()
                        .partial_cmp(&a.relevance_score())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
    }

    /// Apply limit and offset to results
    #[inline]
    pub fn apply_limit_and_offset(&self, results: &mut SmallVec<[MemoryNode; 32]>, query: &MemoryQuery) {
        let offset = query.filter.offset.unwrap_or(0);
        let limit = query.filter.limit;

        // Apply offset
        if offset > 0 {
            if offset >= results.len() {
                results.clear();
                return;
            }
            results.drain(0..offset);
        }

        // Apply limit
        if let Some(limit) = limit {
            if results.len() > limit {
                results.truncate(limit);
            }
        }
    }

    /// Optimize query execution plan
    pub fn optimize_query_plan(&self, query: &MemoryQuery) -> OptimizedQueryPlan {
        let mut plan = OptimizedQueryPlan::new();

        // Analyze query complexity
        let complexity = query.complexity_score();
        plan.complexity_score = complexity;

        // Determine optimal execution order
        if query.filter.memory_types.is_some() {
            plan.execution_steps.push(ExecutionStep::TypeFiltering);
        }

        if query.filter.text_query.is_some() {
            plan.execution_steps.push(ExecutionStep::TextSearch);
        }

        if query.filter.time_range.is_some() {
            plan.execution_steps.push(ExecutionStep::TimeFiltering);
        }

        if !query.filter.metadata_filters.is_empty() {
            plan.execution_steps.push(ExecutionStep::MetadataFiltering);
        }

        if query.sort_order.is_some() {
            plan.execution_steps.push(ExecutionStep::Sorting);
        }

        // Determine optimal parallelization strategy
        plan.parallel_strategy = if complexity > 100 {
            ParallelStrategy::HighParallelism
        } else if complexity > 50 {
            ParallelStrategy::MediumParallelism
        } else {
            ParallelStrategy::LowParallelism
        };

        // Determine memory strategy
        plan.memory_strategy = if query.filter.limit.unwrap_or(usize::MAX) > 10000 {
            MemoryStrategy::Streaming
        } else {
            MemoryStrategy::InMemory
        };

        plan
    }

    /// Execute optimized query
    pub async fn execute_optimized_query(
        &self,
        query: &MemoryQuery,
        manager: &dyn MemoryManager,
        plan: &OptimizedQueryPlan,
    ) -> Result<Vec<MemoryNode>> {
        let mut results: SmallVec<[MemoryNode; 32]> = SmallVec::new();

        debug!("Executing optimized query with {} steps", plan.execution_steps.len());

        // Execute steps in optimized order
        for step in &plan.execution_steps {
            match step {
                ExecutionStep::TypeFiltering => {
                    if let Some(memory_types) = &query.filter.memory_types {
                        self.execute_type_based_query(memory_types, manager, &mut results, query).await?;
                    }
                }
                ExecutionStep::TextSearch => {
                    if let Some(text_query) = &query.filter.text_query {
                        self.execute_text_search_query(text_query, manager, &mut results, query).await?;
                    }
                }
                ExecutionStep::TimeFiltering => {
                    self.execute_time_based_filtering(&mut results, query).await?;
                }
                ExecutionStep::MetadataFiltering => {
                    self.execute_metadata_filtering(&mut results, query).await?;
                }
                ExecutionStep::Sorting => {
                    if let Some(sort_order) = &query.sort_order {
                        self.apply_sorting(&mut results, sort_order);
                    }
                }
            }

            // Early termination check
            if self.config.enable_early_termination && self.has_reached_limit(&results, query) {
                debug!("Early termination triggered");
                break;
            }
        }

        // Apply final limit and offset
        self.apply_limit_and_offset(&mut results, query);

        Ok(results.into_vec())
    }

    /// Analyze query performance
    pub fn analyze_query_performance(&self, stats: &QueryExecutionStats) -> QueryPerformanceAnalysis {
        let mut analysis = QueryPerformanceAnalysis::new();

        // Analyze throughput
        analysis.throughput = stats.throughput();
        analysis.throughput_rating = if analysis.throughput > 10000.0 {
            PerformanceRating::Excellent
        } else if analysis.throughput > 5000.0 {
            PerformanceRating::Good
        } else if analysis.throughput > 1000.0 {
            PerformanceRating::Average
        } else {
            PerformanceRating::Poor
        };

        // Analyze memory efficiency
        analysis.memory_efficiency = stats.memory_efficiency();
        analysis.memory_rating = if analysis.memory_efficiency > 1000.0 {
            PerformanceRating::Excellent
        } else if analysis.memory_efficiency > 500.0 {
            PerformanceRating::Good
        } else if analysis.memory_efficiency > 100.0 {
            PerformanceRating::Average
        } else {
            PerformanceRating::Poor
        };

        // Analyze selectivity
        analysis.selectivity = stats.selectivity_ratio();
        analysis.selectivity_rating = if analysis.selectivity > 0.8 {
            PerformanceRating::Poor // High selectivity means poor filtering
        } else if analysis.selectivity > 0.5 {
            PerformanceRating::Average
        } else if analysis.selectivity > 0.1 {
            PerformanceRating::Good
        } else {
            PerformanceRating::Excellent
        };

        // Overall rating
        analysis.overall_rating = match (
            &analysis.throughput_rating,
            &analysis.memory_rating,
            &analysis.selectivity_rating,
        ) {
            (PerformanceRating::Excellent, PerformanceRating::Excellent, PerformanceRating::Excellent) => {
                PerformanceRating::Excellent
            }
            (PerformanceRating::Poor, _, _) | (_, PerformanceRating::Poor, _) | (_, _, PerformanceRating::Poor) => {
                PerformanceRating::Poor
            }
            _ => PerformanceRating::Good,
        };

        analysis
    }
}

/// Optimized query execution plan
#[derive(Debug, Clone)]
pub struct OptimizedQueryPlan {
    /// Execution steps in optimal order
    pub execution_steps: Vec<ExecutionStep>,
    /// Parallel execution strategy
    pub parallel_strategy: ParallelStrategy,
    /// Memory management strategy
    pub memory_strategy: MemoryStrategy,
    /// Query complexity score
    pub complexity_score: u32,
}

impl OptimizedQueryPlan {
    /// Create new optimized query plan
    #[inline]
    pub fn new() -> Self {
        Self {
            execution_steps: Vec::new(),
            parallel_strategy: ParallelStrategy::MediumParallelism,
            memory_strategy: MemoryStrategy::InMemory,
            complexity_score: 0,
        }
    }
}

/// Query execution step
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStep {
    TypeFiltering,
    TextSearch,
    TimeFiltering,
    MetadataFiltering,
    Sorting,
}

/// Parallel execution strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParallelStrategy {
    LowParallelism,
    MediumParallelism,
    HighParallelism,
}

/// Memory management strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryStrategy {
    InMemory,
    Streaming,
}

/// Query performance analysis
#[derive(Debug, Clone)]
pub struct QueryPerformanceAnalysis {
    /// Throughput (items per second)
    pub throughput: f64,
    /// Throughput performance rating
    pub throughput_rating: PerformanceRating,
    /// Memory efficiency (items per MB)
    pub memory_efficiency: f64,
    /// Memory efficiency rating
    pub memory_rating: PerformanceRating,
    /// Selectivity ratio
    pub selectivity: f64,
    /// Selectivity rating
    pub selectivity_rating: PerformanceRating,
    /// Overall performance rating
    pub overall_rating: PerformanceRating,
}

impl QueryPerformanceAnalysis {
    /// Create new performance analysis
    #[inline]
    pub fn new() -> Self {
        Self {
            throughput: 0.0,
            throughput_rating: PerformanceRating::Average,
            memory_efficiency: 0.0,
            memory_rating: PerformanceRating::Average,
            selectivity: 0.0,
            selectivity_rating: PerformanceRating::Average,
            overall_rating: PerformanceRating::Average,
        }
    }
}

/// Performance rating
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformanceRating {
    Excellent,
    Good,
    Average,
    Poor,
}