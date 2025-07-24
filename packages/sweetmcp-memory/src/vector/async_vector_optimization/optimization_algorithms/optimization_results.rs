//! Optimization result types and supporting structures
//!
//! This module provides blazing-fast result handling with zero allocation
//! optimizations and elegant ergonomic interfaces for optimization outcomes.

use std::time::Duration;

/// Dimension reduction optimization result
#[derive(Debug, Clone)]
pub struct DimensionReductionResult {
    pub original_dimensions: usize,
    pub target_dimensions: usize,
    pub compression_ratio: f64,
    pub execution_time: Duration,
}

impl DimensionReductionResult {
    #[inline]
    pub fn new(original: usize, target: usize, ratio: f64, time: Duration) -> Self {
        Self {
            original_dimensions: original,
            target_dimensions: target,
            compression_ratio: ratio,
            execution_time: time,
        }
    }

    /// Get compression percentage
    #[inline]
    pub fn compression_percentage(&self) -> f64 {
        (1.0 - self.compression_ratio) * 100.0
    }

    /// Check if optimization was successful
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.target_dimensions < self.original_dimensions && self.compression_ratio > 0.0
    }

    /// Get performance score
    #[inline]
    pub fn performance_score(&self) -> f64 {
        if self.execution_time.as_millis() == 0 {
            return 1.0;
        }
        
        let time_score = 1.0 / (self.execution_time.as_millis() as f64 / 1000.0);
        let compression_score = self.compression_ratio;
        
        (time_score + compression_score) / 2.0
    }
}

/// Vector quantization optimization result
#[derive(Debug, Clone)]
pub struct QuantizationResult {
    pub vectors_quantized: usize,
    pub quantization_levels: usize,
    pub compression_ratio: f64,
    pub execution_time: Duration,
}

impl QuantizationResult {
    #[inline]
    pub fn new(vectors: usize, levels: usize, ratio: f64, time: Duration) -> Self {
        Self {
            vectors_quantized: vectors,
            quantization_levels: levels,
            compression_ratio: ratio,
            execution_time: time,
        }
    }

    /// Get compression percentage
    #[inline]
    pub fn compression_percentage(&self) -> f64 {
        self.compression_ratio * 100.0
    }

    /// Check if optimization was successful
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.vectors_quantized > 0 && self.compression_ratio > 0.0
    }

    /// Get throughput (vectors per second)
    #[inline]
    pub fn throughput(&self) -> f64 {
        if self.execution_time.as_secs_f64() == 0.0 {
            return 0.0;
        }
        
        self.vectors_quantized as f64 / self.execution_time.as_secs_f64()
    }

    /// Get performance score
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let throughput_score = self.throughput() / 1000.0; // Normalize to 1000 vectors/sec
        let compression_score = self.compression_ratio;
        
        (throughput_score.min(1.0) + compression_score) / 2.0
    }
}

/// Index optimization result
#[derive(Debug, Clone)]
pub struct IndexOptimizationResult {
    pub indices_created: usize,
    pub efficiency_improvement: f64,
    pub execution_time: Duration,
}

impl IndexOptimizationResult {
    #[inline]
    pub fn new(indices: usize, improvement: f64, time: Duration) -> Self {
        Self {
            indices_created: indices,
            efficiency_improvement: improvement,
            execution_time: time,
        }
    }

    /// Get improvement percentage
    #[inline]
    pub fn improvement_percentage(&self) -> f64 {
        self.efficiency_improvement * 100.0
    }

    /// Check if optimization was successful
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.indices_created > 0 && self.efficiency_improvement > 0.0
    }

    /// Get performance score
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let time_score = 1.0 / (self.execution_time.as_millis() as f64 / 1000.0).max(0.1);
        let improvement_score = self.efficiency_improvement;
        
        (time_score.min(1.0) + improvement_score) / 2.0
    }
}

/// Cache optimization result
#[derive(Debug, Clone)]
pub struct CacheOptimizationResult {
    pub cache_size: usize,
    pub hit_rate_improvement: f64,
    pub execution_time: Duration,
}

impl CacheOptimizationResult {
    #[inline]
    pub fn new(size: usize, improvement: f64, time: Duration) -> Self {
        Self {
            cache_size: size,
            hit_rate_improvement: improvement,
            execution_time: time,
        }
    }

    /// Get improvement percentage
    #[inline]
    pub fn improvement_percentage(&self) -> f64 {
        self.hit_rate_improvement * 100.0
    }

    /// Check if optimization was successful
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.cache_size > 0 && self.hit_rate_improvement > 0.0
    }

    /// Get performance score
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let time_score = 1.0 / (self.execution_time.as_millis() as f64 / 100.0).max(0.1);
        let improvement_score = self.hit_rate_improvement;
        
        (time_score.min(1.0) + improvement_score) / 2.0
    }
}

/// Batch optimization result
#[derive(Debug, Clone)]
pub struct BatchOptimizationResult {
    pub optimal_batch_size: usize,
    pub throughput_improvement: f64,
    pub execution_time: Duration,
}

impl BatchOptimizationResult {
    #[inline]
    pub fn new(batch_size: usize, improvement: f64, time: Duration) -> Self {
        Self {
            optimal_batch_size: batch_size,
            throughput_improvement: improvement,
            execution_time: time,
        }
    }

    /// Get improvement percentage
    #[inline]
    pub fn improvement_percentage(&self) -> f64 {
        self.throughput_improvement * 100.0
    }

    /// Check if optimization was successful
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.optimal_batch_size > 0 && self.throughput_improvement > 0.0
    }

    /// Get performance score
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let time_score = 1.0 / (self.execution_time.as_millis() as f64 / 50.0).max(0.1);
        let improvement_score = self.throughput_improvement;
        
        (time_score.min(1.0) + improvement_score) / 2.0
    }
}

/// Memory layout optimization result
#[derive(Debug, Clone)]
pub struct MemoryLayoutResult {
    pub cache_efficiency_improvement: f64,
    pub execution_time: Duration,
}

impl MemoryLayoutResult {
    #[inline]
    pub fn new(improvement: f64, time: Duration) -> Self {
        Self {
            cache_efficiency_improvement: improvement,
            execution_time: time,
        }
    }

    /// Get improvement percentage
    #[inline]
    pub fn improvement_percentage(&self) -> f64 {
        self.cache_efficiency_improvement * 100.0
    }

    /// Check if optimization was successful
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.cache_efficiency_improvement > 0.0
    }

    /// Get performance score
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let time_score = 1.0 / (self.execution_time.as_millis() as f64 / 100.0).max(0.1);
        let improvement_score = self.cache_efficiency_improvement;
        
        (time_score.min(1.0) + improvement_score) / 2.0
    }
}

/// Supporting structures for optimization algorithms

/// Index structure representation
#[derive(Debug, Clone)]
pub struct IndexStructure {
    pub name: String,
    pub size: usize,
}

impl IndexStructure {
    #[inline]
    pub fn new(name: &str, size: usize) -> Self {
        Self {
            name: name.to_string(),
            size,
        }
    }

    /// Get index type
    #[inline]
    pub fn index_type(&self) -> IndexType {
        match self.name.as_str() {
            "hierarchical" => IndexType::Hierarchical,
            "lsh" => IndexType::LSH,
            "flat" => IndexType::Flat,
            _ => IndexType::Unknown,
        }
    }

    /// Get memory usage estimate
    #[inline]
    pub fn estimated_memory_usage(&self) -> usize {
        match self.index_type() {
            IndexType::Hierarchical => self.size * 8, // 8 bytes per entry
            IndexType::LSH => self.size * 16, // 16 bytes per entry
            IndexType::Flat => self.size * 4, // 4 bytes per entry
            IndexType::Unknown => self.size,
        }
    }
}

/// Index type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    Hierarchical,
    LSH,
    Flat,
    Unknown,
}

/// Access patterns for cache optimization
#[derive(Debug, Clone)]
pub struct AccessPatterns {
    pub hot_vectors: usize,
    pub cold_vectors: usize,
    pub access_frequency: f64,
}

impl AccessPatterns {
    /// Get total vectors
    #[inline]
    pub fn total_vectors(&self) -> usize {
        self.hot_vectors + self.cold_vectors
    }

    /// Get hot vector ratio
    #[inline]
    pub fn hot_vector_ratio(&self) -> f64 {
        if self.total_vectors() == 0 {
            return 0.0;
        }
        
        self.hot_vectors as f64 / self.total_vectors() as f64
    }

    /// Check if access pattern is skewed
    #[inline]
    pub fn is_skewed(&self) -> bool {
        self.hot_vector_ratio() > 0.8 || self.hot_vector_ratio() < 0.2
    }
}

/// Cache layout optimization
#[derive(Debug, Clone)]
pub struct CacheLayout {
    pub hot_cache_size: usize,
    pub cold_cache_size: usize,
    pub hit_rate_improvement: f64,
}

impl CacheLayout {
    /// Get total cache size
    #[inline]
    pub fn total_cache_size(&self) -> usize {
        self.hot_cache_size + self.cold_cache_size
    }

    /// Get hot cache ratio
    #[inline]
    pub fn hot_cache_ratio(&self) -> f64 {
        if self.total_cache_size() == 0 {
            return 0.0;
        }
        
        self.hot_cache_size as f64 / self.total_cache_size() as f64
    }

    /// Check if layout is balanced
    #[inline]
    pub fn is_balanced(&self) -> bool {
        let ratio = self.hot_cache_ratio();
        ratio >= 0.6 && ratio <= 0.9
    }
}

/// Memory layout analysis
#[derive(Debug, Clone)]
pub struct MemoryLayoutAnalysis {
    pub fragmentation_level: f64,
    pub cache_misses: usize,
    pub alignment_issues: usize,
}

impl MemoryLayoutAnalysis {
    /// Check if fragmentation is high
    #[inline]
    pub fn is_fragmentation_high(&self) -> bool {
        self.fragmentation_level > 0.5
    }

    /// Check if cache performance is poor
    #[inline]
    pub fn is_cache_performance_poor(&self) -> bool {
        self.cache_misses > 1000
    }

    /// Check if alignment needs optimization
    #[inline]
    pub fn needs_alignment_optimization(&self) -> bool {
        self.alignment_issues > 10
    }

    /// Get overall health score
    #[inline]
    pub fn health_score(&self) -> f64 {
        let fragmentation_score = 1.0 - self.fragmentation_level;
        let cache_score = 1.0 - (self.cache_misses as f64 / 10000.0).min(1.0);
        let alignment_score = 1.0 - (self.alignment_issues as f64 / 100.0).min(1.0);
        
        (fragmentation_score + cache_score + alignment_score) / 3.0
    }
}

/// Optimized memory layout
#[derive(Debug, Clone)]
pub struct OptimizedLayout {
    pub efficiency_improvement: f64,
    pub alignment_optimizations: usize,
    pub fragmentation_reduction: f64,
}

impl OptimizedLayout {
    /// Check if optimization is significant
    #[inline]
    pub fn is_significant(&self) -> bool {
        self.efficiency_improvement > 0.1 || 
        self.alignment_optimizations > 5 ||
        self.fragmentation_reduction > 0.2
    }

    /// Get overall optimization score
    #[inline]
    pub fn optimization_score(&self) -> f64 {
        let efficiency_score = self.efficiency_improvement;
        let alignment_score = (self.alignment_optimizations as f64 / 50.0).min(1.0);
        let fragmentation_score = self.fragmentation_reduction;
        
        (efficiency_score + alignment_score + fragmentation_score) / 3.0
    }
}

/// Combined optimization result
#[derive(Debug, Clone)]
pub struct CombinedOptimizationResult {
    pub dimension_reduction: Option<DimensionReductionResult>,
    pub quantization: Option<QuantizationResult>,
    pub index_optimization: Option<IndexOptimizationResult>,
    pub cache_optimization: Option<CacheOptimizationResult>,
    pub batch_optimization: Option<BatchOptimizationResult>,
    pub memory_layout: Option<MemoryLayoutResult>,
    pub total_execution_time: Duration,
}

impl CombinedOptimizationResult {
    /// Create new combined result
    #[inline]
    pub fn new() -> Self {
        Self {
            dimension_reduction: None,
            quantization: None,
            index_optimization: None,
            cache_optimization: None,
            batch_optimization: None,
            memory_layout: None,
            total_execution_time: Duration::from_millis(0),
        }
    }

    /// Get overall performance score
    #[inline]
    pub fn overall_performance_score(&self) -> f64 {
        let mut total_score = 0.0;
        let mut count = 0;

        if let Some(ref result) = self.dimension_reduction {
            total_score += result.performance_score();
            count += 1;
        }
        if let Some(ref result) = self.quantization {
            total_score += result.performance_score();
            count += 1;
        }
        if let Some(ref result) = self.index_optimization {
            total_score += result.performance_score();
            count += 1;
        }
        if let Some(ref result) = self.cache_optimization {
            total_score += result.performance_score();
            count += 1;
        }
        if let Some(ref result) = self.batch_optimization {
            total_score += result.performance_score();
            count += 1;
        }
        if let Some(ref result) = self.memory_layout {
            total_score += result.performance_score();
            count += 1;
        }

        if count == 0 {
            0.0
        } else {
            total_score / count as f64
        }
    }

    /// Check if any optimization was successful
    #[inline]
    pub fn has_successful_optimizations(&self) -> bool {
        self.dimension_reduction.as_ref().map_or(false, |r| r.is_successful()) ||
        self.quantization.as_ref().map_or(false, |r| r.is_successful()) ||
        self.index_optimization.as_ref().map_or(false, |r| r.is_successful()) ||
        self.cache_optimization.as_ref().map_or(false, |r| r.is_successful()) ||
        self.batch_optimization.as_ref().map_or(false, |r| r.is_successful()) ||
        self.memory_layout.as_ref().map_or(false, |r| r.is_successful())
    }

    /// Get optimization count
    #[inline]
    pub fn optimization_count(&self) -> usize {
        let mut count = 0;
        if self.dimension_reduction.is_some() { count += 1; }
        if self.quantization.is_some() { count += 1; }
        if self.index_optimization.is_some() { count += 1; }
        if self.cache_optimization.is_some() { count += 1; }
        if self.batch_optimization.is_some() { count += 1; }
        if self.memory_layout.is_some() { count += 1; }
        count
    }
}

impl Default for CombinedOptimizationResult {
    fn default() -> Self {
        Self::new()
    }
}
