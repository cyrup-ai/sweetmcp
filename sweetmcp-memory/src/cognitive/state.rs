//! Cognitive state management for memory nodes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Cognitive state representing mental context and processing state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    pub id: Uuid,
    pub semantic_context: SemanticContext,
    pub emotional_valence: EmotionalValence,
    pub processing_depth: f32,
    pub activation_level: f32,
    pub associations: Vec<Association>,
    pub timestamp: Instant,
}

/// Semantic context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    pub primary_concepts: Vec<String>,
    pub secondary_concepts: Vec<String>,
    pub domain_tags: Vec<String>,
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

/// Manager for cognitive states
pub struct CognitiveStateManager {
    states: Arc<RwLock<HashMap<Uuid, CognitiveState>>>,
    state_index: Arc<RwLock<StateIndex>>,
}

/// Index for efficient state lookup
struct StateIndex {
    by_concept: HashMap<String, Vec<Uuid>>,
    by_domain: HashMap<String, Vec<Uuid>>,
    by_time: Vec<(Instant, Uuid)>,
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
            associations: Vec::new(),
            timestamp: Instant::now(),
        }
    }

    /// Check if state is still active
    pub fn is_active(&self, decay_time: Duration) -> bool {
        let elapsed = self.timestamp.elapsed();
        let decay_factor = (-elapsed.as_secs_f64() / decay_time.as_secs_f64()).exp();
        self.activation_level * decay_factor as f32 > 0.1
    }

    /// Add an association to another state
    pub fn add_association(
        &mut self,
        target_id: Uuid,
        strength: f32,
        association_type: AssociationType,
    ) {
        self.associations.push(Association {
            target_id,
            strength: strength.clamp(0.0, 1.0),
            association_type,
        });
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
    /// Create a new cognitive state manager
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            state_index: Arc::new(RwLock::new(StateIndex::new())),
        }
    }

    /// Add a new cognitive state
    pub async fn add_state(&self, state: CognitiveState) -> Uuid {
        let id = state.id;

        // Update index
        {
            let mut index = self.state_index.write().await;

            // Index by primary concepts
            for concept in &state.semantic_context.primary_concepts {
                index
                    .by_concept
                    .entry(concept.clone())
                    .or_insert_with(Vec::new)
                    .push(id);
            }

            // Index by domain tags
            for domain in &state.semantic_context.domain_tags {
                index
                    .by_domain
                    .entry(domain.clone())
                    .or_insert_with(Vec::new)
                    .push(id);
            }

            // Index by time
            index.by_time.push((state.timestamp, id));
        }

        // Store state
        self.states.write().await.insert(id, state);

        id
    }

    /// Get a cognitive state by ID
    pub async fn get_state(&self, id: &Uuid) -> Option<CognitiveState> {
        self.states.read().await.get(id).cloned()
    }

    /// Find states by concept
    pub async fn find_by_concept(&self, concept: &str) -> Vec<CognitiveState> {
        let index = self.state_index.read().await;
        let states = self.states.read().await;

        if let Some(ids) = index.by_concept.get(concept) {
            ids.iter()
                .filter_map(|id| states.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Find states by domain
    pub async fn find_by_domain(&self, domain: &str) -> Vec<CognitiveState> {
        let index = self.state_index.read().await;
        let states = self.states.read().await;

        if let Some(ids) = index.by_domain.get(domain) {
            ids.iter()
                .filter_map(|id| states.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clean up inactive states
    pub async fn cleanup_inactive(&self, decay_time: Duration) {
        let mut states = self.states.write().await;
        let mut index = self.state_index.write().await;

        // Find inactive states
        let inactive_ids: Vec<Uuid> = states
            .iter()
            .filter(|(_, state)| !state.is_active(decay_time))
            .map(|(id, _)| *id)
            .collect();

        // Remove from main storage
        for id in &inactive_ids {
            states.remove(id);
        }

        // Update indices
        for concepts in index.by_concept.values_mut() {
            concepts.retain(|id| !inactive_ids.contains(id));
        }

        for domains in index.by_domain.values_mut() {
            domains.retain(|id| !inactive_ids.contains(id));
        }

        index.by_time.retain(|(_, id)| !inactive_ids.contains(id));
    }

    /// Analyze memory context (placeholder for quantum router)
    pub async fn analyze_memory_context(
        &self,
        _memory: &crate::memory::MemoryNode,
    ) -> crate::cognitive::quantum::types::CognitiveResult<CognitiveState> {
        // Simplified implementation
        let semantic_context = SemanticContext {
            primary_concepts: vec!["default".to_string()],
            secondary_concepts: vec![],
            domain_tags: vec![],
            abstraction_level: AbstractionLevel::Intermediate,
        };

        Ok(CognitiveState::new(semantic_context))
    }
}

impl StateIndex {
    fn new() -> Self {
        Self {
            by_concept: HashMap::new(),
            by_domain: HashMap::new(),
            by_time: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cognitive_state_creation() {
        let context = SemanticContext {
            primary_concepts: vec!["test".to_string()],
            secondary_concepts: vec![],
            domain_tags: vec!["testing".to_string()],
            abstraction_level: AbstractionLevel::Concrete,
        };

        let state = CognitiveState::new(context);

        assert_eq!(state.semantic_context.primary_concepts[0], "test");
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

        let context = SemanticContext {
            primary_concepts: vec!["rust".to_string(), "memory".to_string()],
            secondary_concepts: vec![],
            domain_tags: vec!["programming".to_string()],
            abstraction_level: AbstractionLevel::Abstract,
        };

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
