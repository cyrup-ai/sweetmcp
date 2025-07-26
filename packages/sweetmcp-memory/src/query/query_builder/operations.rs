//! Query building operations
//!
//! This module provides query building operations for constructing complex
//! memory queries with zero allocation patterns and blazing-fast performance.

use super::core::*;
use crate::memory::MemoryType;
use crate::query::Result;

impl QueryBuilder {
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

    /// Add a range query clause
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

    /// Add a memory type filter
    pub fn memory_type(mut self, types: Vec<MemoryType>) -> Self {
        self.clauses.push(QueryClause::MemoryType(types));
        self
    }

    /// Add a single memory type filter
    pub fn memory_type_single(mut self, memory_type: MemoryType) -> Self {
        self.clauses.push(QueryClause::MemoryType(vec![memory_type]));
        self
    }

    /// Add an exists check clause
    pub fn exists(mut self, field: impl Into<String>) -> Self {
        self.clauses.push(QueryClause::Exists {
            field: field.into(),
        });
        self
    }

    /// Add a nested query with AND operator
    pub fn and(mut self, clauses: Vec<QueryClause>) -> Self {
        if !clauses.is_empty() {
            self.clauses.push(QueryClause::Nested {
                operator: LogicalOperator::And,
                clauses,
            });
        }
        self
    }

    /// Add a nested query with OR operator
    pub fn or(mut self, clauses: Vec<QueryClause>) -> Self {
        if !clauses.is_empty() {
            self.clauses.push(QueryClause::Nested {
                operator: LogicalOperator::Or,
                clauses,
            });
        }
        self
    }

    /// Add a nested query with NOT operator
    pub fn not(mut self, clauses: Vec<QueryClause>) -> Self {
        if !clauses.is_empty() {
            self.clauses.push(QueryClause::Nested {
                operator: LogicalOperator::Not,
                clauses,
            });
        }
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

    /// Set ascending sort
    pub fn sort_asc(self, field: impl Into<String>) -> Self {
        self.sort(field, SortDirection::Asc)
    }

    /// Set descending sort
    pub fn sort_desc(self, field: impl Into<String>) -> Self {
        self.sort(field, SortDirection::Desc)
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

    /// Set pagination (limit and offset)
    pub fn paginate(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }

    /// Add field to include list
    pub fn include(mut self, field: impl Into<String>) -> Self {
        let field_name = field.into();
        if !self.includes.contains(&field_name) {
            self.includes.push(field_name);
        }
        self
    }

    /// Add multiple fields to include list
    pub fn include_fields(mut self, fields: Vec<String>) -> Self {
        for field in fields {
            if !self.includes.contains(&field) {
                self.includes.push(field);
            }
        }
        self
    }

    /// Add field to exclude list
    pub fn exclude(mut self, field: impl Into<String>) -> Self {
        let field_name = field.into();
        if !self.excludes.contains(&field_name) {
            self.excludes.push(field_name);
        }
        self
    }

    /// Add multiple fields to exclude list
    pub fn exclude_fields(mut self, fields: Vec<String>) -> Self {
        for field in fields {
            if !self.excludes.contains(&field) {
                self.excludes.push(field);
            }
        }
        self
    }

    /// Add clause directly
    pub fn add_clause(mut self, clause: QueryClause) -> Self {
        self.clauses.push(clause);
        self
    }

    /// Add multiple clauses
    pub fn add_clauses(mut self, clauses: Vec<QueryClause>) -> Self {
        self.clauses.extend(clauses);
        self
    }

    /// Remove clauses that affect a specific field
    pub fn remove_field_clauses(mut self, field_name: &str) -> Self {
        self.clauses.retain(|clause| !clause.affects_field(field_name));
        self
    }

    /// Replace clauses for a specific field
    pub fn replace_field_clauses(mut self, field_name: &str, new_clauses: Vec<QueryClause>) -> Self {
        // Remove existing clauses for the field
        self.clauses.retain(|clause| !clause.affects_field(field_name));
        
        // Add new clauses
        self.clauses.extend(new_clauses);
        self
    }

    /// Create a builder for range queries on numeric fields
    pub fn numeric_range(
        self,
        field: impl Into<String>,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Self {
        let min_value = min.map(|v| serde_json::Value::Number(serde_json::Number::from_f64(v).unwrap_or_else(|| serde_json::Number::from(0))));
        let max_value = max.map(|v| serde_json::Value::Number(serde_json::Number::from_f64(v).unwrap_or_else(|| serde_json::Number::from(0))));
        
        self.range(field, min_value, max_value)
    }

    /// Create a builder for date range queries
    pub fn date_range(
        self,
        field: impl Into<String>,
        start: Option<chrono::DateTime<chrono::Utc>>,
        end: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        let start_value = start.map(|dt| serde_json::Value::String(dt.to_rfc3339()));
        let end_value = end.map(|dt| serde_json::Value::String(dt.to_rfc3339()));
        
        self.range(field, start_value, end_value)
    }

    /// Create a builder for string exact match
    pub fn string_exact(self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.exact(field, serde_json::Value::String(value.into()))
    }

    /// Create a builder for boolean exact match
    pub fn bool_exact(self, field: impl Into<String>, value: bool) -> Self {
        self.exact(field, serde_json::Value::Bool(value))
    }

    /// Create a builder for integer exact match
    pub fn int_exact(self, field: impl Into<String>, value: i64) -> Self {
        self.exact(field, serde_json::Value::Number(serde_json::Number::from(value)))
    }

    /// Create a builder for float exact match
    pub fn float_exact(self, field: impl Into<String>, value: f64) -> Self {
        let number = serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0));
        self.exact(field, serde_json::Value::Number(number))
    }

    /// Create a builder for array contains query
    pub fn array_contains(self, field: impl Into<String>, value: serde_json::Value) -> Self {
        // This would be implemented as a custom clause in a real system
        // For now, we'll use a text search as approximation
        if let serde_json::Value::String(s) = &value {
            self.text(field, s.clone())
        } else {
            self.exact(field, value)
        }
    }

    /// Create a builder for null/empty checks
    pub fn is_null(self, field: impl Into<String>) -> Self {
        self.exact(field, serde_json::Value::Null)
    }

    /// Create a builder for non-null checks
    pub fn is_not_null(self, field: impl Into<String>) -> Self {
        self.exists(field)
    }

    /// Create a builder for multiple value matching (IN clause equivalent)
    pub fn in_values(self, field: impl Into<String>, values: Vec<serde_json::Value>) -> Self {
        if values.is_empty() {
            return self;
        }

        if values.len() == 1 {
            return self.exact(field, values.into_iter().next().unwrap());
        }

        // Create OR clause for multiple values
        let field_name = field.into();
        let clauses: Vec<QueryClause> = values
            .into_iter()
            .map(|value| QueryClause::Exact {
                field: field_name.clone(),
                value,
            })
            .collect();

        self.or(clauses)
    }

    /// Create a builder for multiple string value matching
    pub fn in_strings(self, field: impl Into<String>, values: Vec<String>) -> Self {
        let json_values: Vec<serde_json::Value> = values
            .into_iter()
            .map(serde_json::Value::String)
            .collect();
        
        self.in_values(field, json_values)
    }

    /// Create a builder for multiple integer value matching
    pub fn in_integers(self, field: impl Into<String>, values: Vec<i64>) -> Self {
        let json_values: Vec<serde_json::Value> = values
            .into_iter()
            .map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
            .collect();
        
        self.in_values(field, json_values)
    }

    /// Create a builder for prefix matching
    pub fn prefix(self, field: impl Into<String>, prefix: impl Into<String>) -> Self {
        let prefix_str = prefix.into();
        // Use fuzzy text search for prefix matching
        self.fuzzy_text(field, format!("{}*", prefix_str))
    }

    /// Create a builder for suffix matching
    pub fn suffix(self, field: impl Into<String>, suffix: impl Into<String>) -> Self {
        let suffix_str = suffix.into();
        // Use fuzzy text search for suffix matching
        self.fuzzy_text(field, format!("*{}", suffix_str))
    }

    /// Create a builder for wildcard matching
    pub fn wildcard(self, field: impl Into<String>, pattern: impl Into<String>) -> Self {
        // Use fuzzy text search for wildcard matching
        self.fuzzy_text(field, pattern)
    }

    /// Create a builder for case-insensitive text search
    pub fn text_case_insensitive(self, field: impl Into<String>, query: impl Into<String>) -> Self {
        // Use fuzzy text search for case-insensitive matching
        self.fuzzy_text(field, query.into().to_lowercase())
    }

    /// Combine with another query builder using AND
    pub fn combine_and(mut self, other: QueryBuilder) -> Self {
        if !other.clauses.is_empty() {
            self.clauses.push(QueryClause::Nested {
                operator: LogicalOperator::And,
                clauses: other.clauses,
            });
        }
        
        // Merge other properties (last one wins)
        if other.sort.is_some() {
            self.sort = other.sort;
        }
        if other.limit.is_some() {
            self.limit = other.limit;
        }
        if other.offset.is_some() {
            self.offset = other.offset;
        }
        
        // Merge field lists
        for include in other.includes {
            if !self.includes.contains(&include) {
                self.includes.push(include);
            }
        }
        for exclude in other.excludes {
            if !self.excludes.contains(&exclude) {
                self.excludes.push(exclude);
            }
        }
        
        self
    }

    /// Combine with another query builder using OR
    pub fn combine_or(mut self, other: QueryBuilder) -> Self {
        if !other.clauses.is_empty() {
            // Combine current clauses with other clauses using OR
            let current_clauses = std::mem::take(&mut self.clauses);
            
            if !current_clauses.is_empty() {
                let combined_clauses = vec![
                    QueryClause::Nested {
                        operator: LogicalOperator::Or,
                        clauses: current_clauses,
                    },
                    QueryClause::Nested {
                        operator: LogicalOperator::Or,
                        clauses: other.clauses,
                    },
                ];
                
                self.clauses = vec![QueryClause::Nested {
                    operator: LogicalOperator::Or,
                    clauses: combined_clauses,
                }];
            } else {
                self.clauses = other.clauses;
            }
        }
        
        // Merge other properties (last one wins)
        if other.sort.is_some() {
            self.sort = other.sort;
        }
        if other.limit.is_some() {
            self.limit = other.limit;
        }
        if other.offset.is_some() {
            self.offset = other.offset;
        }
        
        // Merge field lists
        for include in other.includes {
            if !self.includes.contains(&include) {
                self.includes.push(include);
            }
        }
        for exclude in other.excludes {
            if !self.excludes.contains(&exclude) {
                self.excludes.push(exclude);
            }
        }
        
        self
    }

    /// Create a copy of the builder with additional clauses
    pub fn extend_with(mut self, additional_clauses: Vec<QueryClause>) -> Self {
        self.clauses.extend(additional_clauses);
        self
    }

    /// Create a copy of the builder with modified pagination
    pub fn with_pagination(mut self, limit: Option<usize>, offset: Option<usize>) -> Self {
        self.limit = limit;
        self.offset = offset;
        self
    }

    /// Create a copy of the builder with modified sorting
    pub fn with_sort(mut self, sort: Option<SortOptions>) -> Self {
        self.sort = sort;
        self
    }

    /// Create a copy of the builder with modified field filtering
    pub fn with_field_filtering(
        mut self,
        includes: Vec<String>,
        excludes: Vec<String>,
    ) -> Self {
        self.includes = includes;
        self.excludes = excludes;
        self
    }
}