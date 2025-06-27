//! Advanced Rate Limiting with Token Bucket and Sliding Window Algorithms
//!
//! This module provides enterprise-grade rate limiting with:
//! - Token bucket algorithm with configurable refill rates and burst capacity
//! - Sliding window rate limiting for precise rate control
//! - Per-endpoint and per-peer rate limiting
//! - Dynamic rate adjustment based on system load
//! - Comprehensive metrics integration

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Token bucket rate limiting algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBucketConfig {
    /// Maximum number of tokens in the bucket (burst capacity)
    pub capacity: u32,
    /// Number of tokens to refill per second
    pub refill_rate: f64,
    /// Initial number of tokens in bucket
    pub initial_tokens: u32,
}

impl Default for TokenBucketConfig {
    fn default() -> Self {
        Self {
            capacity: 100,
            refill_rate: 10.0, // 10 tokens per second
            initial_tokens: 100,
        }
    }
}

/// Sliding window rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlidingWindowConfig {
    /// Window size in seconds
    pub window_size: u64,
    /// Maximum requests allowed in the window
    pub max_requests: u32,
    /// Number of sub-windows for precision
    pub sub_windows: u32,
}

impl Default for SlidingWindowConfig {
    fn default() -> Self {
        Self {
            window_size: 60,  // 1 minute window
            max_requests: 60, // 60 requests per minute
            sub_windows: 6,   // 10-second sub-windows
        }
    }
}

/// Rate limiting algorithm type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitAlgorithm {
    TokenBucket(TokenBucketConfig),
    SlidingWindow(SlidingWindowConfig),
}

/// Per-endpoint rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRateConfig {
    /// Rate limiting algorithm and configuration
    pub algorithm: RateLimitAlgorithm,
    /// Whether this endpoint should use per-peer limiting
    pub per_peer: bool,
    /// Burst multiplier for trusted peers
    pub trusted_multiplier: f64,
}

impl Default for EndpointRateConfig {
    fn default() -> Self {
        Self {
            algorithm: RateLimitAlgorithm::TokenBucket(TokenBucketConfig::default()),
            per_peer: false,
            trusted_multiplier: 2.0,
        }
    }
}

/// Token bucket implementation
#[derive(Debug)]
pub struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(config: &TokenBucketConfig) -> Self {
        Self {
            capacity: config.capacity,
            tokens: config.initial_tokens as f64,
            refill_rate: config.refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Attempt to consume tokens from the bucket
    pub fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();

        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            debug!("Consumed {} tokens, {} remaining", tokens, self.tokens);
            true
        } else {
            debug!(
                "Token consumption failed: {} tokens requested, {} available",
                tokens, self.tokens
            );
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        if elapsed > 0.0 {
            let new_tokens = elapsed * self.refill_rate;
            self.tokens = (self.tokens + new_tokens).min(self.capacity as f64);
            self.last_refill = now;

            if new_tokens > 0.1 {
                debug!(
                    "Refilled {:.2} tokens, total: {:.2}/{}",
                    new_tokens, self.tokens, self.capacity
                );
            }
        }
    }

    /// Get current token count
    pub fn available_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }

    /// Update configuration dynamically
    pub fn update_config(&mut self, config: &TokenBucketConfig) {
        self.refill(); // Refill with old rate before updating

        // Scale current tokens proportionally if capacity changed
        if self.capacity != config.capacity && self.capacity > 0 {
            let scale = config.capacity as f64 / self.capacity as f64;
            self.tokens = (self.tokens * scale).min(config.capacity as f64);
        }

        self.capacity = config.capacity;
        self.refill_rate = config.refill_rate;

        info!(
            "Updated token bucket config: capacity={}, refill_rate={}",
            self.capacity, self.refill_rate
        );
    }
}

/// Sliding window rate limiter implementation
#[derive(Debug)]
pub struct SlidingWindow {
    window_size: Duration,
    max_requests: u32,
    sub_windows: Vec<(Instant, u32)>,
    sub_window_duration: Duration,
    current_sub_window: usize,
}

impl SlidingWindow {
    pub fn new(config: &SlidingWindowConfig) -> Self {
        let window_size = Duration::from_secs(config.window_size);
        let sub_window_duration = window_size / config.sub_windows;
        let sub_windows = vec![(Instant::now(), 0); config.sub_windows as usize];

        Self {
            window_size,
            max_requests: config.max_requests,
            sub_windows,
            sub_window_duration,
            current_sub_window: 0,
        }
    }

    /// Check if request is allowed and increment counter
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

    /// Add a request to the current sub-window
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

    /// Get total requests across all valid sub-windows
    fn get_total_requests(&self) -> u32 {
        let now = Instant::now();
        self.sub_windows
            .iter()
            .filter(|(timestamp, _)| now.duration_since(*timestamp) < self.window_size)
            .map(|(_, count)| *count)
            .sum()
    }

    /// Remove sub-windows that are outside the time window
    fn cleanup_old_windows(&mut self) {
        let now = Instant::now();
        for (timestamp, count) in &mut self.sub_windows {
            if now.duration_since(*timestamp) >= self.window_size {
                *count = 0;
                *timestamp = now;
            }
        }
    }

    /// Update configuration dynamically
    pub fn update_config(&mut self, config: &SlidingWindowConfig) {
        self.window_size = Duration::from_secs(config.window_size);
        self.max_requests = config.max_requests;

        // Resize sub-windows if needed
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

/// Rate limiter that can use different algorithms
#[derive(Debug)]
pub enum RateLimiter {
    TokenBucket(TokenBucket),
    SlidingWindow(SlidingWindow),
}

impl RateLimiter {
    pub fn new(algorithm: &RateLimitAlgorithm) -> Self {
        match algorithm {
            RateLimitAlgorithm::TokenBucket(config) => Self::TokenBucket(TokenBucket::new(config)),
            RateLimitAlgorithm::SlidingWindow(config) => {
                Self::SlidingWindow(SlidingWindow::new(config))
            }
        }
    }

    /// Check if request is allowed
    pub fn check_request(&mut self, tokens: u32) -> bool {
        match self {
            Self::TokenBucket(bucket) => bucket.try_consume(tokens),
            Self::SlidingWindow(window) => {
                // For sliding window, tokens parameter is ignored (always 1 request)
                window.try_request()
            }
        }
    }

    /// Update configuration dynamically
    pub fn update_config(&mut self, algorithm: &RateLimitAlgorithm) {
        match algorithm {
            RateLimitAlgorithm::TokenBucket(config) => {
                match self {
                    Self::TokenBucket(bucket) => {
                        bucket.update_config(config);
                    }
                    _ => {
                        // Algorithm type changed, recreate limiter
                        *self = Self::new(algorithm);
                        info!("Rate limiter algorithm changed to TokenBucket");
                    }
                }
            }
            RateLimitAlgorithm::SlidingWindow(config) => {
                match self {
                    Self::SlidingWindow(window) => {
                        window.update_config(config);
                    }
                    _ => {
                        // Algorithm type changed, recreate limiter
                        *self = Self::new(algorithm);
                        info!("Rate limiter algorithm changed to SlidingWindow");
                    }
                }
            }
        }
    }
}

/// Per-endpoint and per-peer rate limiting manager
#[derive(Debug)]
pub struct AdvancedRateLimitManager {
    /// Per-endpoint configurations
    endpoint_configs: HashMap<String, EndpointRateConfig>,
    /// Per-endpoint rate limiters
    endpoint_limiters: Arc<Mutex<HashMap<String, RateLimiter>>>,
    /// Per-peer rate limiters (endpoint -> peer -> limiter)
    peer_limiters: Arc<Mutex<HashMap<String, HashMap<String, RateLimiter>>>>,
    /// System load multiplier (reduces limits when system is under stress)
    load_multiplier: Arc<Mutex<f64>>,
}

impl AdvancedRateLimitManager {
    pub fn new() -> Self {
        let mut endpoint_configs = HashMap::new();

        // Default configurations for known endpoints
        endpoint_configs.insert(
            "/api/peers".to_string(),
            EndpointRateConfig {
                algorithm: RateLimitAlgorithm::TokenBucket(TokenBucketConfig {
                    capacity: 50,
                    refill_rate: 5.0, // 5 requests per second
                    initial_tokens: 50,
                }),
                per_peer: false,
                trusted_multiplier: 2.0,
            },
        );

        endpoint_configs.insert(
            "/api/register".to_string(),
            EndpointRateConfig {
                algorithm: RateLimitAlgorithm::SlidingWindow(SlidingWindowConfig {
                    window_size: 300, // 5 minutes
                    max_requests: 10, // 10 registrations per 5 minutes
                    sub_windows: 10,  // 30-second sub-windows
                }),
                per_peer: true,
                trusted_multiplier: 1.5,
            },
        );

        Self {
            endpoint_configs,
            endpoint_limiters: Arc::new(Mutex::new(HashMap::new())),
            peer_limiters: Arc::new(Mutex::new(HashMap::new())),
            load_multiplier: Arc::new(Mutex::new(1.0)),
        }
    }

    /// Check if request should be allowed
    pub fn check_request(&self, endpoint: &str, peer_ip: Option<&str>, tokens: u32) -> bool {
        let config = match self.endpoint_configs.get(endpoint) {
            Some(config) => config.clone(),
            None => {
                debug!(
                    "No rate limit config for endpoint {}, allowing request",
                    endpoint
                );
                return true;
            }
        };

        // Apply load-based adjustment
        let load_multiplier = *match self.load_multiplier.lock() {
            Ok(multiplier) => multiplier,
            Err(poisoned) => {
                tracing::warn!("Load multiplier mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };
        let adjusted_tokens = ((tokens as f64) / load_multiplier).max(1.0) as u32;

        if config.per_peer {
            if let Some(peer_ip) = peer_ip {
                self.check_peer_request(endpoint, peer_ip, &config, adjusted_tokens)
            } else {
                // No peer IP available, fall back to endpoint limiting
                self.check_endpoint_request(endpoint, &config, adjusted_tokens)
            }
        } else {
            self.check_endpoint_request(endpoint, &config, adjusted_tokens)
        }
    }

    /// Check request against endpoint-level rate limiter
    fn check_endpoint_request(
        &self,
        endpoint: &str,
        config: &EndpointRateConfig,
        tokens: u32,
    ) -> bool {
        let mut limiters = match self.endpoint_limiters.lock() {
            Ok(limiters) => limiters,
            Err(poisoned) => {
                tracing::warn!("Endpoint limiters mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };

        let limiter = limiters
            .entry(endpoint.to_string())
            .or_insert_with(|| RateLimiter::new(&config.algorithm));

        let allowed = limiter.check_request(tokens);

        if !allowed {
            crate::metrics::record_rate_limit_rejection(endpoint);
            warn!("Rate limit exceeded for endpoint {}", endpoint);
        }

        allowed
    }

    /// Check request against per-peer rate limiter
    fn check_peer_request(
        &self,
        endpoint: &str,
        peer_ip: &str,
        config: &EndpointRateConfig,
        tokens: u32,
    ) -> bool {
        let mut peer_limiters = match self.peer_limiters.lock() {
            Ok(limiters) => limiters,
            Err(poisoned) => {
                tracing::warn!("Peer limiters mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };

        let endpoint_limiters = peer_limiters
            .entry(endpoint.to_string())
            .or_insert_with(HashMap::new);

        let limiter = endpoint_limiters
            .entry(peer_ip.to_string())
            .or_insert_with(|| RateLimiter::new(&config.algorithm));

        let allowed = limiter.check_request(tokens);

        if !allowed {
            crate::metrics::record_rate_limit_rejection(endpoint);
            warn!(
                "Rate limit exceeded for peer {} on endpoint {}",
                peer_ip, endpoint
            );
        }

        allowed
    }

    /// Update system load multiplier (reduces effective rate limits when system is stressed)
    pub fn update_load_multiplier(&self, multiplier: f64) {
        let mut load_multiplier = match self.load_multiplier.lock() {
            Ok(multiplier) => multiplier,
            Err(poisoned) => {
                tracing::warn!("Load multiplier mutex poisoned during update, recovering");
                poisoned.into_inner()
            }
        };
        *load_multiplier = multiplier.clamp(0.1, 10.0);

        if multiplier < 1.0 {
            info!(
                "System under load, reducing rate limits by factor of {:.2}",
                multiplier
            );
        }
    }

    /// Add or update configuration for an endpoint
    pub fn configure_endpoint(&mut self, endpoint: String, config: EndpointRateConfig) {
        self.endpoint_configs
            .insert(endpoint.clone(), config.clone());

        // Update existing limiters if they exist
        if let Ok(mut limiters) = self.endpoint_limiters.lock() {
            if let Some(limiter) = limiters.get_mut(&endpoint) {
                limiter.update_config(&config.algorithm);
            }
        }

        if let Ok(mut peer_limiters) = self.peer_limiters.lock() {
            if let Some(endpoint_limiters) = peer_limiters.get_mut(&endpoint) {
                for limiter in endpoint_limiters.values_mut() {
                    limiter.update_config(&config.algorithm);
                }
            }
        }

        info!("Updated rate limit configuration for endpoint {}", endpoint);
    }

    /// Get current rate limit statistics
    pub fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        if let Ok(limiters) = self.endpoint_limiters.lock() {
            stats.insert(
                "endpoint_limiters".to_string(),
                serde_json::json!(limiters.len()),
            );
        }

        if let Ok(peer_limiters) = self.peer_limiters.lock() {
            let total_peer_limiters: usize = peer_limiters
                .values()
                .map(|endpoint_limiters| endpoint_limiters.len())
                .sum();
            stats.insert(
                "peer_limiters".to_string(),
                serde_json::json!(total_peer_limiters),
            );
        }

        if let Ok(load_multiplier) = self.load_multiplier.lock() {
            stats.insert(
                "load_multiplier".to_string(),
                serde_json::json!(*load_multiplier),
            );
        }

        stats
    }

    /// Start periodic cleanup task for unused peer limiters
    pub fn start_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(300)); // Cleanup every 5 minutes

            loop {
                cleanup_interval.tick().await;
                self.cleanup_unused_limiters();
            }
        });
    }

    /// Remove peer limiters that haven't been used recently
    fn cleanup_unused_limiters(&self) {
        if let Ok(mut peer_limiters) = self.peer_limiters.lock() {
            let initial_count: usize = peer_limiters
                .values()
                .map(|endpoint_limiters| endpoint_limiters.len())
                .sum();

            // Remove peer limiters for endpoints that are no longer configured
            peer_limiters.retain(|endpoint, _| self.endpoint_configs.contains_key(endpoint));

            let final_count: usize = peer_limiters
                .values()
                .map(|endpoint_limiters| endpoint_limiters.len())
                .sum();

            if initial_count != final_count {
                info!(
                    "Cleaned up {} unused peer rate limiters",
                    initial_count - final_count
                );
            }
        }
    }
}
