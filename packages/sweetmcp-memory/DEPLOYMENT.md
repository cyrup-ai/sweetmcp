# Deployment Guide for Cognitive Memory System

## Prerequisites

- Rust 1.75+ (for async traits and latest features)
- SurrealDB 1.0+ instance
- (Optional) OpenAI API key for LLM features
- 8GB+ RAM recommended for production

## Environment Setup

### 1. Database Configuration

```bash
# Start SurrealDB
surreal start --log debug --user root --pass root memory://cognitive

# Or use file-based storage
surreal start --log debug --user root --pass root file://./data/cognitive.db
```

### 2. Environment Variables

```bash
# Required
export SURREAL_URL="http://localhost:8000"
export SURREAL_NAMESPACE="cognitive"
export SURREAL_DATABASE="memory"

# Optional - for integration tests
export SURREAL_TEST_URL="memory://test"

# Optional - for LLM features
export OPENAI_API_KEY="your-api-key"
export ANTHROPIC_API_KEY="your-api-key"

# Optional - for quantum hardware backends
export IBM_QUANTUM_TOKEN="your-token"
export GOOGLE_QUANTUM_PROJECT="your-project"
```

## Configuration Examples

### Basic Configuration

```rust
use surreal_memory::{CognitiveSettings, CognitiveMemoryManager};
use std::time::Duration;

let settings = CognitiveSettings {
    enabled: true,
    llm_provider: "openai".to_string(),
    attention_heads: 8,
    evolution_rate: 0.1,
    quantum_coherence_time: Duration::from_secs(300),
};

let manager = CognitiveMemoryManager::new(
    &std::env::var("SURREAL_URL").unwrap(),
    &std::env::var("SURREAL_NAMESPACE").unwrap(),
    &std::env::var("SURREAL_DATABASE").unwrap(),
    settings,
).await?;
```

### Advanced Configuration

```rust
use surreal_memory::cognitive::quantum::{QuantumConfig, QuantumHardwareBackend};

let quantum_config = QuantumConfig {
    max_superposition_states: 1000,
    default_coherence_time: Duration::from_millis(100),
    decoherence_threshold: 0.1,
    max_entanglement_depth: 5,
    error_correction_enabled: true,
    real_time_optimization: true,
    hardware_backend: QuantumHardwareBackend::Simulator {
        precision: SimulationPrecision::Float64,
        parallelization: true,
        gpu_acceleration: false,
    },
    simulation_parameters: SimulationParameters {
        shot_count: 1024,
        noise_model: NoiseModel::realistic(),
        optimization_level: 2,
        basis_gates: vec!["u1", "u2", "u3", "cx"].iter()
            .map(|s| s.to_string()).collect(),
        coupling_map: Some(CouplingMap::linear(5)),
    },
};
```

### Production Configuration

```rust
// config.toml
[database]
connection_string = "http://localhost:8000"
namespace = "production"
database = "cognitive_memory"
username = "admin"
password = "${SURREAL_PASSWORD}"

[cognitive]
enabled = true
llm_provider = "anthropic"
attention_heads = 16
evolution_rate = 0.05
quantum_coherence_time_ms = 500

[quantum]
max_superposition_states = 5000
error_correction_enabled = true
real_time_optimization = true

[performance]
max_memory_mb = 4096
max_concurrent_queries = 100
cache_size_mb = 512

[monitoring]
metrics_port = 9090
enable_tracing = true
log_level = "info"
```

## Deployment Steps

### 1. Build for Production

```bash
# Optimize for production
cargo build --release --features "cognitive quantum-routing evolution"

# Run tests
cargo test --release

# Check for security issues
cargo audit
```

### 2. Database Migration

```rust
use surreal_memory::migration::CognitiveMigration;

// Migrate existing memories to cognitive format
let migration = CognitiveMigration::new(&manager);
let report = migration.migrate_all().await?;

println!("Migrated {} memories", report.migrated_count);
println!("Errors: {}", report.errors.len());
```

### 3. Service Configuration

```yaml
# docker-compose.yml
version: '3.8'

services:
  surrealdb:
    image: surrealdb/surrealdb:latest
    command: start --log debug --user root --pass ${SURREAL_PASSWORD}
    ports:
      - "8000:8000"
    volumes:
      - ./data:/data
    environment:
      - SURREAL_PATH=/data/cognitive.db

  cognitive-memory:
    build: .
    depends_on:
      - surrealdb
    environment:
      - SURREAL_URL=http://surrealdb:8000
      - SURREAL_NAMESPACE=production
      - SURREAL_DATABASE=cognitive
      - RUST_LOG=info
    ports:
      - "8080:8080"
      - "9090:9090"  # Metrics
```

### 4. Monitoring Setup

```rust
use prometheus::{Encoder, TextEncoder, Counter, Histogram};

lazy_static! {
    static ref QUERY_COUNTER: Counter = Counter::new(
        "cognitive_queries_total", 
        "Total number of cognitive queries"
    ).unwrap();
    
    static ref QUERY_LATENCY: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "cognitive_query_duration_seconds",
            "Query latency in seconds"
        )
    ).unwrap();
}

// In your query handler
let timer = QUERY_LATENCY.start_timer();
let result = manager.search_by_content(query).await;
timer.observe_duration();
QUERY_COUNTER.inc();
```

## Performance Tuning

### Memory Optimization

```rust
// Tune garbage collection
let gc_config = GarbageCollectorConfig {
    collection_threshold: 0.8,
    collection_interval: Duration::from_secs(60),
    max_memory_usage: 4 * 1024 * 1024 * 1024, // 4GB
};

// Tune cache sizes
let cache_config = CacheConfig {
    embedding_cache_size: 10000,
    state_cache_size: 5000,
    ttl: Duration::from_secs(300),
};
```

### Latency Optimization

```rust
// Use connection pooling
let pool_config = PoolConfig {
    min_connections: 10,
    max_connections: 100,
    connection_timeout: Duration::from_secs(5),
};

// Enable query batching
let batch_config = BatchConfig {
    max_batch_size: 50,
    batch_timeout: Duration::from_millis(10),
};
```

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   ```bash
   # Check memory usage
   ps aux | grep cognitive
   
   # Adjust settings
   export COGNITIVE_MAX_STATES=500
   export COGNITIVE_GC_INTERVAL=30
   ```

2. **Slow Queries**
   ```rust
   // Enable query profiling
   manager.enable_profiling(true);
   
   // Check slow query log
   let slow_queries = manager.get_slow_queries().await?;
   ```

3. **Evolution Not Working**
   ```rust
   // Check evolution metrics
   let metrics = manager.get_evolution_metrics().await?;
   println!("Current generation: {}", metrics.generation);
   println!("Fitness trend: {:?}", metrics.fitness_trend);
   ```

### Debug Mode

```rust
// Enable debug logging
env_logger::Builder::from_env(
    Env::default().default_filter_or("surreal_memory=debug")
).init();

// Enable quantum state visualization
manager.enable_quantum_visualization(true);

// Dump cognitive state
let state = manager.dump_cognitive_state().await?;
std::fs::write("cognitive_state.json", serde_json::to_string_pretty(&state)?)?;
```

## Security Considerations

1. **API Keys**: Store in environment variables or secure vaults
2. **Database Access**: Use role-based access control
3. **Network Security**: Use TLS for all connections
4. **Input Validation**: Sanitize all user queries
5. **Rate Limiting**: Implement query rate limits

## Backup and Recovery

```bash
# Backup cognitive state
surreal export --ns cognitive --db memory backup.sql

# Backup evolution history
cargo run --bin backup-evolution -- --output evolution_backup.json

# Restore from backup
surreal import backup.sql
cargo run --bin restore-evolution -- --input evolution_backup.json
```

## Scaling Considerations

### Horizontal Scaling

```yaml
# kubernetes.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cognitive-memory
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cognitive-memory
  template:
    spec:
      containers:
      - name: cognitive-memory
        image: cognitive-memory:latest
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi"
            cpu: "2"
```

### Vertical Scaling

- Increase `max_superposition_states` for more concurrent quantum states
- Increase `attention_heads` for better memory scoring
- Adjust `evolution_rate` based on workload stability

## Maintenance

### Regular Tasks

1. **Weekly**: Check evolution metrics and fitness trends
2. **Monthly**: Archive old cognitive states
3. **Quarterly**: Review and update quantum hardware backends
4. **Annually**: Major version upgrades and schema migrations

### Health Checks

```rust
// Health check endpoint
async fn health_check(manager: &CognitiveMemoryManager) -> Result<HealthStatus> {
    let db_health = manager.check_database_connection().await?;
    let cognitive_health = manager.check_cognitive_components().await?;
    let memory_usage = manager.get_memory_usage().await?;
    
    Ok(HealthStatus {
        database: db_health,
        cognitive: cognitive_health,
        memory_usage_mb: memory_usage / 1024 / 1024,
        status: if db_health && cognitive_health { "healthy" } else { "degraded" },
    })
}
```

## Support

- Documentation: `/target/doc/surreal_memory/index.html`
- Issues: [GitHub Issues](https://github.com/cyrup/surreal_memory/issues)
- Community: [Discord](https://discord.gg/cyrup)

---

For more examples and advanced usage, see the `/examples` directory.