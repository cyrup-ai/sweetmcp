//! MCTS factory for creating optimized instances
//!
//! This module provides blazing-fast MCTS factory with zero allocation
//! optimizations and elegant ergonomic interfaces for creating MCTS instances.

use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tracing::debug;

use crate::cognitive::committee::CommitteeEvent;
use crate::cognitive::performance::PerformanceAnalyzer;
use crate::cognitive::types::{CognitiveError, OptimizationSpec};

use super::{
    types::CodeState,
    controller::MCTS,
};

/// MCTS factory for creating configured instances
pub struct MCTSFactory;

impl MCTSFactory {
    /// Create MCTS optimized for speed
    #[inline]
    pub async fn create_speed_optimized(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
    ) -> Result<MCTS, CognitiveError> {
        debug!("Creating speed-optimized MCTS instance");
        
        MCTS::with_config(
            initial_state,
            performance_analyzer,
            spec,
            user_objective,
            event_tx,
            2.0, // Higher exploration for faster convergence
            5000, // Fewer iterations for speed
            Some(Duration::from_secs(60)), // 1 minute timeout
        ).await
    }

    /// Create MCTS optimized for quality
    #[inline]
    pub async fn create_quality_optimized(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
    ) -> Result<MCTS, CognitiveError> {
        debug!("Creating quality-optimized MCTS instance");
        
        MCTS::with_config(
            initial_state,
            performance_analyzer,
            spec,
            user_objective,
            event_tx,
            1.0, // Lower exploration, more exploitation for quality
            50000, // More iterations for quality
            Some(Duration::from_secs(1800)), // 30 minute timeout
        ).await
    }

    /// Create balanced MCTS
    #[inline]
    pub async fn create_balanced(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
    ) -> Result<MCTS, CognitiveError> {
        debug!("Creating balanced MCTS instance");
        
        MCTS::new(
            initial_state,
            performance_analyzer,
            spec,
            user_objective,
            event_tx,
        ).await
    }

    /// Create MCTS optimized for memory efficiency
    #[inline]
    pub async fn create_memory_optimized(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
    ) -> Result<MCTS, CognitiveError> {
        debug!("Creating memory-optimized MCTS instance");
        
        MCTS::with_config(
            initial_state,
            performance_analyzer,
            spec,
            user_objective,
            event_tx,
            1.5, // Moderate exploration
            10000, // Moderate iterations
            Some(Duration::from_secs(300)), // 5 minute timeout
        ).await
    }

    /// Create MCTS for real-time applications
    #[inline]
    pub async fn create_realtime(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
    ) -> Result<MCTS, CognitiveError> {
        debug!("Creating real-time MCTS instance");
        
        MCTS::with_config(
            initial_state,
            performance_analyzer,
            spec,
            user_objective,
            event_tx,
            3.0, // High exploration for quick decisions
            1000, // Very few iterations for real-time
            Some(Duration::from_secs(10)), // 10 second timeout
        ).await
    }

    /// Create MCTS for batch processing
    #[inline]
    pub async fn create_batch_processing(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
    ) -> Result<MCTS, CognitiveError> {
        debug!("Creating batch processing MCTS instance");
        
        MCTS::with_config(
            initial_state,
            performance_analyzer,
            spec,
            user_objective,
            event_tx,
            0.7, // Low exploration, high exploitation for batch
            100000, // Many iterations for thorough processing
            Some(Duration::from_secs(3600)), // 1 hour timeout
        ).await
    }

    /// Create MCTS with adaptive configuration
    #[inline]
    pub async fn create_adaptive(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
        requirements: &PerformanceRequirements,
    ) -> Result<MCTS, CognitiveError> {
        debug!("Creating adaptive MCTS instance based on requirements");
        
        // Analyze requirements to determine optimal configuration
        let (exploration_constant, max_iterations, timeout) = Self::analyze_requirements(requirements);
        
        MCTS::with_config(
            initial_state,
            performance_analyzer,
            spec,
            user_objective,
            event_tx,
            exploration_constant,
            max_iterations,
            timeout,
        ).await
    }

    /// Create MCTS with specific optimization profile
    #[inline]
    pub async fn create_with_profile(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
        profile: OptimizationProfile,
    ) -> Result<MCTS, CognitiveError> {
        debug!("Creating MCTS instance with profile: {:?}", profile);
        
        match profile {
            OptimizationProfile::Speed => {
                Self::create_speed_optimized(
                    initial_state, performance_analyzer, spec, user_objective, event_tx
                ).await
            }
            OptimizationProfile::Quality => {
                Self::create_quality_optimized(
                    initial_state, performance_analyzer, spec, user_objective, event_tx
                ).await
            }
            OptimizationProfile::Balanced => {
                Self::create_balanced(
                    initial_state, performance_analyzer, spec, user_objective, event_tx
                ).await
            }
            OptimizationProfile::MemoryOptimized => {
                Self::create_memory_optimized(
                    initial_state, performance_analyzer, spec, user_objective, event_tx
                ).await
            }
            OptimizationProfile::Realtime => {
                Self::create_realtime(
                    initial_state, performance_analyzer, spec, user_objective, event_tx
                ).await
            }
            OptimizationProfile::BatchProcessing => {
                Self::create_batch_processing(
                    initial_state, performance_analyzer, spec, user_objective, event_tx
                ).await
            }
        }
    }

    /// Create multiple MCTS instances for distributed processing
    #[inline]
    pub async fn create_distributed(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
        instance_count: usize,
        profile: OptimizationProfile,
    ) -> Result<Vec<MCTS>, CognitiveError> {
        debug!("Creating {} distributed MCTS instances", instance_count);
        
        if instance_count == 0 {
            return Err(CognitiveError::InvalidConfiguration(
                "Instance count must be greater than 0".to_string()
            ));
        }

        let mut instances = Vec::with_capacity(instance_count);
        
        for i in 0..instance_count {
            // Create slightly varied configurations for load distribution
            let varied_state = Self::create_varied_state(&initial_state, i);
            let instance = Self::create_with_profile(
                varied_state,
                performance_analyzer.clone(),
                spec.clone(),
                user_objective.clone(),
                event_tx.clone(),
                profile,
            ).await?;
            
            instances.push(instance);
        }
        
        Ok(instances)
    }

    /// Analyze performance requirements to determine optimal configuration
    #[inline]
    fn analyze_requirements(requirements: &PerformanceRequirements) -> (f64, u64, Option<Duration>) {
        let exploration_constant = if requirements.max_latency_ms < 100 {
            3.0 // High exploration for very low latency
        } else if requirements.max_latency_ms < 1000 {
            2.0 // Moderate exploration for low latency
        } else if requirements.quality_priority {
            1.0 // Low exploration for high quality
        } else {
            1.41 // Default sqrt(2)
        };

        let max_iterations = if requirements.max_latency_ms < 100 {
            500 // Very few iterations for ultra-low latency
        } else if requirements.max_latency_ms < 1000 {
            2000 // Few iterations for low latency
        } else if requirements.quality_priority {
            50000 // Many iterations for quality
        } else {
            10000 // Default iterations
        };

        let timeout = if requirements.max_latency_ms < 100 {
            Some(Duration::from_millis(50))
        } else if requirements.max_latency_ms < 1000 {
            Some(Duration::from_millis(requirements.max_latency_ms))
        } else {
            Some(Duration::from_millis(requirements.max_latency_ms.min(300000)))
        };

        (exploration_constant, max_iterations, timeout)
    }

    /// Create varied initial state for distributed processing
    #[inline]
    fn create_varied_state(base_state: &CodeState, variation_index: usize) -> CodeState {
        // Create slight variations in the initial state for distributed processing
        let variation_factor = 1.0 + (variation_index as f64 * 0.01);
        
        CodeState::new(
            format!("// Variation {}\n{}", variation_index, base_state.code),
            base_state.latency * variation_factor,
            base_state.memory * variation_factor,
            base_state.relevance,
        )
    }

    /// Get recommended profile for given requirements
    #[inline]
    pub fn recommend_profile(requirements: &PerformanceRequirements) -> OptimizationProfile {
        if requirements.max_latency_ms < 100 {
            OptimizationProfile::Realtime
        } else if requirements.max_latency_ms < 1000 {
            OptimizationProfile::Speed
        } else if requirements.quality_priority {
            OptimizationProfile::Quality
        } else if requirements.memory_limit_mb < 512 {
            OptimizationProfile::MemoryOptimized
        } else if requirements.is_batch_workload {
            OptimizationProfile::BatchProcessing
        } else {
            OptimizationProfile::Balanced
        }
    }
}

/// Optimization profiles for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationProfile {
    /// Speed-optimized for fast results
    Speed,
    /// Quality-optimized for best results
    Quality,
    /// Balanced performance
    Balanced,
    /// Memory usage optimization
    MemoryOptimized,
    /// Real-time processing
    Realtime,
    /// Batch processing optimization
    BatchProcessing,
}

/// Performance requirements for MCTS configuration
#[derive(Debug, Clone)]
pub struct PerformanceRequirements {
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: u64,
    /// Memory limit in megabytes
    pub memory_limit_mb: u64,
    /// Whether quality is prioritized over speed
    pub quality_priority: bool,
    /// Whether this is a batch workload
    pub is_batch_workload: bool,
    /// Whether real-time guarantees are needed
    pub requires_realtime: bool,
}

impl PerformanceRequirements {
    /// Create new performance requirements
    #[inline]
    pub fn new() -> Self {
        Self {
            max_latency_ms: 1000,
            memory_limit_mb: 1024,
            quality_priority: false,
            is_batch_workload: false,
            requires_realtime: false,
        }
    }

    /// Create requirements for real-time applications
    #[inline]
    pub fn realtime() -> Self {
        Self {
            max_latency_ms: 50,
            memory_limit_mb: 512,
            quality_priority: false,
            is_batch_workload: false,
            requires_realtime: true,
        }
    }

    /// Create requirements for quality-focused applications
    #[inline]
    pub fn quality_focused() -> Self {
        Self {
            max_latency_ms: 30000,
            memory_limit_mb: 4096,
            quality_priority: true,
            is_batch_workload: false,
            requires_realtime: false,
        }
    }

    /// Create requirements for batch processing
    #[inline]
    pub fn batch_processing() -> Self {
        Self {
            max_latency_ms: 3600000, // 1 hour
            memory_limit_mb: 8192,
            quality_priority: true,
            is_batch_workload: true,
            requires_realtime: false,
        }
    }

    /// Create requirements for memory-constrained environments
    #[inline]
    pub fn memory_constrained() -> Self {
        Self {
            max_latency_ms: 5000,
            memory_limit_mb: 256,
            quality_priority: false,
            is_batch_workload: false,
            requires_realtime: false,
        }
    }
}

impl Default for PerformanceRequirements {
    fn default() -> Self {
        Self::new()
    }
}