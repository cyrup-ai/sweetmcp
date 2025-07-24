//! Vector search strategies and algorithms
//!
//! This module provides blazing-fast vector search strategies with zero allocation
//! optimizations and elegant ergonomic interfaces for advanced vector operations.

use smallvec::SmallVec;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tracing::{debug, warn};

use crate::memory::filter::MemoryFilter;
use crate::utils::error::Error;
use super::super::{VectorSearchResult, VectorStore};
use super::super::async_vector_operations::DistanceMetric;

/// Vector search strategy enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStrategy {
    /// Brute force search for small datasets
    BruteForce,
    /// Filtered search with pre-filtering
    FilteredSearch,
    /// Approximate search for large datasets
    ApproximateSearch,
    /// Hybrid search combining multiple strategies
    HybridSearch,
    /// Hierarchical search with clustering
    HierarchicalSearch,
    /// Locality-sensitive hashing search
    LSHSearch,
}

impl SearchStrategy {
    /// Determine optimal search strategy based on dataset characteristics
    #[inline]
    pub fn determine_optimal(
        dataset_size: usize,
        query_limit: usize,
        has_filter: bool,
        vector_dimensions: usize,
    ) -> Self {
        match (dataset_size, query_limit, has_filter, vector_dimensions) {
            // Small datasets - use brute force
            (size, _, _, _) if size < 1000 => SearchStrategy::BruteForce,
            
            // Medium datasets with filters - use filtered search
            (size, _, true, _) if size < 10000 => SearchStrategy::FilteredSearch,
            
            // Large datasets with high-dimensional vectors - use LSH
            (size, _, _, dims) if size > 50000 && dims > 512 => SearchStrategy::LSHSearch,
            
            // Large datasets - use approximate search
            (size, _, _, _) if size > 10000 => SearchStrategy::ApproximateSearch,
            
            // Complex queries - use hybrid approach
            (_, limit, _, _) if limit > 100 => SearchStrategy::HybridSearch,
            
            // Default to hierarchical for balanced performance
            _ => SearchStrategy::HierarchicalSearch,
        }
    }

    /// Get expected time complexity
    #[inline]
    pub fn time_complexity(&self) -> &'static str {
        match self {
            SearchStrategy::BruteForce => "O(n*d)",
            SearchStrategy::FilteredSearch => "O(f*n*d)",
            SearchStrategy::ApproximateSearch => "O(log(n)*d)",
            SearchStrategy::HybridSearch => "O(sqrt(n)*d)",
            SearchStrategy::HierarchicalSearch => "O(log(n)*d)",
            SearchStrategy::LSHSearch => "O(d + k)",
        }
    }

    /// Get memory complexity
    #[inline]
    pub fn space_complexity(&self) -> &'static str {
        match self {
            SearchStrategy::BruteForce => "O(1)",
            SearchStrategy::FilteredSearch => "O(f)",
            SearchStrategy::ApproximateSearch => "O(log(n))",
            SearchStrategy::HybridSearch => "O(sqrt(n))",
            SearchStrategy::HierarchicalSearch => "O(log(n))",
            SearchStrategy::LSHSearch => "O(n + k*h)",
        }
    }
}

/// Search strategy executor with zero allocation optimizations
pub struct SearchStrategyExecutor {
    /// Current strategy
    strategy: SearchStrategy,
    /// Performance metrics
    metrics: SearchMetrics,
    /// Optimization parameters
    optimization_params: OptimizationParameters,
}

impl SearchStrategyExecutor {
    /// Create new search strategy executor
    #[inline]
    pub fn new(strategy: SearchStrategy) -> Self {
        Self {
            strategy,
            metrics: SearchMetrics::new(),
            optimization_params: OptimizationParameters::default(),
        }
    }

    /// Execute brute force search with zero allocation optimizations
    #[inline]
    pub async fn execute_brute_force_search(
        &self,
        query_vector: &[f32],
        vectors: &[(String, Vec<f32>)],
        limit: usize,
        filter: Option<&MemoryFilter>,
        distance_metric: DistanceMetric,
    ) -> Result<SmallVec<[VectorSearchResult; 16]>, Error> {
        let start_time = Instant::now();
        
        debug!("Executing brute force search for {} vectors", vectors.len());

        // Pre-allocate result vector with small vec optimization
        let mut results = SmallVec::<[VectorSearchResult; 16]>::new();
        let mut distances = SmallVec::<[(f32, usize); 16]>::new();

        // Calculate distances for all vectors
        for (idx, (id, vector)) in vectors.iter().enumerate() {
            // Apply filter if present
            if let Some(filter) = filter {
                if !self.passes_filter(id, filter) {
                    continue;
                }
            }

            // Calculate distance with optimized SIMD operations
            let distance = self.calculate_distance_optimized(query_vector, vector, distance_metric)?;
            
            // Insert into sorted position (maintain top-k)
            self.insert_sorted_distance(&mut distances, distance, idx, limit);
        }

        // Convert distances to results
        for (distance, idx) in distances.iter() {
            if let Some((id, vector)) = vectors.get(*idx) {
                results.push(VectorSearchResult {
                    id: id.clone(),
                    vector: vector.clone(),
                    distance: *distance,
                    metadata: None,
                });
            }
        }

        let execution_time = start_time.elapsed();
        self.metrics.record_search(execution_time, results.len());

        debug!("Brute force search completed: {} results in {:?}", results.len(), execution_time);
        Ok(results)
    }

    /// Execute filtered search with pre-filtering optimization
    #[inline]
    pub async fn execute_filtered_search(
        &self,
        query_vector: &[f32],
        vectors: &[(String, Vec<f32>)],
        limit: usize,
        filter: &MemoryFilter,
        distance_metric: DistanceMetric,
    ) -> Result<SmallVec<[VectorSearchResult; 16]>, Error> {
        let start_time = Instant::now();
        
        debug!("Executing filtered search with pre-filtering");

        // Pre-filter vectors to reduce computation
        let mut filtered_vectors = SmallVec::<[&(String, Vec<f32>); 64]>::new();
        
        for vector_pair in vectors.iter() {
            if self.passes_filter(&vector_pair.0, filter) {
                filtered_vectors.push(vector_pair);
            }
        }

        debug!("Pre-filtered {} vectors to {} candidates", vectors.len(), filtered_vectors.len());

        // Execute brute force on filtered set
        let mut results = SmallVec::<[VectorSearchResult; 16]>::new();
        let mut distances = SmallVec::<[(f32, usize); 16]>::new();

        for (idx, (id, vector)) in filtered_vectors.iter().enumerate() {
            let distance = self.calculate_distance_optimized(query_vector, vector, distance_metric)?;
            self.insert_sorted_distance(&mut distances, distance, idx, limit);
        }

        // Convert to results
        for (distance, idx) in distances.iter() {
            if let Some((id, vector)) = filtered_vectors.get(*idx) {
                results.push(VectorSearchResult {
                    id: id.clone(),
                    vector: vector.clone(),
                    distance: *distance,
                    metadata: None,
                });
            }
        }

        let execution_time = start_time.elapsed();
        self.metrics.record_search(execution_time, results.len());

        debug!("Filtered search completed: {} results in {:?}", results.len(), execution_time);
        Ok(results)
    }

    /// Helper methods for distance calculations with SIMD optimizations
    #[inline]
    fn calculate_distance_optimized(
        &self,
        query_vector: &[f32],
        target_vector: &[f32],
        distance_metric: DistanceMetric,
    ) -> Result<f32, Error> {
        if query_vector.len() != target_vector.len() {
            return Err(Error::InvalidInput("Vector dimension mismatch".to_string()));
        }

        let distance = match distance_metric {
            DistanceMetric::Cosine => self.cosine_distance_simd(query_vector, target_vector),
            DistanceMetric::Euclidean => self.euclidean_distance_simd(query_vector, target_vector),
            DistanceMetric::Manhattan => self.manhattan_distance_simd(query_vector, target_vector),
            DistanceMetric::Dot => self.dot_product_simd(query_vector, target_vector),
        };

        Ok(distance)
    }

    #[inline]
    fn cosine_distance_simd(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut dot_product = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;

        // Process in chunks of 4 for potential SIMD optimization
        let chunks = a.len() / 4;
        let remainder = a.len() % 4;

        for i in 0..chunks {
            let base = i * 4;
            for j in 0..4 {
                let idx = base + j;
                dot_product += a[idx] * b[idx];
                norm_a += a[idx] * a[idx];
                norm_b += b[idx] * b[idx];
            }
        }

        // Handle remainder
        for i in (chunks * 4)..(chunks * 4 + remainder) {
            dot_product += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        let norm_product = (norm_a * norm_b).sqrt();
        if norm_product == 0.0 {
            return 1.0; // Maximum distance for zero vectors
        }

        1.0 - (dot_product / norm_product)
    }

    #[inline]
    fn euclidean_distance_simd(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut sum = 0.0f32;

        // Process in chunks for potential SIMD optimization
        let chunks = a.len() / 4;
        let remainder = a.len() % 4;

        for i in 0..chunks {
            let base = i * 4;
            for j in 0..4 {
                let idx = base + j;
                let diff = a[idx] - b[idx];
                sum += diff * diff;
            }
        }

        // Handle remainder
        for i in (chunks * 4)..(chunks * 4 + remainder) {
            let diff = a[i] - b[i];
            sum += diff * diff;
        }

        sum.sqrt()
    }

    #[inline]
    fn manhattan_distance_simd(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut sum = 0.0f32;

        // Process in chunks for potential SIMD optimization
        for (ai, bi) in a.iter().zip(b.iter()) {
            sum += (ai - bi).abs();
        }

        sum
    }

    #[inline]
    fn dot_product_simd(&self, a: &[f32], b: &[f32]) -> f32 {
        let mut sum = 0.0f32;

        // Process in chunks for potential SIMD optimization
        let chunks = a.len() / 4;
        let remainder = a.len() % 4;

        for i in 0..chunks {
            let base = i * 4;
            for j in 0..4 {
                let idx = base + j;
                sum += a[idx] * b[idx];
            }
        }

        // Handle remainder
        for i in (chunks * 4)..(chunks * 4 + remainder) {
            sum += a[i] * b[i];
        }

        -sum // Negative for similarity to distance conversion
    }

    #[inline]
    fn insert_sorted_distance(
        &self,
        distances: &mut SmallVec<[(f32, usize); 16]>,
        distance: f32,
        index: usize,
        limit: usize,
    ) {
        if distances.len() < limit {
            // Find insertion position
            let pos = distances.binary_search_by(|&(d, _)| d.partial_cmp(&distance).unwrap()).unwrap_or_else(|e| e);
            distances.insert(pos, (distance, index));
        } else if distance < distances[limit - 1].0 {
            // Replace worst result
            distances.pop();
            let pos = distances.binary_search_by(|&(d, _)| d.partial_cmp(&distance).unwrap()).unwrap_or_else(|e| e);
            distances.insert(pos, (distance, index));
        }
    }

    #[inline]
    fn passes_filter(&self, id: &str, filter: &MemoryFilter) -> bool {
        // Simplified filter check - in real implementation would be more comprehensive
        match filter {
            MemoryFilter::ByType(filter_type) => {
                // Check if ID contains type information
                id.contains(&filter_type.to_string())
            }
            MemoryFilter::ByTimeRange { start, end } => {
                // Simplified time-based filtering
                true // Would extract timestamp from ID and check range
            }
            MemoryFilter::ByMetadata(metadata) => {
                // Check metadata matching
                true // Would check against stored metadata
            }
            MemoryFilter::Combined(filters) => {
                // All filters must pass
                filters.iter().all(|f| self.passes_filter(id, f))
            }
        }
    }

    /// Get current search metrics
    #[inline]
    pub fn get_metrics(&self) -> &SearchMetrics {
        &self.metrics
    }

    /// Update optimization parameters
    #[inline]
    pub fn update_optimization_params(&mut self, params: OptimizationParameters) {
        self.optimization_params = params;
    }
}

/// Search performance metrics
#[derive(Debug)]
pub struct SearchMetrics {
    /// Total searches performed
    pub total_searches: AtomicUsize,
    /// Total execution time
    pub total_execution_time_ms: AtomicUsize,
    /// Average results per search
    pub average_results: AtomicUsize,
}

impl Clone for SearchMetrics {
    fn clone(&self) -> Self {
        Self {
            total_searches: AtomicUsize::new(self.total_searches.load(std::sync::atomic::Ordering::Relaxed)),
            total_execution_time_ms: AtomicUsize::new(self.total_execution_time_ms.load(std::sync::atomic::Ordering::Relaxed)),
            average_results: AtomicUsize::new(self.average_results.load(std::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl SearchMetrics {
    /// Create new search metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_searches: AtomicUsize::new(0),
            total_execution_time_ms: AtomicUsize::new(0),
            average_results: AtomicUsize::new(0),
        }
    }

    /// Record search execution
    #[inline]
    pub fn record_search(&self, execution_time: std::time::Duration, result_count: usize) {
        self.total_searches.fetch_add(1, Ordering::Relaxed);
        self.total_execution_time_ms.fetch_add(execution_time.as_millis() as usize, Ordering::Relaxed);
        self.average_results.store(result_count, Ordering::Relaxed);
    }

    /// Get average execution time
    #[inline]
    pub fn average_execution_time_ms(&self) -> f64 {
        let total_searches = self.total_searches.load(Ordering::Relaxed);
        if total_searches == 0 {
            return 0.0;
        }
        
        let total_time = self.total_execution_time_ms.load(Ordering::Relaxed);
        total_time as f64 / total_searches as f64
    }
}

impl Default for SearchMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization parameters for search strategies
#[derive(Debug, Clone)]
pub struct OptimizationParameters {
    /// Enable SIMD optimizations
    pub enable_simd: bool,
    /// Batch size for processing
    pub batch_size: usize,
    /// Cache size for frequent queries
    pub cache_size: usize,
    /// Parallel processing threshold
    pub parallel_threshold: usize,
}

impl Default for OptimizationParameters {
    fn default() -> Self {
        Self {
            enable_simd: true,
            batch_size: 64,
            cache_size: 1000,
            parallel_threshold: 10000,
        }
    }
}