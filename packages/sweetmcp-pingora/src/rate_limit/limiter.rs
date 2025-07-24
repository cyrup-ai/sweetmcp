//! Core rate limiter implementation
//!
//! This module provides the core rate limiting functionality with advanced algorithms
//! and zero allocation fast paths for blazing-fast performance.

use super::algorithms::{TokenBucket, SlidingWindow, RateLimitAlgorithm};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
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

/// Advanced rate limiting manager with multiple algorithms and per-endpoint/peer tracking
pub struct AdvancedRateLimitManager {
    /// Global rate limiting configuration
    global_config: RateLimitConfig,
    /// Per-endpoint rate limiters with zero allocation lookup
    endpoint_limiters: Arc<DashMap<String, Box<dyn RateLimitAlgorithm + Send + Sync>>>,
    /// Per-peer rate limiters with optimized peer tracking
    peer_limiters: Arc<DashMap<String, Box<dyn RateLimitAlgorithm + Send + Sync>>>,
    /// Global statistics with atomic counters
    stats: Arc<RateLimitStats>,
    /// Cleanup task handle for background maintenance
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
    /// Manager operational state
    operational: Arc<std::sync::atomic::AtomicBool>,
}

impl AdvancedRateLimitManager {
    /// Create new advanced rate limiting manager with optimized initialization
    pub fn new(
        requests_per_second: f64,
        burst_size: u32,
        window_size_seconds: u64,
    ) -> Self {
        let global_config = RateLimitConfig {
            token_bucket: TokenBucketConfig {
                capacity: burst_size,
                refill_rate: requests_per_second,
                initial_tokens: burst_size,
            },
            sliding_window: SlidingWindowConfig {
                window_size: window_size_seconds,
                max_requests: (requests_per_second * window_size_seconds as f64) as u32,
                sub_windows: std::cmp::min(window_size_seconds, 60) as u32, // Max 60 sub-windows
            },
            algorithm: RateLimitAlgorithmType::TokenBucket,
            enabled: true,
        };

        Self {
            global_config,
            endpoint_limiters: Arc::new(DashMap::new()),
            peer_limiters: Arc::new(DashMap::new()),
            stats: Arc::new(RateLimitStats::new()),
            cleanup_handle: None,
            operational: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    /// Check if request is allowed with fast path validation
    pub fn check_request(
        &self,
        endpoint: &str,
        peer_id: Option<&str>,
        request_count: u32,
    ) -> bool {
        // Fast path: Check if rate limiting is disabled
        if !self.global_config.enabled {
            return true;
        }

        // Fast path: Check if manager is operational
        if !self.operational.load(Ordering::Relaxed) {
            return true; // Allow requests if manager is not operational
        }

        let mut allowed = true;

        // Check endpoint-specific rate limit with optimized endpoint checking
        if let Some(mut limiter) = self.endpoint_limiters.get_mut(endpoint) {
            for _ in 0..request_count {
                if !limiter.try_request() {
                    allowed = false;
                    break;
                }
            }
        } else {
            // Create new endpoint limiter with fast limiter creation
            let limiter = self.create_limiter_for_endpoint(endpoint);
            let mut requests_allowed = 0;
            for _ in 0..request_count {
                if limiter.try_request() {
                    requests_allowed += 1;
                } else {
                    break;
                }
            }
            allowed = requests_allowed == request_count;
            self.endpoint_limiters.insert(endpoint.to_string(), limiter);
        }

        // Check peer-specific rate limit if peer_id is provided
        if allowed && let Some(peer) = peer_id {
            if let Some(mut limiter) = self.peer_limiters.get_mut(peer) {
                for _ in 0..request_count {
                    if !limiter.try_request() {
                        allowed = false;
                        break;
                    }
                }
            } else {
                // Create new peer limiter with optimized peer limiter creation
                let limiter = self.create_limiter_for_peer(peer);
                let mut requests_allowed = 0;
                for _ in 0..request_count {
                    if limiter.try_request() {
                        requests_allowed += 1;
                    } else {
                        break;
                    }
                }
                allowed = requests_allowed == request_count;
                self.peer_limiters.insert(peer.to_string(), limiter);
            }
        }

        // Update statistics with atomic operations
        if allowed {
            self.stats.requests_allowed.fetch_add(request_count as u64, Ordering::Relaxed);
        } else {
            self.stats.requests_denied.fetch_add(request_count as u64, Ordering::Relaxed);
        }

        allowed
    }

    /// Create rate limiter for specific endpoint with optimized creation
    fn create_limiter_for_endpoint(&self, endpoint: &str) -> Box<dyn RateLimitAlgorithm + Send + Sync> {
        // Use endpoint-specific configuration if available, otherwise use global config
        match self.global_config.algorithm {
            RateLimitAlgorithmType::TokenBucket => {
                Box::new(TokenBucket::new(self.global_config.token_bucket.clone()))
            }
            RateLimitAlgorithmType::SlidingWindow => {
                Box::new(SlidingWindow::new(self.global_config.sliding_window.clone()))
            }
            RateLimitAlgorithmType::Hybrid => {
                // For hybrid, use token bucket for burst control
                Box::new(TokenBucket::new(self.global_config.token_bucket.clone()))
            }
        }
    }

    /// Create rate limiter for specific peer with optimized peer limiter creation
    fn create_limiter_for_peer(&self, peer: &str) -> Box<dyn RateLimitAlgorithm + Send + Sync> {
        // Use peer-specific configuration if available, otherwise use global config
        match self.global_config.algorithm {
            RateLimitAlgorithmType::TokenBucket => {
                Box::new(TokenBucket::new(self.global_config.token_bucket.clone()))
            }
            RateLimitAlgorithmType::SlidingWindow => {
                Box::new(SlidingWindow::new(self.global_config.sliding_window.clone()))
            }
            RateLimitAlgorithmType::Hybrid => {
                // For hybrid, use sliding window for precise peer control
                Box::new(SlidingWindow::new(self.global_config.sliding_window.clone()))
            }
        }
    }

    /// Start background cleanup task with optimized cleanup scheduling
    pub fn start_cleanup_task(&mut self) {
        if self.cleanup_handle.is_some() {
            return; // Already running
        }

        let endpoint_limiters = Arc::clone(&self.endpoint_limiters);
        let peer_limiters = Arc::clone(&self.peer_limiters);
        let operational = Arc::clone(&self.operational);

        let handle = tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(Duration::from_secs(60)); // Cleanup every minute
            cleanup_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            while operational.load(Ordering::Relaxed) {
                cleanup_interval.tick().await;

                // Clean up inactive endpoint limiters with optimized cleanup
                endpoint_limiters.retain(|_, limiter| limiter.is_active());
                
                // Clean up inactive peer limiters with fast peer cleanup
                peer_limiters.retain(|_, limiter| limiter.is_active());

                debug!(
                    "Rate limiter cleanup: {} endpoint limiters, {} peer limiters",
                    endpoint_limiters.len(),
                    peer_limiters.len()
                );
            }
        });

        self.cleanup_handle = Some(handle);
        info!("Rate limiter cleanup task started");
    }

    /// Stop background cleanup task with graceful shutdown
    pub async fn stop_cleanup_task(&mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            self.operational.store(false, Ordering::Relaxed);
            handle.abort();
            info!("Rate limiter cleanup task stopped");
        }
    }

    /// Check if rate limiting is currently active with fast operational check
    pub fn is_rate_limiting_active(&self) -> bool {
        self.global_config.enabled && self.operational.load(Ordering::Relaxed)
    }

    /// Check if manager is operational with zero allocation check
    pub fn is_operational(&self) -> bool {
        self.operational.load(Ordering::Relaxed)
    }

    /// Get current statistics with atomic access
    pub fn get_stats(&self) -> RateLimitStatsSnapshot {
        RateLimitStatsSnapshot {
            requests_allowed: self.stats.requests_allowed.load(Ordering::Relaxed),
            requests_denied: self.stats.requests_denied.load(Ordering::Relaxed),
            active_endpoint_limiters: self.endpoint_limiters.len(),
            active_peer_limiters: self.peer_limiters.len(),
        }
    }

    /// Reset statistics with atomic reset
    pub fn reset_stats(&self) {
        self.stats.requests_allowed.store(0, Ordering::Relaxed);
        self.stats.requests_denied.store(0, Ordering::Relaxed);
    }

    /// Update global configuration with optimized config update
    pub fn update_config(&mut self, config: RateLimitConfig) {
        self.global_config = config;
        
        // Clear existing limiters to force recreation with new config
        self.endpoint_limiters.clear();
        self.peer_limiters.clear();
        
        info!("Rate limiter configuration updated");
    }

    /// Get current configuration with zero allocation access
    pub fn get_config(&self) -> &RateLimitConfig {
        &self.global_config
    }
}

/// Rate limiting configuration with comprehensive settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub token_bucket: TokenBucketConfig,
    pub sliding_window: SlidingWindowConfig,
    pub algorithm: RateLimitAlgorithmType,
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            token_bucket: TokenBucketConfig::default(),
            sliding_window: SlidingWindowConfig::default(),
            algorithm: RateLimitAlgorithmType::TokenBucket,
            enabled: true,
        }
    }
}

/// Rate limiting algorithm type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitAlgorithmType {
    TokenBucket,
    SlidingWindow,
    Hybrid,
}

/// Rate limiting statistics with atomic counters
pub struct RateLimitStats {
    pub requests_allowed: AtomicU64,
    pub requests_denied: AtomicU64,
}

impl RateLimitStats {
    pub fn new() -> Self {
        Self {
            requests_allowed: AtomicU64::new(0),
            requests_denied: AtomicU64::new(0),
        }
    }
}

/// Snapshot of rate limiting statistics
#[derive(Debug, Clone)]
pub struct RateLimitStatsSnapshot {
    pub requests_allowed: u64,
    pub requests_denied: u64,
    pub active_endpoint_limiters: usize,
    pub active_peer_limiters: usize,
}

impl RateLimitStatsSnapshot {
    /// Calculate request success rate with optimized calculation
    pub fn success_rate(&self) -> f64 {
        let total = self.requests_allowed + self.requests_denied;
        if total == 0 {
            1.0
        } else {
            self.requests_allowed as f64 / total as f64
        }
    }

    /// Calculate request denial rate with fast calculation
    pub fn denial_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }
}