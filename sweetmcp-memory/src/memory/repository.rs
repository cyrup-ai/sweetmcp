//! Memory repository for managing in-memory cache and indexing

use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::memory::{MemoryNode, MemoryRelationship, MemoryType, filter::MemoryFilter};
use crate::utils::Result;

/// In-memory repository for fast memory access and indexing
pub struct MemoryRepository {
    /// Primary memory storage by ID
    memories: HashMap<String, Arc<MemoryNode>>,

    /// Index by memory type
    type_index: HashMap<MemoryType, HashSet<String>>,

    /// Index by user ID
    user_index: HashMap<String, HashSet<String>>,

    /// Index by agent ID
    agent_index: HashMap<String, HashSet<String>>,

    /// Index by tags
    tag_index: HashMap<String, HashSet<String>>,

    /// Time-based index (sorted by creation time)
    time_index: BTreeMap<DateTime<Utc>, HashSet<String>>,

    /// Relationships storage
    relationships: HashMap<String, Vec<MemoryRelationship>>,
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
        self.memories
            .insert(memory_ref.id.clone(), memory_arc.clone());

        // Update type index
        self.type_index
            .entry(memory_ref.memory_type.clone())
            .or_insert_with(HashSet::new)
            .insert(memory_ref.id.clone());

        // Update user index
        if let Some(user_id) = &memory_ref.metadata.user_id {
            self.user_index
                .entry(user_id.clone())
                .or_insert_with(HashSet::new)
                .insert(memory_ref.id.clone());
        }

        // Update agent index
        if let Some(agent_id) = &memory_ref.metadata.agent_id {
            self.agent_index
                .entry(agent_id.clone())
                .or_insert_with(HashSet::new)
                .insert(memory_ref.id.clone());
        }

        // Update tag index
        for tag in &memory_ref.metadata.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(memory_ref.id.clone());
        }

        // Update time index
        self.time_index
            .entry(memory_ref.created_at)
            .or_insert_with(HashSet::new)
            .insert(memory_ref.id.clone());
    }

    /// Update a memory in the repository
    pub fn update(&mut self, memory: MemoryNode) {
        // Remove old indexes
        if let Some(old_memory) = self.memories.get(&memory.id).cloned() {
            self.remove_from_indexes(&old_memory);
        }

        // Add updated memory
        self.add(memory);
    }

    /// Remove a memory from the repository
    pub fn remove(&mut self, id: &str) -> Option<Arc<MemoryNode>> {
        if let Some(memory) = self.memories.remove(id) {
            self.remove_from_indexes(&memory);
            self.relationships.remove(id);
            Some(memory)
        } else {
            None
        }
    }

    /// Get a memory by ID
    pub fn get(&self, id: &str) -> Option<Arc<MemoryNode>> {
        self.memories.get(id).cloned()
    }

    /// Query memories by filter
    pub fn query(&self, filter: &MemoryFilter) -> Result<Vec<MemoryNode>> {
        let mut result_ids = HashSet::new();
        let mut first_filter = true;

        // Filter by memory types
        if let Some(types) = &filter.memory_types {
            let type_ids: HashSet<String> = types
                .iter()
                .flat_map(|t| self.type_index.get(t).cloned().unwrap_or_default())
                .collect();

            if first_filter {
                result_ids = type_ids;
                first_filter = false;
            } else {
                result_ids = result_ids.intersection(&type_ids).cloned().collect();
            }
        }

        // Filter by user ID
        if let Some(user_id) = &filter.user_id {
            if let Some(user_ids) = self.user_index.get(user_id) {
                if first_filter {
                    result_ids = user_ids.clone();
                    first_filter = false;
                } else {
                    result_ids = result_ids.intersection(user_ids).cloned().collect();
                }
            } else {
                return Ok(Vec::new());
            }
        }

        // Filter by agent ID
        if let Some(agent_id) = &filter.agent_id {
            if let Some(agent_ids) = self.agent_index.get(agent_id) {
                if first_filter {
                    result_ids = agent_ids.clone();
                    first_filter = false;
                } else {
                    result_ids = result_ids.intersection(agent_ids).cloned().collect();
                }
            } else {
                return Ok(Vec::new());
            }
        }

        // Filter by tags
        if let Some(tags) = &filter.tags {
            let tag_ids: HashSet<String> = tags
                .iter()
                .filter_map(|tag| self.tag_index.get(tag))
                .flat_map(|ids| ids.iter())
                .cloned()
                .collect();

            if first_filter {
                result_ids = tag_ids;
                first_filter = false;
            } else {
                result_ids = result_ids.intersection(&tag_ids).cloned().collect();
            }
        }

        // If no filters applied, get all memories
        if first_filter {
            result_ids = self.memories.keys().cloned().collect();
        }

        // Apply time range filter
        let mut results: Vec<Arc<MemoryNode>> = result_ids
            .into_iter()
            .filter_map(|id| self.memories.get(&id))
            .filter(|memory| {
                if let Some(time_range) = &filter.time_range {
                    let in_range = time_range
                        .start
                        .map_or(true, |start| memory.created_at >= start)
                        && time_range.end.map_or(true, |end| memory.created_at < end);
                    in_range
                } else {
                    true
                }
            })
            .filter(|memory| {
                if let Some((min, max)) = filter.importance_range {
                    memory.metadata.importance >= min && memory.metadata.importance <= max
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // Sort by creation time (newest first) by default
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply pagination
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(results.len());

        Ok(results
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|arc| (*arc).clone())
            .collect())
    }

    /// Get statistics about the repository
    pub fn stats(&self) -> RepositoryStats {
        RepositoryStats {
            total_memories: self.memories.len(),
            memories_by_type: self
                .type_index
                .iter()
                .map(|(t, ids)| (t.clone(), ids.len()))
                .collect(),
            total_relationships: self.relationships.values().map(|v| v.len()).sum(),
            unique_users: self.user_index.len(),
            unique_agents: self.agent_index.len(),
            unique_tags: self.tag_index.len(),
        }
    }

    /// Remove a memory from all indexes
    fn remove_from_indexes(&mut self, memory: &MemoryNode) {
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
