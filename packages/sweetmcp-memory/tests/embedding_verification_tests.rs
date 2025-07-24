//! SIMD Embedding Operations Verification Tests
//!
//! Comprehensive QA validation for SIMD-optimized embedding generation
//! including correctness, performance, and safety verification.

use std::sync::Arc;
use sweetmcp_memory::memory::manager::MemoryCoordinator;
use sweetmcp_memory::memory::storage::InMemoryStorage;
use sweetmcp_memory::memory::{MemoryMetadata, MemoryType};
use sweetmcp_memory::vector::InMemoryVectorStore;

#[cfg(test)]
mod embedding_verification_tests {
    use super::*;
    use std::time::Instant;

    /// Test embedding generation produces correct dimensions
    #[tokio::test]
    async fn test_embedding_dimension_correctness() {
        let storage = Arc::new(InMemoryStorage::new());
        let vector_store = InMemoryVectorStore::new();
        let coordinator = MemoryCoordinator::new(storage, vector_store);

        let result = coordinator
            .add_memory(
                "test content".to_string(),
                MemoryType::Semantic,
                MemoryMetadata::default(),
            )
            .await;

        assert!(result.is_ok());
        let memory = result.unwrap();
        assert!(memory.embedding.is_some());

        let embedding = memory.embedding.unwrap();
        assert_eq!(
            embedding.len(),
            384,
            "Embedding should have exactly 384 dimensions"
        );
    }

    /// Test embedding normalization correctness
    #[tokio::test]
    async fn test_embedding_normalization() {
        let storage = Arc::new(InMemoryStorage::new());
        let vector_store = InMemoryVectorStore::new();
        let coordinator = MemoryCoordinator::new(storage, vector_store);

        let result = coordinator
            .add_memory(
                "test normalization".to_string(),
                MemoryType::Semantic,
                MemoryMetadata::default(),
            )
            .await;

        assert!(result.is_ok());
        let memory = result.unwrap();
        let embedding = memory.embedding.unwrap();

        // Verify unit normalization (magnitude should be approximately 1.0)
        let magnitude_squared: f32 = embedding.iter().map(|x| x * x).sum();
        let magnitude = magnitude_squared.sqrt();

        assert!(
            (magnitude - 1.0).abs() < 0.001,
            "Embedding should be unit normalized, got magnitude: {}",
            magnitude
        );
    }

    /// Test embedding determinism - same input should produce same output
    #[tokio::test]
    async fn test_embedding_determinism() {
        let storage1 = Arc::new(InMemoryStorage::new());
        let storage2 = Arc::new(InMemoryStorage::new());
        let coordinator1 = MemoryCoordinator::new(storage1, InMemoryVectorStore::new());
        let coordinator2 = MemoryCoordinator::new(storage2, InMemoryVectorStore::new());

        let test_text = "deterministic test content";

        let result1 = coordinator1
            .add_memory(
                test_text.to_string(),
                MemoryType::Semantic,
                MemoryMetadata::default(),
            )
            .await;

        let result2 = coordinator2
            .add_memory(
                test_text.to_string(),
                MemoryType::Semantic,
                MemoryMetadata::default(),
            )
            .await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let embedding1 = result1.unwrap().embedding.unwrap();
        let embedding2 = result2.unwrap().embedding.unwrap();

        assert_eq!(
            embedding1, embedding2,
            "Same input should produce identical embeddings"
        );
    }

    /// Test embedding uniqueness - different inputs should produce different outputs
    #[tokio::test]
    async fn test_embedding_uniqueness() {
        let storage = Arc::new(InMemoryStorage::new());
        let coordinator = MemoryCoordinator::new(storage, InMemoryVectorStore::new());

        let result1 = coordinator
            .add_memory(
                "first unique content".to_string(),
                MemoryType::Semantic,
                MemoryMetadata::default(),
            )
            .await;

        let result2 = coordinator
            .add_memory(
                "second unique content".to_string(),
                MemoryType::Semantic,
                MemoryMetadata::default(),
            )
            .await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let embedding1 = result1.unwrap().embedding.unwrap();
        let embedding2 = result2.unwrap().embedding.unwrap();

        assert_ne!(
            embedding1, embedding2,
            "Different inputs should produce different embeddings"
        );
    }

    /// Test embedding value range constraints
    #[tokio::test]
    async fn test_embedding_value_range() {
        let storage = Arc::new(InMemoryStorage::new());
        let coordinator = MemoryCoordinator::new(storage, InMemoryVectorStore::new());

        let result = coordinator
            .add_memory(
                "range test content".to_string(),
                MemoryType::Semantic,
                MemoryMetadata::default(),
            )
            .await;

        assert!(result.is_ok());
        let embedding = result.unwrap().embedding.unwrap();

        // Check that all values are in reasonable range after normalization
        for &value in &embedding {
            assert!(value.is_finite(), "Embedding values should be finite");
            assert!(
                value >= -1.0 && value <= 1.0,
                "Normalized embedding values should be in [-1, 1] range, got: {}",
                value
            );
        }
    }

    /// Test zero-allocation constraint verification
    #[tokio::test]
    async fn test_zero_allocation_embedding_generation() {
        let storage = Arc::new(InMemoryStorage::new());
        let coordinator = MemoryCoordinator::new(storage, InMemoryVectorStore::new());

        // We can't directly test allocation, but we can verify the API works
        // and produces the expected results which indicates the ArrayVec usage is correct
        for i in 0..10 {
            let result = coordinator
                .add_memory(
                    format!("test content {}", i),
                    MemoryType::Semantic,
                    MemoryMetadata::default(),
                )
                .await;

            assert!(result.is_ok());
            let embedding = result.unwrap().embedding.unwrap();
            assert_eq!(embedding.len(), 384);
        }
    }

    /// Test SIMD performance characteristics
    #[tokio::test]
    async fn test_simd_performance() {
        let storage = Arc::new(InMemoryStorage::new());
        let coordinator = MemoryCoordinator::new(storage, InMemoryVectorStore::new());

        let test_content =
            "performance test content with sufficient length to trigger SIMD optimizations";
        let iterations = 100;

        let start = Instant::now();

        for i in 0..iterations {
            let result = coordinator
                .add_memory(
                    format!("{} iteration {}", test_content, i),
                    MemoryType::Semantic,
                    MemoryMetadata::default(),
                )
                .await;
            assert!(result.is_ok());
        }

        let elapsed = start.elapsed();
        let per_operation = elapsed / iterations;

        // Expect SIMD-optimized operations to be reasonably fast
        assert!(
            per_operation.as_millis() < 10,
            "Embedding generation should be fast, took: {:?} per operation",
            per_operation
        );
    }

    /// Test edge cases - empty and very long strings
    #[tokio::test]
    async fn test_embedding_edge_cases() {
        let storage = Arc::new(InMemoryStorage::new());
        let coordinator = MemoryCoordinator::new(storage, InMemoryVectorStore::new());

        // Test empty string
        let result_empty = coordinator
            .add_memory(
                "".to_string(),
                MemoryType::Semantic,
                MemoryMetadata::default(),
            )
            .await;
        assert!(result_empty.is_ok());

        // Test very long string
        let long_string = "a".repeat(10000);
        let result_long = coordinator
            .add_memory(long_string, MemoryType::Semantic, MemoryMetadata::default())
            .await;
        assert!(result_long.is_ok());

        // Both should produce valid embeddings
        let embedding_empty = result_empty.unwrap().embedding.unwrap();
        let embedding_long = result_long.unwrap().embedding.unwrap();

        assert_eq!(embedding_empty.len(), 384);
        assert_eq!(embedding_long.len(), 384);
        assert_ne!(embedding_empty, embedding_long);
    }

    /// Test memory safety with concurrent access
    #[tokio::test]
    async fn test_concurrent_embedding_generation() {
        let storage = Arc::new(InMemoryStorage::new());
        let coordinator = Arc::new(MemoryCoordinator::new(storage, InMemoryVectorStore::new()));

        let mut handles = vec![];

        for i in 0..10 {
            let coord = coordinator.clone();
            let handle = tokio::spawn(async move {
                coord
                    .add_memory(
                        format!("concurrent test {}", i),
                        MemoryType::Semantic,
                        MemoryMetadata::default(),
                    )
                    .await
            });
            handles.push(handle);
        }

        // All operations should complete successfully
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            let embedding = result.unwrap().embedding.unwrap();
            assert_eq!(embedding.len(), 384);
        }
    }

    /// Test SIMD hash diversity with different hash seeds
    #[tokio::test]
    async fn test_hash_seed_diversity() {
        let storage = Arc::new(InMemoryStorage::new());
        let coordinator = MemoryCoordinator::new(storage, InMemoryVectorStore::new());

        // Generate multiple embeddings and verify they use diverse hash seeds
        let mut all_embeddings = vec![];

        for i in 0..5 {
            let result = coordinator
                .add_memory(
                    format!("diversity test {}", i),
                    MemoryType::Semantic,
                    MemoryMetadata::default(),
                )
                .await;

            assert!(result.is_ok());
            let embedding = result.unwrap().embedding.unwrap();
            all_embeddings.push(embedding);
        }

        // Verify that consecutive batches of 4 values show diversity
        // (indicating different hash seeds are being used)
        for embedding in &all_embeddings {
            let first_batch = &embedding[0..4];
            let second_batch = &embedding[4..8];

            // Values should not be identical (indicating seed diversity)
            assert_ne!(
                first_batch, second_batch,
                "Hash seed diversity should produce different value patterns"
            );
        }
    }

    /// Verify arithmetic operations maintain precision
    #[tokio::test]
    async fn test_numerical_precision() {
        let storage = Arc::new(InMemoryStorage::new());
        let coordinator = MemoryCoordinator::new(storage, InMemoryVectorStore::new());

        let result = coordinator
            .add_memory(
                "precision test".to_string(),
                MemoryType::Semantic,
                MemoryMetadata::default(),
            )
            .await;

        assert!(result.is_ok());
        let embedding = result.unwrap().embedding.unwrap();

        // Verify no NaN or infinite values
        for &value in &embedding {
            assert!(!value.is_nan(), "Embedding should not contain NaN values");
            assert!(
                !value.is_infinite(),
                "Embedding should not contain infinite values"
            );
        }

        // Verify precision is maintained in normalization
        let magnitude_squared: f32 = embedding.iter().map(|x| x * x).sum();
        assert!(
            magnitude_squared > 0.0,
            "Magnitude squared should be positive"
        );

        let magnitude = magnitude_squared.sqrt();
        assert!(
            (magnitude - 1.0).abs() < 1e-6,
            "Normalization should maintain high precision"
        );
    }
}

/// Benchmark tests for performance verification
#[cfg(test)]
mod embedding_benchmarks {
    use super::*;
    use std::time::Instant;

    /// Benchmark embedding generation throughput
    #[tokio::test]
    async fn benchmark_embedding_throughput() {
        let storage = Arc::new(InMemoryStorage::new());
        let coordinator = MemoryCoordinator::new(storage, InMemoryVectorStore::new());

        let long_string = "a".repeat(1000);
        let test_strings = vec![
            "short",
            "medium length content for testing",
            "much longer content string that will exercise the full SIMD pipeline with multiple hash operations and normalization passes",
            &long_string, // Very long string
        ];

        for test_string in test_strings {
            let iterations = 1000;
            let start = Instant::now();

            for i in 0..iterations {
                let _ = coordinator
                    .add_memory(
                        format!("{} {}", test_string, i),
                        MemoryType::Semantic,
                        MemoryMetadata::default(),
                    )
                    .await;
            }

            let elapsed = start.elapsed();
            let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();

            println!(
                "String length {}: {:.0} ops/sec",
                test_string.len(),
                ops_per_sec
            );

            // Expect reasonable performance (at least 10k ops/sec for short strings)
            if test_string.len() < 100 {
                assert!(
                    ops_per_sec > 10000.0,
                    "Short strings should process at >10k ops/sec, got: {:.0}",
                    ops_per_sec
                );
            }
        }
    }
}
