// src/api/sdk.rs
//! SDK for interacting with mem0 from Rust applications.

use std::sync::Arc;
use serde_json::Value;

use crate::memory::memory_manager::MemoryManager;
use crate::memory::{MemoryNode, MemoryType};
use crate::vector::vector_search::{VectorSearch, SearchOptions};
use crate::llm::completion::CompletionService;
use crate::llm::content_analyzer::LLMContentAnalyzer;
use crate::utils::error::Result;
// Removed less decomposed Memory schema - using MemoryNode instead
use crate::memory::memory_relationship::MemoryRelationship;

/// Memory SDK for interacting with mem0
pub struct MemorySDK {
    /// Memory manager
    memory_manager: Arc<dyn MemoryManager>,
    /// Vector search
    vector_search: Arc<VectorSearch>,
    /// Completion service
    completion_service: Arc<CompletionService>,
    /// Content analyzer
    content_analyzer: LLMContentAnalyzer,
}

impl MemorySDK {
    /// Create a new MemorySDK
    pub fn new(
        memory_manager: Arc<dyn MemoryManager>,
        vector_search: Arc<VectorSearch>,
        completion_service: Arc<CompletionService>,
    ) -> Self {
        let content_analyzer = LLMContentAnalyzer::new(completion_service.provider());
        
        Self {
            memory_manager,
            vector_search,
            completion_service,
            content_analyzer,
        }
    }
    
    /// Create a memory
    pub async fn create_memory(
        &self,
        content: &str,
        metadata: Option<Value>,
        embedding: Option<&[f32]>,
    ) -> Result<String> {
        let mut memory_node = MemoryNode::new(content.to_string(), MemoryType::Fact);
        if let Some(meta) = metadata {
            // Convert Value to MemoryMetadata through custom field
            if let serde_json::Value::Object(ref map) = meta {
                for (key, value) in map {
                    memory_node = memory_node.with_custom_metadata(key.clone(), value.to_string());
                }
            }
        }
        if let Some(emb) = embedding {
            memory_node.embedding = Some(emb.to_vec());
        }
        let result = self.memory_manager.create_memory(memory_node).await?;
        Ok(result.id)
    }
    
    /// Get a memory by ID
    pub async fn get_memory(&self, id: &str) -> Result<Option<MemoryNode>> {
        self.memory_manager.get_memory(id).await
    }
    
    /// Update a memory
    pub async fn update_memory(
        &self,
        id: &str,
        content: Option<&str>,
        metadata: Option<Value>,
        embedding: Option<&[f32]>,
    ) -> Result<()> {
        // First get the existing memory
        let mut memory_node = self.memory_manager.get_memory(id).await?
            .ok_or_else(|| crate::utils::error::Error::NotFound(format!("Memory with id {} not found", id)))?;
        
        // Update fields if provided
        if let Some(new_content) = content {
            memory_node.content = new_content.to_string();
        }
        if let Some(new_metadata) = metadata {
            // Convert Value to MemoryMetadata through custom field
            if let serde_json::Value::Object(ref map) = new_metadata {
                for (key, value) in map {
                    memory_node = memory_node.with_custom_metadata(key.clone(), value.to_string());
                }
            }
        }
        if let Some(new_embedding) = embedding {
            memory_node.embedding = Some(new_embedding.to_vec());
        }
        
        self.memory_manager.update_memory(memory_node).await?;
        Ok(())
    }
    
    /// Delete a memory  
    pub async fn delete_memory(&self, id: &str) -> Result<()> {
        let _deleted = self.memory_manager.delete_memory(id).await?;
        Ok(())
    }
    
    /// List memories
    pub async fn list_memories(
        &self,
        limit: usize,
        offset: usize,
        filter: Option<&str>,
    ) -> Result<Vec<MemoryNode>> {
        self.memory_manager.list_memories(limit, offset, filter).await
    }
    
    /// Search memories by text
    pub async fn search_memories(
        &self,
        query: &str,
        limit: Option<usize>,
        min_similarity: Option<f32>,
    ) -> Result<Vec<MemoryNode>> {
        let search_options = SearchOptions {
            limit,
            min_similarity,
            filters: None,
            include_vectors: Some(false),
            include_metadata: Some(true),
        };
        
        let search_results = self.vector_search.search_by_text(query, Some(search_options)).await?;
        
        let mut memories = Vec::new();
        for result in search_results {
            if let Ok(Some(memory)) = self.memory_manager.get_memory(&result.id).await {
                memories.push(memory);
            }
        }
        
        Ok(memories)
    }
    
    /// Create a relationship between memories
    pub async fn create_relationship(
        &self,
        source_id: &str,
        target_id: &str,
        relationship_type: &str,
        metadata: Option<Value>,
    ) -> Result<String> {
        // Create MemoryRelationship object for more decomposed interface
        let relationship = crate::memory::MemoryRelationship::new(
            source_id.to_string(),
            target_id.to_string(),
            relationship_type.to_string(),
        );
        let result = self.memory_manager.create_relationship(relationship).await?;
        Ok(result.id)
    }
    
    /// Get a relationship by ID
    pub async fn get_relationship(&self, id: &str) -> Result<MemoryRelationship> {
        self.memory_manager.get_relationship(id).await
    }
    
    /// Delete a relationship
    pub async fn delete_relationship(&self, id: &str) -> Result<()> {
        let _deleted = self.memory_manager.delete_relationship(id).await?;
        Ok(())
    }
    
    /// List relationships
    pub async fn list_relationships(
        &self,
        limit: usize,
        offset: usize,
        filter: Option<&str>,
    ) -> Result<Vec<MemoryRelationship>> {
        self.memory_manager.list_relationships(limit, offset, filter).await
    }
    
    /// Get relationships for a memory
    pub async fn get_memory_relationships(
        &self,
        memory_id: &str,
        relationship_type: Option<&str>,
        direction: Option<&str>,
    ) -> Result<Vec<MemoryRelationship>> {
        self.memory_manager.get_memory_relationships(memory_id, relationship_type, direction).await
    }
    
    /// Analyze content
    pub async fn analyze_content(&self, content: &str) -> Result<crate::llm::content_analyzer::ContentAnalysis> {
        self.content_analyzer.analyze_content(content).await
            .map_err(|e| crate::utils::error::Error::LLM(e.to_string()))
    }
    
    /// Analyze relationship between contents
    pub async fn analyze_relationship(
        &self,
        content1: &str,
        content2: &str,
    ) -> Result<crate::llm::content_analyzer::RelationshipAnalysis> {
        self.content_analyzer.analyze_relationship(content1, content2).await
            .map_err(|e| crate::utils::error::Error::LLM(e.to_string()))
    }
    
    /// Generate completion
    pub async fn generate_completion(
        &self,
        messages: Vec<std::collections::HashMap<String, String>>,
    ) -> Result<String> {
        self.completion_service.generate_completion(messages).await
            .map_err(|e| crate::utils::error::Error::LLM(e.to_string()))
    }
    
    /// Generate completion with tools
    pub async fn generate_completion_with_tools(
        &self,
        messages: Vec<std::collections::HashMap<String, String>>,
        tools: Vec<std::collections::HashMap<String, String>>,
    ) -> Result<std::collections::HashMap<String, String>> {
        self.completion_service.generate_completion_with_tools(messages, tools).await
            .map_err(|e| crate::utils::error::Error::LLM(e.to_string()))
    }
    
    /// Generate JSON completion
    pub async fn generate_json_completion(
        &self,
        messages: Vec<std::collections::HashMap<String, String>>,
    ) -> Result<String> {
        self.completion_service.generate_json_completion(messages).await
            .map_err(|e| crate::utils::error::Error::LLM(e.to_string()))
    }
    
    /// Get the memory manager
    pub fn memory_manager(&self) -> Arc<dyn MemoryManager> {
        Arc::clone(&self.memory_manager)
    }
    
    /// Get the vector search
    pub fn vector_search(&self) -> Arc<VectorSearch> {
        Arc::clone(&self.vector_search)
    }
    
    /// Get the completion service
    pub fn completion_service(&self) -> Arc<CompletionService> {
        Arc::clone(&self.completion_service)
    }
    
    /// Get the content analyzer
    pub fn content_analyzer(&self) -> &LLMContentAnalyzer {
        &self.content_analyzer
    }
}

/// SDK builder for configuring and creating a MemorySDK
pub struct MemorySDKBuilder {
    /// Memory manager
    memory_manager: Option<Arc<dyn MemoryManager>>,
    /// Vector search
    vector_search: Option<Arc<VectorSearch>>,
    /// Completion service
    completion_service: Option<Arc<CompletionService>>,
}

impl MemorySDKBuilder {
    /// Create a new MemorySDKBuilder
    pub fn new() -> Self {
        Self {
            memory_manager: None,
            vector_search: None,
            completion_service: None,
        }
    }
    
    /// Set memory manager
    pub fn with_memory_manager(mut self, memory_manager: Arc<dyn MemoryManager>) -> Self {
        self.memory_manager = Some(memory_manager);
        self
    }
    
    /// Set vector search
    pub fn with_vector_search(mut self, vector_search: Arc<VectorSearch>) -> Self {
        self.vector_search = Some(vector_search);
        self
    }
    
    /// Set completion service
    pub fn with_completion_service(mut self, completion_service: Arc<CompletionService>) -> Self {
        self.completion_service = Some(completion_service);
        self
    }
    
    /// Build the MemorySDK
    pub fn build(self) -> Result<MemorySDK> {
        let memory_manager = self.memory_manager
            .ok_or_else(|| crate::utils::error::Error::InvalidInput(
                "Memory manager is required".to_string()
            ))?;
        
        let vector_search = self.vector_search
            .ok_or_else(|| crate::utils::error::Error::InvalidInput(
                "Vector search is required".to_string()
            ))?;
        
        let completion_service = self.completion_service
            .ok_or_else(|| crate::utils::error::Error::InvalidInput(
                "Completion service is required".to_string()
            ))?;
        
        Ok(MemorySDK::new(memory_manager, vector_search, completion_service))
    }
}

impl Default for MemorySDKBuilder {
    fn default() -> Self {
        Self::new()
    }
}


