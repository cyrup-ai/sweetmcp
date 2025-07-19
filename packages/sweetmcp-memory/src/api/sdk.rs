// src/api/sdk.rs
//! SDK for interacting with mem0 from Rust applications.

use std::sync::Arc;
use serde_json::Value;

use crate::memory::memory_manager::MemoryManager;
use crate::vector::vector_search::{VectorSearch, SearchOptions};
use crate::llm::completion::CompletionService;
use crate::llm::content_analyzer::LLMContentAnalyzer;
use crate::utils::error::Result;
use crate::schema::memory_schema::Memory;
use crate::schema::relationship_schema::Relationship;

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
        self.memory_manager.create_memory(content, metadata, embedding).await
    }
    
    /// Get a memory by ID
    pub async fn get_memory(&self, id: &str) -> Result<Memory> {
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
        self.memory_manager.update_memory(id, content, metadata, embedding).await
    }
    
    /// Delete a memory
    pub async fn delete_memory(&self, id: &str) -> Result<()> {
        self.memory_manager.delete_memory(id).await
    }
    
    /// List memories
    pub async fn list_memories(
        &self,
        limit: usize,
        offset: usize,
        filter: Option<&str>,
    ) -> Result<Vec<Memory>> {
        self.memory_manager.list_memories(limit, offset, filter).await
    }
    
    /// Search memories by text
    pub async fn search_memories(
        &self,
        query: &str,
        limit: Option<usize>,
        min_similarity: Option<f32>,
    ) -> Result<Vec<Memory>> {
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
            if let Ok(memory) = self.memory_manager.get_memory(&result.id).await {
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
        self.memory_manager.create_relationship(source_id, target_id, relationship_type, metadata).await
    }
    
    /// Get a relationship by ID
    pub async fn get_relationship(&self, id: &str) -> Result<Relationship> {
        self.memory_manager.get_relationship(id).await
    }
    
    /// Delete a relationship
    pub async fn delete_relationship(&self, id: &str) -> Result<()> {
        self.memory_manager.delete_relationship(id).await
    }
    
    /// List relationships
    pub async fn list_relationships(
        &self,
        limit: usize,
        offset: usize,
        filter: Option<&str>,
    ) -> Result<Vec<Relationship>> {
        self.memory_manager.list_relationships(limit, offset, filter).await
    }
    
    /// Get relationships for a memory
    pub async fn get_memory_relationships(
        &self,
        memory_id: &str,
        relationship_type: Option<&str>,
        direction: Option<&str>,
    ) -> Result<Vec<Relationship>> {
        self.memory_manager.get_memory_relationships(memory_id, relationship_type, direction).await
    }
    
    /// Analyze content
    pub async fn analyze_content(&self, content: &str) -> Result<crate::llm::content_analyzer::ContentAnalysis> {
        self.content_analyzer.analyze_content(content).await
    }
    
    /// Analyze relationship between contents
    pub async fn analyze_relationship(
        &self,
        content1: &str,
        content2: &str,
    ) -> Result<crate::llm::content_analyzer::RelationshipAnalysis> {
        self.content_analyzer.analyze_relationship(content1, content2).await
    }
    
    /// Generate completion
    pub async fn generate_completion(
        &self,
        messages: Vec<std::collections::HashMap<String, String>>,
    ) -> Result<String> {
        self.completion_service.generate_completion(messages).await
    }
    
    /// Generate completion with tools
    pub async fn generate_completion_with_tools(
        &self,
        messages: Vec<std::collections::HashMap<String, String>>,
        tools: Vec<std::collections::HashMap<String, String>>,
    ) -> Result<std::collections::HashMap<String, String>> {
        self.completion_service.generate_completion_with_tools(messages, tools).await
    }
    
    /// Generate JSON completion
    pub async fn generate_json_completion(
        &self,
        messages: Vec<std::collections::HashMap<String, String>>,
    ) -> Result<String> {
        self.completion_service.generate_json_completion(messages).await
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
            .ok_or_else(|| crate::utils::error::MemoryError::InvalidArgument(
                "Memory manager is required".to_string()
            ))?;
        
        let vector_search = self.vector_search
            .ok_or_else(|| crate::utils::error::MemoryError::InvalidArgument(
                "Vector search is required".to_string()
            ))?;
        
        let completion_service = self.completion_service
            .ok_or_else(|| crate::utils::error::MemoryError::InvalidArgument(
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


