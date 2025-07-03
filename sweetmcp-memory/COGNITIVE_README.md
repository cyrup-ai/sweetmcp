# Cognitive Memory System

A production-grade memory management system with quantum-inspired routing, attention mechanisms, and self-optimization capabilities for the Cyrun platform.

## Overview

The Cognitive Memory System enhances traditional memory storage with:

- **Quantum-Inspired Routing**: Uses superposition states and quantum measurement principles for intelligent query routing
- **Machine Learning Integration**: ML-based error correction and optimization
- **Evolution Engine**: Self-optimizing system that improves performance over time
- **Attention Mechanisms**: Multi-head attention for relevance scoring
- **100% Backward Compatible**: Works seamlessly with existing memory APIs

## Architecture

```
cognitive/
├── Quantum Router      # Quantum-inspired routing with superposition states
├── ML Decoder         # Neural network-based error correction
├── Evolution Engine   # Self-optimization and adaptation
├── Attention System   # Multi-head attention for memory scoring
└── State Manager      # Cognitive state tracking and management
```

## Quick Start

```rust
use surreal_memory::{
    CognitiveMemoryManager,
    CognitiveSettings,
    MemoryNode,
    MemoryType,
};

// Configure cognitive features
let settings = CognitiveSettings {
    enabled: true,
    llm_provider: "openai".to_string(),
    attention_heads: 8,
    evolution_rate: 0.1,
    quantum_coherence_time: Duration::from_secs(300),
};

// Initialize the cognitive memory manager
let manager = CognitiveMemoryManager::new(
    "http://localhost:8000",
    "namespace",
    "database",
    settings,
).await?;

// Use exactly like the standard memory manager
let memory = MemoryNode::new(
    "Rust is a systems programming language".to_string(),
    MemoryType::Semantic,
);

let stored = manager.create_memory(memory).await?;

// Search with cognitive enhancements
let results = manager
    .search_by_content("programming languages")
    .collect()
    .await;
```

## Features

### Quantum-Inspired Routing

The quantum router uses superposition states to explore multiple search strategies simultaneously:

```rust
use surreal_memory::cognitive::quantum::{
    QuantumRouter, EnhancedQuery, QueryIntent,
};

let query = EnhancedQuery {
    original: "What is Rust?".to_string(),
    intent: QueryIntent::Retrieval,
    context_embedding: vec![0.1; 128],
    temporal_context: None,
    cognitive_hints: vec!["programming".to_string()],
    expected_complexity: 0.5,
};

let decision = router.route_query(&query).await?;
```

### Machine Learning Components

ML-based error correction and decoding:

```rust
use surreal_memory::cognitive::quantum::ml_decoder::{
    MLDecoder, MLModelType, QuantumLayer,
};

// Neural network decoder
let decoder = MLDecoder::new(MLModelType::NeuralNetwork {
    layers: vec![10, 5, 2],
});

// Quantum neural network
let quantum_decoder = MLDecoder::new(MLModelType::QuantumNeuralNetwork {
    quantum_layers: vec![QuantumLayer::standard_layer(4)],
});
```

### Evolution Engine

Self-optimization based on performance metrics:

```rust
use surreal_memory::cognitive::evolution::EvolutionEngine;

let mut engine = EvolutionEngine::new(0.1);

// System automatically tracks performance and evolves
let evolution_result = engine.evolve_if_needed().await;
```

### Attention Mechanism

Multi-head attention for memory relevance scoring:

```rust
use surreal_memory::cognitive::attention::AttentionMechanism;

let attention = AttentionMechanism::new(config);
let scores = attention.score_memories(&query_embedding, &memory_embeddings).await;
```

## Performance

The system is designed to meet strict performance targets:

- **Search Quality**: 40%+ improvement in result relevance
- **Latency**: <20% increase for enhanced features
- **Memory Usage**: <30% increase in footprint
- **Evolution**: System adaptation within 24 hours

## Configuration

### Cognitive Settings

```rust
pub struct CognitiveSettings {
    pub enabled: bool,              // Enable/disable cognitive features
    pub llm_provider: String,       // LLM provider (openai, anthropic, etc.)
    pub attention_heads: usize,     // Number of attention heads (default: 8)
    pub evolution_rate: f32,        // Evolution rate (0.0-1.0, default: 0.1)
    pub quantum_coherence_time: Duration, // Quantum state lifetime
}
```

### Quantum Configuration

```rust
pub struct QuantumConfig {
    pub max_superposition_states: usize,  // Max concurrent quantum states
    pub default_coherence_time: Duration, // Default coherence time
    pub decoherence_threshold: f64,       // Decoherence threshold
    pub error_correction_enabled: bool,   // Enable error correction
    pub hardware_backend: QuantumHardwareBackend, // Backend selection
}
```

## Testing

Run all tests:
```bash
cargo test
```

Run integration tests:
```bash
cargo test --test cognitive_integration_tests
```

Run benchmarks:
```bash
cargo bench
```

## Dependencies

The cognitive system adds these dependencies:
- `nalgebra`: Matrix operations and linear algebra
- `petgraph`: Graph algorithms for entanglement networks
- `dashmap`: Concurrent hash maps for performance
- `ordered-float`: Ordered floating point operations

## Migration Guide

### From Standard Memory Manager

```rust
// Before
let manager = SurrealDBMemoryManager::new(url, ns, db).await?;

// After
let settings = CognitiveSettings::default();
let manager = CognitiveMemoryManager::new(url, ns, db, settings).await?;
```

All existing APIs work identically - the cognitive features are additive.

### Gradual Adoption

You can enable cognitive features gradually:

```rust
// Start with cognitive features disabled
let settings = CognitiveSettings {
    enabled: false,
    ..Default::default()
};

// Enable specific features as needed
settings.enabled = true;
settings.attention_heads = 4; // Start with fewer heads
settings.evolution_rate = 0.05; // Slower evolution
```

## Architecture Details

### Module Structure

- `cognitive/quantum/`: Quantum-inspired routing system
  - `router.rs`: Main quantum router implementation
  - `state.rs`: Superposition state management
  - `entanglement.rs`: Entanglement graph and correlations
  - `measurement.rs`: Quantum measurement operations
  - `error_correction.rs`: Quantum error correction
  
- `cognitive/`: High-level cognitive components
  - `manager.rs`: Cognitive memory manager
  - `state.rs`: Cognitive state management
  - `evolution.rs`: Evolution and self-optimization
  - `attention.rs`: Attention mechanisms

### Design Principles

1. **Backward Compatibility**: All enhancements are additive
2. **Performance First**: Meet strict latency and memory targets
3. **Modular Design**: Each component can be used independently
4. **Production Ready**: Comprehensive error handling and logging
5. **Self-Improving**: System optimizes itself over time

## Troubleshooting

### High Memory Usage

If memory usage exceeds targets:
1. Reduce `max_superposition_states` in quantum config
2. Lower `attention_heads` count
3. Increase garbage collection frequency

### Slow Performance

If latency exceeds targets:
1. Disable error correction temporarily
2. Use CPU-only quantum simulation
3. Reduce evolution rate

### Evolution Not Triggering

If the system isn't self-optimizing:
1. Check fitness metrics are being recorded
2. Verify evolution rate > 0
3. Ensure sufficient performance history

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.