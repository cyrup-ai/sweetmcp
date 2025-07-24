//! Optimization results and performance tracking
//!
//! This module provides blazing-fast result tracking with zero allocation
//! optimizations and elegant ergonomic interfaces for optimization outcomes.

use std::time::Duration;

use super::super::optimization_recommendations::RecommendationType;

/// Single optimization result
#[derive(Debug, Clone)]
pub struct SingleOptimizationResult {
    /// Type of optimization performed
    pub optimization_type: RecommendationType,
    /// Whether the optimization was successful
    pub success: bool,
    /// Improvement achieved (percentage)
    pub improvement_achieved: f64,
    /// Execution time for this optimization
    pub execution_time: Duration,
    /// Memory saved in bytes
    pub memory_saved: usize,
    /// Number of items processed
    pub items_processed: usize,
    /// Error message if optimization failed
    pub error_message: Option<String>,
}

impl SingleOptimizationResult {
    /// Create new successful optimization result
    #[inline]
    pub fn success(
        optimization_type: RecommendationType,
        improvement_achieved: f64,
        execution_time: Duration,
        memory_saved: usize,
        items_processed: usize,
    ) -> Self {
        Self {
            optimization_type,
            success: true,
            improvement_achieved,
            execution_time,
            memory_saved,
            items_processed,
            error_message: None,
        }
    }

    /// Create new failed optimization result
    #[inline]
    pub fn failure(
        optimization_type: RecommendationType,
        execution_time: Duration,
        error_message: String,
    ) -> Self {
        Self {
            optimization_type,
            success: false,
            improvement_achieved: 0.0,
            execution_time,
            memory_saved: 0,
            items_processed: 0,
            error_message: Some(error_message),
        }
    }

    /// Create new partial success result
    #[inline]
    pub fn partial_success(
        optimization_type: RecommendationType,
        improvement_achieved: f64,
        execution_time: Duration,
        memory_saved: usize,
        items_processed: usize,
        warning_message: String,
    ) -> Self {
        Self {
            optimization_type,
            success: true,
            improvement_achieved,
            execution_time,
            memory_saved,
            items_processed,
            error_message: Some(warning_message),
        }
    }

    /// Check if result indicates significant improvement
    #[inline]
    pub fn is_significant_improvement(&self) -> bool {
        self.success && self.improvement_achieved >= 5.0
    }

    /// Check if result indicates minor improvement
    #[inline]
    pub fn is_minor_improvement(&self) -> bool {
        self.success && self.improvement_achieved > 0.0 && self.improvement_achieved < 5.0
    }

    /// Check if result indicates no improvement
    #[inline]
    pub fn is_no_improvement(&self) -> bool {
        self.success && self.improvement_achieved <= 0.0
    }

    /// Get efficiency score (improvement per second)
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        if self.execution_time.as_secs_f64() > 0.0 && self.success {
            self.improvement_achieved / self.execution_time.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Get memory efficiency (memory saved per second)
    #[inline]
    pub fn memory_efficiency(&self) -> f64 {
        if self.execution_time.as_secs_f64() > 0.0 && self.success {
            self.memory_saved as f64 / self.execution_time.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Get processing rate (items per second)
    #[inline]
    pub fn processing_rate(&self) -> f64 {
        if self.execution_time.as_secs_f64() > 0.0 {
            self.items_processed as f64 / self.execution_time.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Get result quality score (0.0-1.0)
    #[inline]
    pub fn quality_score(&self) -> f64 {
        if !self.success {
            return 0.0;
        }

        let improvement_score = (self.improvement_achieved / 20.0).min(1.0);
        let efficiency_score = (self.efficiency_score() / 10.0).min(1.0);
        let processing_score = if self.items_processed > 0 { 1.0 } else { 0.5 };
        let memory_score = if self.memory_saved > 0 { 1.0 } else { 0.5 };

        (improvement_score + efficiency_score + processing_score + memory_score) / 4.0
    }

    /// Check if result meets performance threshold
    #[inline]
    pub fn meets_threshold(&self, min_improvement: f64) -> bool {
        self.success && self.improvement_achieved >= min_improvement
    }

    /// Get result summary description
    #[inline]
    pub fn summary(&self) -> String {
        if self.success {
            format!(
                "{:?}: {:.1}% improvement, {} items, {:.1}MB saved in {:?}",
                self.optimization_type,
                self.improvement_achieved,
                self.items_processed,
                self.memory_saved as f64 / (1024.0 * 1024.0),
                self.execution_time
            )
        } else {
            format!(
                "{:?}: Failed after {:?} - {}",
                self.optimization_type,
                self.execution_time,
                self.error_message.as_deref().unwrap_or("Unknown error")
            )
        }
    }

    /// Get detailed result information
    #[inline]
    pub fn detailed_info(&self) -> DetailedResultInfo {
        DetailedResultInfo {
            optimization_type: format!("{:?}", self.optimization_type),
            success: self.success,
            improvement_achieved: self.improvement_achieved,
            execution_time_ms: self.execution_time.as_millis() as u64,
            memory_saved_mb: self.memory_saved as f64 / (1024.0 * 1024.0),
            items_processed: self.items_processed,
            efficiency_score: self.efficiency_score(),
            memory_efficiency: self.memory_efficiency(),
            processing_rate: self.processing_rate(),
            quality_score: self.quality_score(),
            error_message: self.error_message.clone(),
        }
    }

    /// Check if result should be cached
    #[inline]
    pub fn should_cache(&self) -> bool {
        self.success && (self.improvement_achieved > 1.0 || self.memory_saved > 1024 * 1024)
    }

    /// Get cache priority (higher is better for caching)
    #[inline]
    pub fn cache_priority(&self) -> f64 {
        if !self.success {
            return 0.0;
        }

        let improvement_priority = self.improvement_achieved / 10.0;
        let memory_priority = (self.memory_saved as f64) / (1024.0 * 1024.0 * 10.0); // Per 10MB
        let efficiency_priority = self.efficiency_score() / 5.0;

        (improvement_priority + memory_priority + efficiency_priority).min(10.0)
    }
}

/// Overall optimization execution result
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub individual_results: Vec<SingleOptimizationResult>,
    pub total_improvement: f64,
    pub total_execution_time: Duration,
    pub efficiency_score: f64,
    pub success_rate: f64,
    pub total_memory_saved: usize,
    pub total_items_processed: usize,
}

impl OptimizationResult {
    /// Create new optimization result
    #[inline]
    pub fn new(
        individual_results: Vec<SingleOptimizationResult>,
        total_improvement: f64,
        total_execution_time: Duration,
        efficiency_score: f64,
    ) -> Self {
        let success_count = individual_results.iter().filter(|r| r.success).count();
        let success_rate = if individual_results.is_empty() {
            0.0
        } else {
            success_count as f64 / individual_results.len() as f64
        };

        let total_memory_saved = individual_results.iter()
            .map(|r| r.memory_saved)
            .sum();

        let total_items_processed = individual_results.iter()
            .map(|r| r.items_processed)
            .sum();

        Self {
            individual_results,
            total_improvement,
            total_execution_time,
            efficiency_score,
            success_rate,
            total_memory_saved,
            total_items_processed,
        }
    }

    /// Create from individual results
    #[inline]
    pub fn from_results(individual_results: Vec<SingleOptimizationResult>) -> Self {
        let total_improvement = individual_results.iter()
            .filter(|r| r.success)
            .map(|r| r.improvement_achieved)
            .sum();

        let total_execution_time = individual_results.iter()
            .map(|r| r.execution_time)
            .sum();

        let efficiency_score = if total_execution_time.as_secs_f64() > 0.0 {
            total_improvement / total_execution_time.as_secs_f64()
        } else {
            0.0
        };

        Self::new(individual_results, total_improvement, total_execution_time, efficiency_score)
    }

    /// Check if optimization was successful overall
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.success_rate >= 0.5 && self.total_improvement > 0.0
    }

    /// Check if optimization was highly successful
    #[inline]
    pub fn is_highly_successful(&self) -> bool {
        self.success_rate >= 0.8 && self.total_improvement >= 10.0
    }

    /// Get summary description
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "Optimization completed: {:.1}% improvement, {:.1}% success rate, {} operations in {:?}",
            self.total_improvement,
            self.success_rate * 100.0,
            self.individual_results.len(),
            self.total_execution_time
        )
    }

    /// Get detailed summary
    #[inline]
    pub fn detailed_summary(&self) -> String {
        format!(
            "Optimization Results:\n\
             - Total Improvement: {:.1}%\n\
             - Success Rate: {:.1}%\n\
             - Operations: {}\n\
             - Execution Time: {:?}\n\
             - Memory Saved: {:.1}MB\n\
             - Items Processed: {}\n\
             - Efficiency Score: {:.2}",
            self.total_improvement,
            self.success_rate * 100.0,
            self.individual_results.len(),
            self.total_execution_time,
            self.total_memory_saved as f64 / (1024.0 * 1024.0),
            self.total_items_processed,
            self.efficiency_score
        )
    }

    /// Get successful results only
    #[inline]
    pub fn successful_results(&self) -> Vec<&SingleOptimizationResult> {
        self.individual_results.iter().filter(|r| r.success).collect()
    }

    /// Get failed results only
    #[inline]
    pub fn failed_results(&self) -> Vec<&SingleOptimizationResult> {
        self.individual_results.iter().filter(|r| !r.success).collect()
    }

    /// Get results by optimization type
    #[inline]
    pub fn results_by_type(&self, optimization_type: &RecommendationType) -> Vec<&SingleOptimizationResult> {
        self.individual_results.iter()
            .filter(|r| &r.optimization_type == optimization_type)
            .collect()
    }

    /// Get best performing result
    #[inline]
    pub fn best_result(&self) -> Option<&SingleOptimizationResult> {
        self.individual_results.iter()
            .filter(|r| r.success)
            .max_by(|a, b| a.improvement_achieved.partial_cmp(&b.improvement_achieved).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Get worst performing result
    #[inline]
    pub fn worst_result(&self) -> Option<&SingleOptimizationResult> {
        self.individual_results.iter()
            .filter(|r| r.success)
            .min_by(|a, b| a.improvement_achieved.partial_cmp(&b.improvement_achieved).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Get average improvement per successful operation
    #[inline]
    pub fn average_improvement(&self) -> f64 {
        let successful = self.successful_results();
        if successful.is_empty() {
            return 0.0;
        }

        successful.iter().map(|r| r.improvement_achieved).sum::<f64>() / successful.len() as f64
    }

    /// Get average execution time per operation
    #[inline]
    pub fn average_execution_time(&self) -> Duration {
        if self.individual_results.is_empty() {
            return Duration::from_secs(0);
        }

        self.total_execution_time / self.individual_results.len() as u32
    }

    /// Get throughput (operations per second)
    #[inline]
    pub fn throughput(&self) -> f64 {
        if self.total_execution_time.as_secs_f64() == 0.0 || self.individual_results.is_empty() {
            return 0.0;
        }

        self.individual_results.len() as f64 / self.total_execution_time.as_secs_f64()
    }

    /// Get memory efficiency (MB saved per second)
    #[inline]
    pub fn memory_efficiency(&self) -> f64 {
        if self.total_execution_time.as_secs_f64() == 0.0 {
            return 0.0;
        }

        (self.total_memory_saved as f64 / (1024.0 * 1024.0)) / self.total_execution_time.as_secs_f64()
    }

    /// Get processing efficiency (items per second)
    #[inline]
    pub fn processing_efficiency(&self) -> f64 {
        if self.total_execution_time.as_secs_f64() == 0.0 {
            return 0.0;
        }

        self.total_items_processed as f64 / self.total_execution_time.as_secs_f64()
    }

    /// Get overall quality score (0.0-1.0)
    #[inline]
    pub fn quality_score(&self) -> f64 {
        let success_score = self.success_rate;
        let improvement_score = (self.total_improvement / 50.0).min(1.0);
        let efficiency_score = (self.efficiency_score / 10.0).min(1.0);
        let throughput_score = (self.throughput() / 5.0).min(1.0);

        (success_score + improvement_score + efficiency_score + throughput_score) / 4.0
    }

    /// Check if result meets quality threshold
    #[inline]
    pub fn meets_quality_threshold(&self, threshold: f64) -> bool {
        self.quality_score() >= threshold
    }

    /// Get performance metrics
    #[inline]
    pub fn performance_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            total_improvement: self.total_improvement,
            success_rate: self.success_rate,
            efficiency_score: self.efficiency_score,
            throughput: self.throughput(),
            memory_efficiency: self.memory_efficiency(),
            processing_efficiency: self.processing_efficiency(),
            quality_score: self.quality_score(),
            average_improvement: self.average_improvement(),
            average_execution_time: self.average_execution_time(),
        }
    }
}

/// Detailed result information
#[derive(Debug, Clone)]
pub struct DetailedResultInfo {
    pub optimization_type: String,
    pub success: bool,
    pub improvement_achieved: f64,
    pub execution_time_ms: u64,
    pub memory_saved_mb: f64,
    pub items_processed: usize,
    pub efficiency_score: f64,
    pub memory_efficiency: f64,
    pub processing_rate: f64,
    pub quality_score: f64,
    pub error_message: Option<String>,
}

/// Performance metrics summary
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_improvement: f64,
    pub success_rate: f64,
    pub efficiency_score: f64,
    pub throughput: f64,
    pub memory_efficiency: f64,
    pub processing_efficiency: f64,
    pub quality_score: f64,
    pub average_improvement: f64,
    pub average_execution_time: Duration,
}

impl PerformanceMetrics {
    /// Check if metrics indicate excellent performance
    #[inline]
    pub fn is_excellent(&self) -> bool {
        self.success_rate > 0.9 &&
        self.total_improvement > 20.0 &&
        self.efficiency_score > 5.0 &&
        self.quality_score > 0.8
    }

    /// Check if metrics indicate good performance
    #[inline]
    pub fn is_good(&self) -> bool {
        self.success_rate > 0.7 &&
        self.total_improvement > 5.0 &&
        self.efficiency_score > 1.0 &&
        self.quality_score > 0.6
    }

    /// Check if metrics indicate poor performance
    #[inline]
    pub fn is_poor(&self) -> bool {
        self.success_rate < 0.5 ||
        self.total_improvement < 1.0 ||
        self.quality_score < 0.3
    }

    /// Get performance level description
    #[inline]
    pub fn performance_level(&self) -> &'static str {
        if self.is_excellent() {
            "Excellent"
        } else if self.is_good() {
            "Good"
        } else if self.is_poor() {
            "Poor"
        } else {
            "Fair"
        }
    }

    /// Get improvement recommendations
    #[inline]
    pub fn improvement_recommendations(&self) -> Vec<&'static str> {
        let mut recommendations = Vec::new();

        if self.success_rate < 0.7 {
            recommendations.push("Consider adjusting optimization strategy to improve success rate");
        }

        if self.efficiency_score < 1.0 {
            recommendations.push("Optimization efficiency is low, consider parallel execution");
        }

        if self.throughput < 1.0 {
            recommendations.push("Low throughput detected, consider increasing batch sizes");
        }

        if self.memory_efficiency < 0.1 {
            recommendations.push("Memory optimization is inefficient, review memory strategies");
        }

        recommendations
    }
}
