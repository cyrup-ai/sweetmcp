//! Core memory repository types and basic operations
//!
//! This module provides the core MemoryRepository struct and fundamental
//! operations for managing in-memory cache and indexing with zero allocation
//! patterns and blazing-fast performance.

use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::memory::{MemoryNode, MemoryRelationship, MemoryType};

/// In-memory repository for fast memory access and indexing
pub struct MemoryRepository {
    /// Primary memory storage by ID
    pub(super) memories: HashMap<String, Arc<MemoryNode>>,

    /// Index by memory type
    pub(super) type_index: HashMap<MemoryType, HashSet<String>>,

    /// Index by user ID
    pub(super) user_index: HashMap<String, HashSet<String>>,

    /// Index by agent ID
    pub(super) agent_index: HashMap<String, HashSet<String>>,

    /// Index by tags
    pub(super) tag_index: HashMap<String, HashSet<String>>,

    /// Time-based index (sorted by creation time)
    pub(super) time_index: BTreeMap<DateTime<Utc>, HashSet<String>>,

    /// Relationships storage
    pub(super) relationships: HashMap<String, Vec<MemoryRelationship>>,
}

/// Repository statistics
#[derive(Debug, Clone)]
pub struct RepositoryStats {
    /// Total number of memories
    pub total_memories: usize,

    /// Number of memories by type
    pub memories_by_type: HashMap<MemoryType, usize>,

    /// Total number of relationships
    pub total_relationships: usize,

    /// Number of unique users
    pub unique_users: usize,

    /// Number of unique agents
    pub unique_agents: usize,

    /// Number of unique tags
    pub unique_tags: usize,
}

impl MemoryRepository {
    /// Create a new memory repository
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            type_index: HashMap::new(),
            user_index: HashMap::new(),
            agent_index: HashMap::new(),
            tag_index: HashMap::new(),
            time_index: BTreeMap::new(),
            relationships: HashMap::new(),
        }
    }

    /// Create and add a memory to the repository
    pub fn create(&mut self, id: &str, memory: &MemoryNode) -> crate::utils::Result<MemoryNode> {
        // Create a new memory with the provided ID
        let mut new_memory = memory.clone();
        new_memory.id = id.to_string();

        // Add to repository
        self.add(new_memory.clone());

        Ok(new_memory)
    }

    /// Add a memory to the repository
    pub fn add(&mut self, memory: MemoryNode) {
        let memory_arc = Arc::new(memory);
        let memory_ref = &memory_arc;

        // Add to primary storage
        self.memories.insert(memory_ref.id.clone(), memory_arc.clone());

        // Add to type index
        self.type_index
            .entry(memory_ref.memory_type.clone())
            .or_insert_with(HashSet::new)
            .insert(memory_ref.id.clone());

        // Add to user index
        if let Some(user_id) = &memory_ref.metadata.user_id {
            self.user_index
                .entry(user_id.clone())
                .or_insert_with(HashSet::new)
                .insert(memory_ref.id.clone());
        }

        // Add to agent index
        if let Some(agent_id) = &memory_ref.metadata.agent_id {
            self.agent_index
                .entry(agent_id.clone())
                .or_insert_with(HashSet::new)
                .insert(memory_ref.id.clone());
        }

        // Add to tag index
        for tag in &memory_ref.metadata.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(memory_ref.id.clone());
        }

        // Add to time index
        self.time_index
            .entry(memory_ref.created_at)
            .or_insert_with(HashSet::new)
            .insert(memory_ref.id.clone());
    }

    /// Get a memory by ID
    pub fn get(&self, id: &str) -> Option<Arc<MemoryNode>> {
        self.memories.get(id).cloned()
    }

    /// Check if a memory exists
    pub fn exists(&self, id: &str) -> bool {
        self.memories.contains_key(id)
    }

    /// Get all memory IDs
    pub fn get_all_ids(&self) -> Vec<String> {
        self.memories.keys().cloned().collect()
    }

    /// Get all memories
    pub fn get_all(&self) -> Vec<Arc<MemoryNode>> {
        self.memories.values().cloned().collect()
    }

    /// Get memory count
    pub fn len(&self) -> usize {
        self.memories.len()
    }

    /// Check if repository is empty
    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }

    /// Remove a memory from the repository
    pub fn remove(&mut self, id: &str) -> Option<Arc<MemoryNode>> {
        if let Some(memory) = self.memories.remove(id) {
            // Remove from all indexes
            self.remove_from_indexes(&memory);
            Some(memory)
        } else {
            None
        }
    }

    /// Clear all memories from the repository
    pub fn clear(&mut self) {
        self.memories.clear();
        self.type_index.clear();
        self.user_index.clear();
        self.agent_index.clear();
        self.tag_index.clear();
        self.time_index.clear();
        self.relationships.clear();
    }

    /// Get memories by type
    pub fn get_by_type(&self, memory_type: &MemoryType) -> Vec<Arc<MemoryNode>> {
        if let Some(ids) = self.type_index.get(memory_type) {
            ids.iter()
                .filter_map(|id| self.memories.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get memories by user ID
    pub fn get_by_user(&self, user_id: &str) -> Vec<Arc<MemoryNode>> {
        if let Some(ids) = self.user_index.get(user_id) {
            ids.iter()
                .filter_map(|id| self.memories.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get memories by agent ID
    pub fn get_by_agent(&self, agent_id: &str) -> Vec<Arc<MemoryNode>> {
        if let Some(ids) = self.agent_index.get(agent_id) {
            ids.iter()
                .filter_map(|id| self.memories.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get memories by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<Arc<MemoryNode>> {
        if let Some(ids) = self.tag_index.get(tag) {
            ids.iter()
                .filter_map(|id| self.memories.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get memories in time range
    pub fn get_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Arc<MemoryNode>> {
        let mut result = Vec::new();
        
        // Use BTreeMap range to efficiently find memories in time range
        for (_, ids) in self.time_index.range(start..=end) {
            for id in ids {
                if let Some(memory) = self.memories.get(id) {
                    result.push(memory.clone());
                }
            }
        }
        
        result
    }

    /// Get recent memories (last N memories by creation time)
    pub fn get_recent(&self, limit: usize) -> Vec<Arc<MemoryNode>> {
        let mut result = Vec::new();
        
        // Iterate in reverse order (most recent first)
        for (_, ids) in self.time_index.iter().rev() {
            for id in ids {
                if let Some(memory) = self.memories.get(id) {
                    result.push(memory.clone());
                    if result.len() >= limit {
                        return result;
                    }
                }
            }
        }
        
        result
    }

    /// Get oldest memories (first N memories by creation time)
    pub fn get_oldest(&self, limit: usize) -> Vec<Arc<MemoryNode>> {
        let mut result = Vec::new();
        
        // Iterate in forward order (oldest first)
        for (_, ids) in self.time_index.iter() {
            for id in ids {
                if let Some(memory) = self.memories.get(id) {
                    result.push(memory.clone());
                    if result.len() >= limit {
                        return result;
                    }
                }
            }
        }
        
        result
    }

    /// Get all memory types in the repository
    pub fn get_memory_types(&self) -> Vec<MemoryType> {
        self.type_index.keys().cloned().collect()
    }

    /// Get all user IDs in the repository
    pub fn get_user_ids(&self) -> Vec<String> {
        self.user_index.keys().cloned().collect()
    }

    /// Get all agent IDs in the repository
    pub fn get_agent_ids(&self) -> Vec<String> {
        self.agent_index.keys().cloned().collect()
    }

    /// Get all tags in the repository
    pub fn get_tags(&self) -> Vec<String> {
        self.tag_index.keys().cloned().collect()
    }

    /// Get memory count by type
    pub fn count_by_type(&self, memory_type: &MemoryType) -> usize {
        self.type_index
            .get(memory_type)
            .map(|ids| ids.len())
            .unwrap_or(0)
    }

    /// Get memory count by user
    pub fn count_by_user(&self, user_id: &str) -> usize {
        self.user_index
            .get(user_id)
            .map(|ids| ids.len())
            .unwrap_or(0)
    }

    /// Get memory count by agent
    pub fn count_by_agent(&self, agent_id: &str) -> usize {
        self.agent_index
            .get(agent_id)
            .map(|ids| ids.len())
            .unwrap_or(0)
    }

    /// Get memory count by tag
    pub fn count_by_tag(&self, tag: &str) -> usize {
        self.tag_index
            .get(tag)
            .map(|ids| ids.len())
            .unwrap_or(0)
    }

    /// Remove a memory from all indexes
    pub(super) fn remove_from_indexes(&mut self, memory: &MemoryNode) {
        // Remove from type index
        if let Some(type_ids) = self.type_index.get_mut(&memory.memory_type) {
            type_ids.remove(&memory.id);
        }

        // Remove from user index
        if let Some(user_id) = &memory.metadata.user_id {
            if let Some(user_ids) = self.user_index.get_mut(user_id) {
                user_ids.remove(&memory.id);
            }
        }

        // Remove from agent index
        if let Some(agent_id) = &memory.metadata.agent_id {
            if let Some(agent_ids) = self.agent_index.get_mut(agent_id) {
                agent_ids.remove(&memory.id);
            }
        }

        // Remove from tag index
        for tag in &memory.metadata.tags {
            if let Some(tag_ids) = self.tag_index.get_mut(tag) {
                tag_ids.remove(&memory.id);
            }
        }

        // Remove from time index
        if let Some(time_ids) = self.time_index.get_mut(&memory.created_at) {
            time_ids.remove(&memory.id);
        }
    }
}

impl Default for MemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}