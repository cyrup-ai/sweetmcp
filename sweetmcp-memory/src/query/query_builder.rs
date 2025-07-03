//! Query builder for constructing complex queries

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::memory::MemoryType;
use crate::query::{QueryType, Result, QueryError};

/// Query builder for constructing memory queries
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    /// Query clauses
    clauses: Vec<QueryClause>,
    
    /// Sort options
    sort: Option<SortOptions>,
    
    /// Limit
    limit: Option<usize>,
    
    /// Offset
    offset: Option<usize>,
    
    /// Include fields
    includes: Vec<String>,
    
    /// Exclude fields
    excludes: Vec<String>,
}

/// Query clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryClause {
    /// Text search
    Text {
        field: String,
        query: String,
        fuzzy: bool,
    },
    
    /// Exact match
    Exact {
        field: String,
        value: serde_json::Value,
    },
    
    /// Range query
    Range {
        field: String,
        min: Option<serde_json::Value>,
        max: Option<serde_json::Value>,
    },
    
    /// Memory type filter
    MemoryType(Vec<MemoryType>),
    
    /// Exists check
    Exists {
        field: String,
    },
    
    /// Nested query
    Nested {
        operator: LogicalOperator,
        clauses: Vec<QueryClause>,
    },
}

/// Logical operators
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Sort options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOptions {
    /// Field to sort by
    pub field: String,
    
    /// Sort direction
    pub direction: SortDirection,
}

/// Sort direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a text search clause
    pub fn text(mut self, field: impl Into<String>, query: impl Into<String>) -> Self {
        self.clauses.push(QueryClause::Text {
            field: field.into(),
            query: query.into(),
            fuzzy: false,
        });
        self
    }
    
    /// Add a fuzzy text search clause
    pub fn fuzzy_text(mut self, field: impl Into<String>, query: impl Into<String>) -> Self {
        self.clauses.push(QueryClause::Text {
            field: field.into(),
            query: query.into(),
            fuzzy: true,
        });
        self
    }
    
    /// Add an exact match clause
    pub fn exact(mut self, field: impl Into<String>, value: serde_json::Value) -> Self {
        self.clauses.push(QueryClause::Exact {
            field: field.into(),
            value,
        });
        self
    }
    
    /// Add a range clause
    pub fn range(
        mut self,
        field: impl Into<String>,
        min: Option<serde_json::Value>,
        max: Option<serde_json::Value>,
    ) -> Self {
        self.clauses.push(QueryClause::Range {
            field: field.into(),
            min,
            max,
        });
        self
    }
    
    /// Filter by memory types
    pub fn memory_types(mut self, types: Vec<MemoryType>) -> Self {
        self.clauses.push(QueryClause::MemoryType(types));
        self
    }
    
    /// Check if field exists
    pub fn exists(mut self, field: impl Into<String>) -> Self {
        self.clauses.push(QueryClause::Exists {
            field: field.into(),
        });
        self
    }
    
    /// Add a nested AND query
    pub fn and(mut self, builder: QueryBuilder) -> Self {
        self.clauses.push(QueryClause::Nested {
            operator: LogicalOperator::And,
            clauses: builder.clauses,
        });
        self
    }
    
    /// Add a nested OR query
    pub fn or(mut self, builder: QueryBuilder) -> Self {
        self.clauses.push(QueryClause::Nested {
            operator: LogicalOperator::Or,
            clauses: builder.clauses,
        });
        self
    }
    
    /// Add a nested NOT query
    pub fn not(mut self, builder: QueryBuilder) -> Self {
        self.clauses.push(QueryClause::Nested {
            operator: LogicalOperator::Not,
            clauses: builder.clauses,
        });
        self
    }
    
    /// Set sort options
    pub fn sort(mut self, field: impl Into<String>, direction: SortDirection) -> Self {
        self.sort = Some(SortOptions {
            field: field.into(),
            direction,
        });
        self
    }
    
    /// Set limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Set offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
    
    /// Include specific fields
    pub fn include(mut self, field: impl Into<String>) -> Self {
        self.includes.push(field.into());
        self
    }
    
    /// Exclude specific fields
    pub fn exclude(mut self, field: impl Into<String>) -> Self {
        self.excludes.push(field.into());
        self
    }
    
    /// Build the query
    pub fn build(self) -> BuiltQuery {
        BuiltQuery {
            clauses: self.clauses,
            sort: self.sort,
            limit: self.limit,
            offset: self.offset,
            includes: self.includes,
            excludes: self.excludes,
        }
    }
}

/// Built query ready for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltQuery {
    /// Query clauses
    pub clauses: Vec<QueryClause>,
    
    /// Sort options
    pub sort: Option<SortOptions>,
    
    /// Limit
    pub limit: Option<usize>,
    
    /// Offset
    pub offset: Option<usize>,
    
    /// Include fields
    pub includes: Vec<String>,
    
    /// Exclude fields
    pub excludes: Vec<String>,
}

impl BuiltQuery {
    /// Convert to SQL WHERE clause (simplified)
    pub fn to_sql_where(&self) -> Result<String> {
        if self.clauses.is_empty() {
            return Ok("1=1".to_string());
        }
        
        let clause_sql: Result<Vec<String>> = self.clauses
            .iter()
            .map(|clause| clause_to_sql(clause))
            .collect();
        
        Ok(clause_sql?.join(" AND "))
    }
    
    /// Convert to MongoDB query (simplified)
    pub fn to_mongo_query(&self) -> serde_json::Value {
        let mut query = serde_json::Map::new();
        
        for clause in &self.clauses {
            match clause {
                QueryClause::Exact { field, value } => {
                    query.insert(field.clone(), value.clone());
                }
                QueryClause::Text { field, query: q, .. } => {
                    query.insert(
                        field.clone(),
                        serde_json::json!({ "$regex": q, "$options": "i" })
                    );
                }
                _ => {} // Simplified - would handle other cases
            }
        }
        
        serde_json::Value::Object(query)
    }
}

/// Convert a clause to SQL (simplified)
fn clause_to_sql(clause: &QueryClause) -> Result<String> {
    match clause {
        QueryClause::Exact { field, value } => {
            Ok(format!("{} = {}", field, value_to_sql(value)))
        }
        QueryClause::Text { field, query, fuzzy } => {
            if *fuzzy {
                Ok(format!("{} LIKE '%{}%'", field, query))
            } else {
                Ok(format!("{} = '{}'", field, query))
            }
        }
        QueryClause::Range { field, min, max } => {
            let mut parts = Vec::new();
            if let Some(min) = min {
                parts.push(format!("{} >= {}", field, value_to_sql(min)));
            }
            if let Some(max) = max {
                parts.push(format!("{} <= {}", field, value_to_sql(max)));
            }
            Ok(parts.join(" AND "))
        }
        _ => Ok("1=1".to_string()), // Simplified
    }
}

/// Convert a JSON value to SQL (simplified)
fn value_to_sql(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => format!("'{}'", s),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => "NULL".to_string(),
    }
}