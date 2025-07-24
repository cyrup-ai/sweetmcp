//! Committee event system extracted from consensus.rs

use super::evaluation_phases::{EvaluationPhase, RoundStatistics};
use super::steering::{SteeringFeedback, FeedbackType};
use super::super::core::{AgentEvaluation, ConsensusDecision};

/// Committee event enumeration with comprehensive event tracking
#[derive(Debug, Clone)]
pub enum CommitteeEvent {
    /// Evaluation process started
    EvaluationStarted {
        action: String,
        phase: EvaluationPhase,
        agent_count: usize,
        timestamp: std::time::Instant,
    },

    /// Individual agent completed evaluation
    AgentEvaluation {
        agent_id: String,
        phase: EvaluationPhase,
        evaluation: AgentEvaluation,
        execution_time_ms: u64,
    },

    /// Agent evaluation failed
    AgentEvaluationFailed {
        agent_id: String,
        phase: EvaluationPhase,
        error_message: String,
        retry_count: u32,
    },

    /// Evaluation phase completed
    PhaseCompleted {
        phase: EvaluationPhase,
        statistics: RoundStatistics,
        consensus_reached: bool,
        next_phase: Option<EvaluationPhase>,
    },

    /// Steering feedback generated
    SteeringDecision {
        feedback: SteeringFeedback,
        triggered_by_phase: EvaluationPhase,
        round_number: usize,
    },

    /// Final committee decision reached
    FinalDecision {
        action: String,
        decision: ConsensusDecision,
        rounds_completed: usize,
        total_evaluation_time_ms: u64,
        cache_hit: bool,
    },

    /// Consensus threshold reached early
    EarlyConsensus {
        phase: EvaluationPhase,
        decision: ConsensusDecision,
        threshold_exceeded_by: f64,
    },

    /// Committee performance metrics
    PerformanceMetrics {
        total_evaluations: usize,
        successful_evaluations: usize,
        average_evaluation_time_ms: f64,
        cache_hit_rate: f64,
        consensus_rate: f64,
    },

    /// Resource utilization update
    ResourceUtilization {
        active_evaluations: usize,
        queue_length: usize,
        available_permits: usize,
        memory_usage_mb: f64,
    },
}

impl CommitteeEvent {
    /// Get event type as string for logging
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::EvaluationStarted { .. } => "evaluation_started",
            Self::AgentEvaluation { .. } => "agent_evaluation",
            Self::AgentEvaluationFailed { .. } => "agent_evaluation_failed",
            Self::PhaseCompleted { .. } => "phase_completed",
            Self::SteeringDecision { .. } => "steering_decision",
            Self::FinalDecision { .. } => "final_decision",
            Self::EarlyConsensus { .. } => "early_consensus",
            Self::PerformanceMetrics { .. } => "performance_metrics",
            Self::ResourceUtilization { .. } => "resource_utilization",
        }
    }

    /// Get event severity level
    pub fn severity(&self) -> EventSeverity {
        match self {
            Self::AgentEvaluationFailed { .. } => EventSeverity::Warning,
            Self::FinalDecision { .. } => EventSeverity::Info,
            Self::EarlyConsensus { .. } => EventSeverity::Info,
            Self::PerformanceMetrics { .. } => EventSeverity::Debug,
            Self::ResourceUtilization { .. } => EventSeverity::Debug,
            _ => EventSeverity::Trace,
        }
    }

    /// Check if event indicates a problem
    pub fn is_problematic(&self) -> bool {
        match self {
            Self::AgentEvaluationFailed { .. } => true,
            Self::ResourceUtilization { queue_length, .. } => *queue_length > 10,
            _ => false,
        }
    }

    /// Get associated action if available
    pub fn action(&self) -> Option<&str> {
        match self {
            Self::EvaluationStarted { action, .. } => Some(action),
            Self::FinalDecision { action, .. } => Some(action),
            _ => None,
        }
    }

    /// Get associated phase if available
    pub fn phase(&self) -> Option<EvaluationPhase> {
        match self {
            Self::EvaluationStarted { phase, .. } => Some(*phase),
            Self::AgentEvaluation { phase, .. } => Some(*phase),
            Self::AgentEvaluationFailed { phase, .. } => Some(*phase),
            Self::PhaseCompleted { phase, .. } => Some(*phase),
            Self::SteeringDecision { triggered_by_phase, .. } => Some(*triggered_by_phase),
            Self::EarlyConsensus { phase, .. } => Some(*phase),
            _ => None,
        }
    }

    /// Extract performance data if available
    pub fn performance_data(&self) -> Option<PerformanceData> {
        match self {
            Self::AgentEvaluation { execution_time_ms, .. } => {
                Some(PerformanceData {
                    execution_time_ms: *execution_time_ms,
                    success: true,
                    error_count: 0,
                })
            },
            Self::AgentEvaluationFailed { retry_count, .. } => {
                Some(PerformanceData {
                    execution_time_ms: 0,
                    success: false,
                    error_count: *retry_count,
                })
            },
            Self::PhaseCompleted { statistics, .. } => {
                Some(PerformanceData {
                    execution_time_ms: statistics.execution_time_ms,
                    success: statistics.error_count == 0,
                    error_count: statistics.error_count as u32,
                })
            },
            _ => None,
        }
    }

    /// Create evaluation started event
    pub fn evaluation_started(action: String, phase: EvaluationPhase, agent_count: usize) -> Self {
        Self::EvaluationStarted {
            action,
            phase,
            agent_count,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Create agent evaluation event
    pub fn agent_evaluation(
        agent_id: String,
        phase: EvaluationPhase,
        evaluation: AgentEvaluation,
        execution_time_ms: u64,
    ) -> Self {
        Self::AgentEvaluation {
            agent_id,
            phase,
            evaluation,
            execution_time_ms,
        }
    }

    /// Create final decision event
    pub fn final_decision(
        action: String,
        decision: ConsensusDecision,
        rounds_completed: usize,
        total_evaluation_time_ms: u64,
        cache_hit: bool,
    ) -> Self {
        Self::FinalDecision {
            action,
            decision,
            rounds_completed,
            total_evaluation_time_ms,
            cache_hit,
        }
    }
}

/// Event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventSeverity {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

impl EventSeverity {
    /// Check if severity requires immediate attention
    pub fn requires_attention(&self) -> bool {
        matches!(self, Self::Warning | Self::Error)
    }

    /// Get logging level string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG", 
            Self::Info => "INFO",
            Self::Warning => "WARN",
            Self::Error => "ERROR",
        }
    }
}

/// Performance data extracted from events
#[derive(Debug, Clone)]
pub struct PerformanceData {
    pub execution_time_ms: u64,
    pub success: bool,
    pub error_count: u32,
}

impl PerformanceData {
    /// Check if performance is acceptable
    pub fn is_acceptable(&self, max_time_ms: u64, max_errors: u32) -> bool {
        self.success && self.execution_time_ms <= max_time_ms && self.error_count <= max_errors
    }

    /// Get performance score (0.0 to 1.0)
    pub fn score(&self, baseline_time_ms: u64) -> f64 {
        if !self.success {
            return 0.0;
        }

        let time_score = if self.execution_time_ms <= baseline_time_ms {
            1.0
        } else {
            (baseline_time_ms as f64 / self.execution_time_ms as f64).min(1.0)
        };

        let error_penalty = (self.error_count as f64 * 0.1).min(0.5);
        (time_score - error_penalty).max(0.0)
    }
}

/// Event listener trait for extensible event handling
pub trait EventListener: Send + Sync {
    /// Handle a committee event
    fn handle_event(&self, event: &CommitteeEvent);

    /// Check if listener is interested in event type
    fn is_interested(&self, event: &CommitteeEvent) -> bool {
        true // Default: interested in all events
    }

    /// Get listener name for debugging
    fn name(&self) -> &'static str;
}

/// Event bus for distributing committee events
pub struct EventBus {
    listeners: Vec<Box<dyn EventListener>>,
    event_count: std::sync::atomic::AtomicU64,
}

impl EventBus {
    /// Create new event bus
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
            event_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Add event listener
    pub fn add_listener(&mut self, listener: Box<dyn EventListener>) {
        self.listeners.push(listener);
    }

    /// Publish event to all interested listeners
    pub fn publish(&self, event: &CommitteeEvent) {
        self.event_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        for listener in &self.listeners {
            if listener.is_interested(event) {
                listener.handle_event(event);
            }
        }
    }

    /// Get total event count
    pub fn event_count(&self) -> u64 {
        self.event_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get number of registered listeners
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in logging event listener
pub struct LoggingListener {
    min_severity: EventSeverity,
}

impl LoggingListener {
    /// Create new logging listener
    pub fn new(min_severity: EventSeverity) -> Self {
        Self { min_severity }
    }
}

impl EventListener for LoggingListener {
    fn handle_event(&self, event: &CommitteeEvent) {
        let severity = event.severity();
        if severity >= self.min_severity {
            let event_type = event.event_type();
            let action = event.action();
            let phase = event.phase();
            let message = format!("Committee event: {:?}", event);
            
            match severity {
                EventSeverity::Trace => tracing::trace!(target: "committee", event_type, ?action, ?phase, "{}", message),
                EventSeverity::Debug => tracing::debug!(target: "committee", event_type, ?action, ?phase, "{}", message),
                EventSeverity::Info => tracing::info!(target: "committee", event_type, ?action, ?phase, "{}", message),
                EventSeverity::Warning => tracing::warn!(target: "committee", event_type, ?action, ?phase, "{}", message),
                EventSeverity::Error => tracing::error!(target: "committee", event_type, ?action, ?phase, "{}", message),
            }
        }
    }

    fn is_interested(&self, event: &CommitteeEvent) -> bool {
        event.severity() >= self.min_severity
    }

    fn name(&self) -> &'static str {
        "LoggingListener"
    }
}