//! Operation cache for performance optimization
//!
//! This module provides blazing-fast caching with zero allocation optimizations
//! and elegant ergonomic interfaces for optimization result caching.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use super::super::optimization_recommendations::RecommendationType;
use super::optimization_results::SingleOptimizationResult;

/// Operation cache for performance optimization
#[derive(Debug)]
pub struct OperationCache {
    /// Cache entries
    entries: HashMap<RecommendationType, CachedResult>,
    /// Cache capacity
    capacity: usize,
    /// Cache statistics
    stats: CacheStatistics,
}

impl OperationCache {
    /// Create new operation cache
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(50)
    }

    /// Create cache with specific capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity,
            stats: CacheStatistics::new(),
        }
    }

    /// Get cached result
    #[inline]
    pub fn get(&mut self, recommendation_type: &RecommendationType) -> Option<&CachedResult> {
        match self.entries.get(recommendation_type) {
            Some(cached) => {
                self.stats.record_hit();
                Some(cached)
            }
            None => {
                self.stats.record_miss();
                None
            }
        }
    }

    /// Insert result into cache
    #[inline]
    pub fn insert(&mut self, recommendation_type: RecommendationType, result: SingleOptimizationResult) {
        // Evict oldest entry if at capacity
        if self.entries.len() >= self.capacity {
            if let Some(oldest_key) = self.find_oldest_entry() {
                self.entries.remove(&oldest_key);
            }
        }

        let cached_result = CachedResult {
            result,
            timestamp: SystemTime::now(),
        };

        self.entries.insert(recommendation_type, cached_result);
    }

    /// Clear all cache entries
    #[inline]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.stats.reset();
    }

    /// Get cache statistics
    #[inline]
    pub fn statistics(&self) -> CacheStatistics {
        self.stats.clone()
    }

    /// Get cache capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get current cache size
    #[inline]
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is full
    #[inline]
    pub fn is_full(&self) -> bool {
        self.entries.len() >= self.capacity
    }

    /// Check if cache is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get cache utilization ratio (0.0-1.0)
    #[inline]
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            return 0.0;
        }
        self.entries.len() as f64 / self.capacity as f64
    }

    /// Find oldest cache entry for eviction
    #[inline]
    fn find_oldest_entry(&self) -> Option<RecommendationType> {
        self.entries.iter()
            .min_by_key(|(_, cached)| cached.timestamp)
            .map(|(key, _)| key.clone())
    }

    /// Remove expired entries
    #[inline]
    pub fn remove_expired(&mut self, max_age: Duration) {
        let now = SystemTime::now();
        let expired_keys: Vec<RecommendationType> = self.entries.iter()
            .filter_map(|(key, cached)| {
                if now.duration_since(cached.timestamp).unwrap_or(Duration::from_secs(0)) > max_age {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in expired_keys {
            self.entries.remove(&key);
        }
    }

    /// Get all cached recommendation types
    #[inline]
    pub fn cached_types(&self) -> Vec<RecommendationType> {
        self.entries.keys().cloned().collect()
    }

    /// Check if recommendation type is cached
    #[inline]
    pub fn contains(&self, recommendation_type: &RecommendationType) -> bool {
        self.entries.contains_key(recommendation_type)
    }

    /// Remove specific entry
    #[inline]
    pub fn remove(&mut self, recommendation_type: &RecommendationType) -> Option<CachedResult> {
        self.entries.remove(recommendation_type)
    }

    /// Get cache entry age
    #[inline]
    pub fn entry_age(&self, recommendation_type: &RecommendationType) -> Option<Duration> {
        self.entries.get(recommendation_type)
            .and_then(|cached| cached.timestamp.elapsed().ok())
    }

    /// Resize cache capacity
    #[inline]
    pub fn resize(&mut self, new_capacity: usize) {
        self.capacity = new_capacity;
        
        // Remove excess entries if new capacity is smaller
        while self.entries.len() > new_capacity {
            if let Some(oldest_key) = self.find_oldest_entry() {
                self.entries.remove(&oldest_key);
            } else {
                break;
            }
        }
    }

    /// Get cache health status
    #[inline]
    pub fn health_status(&self) -> CacheHealthStatus {
        let hit_rate = self.stats.hit_rate;
        let utilization = self.utilization();
        
        if hit_rate > 0.8 && utilization > 0.3 && utilization < 0.9 {
            CacheHealthStatus::Excellent
        } else if hit_rate > 0.6 && utilization > 0.2 {
            CacheHealthStatus::Good
        } else if hit_rate > 0.3 {
            CacheHealthStatus::Fair
        } else {
            CacheHealthStatus::Poor
        }
    }

    /// Get cache performance metrics
    #[inline]
    pub fn performance_metrics(&self) -> CachePerformanceMetrics {
        CachePerformanceMetrics {
            hit_rate: self.stats.hit_rate,
            utilization: self.utilization(),
            size: self.size(),
            capacity: self.capacity,
            total_requests: self.stats.hits + self.stats.misses,
            health_status: self.health_status(),
        }
    }

    /// Optimize cache by removing least useful entries
    #[inline]
    pub fn optimize(&mut self) {
        // Remove expired entries first (older than 5 minutes)
        self.remove_expired(Duration::from_secs(300));
        
        // If still over 80% capacity, remove oldest entries
        let target_size = (self.capacity as f64 * 0.8) as usize;
        while self.entries.len() > target_size {
            if let Some(oldest_key) = self.find_oldest_entry() {
                self.entries.remove(&oldest_key);
            } else {
                break;
            }
        }
    }
}

impl Default for OperationCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cached optimization result
#[derive(Debug, Clone)]
pub struct CachedResult {
    pub result: SingleOptimizationResult,
    pub timestamp: SystemTime,
}

impl CachedResult {
    /// Create new cached result
    #[inline]
    pub fn new(result: SingleOptimizationResult) -> Self {
        Self {
            result,
            timestamp: SystemTime::now(),
        }
    }

    /// Check if cached result is still recent
    #[inline]
    pub fn is_recent(&self) -> bool {
        self.timestamp.elapsed()
            .map(|elapsed| elapsed < Duration::from_secs(300)) // 5 minutes
            .unwrap_or(false)
    }

    /// Get age of cached result
    #[inline]
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed().unwrap_or(Duration::from_secs(0))
    }

    /// Check if result is expired
    #[inline]
    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.age() > max_age
    }

    /// Check if result is still valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.is_recent() && self.result.success
    }

    /// Get cache value score (higher is better for retention)
    #[inline]
    pub fn value_score(&self) -> f64 {
        let age_penalty = self.age().as_secs_f64() / 3600.0; // Penalty for age in hours
        let success_bonus = if self.result.success { 1.0 } else { 0.0 };
        let improvement_bonus = self.result.improvement_achieved / 10.0;
        
        (success_bonus + improvement_bonus - age_penalty).max(0.0)
    }
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
}

impl CacheStatistics {
    /// Create new cache statistics
    #[inline]
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            hit_rate: 0.0,
        }
    }

    /// Record cache hit
    #[inline]
    pub fn record_hit(&mut self) {
        self.hits += 1;
        self.update_hit_rate();
    }

    /// Record cache miss
    #[inline]
    pub fn record_miss(&mut self) {
        self.misses += 1;
        self.update_hit_rate();
    }

    /// Reset statistics
    #[inline]
    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.hit_rate = 0.0;
    }

    /// Update hit rate calculation
    #[inline]
    fn update_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }

    /// Get total requests
    #[inline]
    pub fn total_requests(&self) -> usize {
        self.hits + self.misses
    }

    /// Check if statistics indicate good performance
    #[inline]
    pub fn is_performance_good(&self) -> bool {
        self.hit_rate > 0.6 && self.total_requests() > 10
    }

    /// Get performance level
    #[inline]
    pub fn performance_level(&self) -> CachePerformanceLevel {
        if self.hit_rate > 0.8 {
            CachePerformanceLevel::Excellent
        } else if self.hit_rate > 0.6 {
            CachePerformanceLevel::Good
        } else if self.hit_rate > 0.3 {
            CachePerformanceLevel::Fair
        } else {
            CachePerformanceLevel::Poor
        }
    }
}

impl Default for CacheStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheHealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl CacheHealthStatus {
    /// Get status description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            CacheHealthStatus::Excellent => "Cache is performing excellently",
            CacheHealthStatus::Good => "Cache is performing well",
            CacheHealthStatus::Fair => "Cache performance is fair",
            CacheHealthStatus::Poor => "Cache performance is poor",
        }
    }

    /// Check if status requires attention
    #[inline]
    pub fn requires_attention(&self) -> bool {
        matches!(self, CacheHealthStatus::Fair | CacheHealthStatus::Poor)
    }

    /// Get status score (0.0-1.0)
    #[inline]
    pub fn score(&self) -> f64 {
        match self {
            CacheHealthStatus::Excellent => 1.0,
            CacheHealthStatus::Good => 0.8,
            CacheHealthStatus::Fair => 0.5,
            CacheHealthStatus::Poor => 0.2,
        }
    }
}

/// Cache performance level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePerformanceLevel {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl CachePerformanceLevel {
    /// Get performance description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            CachePerformanceLevel::Excellent => "Excellent cache performance",
            CachePerformanceLevel::Good => "Good cache performance",
            CachePerformanceLevel::Fair => "Fair cache performance",
            CachePerformanceLevel::Poor => "Poor cache performance",
        }
    }
}

/// Cache performance metrics
#[derive(Debug, Clone)]
pub struct CachePerformanceMetrics {
    pub hit_rate: f64,
    pub utilization: f64,
    pub size: usize,
    pub capacity: usize,
    pub total_requests: usize,
    pub health_status: CacheHealthStatus,
}

impl CachePerformanceMetrics {
    /// Check if metrics indicate excellent performance
    #[inline]
    pub fn is_excellent(&self) -> bool {
        matches!(self.health_status, CacheHealthStatus::Excellent)
    }

    /// Check if metrics indicate good performance
    #[inline]
    pub fn is_good(&self) -> bool {
        matches!(self.health_status, CacheHealthStatus::Good | CacheHealthStatus::Excellent)
    }

    /// Get overall performance score (0.0-1.0)
    #[inline]
    pub fn overall_score(&self) -> f64 {
        let hit_rate_score = self.hit_rate;
        let utilization_score = if self.utilization > 0.3 && self.utilization < 0.9 {
            1.0
        } else if self.utilization > 0.1 {
            0.7
        } else {
            0.3
        };
        let health_score = self.health_status.score();
        
        (hit_rate_score + utilization_score + health_score) / 3.0
    }

    /// Get recommendations for improvement
    #[inline]
    pub fn improvement_recommendations(&self) -> Vec<&'static str> {
        let mut recommendations = Vec::new();
        
        if self.hit_rate < 0.5 {
            recommendations.push("Consider increasing cache size or adjusting eviction policy");
        }
        
        if self.utilization < 0.2 {
            recommendations.push("Cache is underutilized, consider reducing capacity");
        }
        
        if self.utilization > 0.9 {
            recommendations.push("Cache is nearly full, consider increasing capacity");
        }
        
        if self.total_requests < 10 {
            recommendations.push("Insufficient data for reliable metrics");
        }
        
        recommendations
    }
}
