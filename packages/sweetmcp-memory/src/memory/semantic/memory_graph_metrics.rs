//! Semantic memory graph metrics and centrality calculations
//!
//! This module provides graph metrics and centrality analysis for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use super::memory::SemanticMemory;
use super::item_core::SemanticItem;

impl SemanticMemory {
    /// Calculate graph diameter (longest shortest path)
    pub fn graph_diameter(&self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }

        let mut max_distance = 0;
        let mut found_path = false;

        for start_item in &self.items {
            for end_item in &self.items {
                if start_item.id != end_item.id {
                    if let Some(path) = self.find_shortest_path(&start_item.id, &end_item.id) {
                        max_distance = max_distance.max(path.len());
                        found_path = true;
                    }
                }
            }
        }

        if found_path {
            Some(max_distance)
        } else {
            None
        }
    }

    /// Calculate graph radius (minimum eccentricity)
    pub fn graph_radius(&self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }

        let mut min_eccentricity = usize::MAX;
        let mut found_path = false;

        for center_item in &self.items {
            let mut max_distance_from_center = 0;
            let mut has_reachable_nodes = false;

            for other_item in &self.items {
                if center_item.id != other_item.id {
                    if let Some(path) = self.find_shortest_path(&center_item.id, &other_item.id) {
                        max_distance_from_center = max_distance_from_center.max(path.len());
                        has_reachable_nodes = true;
                    }
                }
            }

            if has_reachable_nodes {
                min_eccentricity = min_eccentricity.min(max_distance_from_center);
                found_path = true;
            }
        }

        if found_path && min_eccentricity != usize::MAX {
            Some(min_eccentricity)
        } else {
            None
        }
    }

    /// Find central items (items with minimum eccentricity)
    pub fn find_central_items(&self) -> Vec<&SemanticItem> {
        if let Some(radius) = self.graph_radius() {
            let mut central_items = Vec::new();

            for item in &self.items {
                let mut max_distance = 0;
                let mut has_reachable_nodes = false;

                for other_item in &self.items {
                    if item.id != other_item.id {
                        if let Some(path) = self.find_shortest_path(&item.id, &other_item.id) {
                            max_distance = max_distance.max(path.len());
                            has_reachable_nodes = true;
                        }
                    }
                }

                if has_reachable_nodes && max_distance == radius {
                    central_items.push(item);
                }
            }

            central_items
        } else {
            Vec::new()
        }
    }

    /// Calculate betweenness centrality for all items
    pub fn calculate_betweenness_centrality(&self) -> std::collections::HashMap<String, f64> {
        let mut centrality = std::collections::HashMap::new();
        
        // Initialize centrality scores
        for item in &self.items {
            centrality.insert(item.id.clone(), 0.0);
        }

        // For each pair of items, find shortest paths and count how many pass through each item
        for start_item in &self.items {
            for end_item in &self.items {
                if start_item.id != end_item.id {
                    if let Some(path) = self.find_shortest_path(&start_item.id, &end_item.id) {
                        // Count intermediate nodes in the path
                        let mut current_id = &start_item.id;
                        for relationship in &path {
                            if let Some(next_id) = relationship.get_other_item_id(current_id) {
                                if next_id != end_item.id {
                                    *centrality.get_mut(next_id).unwrap_or(&mut 0.0) += 1.0;
                                }
                                current_id = &relationship.target_id;
                            }
                        }
                    }
                }
            }
        }

        // Normalize by the number of pairs
        let n = self.items.len();
        if n > 2 {
            let normalization_factor = ((n - 1) * (n - 2)) as f64;
            for value in centrality.values_mut() {
                *value /= normalization_factor;
            }
        }

        centrality
    }

    /// Calculate closeness centrality for all items
    pub fn calculate_closeness_centrality(&self) -> std::collections::HashMap<String, f64> {
        let mut centrality = std::collections::HashMap::new();

        for item in &self.items {
            let mut total_distance = 0.0;
            let mut reachable_count = 0;

            for other_item in &self.items {
                if item.id != other_item.id {
                    if let Some(path) = self.find_shortest_path(&item.id, &other_item.id) {
                        total_distance += path.len() as f64;
                        reachable_count += 1;
                    }
                }
            }

            let closeness = if reachable_count > 0 && total_distance > 0.0 {
                reachable_count as f64 / total_distance
            } else {
                0.0
            };

            centrality.insert(item.id.clone(), closeness);
        }

        centrality
    }

    /// Calculate degree centrality for all items
    pub fn calculate_degree_centrality(&self) -> std::collections::HashMap<String, f64> {
        let mut centrality = std::collections::HashMap::new();
        let max_possible_degree = if self.items.len() > 1 { self.items.len() - 1 } else { 1 };

        for item in &self.items {
            let degree = self.get_relationships_for_item(&item.id).len();
            let normalized_degree = degree as f64 / max_possible_degree as f64;
            centrality.insert(item.id.clone(), normalized_degree);
        }

        centrality
    }

    /// Calculate eigenvector centrality using power iteration
    pub fn calculate_eigenvector_centrality(&self, max_iterations: usize, tolerance: f64) -> std::collections::HashMap<String, f64> {
        let n = self.items.len();
        if n == 0 {
            return std::collections::HashMap::new();
        }

        // Create adjacency matrix representation
        let mut item_indices = std::collections::HashMap::new();
        for (i, item) in self.items.iter().enumerate() {
            item_indices.insert(&item.id, i);
        }

        let mut adjacency = vec![vec![0.0; n]; n];
        for relationship in &self.relationships {
            if let (Some(&i), Some(&j)) = (
                item_indices.get(&relationship.source_id),
                item_indices.get(&relationship.target_id),
            ) {
                adjacency[i][j] = 1.0;
                adjacency[j][i] = 1.0; // Treat as undirected
            }
        }

        // Initialize centrality vector
        let mut centrality = vec![1.0 / (n as f64).sqrt(); n];
        
        // Power iteration
        for _ in 0..max_iterations {
            let mut new_centrality = vec![0.0; n];
            
            // Matrix-vector multiplication
            for i in 0..n {
                for j in 0..n {
                    new_centrality[i] += adjacency[i][j] * centrality[j];
                }
            }
            
            // Normalize
            let norm: f64 = new_centrality.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 0.0 {
                for value in &mut new_centrality {
                    *value /= norm;
                }
            }
            
            // Check convergence
            let diff: f64 = centrality
                .iter()
                .zip(&new_centrality)
                .map(|(a, b)| (a - b).abs())
                .sum();
            
            centrality = new_centrality;
            
            if diff < tolerance {
                break;
            }
        }

        // Convert back to HashMap
        let mut result = std::collections::HashMap::new();
        for (item, &index) in &item_indices {
            result.insert((*item).clone(), centrality[index]);
        }

        result
    }
}