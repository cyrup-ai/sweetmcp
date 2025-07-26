//! Main committee implementation extracted from consensus.rs

use crate::cognitive::mcts::types::node_types::CodeState;
use crate::cognitive::types::CognitiveError;
use crate::vector::async_vector_optimization::OptimizationSpec;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, info};

use super::super::core::{CommitteeAgent, EvaluationRubric, ConsensusDecision};
use super::calculation::{ConsensusCalculator, ConsensusQuality};
use super::evaluation_phases::{EvaluationPhase, PhaseExecutor};
use super::steering::SteeringSystem;
use super::events::{CommitteeEvent, EventBus};

/// High-performance committee evaluation system with advanced consensus algorithms
pub struct Committee {
    pub agents: Vec<CommitteeAgent>,
    consensus_calculator: ConsensusCalculator,
    phase_executor: PhaseExecutor,
    steering_system: SteeringSystem,
    cache: Arc<RwLock<HashMap<String, CachedDecision>>>,
    event_tx: mpsc::UnboundedSender<CommitteeEvent>,
    event_bus: Arc<RwLock<EventBus>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
}

/// Cached decision with metadata
#[derive(Debug, Clone)]
struct CachedDecision {
    decision: ConsensusDecision,
    quality: ConsensusQuality,
    created_at: std::time::Instant,
    access_count: u32,
}

/// Performance metrics tracking
#[derive(Debug, Default)]
struct PerformanceMetrics {
    total_evaluations: u64,
    cache_hits: u64,
    total_evaluation_time_ms: u64,
    consensus_reached_count: u64,
    early_consensus_count: u64,
}

impl Committee {
    /// Create new committee with optimized initialization
    pub fn new(
        agents: Vec<CommitteeAgent>,
        consensus_threshold: f64,
        max_concurrent: usize,
        event_tx: mpsc::UnboundedSender<CommitteeEvent>,
    ) -> Self {
        Self {
            agents,
            consensus_calculator: ConsensusCalculator::new(consensus_threshold),
            phase_executor: PhaseExecutor::new(max_concurrent),
            steering_system: SteeringSystem::default(),
            cache: Arc::new(RwLock::new(HashMap::with_capacity(1000))),
            event_tx,
            event_bus: Arc::new(RwLock::new(EventBus::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }

    /// Evaluate action with advanced multi-phase consensus algorithm
    pub async fn evaluate_action(
        &self,
        state: &CodeState,
        action: &str,
        spec: &OptimizationSpec,
        user_objective: &str,
    ) -> Result<ConsensusDecision, CognitiveError> {
        let start_time = std::time::Instant::now();
        let cache_key = self.generate_cache_key(action, user_objective);
        
        // Fast cache lookup with LRU behavior
        if let Some(cached) = self.get_cached_decision(&cache_key).await {
            self.record_cache_hit().await;
            self.emit_event(CommitteeEvent::final_decision(
                action.to_string(),
                cached.decision.clone(),
                0,
                0,
                true,
            )).await;
            return Ok(cached.decision);
        }

        let rubric = EvaluationRubric::from_spec(spec, user_objective);
        let mut rounds = Vec::new();
        let mut current_phase = EvaluationPhase::Initial;

        // Multi-phase evaluation loop with early termination
        loop {
            self.emit_event(CommitteeEvent::evaluation_started(
                action.to_string(),
                current_phase,
                self.agents.len(),
            )).await;

            // Execute current phase
            let previous_evals = if current_phase == EvaluationPhase::Initial {
                None
            } else {
                rounds.last().map(|r: &super::evaluation_phases::EvaluationRound| r.evaluations.as_slice())
            };

            let steering_feedback = if current_phase == EvaluationPhase::Refine {
                self.generate_steering_feedback(&rounds)
            } else {
                None
            };

            let round = self.phase_executor.execute_phase(
                &self.agents,
                state,
                action,
                &rubric,
                current_phase,
                previous_evals,
                steering_feedback.as_deref(),
            ).await?;

            // Calculate consensus for this round
            let consensus = self.consensus_calculator.calculate_consensus(&round.evaluations);
            let quality = self.consensus_calculator.analyze_quality(&round.evaluations);

            self.emit_event(CommitteeEvent::PhaseCompleted {
                phase: current_phase,
                statistics: round.statistics(),
                consensus_reached: self.consensus_calculator.meets_threshold(&consensus),
                next_phase: current_phase.next(),
            }).await;

            rounds.push(round);

            // Check for early consensus
            if self.consensus_calculator.meets_threshold(&consensus) {
                self.emit_event(CommitteeEvent::EarlyConsensus {
                    phase: current_phase,
                    decision: consensus.clone(),
                    threshold_exceeded_by: consensus.overall_score - 0.7, // Assuming 0.7 threshold
                }).await;

                self.finalize_evaluation(action, consensus.clone(), rounds.len(), quality).await?;
                self.cache_decision(&cache_key, consensus.clone(), quality).await;
                self.record_performance_metrics(start_time, true, true).await;
                return Ok(consensus);
            }

            // Check if we should continue to next phase
            match current_phase {
                EvaluationPhase::Initial => current_phase = EvaluationPhase::Review,
                EvaluationPhase::Review => {
                    // Generate steering feedback to decide if refinement is needed
                    if let Some(feedback) = self.generate_steering_feedback(&rounds) {
                        if feedback.should_continue() {
                            current_phase = EvaluationPhase::Refine;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                },
                EvaluationPhase::Refine => break, // Final phase
            }
        }

        // Make final decision from best available consensus
        let final_consensus = self.select_best_consensus(&rounds);
        let final_quality = self.consensus_calculator.analyze_quality(&rounds.last().unwrap().evaluations);

        self.finalize_evaluation(action, final_consensus.clone(), rounds.len(), final_quality.clone()).await?;
        self.cache_decision(&cache_key, final_consensus.clone(), final_quality).await;
        self.record_performance_metrics(start_time, false, false).await;

        Ok(final_consensus)
    }

    /// Generate cache key with SHA-256 hashing for security
    fn generate_cache_key(&self, action: &str, objective: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(action.as_bytes());
        hasher.update(b"|"); // Separator to prevent collision
        hasher.update(objective.as_bytes());
        let hash = hasher.finalize();
        
        // Use first 16 characters of hex for reasonable key length
        format!("{:x}", hash)[..16].to_string()
    }

    /// Get cached decision with LRU access tracking
    async fn get_cached_decision(&self, key: &str) -> Option<CachedDecision> {
        let mut cache = self.cache.write().await;
        if let Some(cached) = cache.get_mut(key) {
            cached.access_count += 1;
            Some(cached.clone())
        } else {
            None
        }
    }

    /// Cache decision with quality metrics
    async fn cache_decision(&self, key: &str, decision: ConsensusDecision, quality: ConsensusQuality) {
        let mut cache = self.cache.write().await;
        
        // Implement simple LRU eviction if cache is full
        if cache.len() >= 1000 {
            // Remove least recently used entries (simplified)
            let keys_to_remove: Vec<String> = cache
                .iter()
                .filter(|(_, cached)| cached.access_count == 0)
                .take(100)
                .map(|(k, _)| k.clone())
                .collect();
            
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
        
        cache.insert(key.to_string(), CachedDecision {
            decision,
            quality,
            created_at: std::time::Instant::now(),
            access_count: 0,
        });
    }

    /// Generate steering feedback using the steering system
    fn generate_steering_feedback(&self, rounds: &[super::evaluation_phases::EvaluationRound]) -> Option<String> {
        self.steering_system
            .generate_steering_feedback(rounds)
            .map(|feedback| feedback.message)
    }

    /// Select best consensus from all evaluation rounds
    fn select_best_consensus(&self, rounds: &[super::evaluation_phases::EvaluationRound]) -> ConsensusDecision {
        if rounds.is_empty() {
            return ConsensusDecision::empty();
        }

        // Find round with highest consensus score
        rounds
            .iter()
            .map(|round| self.consensus_calculator.calculate_consensus(&round.evaluations))
            .max_by(|a, b| a.overall_score.partial_cmp(&b.overall_score).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or_else(ConsensusDecision::empty)
    }

    /// Finalize evaluation with comprehensive logging
    async fn finalize_evaluation(
        &self,
        action: &str,
        decision: ConsensusDecision,
        rounds: usize,
        quality: ConsensusQuality,
    ) -> Result<(), CognitiveError> {
        info!(
            "Committee decision for '{}': progress={}, score={:.2}, confidence={:.2}, rounds={}, quality={:.2}",
            action,
            decision.makes_progress,
            decision.overall_score,
            decision.confidence,
            rounds,
            quality.quality_score()
        );

        let total_time = self.performance_metrics.read().await.total_evaluation_time_ms;
        
        self.emit_event(CommitteeEvent::final_decision(
            action.to_string(),
            decision,
            rounds,
            total_time,
            false,
        )).await;

        Ok(())
    }

    /// Emit event through both channels
    async fn emit_event(&self, event: CommitteeEvent) {
        // Send through unbounded channel
        if let Err(e) = self.event_tx.send(event.clone()) {
            debug!("Failed to send committee event: {}", e);
        }

        // Publish through event bus
        let bus = self.event_bus.read().await;
        bus.publish(&event);
    }

    /// Record cache hit for metrics
    async fn record_cache_hit(&self) {
        let mut metrics = self.performance_metrics.write().await;
        metrics.cache_hits += 1;
        metrics.total_evaluations += 1;
    }

    /// Record comprehensive performance metrics
    async fn record_performance_metrics(&self, start_time: std::time::Instant, consensus_reached: bool, early_consensus: bool) {
        let mut metrics = self.performance_metrics.write().await;
        metrics.total_evaluations += 1;
        metrics.total_evaluation_time_ms += start_time.elapsed().as_millis() as u64;
        
        if consensus_reached {
            metrics.consensus_reached_count += 1;
        }
        
        if early_consensus {
            metrics.early_consensus_count += 1;
        }
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        let metrics = self.performance_metrics.read().await;
        let cache_size = self.cache.read().await.len();
        
        PerformanceStats {
            total_evaluations: metrics.total_evaluations,
            cache_hits: metrics.cache_hits,
            cache_hit_rate: if metrics.total_evaluations > 0 {
                metrics.cache_hits as f64 / metrics.total_evaluations as f64
            } else {
                0.0
            },
            average_evaluation_time_ms: if metrics.total_evaluations > 0 {
                metrics.total_evaluation_time_ms as f64 / metrics.total_evaluations as f64
            } else {
                0.0
            },
            consensus_rate: if metrics.total_evaluations > 0 {
                metrics.consensus_reached_count as f64 / metrics.total_evaluations as f64
            } else {
                0.0
            },
            early_consensus_rate: if metrics.consensus_reached_count > 0 {
                metrics.early_consensus_count as f64 / metrics.consensus_reached_count as f64
            } else {
                0.0
            },
            cache_size,
            active_agents: self.agents.len(),
        }
    }

    /// Clear performance metrics
    pub async fn clear_metrics(&self) {
        let mut metrics = self.performance_metrics.write().await;
        *metrics = PerformanceMetrics::default();
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total_entries = cache.len();
        let total_access_count: u32 = cache.values().map(|c| c.access_count).sum();
        
        CacheStats {
            total_entries,
            average_access_count: if total_entries > 0 {
                total_access_count as f64 / total_entries as f64
            } else {
                0.0
            },
            oldest_entry_age_ms: cache
                .values()
                .map(|c| c.created_at.elapsed().as_millis() as u64)
                .max()
                .unwrap_or(0),
        }
    }
}

/// Performance statistics structure
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_evaluations: u64,
    pub cache_hits: u64,
    pub cache_hit_rate: f64,
    pub average_evaluation_time_ms: f64,
    pub consensus_rate: f64,
    pub early_consensus_rate: f64,
    pub cache_size: usize,
    pub active_agents: usize,
}

/// Cache statistics structure
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,  
    pub average_access_count: f64,
    pub oldest_entry_age_ms: u64,
}