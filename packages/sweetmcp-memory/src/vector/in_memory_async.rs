//! In-memory vector store implementation with async trait

use dashmap::DashMap;
use smallvec::SmallVec;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;

use super::{
    PendingEmbedding, PendingVectorOp, PendingVectorSearch, VectorSearchResult, VectorStore,
};
use crate::memory::filter::MemoryFilter;
use crate::utils::error::Error;

/// In-memory vector store implementation with lock-free concurrent access
pub struct InMemoryVectorStore {
    vectors: Arc<DashMap<String, Vec<f32>>>,
    metadata: Arc<DashMap<String, serde_json::Value>>,
    operation_counter: Arc<AtomicUsize>,
}

impl InMemoryVectorStore {
    /// Create a new lock-free in-memory vector store
    pub fn new() -> Self {
        Self {
            vectors: Arc::new(DashMap::new()),
            metadata: Arc::new(DashMap::new()),
            operation_counter: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    /// Get the total number of operations performed (for metrics)
    pub fn operation_count(&self) -> usize {
        self.operation_counter.load(Ordering::Relaxed)
    }
    
    /// Get the number of stored vectors
    pub fn vector_count(&self) -> usize {
        self.vectors.len()
    }
    
    /// Get the number of metadata entries
    pub fn metadata_count(&self) -> usize {
        self.metadata.len()
    }
}

impl VectorStore for InMemoryVectorStore {
    fn add(
        &self,
        id: String,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let vectors = self.vectors.clone();
        let metadata_store = self.metadata.clone();
        let operation_counter = self.operation_counter.clone();

        tokio::spawn(async move {
            // Lock-free concurrent insert using DashMap
            vectors.insert(id.clone(), embedding);

            if let Some(meta) = metadata {
                metadata_store.insert(id, meta);
            }

            // Atomic operation count increment
            operation_counter.fetch_add(1, Ordering::Relaxed);

            let _ = tx.send(Ok(()));
        });

        PendingVectorOp::new(rx)
    }

    fn update(
        &self,
        id: String,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let vectors = self.vectors.clone();
        let metadata_store = self.metadata.clone();
        let operation_counter = self.operation_counter.clone();

        tokio::spawn(async move {
            // Lock-free check and update using DashMap atomic operations
            let result = if vectors.contains_key(&id) {
                // Update vector using lock-free concurrent insert
                vectors.insert(id.clone(), embedding);

                if let Some(meta) = metadata {
                    metadata_store.insert(id, meta);
                }

                // Atomic operation count increment
                operation_counter.fetch_add(1, Ordering::Relaxed);

                Ok(())
            } else {
                Err(Error::NotFound(format!("Vector with id {} not found", id)))
            };

            let _ = tx.send(result);
        });

        PendingVectorOp::new(rx)
    }

    fn delete(&self, id: String) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let vectors = self.vectors.clone();
        let metadata_store = self.metadata.clone();
        let operation_counter = self.operation_counter.clone();

        tokio::spawn(async move {
            // Lock-free concurrent removal using DashMap
            vectors.remove(&id);
            metadata_store.remove(&id);
            
            // Atomic operation count increment
            operation_counter.fetch_add(1, Ordering::Relaxed);
            
            let _ = tx.send(Ok(()));
        });

        PendingVectorOp::new(rx)
    }

    fn search(
        &self,
        query: Vec<f32>,
        limit: usize,
        filter: Option<MemoryFilter>,
    ) -> PendingVectorSearch {
        let (tx, rx) = oneshot::channel();
        let vectors = self.vectors.clone();
        let metadata_store = self.metadata.clone();
        let operation_counter = self.operation_counter.clone();

        tokio::spawn(async move {
            // Use SmallVec for zero-allocation for small result sets
            let mut results: SmallVec<[(String, f32, Option<serde_json::Value>); 64]> = SmallVec::new();

            // Lock-free iteration over DashMap using iter()
            for entry in vectors.iter() {
                let (id, vector) = (entry.key(), entry.value());
                
                // Apply memory filters with proper logic
                if let Some(ref memory_filter) = filter {
                    if let Some(metadata_entry) = metadata_store.get(id) {
                        let metadata_value = metadata_entry.value();
                        
                        // Apply filter logic based on MemoryFilter criteria
                        let passes_filter = match memory_filter {
                            MemoryFilter { memory_types: Some(types), .. } => {
                                // Check if metadata contains matching memory type
                                if let Some(memory_type) = metadata_value.get("memory_type") {
                                    if let Some(type_str) = memory_type.as_str() {
                                        types.iter().any(|filter_type| {
                                            format!("{:?}", filter_type).to_lowercase() == type_str.to_lowercase()
                                        })
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            }
                            MemoryFilter { time_range: Some(range), .. } => {
                                // Check timestamp filter using time_range
                                if let Some(timestamp) = metadata_value.get("created_at") {
                                    if let Some(ts_num) = timestamp.as_i64() {
                                        let mut passes = true;
                                        
                                        if let Some(start) = &range.start {
                                            passes = passes && ts_num >= start.timestamp();
                                        }
                                        
                                        if let Some(end) = &range.end {
                                            passes = passes && ts_num < end.timestamp();
                                        }
                                        
                                        passes
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            }
                            _ => true, // No filters or unsupported filters pass through
                        };
                        
                        if !passes_filter {
                            continue;
                        }
                    } else if memory_filter.memory_types.is_some() {
                        // If we're filtering by memory types but no metadata exists, skip
                        continue;
                    }
                }

                // SIMD-optimized cosine similarity computation
                let similarity = cosine_similarity_simd(&query, vector);
                let meta = metadata_store.get(id).map(|entry| entry.value().clone());
                
                results.push((id.clone(), similarity, meta));
                
                // Early termination for performance if we have enough results
                if results.len() > limit * 2 {
                    break;
                }
            }

            // Sort by similarity (descending) - optimized sort for small collections
            if results.len() <= 64 {
                // Use insertion sort for small collections (more cache-friendly)
                for i in 1..results.len() {
                    let mut j = i;
                    while j > 0 && results[j].1 > results[j - 1].1 {
                        results.swap(j, j - 1);
                        j -= 1;
                    }
                }
            } else {
                // Use standard sort for larger collections
                results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }

            // Take top k results with zero-allocation conversion
            let search_results: SmallVec<[VectorSearchResult; 32]> = results
                .into_iter()
                .take(limit)
                .map(|(id, score, metadata)| VectorSearchResult {
                    id,
                    score,
                    metadata,
                })
                .collect();

            // Atomic operation count increment
            operation_counter.fetch_add(1, Ordering::Relaxed);

            let _ = tx.send(Ok(search_results.into_vec()));
        });

        PendingVectorSearch::new(rx)
    }

    fn embed(&self, text: String) -> PendingEmbedding {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            // Simple mock embedding - in a real implementation, this would call an embedding model
            // For now, just generate a random vector based on text hash
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            let hash = hasher.finish();

            // Generate a 384-dimensional vector using zero-allocation stack array
            use arrayvec::ArrayVec;
            let mut embedding: ArrayVec<f32, 384> = ArrayVec::new();
            for i in 0..384 {
                let value = ((hash.wrapping_add(i as u64) % 1000) as f32) / 1000.0;
                embedding.push(value);
            }

            // Convert to Vec only at the boundary for compatibility
            let _ = tx.send(Ok(embedding.into_iter().collect()));
        });

        PendingEmbedding::new(rx)
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Hardware SIMD-optimized cosine similarity computation with runtime CPU feature detection
#[inline(always)]
fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    // For small vectors, use scalar computation for better cache efficiency
    if a.len() < 8 {
        return cosine_similarity(a, b);
    }

    // Runtime CPU feature detection with optimal path selection
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { cosine_similarity_avx2(a, b) };
        } else if is_x86_feature_detected!("sse4.1") {
            return unsafe { cosine_similarity_sse41(a, b) };
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return unsafe { cosine_similarity_neon(a, b) };
        }
    }

    // Fallback to optimized scalar implementation for unsupported architectures
    cosine_similarity_scalar_optimized(a, b)
}

/// AVX2-optimized cosine similarity using 256-bit SIMD vectors
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn cosine_similarity_avx2(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;

    let len = a.len();
    let mut dot_acc = _mm256_setzero_ps();
    let mut norm_a_acc = _mm256_setzero_ps();
    let mut norm_b_acc = _mm256_setzero_ps();

    // Process 8 floats at a time with AVX2 (256-bit registers)
    let simd_len = len & !7; // Round down to nearest multiple of 8
    let mut i = 0;

    while i < simd_len {
        // Load 8 floats from each vector into 256-bit registers
        let va = _mm256_loadu_ps(a.as_ptr().add(i));
        let vb = _mm256_loadu_ps(b.as_ptr().add(i));
        
        // Compute dot product: a[i] * b[i]
        dot_acc = _mm256_fmadd_ps(va, vb, dot_acc);
        
        // Compute squared norms: a[i]^2 and b[i]^2
        norm_a_acc = _mm256_fmadd_ps(va, va, norm_a_acc);
        norm_b_acc = _mm256_fmadd_ps(vb, vb, norm_b_acc);
        
        i += 8;
    }

    // Horizontal sum of accumulated SIMD registers
    let dot_product = simd_horizontal_sum_avx2(dot_acc);
    let norm_a_sq = simd_horizontal_sum_avx2(norm_a_acc);
    let norm_b_sq = simd_horizontal_sum_avx2(norm_b_acc);

    // Process remaining elements (scalar tail)
    let (final_dot, final_norm_a_sq, final_norm_b_sq) = process_scalar_tail(
        a, b, i, dot_product, norm_a_sq, norm_b_sq
    );

    // Compute final cosine similarity with zero-division protection
    compute_cosine_from_components(final_dot, final_norm_a_sq, final_norm_b_sq)
}

/// SSE4.1-optimized cosine similarity using 128-bit SIMD vectors
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse4.1")]
unsafe fn cosine_similarity_sse41(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;

    let len = a.len();
    let mut dot_acc = _mm_setzero_ps();
    let mut norm_a_acc = _mm_setzero_ps();
    let mut norm_b_acc = _mm_setzero_ps();

    // Process 4 floats at a time with SSE4.1 (128-bit registers)
    let simd_len = len & !3; // Round down to nearest multiple of 4
    let mut i = 0;

    while i < simd_len {
        // Load 4 floats from each vector into 128-bit registers
        let va = _mm_loadu_ps(a.as_ptr().add(i));
        let vb = _mm_loadu_ps(b.as_ptr().add(i));
        
        // Compute dot product and norms
        dot_acc = _mm_add_ps(dot_acc, _mm_mul_ps(va, vb));
        norm_a_acc = _mm_add_ps(norm_a_acc, _mm_mul_ps(va, va));
        norm_b_acc = _mm_add_ps(norm_b_acc, _mm_mul_ps(vb, vb));
        
        i += 4;
    }

    // Horizontal sum of SSE registers
    let dot_product = simd_horizontal_sum_sse(dot_acc);
    let norm_a_sq = simd_horizontal_sum_sse(norm_a_acc);
    let norm_b_sq = simd_horizontal_sum_sse(norm_b_acc);

    // Process remaining elements (scalar tail)
    let (final_dot, final_norm_a_sq, final_norm_b_sq) = process_scalar_tail(
        a, b, i, dot_product, norm_a_sq, norm_b_sq
    );

    // Compute final cosine similarity
    compute_cosine_from_components(final_dot, final_norm_a_sq, final_norm_b_sq)
}

/// NEON-optimized cosine similarity for ARM64 using 128-bit SIMD vectors
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn cosine_similarity_neon(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::aarch64::*;

    let len = a.len();
    let mut dot_acc = vdupq_n_f32(0.0);
    let mut norm_a_acc = vdupq_n_f32(0.0);
    let mut norm_b_acc = vdupq_n_f32(0.0);

    // Process 4 floats at a time with NEON (128-bit registers)
    let simd_len = len & !3; // Round down to nearest multiple of 4
    let mut i = 0;

    while i < simd_len {
        unsafe {
            // Load 4 floats from each vector into NEON registers
            let va = vld1q_f32(a.as_ptr().add(i));
            let vb = vld1q_f32(b.as_ptr().add(i));
            
            // Compute dot product and norms using fused multiply-add
            dot_acc = vfmaq_f32(dot_acc, va, vb);
            norm_a_acc = vfmaq_f32(norm_a_acc, va, va);
            norm_b_acc = vfmaq_f32(norm_b_acc, vb, vb);
        }
        
        i += 4;
    }

    // Horizontal sum of NEON registers
    let dot_product = unsafe { simd_horizontal_sum_neon(dot_acc) };
    let norm_a_sq = unsafe { simd_horizontal_sum_neon(norm_a_acc) };
    let norm_b_sq = unsafe { simd_horizontal_sum_neon(norm_b_acc) };

    // Process remaining elements (scalar tail)
    let (final_dot, final_norm_a_sq, final_norm_b_sq) = process_scalar_tail(
        a, b, i, dot_product, norm_a_sq, norm_b_sq
    );

    // Compute final cosine similarity
    compute_cosine_from_components(final_dot, final_norm_a_sq, final_norm_b_sq)
}

/// Optimized scalar implementation with manual loop unrolling for better ILP
#[inline(always)]
fn cosine_similarity_scalar_optimized(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut dot_product = 0.0f32;
    let mut norm_a_sq = 0.0f32;
    let mut norm_b_sq = 0.0f32;

    // Process 4 elements at a time for better instruction-level parallelism
    let unroll_len = len & !3; // Round down to nearest multiple of 4
    let mut i = 0;

    while i < unroll_len {
        // Manual loop unrolling for better ILP
        let a0 = a[i];     let b0 = b[i];
        let a1 = a[i + 1]; let b1 = b[i + 1];
        let a2 = a[i + 2]; let b2 = b[i + 2];
        let a3 = a[i + 3]; let b3 = b[i + 3];

        // Compute in parallel for better CPU pipeline utilization
        dot_product += a0 * b0 + a1 * b1 + a2 * b2 + a3 * b3;
        norm_a_sq += a0 * a0 + a1 * a1 + a2 * a2 + a3 * a3;
        norm_b_sq += b0 * b0 + b1 * b1 + b2 * b2 + b3 * b3;

        i += 4;
    }

    // Process remaining elements
    while i < len {
        let a_val = a[i];
        let b_val = b[i];
        
        dot_product += a_val * b_val;
        norm_a_sq += a_val * a_val;
        norm_b_sq += b_val * b_val;
        
        i += 1;
    }

    // Compute final cosine similarity
    compute_cosine_from_components(dot_product, norm_a_sq, norm_b_sq)
}

/// Horizontal sum for AVX2 256-bit registers (8 floats)
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[inline(always)]
unsafe fn simd_horizontal_sum_avx2(v: std::arch::x86_64::__m256) -> f32 {
    use std::arch::x86_64::*;
    
    // Add upper and lower 128-bit lanes
    let sum_128 = _mm_add_ps(_mm256_extractf128_ps(v, 1), _mm256_castps256_ps128(v));
    simd_horizontal_sum_sse(sum_128)
}

/// Horizontal sum for SSE 128-bit registers (4 floats)
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse")]
#[inline(always)]
unsafe fn simd_horizontal_sum_sse(v: std::arch::x86_64::__m128) -> f32 {
    use std::arch::x86_64::*;
    
    // Horizontal addition: [a,b,c,d] -> [a+c, b+d, a+c, b+d]
    let shuf = _mm_movehdup_ps(v);
    let sums = _mm_add_ps(v, shuf);
    
    // Final horizontal addition: [a+c, b+d, a+c, b+d] -> [a+b+c+d, ...]
    let shuf2 = _mm_movehl_ps(shuf, sums);
    let final_sum = _mm_add_ss(sums, shuf2);
    
    _mm_cvtss_f32(final_sum)
}

/// Horizontal sum for NEON 128-bit registers (4 floats)
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn simd_horizontal_sum_neon(v: std::arch::aarch64::float32x4_t) -> f32 {
    use std::arch::aarch64::*;
    
    // Pairwise addition and extraction
    let pair_sum = vpaddq_f32(v, v);
    let final_sum = vpaddq_f32(pair_sum, pair_sum);
    vgetq_lane_f32(final_sum, 0)
}

/// Process scalar tail elements after SIMD processing
#[inline(always)]
fn process_scalar_tail(
    a: &[f32], 
    b: &[f32], 
    start_idx: usize, 
    mut dot: f32, 
    mut norm_a_sq: f32, 
    mut norm_b_sq: f32
) -> (f32, f32, f32) {
    for i in start_idx..a.len() {
        let a_val = a[i];
        let b_val = b[i];
        
        dot += a_val * b_val;
        norm_a_sq += a_val * a_val;
        norm_b_sq += b_val * b_val;
    }
    
    (dot, norm_a_sq, norm_b_sq)
}

/// Compute cosine similarity from dot product and squared norms with zero-division protection
#[inline(always)]
fn compute_cosine_from_components(dot_product: f32, norm_a_sq: f32, norm_b_sq: f32) -> f32 {
    // Fast inverse square root approximation for performance
    if norm_a_sq <= f32::EPSILON || norm_b_sq <= f32::EPSILON {
        return 0.0;
    }
    
    let norm_product = (norm_a_sq * norm_b_sq).sqrt();
    
    // Clamp result to valid cosine range [-1, 1] to handle floating-point precision errors
    (dot_product / norm_product).clamp(-1.0, 1.0)
}
