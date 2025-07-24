//! Query execution operations and processing logic
//!
//! This module provides the core query execution operations with zero-allocation
//! patterns and blazing-fast performance for memory query processing.

use futures::StreamExt;
use smallvec::SmallVec;
use std::time::Instant;
use tracing::{debug, warn};

use crate::utils::{Result, error::Error};
use super::core::{MemoryQuery, SortOrder};
use super::super::{
    memory_manager::MemoryManager,
    memory_node::MemoryNode,
};
use super::executor_core::{MemoryQueryExecutor, QueryConfig, QueryExecutionStats};

impl MemoryQueryExecutor {
    /// Execute a query with the configured settings
    pub async fn execute_query(
        &self,
        query: &MemoryQuery,
        manager: &dyn MemoryManager,
    ) -> Result<Vec<MemoryNode>> {
        // Validate query first
        if let Err(e) = query.validate() {
            return Err(Error::InvalidQuery(e));
        }

        // Start profiling if enabled
        let start_time = if self.config.enable_profiling {
            Some(Instant::now())
        } else {
            None
        };

        // Execute the query with timeout
        let query_future = self.execute_query_internal(query, manager);
        let timeout_duration = self.config.timeout_duration();
        
        let result = tokio::time::timeout(timeout_duration, query_future).await;

        match result {
            Ok(query_result) => {
                // Log profiling information if enabled
                if let (Some(start), true) = (start_time, self.config.enable_profiling) {
                    let duration = start.elapsed();
                    debug!(
                        "Query executed in {:?}, complexity: {}",
                        duration,
                        query.complexity_score()
                    );
                }
                query_result
            }
            Err(_) => Err(Error::QueryTimeout {
                timeout_ms: self.config.timeout_ms,
            }),
        }
    }

    /// Execute a query with detailed statistics
    pub async fn execute_query_with_stats(
        &self,
        query: &MemoryQuery,
        manager: &dyn MemoryManager,
    ) -> Result<(Vec<MemoryNode>, QueryExecutionStats)> {
        let start_time = Instant::now();
        let mut stats = QueryExecutionStats::new();
        
        // Validate query first
        if let Err(e) = query.validate() {
            return Err(Error::InvalidQuery(e));
        }

        // Execute the query with timeout and statistics tracking
        let query_future = self.execute_query_with_stats_internal(query, manager, &mut stats);
        let timeout_duration = self.config.timeout_duration();
        
        let result = tokio::time::timeout(timeout_duration, query_future).await;

        // Update final statistics
        stats.execution_time = start_time.elapsed();
        stats.was_optimized = self.config.is_optimization_enabled();

        match result {
            Ok(query_result) => {
                stats.items_returned = query_result.len();
                Ok((query_result, stats))
            }
            Err(_) => Err(Error::QueryTimeout {
                timeout_ms: self.config.timeout_ms,
            }),
        }
    }

    /// Internal query execution implementation
    async fn execute_query_internal(
        &self,
        query: &MemoryQuery,
        manager: &dyn MemoryManager,
    ) -> Result<Vec<MemoryNode>> {
        // Use zero-allocation SmallVec for small result sets
        let mut results: SmallVec<[MemoryNode; 32]> = SmallVec::new();

        // Execute different query components based on what's specified
        if let Some(memory_types) = &query.filter.memory_types {
            if !memory_types.is_empty() {
                self.execute_type_based_query(memory_types, manager, &mut results, query).await?;
            }
        }

        // Execute text search if text query provided and we haven't hit the limit
        if let Some(text_query) = &query.filter.text_query {
            if !text_query.is_empty() && !self.has_reached_limit(&results, query) {
                self.execute_text_search_query(text_query, manager, &mut results, query).await?;
            }
        }

        // Execute time-based filtering if specified
        if query.filter.time_range.is_some() && !self.has_reached_limit(&results, query) {
            self.execute_time_based_filtering(&mut results, query).await?;
        }

        // Execute metadata filtering if specified
        if !query.filter.metadata_filters.is_empty() && !self.has_reached_limit(&results, query) {
            self.execute_metadata_filtering(&mut results, query).await?;
        }

        // Apply sorting if specified
        if let Some(sort_order) = &query.sort_order {
            self.apply_sorting(&mut results, sort_order);
        }

        // Apply final limit and offset
        self.apply_limit_and_offset(&mut results, query);

        Ok(results.into_vec())
    }

    /// Internal query execution with statistics tracking
    async fn execute_query_with_stats_internal(
        &self,
        query: &MemoryQuery,
        manager: &dyn MemoryManager,
        stats: &mut QueryExecutionStats,
    ) -> Result<Vec<MemoryNode>> {
        // Use zero-allocation SmallVec for small result sets
        let mut results: SmallVec<[MemoryNode; 32]> = SmallVec::new();
        let mut peak_memory = 0;

        // Execute different query components based on what's specified
        if let Some(memory_types) = &query.filter.memory_types {
            if !memory_types.is_empty() {
                self.execute_type_based_query_with_stats(
                    memory_types, 
                    manager, 
                    &mut results, 
                    query, 
                    stats,
                    &mut peak_memory
                ).await?;
            }
        }

        // Execute text search if text query provided and we haven't hit the limit
        if let Some(text_query) = &query.filter.text_query {
            if !text_query.is_empty() && !self.has_reached_limit(&results, query) {
                self.execute_text_search_query_with_stats(
                    text_query, 
                    manager, 
                    &mut results, 
                    query, 
                    stats,
                    &mut peak_memory
                ).await?;
            }
        }

        // Execute time-based filtering if specified
        if query.filter.time_range.is_some() && !self.has_reached_limit(&results, query) {
            self.execute_time_based_filtering_with_stats(&mut results, query, stats).await?;
        }

        // Execute metadata filtering if specified
        if !query.filter.metadata_filters.is_empty() && !self.has_reached_limit(&results, query) {
            self.execute_metadata_filtering_with_stats(&mut results, query, stats).await?;
        }

        // Apply sorting if specified
        if let Some(sort_order) = &query.sort_order {
            self.apply_sorting(&mut results, sort_order);
        }

        // Apply final limit and offset
        self.apply_limit_and_offset(&mut results, query);

        // Update final statistics
        stats.peak_memory_usage = peak_memory;
        stats.parallel_operations = self.config.max_parallel;

        Ok(results.into_vec())
    }

    /// Execute type-based query
    async fn execute_type_based_query(
        &self,
        memory_types: &[super::super::MemoryType],
        manager: &dyn MemoryManager,
        results: &mut SmallVec<[MemoryNode; 32]>,
        query: &MemoryQuery,
    ) -> Result<()> {
        // Execute type-based queries in parallel
        let type_futures = memory_types.iter().map(|memory_type| {
            manager.get_memories_by_type(*memory_type)
        });

        // Use futures::stream to handle parallel execution
        let mut tasks = futures::stream::iter(type_futures)
            .buffer_unordered(self.config.max_parallel);

        // Collect results from parallel streams
        while let Some(type_results) = tasks.next().await {
            results.extend(type_results);
            
            // Early termination if limit reached
            if self.has_reached_limit(results, query) {
                break;
            }

            // Memory usage check
            if self.estimate_memory_usage(results) > self.config.max_memory_bytes {
                warn!("Query memory usage exceeded limit, truncating results");
                break;
            }
        }

        Ok(())
    }

    /// Execute type-based query with statistics
    async fn execute_type_based_query_with_stats(
        &self,
        memory_types: &[super::super::MemoryType],
        manager: &dyn MemoryManager,
        results: &mut SmallVec<[MemoryNode; 32]>,
        query: &MemoryQuery,
        stats: &mut QueryExecutionStats,
        peak_memory: &mut usize,
    ) -> Result<()> {
        // Execute type-based queries in parallel
        let type_futures = memory_types.iter().map(|memory_type| {
            manager.get_memories_by_type(*memory_type)
        });

        // Use futures::stream to handle parallel execution
        let mut tasks = futures::stream::iter(type_futures)
            .buffer_unordered(self.config.max_parallel);

        // Collect results from parallel streams
        while let Some(type_results) = tasks.next().await {
            stats.items_processed += type_results.len();
            results.extend(type_results);
            
            // Update peak memory usage
            let current_memory = self.estimate_memory_usage(results);
            *peak_memory = (*peak_memory).max(current_memory);
            
            // Early termination if limit reached
            if self.has_reached_limit(results, query) {
                stats.early_termination = true;
                break;
            }

            // Memory usage check
            if current_memory > self.config.max_memory_bytes {
                warn!("Query memory usage exceeded limit, truncating results");
                stats.early_termination = true;
                break;
            }
        }

        Ok(())
    }

    /// Execute text search query
    async fn execute_text_search_query(
        &self,
        text_query: &str,
        manager: &dyn MemoryManager,
        results: &mut SmallVec<[MemoryNode; 32]>,
        query: &MemoryQuery,
    ) -> Result<()> {
        let search_results = manager.search_memories_by_text(text_query).await?;
        
        for node in search_results {
            if self.has_reached_limit(results, query) {
                break;
            }
            
            results.push(node);
            
            // Memory usage check
            if self.estimate_memory_usage(results) > self.config.max_memory_bytes {
                warn!("Query memory usage exceeded limit during text search");
                break;
            }
        }

        Ok(())
    }

    /// Execute text search query with statistics
    async fn execute_text_search_query_with_stats(
        &self,
        text_query: &str,
        manager: &dyn MemoryManager,
        results: &mut SmallVec<[MemoryNode; 32]>,
        query: &MemoryQuery,
        stats: &mut QueryExecutionStats,
        peak_memory: &mut usize,
    ) -> Result<()> {
        let search_results = manager.search_memories_by_text(text_query).await?;
        
        for node in search_results {
            stats.items_processed += 1;
            
            if self.has_reached_limit(results, query) {
                stats.early_termination = true;
                break;
            }
            
            results.push(node);
            
            // Update peak memory usage
            let current_memory = self.estimate_memory_usage(results);
            *peak_memory = (*peak_memory).max(current_memory);
            
            // Memory usage check
            if current_memory > self.config.max_memory_bytes {
                warn!("Query memory usage exceeded limit during text search");
                stats.early_termination = true;
                break;
            }
        }

        Ok(())
    }

    /// Check if query has reached its limit
    #[inline]
    fn has_reached_limit(&self, results: &SmallVec<[MemoryNode; 32]>, query: &MemoryQuery) -> bool {
        if let Some(limit) = query.filter.limit {
            results.len() >= limit
        } else {
            false
        }
    }

    /// Estimate memory usage of current results
    #[inline]
    fn estimate_memory_usage(&self, results: &SmallVec<[MemoryNode; 32]>) -> usize {
        // Rough estimation: each MemoryNode is approximately 1KB
        results.len() * 1024
    }
}