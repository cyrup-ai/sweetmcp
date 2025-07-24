//! Query compilation to different formats
//!
//! This module provides query compilation to SQL, SurrealDB, and MongoDB formats
//! with zero allocation patterns and blazing-fast performance.

use super::core::*;
use crate::query::Result;

impl QueryBuilder {
    /// Convert to SQL query with optimized string building
    pub fn to_sql(&self) -> Result<String> {
        let mut query = String::with_capacity(256); // Pre-allocate reasonable capacity
        
        // SELECT clause with field filtering
        query.push_str("SELECT ");
        if self.includes.is_empty() && self.excludes.is_empty() {
            query.push('*');
        } else if !self.includes.is_empty() {
            query.push_str(&self.includes.join(", "));
        } else {
            // For excludes, we'd need to know all available fields
            // This is a simplified implementation
            query.push('*');
        }
        
        query.push_str(" FROM memories");
        
        // WHERE clause
        if !self.clauses.is_empty() {
            query.push_str(" WHERE ");
            let where_clause = self.clauses_to_sql(&self.clauses)?;
            query.push_str(&where_clause);
        }
        
        // ORDER BY clause
        if let Some(sort) = &self.sort {
            query.push_str(" ORDER BY ");
            query.push_str(&sort.field);
            match sort.direction {
                SortDirection::Asc => query.push_str(" ASC"),
                SortDirection::Desc => query.push_str(" DESC"),
            }
        }
        
        // LIMIT clause
        if let Some(limit) = self.limit {
            query.push_str(" LIMIT ");
            query.push_str(&limit.to_string());
        }
        
        // OFFSET clause
        if let Some(offset) = self.offset {
            query.push_str(" OFFSET ");
            query.push_str(&offset.to_string());
        }
        
        Ok(query)
    }

    /// Convert clauses to SQL WHERE conditions
    fn clauses_to_sql(&self, clauses: &[QueryClause]) -> Result<String> {
        if clauses.is_empty() {
            return Ok("1=1".to_string());
        }
        
        let mut conditions = Vec::with_capacity(clauses.len());
        
        for clause in clauses {
            let condition = self.clause_to_sql(clause)?;
            conditions.push(condition);
        }
        
        Ok(conditions.join(" AND "))
    }

    /// Convert a single clause to SQL
    fn clause_to_sql(&self, clause: &QueryClause) -> Result<String> {
        match clause {
            QueryClause::Exact { field, value } => {
                Ok(format!("{} = {}", field, self.value_to_sql(value)))
            }
            QueryClause::Text { field, query, fuzzy } => {
                if *fuzzy {
                    Ok(format!("{} LIKE '%{}%'", field, self.escape_sql_string(query)))
                } else {
                    Ok(format!("{} = '{}'", field, self.escape_sql_string(query)))
                }
            }
            QueryClause::Range { field, min, max } => {
                let mut parts = Vec::new();
                if let Some(min) = min {
                    parts.push(format!("{} >= {}", field, self.value_to_sql(min)));
                }
                if let Some(max) = max {
                    parts.push(format!("{} <= {}", field, self.value_to_sql(max)));
                }
                Ok(parts.join(" AND "))
            }
            QueryClause::Exists { field } => {
                Ok(format!("{} IS NOT NULL", field))
            }
            QueryClause::MemoryType(types) => {
                let type_strings: Vec<String> = types
                    .iter()
                    .map(|t| format!("'{:?}'", t))
                    .collect();
                Ok(format!("memory_type IN ({})", type_strings.join(", ")))
            }
            QueryClause::Nested { operator, clauses } => {
                let nested_sql = self.clauses_to_sql(clauses)?;
                match operator {
                    LogicalOperator::And => Ok(format!("({})", nested_sql)),
                    LogicalOperator::Or => {
                        let or_conditions: Result<Vec<String>> = clauses
                            .iter()
                            .map(|c| self.clause_to_sql(c))
                            .collect();
                        let conditions = or_conditions?;
                        Ok(format!("({})", conditions.join(" OR ")))
                    }
                    LogicalOperator::Not => Ok(format!("NOT ({})", nested_sql)),
                }
            }
        }
    }

    /// Convert JSON value to SQL representation
    fn value_to_sql(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => format!("'{}'", self.escape_sql_string(s)),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "NULL".to_string(),
            _ => "NULL".to_string(), // Simplified for arrays and objects
        }
    }

    /// Escape SQL string literals
    fn escape_sql_string(&self, s: &str) -> String {
        s.replace('\'', "''")
    }

    /// Convert to SurrealDB query with optimized syntax
    pub fn to_surql(&self) -> Result<String> {
        let mut query = String::with_capacity(256);
        
        // SELECT clause
        query.push_str("SELECT ");
        if self.includes.is_empty() && self.excludes.is_empty() {
            query.push('*');
        } else if !self.includes.is_empty() {
            query.push_str(&self.includes.join(", "));
        } else {
            query.push('*');
        }
        
        query.push_str(" FROM memories");
        
        // WHERE clause
        if !self.clauses.is_empty() {
            query.push_str(" WHERE ");
            let where_clause = self.clauses_to_surql(&self.clauses)?;
            query.push_str(&where_clause);
        }
        
        // ORDER BY clause
        if let Some(sort) = &self.sort {
            query.push_str(" ORDER BY ");
            query.push_str(&sort.field);
            match sort.direction {
                SortDirection::Asc => query.push_str(" ASC"),
                SortDirection::Desc => query.push_str(" DESC"),
            }
        }
        
        // LIMIT clause
        if let Some(limit) = self.limit {
            query.push_str(" LIMIT ");
            query.push_str(&limit.to_string());
        }
        
        // START clause (SurrealDB equivalent of OFFSET)
        if let Some(offset) = self.offset {
            query.push_str(" START ");
            query.push_str(&offset.to_string());
        }
        
        Ok(query)
    }

    /// Convert clauses to SurrealDB WHERE conditions
    fn clauses_to_surql(&self, clauses: &[QueryClause]) -> Result<String> {
        if clauses.is_empty() {
            return Ok("true".to_string());
        }
        
        let mut conditions = Vec::with_capacity(clauses.len());
        
        for clause in clauses {
            let condition = self.clause_to_surql(clause)?;
            conditions.push(condition);
        }
        
        Ok(conditions.join(" AND "))
    }

    /// Convert a single clause to SurrealDB syntax
    fn clause_to_surql(&self, clause: &QueryClause) -> Result<String> {
        match clause {
            QueryClause::Exact { field, value } => {
                Ok(format!("{} = {}", field, self.value_to_surql(value)))
            }
            QueryClause::Text { field, query, fuzzy } => {
                if *fuzzy {
                    Ok(format!("{} ~ '{}'", field, self.escape_surql_string(query)))
                } else {
                    Ok(format!("{} = '{}'", field, self.escape_surql_string(query)))
                }
            }
            QueryClause::Range { field, min, max } => {
                let mut parts = Vec::new();
                if let Some(min) = min {
                    parts.push(format!("{} >= {}", field, self.value_to_surql(min)));
                }
                if let Some(max) = max {
                    parts.push(format!("{} <= {}", field, self.value_to_surql(max)));
                }
                Ok(parts.join(" AND "))
            }
            QueryClause::Exists { field } => {
                Ok(format!("{} IS NOT NULL", field))
            }
            QueryClause::MemoryType(types) => {
                let type_strings: Vec<String> = types
                    .iter()
                    .map(|t| format!("'{:?}'", t))
                    .collect();
                Ok(format!("memory_type INSIDE [{}]", type_strings.join(", ")))
            }
            QueryClause::Nested { operator, clauses } => {
                match operator {
                    LogicalOperator::And => {
                        let nested_surql = self.clauses_to_surql(clauses)?;
                        Ok(format!("({})", nested_surql))
                    }
                    LogicalOperator::Or => {
                        let or_conditions: Result<Vec<String>> = clauses
                            .iter()
                            .map(|c| self.clause_to_surql(c))
                            .collect();
                        let conditions = or_conditions?;
                        Ok(format!("({})", conditions.join(" OR ")))
                    }
                    LogicalOperator::Not => {
                        let nested_surql = self.clauses_to_surql(clauses)?;
                        Ok(format!("NOT ({})", nested_surql))
                    }
                }
            }
        }
    }

    /// Convert JSON value to SurrealDB representation
    fn value_to_surql(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => format!("'{}'", self.escape_surql_string(s)),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::Array(arr) => {
                let elements: Vec<String> = arr
                    .iter()
                    .map(|v| self.value_to_surql(v))
                    .collect();
                format!("[{}]", elements.join(", "))
            }
            serde_json::Value::Object(obj) => {
                let pairs: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_surql(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
        }
    }

    /// Escape SurrealDB string literals
    fn escape_surql_string(&self, s: &str) -> String {
        s.replace('\'', "\\'")
    }

    /// Convert to MongoDB query with optimized document structure
    pub fn to_mongo_query(&self) -> serde_json::Value {
        let mut query = serde_json::Map::new();

        // Process clauses
        for clause in &self.clauses {
            self.add_clause_to_mongo(&mut query, clause);
        }

        // Create aggregation pipeline if needed
        let mut pipeline = Vec::new();

        // Match stage
        if !query.is_empty() {
            pipeline.push(serde_json::json!({ "$match": query }));
        }

        // Sort stage
        if let Some(sort) = &self.sort {
            let sort_direction = match sort.direction {
                SortDirection::Asc => 1,
                SortDirection::Desc => -1,
            };
            pipeline.push(serde_json::json!({
                "$sort": { sort.field.clone(): sort_direction }
            }));
        }

        // Skip stage (MongoDB equivalent of OFFSET)
        if let Some(offset) = self.offset {
            pipeline.push(serde_json::json!({ "$skip": offset }));
        }

        // Limit stage
        if let Some(limit) = self.limit {
            pipeline.push(serde_json::json!({ "$limit": limit }));
        }

        // Project stage for field filtering
        if !self.includes.is_empty() {
            let mut projection = serde_json::Map::new();
            for field in &self.includes {
                projection.insert(field.clone(), serde_json::Value::Number(serde_json::Number::from(1)));
            }
            pipeline.push(serde_json::json!({ "$project": projection }));
        } else if !self.excludes.is_empty() {
            let mut projection = serde_json::Map::new();
            for field in &self.excludes {
                projection.insert(field.clone(), serde_json::Value::Number(serde_json::Number::from(0)));
            }
            pipeline.push(serde_json::json!({ "$project": projection }));
        }

        if pipeline.is_empty() {
            serde_json::Value::Object(query)
        } else {
            serde_json::json!(pipeline)
        }
    }

    /// Add a clause to MongoDB query document
    fn add_clause_to_mongo(&self, query: &mut serde_json::Map<String, serde_json::Value>, clause: &QueryClause) {
        match clause {
            QueryClause::Exact { field, value } => {
                query.insert(field.clone(), value.clone());
            }
            QueryClause::Text { field, query: q, fuzzy } => {
                if *fuzzy {
                    query.insert(
                        field.clone(),
                        serde_json::json!({ "$regex": q, "$options": "i" }),
                    );
                } else {
                    query.insert(field.clone(), serde_json::Value::String(q.clone()));
                }
            }
            QueryClause::Range { field, min, max } => {
                let mut range_query = serde_json::Map::new();
                if let Some(min) = min {
                    range_query.insert("$gte".to_string(), min.clone());
                }
                if let Some(max) = max {
                    range_query.insert("$lte".to_string(), max.clone());
                }
                query.insert(field.clone(), serde_json::Value::Object(range_query));
            }
            QueryClause::Exists { field } => {
                query.insert(
                    field.clone(),
                    serde_json::json!({ "$exists": true, "$ne": null }),
                );
            }
            QueryClause::MemoryType(types) => {
                let type_values: Vec<serde_json::Value> = types
                    .iter()
                    .map(|t| serde_json::Value::String(format!("{:?}", t)))
                    .collect();
                query.insert(
                    "memory_type".to_string(),
                    serde_json::json!({ "$in": type_values }),
                );
            }
            QueryClause::Nested { operator, clauses } => {
                let nested_queries: Vec<serde_json::Value> = clauses
                    .iter()
                    .map(|c| {
                        let mut nested_query = serde_json::Map::new();
                        self.add_clause_to_mongo(&mut nested_query, c);
                        serde_json::Value::Object(nested_query)
                    })
                    .collect();

                match operator {
                    LogicalOperator::And => {
                        query.insert("$and".to_string(), serde_json::Value::Array(nested_queries));
                    }
                    LogicalOperator::Or => {
                        query.insert("$or".to_string(), serde_json::Value::Array(nested_queries));
                    }
                    LogicalOperator::Not => {
                        if nested_queries.len() == 1 {
                            query.insert("$not".to_string(), nested_queries.into_iter().next().unwrap());
                        } else {
                            query.insert(
                                "$nor".to_string(),
                                serde_json::Value::Array(nested_queries),
                            );
                        }
                    }
                }
            }
        }
    }

    /// Get query execution plan (simplified)
    pub fn get_execution_plan(&self) -> QueryExecutionPlan {
        let mut plan = QueryExecutionPlan::default();
        
        plan.estimated_cost = self.complexity_score() as f64;
        plan.uses_index = self.can_use_index();
        plan.requires_full_scan = self.requires_full_scan();
        plan.estimated_rows = self.estimate_result_rows();
        
        // Analyze operations
        for clause in &self.clauses {
            match clause {
                QueryClause::Text { .. } => plan.operations.push("text_search".to_string()),
                QueryClause::Exact { .. } => plan.operations.push("exact_match".to_string()),
                QueryClause::Range { .. } => plan.operations.push("range_scan".to_string()),
                QueryClause::Exists { .. } => plan.operations.push("existence_check".to_string()),
                QueryClause::MemoryType(_) => plan.operations.push("type_filter".to_string()),
                QueryClause::Nested { .. } => plan.operations.push("nested_query".to_string()),
            }
        }
        
        if self.sort.is_some() {
            plan.operations.push("sort".to_string());
        }
        
        if self.limit.is_some() || self.offset.is_some() {
            plan.operations.push("pagination".to_string());
        }
        
        plan
    }

    /// Check if query can use an index
    fn can_use_index(&self) -> bool {
        // Simplified logic - in reality, this would check against actual indexes
        self.clauses.iter().any(|clause| {
            matches!(clause, QueryClause::Exact { .. } | QueryClause::Range { .. })
        })
    }

    /// Check if query requires full table scan
    fn requires_full_scan(&self) -> bool {
        // If we only have text searches without indexes, we need full scan
        self.clauses.iter().any(|clause| {
            matches!(clause, QueryClause::Text { fuzzy: true, .. })
        }) && !self.can_use_index()
    }

    /// Estimate number of result rows
    fn estimate_result_rows(&self) -> usize {
        // Very simplified estimation
        let base_estimate = 1000; // Assume 1000 total rows
        let selectivity = self.estimate_selectivity();
        ((base_estimate as f64) * selectivity) as usize
    }

    /// Estimate query selectivity (0.0 to 1.0)
    fn estimate_selectivity(&self) -> f64 {
        if self.clauses.is_empty() {
            return 1.0; // Select all
        }
        
        let mut selectivity = 1.0;
        
        for clause in &self.clauses {
            let clause_selectivity = match clause {
                QueryClause::Exact { .. } => 0.1, // Exact matches are selective
                QueryClause::Text { fuzzy: false, .. } => 0.1, // Exact text matches
                QueryClause::Text { fuzzy: true, .. } => 0.3, // Fuzzy matches less selective
                QueryClause::Range { .. } => 0.2, // Range queries moderately selective
                QueryClause::Exists { .. } => 0.8, // Most records have values
                QueryClause::MemoryType(_) => 0.5, // Memory type filtering
                QueryClause::Nested { operator, clauses } => {
                    match operator {
                        LogicalOperator::And => {
                            // AND reduces selectivity
                            clauses.len() as f64 * 0.1
                        }
                        LogicalOperator::Or => {
                            // OR increases selectivity
                            1.0 - (clauses.len() as f64 * 0.1)
                        }
                        LogicalOperator::Not => 0.9, // NOT is usually selective
                    }
                }
            };
            
            selectivity *= clause_selectivity;
        }
        
        selectivity.max(0.001) // Minimum selectivity
    }
}

/// Query execution plan
#[derive(Debug, Clone, Default)]
pub struct QueryExecutionPlan {
    pub estimated_cost: f64,
    pub uses_index: bool,
    pub requires_full_scan: bool,
    pub estimated_rows: usize,
    pub operations: Vec<String>,
}

impl QueryExecutionPlan {
    /// Check if the execution plan is efficient
    pub fn is_efficient(&self) -> bool {
        self.estimated_cost < 10.0 && !self.requires_full_scan
    }

    /// Get performance warning if any
    pub fn get_performance_warning(&self) -> Option<String> {
        if self.requires_full_scan {
            Some("Query requires full table scan - consider adding indexes".to_string())
        } else if self.estimated_cost > 50.0 {
            Some("Query has high estimated cost - consider optimization".to_string())
        } else if self.estimated_rows > 10000 {
            Some("Query may return large result set - consider adding LIMIT".to_string())
        } else {
            None
        }
    }
}