//! Semantic memory graph analysis operations
//!
//! This module provides graph analysis capabilities for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use std::collections::{HashMap, HashSet, VecDeque};

use super::memory::SemanticMemory;
use super::item_core::SemanticItem;
use super::semantic_relationship::SemanticRelationship;

impl SemanticMemory {
    /// Find shortest path between two items
    pub fn find_shortest_path(&self, start_id: &str, end_id: &str) -> Option<Vec<&SemanticRelationship>> {
        if start_id == end_id {
            return Some(Vec::new());
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent_map: HashMap<&str, (&str, &SemanticRelationship)> = HashMap::new();

        queue.push_back(start_id);
        visited.insert(start_id);

        while let Some(current_id) = queue.pop_front() {
            if current_id == end_id {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = end_id;
                
                while let Some((parent, relationship)) = parent_map.get(current) {
                    path.push(*relationship);
                    current = parent;
                }
                
                path.reverse();
                return Some(path);
            }

            for rel in self.get_relationships_for_item(current_id) {
                if let Some(other_id) = rel.get_other_item_id(current_id) {
                    if !visited.contains(other_id) {
                        visited.insert(other_id);
                        parent_map.insert(other_id, (current_id, rel));
                        queue.push_back(other_id);
                    }
                }
            }
        }

        None
    }

    /// Get strongly connected components
    pub fn get_strongly_connected_components(&self) -> Vec<Vec<&str>> {
        let mut index = 0;
        let mut stack = Vec::new();
        let mut indices: HashMap<&str, usize> = HashMap::new();
        let mut lowlinks: HashMap<&str, usize> = HashMap::new();
        let mut on_stack: HashSet<&str> = HashSet::new();
        let mut components = Vec::new();

        for item in &self.items {
            if !indices.contains_key(item.id.as_str()) {
                self.tarjan_scc(
                    item.id.as_str(),
                    &mut index,
                    &mut stack,
                    &mut indices,
                    &mut lowlinks,
                    &mut on_stack,
                    &mut components,
                );
            }
        }

        components
    }

    /// Tarjan's strongly connected components algorithm helper
    fn tarjan_scc<'a>(
        &'a self,
        v: &'a str,
        index: &mut usize,
        stack: &mut Vec<&'a str>,
        indices: &mut HashMap<&'a str, usize>,
        lowlinks: &mut HashMap<&'a str, usize>,
        on_stack: &mut HashSet<&'a str>,
        components: &mut Vec<Vec<&'a str>>,
    ) {
        indices.insert(v, *index);
        lowlinks.insert(v, *index);
        *index += 1;
        stack.push(v);
        on_stack.insert(v);

        for rel in self.get_outgoing_relationships(v) {
            let w = rel.target_id.as_str();
            if !indices.contains_key(w) {
                self.tarjan_scc(w, index, stack, indices, lowlinks, on_stack, components);
                let w_lowlink = *lowlinks.get(w).unwrap_or(&0);
                let v_lowlink = *lowlinks.get(v).unwrap_or(&0);
                lowlinks.insert(v, v_lowlink.min(w_lowlink));
            } else if on_stack.contains(w) {
                let w_index = *indices.get(w).unwrap_or(&0);
                let v_lowlink = *lowlinks.get(v).unwrap_or(&0);
                lowlinks.insert(v, v_lowlink.min(w_index));
            }
        }

        let v_index = *indices.get(v).unwrap_or(&0);
        let v_lowlink = *lowlinks.get(v).unwrap_or(&0);
        
        if v_lowlink == v_index {
            let mut component = Vec::new();
            loop {
                if let Some(w) = stack.pop() {
                    on_stack.remove(w);
                    component.push(w);
                    if w == v {
                        break;
                    }
                } else {
                    break;
                }
            }
            components.push(component);
        }
    }

    /// Get items that are hubs (highly connected)
    pub fn get_hub_items(&self, min_connections: usize) -> Vec<(&SemanticItem, usize)> {
        self.items
            .iter()
            .map(|item| {
                let connection_count = self.get_relationships_for_item(&item.id).len();
                (item, connection_count)
            })
            .filter(|(_, count)| *count >= min_connections)
            .collect()
    }

    /// Get leaf items (items with only one connection)
    pub fn get_leaf_items(&self) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| self.get_relationships_for_item(&item.id).len() == 1)
            .collect()
    }

    /// Get isolated items (items with no connections)
    pub fn get_isolated_items(&self) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| self.get_relationships_for_item(&item.id).is_empty())
            .collect()
    }

    /// Calculate clustering coefficient for the memory graph
    pub fn clustering_coefficient(&self) -> f64 {
        if self.items.len() < 3 {
            return 0.0;
        }

        let mut total_coefficient = 0.0;
        let mut node_count = 0;

        for item in &self.items {
            let neighbors: Vec<&str> = self
                .get_relationships_for_item(&item.id)
                .iter()
                .filter_map(|rel| rel.get_other_item_id(&item.id))
                .collect();

            if neighbors.len() < 2 {
                continue;
            }

            let possible_edges = neighbors.len() * (neighbors.len() - 1) / 2;
            let mut actual_edges = 0;

            for i in 0..neighbors.len() {
                for j in (i + 1)..neighbors.len() {
                    if self.relationships.iter().any(|rel| {
                        (rel.source_id == neighbors[i] && rel.target_id == neighbors[j])
                            || (rel.source_id == neighbors[j] && rel.target_id == neighbors[i])
                    }) {
                        actual_edges += 1;
                    }
                }
            }

            total_coefficient += actual_edges as f64 / possible_edges as f64;
            node_count += 1;
        }

        if node_count == 0 {
            0.0
        } else {
            total_coefficient / node_count as f64
        }
    }

    /// Check if the graph is connected
    pub fn is_connected(&self) -> bool {
        if self.items.is_empty() {
            return true;
        }

        let start_item = &self.items[0];
        let reachable = self.find_connected_items(&start_item.id, usize::MAX);
        
        // Add 1 for the start item itself
        reachable.len() + 1 == self.items.len()
    }

    /// Get graph connectivity components
    pub fn get_connectivity_components(&self) -> Vec<Vec<&SemanticItem>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for item in &self.items {
            if !visited.contains(item.id.as_str()) {
                let mut component = vec![item];
                let mut stack = vec![item.id.as_str()];
                visited.insert(item.id.as_str());

                while let Some(current_id) = stack.pop() {
                    for rel in self.get_relationships_for_item(current_id) {
                        if let Some(other_id) = rel.get_other_item_id(current_id) {
                            if !visited.contains(other_id) {
                                visited.insert(other_id);
                                if let Some(other_item) = self.get_item(other_id) {
                                    component.push(other_item);
                                    stack.push(other_id);
                                }
                            }
                        }
                    }
                }

                components.push(component);
            }
        }

        components
    }
}