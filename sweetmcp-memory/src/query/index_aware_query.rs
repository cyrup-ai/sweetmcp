//! Index-aware query optimization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::query::{QueryPlan, QueryStep, QueryType, Result, QueryError};

/// Index information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    /// Index name
    pub name: String,
    
    /// Indexed fields
    pub fields: Vec<String>,
    
    /// Index type
    pub index_type: IndexType,
    
    /// Index statistics
    pub stats: IndexStats,
}

/// Index types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexType {
    /// B-tree index
    BTree,
    /// Hash index
    Hash,
    /// Full-text search index
    FullText,
    /// Vector index
    Vector,
    /// Composite index
    Composite,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Number of entries
    pub entry_count: u64,
    
    /// Index size in bytes
    pub size_bytes: u64,
    
    /// Average query time in milliseconds
    pub avg_query_time_ms: f64,
    
    /// Last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Index-aware query planner
pub struct IndexAwareQueryPlanner {
    /// Available indexes
    indexes: HashMap<String, IndexInfo>,
}

impl IndexAwareQueryPlanner {
    /// Create a new planner
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
        }
    }
    
    /// Register an index
    pub fn register_index(&mut self, index: IndexInfo) {
        self.indexes.insert(index.name.clone(), index);
    }
    
    /// Plan a query using available indexes
    pub fn plan_query(&self, query_type: QueryType, fields: &[String]) -> Result<QueryPlan> {
        let mut steps = Vec::new();
        let mut use_index = false;
        let mut index_name = None;
        let mut cost = 100.0; // Base cost
        
        // Check if we can use an index
        for (name, index) in &self.indexes {
            if self.can_use_index(index, query_type, fields) {
                use_index = true;
                index_name = Some(name.clone());
                cost = 10.0; // Much lower cost with index
                
                steps.push(QueryStep {
                    name: "Index Lookup".to_string(),
                    description: format!("Use index '{}' for fast lookup", name),
                    cost: 5.0,
                    parallel: false,
                });
                
                break;
            }
        }
        
        if !use_index {
            steps.push(QueryStep {
                name: "Full Scan".to_string(),
                description: "Scan all documents (no suitable index found)".to_string(),
                cost: 80.0,
                parallel: true,
            });
        }
        
        // Add filtering step
        steps.push(QueryStep {
            name: "Filter Results".to_string(),
            description: "Apply query filters".to_string(),
            cost: 10.0,
            parallel: true,
        });
        
        // Add sorting step if needed
        if query_type == QueryType::Similarity {
            steps.push(QueryStep {
                name: "Sort by Score".to_string(),
                description: "Sort results by similarity score".to_string(),
                cost: 5.0,
                parallel: false,
            });
        }
        
        Ok(QueryPlan {
            query_type,
            cost,
            use_index,
            index_name,
            steps,
        })
    }
    
    /// Check if an index can be used for a query
    fn can_use_index(
        &self,
        index: &IndexInfo,
        query_type: QueryType,
        fields: &[String],
    ) -> bool {
        match query_type {
            QueryType::Exact => {
                // Check if all query fields are in the index
                fields.iter().all(|field| index.fields.contains(field))
                    && matches!(index.index_type, IndexType::BTree | IndexType::Hash)
            }
            QueryType::FullText => {
                matches!(index.index_type, IndexType::FullText)
                    && fields.iter().any(|field| index.fields.contains(field))
            }
            QueryType::Similarity => {
                matches!(index.index_type, IndexType::Vector)
            }
            _ => false,
        }
    }
}

impl Default for IndexAwareQueryPlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Index optimizer
pub struct IndexOptimizer {
    /// Query history
    query_history: Vec<QueryInfo>,
    
    /// Optimization threshold
    threshold: f64,
}

/// Query information for optimization
#[derive(Debug, Clone)]
struct QueryInfo {
    fields: Vec<String>,
    query_type: QueryType,
    execution_time_ms: f64,
    result_count: usize,
}

impl IndexOptimizer {
    /// Create a new optimizer
    pub fn new(threshold: f64) -> Self {
        Self {
            query_history: Vec::new(),
            threshold,
        }
    }
    
    /// Record a query execution
    pub fn record_query(
        &mut self,
        fields: Vec<String>,
        query_type: QueryType,
        execution_time_ms: f64,
        result_count: usize,
    ) {
        self.query_history.push(QueryInfo {
            fields,
            query_type,
            execution_time_ms,
            result_count,
        });
        
        // Keep only recent history
        if self.query_history.len() > 1000 {
            self.query_history.drain(0..self.query_history.len() - 1000);
        }
    }
    
    /// Recommend indexes based on query patterns
    pub fn recommend_indexes(&self) -> Vec<IndexRecommendation> {
        let mut recommendations = Vec::new();
        let mut field_usage: HashMap<Vec<String>, (usize, f64)> = HashMap::new();
        
        // Analyze query patterns
        for query in &self.query_history {
            if query.execution_time_ms > self.threshold {
                let key = query.fields.clone();
                let entry = field_usage.entry(key).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += query.execution_time_ms;
            }
        }
        
        // Generate recommendations
        for (fields, (count, total_time)) in field_usage {
            if count >= 5 {
                // Recommend index for frequently slow queries
                recommendations.push(IndexRecommendation {
                    fields: fields.clone(),
                    index_type: IndexType::BTree,
                    estimated_improvement: total_time * 0.8, // Estimate 80% improvement
                    reason: format!(
                        "Frequent slow queries on fields: {:?} ({} occurrences)",
                        fields, count
                    ),
                });
            }
        }
        
        recommendations
    }
}

/// Index recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRecommendation {
    /// Fields to index
    pub fields: Vec<String>,
    
    /// Recommended index type
    pub index_type: IndexType,
    
    /// Estimated performance improvement in ms
    pub estimated_improvement: f64,
    
    /// Reason for recommendation
    pub reason: String,
}