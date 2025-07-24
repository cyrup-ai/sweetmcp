//! Core query builder types and structures
//!
//! This module provides the foundational types and data structures for
//! constructing complex memory queries with zero allocation patterns and
//! blazing-fast performance.

use serde::{Deserialize, Serialize};
use crate::memory::MemoryType;
use crate::query::Result;

/// Query builder for constructing memory queries
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    /// Query clauses
    pub(crate) clauses: Vec<QueryClause>,
    /// Sort options
    pub(crate) sort: Option<SortOptions>,
    /// Limit
    pub(crate) limit: Option<usize>,
    /// Offset
    pub(crate) offset: Option<usize>,
    /// Include fields
    pub(crate) includes: Vec<String>,
    /// Exclude fields
    pub(crate) excludes: Vec<String>,
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
    Exists { field: String },
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

    /// Get the number of clauses
    pub fn clause_count(&self) -> usize {
        self.clauses.len()
    }

    /// Check if the query builder is empty
    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    /// Get the limit value
    pub fn get_limit(&self) -> Option<usize> {
        self.limit
    }

    /// Get the offset value
    pub fn get_offset(&self) -> Option<usize> {
        self.offset
    }

    /// Get the sort options
    pub fn get_sort(&self) -> Option<&SortOptions> {
        self.sort.as_ref()
    }

    /// Get included fields
    pub fn get_includes(&self) -> &[String] {
        &self.includes
    }

    /// Get excluded fields
    pub fn get_excludes(&self) -> &[String] {
        &self.excludes
    }

    /// Get all clauses
    pub fn get_clauses(&self) -> &[QueryClause] {
        &self.clauses
    }

    /// Clear all clauses
    pub fn clear(&mut self) {
        self.clauses.clear();
        self.sort = None;
        self.limit = None;
        self.offset = None;
        self.includes.clear();
        self.excludes.clear();
    }

    /// Clone the query builder
    pub fn clone_builder(&self) -> Self {
        self.clone()
    }

    /// Check if query has sorting
    pub fn has_sort(&self) -> bool {
        self.sort.is_some()
    }

    /// Check if query has pagination
    pub fn has_pagination(&self) -> bool {
        self.limit.is_some() || self.offset.is_some()
    }

    /// Check if query has field filtering
    pub fn has_field_filtering(&self) -> bool {
        !self.includes.is_empty() || !self.excludes.is_empty()
    }

    /// Get estimated complexity score
    pub fn complexity_score(&self) -> usize {
        let mut score = 0;
        
        // Base score for each clause
        score += self.clauses.len();
        
        // Additional score for nested clauses
        for clause in &self.clauses {
            if let QueryClause::Nested { clauses, .. } = clause {
                score += clauses.len();
            }
        }
        
        // Additional score for sorting
        if self.sort.is_some() {
            score += 1;
        }
        
        // Additional score for field filtering
        score += self.includes.len() + self.excludes.len();
        
        score
    }

    /// Validate the query builder
    pub fn validate(&self) -> Result<()> {
        // Check for conflicting includes/excludes
        for include in &self.includes {
            if self.excludes.contains(include) {
                return Err(crate::query::QueryError::InvalidQuery(
                    format!("Field '{}' cannot be both included and excluded", include)
                ));
            }
        }

        // Validate limit and offset
        if let Some(limit) = self.limit {
            if limit == 0 {
                return Err(crate::query::QueryError::InvalidQuery(
                    "Limit cannot be zero".to_string()
                ));
            }
        }

        // Validate clauses
        for clause in &self.clauses {
            self.validate_clause(clause)?;
        }

        Ok(())
    }

    /// Validate a single clause
    fn validate_clause(&self, clause: &QueryClause) -> Result<()> {
        match clause {
            QueryClause::Text { field, query, .. } => {
                if field.is_empty() {
                    return Err(crate::query::QueryError::InvalidQuery(
                        "Text search field cannot be empty".to_string()
                    ));
                }
                if query.is_empty() {
                    return Err(crate::query::QueryError::InvalidQuery(
                        "Text search query cannot be empty".to_string()
                    ));
                }
            }
            QueryClause::Exact { field, .. } => {
                if field.is_empty() {
                    return Err(crate::query::QueryError::InvalidQuery(
                        "Exact match field cannot be empty".to_string()
                    ));
                }
            }
            QueryClause::Range { field, min, max } => {
                if field.is_empty() {
                    return Err(crate::query::QueryError::InvalidQuery(
                        "Range query field cannot be empty".to_string()
                    ));
                }
                if min.is_none() && max.is_none() {
                    return Err(crate::query::QueryError::InvalidQuery(
                        "Range query must have at least min or max value".to_string()
                    ));
                }
            }
            QueryClause::Exists { field } => {
                if field.is_empty() {
                    return Err(crate::query::QueryError::InvalidQuery(
                        "Exists check field cannot be empty".to_string()
                    ));
                }
            }
            QueryClause::MemoryType(types) => {
                if types.is_empty() {
                    return Err(crate::query::QueryError::InvalidQuery(
                        "Memory type filter cannot be empty".to_string()
                    ));
                }
            }
            QueryClause::Nested { clauses, .. } => {
                if clauses.is_empty() {
                    return Err(crate::query::QueryError::InvalidQuery(
                        "Nested query cannot be empty".to_string()
                    ));
                }
                for nested_clause in clauses {
                    self.validate_clause(nested_clause)?;
                }
            }
        }
        Ok(())
    }

    /// Get query statistics
    pub fn get_statistics(&self) -> QueryStatistics {
        let mut stats = QueryStatistics::default();
        
        stats.total_clauses = self.clauses.len();
        stats.has_sorting = self.sort.is_some();
        stats.has_limit = self.limit.is_some();
        stats.has_offset = self.offset.is_some();
        stats.include_count = self.includes.len();
        stats.exclude_count = self.excludes.len();
        stats.complexity_score = self.complexity_score();

        // Count clause types
        for clause in &self.clauses {
            self.count_clause_types(clause, &mut stats);
        }

        stats
    }

    /// Count clause types recursively
    fn count_clause_types(&self, clause: &QueryClause, stats: &mut QueryStatistics) {
        match clause {
            QueryClause::Text { .. } => stats.text_clauses += 1,
            QueryClause::Exact { .. } => stats.exact_clauses += 1,
            QueryClause::Range { .. } => stats.range_clauses += 1,
            QueryClause::MemoryType(_) => stats.memory_type_clauses += 1,
            QueryClause::Exists { .. } => stats.exists_clauses += 1,
            QueryClause::Nested { clauses, .. } => {
                stats.nested_clauses += 1;
                for nested_clause in clauses {
                    self.count_clause_types(nested_clause, stats);
                }
            }
        }
    }
}

/// Query statistics for analysis
#[derive(Debug, Clone, Default)]
pub struct QueryStatistics {
    pub total_clauses: usize,
    pub text_clauses: usize,
    pub exact_clauses: usize,
    pub range_clauses: usize,
    pub memory_type_clauses: usize,
    pub exists_clauses: usize,
    pub nested_clauses: usize,
    pub has_sorting: bool,
    pub has_limit: bool,
    pub has_offset: bool,
    pub include_count: usize,
    pub exclude_count: usize,
    pub complexity_score: usize,
}

impl QueryStatistics {
    /// Check if the query is simple (low complexity)
    pub fn is_simple(&self) -> bool {
        self.complexity_score <= 3 && self.nested_clauses == 0
    }

    /// Check if the query is complex (high complexity)
    pub fn is_complex(&self) -> bool {
        self.complexity_score > 10 || self.nested_clauses > 2
    }

    /// Get the dominant clause type
    pub fn dominant_clause_type(&self) -> &'static str {
        let clause_counts = [
            ("text", self.text_clauses),
            ("exact", self.exact_clauses),
            ("range", self.range_clauses),
            ("memory_type", self.memory_type_clauses),
            ("exists", self.exists_clauses),
            ("nested", self.nested_clauses),
        ];

        clause_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| *name)
            .unwrap_or("none")
    }
}

impl SortOptions {
    /// Create new sort options
    pub fn new(field: String, direction: SortDirection) -> Self {
        Self { field, direction }
    }

    /// Create ascending sort
    pub fn asc(field: String) -> Self {
        Self::new(field, SortDirection::Asc)
    }

    /// Create descending sort
    pub fn desc(field: String) -> Self {
        Self::new(field, SortDirection::Desc)
    }

    /// Check if sorting is ascending
    pub fn is_ascending(&self) -> bool {
        matches!(self.direction, SortDirection::Asc)
    }

    /// Check if sorting is descending
    pub fn is_descending(&self) -> bool {
        matches!(self.direction, SortDirection::Desc)
    }
}

impl LogicalOperator {
    /// Check if operator is AND
    pub fn is_and(&self) -> bool {
        matches!(self, LogicalOperator::And)
    }

    /// Check if operator is OR
    pub fn is_or(&self) -> bool {
        matches!(self, LogicalOperator::Or)
    }

    /// Check if operator is NOT
    pub fn is_not(&self) -> bool {
        matches!(self, LogicalOperator::Not)
    }

    /// Get operator precedence (higher number = higher precedence)
    pub fn precedence(&self) -> u8 {
        match self {
            LogicalOperator::Not => 3,
            LogicalOperator::And => 2,
            LogicalOperator::Or => 1,
        }
    }
}

impl QueryClause {
    /// Get the field name for this clause (if applicable)
    pub fn field_name(&self) -> Option<&str> {
        match self {
            QueryClause::Text { field, .. } => Some(field),
            QueryClause::Exact { field, .. } => Some(field),
            QueryClause::Range { field, .. } => Some(field),
            QueryClause::Exists { field } => Some(field),
            QueryClause::MemoryType(_) => None,
            QueryClause::Nested { .. } => None,
        }
    }

    /// Check if this clause affects a specific field
    pub fn affects_field(&self, field_name: &str) -> bool {
        match self {
            QueryClause::Text { field, .. } => field == field_name,
            QueryClause::Exact { field, .. } => field == field_name,
            QueryClause::Range { field, .. } => field == field_name,
            QueryClause::Exists { field } => field == field_name,
            QueryClause::MemoryType(_) => false,
            QueryClause::Nested { clauses, .. } => {
                clauses.iter().any(|clause| clause.affects_field(field_name))
            }
        }
    }

    /// Get complexity score for this clause
    pub fn complexity_score(&self) -> usize {
        match self {
            QueryClause::Text { .. } => 1,
            QueryClause::Exact { .. } => 1,
            QueryClause::Range { .. } => 2,
            QueryClause::Exists { .. } => 1,
            QueryClause::MemoryType(_) => 1,
            QueryClause::Nested { clauses, .. } => {
                1 + clauses.iter().map(|c| c.complexity_score()).sum::<usize>()
            }
        }
    }
}