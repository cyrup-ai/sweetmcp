//! Quick test to verify the cognitive system compiles and initializes

use std::sync::Arc;
use surreal_memory::cognitive::quantum::{
    EnhancedQuery, QuantumConfig, QuantumRouter, QueryIntent,
};
use surreal_memory::cognitive::state::CognitiveStateManager;

#[tokio::test]
async fn test_quantum_router_basic() {
    // Create state manager
    let state_manager = Arc::new(CognitiveStateManager::new());

    // Create quantum config
    let config = QuantumConfig::default();

    // Create quantum router
    let router = QuantumRouter::new(state_manager, config)
        .await
        .expect("Failed to create quantum router");

    // Create test query
    let query = EnhancedQuery {
        original: "What is Rust?".to_string(),
        intent: QueryIntent::Retrieval,
        context_embedding: vec![0.1; 64],
        temporal_context: None,
        cognitive_hints: vec!["programming".to_string(), "systems".to_string()],
        expected_complexity: 0.3,
    };

    // Route the query
    let decision = router
        .route_query(&query)
        .await
        .expect("Failed to route query");

    // Verify results
    assert!(decision.confidence > 0.0);
    assert!(!decision.target_context.is_empty());
    println!("Routing decision: {:?}", decision.strategy);
    println!("Target context: {}", decision.target_context);
    println!("Confidence: {}", decision.confidence);
    println!("Reasoning: {}", decision.reasoning);
}

#[test]
fn test_cognitive_memory_node_creation() {
    use surreal_memory::{CognitiveMemoryNode, MemoryType};

    let node = CognitiveMemoryNode::new("Test content".to_string(), MemoryType::Semantic);

    assert_eq!(node.base.content, "Test content");
    assert!(!node.is_enhanced());
}

#[test]
fn test_complex_number_operations() {
    use surreal_memory::cognitive::quantum::Complex64;

    let c1 = Complex64::new(3.0, 4.0);
    let c2 = Complex64::new(1.0, 2.0);

    // Test magnitude (3-4-5 triangle)
    assert_eq!(c1.magnitude(), 5.0);

    // Test addition
    let sum = c1 + c2;
    assert_eq!(sum.real, 4.0);
    assert_eq!(sum.imaginary, 6.0);

    // Test multiplication
    let product = c1 * c2;
    assert_eq!(product.real, -5.0); // 3*1 - 4*2 = 3 - 8 = -5
    assert_eq!(product.imaginary, 10.0); // 3*2 + 4*1 = 6 + 4 = 10
}

#[tokio::test]
async fn test_cognitive_state_manager() {
    use surreal_memory::cognitive::state::{
        AbstractionLevel, CognitiveState, CognitiveStateManager, SemanticContext,
    };

    let manager = CognitiveStateManager::new();

    // Create a cognitive state
    let context = SemanticContext {
        primary_concepts: vec!["rust".to_string(), "memory".to_string()],
        secondary_concepts: vec!["systems".to_string()],
        domain_tags: vec!["programming".to_string()],
        abstraction_level: AbstractionLevel::Intermediate,
    };

    let state = CognitiveState::new(context);
    let state_id = manager.add_state(state).await;

    // Retrieve the state
    let retrieved = manager.get_state(&state_id).await;
    assert!(retrieved.is_some());

    // Search by concept
    let found = manager.find_by_concept("rust").await;
    assert_eq!(found.len(), 1);

    // Search by domain
    let found = manager.find_by_domain("programming").await;
    assert_eq!(found.len(), 1);
}

#[test]
fn test_evolution_metadata() {
    use surreal_memory::cognitive::evolution::EvolutionMetadata;
    use surreal_memory::{MemoryNode, MemoryType};

    let memory = MemoryNode::new("test".to_string(), MemoryType::Semantic);
    let metadata = EvolutionMetadata::new(&memory);

    assert_eq!(metadata.generation, 0);
    assert_eq!(metadata.fitness_score, 0.5);
    assert_eq!(metadata.mutation_count, 0);
}

#[tokio::test]
async fn test_attention_mechanism() {
    use surreal_memory::cognitive::attention::{AttentionConfig, AttentionMechanism};

    let config = AttentionConfig {
        num_heads: 2,
        hidden_dim: 8,
        dropout_rate: 0.0,
        use_causal_mask: false,
    };

    let mut attention = AttentionMechanism::new(config);

    // Test similarity calculation
    let query = vec![1.0, 0.0, 0.0];
    let memories = vec![
        ("mem1".to_string(), vec![1.0, 0.0, 0.0]), // Same direction
        ("mem2".to_string(), vec![0.0, 1.0, 0.0]), // Orthogonal
        ("mem3".to_string(), vec![-1.0, 0.0, 0.0]), // Opposite
    ];

    let scores = attention.score_memories(&query, &memories).await;

    assert_eq!(scores[0].0, "mem1"); // Same direction should score highest
    assert!(scores[0].1 > 0.99); // Should be ~1.0
    assert!(scores[1].1.abs() < 0.01); // Orthogonal should be ~0
    assert!(scores[2].1 < -0.99); // Opposite should be ~-1.0
}

#[test]
fn test_measurement_basis() {
    use surreal_memory::cognitive::quantum::measurement::{BasisType, MeasurementBasis};

    let computational = MeasurementBasis::computational();
    assert!(matches!(computational.basis_type, BasisType::Computational));
    assert_eq!(computational.basis_vectors.len(), 2);

    let hadamard = MeasurementBasis::hadamard();
    assert!(matches!(hadamard.basis_type, BasisType::Hadamard));
    assert_eq!(hadamard.basis_vectors.len(), 2);
}

fn main() {
    println!("Run with 'cargo test' to execute all tests");
}
