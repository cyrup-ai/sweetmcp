//! Committee performance metrics
//!
//! This module provides performance tracking and metrics for committee operations
//! with zero allocation patterns and efficient monitoring capabilities.

/// Performance metrics for committee operations
/// 
/// Tracks various performance and operational metrics for monitoring
/// and optimization of committee-based evaluations.
#[derive(Debug, Clone)]
pub struct CommitteeMetrics {
    /// Total number of evaluations performed
    total_evaluations: usize,
    /// Total number of consensus decisions reached
    total_decisions: usize,
    /// Total time spent on evaluations (in milliseconds)
    total_evaluation_time_ms: u64,
    /// Number of cache hits for optimization specs
    cache_hits: usize,
    /// Number of cache misses for optimization specs
    cache_misses: usize,
    /// Number of timeouts encountered
    timeout_count: usize,
    /// Number of successful parallel executions
    parallel_successes: usize,
    /// Number of failed parallel executions
    parallel_failures: usize,
}

impl CommitteeMetrics {
    /// Create new committee metrics
    /// 
    /// # Returns
    /// New CommitteeMetrics with zero values
    pub fn new() -> Self {
        Self {
            total_evaluations: 0,
            total_decisions: 0,
            total_evaluation_time_ms: 0,
            cache_hits: 0,
            cache_misses: 0,
            timeout_count: 0,
            parallel_successes: 0,
            parallel_failures: 0,
        }
    }

    /// Increment evaluation count
    pub fn increment_evaluations(&mut self) {
        self.total_evaluations += 1;
    }

    /// Increment decision count
    pub fn increment_decisions(&mut self) {
        self.total_decisions += 1;
    }

    /// Add evaluation time
    /// 
    /// # Arguments
    /// * `time_ms` - Time in milliseconds to add
    pub fn add_evaluation_time(&mut self, time_ms: u64) {
        self.total_evaluation_time_ms += time_ms;
    }

    /// Record cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Record cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    /// Record timeout
    pub fn record_timeout(&mut self) {
        self.timeout_count += 1;
    }

    /// Record parallel execution success
    pub fn record_parallel_success(&mut self) {
        self.parallel_successes += 1;
    }

    /// Record parallel execution failure
    pub fn record_parallel_failure(&mut self) {
        self.parallel_failures += 1;
    }

    /// Get total evaluations count
    /// 
    /// # Returns
    /// Total number of evaluations performed
    pub fn total_evaluations(&self) -> usize {
        self.total_evaluations
    }

    /// Get total decisions count
    /// 
    /// # Returns
    /// Total number of consensus decisions reached
    pub fn total_decisions(&self) -> usize {
        self.total_decisions
    }

    /// Get total evaluation time
    /// 
    /// # Returns
    /// Total evaluation time in milliseconds
    pub fn total_evaluation_time_ms(&self) -> u64 {
        self.total_evaluation_time_ms
    }

    /// Get timeout count
    /// 
    /// # Returns
    /// Number of timeouts encountered
    pub fn timeout_count(&self) -> usize {
        self.timeout_count
    }

    /// Calculate cache hit rate
    /// 
    /// # Returns
    /// Cache hit rate as a percentage (0.0 to 1.0)
    pub fn cache_hit_rate(&self) -> f64 {
        let total_cache_requests = self.cache_hits + self.cache_misses;
        if total_cache_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_cache_requests as f64
        }
    }

    /// Calculate average evaluation time
    /// 
    /// # Returns
    /// Average evaluation time in milliseconds
    pub fn average_evaluation_time_ms(&self) -> f64 {
        if self.total_evaluations == 0 {
            0.0
        } else {
            self.total_evaluation_time_ms as f64 / self.total_evaluations as f64
        }
    }

    /// Calculate parallel execution success rate
    /// 
    /// # Returns
    /// Success rate as a percentage (0.0 to 1.0)
    pub fn parallel_success_rate(&self) -> f64 {
        let total_parallel = self.parallel_successes + self.parallel_failures;
        if total_parallel == 0 {
            0.0
        } else {
            self.parallel_successes as f64 / total_parallel as f64
        }
    }

    /// Calculate timeout rate
    /// 
    /// # Returns
    /// Timeout rate as a percentage (0.0 to 1.0)
    pub fn timeout_rate(&self) -> f64 {
        if self.total_evaluations == 0 {
            0.0
        } else {
            self.timeout_count as f64 / self.total_evaluations as f64
        }
    }

    /// Calculate decisions per evaluation ratio
    /// 
    /// # Returns
    /// Ratio of decisions to evaluations
    pub fn decision_efficiency(&self) -> f64 {
        if self.total_evaluations == 0 {
            0.0
        } else {
            self.total_decisions as f64 / self.total_evaluations as f64
        }
    }

    /// Get detailed metrics breakdown
    /// 
    /// # Returns
    /// Detailed metrics as key-value pairs
    pub fn detailed_breakdown(&self) -> Vec<(String, String)> {
        vec![
            ("Total Evaluations".to_string(), self.total_evaluations.to_string()),
            ("Total Decisions".to_string(), self.total_decisions.to_string()),
            ("Total Time (ms)".to_string(), self.total_evaluation_time_ms.to_string()),
            ("Average Time (ms)".to_string(), format!("{:.2}", self.average_evaluation_time_ms())),
            ("Cache Hit Rate".to_string(), format!("{:.1}%", self.cache_hit_rate() * 100.0)),
            ("Parallel Success Rate".to_string(), format!("{:.1}%", self.parallel_success_rate() * 100.0)),
            ("Timeout Rate".to_string(), format!("{:.1}%", self.timeout_rate() * 100.0)),
            ("Decision Efficiency".to_string(), format!("{:.2}", self.decision_efficiency())),
            ("Cache Hits".to_string(), self.cache_hits.to_string()),
            ("Cache Misses".to_string(), self.cache_misses.to_string()),
            ("Timeouts".to_string(), self.timeout_count.to_string()),
            ("Parallel Successes".to_string(), self.parallel_successes.to_string()),
            ("Parallel Failures".to_string(), self.parallel_failures.to_string()),
        ]
    }

    /// Reset all metrics to zero
    /// 
    /// Clears all accumulated metrics for a fresh start.
    pub fn reset(&mut self) {
        self.total_evaluations = 0;
        self.total_decisions = 0;
        self.total_evaluation_time_ms = 0;
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.timeout_count = 0;
        self.parallel_successes = 0;
        self.parallel_failures = 0;
    }

    /// Merge metrics from another instance
    /// 
    /// # Arguments
    /// * `other` - Other metrics instance to merge
    pub fn merge(&mut self, other: &CommitteeMetrics) {
        self.total_evaluations += other.total_evaluations;
        self.total_decisions += other.total_decisions;
        self.total_evaluation_time_ms += other.total_evaluation_time_ms;
        self.cache_hits += other.cache_hits;
        self.cache_misses += other.cache_misses;
        self.timeout_count += other.timeout_count;
        self.parallel_successes += other.parallel_successes;
        self.parallel_failures += other.parallel_failures;
    }

    /// Get metrics summary
    /// 
    /// # Returns
    /// Human-readable summary of metrics
    pub fn summary(&self) -> String {
        format!(
            "Evaluations: {}, Decisions: {}, Avg Time: {:.1}ms, Cache Hit: {:.1}%, Parallel Success: {:.1}%",
            self.total_evaluations,
            self.total_decisions,
            self.average_evaluation_time_ms(),
            self.cache_hit_rate() * 100.0,
            self.parallel_success_rate() * 100.0
        )
    }

    /// Get compact summary for logging
    /// 
    /// # Returns
    /// Compact summary suitable for log messages
    pub fn compact_summary(&self) -> String {
        format!(
            "E:{} D:{} T:{:.0}ms CH:{:.0}% PS:{:.0}%",
            self.total_evaluations,
            self.total_decisions,
            self.average_evaluation_time_ms(),
            self.cache_hit_rate() * 100.0,
            self.parallel_success_rate() * 100.0
        )
    }
}

impl Default for CommitteeMetrics {
    fn default() -> Self {
        Self::new()
    }
}