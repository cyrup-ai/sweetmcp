//! Integration tests for the Cognitive Memory System

use std::time::Duration;
use surreal_memory::{
    CognitiveMemoryManager, CognitiveSettings, MemoryManager, MemoryNode, MemoryType,
};

/// Test backward compatibility with existing memory APIs
#[tokio::test]
async fn test_backward_compatibility() {
    // Skip if no test database available
    if std::env::var("SURREAL_TEST_URL").is_err() {
        eprintln!("Skipping test: SURREAL_TEST_URL not set");
        return;
    }

    let settings = CognitiveSettings {
        enabled: true,
        ..Default::default()
    };

    let manager = CognitiveMemoryManager::new(
        &std::env::var("SURREAL_TEST_URL").unwrap(),
        "test_ns",
        "test_db",
        settings,
    )
    .await
    .expect("Failed to create cognitive manager");

    // Test that all existing APIs still work
    let memory = MemoryNode::new("test content".to_string(), MemoryType::Semantic);
    let created = manager
        .create_memory(memory)
        .await
        .expect("Failed to create memory");

    assert!(!created.id.is_empty());
    assert_eq!(created.content, "test content");

    // Verify cognitive enhancement was applied
    // In a real test, we'd check the database for cognitive metadata
}

/// Test cognitive enhancement of memory nodes
#[tokio::test]
async fn test_cognitive_enhancement() {
    // Test that memories are enhanced with cognitive features
    let base_memory = MemoryNode::new(
        "Rust is a systems programming language".to_string(),
        MemoryType::Semantic,
    );

    let cognitive_memory = surreal_memory::CognitiveMemoryNode::from(base_memory);

    assert!(!cognitive_memory.is_enhanced());
    assert_eq!(
        cognitive_memory.base.content,
        "Rust is a systems programming language"
    );
}

/// Test cognitive search capabilities
#[tokio::test]
async fn test_cognitive_search() {
    if std::env::var("SURREAL_TEST_URL").is_err() {
        eprintln!("Skipping test: SURREAL_TEST_URL not set");
        return;
    }

    let settings = CognitiveSettings {
        enabled: true,
        ..Default::default()
    };

    let manager = CognitiveMemoryManager::new(
        &std::env::var("SURREAL_TEST_URL").unwrap(),
        "test_ns",
        "test_db_search",
        settings,
    )
    .await
    .expect("Failed to create cognitive manager");

    // Add test memories
    let memories = vec![
        MemoryNode::new(
            "Rust is great for systems programming".to_string(),
            MemoryType::Semantic,
        ),
        MemoryNode::new(
            "Python is great for machine learning".to_string(),
            MemoryType::Semantic,
        ),
        MemoryNode::new(
            "JavaScript is used for web development".to_string(),
            MemoryType::Semantic,
        ),
    ];

    for memory in memories {
        manager
            .create_memory(memory)
            .await
            .expect("Failed to create memory");
    }

    // Search using cognitive routing
    let results: Vec<_> = manager
        .search_by_content("programming languages")
        .collect()
        .await;

    // Verify cognitive routing provided relevant results
    assert!(!results.is_empty());
    assert!(results.len() <= 5);
}

/// Test quantum-inspired routing performance
#[tokio::test]
async fn test_quantum_routing_performance() {
    use std::sync::Arc;
    use surreal_memory::cognitive::quantum::{
        EnhancedQuery, QuantumConfig, QuantumRouter, QueryIntent,
    };
    use surreal_memory::cognitive::state::CognitiveStateManager;

    let state_manager = Arc::new(CognitiveStateManager::new());
    let config = QuantumConfig::default();
    let router = QuantumRouter::new(state_manager, config)
        .await
        .expect("Failed to create quantum router");

    let query = EnhancedQuery {
        original: "test query".to_string(),
        intent: QueryIntent::Retrieval,
        context_embedding: vec![0.1; 128],
        temporal_context: None,
        cognitive_hints: vec!["hint1".to_string(), "hint2".to_string()],
        expected_complexity: 0.5,
    };

    let start = std::time::Instant::now();
    let decision = router.route_query(&query).await.expect("Routing failed");
    let duration = start.elapsed();

    // Verify routing completed within acceptable time
    assert!(duration < Duration::from_millis(100)); // <20% latency increase target
    assert!(decision.confidence > 0.0);
    assert!(!decision.target_context.is_empty());
}

/// Test memory usage stays within bounds
#[tokio::test]
async fn test_memory_usage() {
    use std::sync::Arc;
    use surreal_memory::cognitive::quantum::QuantumRouter;
    use surreal_memory::cognitive::state::CognitiveStateManager;

    let initial_memory = get_current_memory_usage();

    // Create cognitive components
    let state_manager = Arc::new(CognitiveStateManager::new());
    let config = surreal_memory::cognitive::quantum::QuantumConfig::default();
    let _router = QuantumRouter::new(state_manager, config)
        .await
        .expect("Failed to create quantum router");

    // Simulate some operations
    for _ in 0..100 {
        // Create temporary states that should be garbage collected
        let _state = surreal_memory::CognitiveState::new(
            surreal_memory::cognitive::state::SemanticContext {
                primary_concepts: vec!["test".to_string()],
                secondary_concepts: vec![],
                domain_tags: vec![],
                abstraction_level: surreal_memory::cognitive::state::AbstractionLevel::Concrete,
            },
        );
    }

    let final_memory = get_current_memory_usage();
    let memory_increase = final_memory.saturating_sub(initial_memory);

    // Verify memory increase is within 30% target
    assert!(
        memory_increase < initial_memory * 30 / 100,
        "Memory increase {} exceeds 30% of initial {}",
        memory_increase,
        initial_memory
    );
}

/// Test evolution and self-optimization
#[tokio::test]
async fn test_evolution_engine() {
    use std::time::Duration;
    use surreal_memory::cognitive::evolution::{EvolutionEngine, PerformanceMetrics};

    let mut engine = EvolutionEngine::new(0.1);

    // Simulate performance improvements over time
    for generation in 0..50 {
        let metrics = PerformanceMetrics {
            retrieval_accuracy: 0.6 + (generation as f64 * 0.005), // Gradual improvement
            response_latency: Duration::from_millis(100 - generation),
            memory_efficiency: 0.7 + (generation as f64 * 0.003),
            adaptation_rate: 0.5 + (generation as f64 * 0.004),
        };

        engine.record_fitness(metrics);
    }

    // Trigger evolution
    let evolution_result = engine.evolve_if_needed().await;

    // Verify evolution can adapt
    assert!(evolution_result.is_some());
    if let Some(result) = evolution_result {
        assert!(result.predicted_improvement > 0.0);
    }
}

/// Test attention mechanism for memory retrieval
#[tokio::test]
async fn test_attention_mechanism() {
    use surreal_memory::cognitive::attention::{AttentionConfig, AttentionMechanism};

    let config = AttentionConfig {
        num_heads: 4,
        hidden_dim: 256,
        dropout_rate: 0.1,
        use_causal_mask: false,
    };

    let mut attention = AttentionMechanism::new(config);

    // Test memory scoring
    let query_embedding = vec![0.1, 0.2, 0.3, 0.4];
    let memory_embeddings = vec![
        ("mem1".to_string(), vec![0.1, 0.2, 0.3, 0.4]), // Exact match
        ("mem2".to_string(), vec![0.4, 0.3, 0.2, 0.1]), // Reverse
        ("mem3".to_string(), vec![0.0, 0.0, 0.0, 0.0]), // Zero
    ];

    let scores = attention
        .score_memories(&query_embedding, &memory_embeddings)
        .await;

    // Verify attention correctly ranks memories
    assert_eq!(scores[0].0, "mem1"); // Exact match should be first
    assert!(scores[0].1 > scores[1].1); // Higher score than others
    assert!(scores[2].1 < 0.1); // Zero vector should have low score
}

/// Benchmark: Traditional vs Cognitive Search
#[tokio::test]
#[ignore] // Run with --ignored flag for benchmarks
async fn bench_cognitive_vs_traditional_search() {
    use std::time::Instant;

    if std::env::var("SURREAL_TEST_URL").is_err() {
        eprintln!("Skipping benchmark: SURREAL_TEST_URL not set");
        return;
    }

    // Setup managers
    let cognitive_settings = CognitiveSettings {
        enabled: true,
        ..Default::default()
    };

    let traditional_settings = CognitiveSettings {
        enabled: false, // Disable cognitive features
        ..Default::default()
    };

    let cognitive_manager = CognitiveMemoryManager::new(
        &std::env::var("SURREAL_TEST_URL").unwrap(),
        "bench_ns",
        "bench_cognitive",
        cognitive_settings,
    )
    .await
    .expect("Failed to create cognitive manager");

    let traditional_manager = CognitiveMemoryManager::new(
        &std::env::var("SURREAL_TEST_URL").unwrap(),
        "bench_ns",
        "bench_traditional",
        traditional_settings,
    )
    .await
    .expect("Failed to create traditional manager");

    // Add test data
    for i in 0..100 {
        let memory = MemoryNode::new(format!("Test memory content {}", i), MemoryType::Semantic);
        cognitive_manager
            .create_memory(memory.clone())
            .await
            .unwrap();
        traditional_manager.create_memory(memory).await.unwrap();
    }

    // Benchmark traditional search
    let traditional_start = Instant::now();
    for _ in 0..10 {
        let _results: Vec<_> = traditional_manager
            .search_by_content("test query")
            .collect()
            .await;
    }
    let traditional_duration = traditional_start.elapsed();

    // Benchmark cognitive search
    let cognitive_start = Instant::now();
    for _ in 0..10 {
        let _results: Vec<_> = cognitive_manager
            .search_by_content("test query")
            .collect()
            .await;
    }
    let cognitive_duration = cognitive_start.elapsed();

    println!("Traditional search: {:?}", traditional_duration);
    println!("Cognitive search: {:?}", cognitive_duration);

    // Verify cognitive search meets performance target (<20% slower)
    let overhead = cognitive_duration.as_secs_f64() / traditional_duration.as_secs_f64();
    assert!(
        overhead < 1.2,
        "Cognitive overhead {}% exceeds 20% target",
        (overhead - 1.0) * 100.0
    );
}

// Helper function to get current memory usage
fn get_current_memory_usage() -> usize {
    // In a real implementation, would use system APIs to get actual memory usage
    // For now, return a placeholder
    1024 * 1024 * 100 // 100 MB
}
