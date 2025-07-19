//! Memory relationship management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of relationships between memories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// Causal relationship (A causes B)
    CausedBy,
    /// Temporal relationship (A happens before B)
    PrecedesTemporally,
    /// Semantic similarity
    SimilarTo,
    /// Contradiction relationship
    Contradicts,
    /// Supporting evidence
    Supports,
    /// Part-whole relationship
    PartOf,
    /// Generalization relationship
    GeneralizationOf,
    /// Specialization relationship
    SpecializationOf,
    /// Association relationship
    AssociatedWith,
    /// Custom relationship type
    Custom(String),
}

impl RelationshipType {
    /// Check if this relationship type is bidirectional
    pub fn is_bidirectional(&self) -> bool {
        matches!(
            self,
            Self::SimilarTo | Self::Contradicts | Self::AssociatedWith
        )
    }

    /// Get the inverse relationship type
    pub fn inverse(&self) -> Option<Self> {
        match self {
            Self::CausedBy => Some(Self::Custom("causes".to_string())),
            Self::PrecedesTemporally => Some(Self::Custom("follows_temporally".to_string())),
            Self::PartOf => Some(Self::Custom("has_part".to_string())),
            Self::GeneralizationOf => Some(Self::SpecializationOf),
            Self::SpecializationOf => Some(Self::GeneralizationOf),
            Self::Supports => Some(Self::Custom("supported_by".to_string())),
            _ => None,
        }
    }
}

impl From<&str> for RelationshipType {
    fn from(s: &str) -> Self {
        match s {
            "caused_by" => Self::CausedBy,
            "precedes_temporally" => Self::PrecedesTemporally,
            "similar_to" => Self::SimilarTo,
            "contradicts" => Self::Contradicts,
            "supports" => Self::Supports,
            "part_of" => Self::PartOf,
            "generalization_of" => Self::GeneralizationOf,
            "specialization_of" => Self::SpecializationOf,
            "associated_with" => Self::AssociatedWith,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Relationship strength calculator
pub struct RelationshipStrengthCalculator {
    /// Weights for different factors
    weights: HashMap<String, f32>,
}

impl RelationshipStrengthCalculator {
    /// Create a new calculator with default weights
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        weights.insert("temporal_proximity".to_string(), 0.3);
        weights.insert("semantic_similarity".to_string(), 0.4);
        weights.insert("co_occurrence".to_string(), 0.2);
        weights.insert("user_feedback".to_string(), 0.1);

        Self { weights }
    }

    /// Calculate relationship strength based on various factors
    pub fn calculate_strength(
        &self,
        temporal_distance: Option<chrono::Duration>,
        semantic_similarity: Option<f32>,
        co_occurrence_count: Option<u32>,
        user_rating: Option<f32>,
    ) -> f32 {
        let mut strength = 0.0;
        let mut total_weight = 0.0;

        // Temporal proximity factor
        if let Some(distance) = temporal_distance {
            let temporal_score = 1.0 / (1.0 + distance.num_hours() as f32 / 24.0);
            strength += temporal_score * self.weights.get("temporal_proximity").unwrap_or(&0.0);
            total_weight += self.weights.get("temporal_proximity").unwrap_or(&0.0);
        }

        // Semantic similarity factor
        if let Some(similarity) = semantic_similarity {
            strength += similarity * self.weights.get("semantic_similarity").unwrap_or(&0.0);
            total_weight += self.weights.get("semantic_similarity").unwrap_or(&0.0);
        }

        // Co-occurrence factor
        if let Some(count) = co_occurrence_count {
            let co_occurrence_score = (count as f32).ln() / 10.0;
            strength +=
                co_occurrence_score.min(1.0) * self.weights.get("co_occurrence").unwrap_or(&0.0);
            total_weight += self.weights.get("co_occurrence").unwrap_or(&0.0);
        }

        // User feedback factor
        if let Some(rating) = user_rating {
            strength += rating * self.weights.get("user_feedback").unwrap_or(&0.0);
            total_weight += self.weights.get("user_feedback").unwrap_or(&0.0);
        }

        // Normalize
        if total_weight > 0.0 {
            strength / total_weight
        } else {
            0.5 // Default strength
        }
    }
}

impl Default for RelationshipStrengthCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Relationship graph for managing memory connections
pub struct RelationshipGraph {
    /// Adjacency list representation
    edges: HashMap<String, Vec<(String, RelationshipType, f32)>>,

    /// Reverse edges for bidirectional traversal
    reverse_edges: HashMap<String, Vec<(String, RelationshipType, f32)>>,
}

impl RelationshipGraph {
    /// Create a new relationship graph
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
        }
    }

    /// Add a relationship
    pub fn add_relationship(
        &mut self,
        source: String,
        target: String,
        relationship_type: RelationshipType,
        strength: f32,
    ) {
        // Add forward edge
        self.edges
            .entry(source.clone())
            .or_insert_with(Vec::new)
            .push((target.clone(), relationship_type.clone(), strength));

        // Add reverse edge
        self.reverse_edges
            .entry(target.clone())
            .or_insert_with(Vec::new)
            .push((source.clone(), relationship_type.clone(), strength));

        // If bidirectional, add the opposite direction
        if relationship_type.is_bidirectional() {
            self.edges
                .entry(target.clone())
                .or_insert_with(Vec::new)
                .push((source.clone(), relationship_type.clone(), strength));

            self.reverse_edges
                .entry(source)
                .or_insert_with(Vec::new)
                .push((target, relationship_type, strength));
        }
    }

    /// Get outgoing relationships for a memory
    pub fn get_outgoing(&self, memory_id: &str) -> Vec<(String, RelationshipType, f32)> {
        self.edges.get(memory_id).cloned().unwrap_or_default()
    }

    /// Get incoming relationships for a memory
    pub fn get_incoming(&self, memory_id: &str) -> Vec<(String, RelationshipType, f32)> {
        self.reverse_edges
            .get(memory_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Find path between two memories
    pub fn find_path(&self, start: &str, end: &str, max_depth: usize) -> Option<Vec<String>> {
        // Simple BFS implementation
        use std::collections::{HashSet, VecDeque};

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent_map: HashMap<String, String> = HashMap::new();

        queue.push_back(start.to_string());
        visited.insert(start.to_string());

        while let Some(current) = queue.pop_front() {
            if current == end {
                // Reconstruct path
                let mut path = vec![end.to_string()];
                let mut node = end.to_string();

                while let Some(parent) = parent_map.get(&node) {
                    path.push(parent.clone());
                    node = parent.clone();
                }

                path.reverse();
                return Some(path);
            }

            if visited.len() >= max_depth {
                continue;
            }

            if let Some(neighbors) = self.edges.get(&current) {
                for (neighbor, _, _) in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        parent_map.insert(neighbor.clone(), current.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        None
    }
}

impl Default for RelationshipGraph {
    fn default() -> Self {
        Self::new()
    }
}
