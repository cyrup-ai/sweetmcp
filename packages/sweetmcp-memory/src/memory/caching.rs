//! Lock-free caching operations extracted from memory manager

use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::memory::{MemoryNode, filter::MemoryFilter};
use crate::utils::Result;

/// Lock-free memory cache using DashMap for blazing-fast retrieval
pub struct MemoryCache {
    /// Lock-free memory cache using DashMap for blazing-fast retrieval
    memory_cache: DashMap<String, MemoryNode>,
    /// Atomic counter for memory statistics
    memory_count: AtomicUsize,
}

impl MemoryCache {
    /// Create a new lock-free memory cache
    pub fn new() -> Self {
        Self {
            memory_cache: DashMap::new(),
            memory_count: AtomicUsize::new(0),
        }
    }

    /// Get memory from cache (lock-free cache lookup)
    pub fn get(&self, id: &str) -> Option<MemoryNode> {
        // Try cache first (lock-free read)
        self.memory_cache.get(id).map(|cached| cached.clone())
    }

    /// Insert memory into cache (lock-free operation)
    pub fn insert(&self, memory: MemoryNode) {
        // Add to lock-free memory cache using DashMap
        self.memory_cache.insert(memory.id.clone(), memory);
        // Atomically increment memory count
        self.memory_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Update memory in cache
    pub fn update(&self, memory: MemoryNode) {
        // Update in lock-free cache using DashMap
        self.memory_cache.insert(memory.id.clone(), memory);
    }

    /// Remove memory from cache (lock-free operation)  
    pub fn remove(&self, id: &str) -> bool {
        // Remove from lock-free cache using DashMap
        if self.memory_cache.remove(id).is_some() {
            // Only decrement if item was actually in cache
            self.memory_count.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Get memories by filter (lock-free operation using cache)
    pub fn get_memories_by_filter(&self, filter: MemoryFilter) -> Vec<MemoryNode> {
        // Use lock-free cache iteration for filtering
        self.memory_cache
            .iter()
            .filter_map(|entry| {
                let memory = entry.value();
                // Simple filter matching - in production this would be more sophisticated
                match filter.memory_types.as_ref() {
                    Some(mem_types) if !mem_types.contains(&memory.memory_type) => None,
                    _ => Some(memory.clone()),
                }
            })
            .collect()
    }

    /// Get memory count (lock-free atomic read)
    pub fn memory_count(&self) -> usize {
        self.memory_count.load(Ordering::Relaxed)
    }

    /// Cache-first retrieval strategy for multiple memories
    pub fn get_multiple_with_fallback<F>(&self, ids: Vec<String>, fallback: F) -> Vec<MemoryNode>
    where
        F: Fn(String) -> Option<MemoryNode>,
    {
        let mut memories = Vec::new();
        for id in ids {
            // Try cache first (lock-free read)
            if let Some(cached_memory) = self.memory_cache.get(&id) {
                memories.push(cached_memory.clone());
            } else if let Some(memory) = fallback(id.clone()) {
                // Cache miss - add to cache for future access
                self.memory_cache.insert(memory.id.clone(), memory.clone());
                memories.push(memory);
            }
        }
        memories
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}