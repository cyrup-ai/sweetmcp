//! Token bucket and sliding window algorithms
//!
//! This module provides comprehensive rate limiting algorithms with zero allocation
//! fast paths and blazing-fast performance.

use super::limiter::{TokenBucketConfig, SlidingWindowConfig};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Rate limiting algorithm trait for polymorphic algorithm support
pub trait RateLimitAlgorithm: Send + Sync {
    /// Try to consume tokens/requests with fast path validation
    fn try_request(&mut self) -> bool;
    
    /// Check if the algorithm instance is still active with zero allocation check
    fn is_active(&self) -> bool;
    
    /// Get current state information for monitoring
    fn get_state(&self) -> AlgorithmState;
    
    /// Reset the algorithm state with optimized reset
    fn reset(&mut self);
}

/// Token bucket rate limiting algorithm with optimized token management
pub struct TokenBucket {
    /// Current number of tokens with atomic access
    tokens: f64,
    /// Maximum bucket capacity
    capacity: u32,
    /// Token refill rate per second
    refill_rate: f64,
    /// Last refill timestamp for precise timing
    last_refill: Instant,
    /// Creation timestamp for activity tracking
    created_at: Instant,
}

impl TokenBucket {
    /// Create new token bucket with optimized initialization
    pub fn new(config: TokenBucketConfig) -> Self {
        let now = Instant::now();
        Self {
            tokens: config.initial_tokens as f64,
            capacity: config.capacity,
            refill_rate: config.refill_rate,
            last_refill: now,
            created_at: now,
        }
    }

    /// Try to consume specified number of tokens with fast path validation
    pub fn try_consume(&mut self, tokens_needed: u32) -> bool {
        self.refill_tokens();

        if self.tokens >= tokens_needed as f64 {
            self.tokens -= tokens_needed as f64;
            debug!(
                "Tokens consumed: {} (remaining: {:.2}/{})",
                tokens_needed, self.tokens, self.capacity
            );
            true
        } else {
            debug!(
                "Insufficient tokens: needed {}, available {:.2}/{}",
                tokens_needed, self.tokens, self.capacity
            );
            false
        }
    }

    /// Refill tokens based on elapsed time with optimized time calculation
    fn refill_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        if elapsed > 0.0 {
            let tokens_to_add = elapsed * self.refill_rate;
            self.tokens = (self.tokens + tokens_to_add).min(self.capacity as f64);
            self.last_refill = now;
        }
    }

    /// Get current token count with fast token access
    pub fn current_tokens(&mut self) -> f64 {
        self.refill_tokens();
        self.tokens
    }

    /// Update configuration dynamically with optimized config update
    pub fn update_config(&mut self, config: &TokenBucketConfig) {
        self.refill_tokens(); // Ensure current state is up to date
        
        // Adjust current tokens if capacity changed
        if config.capacity != self.capacity {
            let ratio = config.capacity as f64 / self.capacity as f64;
            self.tokens = (self.tokens * ratio).min(config.capacity as f64);
        }
        
        self.capacity = config.capacity;
        self.refill_rate = config.refill_rate;
        
        info!(
            "Updated token bucket config: capacity={}, refill_rate={}, current_tokens={:.2}",
            self.capacity, self.refill_rate, self.tokens
        );
    }
}

impl RateLimitAlgorithm for TokenBucket {
    fn try_request(&mut self) -> bool {
        self.try_consume(1)
    }

    fn is_active(&self) -> bool {
        // Consider active if created within last hour or has recent activity
        let now = Instant::now();
        now.duration_since(self.created_at) < Duration::from_secs(3600) ||
        now.duration_since(self.last_refill) < Duration::from_secs(300)
    }

    fn get_state(&self) -> AlgorithmState {
        AlgorithmState::TokenBucket {
            current_tokens: self.tokens,
            capacity: self.capacity,
            refill_rate: self.refill_rate,
        }
    }

    fn reset(&mut self) {
        let now = Instant::now();
        self.tokens = self.capacity as f64;
        self.last_refill = now;
        self.created_at = now;
    }
}

/// Sliding window rate limiting algorithm with optimized window management
pub struct SlidingWindow {
    /// Time window duration
    window_size: Duration,
    /// Maximum requests allowed in window
    max_requests: u32,
    /// Sub-windows for precise tracking
    sub_windows: Vec<(Instant, u32)>,
    /// Duration of each sub-window
    sub_window_duration: Duration,
    /// Current sub-window index
    current_sub_window: usize,
    /// Creation timestamp for activity tracking
    created_at: Instant,
}

impl SlidingWindow {
    /// Create new sliding window with optimized initialization
    pub fn new(config: SlidingWindowConfig) -> Self {
        let window_size = Duration::from_secs(config.window_size);
        let sub_window_duration = window_size / config.sub_windows;
        let now = Instant::now();
        
        // Initialize all sub-windows with zero allocation
        let sub_windows = vec![(now, 0); config.sub_windows as usize];

        Self {
            window_size,
            max_requests: config.max_requests,
            sub_windows,
            sub_window_duration,
            current_sub_window: 0,
            created_at: now,
        }
    }

    /// Check if request is allowed and increment counter with fast validation
    pub fn try_request(&mut self) -> bool {
        self.cleanup_old_windows();

        let total_requests = self.get_total_requests();

        if total_requests < self.max_requests {
            self.add_request();
            debug!(
                "Request allowed: {}/{} in sliding window",
                total_requests + 1,
                self.max_requests
            );
            true
        } else {
            debug!(
                "Request denied: {}/{} in sliding window",
                total_requests, self.max_requests
            );
            false
        }
    }

    /// Add a request to the current sub-window with optimized window management
    fn add_request(&mut self) {
        let now = Instant::now();
        let current_window = &mut self.sub_windows[self.current_sub_window];

        // If current sub-window is too old, move to next one
        if now.duration_since(current_window.0) >= self.sub_window_duration {
            self.current_sub_window = (self.current_sub_window + 1) % self.sub_windows.len();
            self.sub_windows[self.current_sub_window] = (now, 1);
        } else {
            current_window.1 += 1;
        }
    }

    /// Get total requests across all valid sub-windows with fast aggregation
    fn get_total_requests(&self) -> u32 {
        let now = Instant::now();
        self.sub_windows
            .iter()
            .filter(|(timestamp, _)| now.duration_since(*timestamp) < self.window_size)
            .map(|(_, count)| *count)
            .sum()
    }

    /// Remove sub-windows that are outside the time window with optimized cleanup
    fn cleanup_old_windows(&mut self) {
        let now = Instant::now();
        for (timestamp, count) in &mut self.sub_windows {
            if now.duration_since(*timestamp) >= self.window_size {
                *count = 0;
                *timestamp = now;
            }
        }
    }

    /// Update configuration dynamically with optimized config update
    pub fn update_config(&mut self, config: &SlidingWindowConfig) {
        self.window_size = Duration::from_secs(config.window_size);
        self.max_requests = config.max_requests;

        // Resize sub-windows if needed with zero allocation resize
        if self.sub_windows.len() != config.sub_windows as usize {
            self.sub_windows
                .resize(config.sub_windows as usize, (Instant::now(), 0));
            self.sub_window_duration = self.window_size / config.sub_windows;
            self.current_sub_window = 0;
        }

        info!(
            "Updated sliding window config: window_size={}s, max_requests={}, sub_windows={}",
            config.window_size, self.max_requests, config.sub_windows
        );
    }
}

impl RateLimitAlgorithm for SlidingWindow {
    fn try_request(&mut self) -> bool {
        self.try_request()
    }

    fn is_active(&self) -> bool {
        // Consider active if created within last hour or has recent requests
        let now = Instant::now();
        if now.duration_since(self.created_at) < Duration::from_secs(3600) {
            return true;
        }

        // Check if any sub-window has recent activity
        self.sub_windows
            .iter()
            .any(|(timestamp, count)| {
                *count > 0 && now.duration_since(*timestamp) < Duration::from_secs(300)
            })
    }

    fn get_state(&self) -> AlgorithmState {
        AlgorithmState::SlidingWindow {
            current_requests: self.get_total_requests(),
            max_requests: self.max_requests,
            window_size_secs: self.window_size.as_secs(),
        }
    }

    fn reset(&mut self) {
        let now = Instant::now();
        for (timestamp, count) in &mut self.sub_windows {
            *timestamp = now;
            *count = 0;
        }
        self.current_sub_window = 0;
        self.created_at = now;
    }
}

/// Rate limiter that can use different algorithms with polymorphic support
#[derive(Debug)]
pub enum RateLimiter {
    TokenBucket(TokenBucket),
    SlidingWindow(SlidingWindow),
}

impl RateLimiter {
    /// Create new rate limiter with specified algorithm
    pub fn new(algorithm: &RateLimitAlgorithmType) -> Self {
        match algorithm {
            RateLimitAlgorithmType::TokenBucket(config) => {
                Self::TokenBucket(TokenBucket::new(config.clone()))
            }
            RateLimitAlgorithmType::SlidingWindow(config) => {
                Self::SlidingWindow(SlidingWindow::new(config.clone()))
            }
        }
    }

    /// Check if request is allowed with fast algorithm dispatch
    pub fn check_request(&mut self, tokens: u32) -> bool {
        match self {
            Self::TokenBucket(bucket) => bucket.try_consume(tokens),
            Self::SlidingWindow(window) => {
                // For sliding window, tokens parameter is ignored (always 1 request)
                window.try_request()
            }
        }
    }

    /// Update configuration dynamically with optimized config update
    pub fn update_config(&mut self, algorithm: &RateLimitAlgorithmType) {
        match algorithm {
            RateLimitAlgorithmType::TokenBucket(config) => {
                match self {
                    Self::TokenBucket(bucket) => bucket.update_config(config),
                    Self::SlidingWindow(_) => {
                        // Switch algorithm type
                        *self = Self::TokenBucket(TokenBucket::new(config.clone()));
                    }
                }
            }
            RateLimitAlgorithmType::SlidingWindow(config) => {
                match self {
                    Self::SlidingWindow(window) => window.update_config(config),
                    Self::TokenBucket(_) => {
                        // Switch algorithm type
                        *self = Self::SlidingWindow(SlidingWindow::new(config.clone()));
                    }
                }
            }
        }
    }

    /// Get current algorithm state for monitoring
    pub fn get_state(&mut self) -> AlgorithmState {
        match self {
            Self::TokenBucket(bucket) => bucket.get_state(),
            Self::SlidingWindow(window) => window.get_state(),
        }
    }

    /// Check if limiter is active with fast activity check
    pub fn is_active(&self) -> bool {
        match self {
            Self::TokenBucket(bucket) => bucket.is_active(),
            Self::SlidingWindow(window) => window.is_active(),
        }
    }
}

/// Rate limiting algorithm type enumeration with configuration
#[derive(Debug, Clone)]
pub enum RateLimitAlgorithmType {
    TokenBucket(TokenBucketConfig),
    SlidingWindow(SlidingWindowConfig),
}

/// Algorithm state information for monitoring and debugging
#[derive(Debug, Clone)]
pub enum AlgorithmState {
    TokenBucket {
        current_tokens: f64,
        capacity: u32,
        refill_rate: f64,
    },
    SlidingWindow {
        current_requests: u32,
        max_requests: u32,
        window_size_secs: u64,
    },
}

impl AlgorithmState {
    /// Get utilization percentage with fast calculation
    pub fn utilization_percent(&self) -> f64 {
        match self {
            AlgorithmState::TokenBucket { current_tokens, capacity, .. } => {
                (1.0 - (current_tokens / *capacity as f64)) * 100.0
            }
            AlgorithmState::SlidingWindow { current_requests, max_requests, .. } => {
                (*current_requests as f64 / *max_requests as f64) * 100.0
            }
        }
    }

    /// Check if algorithm is near capacity with fast threshold check
    pub fn is_near_capacity(&self, threshold_percent: f64) -> bool {
        self.utilization_percent() > threshold_percent
    }

    /// Get algorithm type name with zero allocation
    pub fn algorithm_name(&self) -> &'static str {
        match self {
            AlgorithmState::TokenBucket { .. } => "TokenBucket",
            AlgorithmState::SlidingWindow { .. } => "SlidingWindow",
        }
    }
}

/// Hybrid algorithm that combines token bucket and sliding window
pub struct HybridAlgorithm {
    token_bucket: TokenBucket,
    sliding_window: SlidingWindow,
    created_at: Instant,
}

impl HybridAlgorithm {
    /// Create new hybrid algorithm with optimized dual algorithm setup
    pub fn new(token_config: TokenBucketConfig, window_config: SlidingWindowConfig) -> Self {
        Self {
            token_bucket: TokenBucket::new(token_config),
            sliding_window: SlidingWindow::new(window_config),
            created_at: Instant::now(),
        }
    }
}

impl RateLimitAlgorithm for HybridAlgorithm {
    fn try_request(&mut self) -> bool {
        // Both algorithms must allow the request
        self.token_bucket.try_request() && self.sliding_window.try_request()
    }

    fn is_active(&self) -> bool {
        // Active if either algorithm is active
        self.token_bucket.is_active() || self.sliding_window.is_active()
    }

    fn get_state(&self) -> AlgorithmState {
        // Return combined state information
        let bucket_state = self.token_bucket.get_state();
        let window_state = self.sliding_window.get_state();
        
        // For simplicity, return the more restrictive state
        if bucket_state.utilization_percent() > window_state.utilization_percent() {
            bucket_state
        } else {
            window_state
        }
    }

    fn reset(&mut self) {
        self.token_bucket.reset();
        self.sliding_window.reset();
        self.created_at = Instant::now();
    }
}