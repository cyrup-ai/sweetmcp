//! Cognitive state management for memory nodes

use arrayvec::{ArrayString, ArrayVec};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Cognitive state representing mental context and processing state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    pub id: Uuid,
    pub semantic_context: SemanticContext,
    pub emotional_valence: EmotionalValence,
    pub processing_depth: f32,
    pub activation_level: f32,
    pub associations: ArrayVec<Association, 16>,        // Max 16 associations per state
    pub timestamp: Instant,
}

/// Semantic context information with zero-allocation patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    pub primary_concepts: ArrayVec<ArrayString<32>, 8>,     // Max 8 concepts, 32 chars each
    pub secondary_concepts: ArrayVec<ArrayString<32>, 16>,  // Max 16 concepts, 32 chars each
    pub domain_tags: ArrayVec<ArrayString<24>, 4>,          // Max 4 domains, 24 chars each
    pub abstraction_level: AbstractionLevel,
}

/// Emotional valence of cognitive state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalValence {
    pub arousal: f32,   // -1.0 to 1.0
    pub valence: f32,   // -1.0 to 1.0
    pub dominance: f32, // -1.0 to 1.0
}

/// Level of abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbstractionLevel {
    Concrete,
    Intermediate,
    Abstract,
    MetaCognitive,
}

/// Association between cognitive states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Association {
    pub target_id: Uuid,
    pub strength: f32,
    pub association_type: AssociationType,
}

/// Types of associations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssociationType {
    Semantic,
    Temporal,
    Causal,
    Emotional,
    Structural,
}

/// Lock-free manager for cognitive states with cache-line alignment
#[repr(align(64))]
pub struct CognitiveStateManager {
    states: DashMap<Uuid, CognitiveState>,
    by_concept: DashMap<ArrayString<32>, ArrayVec<Uuid, 64>>,  // Max 64 states per concept
    by_domain: DashMap<ArrayString<24>, ArrayVec<Uuid, 64>>,   // Max 64 states per domain
    by_time: DashMap<u64, ArrayVec<Uuid, 16>>,                 // Time bucketed by minute, max 16 per minute
}

impl CognitiveState {
    /// Create a new cognitive state
    pub fn new(semantic_context: SemanticContext) -> Self {
        Self {
            id: Uuid::new_v4(),
            semantic_context,
            emotional_valence: EmotionalValence::neutral(),
            processing_depth: 0.5,
            activation_level: 1.0,
            associations: ArrayVec::new(),
            timestamp: Instant::now(),
        }
    }

    /// Check if state is still active
    pub fn is_active(&self, decay_time: Duration) -> bool {
        let elapsed = self.timestamp.elapsed();
        let decay_factor = (-elapsed.as_secs_f64() / decay_time.as_secs_f64()).exp();
        self.activation_level * decay_factor as f32 > 0.1
    }

    /// Add an association to another state (with bounds checking)
    pub fn add_association(
        &mut self,
        target_id: Uuid,
        strength: f32,
        association_type: AssociationType,
    ) -> Result<(), ()> {
        let association = Association {
            target_id,
            strength: strength.clamp(0.0, 1.0),
            association_type,
        };
        
        // Use try_push to handle capacity limit gracefully
        self.associations.try_push(association).map_err(|_| ())
    }

    /// Update activation level
    pub fn activate(&mut self, boost: f32) {
        self.activation_level = (self.activation_level + boost).min(1.0);
        self.timestamp = Instant::now();
    }
}

impl EmotionalValence {
    /// Create neutral emotional state
    pub fn neutral() -> Self {
        Self {
            arousal: 0.0,
            valence: 0.0,
            dominance: 0.0,
        }
    }

    /// Create from specific values
    pub fn new(arousal: f32, valence: f32, dominance: f32) -> Self {
        Self {
            arousal: arousal.clamp(-1.0, 1.0),
            valence: valence.clamp(-1.0, 1.0),
            dominance: dominance.clamp(-1.0, 1.0),
        }
    }

    /// Calculate emotional distance
    pub fn distance(&self, other: &EmotionalValence) -> f32 {
        let da = self.arousal - other.arousal;
        let dv = self.valence - other.valence;
        let dd = self.dominance - other.dominance;
        (da * da + dv * dv + dd * dd).sqrt()
    }
}

impl CognitiveStateManager {
    /// Create a new lock-free cognitive state manager
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
            by_concept: DashMap::new(),
            by_domain: DashMap::new(),
            by_time: DashMap::new(),
        }
    }

    /// Add a new cognitive state using lock-free operations
    pub async fn add_state(&self, state: CognitiveState) -> Uuid {
        let id = state.id;

        // Index by primary concepts (lock-free)
        for concept in &state.semantic_context.primary_concepts {
            let concept_key = ArrayString::from(concept.as_str()).unwrap_or_else(|_| {
                // Truncate if too long
                ArrayString::from(&concept.as_str()[..32]).expect("Truncated string should fit")
            });
            
            self.by_concept.entry(concept_key).or_insert_with(ArrayVec::new).push(id);
        }

        // Index by domain tags (lock-free)
        for domain in &state.semantic_context.domain_tags {
            let domain_key = ArrayString::from(domain.as_str()).unwrap_or_else(|_| {
                // Truncate if too long
                ArrayString::from(&domain.as_str()[..24]).expect("Truncated string should fit")
            });
            
            self.by_domain.entry(domain_key).or_insert_with(ArrayVec::new).push(id);
        }

        // Index by time (bucketed by minute for efficiency)
        let time_bucket = state.timestamp.elapsed().as_secs() / 60;
        self.by_time.entry(time_bucket).or_insert_with(ArrayVec::new).push(id);

        // Store state (lock-free)
        self.states.insert(id, state);

        id
    }

    /// Get a cognitive state by ID using lock-free operations
    pub async fn get_state(&self, id: &Uuid) -> Option<CognitiveState> {
        self.states.get(id).map(|entry| entry.value().clone())
    }

    /// Find states by concept using lock-free operations
    pub async fn find_by_concept(&self, concept: &str) -> Vec<CognitiveState> {
        let concept_key = ArrayString::from(concept).unwrap_or_else(|_| {
            // Truncate if too long
            ArrayString::from(&concept[..32]).expect("Truncated string should fit")
        });
        
        if let Some(ids_entry) = self.by_concept.get(&concept_key) {
            ids_entry.iter()
                .filter_map(|id| self.states.get(id).map(|entry| entry.value().clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Find states by domain using lock-free operations
    pub async fn find_by_domain(&self, domain: &str) -> Vec<CognitiveState> {
        let domain_key = ArrayString::from(domain).unwrap_or_else(|_| {
            // Truncate if too long
            ArrayString::from(&domain[..24]).expect("Truncated string should fit")
        });
        
        if let Some(ids_entry) = self.by_domain.get(&domain_key) {
            ids_entry.iter()
                .filter_map(|id| self.states.get(id).map(|entry| entry.value().clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clean up inactive states using lock-free operations
    pub async fn cleanup_inactive(&self, decay_time: Duration) {
        // Collect inactive state IDs
        let mut inactive_ids = Vec::new();
        
        for entry in self.states.iter() {
            if !entry.value().is_active(decay_time) {
                inactive_ids.push(*entry.key());
            }
        }

        // Remove from main storage (lock-free)
        for id in &inactive_ids {
            self.states.remove(id);
        }

        // Update concept indices (lock-free)
        for mut concept_entry in self.by_concept.iter_mut() {
            concept_entry.value_mut().retain(|id| !inactive_ids.contains(id));
        }

        // Update domain indices (lock-free)
        for mut domain_entry in self.by_domain.iter_mut() {
            domain_entry.value_mut().retain(|id| !inactive_ids.contains(id));
        }

        // Update time indices (lock-free)
        for mut time_entry in self.by_time.iter_mut() {
            time_entry.value_mut().retain(|id| !inactive_ids.contains(id));
        }
    }

    /// Analyze memory context using zero-allocation patterns
    pub async fn analyze_memory_context(
        &self,
        _memory: &crate::memory::MemoryNode,
    ) -> crate::cognitive::quantum::types::CognitiveResult<CognitiveState> {
        // Simplified implementation with zero-allocation patterns
        let mut semantic_context = SemanticContext {
            primary_concepts: ArrayVec::new(),
            secondary_concepts: ArrayVec::new(),
            domain_tags: ArrayVec::new(),
            abstraction_level: AbstractionLevel::Intermediate,
        };

        // Add default concept with bounds checking
        let default_concept = ArrayString::from("default").expect("Default concept should fit");
        semantic_context.primary_concepts.push(default_concept);

        Ok(CognitiveState::new(semantic_context))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cognitive_state_creation() {
        let mut context = SemanticContext {
            primary_concepts: ArrayVec::new(),
            secondary_concepts: ArrayVec::new(),
            domain_tags: ArrayVec::new(),
            abstraction_level: AbstractionLevel::Concrete,
        };

        context.primary_concepts.push(ArrayString::from("test").expect("Should fit"));
        context.domain_tags.push(ArrayString::from("testing").expect("Should fit"));

        let state = CognitiveState::new(context);

        assert_eq!(state.semantic_context.primary_concepts[0].as_str(), "test");
        assert_eq!(state.activation_level, 1.0);
        assert!(state.is_active(Duration::from_secs(300)));
    }

    #[test]
    fn test_emotional_valence() {
        let v1 = EmotionalValence::new(0.5, 0.5, 0.0);
        let v2 = EmotionalValence::new(-0.5, -0.5, 0.0);

        let distance = v1.distance(&v2);
        assert!((distance - 1.414).abs() < 0.01); // sqrt(2)
    }

    #[tokio::test]
    async fn test_state_manager() {
        let manager = CognitiveStateManager::new();

        let mut context = SemanticContext {
            primary_concepts: ArrayVec::new(),
            secondary_concepts: ArrayVec::new(),
            domain_tags: ArrayVec::new(),
            abstraction_level: AbstractionLevel::Abstract,
        };

        context.primary_concepts.push(ArrayString::from("rust").expect("Should fit"));
        context.primary_concepts.push(ArrayString::from("memory").expect("Should fit"));
        context.domain_tags.push(ArrayString::from("programming").expect("Should fit"));

        let state = CognitiveState::new(context);
        let id = manager.add_state(state).await;

        // Test retrieval
        let retrieved = manager.get_state(&id).await;
        assert!(retrieved.is_some());

        // Test concept search
        let found = manager.find_by_concept("rust").await;
        assert_eq!(found.len(), 1);

        // Test domain search
        let found = manager.find_by_domain("programming").await;
        assert_eq!(found.len(), 1);
    }
}
