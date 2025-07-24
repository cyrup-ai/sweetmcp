//! Memory query module
//!
//! This module provides comprehensive memory querying functionality including
//! basic queries, complex query building, and query execution with zero allocation
//! patterns and blazing-fast performance.

pub mod core;
pub mod executor_core;
pub mod executor_operations;
pub mod executor_optimization;
pub mod builder;

// Re-export core types and traits for ergonomic usage
pub use core::{
    MemoryQuery, SortOrder, MemoryQueryResult, QueryStatistics, ExecutionTimeCategory,
};

pub use executor_core::{
    MemoryQueryExecutor, QueryConfig, QueryExecutionStats,
};
pub use executor_optimization::{
    OptimizedQueryPlan, ExecutionStep, ParallelStrategy, MemoryStrategy,
    QueryPerformanceAnalysis, PerformanceRating,
};

pub use builder::{
    ComplexQueryBuilder, QueryCondition, LogicalOperator,
};

/// Create a new memory query
pub fn query() -> MemoryQuery {
    MemoryQuery::new()
}

/// Create a new complex query builder
pub fn complex_query() -> ComplexQueryBuilder {
    ComplexQueryBuilder::new()
}

/// Create a new query executor with default configuration
pub fn executor() -> MemoryQueryExecutor {
    MemoryQueryExecutor::with_defaults()
}

/// Create a new query executor with custom configuration
pub fn executor_with_config(config: QueryConfig) -> MemoryQueryExecutor {
    MemoryQueryExecutor::new(config)
}

/// Create a query for recent memories
pub fn recent_memories(days: u32) -> ComplexQueryBuilder {
    ComplexQueryBuilder::recent_memories(days)
}

/// Create a query for user memories
pub fn user_memories(user_id: impl Into<String>) -> ComplexQueryBuilder {
    ComplexQueryBuilder::user_memories(user_id)
}

/// Create a query for project memories
pub fn project_memories(project_id: impl Into<String>) -> ComplexQueryBuilder {
    ComplexQueryBuilder::project_memories(project_id)
}

/// Create a query for important memories
pub fn important_memories(min_importance: f32) -> ComplexQueryBuilder {
    ComplexQueryBuilder::important_memories(min_importance)
}

/// Create a text search query
pub fn text_search(text: impl Into<String>, min_score: f32) -> ComplexQueryBuilder {
    ComplexQueryBuilder::text_search(text, min_score)
}