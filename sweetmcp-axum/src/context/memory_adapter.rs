//! Memory system adapter for context API
//! This module provides integration between MCP context API and the sophisticated memory system

use anyhow::{Result, anyhow};
use futures::StreamExt;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

use sweetmcp_memory::{MemoryConfig, MemoryManager, MemoryNode, MemoryType};

/// Adapter that bridges MCP context API with the memory system
pub struct MemoryContextAdapter {
    memory_manager: Arc<dyn MemoryManager>,
    subscriptions: Arc<RwLock<Vec<String>>>,
}

impl MemoryContextAdapter {
    /// Create a new memory context adapter with the given configuration
    pub async fn new(config: MemoryConfig) -> Result<Self> {
        // Initialize the memory system
        let manager = sweetmcp_memory::initialize(&config)
            .await
            .map_err(|e| anyhow!("Failed to initialize memory system: {}", e))?;

        Ok(Self {
            memory_manager: Arc::new(manager),
            subscriptions: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Store a context value in the memory system
    pub async fn store_context(&self, key: String, value: Value) -> Result<()> {
        // Create a memory node for the context
        let content = serde_json::to_string(&value)?;
        let mut memory = MemoryNode::new(content, MemoryType::Semantic);

        // Use the key as the memory ID for easy retrieval
        memory.id = format!("context:{}", key);

        // Add metadata to identify this as a context entry
        memory = memory.with_custom_metadata("type".to_string(), "context".to_string());
        memory = memory.with_custom_metadata("key".to_string(), key.clone());

        // Store in memory system
        self.memory_manager
            .create_memory(memory)
            .await
            .map_err(|e| anyhow!("Failed to store context: {}", e))?;

        Ok(())
    }

    /// Retrieve a context value from the memory system
    pub async fn get_context(&self, key: &str) -> Result<Option<Value>> {
        let memory_id = format!("context:{}", key);

        match self.memory_manager.get_memory(&memory_id).await {
            Ok(Some(memory)) => {
                // Parse the content back to JSON value
                let value = serde_json::from_str(&memory.content)?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow!("Failed to get context: {}", e)),
        }
    }

    /// Add a subscription
    pub async fn add_subscription(&self, uri: String) -> Result<()> {
        let mut subs = self.subscriptions.write().await;
        if !subs.contains(&uri) {
            subs.push(uri);
        }
        Ok(())
    }

    /// Remove a subscription
    pub async fn remove_subscription(&self, uri: &str) -> Result<()> {
        let mut subs = self.subscriptions.write().await;
        subs.retain(|s| s != uri);
        Ok(())
    }

    /// Get all active subscriptions
    pub async fn get_subscriptions(&self) -> Vec<String> {
        self.subscriptions.read().await.clone()
    }

    /// Search for context entries by pattern
    pub async fn search_contexts(&self, pattern: &str) -> Result<Vec<(String, Value)>> {
        let mut results = Vec::new();

        // Search for memories containing the pattern
        let mut stream = self.memory_manager.search_by_content(pattern);

        while let Some(result) = stream.next().await {
            match result {
                Ok(memory) => {
                    // Check if this is a context entry
                    if memory.id.starts_with("context:") {
                        if let Ok(value) = serde_json::from_str(&memory.content) {
                            let key = memory.id.strip_prefix("context:").unwrap_or(&memory.id);
                            results.push((key.to_string(), value));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Error searching contexts: {}", e);
                }
            }
        }

        Ok(results)
    }
}
