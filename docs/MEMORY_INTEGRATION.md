# Memory System Integration Guide for MCP API

This guide explains how to integrate the `surreal_memory` crate into the MCP API server to provide sophisticated memory capabilities for AI agents.

## Overview

The `surreal_memory` crate provides:
- **SurrealDB-based storage** with native vector search and graph relationships
- **File-based storage** using kv-surrealkv (no external database server needed)
- **Local LLM integration** with ollama for embeddings and text generation
- **Complete MemoryManager API** for all memory operations

## Dependencies

The memory system is already added to the MCP project:

```toml
# Cargo.toml
surreal_memory = { path = "../memory", default-features = false, features = ["surreal-vector"] }
```

## Configuration

### Complete MemoryConfig Setup

```rust
use surreal_memory::{
    MemoryConfig, initialize,
    utils::config::{
        DatabaseConfig, DatabaseType,
        VectorStoreConfig, VectorStoreType, 
        EmbeddingModelConfig, EmbeddingModelType,
        LLMConfig, LLMProvider,
        CacheConfig, CacheType,
        LoggingConfig, LogLevel
    }
};

async fn create_memory_config() -> MemoryConfig {
    MemoryConfig {
        database: DatabaseConfig {
            db_type: DatabaseType::SurrealDB,
            connection_string: "file://./data/mcp_memory.db".to_string(),
            namespace: "mcp".to_string(),
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
            file: Some("./logs/mcp_memory.log".to_string()),
            console: true,
            options: None,
        },
        
        api: None,
    }
}
```

### Memory Manager Initialization

```rust
use surreal_memory::{MemoryManager, SurrealDBMemoryManager};

// Initialize the memory system
let config = create_memory_config().await;
let memory_manager = initialize(&config).await?;
```

## MCP Tool Implementation

Following the mem0 pattern, implement these three MCP tools:

### 1. save_memory Tool

```rust
use surreal_memory::memory::{MemoryNode, MemoryType};
use serde_json::json;

async fn handle_save_memory(
    memory_manager: &SurrealDBMemoryManager,
    text: String,
    memory_type: Option<String>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mem_type = match memory_type.as_deref() {
        Some("episodic") => MemoryType::Episodic,
        Some("procedural") => MemoryType::Procedural,
        Some(custom) => MemoryType::Custom(custom.to_string()),
        _ => MemoryType::Semantic,
    };
    
    let memory = MemoryNode::new(text.clone(), mem_type);
    let stored = memory_manager.create_memory(memory).await?;
    
    Ok(json!({
        "success": true,
        "memory_id": stored.id,
        "content": stored.content,
        "type": stored.memory_type,
        "created_at": stored.created_at
    }))
}
```

### 2. search_memories Tool

```rust
use futures::StreamExt;

async fn handle_search_memories(
    memory_manager: &SurrealDBMemoryManager,
    query: String,
    limit: Option<usize>,
    memory_type: Option<String>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let limit = limit.unwrap_or(5);
    let mut results = Vec::new();
    
    let mut stream = if let Some(mem_type) = memory_type {
        let parsed_type = match mem_type.as_str() {
            "episodic" => MemoryType::Episodic,
            "procedural" => MemoryType::Procedural,
            "semantic" => MemoryType::Semantic,
            custom => MemoryType::Custom(custom.to_string()),
        };
        memory_manager.query_by_type(parsed_type)
    } else {
        memory_manager.search_by_content(&query)
    };
    
    while let Some(memory_result) = stream.next().await {
        if results.len() >= limit { break; }
        
        match memory_result {
            Ok(memory) => {
                results.push(json!({
                    "id": memory.id,
                    "content": memory.content,
                    "type": memory.memory_type,
                    "created_at": memory.created_at,
                    "importance": memory.metadata.importance
                }));
            }
            Err(e) => tracing::warn!("Error retrieving memory: {}", e),
        }
    }
    
    Ok(json!({
        "success": true,
        "query": query,
        "results": results,
        "count": results.len()
    }))
}
```

### 3. get_all_memories Tool

```rust
async fn handle_get_all_memories(
    memory_manager: &SurrealDBMemoryManager,
    limit: Option<usize>,
    memory_type: Option<String>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let limit = limit.unwrap_or(100);
    let mut all_memories = Vec::new();
    
    let types_to_query = if let Some(mem_type) = memory_type {
        vec![match mem_type.as_str() {
            "episodic" => MemoryType::Episodic,
            "procedural" => MemoryType::Procedural,
            "semantic" => MemoryType::Semantic,
            custom => MemoryType::Custom(custom.to_string()),
        }]
    } else {
        vec![MemoryType::Semantic, MemoryType::Episodic, MemoryType::Procedural]
    };
    
    for memory_type in types_to_query {
        let mut stream = memory_manager.query_by_type(memory_type);
        
        while let Some(memory_result) = stream.next().await {
            if all_memories.len() >= limit { break; }
            
            match memory_result {
                Ok(memory) => {
                    all_memories.push(json!({
                        "id": memory.id,
                        "content": memory.content,
                        "type": memory.memory_type,
                        "created_at": memory.created_at,
                        "updated_at": memory.updated_at,
                        "importance": memory.metadata.importance
                    }));
                }
                Err(e) => tracing::warn!("Error retrieving memory: {}", e),
            }
        }
    }
    
    Ok(json!({
        "success": true,
        "memories": all_memories,
        "count": all_memories.len()
    }))
}
```

## Integration with MCP Router

Add the memory tools to your MCP tool registry:

```rust
// In src/tool/mod.rs or wherever tools are registered

use crate::memory_handlers::{handle_save_memory, handle_search_memories, handle_get_all_memories};

pub fn register_memory_tools(memory_manager: SurrealDBMemoryManager) {
    // Register save_memory tool
    register_tool("save_memory", move |args| {
        let manager = memory_manager.clone();
        async move {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let memory_type = args.get("type").and_then(|v| v.as_str()).map(|s| s.to_string());
            
            handle_save_memory(&manager, text, memory_type).await
        }
    });
    
    // Register search_memories tool
    register_tool("search_memories", move |args| {
        let manager = memory_manager.clone();
        async move {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let limit = args.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);
            let memory_type = args.get("type").and_then(|v| v.as_str()).map(|s| s.to_string());
            
            handle_search_memories(&manager, query, limit, memory_type).await
        }
    });
    
    // Register get_all_memories tool
    register_tool("get_all_memories", move |args| {
        let manager = memory_manager.clone();
        async move {
            let limit = args.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);
            let memory_type = args.get("type").and_then(|v| v.as_str()).map(|s| s.to_string());
            
            handle_get_all_memories(&manager, limit, memory_type).await
        }
    });
}
```

## Setup Requirements

### 1. Ollama Setup
```bash
# Install ollama
curl https://ollama.ai/install.sh | sh

# Start ollama server
ollama serve

# Pull required models
ollama pull llama2
ollama pull nomic-embed-text

# Verify setup
curl http://localhost:11434/api/tags
```

### 2. Data Directory
```bash
# Create data directory for memory storage
mkdir -p ./data

# The memory system will create the database file automatically
# File will be created at: ./data/mcp_memory.db
```

### 3. Logs Directory
```bash
# Create logs directory
mkdir -p ./logs

# Memory system will write logs to: ./logs/mcp_memory.log
```

## Usage Examples

### Save a Memory
```json
{
  "method": "tools/call",
  "params": {
    "name": "save_memory",
    "arguments": {
      "text": "The user prefers dark mode in their IDE",
      "type": "semantic"
    }
  }
}
```

### Search Memories
```json
{
  "method": "tools/call",
  "params": {
    "name": "search_memories",
    "arguments": {
      "query": "user preferences",
      "limit": 5
    }
  }
}
```

### Get All Memories
```json
{
  "method": "tools/call",
  "params": {
    "name": "get_all_memories",
    "arguments": {
      "limit": 10,
      "type": "semantic"
    }
  }
}
```

## Key Benefits

1. **No External Dependencies**: File-based storage with kv-surrealkv
2. **Local AI Models**: ollama provides privacy and offline capabilities  
3. **Native Vector Search**: SurrealDB handles all vector operations efficiently
4. **Streaming Results**: Memory queries return streams for optimal performance
5. **Rich Memory Types**: Support for semantic, episodic, and procedural memories
6. **Production Ready**: Complete error handling and logging

## Next Steps

1. Initialize memory system in main.rs
2. Add memory tool handlers to router
3. Create data and logs directories
4. Set up ollama with required models
5. Test memory operations via MCP tools

This integration provides a complete, production-ready memory system for AI agents using local storage and models.