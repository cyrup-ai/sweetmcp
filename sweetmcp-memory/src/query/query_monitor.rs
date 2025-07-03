//! Query monitoring and performance tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::query::{QueryType, QueryStats, Result};

/// Query monitor for tracking query performance
pub struct QueryMonitor {
    /// Query history
    history: Arc<RwLock<Vec<QueryRecord>>>,
    
    /// Active queries
    active: Arc<RwLock<HashMap<String, ActiveQuery>>>,
    
    /// Configuration
    config: QueryMonitorConfig,
}

/// Query monitor configuration
#[derive(Debug, Clone)]
pub struct QueryMonitorConfig {
    /// Maximum history size
    pub max_history: usize,
    
    /// Enable query logging
    pub enable_logging: bool,
    
    /// Slow query threshold in milliseconds
    pub slow_query_threshold_ms: u64,
}

impl Default for QueryMonitorConfig {
    fn default() -> Self {
        Self {
            max_history: 10000,
            enable_logging: true,
            slow_query_threshold_ms: 1000,
        }
    }
}

/// Query record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRecord {
    /// Query ID
    pub id: String,
    
    /// Query type
    pub query_type: QueryType,
    
    /// Query text/pattern
    pub query: String,
    
    /// Start time
    pub started_at: DateTime<Utc>,
    
    /// End time
    pub ended_at: DateTime<Utc>,
    
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    
    /// Query statistics
    pub stats: QueryStats,
    
    /// Was this a slow query
    pub is_slow: bool,
    
    /// Error if query failed
    pub error: Option<String>,
}

/// Active query tracking
#[derive(Debug, Clone)]
struct ActiveQuery {
    /// Query ID
    id: String,
    
    /// Query type
    query_type: QueryType,
    
    /// Query text
    query: String,
    
    /// Start time
    started_at: DateTime<Utc>,
}

impl QueryMonitor {
    /// Create a new query monitor
    pub fn new(config: QueryMonitorConfig) -> Self {
        Self {
            history: Arc::new(RwLock::new(Vec::new())),
            active: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Start monitoring a query
    pub async fn start_query(
        &self,
        id: String,
        query_type: QueryType,
        query: String,
    ) -> QueryHandle {
        let active_query = ActiveQuery {
            id: id.clone(),
            query_type,
            query: query.clone(),
            started_at: Utc::now(),
        };
        
        self.active.write().await.insert(id.clone(), active_query);
        
        QueryHandle {
            monitor: self,
            id,
            started_at: Utc::now(),
        }
    }
    
    /// Complete a query
    async fn complete_query(
        &self,
        id: String,
        stats: QueryStats,
        error: Option<String>,
    ) -> Result<()> {
        if let Some(active) = self.active.write().await.remove(&id) {
            let ended_at = Utc::now();
            let execution_time_ms = ended_at
                .signed_duration_since(active.started_at)
                .num_milliseconds() as u64;
            
            let is_slow = execution_time_ms > self.config.slow_query_threshold_ms;
            
            let record = QueryRecord {
                id: active.id,
                query_type: active.query_type,
                query: active.query,
                started_at: active.started_at,
                ended_at,
                execution_time_ms,
                stats,
                is_slow,
                error,
            };
            
            // Log slow queries
            if is_slow && self.config.enable_logging {
                tracing::warn!(
                    "Slow query detected: {} ms - {}",
                    execution_time_ms,
                    record.query
                );
            }
            
            // Add to history
            let mut history = self.history.write().await;
            history.push(record);
            
            // Trim history if needed
            if history.len() > self.config.max_history {
                history.drain(0..history.len() - self.config.max_history);
            }
        }
        
        Ok(())
    }
    
    /// Get query history
    pub async fn get_history(&self) -> Vec<QueryRecord> {
        self.history.read().await.clone()
    }
    
    /// Get slow queries
    pub async fn get_slow_queries(&self) -> Vec<QueryRecord> {
        self.history
            .read()
            .await
            .iter()
            .filter(|r| r.is_slow)
            .cloned()
            .collect()
    }
    
    /// Get active queries
    pub async fn get_active_queries(&self) -> Vec<ActiveQuery> {
        self.active.read().await.values().cloned().collect()
    }
    
    /// Get query statistics summary
    pub async fn get_summary(&self) -> QuerySummary {
        let history = self.history.read().await;
        
        let total_queries = history.len();
        let failed_queries = history.iter().filter(|r| r.error.is_some()).count();
        let slow_queries = history.iter().filter(|r| r.is_slow).count();
        
        let avg_execution_time = if total_queries > 0 {
            history.iter().map(|r| r.execution_time_ms).sum::<u64>() / total_queries as u64
        } else {
            0
        };
        
        let queries_by_type = history
            .iter()
            .fold(HashMap::new(), |mut acc, record| {
                *acc.entry(record.query_type).or_insert(0) += 1;
                acc
            });
        
        QuerySummary {
            total_queries,
            failed_queries,
            slow_queries,
            avg_execution_time_ms: avg_execution_time,
            queries_by_type,
        }
    }
}

/// Query handle for tracking query completion
pub struct QueryHandle<'a> {
    monitor: &'a QueryMonitor,
    id: String,
    started_at: DateTime<Utc>,
}

impl<'a> QueryHandle<'a> {
    /// Complete the query successfully
    pub async fn complete(self, stats: QueryStats) {
        self.monitor
            .complete_query(self.id, stats, None)
            .await
            .ok();
    }
    
    /// Complete the query with an error
    pub async fn fail(self, error: String) {
        let stats = QueryStats {
            execution_time_ms: 0,
            documents_scanned: 0,
            documents_returned: 0,
            index_used: false,
            cache_hit_rate: 0.0,
        };
        
        self.monitor
            .complete_query(self.id, stats, Some(error))
            .await
            .ok();
    }
}

/// Query summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySummary {
    /// Total number of queries
    pub total_queries: usize,
    
    /// Number of failed queries
    pub failed_queries: usize,
    
    /// Number of slow queries
    pub slow_queries: usize,
    
    /// Average execution time
    pub avg_execution_time_ms: u64,
    
    /// Queries by type
    pub queries_by_type: HashMap<QueryType, usize>,
}