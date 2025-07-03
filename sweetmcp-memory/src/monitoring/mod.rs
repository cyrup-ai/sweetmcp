//! Monitoring module for mem0-rs
//! 
//! This module provides system monitoring, health checks, metrics collection,
//! and performance tracking for the memory system.

pub mod health;
pub mod memory_usage;
pub mod metrics;
pub mod operations;
pub mod performance;

#[cfg(test)]
pub mod tests;

// Re-export main types
pub use health::*;
pub use memory_usage::*;
pub use metrics::*;
pub use operations::*;
pub use performance::*;

use prometheus::{Registry, Counter, Histogram, Gauge, HistogramVec, CounterVec, GaugeVec};
use std::sync::Arc;
use std::time::Duration;

/// Monitoring system for mem0
pub struct Monitor {
    registry: Registry,
    
    // Counters
    pub memory_operations: CounterVec,
    pub api_requests: CounterVec,
    pub errors: CounterVec,
    
    // Gauges
    pub active_connections: Gauge,
    pub memory_count: GaugeVec,
    pub cache_size: Gauge,
    
    // Histograms
    pub operation_duration: HistogramVec,
    pub query_latency: Histogram,
    pub api_latency: HistogramVec,
}

impl Monitor {
    /// Create a new monitor instance
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        // Initialize counters
        let memory_operations = CounterVec::new(
            prometheus::Opts::new("memory_operations_total", "Total memory operations"),
            &["operation", "memory_type"]
        )?;
        registry.register(Box::new(memory_operations.clone()))?;
        
        let api_requests = CounterVec::new(
            prometheus::Opts::new("api_requests_total", "Total API requests"),
            &["method", "endpoint", "status"]
        )?;
        registry.register(Box::new(api_requests.clone()))?;
        
        let errors = CounterVec::new(
            prometheus::Opts::new("errors_total", "Total errors"),
            &["error_type", "component"]
        )?;
        registry.register(Box::new(errors.clone()))?;
        
        // Initialize gauges
        let active_connections = Gauge::new(
            prometheus::Opts::new("active_connections", "Number of active connections")
        )?;
        registry.register(Box::new(active_connections.clone()))?;
        
        let memory_count = GaugeVec::new(
            prometheus::Opts::new("memory_count", "Number of memories by type"),
            &["memory_type", "user_id"]
        )?;
        registry.register(Box::new(memory_count.clone()))?;
        
        let cache_size = Gauge::new(
            prometheus::Opts::new("cache_size_bytes", "Cache size in bytes")
        )?;
        registry.register(Box::new(cache_size.clone()))?;
        
        // Initialize histograms
        let operation_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("operation_duration_seconds", "Operation duration"),
            &["operation", "memory_type"]
        )?;
        registry.register(Box::new(operation_duration.clone()))?;
        
        let query_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new("query_latency_seconds", "Query latency")
        )?;
        registry.register(Box::new(query_latency.clone()))?;
        
        let api_latency = HistogramVec::new(
            prometheus::HistogramOpts::new("api_latency_seconds", "API endpoint latency"),
            &["method", "endpoint"]
        )?;
        registry.register(Box::new(api_latency.clone()))?;
        
        Ok(Self {
            registry,
            memory_operations,
            api_requests,
            errors,
            active_connections,
            memory_count,
            cache_size,
            operation_duration,
            query_latency,
            api_latency,
        })
    }
    
    /// Get the prometheus registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    
    /// Record a memory operation
    pub fn record_memory_operation(&self, operation: &str, memory_type: &str) {
        self.memory_operations
            .with_label_values(&[operation, memory_type])
            .inc();
    }
    
    /// Record an API request
    pub fn record_api_request(&self, method: &str, endpoint: &str, status: u16) {
        self.api_requests
            .with_label_values(&[method, endpoint, &status.to_string()])
            .inc();
    }
    
    /// Record an error
    pub fn record_error(&self, error_type: &str, component: &str) {
        self.errors
            .with_label_values(&[error_type, component])
            .inc();
    }
}

impl Default for Monitor {
    fn default() -> Self {
        Self::new().expect("Failed to create monitor")
    }
}