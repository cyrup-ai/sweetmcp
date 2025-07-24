//! Network topology analysis for quantum entanglement networks
//!
//! This module provides blazing-fast topology analysis with zero allocation
//! optimizations for quantum entanglement network structure examination.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::super::super::node_state::QuantumMCTSNode;

/// Network topology analysis results with comprehensive metrics
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    /// Total number of nodes in the network
    pub total_nodes: usize,
    /// Total number of entanglements
    pub total_entanglements: usize,
    /// Average number of entanglements per node
    pub average_degree: f64,
    /// Maximum entanglements for any single node
    pub max_degree: usize,
    /// Network density (0.0 to 1.0)
    pub network_density: f64,
    /// Whether the network is fully connected
    pub is_connected: bool,
    /// Clustering coefficient for small-world properties
    pub clustering_coefficient: f64,
}

impl NetworkTopology {
    /// Check if network has good connectivity for quantum effects
    #[inline]
    pub fn has_good_connectivity(&self) -> bool {
        self.network_density > 0.1 && self.average_degree > 2.0 && self.clustering_coefficient > 0.3
    }
    
    /// Check if network is sparse (may need more entanglements)
    #[inline]
    pub fn is_sparse(&self) -> bool {
        self.network_density < 0.05 || self.average_degree < 1.0
    }
    
    /// Check if network is overly dense (may need pruning)
    #[inline]
    pub fn is_overly_dense(&self) -> bool {
        self.network_density > 0.8 || self.average_degree > 20.0
    }
    
    /// Calculate network efficiency score (0.0 to 1.0)
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        if self.total_nodes <= 1 {
            return 1.0;
        }
        
        let optimal_density = 0.3; // Sweet spot for quantum networks
        let density_score = 1.0 - (self.network_density - optimal_density).abs() / optimal_density;
        
        let optimal_clustering = 0.5;
        let clustering_score = 1.0 - (self.clustering_coefficient - optimal_clustering).abs() / optimal_clustering;
        
        let connectivity_score = if self.is_connected { 1.0 } else { 0.5 };
        
        // Weighted combination favoring connectivity and clustering
        (density_score * 0.3 + clustering_score * 0.4 + connectivity_score * 0.3).max(0.0).min(1.0)
    }
    
    /// Get recommendations for network optimization
    #[inline]
    pub fn optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.is_sparse() {
            recommendations.push("Network is too sparse - consider creating more entanglements".to_string());
        }
        
        if self.is_overly_dense() {
            recommendations.push("Network is overly dense - consider pruning weak entanglements".to_string());
        }
        
        if !self.is_connected {
            recommendations.push("Network has disconnected components - create bridge entanglements".to_string());
        }
        
        if self.clustering_coefficient < 0.2 {
            recommendations.push("Low clustering coefficient - create more local entanglements".to_string());
        }
        
        if self.max_degree > 50 {
            recommendations.push("Some nodes are over-entangled - distribute connections more evenly".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Network topology appears well-balanced".to_string());
        }
        
        recommendations
    }

    /// Calculate network resilience score
    #[inline]
    pub fn resilience_score(&self) -> f64 {
        if self.total_nodes <= 1 {
            return 1.0;
        }

        // Resilience factors
        let connectivity_factor = if self.is_connected { 1.0 } else { 0.0 };
        let redundancy_factor = (self.average_degree / 3.0).min(1.0); // Optimal around 3 connections
        let distribution_factor = 1.0 - (self.max_degree as f64 / self.total_nodes as f64).min(1.0);
        let clustering_factor = self.clustering_coefficient;

        // Weighted combination
        (connectivity_factor * 0.4 + 
         redundancy_factor * 0.3 + 
         distribution_factor * 0.2 + 
         clustering_factor * 0.1).max(0.0).min(1.0)
    }

    /// Get network health status
    #[inline]
    pub fn health_status(&self) -> NetworkHealthStatus {
        let efficiency = self.efficiency_score();
        let resilience = self.resilience_score();
        let overall_score = (efficiency + resilience) / 2.0;

        match overall_score {
            s if s >= 0.8 => NetworkHealthStatus::Excellent,
            s if s >= 0.6 => NetworkHealthStatus::Good,
            s if s >= 0.4 => NetworkHealthStatus::Fair,
            s if s >= 0.2 => NetworkHealthStatus::Poor,
            _ => NetworkHealthStatus::Critical,
        }
    }
}

/// Network topology analyzer for quantum entanglement networks
pub struct NetworkTopologyAnalyzer;

impl NetworkTopologyAnalyzer {
    /// Analyze network topology from entanglement graph
    #[inline]
    pub async fn analyze_topology(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<NetworkTopology, CognitiveError> {
        let total_nodes = nodes.len();
        let total_entanglements = graph.entanglement_count().await?;
        
        if total_nodes == 0 {
            return Ok(NetworkTopology {
                total_nodes: 0,
                total_entanglements: 0,
                average_degree: 0.0,
                max_degree: 0,
                network_density: 0.0,
                is_connected: false,
                clustering_coefficient: 0.0,
            });
        }

        // Calculate degree statistics
        let mut degrees = Vec::new();
        let mut total_degree = 0;
        let mut max_degree = 0;

        for node_id in nodes.keys() {
            let degree = graph.get_entanglement_count(node_id).await.unwrap_or(0);
            degrees.push(degree);
            total_degree += degree;
            max_degree = max_degree.max(degree);
        }

        let average_degree = total_degree as f64 / total_nodes as f64;
        
        // Calculate network density
        let max_possible_edges = if total_nodes > 1 {
            total_nodes * (total_nodes - 1) / 2
        } else {
            1
        };
        let network_density = total_entanglements as f64 / max_possible_edges as f64;

        // Check connectivity
        let is_connected = Self::check_connectivity(graph, nodes).await?;

        // Calculate clustering coefficient
        let clustering_coefficient = Self::calculate_clustering_coefficient(graph, nodes).await?;

        Ok(NetworkTopology {
            total_nodes,
            total_entanglements,
            average_degree,
            max_degree,
            network_density,
            is_connected,
            clustering_coefficient,
        })
    }

    /// Check if the network is fully connected
    #[inline]
    async fn check_connectivity(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<bool, CognitiveError> {
        if nodes.is_empty() {
            return Ok(true);
        }

        // Use BFS to check if all nodes are reachable from the first node
        let start_node = nodes.keys().next().unwrap();
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        
        queue.push_back(start_node.clone());
        visited.insert(start_node.clone());

        while let Some(current) = queue.pop_front() {
            let neighbors = graph.get_entangled_nodes(&current).await?;
            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    queue.push_back(neighbor);
                }
            }
        }

        Ok(visited.len() == nodes.len())
    }

    /// Calculate clustering coefficient for the network
    #[inline]
    async fn calculate_clustering_coefficient(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<f64, CognitiveError> {
        if nodes.len() < 3 {
            return Ok(0.0);
        }

        let mut total_clustering = 0.0;
        let mut node_count = 0;

        for node_id in nodes.keys() {
            let neighbors = graph.get_entangled_nodes(node_id).await?;
            if neighbors.len() < 2 {
                continue;
            }

            // Count triangles (connections between neighbors)
            let mut triangle_count = 0;
            let mut possible_triangles = 0;

            for i in 0..neighbors.len() {
                for j in (i + 1)..neighbors.len() {
                    possible_triangles += 1;
                    if graph.has_entanglement(&neighbors[i], &neighbors[j]).await? {
                        triangle_count += 1;
                    }
                }
            }

            if possible_triangles > 0 {
                let local_clustering = triangle_count as f64 / possible_triangles as f64;
                total_clustering += local_clustering;
                node_count += 1;
            }
        }

        if node_count == 0 {
            Ok(0.0)
        } else {
            Ok(total_clustering / node_count as f64)
        }
    }

    /// Calculate betweenness centrality for nodes
    #[inline]
    pub async fn calculate_betweenness_centrality(
        graph: &EntanglementGraph,
        nodes: &HashMap<String, Arc<RwLock<QuantumMCTSNode>>>,
    ) -> Result<HashMap<String, f64>, CognitiveError> {
        let mut centrality = HashMap::new();
        
        if nodes.len() < 3 {
            for node_id in nodes.keys() {
                centrality.insert(node_id.clone(), 0.0);
            }
            return Ok(centrality);
        }

        // Initialize centrality scores
        for node_id in nodes.keys() {
            centrality.insert(node_id.clone(), 0.0);
        }

        // For each pair of nodes, find shortest paths and count how many go through each node
        let node_ids: Vec<_> = nodes.keys().cloned().collect();
        
        for i in 0..node_ids.len() {
            for j in (i + 1)..node_ids.len() {
                let source = &node_ids[i];
                let target = &node_ids[j];
                
                // Find all shortest paths between source and target
                let paths = Self::find_shortest_paths(graph, source, target).await?;
                
                if !paths.is_empty() {
                    // Count how many paths go through each intermediate node
                    for path in &paths {
                        for k in 1..(path.len() - 1) {
                            let intermediate = &path[k];
                            if let Some(score) = centrality.get_mut(intermediate) {
                                *score += 1.0 / paths.len() as f64;
                            }
                        }
                    }
                }
            }
        }

        // Normalize by the number of node pairs
        let normalization_factor = if nodes.len() > 2 {
            ((nodes.len() - 1) * (nodes.len() - 2) / 2) as f64
        } else {
            1.0
        };

        for score in centrality.values_mut() {
            *score /= normalization_factor;
        }

        Ok(centrality)
    }

    /// Find all shortest paths between two nodes
    #[inline]
    async fn find_shortest_paths(
        graph: &EntanglementGraph,
        source: &str,
        target: &str,
    ) -> Result<Vec<Vec<String>>, CognitiveError> {
        let mut paths = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashMap::new();
        
        queue.push_back(vec![source.to_string()]);
        visited.insert(source.to_string(), 0);
        
        let mut shortest_length = None;
        
        while let Some(current_path) = queue.pop_front() {
            let current_node = current_path.last().unwrap();
            
            if current_node == target {
                if shortest_length.is_none() {
                    shortest_length = Some(current_path.len());
                    paths.push(current_path);
                } else if current_path.len() == shortest_length.unwrap() {
                    paths.push(current_path);
                } else {
                    break; // Found longer path, stop searching
                }
                continue;
            }
            
            // Skip if we've already found shorter paths
            if let Some(min_len) = shortest_length {
                if current_path.len() >= min_len {
                    continue;
                }
            }
            
            let neighbors = graph.get_entangled_nodes(current_node).await?;
            for neighbor in neighbors {
                if !current_path.contains(&neighbor) {
                    let path_length = current_path.len() + 1;
                    
                    // Only continue if this is a shortest path to the neighbor
                    if let Some(&existing_length) = visited.get(&neighbor) {
                        if path_length > existing_length {
                            continue;
                        }
                    } else {
                        visited.insert(neighbor.clone(), path_length);
                    }
                    
                    let mut new_path = current_path.clone();
                    new_path.push(neighbor);
                    queue.push_back(new_path);
                }
            }
        }
        
        Ok(paths)
    }
}

/// Network health status categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkHealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl std::fmt::Display for NetworkHealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkHealthStatus::Excellent => write!(f, "Excellent"),
            NetworkHealthStatus::Good => write!(f, "Good"),
            NetworkHealthStatus::Fair => write!(f, "Fair"),
            NetworkHealthStatus::Poor => write!(f, "Poor"),
            NetworkHealthStatus::Critical => write!(f, "Critical"),
        }
    }
}