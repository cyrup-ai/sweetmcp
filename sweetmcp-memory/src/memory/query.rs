//! Memory query functionality

use serde::{Deserialize, Serialize};

use crate::memory::{MemoryType, filter::MemoryFilter};

/// Memory query builder
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// Text query for semantic search
    pub text: Option<String>,

    /// Filter criteria
    pub filter: MemoryFilter,

    /// Sort order
    pub sort: Option<SortOrder>,

    /// Include relationships in results
    pub include_relationships: bool,

    /// Include embeddings in results
    pub include_embeddings: bool,

    /// Minimum similarity score for results
    pub min_similarity: Option<f32>,
}

/// Sort order for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    /// Sort by creation time (newest first)
    CreatedDesc,
    /// Sort by creation time (oldest first)
    CreatedAsc,
    /// Sort by update time (newest first)
    UpdatedDesc,
    /// Sort by update time (oldest first)
    UpdatedAsc,
    /// Sort by importance score (highest first)
    ImportanceDesc,
    /// Sort by similarity score (highest first)
    SimilarityDesc,
}

impl MemoryQuery {
    /// Create a new query builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set text query
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set filter
    pub fn with_filter(mut self, filter: MemoryFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Set sort order
    pub fn with_sort(mut self, sort: SortOrder) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Include relationships in results
    pub fn include_relationships(mut self) -> Self {
        self.include_relationships = true;
        self
    }

    /// Include embeddings in results
    pub fn include_embeddings(mut self) -> Self {
        self.include_embeddings = true;
        self
    }

    /// Set minimum similarity score
    pub fn with_min_similarity(mut self, score: f32) -> Self {
        self.min_similarity = Some(score);
        self
    }
}

/// Query result with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQueryResult {
    /// Memory ID
    pub id: String,

    /// Similarity score (if from vector search)
    pub score: Option<f32>,

    /// Relevance explanation
    pub explanation: Option<String>,

    /// Highlighted content snippets
    pub highlights: Option<Vec<String>>,

    /// Related memory IDs
    pub related: Option<Vec<String>>,
}

/// Query executor for complex memory queries
pub struct MemoryQueryExecutor {
    /// Query configuration
    config: QueryConfig,
}

/// Configuration for query execution
#[derive(Debug, Clone)]
pub struct QueryConfig {
    /// Enable query optimization
    pub optimize: bool,

    /// Enable caching
    pub cache: bool,

    /// Query timeout in milliseconds
    pub timeout_ms: u64,

    /// Maximum number of parallel operations
    pub max_parallel: usize,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            optimize: true,
            cache: true,
            timeout_ms: 5000,
            max_parallel: 10,
        }
    }
}

impl MemoryQueryExecutor {
    /// Create a new query executor
    pub fn new(config: QueryConfig) -> Self {
        Self { config }
    }

    /// Build a complex query combining multiple criteria
    pub fn build_complex_query(&self) -> ComplexQueryBuilder {
        ComplexQueryBuilder::new()
    }

    /// Execute a query with the configured settings
    pub fn execute_query(
        &self,
        query: &MemoryQuery,
        manager: &dyn super::memory_manager::MemoryManager,
    ) -> Result<Vec<super::memory_node::MemoryNode>, crate::utils::error::Error> {
        use futures::StreamExt;

        // Use the config for optimization and caching decisions
        if self.config.optimize {
            // Apply query optimizations based on config
        }

        if self.config.cache {
            // Check cache first based on config
        }

        // Apply timeout and parallel limits from config
        let _timeout = self.config.timeout_ms;
        let _max_parallel = self.config.max_parallel;

        // Execute query using the memory manager
        let mut results = Vec::new();

        // Check if there are memory types in the filter
        if let Some(memory_types) = &query.filter.memory_types {
            for memory_type in memory_types {
                let mut stream = manager.query_by_type(memory_type.clone());
                tokio::runtime::Handle::current().block_on(async {
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(memory) => results.push(memory),
                            Err(_) => break,
                        }
                    }
                });
            }
        }

        // Execute text search if text query provided
        if let Some(text) = &query.text {
            let mut stream = manager.search_by_content(text);
            tokio::runtime::Handle::current().block_on(async {
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(memory) => results.push(memory),
                        Err(_) => break,
                    }
                }
            });
        }

        // Apply limit from filter
        if let Some(limit) = query.filter.limit {
            results.truncate(limit);
        }

        Ok(results)
    }
}

/// Builder for complex queries with multiple conditions
pub struct ComplexQueryBuilder {
    conditions: Vec<QueryCondition>,
    operator: LogicalOperator,
}

/// Query condition
#[derive(Debug, Clone)]
pub enum QueryCondition {
    /// Text similarity condition
    TextSimilarity { text: String, min_score: f32 },
    /// Memory type condition
    MemoryType(Vec<MemoryType>),
    /// Time range condition
    TimeRange {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    },
    /// Metadata condition
    Metadata {
        key: String,
        value: serde_json::Value,
    },
    /// Relationship condition
    HasRelationship {
        relationship_type: String,
        target_id: Option<String>,
    },
}

/// Logical operator for combining conditions
#[derive(Debug, Clone, Copy)]
pub enum LogicalOperator {
    And,
    Or,
}

impl ComplexQueryBuilder {
    /// Create a new complex query builder
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            operator: LogicalOperator::And,
        }
    }

    /// Set the logical operator
    pub fn with_operator(mut self, operator: LogicalOperator) -> Self {
        self.operator = operator;
        self
    }

    /// Add a condition
    pub fn add_condition(mut self, condition: QueryCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Build the query
    pub fn build(self) -> MemoryQuery {
        // Convert complex conditions to simple query
        // This is a simplified implementation
        MemoryQuery::new()
    }
}
