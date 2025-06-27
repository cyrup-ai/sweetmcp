//! Circuit breaker implementation for preventing cascading failures
//!
//! This module provides:
//! - Hystrix-style circuit breaker per peer
//! - Configurable error thresholds
//! - Half-open state testing
//! - Metrics emission for state changes

#![allow(dead_code)]

use once_cell::sync::Lazy;
use prometheus::{register_int_counter, register_int_gauge, IntCounter, IntGauge};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed - requests flow normally
    Closed,
    /// Circuit is open - requests are rejected
    Open,
    /// Circuit is half-open - testing if service recovered
    HalfOpen,
}

/// Configuration for circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Error threshold percentage to open circuit (0-100)
    pub error_threshold_percentage: u32,
    /// Minimum number of requests before calculating error percentage
    pub request_volume_threshold: u32,
    /// Duration to wait before entering half-open state
    pub sleep_window: Duration,
    /// Number of requests to allow in half-open state
    pub half_open_requests: u32,
    /// Time window for calculating error rate
    pub metrics_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            error_threshold_percentage: 50,
            request_volume_threshold: 20,
            sleep_window: Duration::from_secs(5),
            half_open_requests: 3,
            metrics_window: Duration::from_secs(10),
        }
    }
}

/// Circuit breaker for a single peer
pub struct CircuitBreaker {
    /// Configuration
    config: CircuitBreakerConfig,
    /// Current state
    state: Arc<RwLock<CircuitState>>,
    /// Total requests in current window
    total_requests: AtomicU64,
    /// Failed requests in current window
    failed_requests: AtomicU64,
    /// Timestamp when circuit was opened
    opened_at: Arc<RwLock<Option<Instant>>>,
    /// Requests allowed in half-open state
    half_open_remaining: AtomicU32,
    /// Window start time
    window_start: Arc<RwLock<Instant>>,
    /// Peer identifier for metrics
    peer_id: String,
}

// Prometheus metrics
static CIRCUIT_STATE_GAUGE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "sweetmcp_circuit_breaker_state",
        "Current circuit breaker state (0=closed, 1=open, 2=half-open)"
    )
    .unwrap_or_else(|e| {
        tracing::warn!("Failed to register circuit breaker state gauge: {}", e);
        IntGauge::new(
            "sweetmcp_circuit_breaker_state_fallback",
            "Fallback circuit breaker state gauge",
        )
        .unwrap_or_else(|e| {
            tracing::error!("Critical: Cannot create circuit breaker gauge: {}", e);
            std::process::exit(1)
        })
    })
});

static CIRCUIT_OPENED_COUNTER: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "sweetmcp_circuit_breaker_opened_total",
        "Total number of times circuit breaker opened"
    )
    .unwrap_or_else(|e| {
        tracing::warn!("Failed to register circuit breaker opened counter: {}", e);
        IntCounter::new(
            "sweetmcp_circuit_breaker_opened_total_fallback",
            "Fallback circuit breaker opened counter",
        )
        .unwrap_or_else(|e| {
            tracing::error!("Critical: Cannot create circuit breaker counter: {}", e);
            std::process::exit(1)
        })
    })
});

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(peer_id: String, config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            total_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            opened_at: Arc::new(RwLock::new(None)),
            half_open_remaining: AtomicU32::new(0),
            window_start: Arc::new(RwLock::new(Instant::now())),
            peer_id,
        }
    }

    /// Check if request should be allowed
    pub async fn should_allow_request(&self) -> bool {
        // Check if window needs reset
        self.maybe_reset_window().await;

        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to half-open
                if self.should_attempt_reset().await {
                    self.transition_to_half_open().await;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let remaining = self.half_open_remaining.fetch_sub(1, Ordering::SeqCst);
                remaining > 0
            }
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);

        let state = *self.state.read().await;

        match state {
            CircuitState::HalfOpen => {
                // Check if we should close the circuit
                let total = self.total_requests.load(Ordering::SeqCst);
                let failed = self.failed_requests.load(Ordering::SeqCst);

                if total >= self.config.half_open_requests as u64 {
                    let error_percentage = if total > 0 { (failed * 100) / total } else { 0 };

                    if error_percentage < self.config.error_threshold_percentage as u64 {
                        self.transition_to_closed().await;
                    } else {
                        self.transition_to_open().await;
                    }
                }
            }
            _ => {
                // Just record the success
            }
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        self.failed_requests.fetch_add(1, Ordering::SeqCst);

        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                // Check if we should open the circuit
                if self.should_trip().await {
                    self.transition_to_open().await;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state reopens the circuit
                self.transition_to_open().await;
            }
            _ => {
                // Already open
            }
        }
    }

    /// Check if circuit should trip (open)
    async fn should_trip(&self) -> bool {
        let total = self.total_requests.load(Ordering::SeqCst);
        let failed = self.failed_requests.load(Ordering::SeqCst);

        // Need minimum volume before making decisions
        if total < self.config.request_volume_threshold as u64 {
            return false;
        }

        let error_percentage = if total > 0 { (failed * 100) / total } else { 0 };

        error_percentage >= self.config.error_threshold_percentage as u64
    }

    /// Check if we should attempt to reset (transition to half-open)
    async fn should_attempt_reset(&self) -> bool {
        let opened_at = self.opened_at.read().await;
        if let Some(opened_time) = *opened_at {
            opened_time.elapsed() >= self.config.sleep_window
        } else {
            false
        }
    }

    /// Transition to open state
    async fn transition_to_open(&self) {
        let mut state = self.state.write().await;
        if *state != CircuitState::Open {
            *state = CircuitState::Open;
            *self.opened_at.write().await = Some(Instant::now());

            warn!("Circuit breaker opened for peer: {}", self.peer_id);
            CIRCUIT_STATE_GAUGE.set(1);
            CIRCUIT_OPENED_COUNTER.inc();

            // Emit metrics
            crate::metrics::record_circuit_breaker_state(&self.peer_id, "open");
        }
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        if *state != CircuitState::HalfOpen {
            *state = CircuitState::HalfOpen;
            self.half_open_remaining
                .store(self.config.half_open_requests, Ordering::SeqCst);

            // Reset counters for half-open testing
            self.total_requests.store(0, Ordering::SeqCst);
            self.failed_requests.store(0, Ordering::SeqCst);

            info!("Circuit breaker half-open for peer: {}", self.peer_id);
            CIRCUIT_STATE_GAUGE.set(2);

            // Emit metrics
            crate::metrics::record_circuit_breaker_state(&self.peer_id, "half_open");
        }
    }

    /// Transition to closed state
    async fn transition_to_closed(&self) {
        let mut state = self.state.write().await;
        if *state != CircuitState::Closed {
            *state = CircuitState::Closed;
            *self.opened_at.write().await = None;

            info!("Circuit breaker closed for peer: {}", self.peer_id);
            CIRCUIT_STATE_GAUGE.set(0);

            // Emit metrics
            crate::metrics::record_circuit_breaker_state(&self.peer_id, "closed");
        }
    }

    /// Reset metrics window if needed
    async fn maybe_reset_window(&self) {
        let mut window_start = self.window_start.write().await;
        if window_start.elapsed() >= self.config.metrics_window {
            *window_start = Instant::now();
            self.total_requests.store(0, Ordering::SeqCst);
            self.failed_requests.store(0, Ordering::SeqCst);
            debug!("Reset metrics window for peer: {}", self.peer_id);
        }
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> (u64, u64) {
        (
            self.total_requests.load(Ordering::SeqCst),
            self.failed_requests.load(Ordering::SeqCst),
        )
    }
}

/// Circuit breaker manager for all peers
pub struct CircuitBreakerManager {
    /// Circuit breakers per peer
    breakers: Arc<RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>>,
    /// Default configuration
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    /// Create a new manager
    pub fn new(default_config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            default_config,
        }
    }

    /// Get or create circuit breaker for a peer
    pub async fn get_breaker(&self, peer_id: &str) -> Arc<CircuitBreaker> {
        let breakers = self.breakers.read().await;
        if let Some(breaker) = breakers.get(peer_id) {
            return breaker.clone();
        }
        drop(breakers);

        // Create new breaker
        let mut breakers = self.breakers.write().await;
        let breaker = Arc::new(CircuitBreaker::new(
            peer_id.to_string(),
            self.default_config.clone(),
        ));
        breakers.insert(peer_id.to_string(), breaker.clone());
        breaker
    }

    /// Remove a circuit breaker
    pub async fn remove_breaker(&self, peer_id: &str) {
        let mut breakers = self.breakers.write().await;
        breakers.remove(peer_id);
    }

    /// Get all circuit breakers
    pub async fn get_all_breakers(&self) -> Vec<(String, Arc<CircuitBreaker>)> {
        let breakers = self.breakers.read().await;
        breakers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
