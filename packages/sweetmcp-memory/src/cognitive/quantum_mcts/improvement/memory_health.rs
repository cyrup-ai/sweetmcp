//! Memory health monitoring and analysis with predictive insights
//!
//! This module provides comprehensive memory health analysis with allocation tracking,
//! trend analysis, and automated cleanup recommendations for optimal performance.

use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Allocation statistics for detailed memory analysis
#[derive(Debug, Clone, Default)]
pub struct AllocationStats {
    /// Total allocations performed
    pub total_allocations: u64,
    /// Total deallocations performed  
    pub total_deallocations: u64,
    /// Current outstanding allocations
    pub outstanding_allocations: u64,
    /// Peak outstanding allocations
    pub peak_outstanding: u64,
    /// Average allocation size in bytes
    pub average_allocation_size: f64,
    /// Total bytes allocated
    pub total_bytes_allocated: u64,
    /// Total bytes deallocated
    pub total_bytes_deallocated: u64,
    /// Current bytes outstanding
    pub outstanding_bytes: u64,
    /// Peak bytes outstanding
    pub peak_outstanding_bytes: u64,
    /// Allocation failure count
    pub allocation_failures: u64,
    /// Last allocation timestamp
    pub last_allocation_time: Option<Instant>,
}

impl AllocationStats {
    /// Create new allocation statistics
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a new allocation
    pub fn record_allocation(&mut self, size: usize) {
        self.total_allocations += 1;
        self.outstanding_allocations += 1;
        self.total_bytes_allocated += size as u64;
        self.outstanding_bytes += size as u64;
        
        // Update peaks
        self.peak_outstanding = self.peak_outstanding.max(self.outstanding_allocations);
        self.peak_outstanding_bytes = self.peak_outstanding_bytes.max(self.outstanding_bytes);
        
        // Update average allocation size
        self.average_allocation_size = self.total_bytes_allocated as f64 / self.total_allocations as f64;
        
        self.last_allocation_time = Some(Instant::now());
    }
    
    /// Record a deallocation
    pub fn record_deallocation(&mut self, size: usize) {
        self.total_deallocations += 1;
        self.outstanding_allocations = self.outstanding_allocations.saturating_sub(1);
        self.total_bytes_deallocated += size as u64;
        self.outstanding_bytes = self.outstanding_bytes.saturating_sub(size as u64);
    }
    
    /// Record an allocation failure
    pub fn record_allocation_failure(&mut self) {
        self.allocation_failures += 1;
    }
    
    /// Get allocation efficiency ratio
    pub fn allocation_efficiency(&self) -> f64 {
        if self.total_allocations > 0 {
            (self.total_allocations - self.allocation_failures) as f64 / self.total_allocations as f64
        } else {
            1.0
        }
    }
    
    /// Get memory fragmentation estimate
    pub fn fragmentation_estimate(&self) -> f64 {
        if self.outstanding_allocations > 0 && self.outstanding_bytes > 0 {
            let avg_outstanding_size = self.outstanding_bytes as f64 / self.outstanding_allocations as f64;
            let size_variance = (avg_outstanding_size - self.average_allocation_size).abs();
            size_variance / self.average_allocation_size.max(1.0)
        } else {
            0.0
        }
    }
}

/// Memory trend analysis for predictive insights
#[derive(Debug, Clone)]
pub struct MemoryTrend {
    /// Historical usage samples
    usage_samples: VecDeque<(Instant, usize)>,
    /// Maximum samples to keep
    max_samples: usize,
    /// Trend direction
    direction: TrendDirection,
    /// Trend strength (0.0 to 1.0)
    strength: f64,
    /// Predicted usage in next time window
    predicted_usage: Option<usize>,
}

impl MemoryTrend {
    /// Create new memory trend analyzer
    pub fn new(max_samples: usize) -> Self {
        Self {
            usage_samples: VecDeque::with_capacity(max_samples),
            max_samples,
            direction: TrendDirection::Stable,
            strength: 0.0,
            predicted_usage: None,
        }
    }
    
    /// Add new usage sample
    pub fn add_sample(&mut self, usage: usize) {
        let now = Instant::now();
        
        // Remove old samples if at capacity
        if self.usage_samples.len() >= self.max_samples {
            self.usage_samples.pop_front();
        }
        
        self.usage_samples.push_back((now, usage));
        self.analyze_trend();
    }
    
    /// Analyze current trend
    fn analyze_trend(&mut self) {
        if self.usage_samples.len() < 3 {
            self.direction = TrendDirection::Insufficient;
            self.strength = 0.0;
            return;
        }
        
        // Calculate linear regression slope
        let n = self.usage_samples.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        
        let _start_time = self.usage_samples[0].0;
        
        for (i, (_time, usage)) in self.usage_samples.iter().enumerate() {
            let x = i as f64;
            let y = *usage as f64;
            
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let correlation = slope.abs() / (sum_y / n).max(1.0);
        
        // Determine trend direction and strength
        if slope > 1.0 {
            self.direction = TrendDirection::Increasing;
            self.strength = correlation.min(1.0);
        } else if slope < -1.0 {
            self.direction = TrendDirection::Decreasing;
            self.strength = correlation.min(1.0);
        } else {
            self.direction = TrendDirection::Stable;
            self.strength = 1.0 - correlation.min(1.0);
        }
        
        // Predict next usage
        if self.usage_samples.len() >= 2 {
            let last_usage = self.usage_samples.back().unwrap().1 as f64;
            self.predicted_usage = Some((last_usage + slope) as usize);
        }
    }
    
    /// Get trend direction
    pub fn direction(&self) -> TrendDirection {
        self.direction
    }
    
    /// Get trend strength
    pub fn strength(&self) -> f64 {
        self.strength
    }
    
    /// Get predicted usage
    pub fn predicted_usage(&self) -> Option<usize> {
        self.predicted_usage
    }
    
    /// Check if trend is concerning
    pub fn is_concerning(&self, threshold_usage: usize) -> bool {
        match (self.direction, self.predicted_usage) {
            (TrendDirection::Increasing, Some(predicted)) => {
                predicted > threshold_usage && self.strength > 0.7
            }
            _ => false,
        }
    }
}

/// Trend direction enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Insufficient,
}

/// Cleanup recommendations for memory optimization
#[derive(Debug, Clone)]
pub struct CleanupRecommendation {
    /// Priority level of the recommendation
    pub priority: CleanupPriority,
    /// Recommended action
    pub action: CleanupAction,
    /// Expected memory savings in bytes
    pub expected_savings: usize,
    /// Estimated effort required
    pub effort_level: EffortLevel,
    /// Human-readable description
    pub description: String,
}

impl CleanupRecommendation {
    /// Create new cleanup recommendation
    pub fn new(
        priority: CleanupPriority,
        action: CleanupAction,
        expected_savings: usize,
        effort_level: EffortLevel,
        description: String,
    ) -> Self {
        Self {
            priority,
            action,
            expected_savings,
            effort_level,
            description,
        }
    }
    
    /// Get priority score for sorting
    pub fn priority_score(&self) -> u8 {
        match self.priority {
            CleanupPriority::Critical => 4,
            CleanupPriority::High => 3,
            CleanupPriority::Medium => 2,
            CleanupPriority::Low => 1,
        }
    }
}

/// Cleanup priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CleanupPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Cleanup action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupAction {
    GarbageCollect,
    CompactMemory,
    ClearCache,
    ReduceTreeSize,
    LimitParallelism,
    ForceCleanup,
}

/// Effort level for cleanup actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffortLevel {
    Minimal,
    Low,
    Medium,
    High,
}

/// Comprehensive memory health status
#[derive(Debug, Clone)]
pub struct MemoryHealth {
    /// Overall health status
    pub status: MemoryHealthStatus,
    /// Allocation statistics
    pub allocation_stats: AllocationStats,
    /// Memory trend analysis
    pub trend: MemoryTrend,
    /// Active cleanup recommendations
    pub recommendations: Vec<CleanupRecommendation>,
    /// Health score (0.0 to 1.0)
    pub health_score: f64,
    /// Last analysis timestamp
    pub last_analysis: Instant,
}

impl MemoryHealth {
    /// Create new memory health monitor
    pub fn new() -> Self {
        Self {
            status: MemoryHealthStatus::Good,
            allocation_stats: AllocationStats::new(),
            trend: MemoryTrend::new(50),
            recommendations: Vec::new(),
            health_score: 1.0,
            last_analysis: Instant::now(),
        }
    }
    
    /// Analyze current memory health
    pub fn analyze(&mut self, current_usage: usize, max_capacity: usize) {
        self.trend.add_sample(current_usage);
        
        // Calculate health score
        let usage_ratio = current_usage as f64 / max_capacity as f64;
        let allocation_efficiency = self.allocation_stats.allocation_efficiency();
        let fragmentation = self.allocation_stats.fragmentation_estimate();
        
        self.health_score = (allocation_efficiency * 0.4 + (1.0 - usage_ratio) * 0.4 + (1.0 - fragmentation) * 0.2)
            .clamp(0.0, 1.0);
        
        // Determine status
        self.status = if self.health_score > 0.8 {
            MemoryHealthStatus::Good
        } else if self.health_score > 0.6 {
            MemoryHealthStatus::Moderate
        } else if self.health_score > 0.4 {
            MemoryHealthStatus::Warning
        } else {
            MemoryHealthStatus::Critical
        };
        
        // Generate recommendations
        self.generate_recommendations(current_usage, max_capacity);
        
        self.last_analysis = Instant::now();
    }
    
    /// Generate cleanup recommendations
    fn generate_recommendations(&mut self, current_usage: usize, max_capacity: usize) {
        self.recommendations.clear();
        
        let usage_ratio = current_usage as f64 / max_capacity as f64;
        
        // High usage recommendations
        if usage_ratio > 0.8 {
            self.recommendations.push(CleanupRecommendation::new(
                CleanupPriority::High,
                CleanupAction::GarbageCollect,
                (current_usage as f64 * 0.1) as usize,
                EffortLevel::Low,
                "Perform garbage collection to free unused memory".to_string(),
            ));
        }
        
        // Fragmentation recommendations
        if self.allocation_stats.fragmentation_estimate() > 0.3 {
            self.recommendations.push(CleanupRecommendation::new(
                CleanupPriority::Medium,
                CleanupAction::CompactMemory,
                (current_usage as f64 * 0.05) as usize,
                EffortLevel::Medium,
                "Compact memory to reduce fragmentation".to_string(),
            ));
        }
        
        // Trend-based recommendations
        if self.trend.is_concerning(max_capacity) {
            self.recommendations.push(CleanupRecommendation::new(
                CleanupPriority::High,
                CleanupAction::LimitParallelism,
                0,
                EffortLevel::Minimal,
                "Limit parallelism to control memory growth".to_string(),
            ));
        }
        
        // Critical usage recommendations
        if usage_ratio > 0.95 {
            self.recommendations.push(CleanupRecommendation::new(
                CleanupPriority::Critical,
                CleanupAction::ForceCleanup,
                (current_usage as f64 * 0.2) as usize,
                EffortLevel::High,
                "Emergency cleanup required - memory nearly exhausted".to_string(),
            ));
        }
        
        // Sort recommendations by priority
        self.recommendations.sort_by(|a, b| b.priority_score().cmp(&a.priority_score()));
    }
}

/// Memory health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryHealthStatus {
    Good,
    Moderate,
    Warning,
    Critical,
}

impl Default for MemoryHealth {
    fn default() -> Self {
        Self::new()
    }
}