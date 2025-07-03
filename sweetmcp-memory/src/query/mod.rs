//! Query optimization and building module for mem0-rs
//! 
//! This module provides advanced query capabilities including
//! query building, optimization, monitoring, and index-aware querying.

pub mod index_aware_query;
pub mod query_builder;
pub mod query_monitor;
pub mod query_optimizer;

// Re-export main types
pub use index_aware_query::*;
pub use query_builder::*;
pub use query_monitor::*;
pub use query_optimizer::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query result type
pub type Result<T> = std::result::Result<T, QueryError>;

/// Query error types
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Timeout: query exceeded {0:?}")]
    Timeout(std::time::Duration),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Index not found: {0}")]
    IndexNotFound(String),
}

/// Query type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    /// Simple equality match
    Exact,
    /// Vector similarity search
    Similarity,
    /// Full-text search
    FullText,
    /// Graph traversal
    GraphTraversal,
    /// Hybrid query combining multiple types
    Hybrid,
}

/// Query execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    /// Query type
    pub query_type: QueryType,
    /// Estimated cost
    pub cost: f64,
    /// Whether to use index
    pub use_index: bool,
    /// Index name if applicable
    pub index_name: Option<String>,
    /// Execution steps
    pub steps: Vec<QueryStep>,
}

/// Individual query execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStep {
    /// Step name
    pub name: String,
    /// Step description
    pub description: String,
    /// Estimated cost
    pub cost: f64,
    /// Whether this step can be parallelized
    pub parallel: bool,
}

/// Query statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    /// Total execution time in milliseconds
    pub execution_time_ms: u64,
    /// Number of documents scanned
    pub documents_scanned: u64,
    /// Number of documents returned
    pub documents_returned: u64,
    /// Whether an index was used
    pub index_used: bool,
    /// Cache hit rate
    pub cache_hit_rate: f32,
}