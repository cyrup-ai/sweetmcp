# Memory System Configuration Guide

The memory system provides sophisticated storage and retrieval capabilities for AI agents, with built-in vector search and graph relationships powered by SurrealDB.

## Quick Start

To use the memory system, you need:

1. **SurrealDB** running (handles all vector search and graph operations internally)
2. **API key** for embeddings (OpenAI or Anthropic)
3. **A few environment variables**

That's it! The system handles all the complexity internally.

## Required Configuration

### Environment Variables

```bash
# SurrealDB Connection
SURREAL_URL=http://localhost:8000      # Your SurrealDB instance
SURREAL_NAMESPACE=production            # Namespace for isolation
SURREAL_DATABASE=agent_memory           # Database name
SURREAL_USERNAME=root                   # Optional: for authentication
SURREAL_PASSWORD=your_password          # Optional: for authentication

# Embeddings (choose one)
OPENAI_API_KEY=sk-...                  # For OpenAI embeddings
# OR
ANTHROPIC_API_KEY=sk-ant-...           # For Anthropic embeddings
```

## How Memory Works

The memory system automatically:
- Generates embeddings for your content using OpenAI/Anthropic
- Stores them in SurrealDB with vector indexes
- Creates graph relationships between related memories
- Provides semantic search across all memories

## Basic Usage

```rust
use surreal_memory::{MemoryManager, MemoryNode, MemoryType};

// Create a memory
let memory = MemoryNode::new(
    "The user prefers dark mode interfaces".to_string(),
    MemoryType::Semantic
);
manager.create_memory(memory).await?;

// Search memories semantically
let results = manager
    .search_by_content("user interface preferences")
    .collect()
    .await;
```

## Memory Types

- **Semantic**: General knowledge and facts
- **Episodic**: Event-based memories with temporal context
- **Procedural**: How-to knowledge and processes

## Configuration Examples

### Local Development with kv-surrealkv

```toml
[database]
connection_string = "file://./data/memory.db"  # File-based storage
namespace = "dev"
database = "test_memory"

[vector_store]
store_type = "SurrealDB"  # Always uses SurrealDB's native vector search
dimension = 768           # nomic-embed-text dimension

[llm]
provider = "Custom"       # For ollama
model_name = "llama2"
api_base = "http://localhost:11434/api/generate"
```

### Local Development with OpenAI

```toml
[database]
connection_string = "http://localhost:8000"
namespace = "dev"
database = "test_memory"

[vector_store]
store_type = "SurrealDB"  # Always uses SurrealDB's native vector search
dimension = 1536          # OpenAI ada-002 embeddings

[llm]
provider = "OpenAI"
model_name = "gpt-4"
api_key = "${OPENAI_API_KEY}"
```

### Production Deployment

```toml
[database]
connection_string = "${SURREAL_URL}"
namespace = "production"
database = "agent_memory"
username = "${SURREAL_USERNAME}"
password = "${SURREAL_PASSWORD}"
pool_size = 20

[vector_store]
store_type = "SurrealDB"
dimension = 1536

[cache]
enabled = true
cache_type = "Memory"  # Or "Redis" for distributed caching
size = 10000
ttl = 3600
```

## Advanced Features

The system automatically provides:

- **Vector Similarity Search**: Find semantically related memories
- **Graph Traversal**: Navigate relationships between memories
- **Temporal Queries**: Search memories by time
- **Importance Scoring**: Memories are ranked by relevance
- **Auto-cleanup**: Old, unused memories fade over time

## Ollama Integration

For local LLM operations with ollama:

### Required Setup
1. Install ollama: `curl https://ollama.ai/install.sh | sh`
2. Start ollama: `ollama serve`
3. Pull models: `ollama pull llama2` and `ollama pull nomic-embed-text`

### Configuration
```rust
use surreal_memory::{MemoryConfig, utils::config::*};

let config = MemoryConfig {
    database: DatabaseConfig {
        db_type: DatabaseType::SurrealDB,
        connection_string: "file://./data/memory.db".to_string(),
        namespace: "production".to_string(),
        database: "agent_memory".to_string(),
        username: None,
        password: None,
        pool_size: Some(10),
        options: None,
    },
    vector_store: VectorStoreConfig {
        store_type: VectorStoreType::SurrealDB,
        embedding_model: EmbeddingModelConfig {
            model_type: EmbeddingModelType::Custom,
            model_name: "nomic-embed-text".to_string(),
            api_key: None,
            api_base: Some("http://localhost:11434/api/embeddings".to_string()),
            options: None,
        },
        dimension: 768,
        connection_string: None,
        api_key: None,
        options: None,
    },
    llm: LLMConfig {
        provider: LLMProvider::Custom,
        model_name: "llama2".to_string(),
        api_key: None,
        api_base: Some("http://localhost:11434/api/generate".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(2048),
        options: None,
    },
    cache: CacheConfig {
        enabled: true,
        cache_type: CacheType::Memory,
        size: Some(10000),
        ttl: Some(3600),
        options: None,
    },
    logging: LoggingConfig {
        level: LogLevel::Info,
        file: Some("./logs/memory.log".to_string()),
        console: true,
        options: None,
    },
    api: None,
};
```

## Troubleshooting

### "Failed to connect to database"
- For file storage: Ensure directory exists for `file://./data/memory.db`
- For SurrealDB server: Check SurrealDB is running: `surreal start --log debug`
- Verify connection string format is correct

### "Invalid API key"
- For OpenAI/Anthropic: Ensure API key is set and has credits
- For ollama: Ensure ollama server is running on localhost:11434

### "ollama connection failed"
- Check ollama is running: `ollama list`
- Verify models are pulled: `ollama pull llama2` and `ollama pull nomic-embed-text`
- Test ollama API: `curl http://localhost:11434/api/tags`

### "Memory not found"
- Memories are indexed asynchronously - allow a moment after creation
- Check the memory ID is correct
- Verify namespace/database match

## Performance Tips

1. **Batch Operations**: Create multiple memories in one transaction
2. **Use Appropriate Types**: Semantic for facts, Episodic for events
3. **Let SurrealDB Handle It**: Don't try to manage vectors manually
4. **Cache Frequently Accessed**: The cache layer significantly improves performance

## What You Don't Need to Worry About

The system handles these automatically:
- Vector index management
- Graph relationship creation
- Embedding generation and updates
- Query optimization
- Concurrent access
- Data consistency

Just focus on storing and retrieving memories - the system does the rest!