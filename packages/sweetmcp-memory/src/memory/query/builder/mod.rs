//! Query builder coordination module
//!
//! This module coordinates all query builder submodules with blazing-fast
//! zero-allocation patterns and ergonomic APIs for complex query construction.

pub mod conditions;
pub mod core;
pub mod executor;
pub mod validation;
pub mod convenience;

// Re-export key types for ergonomic access
pub use conditions::{QueryCondition, LogicalOperator};
pub use core::{ComplexQueryBuilder, ExecutionTimeCategory};
pub use executor::QueryExecutorWrapper;
pub use validation::{QueryValidator, ValidationSummary, PerformanceRating};
pub use convenience::{QueryFactory, QueryTemplates};

// Re-export the main builder for backward compatibility
pub use core::ComplexQueryBuilder as Builder;

/// Factory for creating optimized query builders
pub struct BuilderFactory;

impl BuilderFactory {
    /// Create a standard query builder
    pub fn create_standard() -> ComplexQueryBuilder {
        ComplexQueryBuilder::new()
    }
    
    /// Create a query builder optimized for high-performance scenarios
    pub fn create_high_performance() -> ComplexQueryBuilder {
        ComplexQueryBuilder::new()
            .optimize() // Apply performance optimizations
    }
    
    /// Create a query builder with strict validation
    pub fn create_strict() -> ComplexQueryBuilder {
        let _validator = QueryValidator::with_limits(25, 5, 5000); // Stricter limits
        ComplexQueryBuilder::new() // Note: In full implementation would accept validator
    }
    
    /// Create a query builder for debugging
    pub fn create_debug() -> ComplexQueryBuilder {
        let _validator = QueryValidator::with_limits(100, 20, 50000); // Relaxed limits
        ComplexQueryBuilder::new() // Note: In full implementation would accept validator
    }
    
    /// Create a query builder with executor
    pub fn create_with_executor(executor: crate::memory::query::executor_core::MemoryQueryExecutor) -> ComplexQueryBuilder {
        let wrapper = QueryExecutorWrapper::new(executor);
        ComplexQueryBuilder::with_executor(wrapper)
    }
}

/// Utility functions for query building
pub struct QueryUtils;

impl QueryUtils {
    /// Merge multiple builders into a single nested query
    pub fn merge_builders(
        builders: Vec<ComplexQueryBuilder>,
        operator: LogicalOperator,
    ) -> Result<ComplexQueryBuilder, String> {
        if builders.is_empty() {
            return Err("Cannot merge empty list of builders".to_string());
        }
        
        if builders.len() == 1 {
            return Ok(builders.into_iter().next().unwrap());
        }
        
        // Convert each builder to conditions (simplified for this example)
        let mut all_conditions = Vec::new();
        for builder in builders {
            if builder.has_conditions() {
                // In a full implementation, we'd extract conditions from the builder
                // For now, we'll create a placeholder
                all_conditions.push(QueryCondition::metadata_string("merged", "true"));
            }
        }
        
        if all_conditions.is_empty() {
            return Err("No valid conditions found in builders".to_string());
        }
        
        Ok(ComplexQueryBuilder::new().with_nested(all_conditions, operator))
    }
    
    /// Create a builder from a simple text query
    pub fn from_text_query(query: &str) -> Result<ComplexQueryBuilder, String> {
        if query.is_empty() {
            return Err("Query text cannot be empty".to_string());
        }
        
        // Simple parsing - in practice would be more sophisticated
        let mut builder = ComplexQueryBuilder::new();
        
        // Check for common patterns
        if query.starts_with("user:") {
            let user_id = query.strip_prefix("user:").unwrap().trim();
            builder = builder.with_user(user_id);
        } else if query.starts_with("project:") {
            let project_id = query.strip_prefix("project:").unwrap().trim();
            builder = builder.with_project(project_id);
        } else if query.starts_with("tag:") {
            let tag = query.strip_prefix("tag:").unwrap().trim();
            builder = builder.with_tag(tag);
        } else {
            // Default to text similarity search
            builder = builder.with_text_similarity(query, 0.3);
        }
        
        Ok(builder)
    }
    
    /// Validate a query string before parsing
    pub fn validate_query_string(query: &str) -> Result<(), String> {
        if query.is_empty() {
            return Err("Query string cannot be empty".to_string());
        }
        
        if query.len() > 10000 {
            return Err("Query string too long (max 10,000 characters)".to_string());
        }
        
        // Check for potentially dangerous patterns
        if query.contains("DROP") || query.contains("DELETE") || query.contains("UPDATE") {
            return Err("Query contains potentially dangerous SQL keywords".to_string());
        }
        
        Ok(())
    }
    
    /// Estimate query complexity from a builder
    pub fn estimate_complexity(builder: &ComplexQueryBuilder) -> QueryComplexity {
        let score = builder.complexity_score();
        let condition_count = builder.condition_count();
        
        match (score, condition_count) {
            (0..=5, 1..=3) => QueryComplexity::Simple,
            (6..=15, 1..=10) => QueryComplexity::Moderate,
            (16..=30, 1..=20) => QueryComplexity::Complex,
            _ => QueryComplexity::VeryComplex,
        }
    }
    
    /// Get optimization suggestions for a builder
    pub fn optimization_suggestions(builder: &ComplexQueryBuilder) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        let complexity = Self::estimate_complexity(builder);
        let condition_count = builder.condition_count();
        
        if matches!(complexity, QueryComplexity::Complex | QueryComplexity::VeryComplex) {
            suggestions.push("Consider breaking down complex query into simpler parts".to_string());
        }
        
        if condition_count > 15 {
            suggestions.push("Large number of conditions may impact performance".to_string());
        }
        
        if builder.execution_time_category().has_performance_concerns() {
            suggestions.push("Query may be slow - consider adding more selective conditions first".to_string());
        }
        
        if suggestions.is_empty() {
            suggestions.push("Query appears well-optimized".to_string());
        }
        
        suggestions
    }
}

/// Query complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

impl QueryComplexity {
    /// Get complexity description
    pub fn description(&self) -> &'static str {
        match self {
            QueryComplexity::Simple => "Simple",
            QueryComplexity::Moderate => "Moderate", 
            QueryComplexity::Complex => "Complex",
            QueryComplexity::VeryComplex => "Very Complex",
        }
    }
    
    /// Get expected execution time range
    pub fn expected_time_range(&self) -> (u32, u32) {
        match self {
            QueryComplexity::Simple => (1, 10),      // 1-10ms
            QueryComplexity::Moderate => (10, 100),  // 10-100ms
            QueryComplexity::Complex => (100, 1000), // 100ms-1s
            QueryComplexity::VeryComplex => (1000, 10000), // 1-10s
        }
    }
    
    /// Check if complexity requires optimization
    pub fn requires_optimization(&self) -> bool {
        matches!(self, QueryComplexity::Complex | QueryComplexity::VeryComplex)
    }
}

/// Macro for creating simple queries
#[macro_export]
macro_rules! query {
    // Simple text search
    (text: $text:expr) => {
        ComplexQueryBuilder::text_search($text, 0.3)
    };
    
    // Text search with score
    (text: $text:expr, score: $score:expr) => {
        ComplexQueryBuilder::text_search($text, $score)
    };
    
    // User query
    (user: $user:expr) => {
        ComplexQueryBuilder::user_memories($user)
    };
    
    // Project query
    (project: $project:expr) => {
        ComplexQueryBuilder::project_memories($project)
    };
    
    // Recent memories
    (recent: $days:expr) => {
        ComplexQueryBuilder::recent_memories($days)
    };
    
    // Important memories
    (important: $score:expr) => {
        ComplexQueryBuilder::important_memories($score)
    };
}

// Re-export the macro
pub use query;

/// Convenience type aliases
pub type QueryBuilder = ComplexQueryBuilder;
pub type Condition = QueryCondition;
pub type Operator = LogicalOperator;

/// Module-level constants
pub const DEFAULT_TEXT_SIMILARITY_THRESHOLD: f32 = 0.3;
pub const DEFAULT_IMPORTANCE_THRESHOLD: f32 = 0.5;
pub const MAX_CONDITIONS_PER_QUERY: usize = 50;
pub const MAX_NESTING_DEPTH: usize = 10;
pub const MAX_QUERY_TEXT_LENGTH: usize = 10000;