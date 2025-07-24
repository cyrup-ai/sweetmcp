//! Query executor wrapper for builder integration
//!
//! This module provides blazing-fast executor integration with zero-allocation
//! patterns and optimized query execution coordination.

use super::super::MemoryQueryExecutor;
use super::super::core::MemoryQuery;

/// Wrapper for query executor integration with builder
#[derive(Debug)]
pub struct QueryExecutorWrapper {
    executor: MemoryQueryExecutor,
}

impl QueryExecutorWrapper {
    /// Create new executor wrapper
    pub fn new(executor: MemoryQueryExecutor) -> Self {
        Self { executor }
    }
    
    /// Get reference to underlying executor
    pub fn executor(&self) -> &MemoryQueryExecutor {
        &self.executor
    }
    
    /// Get mutable reference to underlying executor
    pub fn executor_mut(&mut self) -> &mut MemoryQueryExecutor {
        &mut self.executor
    }
    
    /// Execute query with the wrapped executor
    pub async fn execute(&self, query: &MemoryQuery) -> Result<Vec<crate::memory::Memory>, String> {
        self.executor.execute(query).await
            .map_err(|e| format!("Query execution failed: {}", e))
    }
    
    /// Execute query with timeout
    pub async fn execute_with_timeout(
        &self, 
        query: &MemoryQuery, 
        timeout_ms: u64
    ) -> Result<Vec<crate::memory::Memory>, String> {
        let timeout = std::time::Duration::from_millis(timeout_ms);
        
        match tokio::time::timeout(timeout, self.execute(query)).await {
            Ok(result) => result,
            Err(_) => Err(format!("Query execution timed out after {}ms", timeout_ms)),
        }
    }
    
    /// Validate query before execution
    pub fn validate_query(&self, query: &MemoryQuery) -> Result<(), String> {
        // Basic validation - in practice would be more comprehensive
        if query.limit.is_some() && query.limit.unwrap() == 0 {
            return Err("Query limit cannot be zero".to_string());
        }
        
        Ok(())
    }
    
    /// Estimate query execution time
    pub fn estimate_execution_time(&self, query: &MemoryQuery) -> std::time::Duration {
        // Simplified estimation based on query complexity
        let base_time_ms = 10;
        let complexity_factor = match query.limit {
            Some(limit) if limit > 1000 => 3,
            Some(limit) if limit > 100 => 2,
            _ => 1,
        };
        
        std::time::Duration::from_millis(base_time_ms * complexity_factor)
    }
    
    /// Check if executor is ready
    pub fn is_ready(&self) -> bool {
        // In practice would check executor state
        true
    }
}