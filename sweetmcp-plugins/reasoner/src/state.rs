use crate::types::ThoughtNode;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct StateManager {
    cache: Arc<Mutex<LruCache<String, ThoughtNode>>>,
    nodes: Arc<Mutex<HashMap<String, ThoughtNode>>>,
}

impl StateManager {
    pub fn new(cache_size: usize) -> Self {
        let cache_size = NonZeroUsize::new(cache_size).unwrap_or_else(|| {
            // This should never fail since 1 is always non-zero
            unsafe { NonZeroUsize::new_unchecked(1) }
        });
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_node(&self, id: &str) -> Option<ThoughtNode> {
        // Check cache first
        let mut cache = self.cache.lock().await;
        if let Some(node) = cache.get(id) {
            return Some(node.clone());
        }

        // Check main storage
        let nodes = self.nodes.lock().await;
        if let Some(node) = nodes.get(id) {
            let node_clone = node.clone();
            drop(nodes); // Release the lock before modifying the cache

            cache.put(id.to_string(), node_clone.clone());
            return Some(node_clone);
        }

        None
    }

    pub async fn save_node(&self, node: ThoughtNode) {
        let node_id = node.id.clone();
        let node_clone = node.clone();

        // Save to main storage
        let mut nodes = self.nodes.lock().await;
        nodes.insert(node_id.clone(), node);
        drop(nodes); // Release the lock before modifying the cache

        // Update cache
        let mut cache = self.cache.lock().await;
        cache.put(node_id, node_clone);
    }

    pub async fn get_children(&self, node_id: &str) -> Vec<ThoughtNode> {
        let node = match self.get_node(node_id).await {
            Some(n) => n,
            None => return vec![],
        };

        let mut children = vec![];
        for id in &node.children {
            if let Some(child) = self.get_node(id).await {
                children.push(child);
            }
        }

        children
    }

    pub async fn get_path(&self, node_id: &str) -> Vec<ThoughtNode> {
        let mut path = vec![];
        let mut current_id = node_id.to_string();

        while !current_id.is_empty() {
            match self.get_node(&current_id).await {
                Some(node) => {
                    path.insert(0, node.clone());
                    current_id = node.parent_id.unwrap_or_default();
                }
                None => break,
            }
        }

        path
    }

    pub async fn get_all_nodes(&self) -> Vec<ThoughtNode> {
        let nodes = self.nodes.lock().await;
        nodes.values().cloned().collect()
    }

    pub async fn clear(&self) {
        let mut nodes = self.nodes.lock().await;
        nodes.clear();
        drop(nodes);

        let mut cache = self.cache.lock().await;
        cache.clear();
    }
}
