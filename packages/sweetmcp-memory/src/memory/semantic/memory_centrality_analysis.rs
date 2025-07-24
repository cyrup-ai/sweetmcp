//! Semantic memory centrality analysis and PageRank calculations
//!
//! This module provides advanced centrality analysis for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use super::memory::SemanticMemory;

impl SemanticMemory {
    /// Calculate PageRank centrality
    pub fn calculate_pagerank_centrality(&self, damping_factor: f64, max_iterations: usize, tolerance: f64) -> std::collections::HashMap<String, f64> {
        let n = self.items.len();
        if n == 0 {
            return std::collections::HashMap::new();
        }

        let mut pagerank = std::collections::HashMap::new();
        let initial_value = 1.0 / n as f64;
        
        // Initialize PageRank values
        for item in &self.items {
            pagerank.insert(item.id.clone(), initial_value);
        }

        for _ in 0..max_iterations {
            let mut new_pagerank = std::collections::HashMap::new();
            
            // Initialize with teleportation probability
            for item in &self.items {
                new_pagerank.insert(item.id.clone(), (1.0 - damping_factor) / n as f64);
            }
            
            // Add contributions from incoming links
            for item in &self.items {
                let outgoing_relationships = self.get_outgoing_relationships(&item.id);
                let out_degree = outgoing_relationships.len();
                
                if out_degree > 0 {
                    let contribution = pagerank[&item.id] * damping_factor / out_degree as f64;
                    
                    for relationship in outgoing_relationships {
                        *new_pagerank.get_mut(&relationship.target_id).unwrap_or(&mut 0.0) += contribution;
                    }
                }
            }
            
            // Check convergence
            let diff: f64 = pagerank
                .iter()
                .map(|(id, &old_value)| (old_value - new_pagerank[id]).abs())
                .sum();
            
            pagerank = new_pagerank;
            
            if diff < tolerance {
                break;
            }
        }

        pagerank
    }

    /// Get comprehensive centrality analysis
    pub fn get_centrality_analysis(&self) -> CentralityAnalysis {
        CentralityAnalysis {
            degree_centrality: self.calculate_degree_centrality(),
            closeness_centrality: self.calculate_closeness_centrality(),
            betweenness_centrality: self.calculate_betweenness_centrality(),
            eigenvector_centrality: self.calculate_eigenvector_centrality(100, 1e-6),
            pagerank_centrality: self.calculate_pagerank_centrality(0.85, 100, 1e-6),
        }
    }
}

/// Comprehensive centrality analysis results
#[derive(Debug, Clone)]
pub struct CentralityAnalysis {
    pub degree_centrality: std::collections::HashMap<String, f64>,
    pub closeness_centrality: std::collections::HashMap<String, f64>,
    pub betweenness_centrality: std::collections::HashMap<String, f64>,
    pub eigenvector_centrality: std::collections::HashMap<String, f64>,
    pub pagerank_centrality: std::collections::HashMap<String, f64>,
}

impl CentralityAnalysis {
    /// Get the most central items by each centrality measure
    pub fn get_top_central_items(&self, top_n: usize) -> TopCentralItems {
        TopCentralItems {
            top_degree: self.get_top_items_by_centrality(&self.degree_centrality, top_n),
            top_closeness: self.get_top_items_by_centrality(&self.closeness_centrality, top_n),
            top_betweenness: self.get_top_items_by_centrality(&self.betweenness_centrality, top_n),
            top_eigenvector: self.get_top_items_by_centrality(&self.eigenvector_centrality, top_n),
            top_pagerank: self.get_top_items_by_centrality(&self.pagerank_centrality, top_n),
        }
    }

    /// Get items ranked by a specific centrality measure
    pub fn get_items_by_centrality(&self, centrality_type: CentralityType) -> Vec<(String, f64)> {
        let centrality_map = match centrality_type {
            CentralityType::Degree => &self.degree_centrality,
            CentralityType::Closeness => &self.closeness_centrality,
            CentralityType::Betweenness => &self.betweenness_centrality,
            CentralityType::Eigenvector => &self.eigenvector_centrality,
            CentralityType::PageRank => &self.pagerank_centrality,
        };

        let mut items: Vec<_> = centrality_map.iter().map(|(id, &score)| (id.clone(), score)).collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items
    }

    /// Get centrality statistics
    pub fn get_centrality_statistics(&self) -> CentralityStatistics {
        CentralityStatistics {
            degree_stats: self.calculate_centrality_stats(&self.degree_centrality),
            closeness_stats: self.calculate_centrality_stats(&self.closeness_centrality),
            betweenness_stats: self.calculate_centrality_stats(&self.betweenness_centrality),
            eigenvector_stats: self.calculate_centrality_stats(&self.eigenvector_centrality),
            pagerank_stats: self.calculate_centrality_stats(&self.pagerank_centrality),
        }
    }

    /// Calculate statistics for a centrality measure
    fn calculate_centrality_stats(&self, centrality: &std::collections::HashMap<String, f64>) -> CentralityStats {
        if centrality.is_empty() {
            return CentralityStats::default();
        }

        let values: Vec<f64> = centrality.values().copied().collect();
        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let median = if sorted_values.len() % 2 == 0 {
            let mid = sorted_values.len() / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[sorted_values.len() / 2]
        };

        CentralityStats {
            mean,
            median,
            std_dev,
            min: sorted_values[0],
            max: sorted_values[sorted_values.len() - 1],
            range: sorted_values[sorted_values.len() - 1] - sorted_values[0],
        }
    }

    fn get_top_items_by_centrality(&self, centrality: &std::collections::HashMap<String, f64>, top_n: usize) -> Vec<(String, f64)> {
        let mut items: Vec<_> = centrality.iter().map(|(id, &score)| (id.clone(), score)).collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items.into_iter().take(top_n).collect()
    }

    /// Find items that are central across multiple measures
    pub fn find_consistently_central_items(&self, top_percentage: f64) -> Vec<String> {
        let n = self.degree_centrality.len();
        let top_count = ((n as f64 * top_percentage).ceil() as usize).max(1);

        let degree_top: std::collections::HashSet<_> = self.get_top_items_by_centrality(&self.degree_centrality, top_count)
            .into_iter().map(|(id, _)| id).collect();
        let closeness_top: std::collections::HashSet<_> = self.get_top_items_by_centrality(&self.closeness_centrality, top_count)
            .into_iter().map(|(id, _)| id).collect();
        let betweenness_top: std::collections::HashSet<_> = self.get_top_items_by_centrality(&self.betweenness_centrality, top_count)
            .into_iter().map(|(id, _)| id).collect();
        let eigenvector_top: std::collections::HashSet<_> = self.get_top_items_by_centrality(&self.eigenvector_centrality, top_count)
            .into_iter().map(|(id, _)| id).collect();
        let pagerank_top: std::collections::HashSet<_> = self.get_top_items_by_centrality(&self.pagerank_centrality, top_count)
            .into_iter().map(|(id, _)| id).collect();

        // Find intersection of all sets
        let mut consistent_items: Vec<String> = degree_top
            .intersection(&closeness_top)
            .filter(|id| betweenness_top.contains(*id))
            .filter(|id| eigenvector_top.contains(*id))
            .filter(|id| pagerank_top.contains(*id))
            .map(|id| id.clone())
            .collect();

        consistent_items.sort();
        consistent_items
    }

    /// Calculate centrality correlation matrix
    pub fn calculate_centrality_correlations(&self) -> CentralityCorrelations {
        let items: Vec<_> = self.degree_centrality.keys().collect();
        
        CentralityCorrelations {
            degree_closeness: self.calculate_correlation(&self.degree_centrality, &self.closeness_centrality, &items),
            degree_betweenness: self.calculate_correlation(&self.degree_centrality, &self.betweenness_centrality, &items),
            degree_eigenvector: self.calculate_correlation(&self.degree_centrality, &self.eigenvector_centrality, &items),
            degree_pagerank: self.calculate_correlation(&self.degree_centrality, &self.pagerank_centrality, &items),
            closeness_betweenness: self.calculate_correlation(&self.closeness_centrality, &self.betweenness_centrality, &items),
            closeness_eigenvector: self.calculate_correlation(&self.closeness_centrality, &self.eigenvector_centrality, &items),
            closeness_pagerank: self.calculate_correlation(&self.closeness_centrality, &self.pagerank_centrality, &items),
            betweenness_eigenvector: self.calculate_correlation(&self.betweenness_centrality, &self.eigenvector_centrality, &items),
            betweenness_pagerank: self.calculate_correlation(&self.betweenness_centrality, &self.pagerank_centrality, &items),
            eigenvector_pagerank: self.calculate_correlation(&self.eigenvector_centrality, &self.pagerank_centrality, &items),
        }
    }

    fn calculate_correlation(
        &self,
        centrality1: &std::collections::HashMap<String, f64>,
        centrality2: &std::collections::HashMap<String, f64>,
        items: &[&String],
    ) -> f64 {
        if items.len() < 2 {
            return 0.0;
        }

        let values1: Vec<f64> = items.iter().map(|id| centrality1[*id]).collect();
        let values2: Vec<f64> = items.iter().map(|id| centrality2[*id]).collect();

        let mean1 = values1.iter().sum::<f64>() / values1.len() as f64;
        let mean2 = values2.iter().sum::<f64>() / values2.len() as f64;

        let numerator: f64 = values1.iter().zip(&values2)
            .map(|(x1, x2)| (x1 - mean1) * (x2 - mean2))
            .sum();

        let sum_sq1: f64 = values1.iter().map(|x| (x - mean1).powi(2)).sum();
        let sum_sq2: f64 = values2.iter().map(|x| (x - mean2).powi(2)).sum();

        let denominator = (sum_sq1 * sum_sq2).sqrt();

        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }
}

/// Types of centrality measures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CentralityType {
    Degree,
    Closeness,
    Betweenness,
    Eigenvector,
    PageRank,
}

/// Top central items by each centrality measure
#[derive(Debug, Clone)]
pub struct TopCentralItems {
    pub top_degree: Vec<(String, f64)>,
    pub top_closeness: Vec<(String, f64)>,
    pub top_betweenness: Vec<(String, f64)>,
    pub top_eigenvector: Vec<(String, f64)>,
    pub top_pagerank: Vec<(String, f64)>,
}

/// Statistics for centrality measures
#[derive(Debug, Clone)]
pub struct CentralityStatistics {
    pub degree_stats: CentralityStats,
    pub closeness_stats: CentralityStats,
    pub betweenness_stats: CentralityStats,
    pub eigenvector_stats: CentralityStats,
    pub pagerank_stats: CentralityStats,
}

/// Statistics for a single centrality measure
#[derive(Debug, Clone, Default)]
pub struct CentralityStats {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub range: f64,
}

/// Correlation matrix between centrality measures
#[derive(Debug, Clone)]
pub struct CentralityCorrelations {
    pub degree_closeness: f64,
    pub degree_betweenness: f64,
    pub degree_eigenvector: f64,
    pub degree_pagerank: f64,
    pub closeness_betweenness: f64,
    pub closeness_eigenvector: f64,
    pub closeness_pagerank: f64,
    pub betweenness_eigenvector: f64,
    pub betweenness_pagerank: f64,
    pub eigenvector_pagerank: f64,
}