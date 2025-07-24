//! Query builder module
//!
//! This module provides comprehensive query building functionality for constructing
//! complex memory queries with zero allocation patterns and blazing-fast performance.

pub mod core;
pub mod operations;
pub mod compilation;

// Re-export core types and traits for ergonomic usage
pub use core::{
    QueryBuilder, QueryClause, LogicalOperator, SortOptions, SortDirection,
    QueryStatistics,
};

pub use compilation::QueryExecutionPlan;

// Convenience re-exports for common functionality
pub use operations::*; // All the builder methods are implemented as inherent methods

/// Create a new query builder
pub fn query() -> QueryBuilder {
    QueryBuilder::new()
}

/// Create a query builder with text search
pub fn text_query(field: impl Into<String>, query: impl Into<String>) -> QueryBuilder {
    QueryBuilder::new().text(field, query)
}

/// Create a query builder with exact match
pub fn exact_query(field: impl Into<String>, value: serde_json::Value) -> QueryBuilder {
    QueryBuilder::new().exact(field, value)
}

/// Create a query builder with range query
pub fn range_query(
    field: impl Into<String>,
    min: Option<serde_json::Value>,
    max: Option<serde_json::Value>,
) -> QueryBuilder {
    QueryBuilder::new().range(field, min, max)
}

/// Create a query builder with memory type filter
pub fn memory_type_query(memory_type: crate::memory::MemoryType) -> QueryBuilder {
    QueryBuilder::new().memory_type_single(memory_type)
}

/// Create a query builder with exists check
pub fn exists_query(field: impl Into<String>) -> QueryBuilder {
    QueryBuilder::new().exists(field)
}