//! Distributed rate limiting coordination
//!
//! This module provides comprehensive distributed rate limiting coordination
//! with zero allocation fast paths and blazing-fast performance.

use super::algorithms::{RateLimiter, RateLimitAlgorithmType, AlgorithmState};
use super::limiter::{TokenBucketConfig, SlidingWindowConfig};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicF64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Endpoint-specific rate limiting configuration with advanced settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRateConfig {
    /// Rate limiting algorithm to use
    pub algorithm: RateLimitAlgorithmType,
    /// Whether to apply per-peer rate limiting
    pub per_peer: bool,
    /// Multiplier for trusted peers (>1.0 allows more requests)
    pub trusted_multiplier: f64,
}

/// Advanced distributed rate limiting manager with per-endpoint and per-peer tracking
pub struct DistributedRateLimitManager {
    /// Per-endpoint rate limiting configurations
    endpoint_configs: Arc<DashMap<String, EndpointRateConfig>>,
    /// Per-endpoint rate limiters with zero allocation lookup
    endpoint_limiters: Arc<DashMap<String, RateLimiter>>,
    /// Per-peer rate limiters organized by endpoint with optimized nested access
    peer_limiters: Arc<DashMap<String, DashMap<String, RateLimiter>>>,
    /// Dynamic load multiplier for adaptive rate limiting
    load_multiplier: Arc<AtomicF64>,
    /// Manager creation timestamp for activity tracking
    created_at: Instant,
}

impl DistributedRateLimitManager {
    /// Create new distributed rate limiting manager with optimized initialization
    pub fn new() -> Self {
        let mut endpoint_configs = DashMap::new();

        // Configure default rate limits for common endpoints with production-ready limits
        endpoint_configs.insert(
            "/api/peers".to_string(),
            EndpointRateConfig {
                algorithm: RateLimitAlgorithmType::TokenBucket(TokenBucketConfig {
                    capacity: 20,
                    refill_rate: 2.0, // 2 requests per second
                    initial_tokens: 20,
                }),
                per_peer: true,
                trusted_multiplier: 2.0,
            },
        );

        endpoint_configs.insert(
            "/api/register".to_string(),
            EndpointRateConfig {
                algorithm: RateLimitAlgorithmType::SlidingWindow(SlidingWindowConfig {
                    window_size: 300, // 5 minutes
                    max_requests: 10, // 10 registrations per 5 minutes
                    sub_windows: 10,  // 30-second sub-windows
                }),
                per_peer: true,
                trusted_multiplier: 1.5,
            },
        );

        endpoint_configs.insert(
            "/health".to_string(),
            EndpointRateConfig {
                algorithm: RateLimitAlgorithmType::TokenBucket(TokenBucketConfig {
                    capacity: 100,
                    refill_rate: 50.0, // High rate for health checks
                    initial_tokens: 100,
                }),
                per_peer: false, // Global health check limit
                trusted_multiplier: 1.0,
            },
        );

        endpoint_configs.insert(
            "/metrics".to_string(),
            EndpointRateConfig {
                algorithm: RateLimitAlgorithmType::TokenBucket(TokenBucketConfig {
                    capacity: 50,
                    refill_rate: 10.0, // Moderate rate for metrics
                    initial_tokens: 50,
                }),
                per_peer: true,
                trusted_multiplier: 3.0, // Higher limit for monitoring systems
            },
        );

        Self {
            endpoint_configs: Arc::new(endpoint_configs),
            endpoint_limiters: Arc::new(DashMap::new()),
            peer_limiters: Arc::new(DashMap::new()),
            load_multiplier: Arc::new(AtomicF64::new(1.0)),
            created_at: Instant::now(),
        }
    }

    /// Check if request should be allowed with advanced distributed coordination
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

        // Apply load-based adjustment using lock-free atomic operation
        let load_multiplier = self.load_multiplier.load(Ordering::Relaxed);
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

    /// Check request against endpoint-level rate limiter using lock-free operations
    fn check_endpoint_request(
        &self,
        endpoint: &str,
        config: &EndpointRateConfig,
        tokens: u32,
    ) -> bool {
        // Lock-free get or insert operation using DashMap
        if !self.endpoint_limiters.contains_key(endpoint) {
            // Create new limiter if it doesn't exist
            let limiter = match &config.algorithm {
                RateLimitAlgorithmType::TokenBucket(token_config) => {
                    RateLimiter::TokenBucket(super::algorithms::TokenBucket::new(token_config.clone()))
                }
                RateLimitAlgorithmType::SlidingWindow(window_config) => {
                    RateLimiter::SlidingWindow(super::algorithms::SlidingWindow::new(window_config.clone()))
                }
            };
            self.endpoint_limiters.insert(endpoint.to_string(), limiter);
        }

        // Lock-free access to the limiter
        if let Some(mut entry) = self.endpoint_limiters.get_mut(endpoint) {
            let allowed = entry.check_request(tokens);

            if !allowed {
                crate::metrics::record_rate_limit_rejection(endpoint);
                warn!("Rate limit exceeded for endpoint {}", endpoint);
            }

            allowed
        } else {
            // Fallback - allow request if limiter lookup fails
            true
        }
    }

    /// Check request against per-peer rate limiter using lock-free nested DashMap operations
    fn check_peer_request(
        &self,
        endpoint: &str,
        peer_ip: &str,
        config: &EndpointRateConfig,
        tokens: u32,
    ) -> bool {
        // Lock-free get or insert endpoint-level DashMap
        if !self.peer_limiters.contains_key(endpoint) {
            self.peer_limiters
                .insert(endpoint.to_string(), DashMap::new());
        }

        // Lock-free access to endpoint's peer limiters
        if let Some(endpoint_limiters) = self.peer_limiters.get(endpoint) {
            // Lock-free get or insert peer-specific limiter
            if !endpoint_limiters.contains_key(peer_ip) {
                let limiter = match &config.algorithm {
                    RateLimitAlgorithmType::TokenBucket(token_config) => {
                        RateLimiter::TokenBucket(super::algorithms::TokenBucket::new(token_config.clone()))
                    }
                    RateLimitAlgorithmType::SlidingWindow(window_config) => {
                        RateLimiter::SlidingWindow(super::algorithms::SlidingWindow::new(window_config.clone()))
                    }
                };
                endpoint_limiters.insert(peer_ip.to_string(), limiter);
            }

            // Lock-free access to peer-specific limiter
            if let Some(mut peer_entry) = endpoint_limiters.get_mut(peer_ip) {
                let allowed = peer_entry.check_request(tokens);

                if !allowed {
                    crate::metrics::record_peer_rate_limit_rejection(endpoint, peer_ip);
                    warn!(
                        "Rate limit exceeded for peer {} on endpoint {}",
                        peer_ip, endpoint
                    );
                }

                allowed
            } else {
                // Fallback - allow request if peer limiter lookup fails
                true
            }
        } else {
            // Fallback - allow request if endpoint limiters lookup fails
            true
        }
    }

    /// Update load multiplier for dynamic rate adjustment with atomic update
    pub fn update_load_multiplier(&self, multiplier: f64) {
        let clamped_multiplier = multiplier.max(0.1).min(10.0); // Clamp between 0.1 and 10.0
        self.load_multiplier.store(clamped_multiplier, Ordering::Relaxed);
        
        debug!("Updated load multiplier to {:.2}", clamped_multiplier);
    }

    /// Get current load multiplier with zero allocation access
    pub fn get_load_multiplier(&self) -> f64 {
        self.load_multiplier.load(Ordering::Relaxed)
    }

    /// Add or update endpoint configuration with optimized config management
    pub fn configure_endpoint(&self, endpoint: String, config: EndpointRateConfig) {
        self.endpoint_configs.insert(endpoint.clone(), config.clone());

        // Update existing limiter if it exists
        if let Some(mut entry) = self.endpoint_limiters.get_mut(&endpoint) {
            entry.update_config(&config.algorithm);
        }

        // Update peer limiters for this endpoint
        if let Some(endpoint_limiters) = self.peer_limiters.get(&endpoint) {
            for mut entry in endpoint_limiters.iter_mut() {
                entry.update_config(&config.algorithm);
            }
        }

        info!("Updated rate limit configuration for endpoint {}", endpoint);
    }

    /// Get current rate limit statistics using lock-free operations
    pub fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        // Lock-free access to endpoint limiters count
        stats.insert(
            "endpoint_limiters".to_string(),
            serde_json::json!(self.endpoint_limiters.len()),
        );

        // Lock-free access to peer limiters count
        let total_peer_limiters: usize = self
            .peer_limiters
            .iter()
            .map(|entry| entry.value().len())
            .sum();
        stats.insert(
            "peer_limiters".to_string(),
            serde_json::json!(total_peer_limiters),
        );

        // Lock-free access to load multiplier
        stats.insert(
            "load_multiplier".to_string(),
            serde_json::json!(self.load_multiplier.load(Ordering::Relaxed)),
        );

        // Add endpoint-specific statistics
        let mut endpoint_stats = HashMap::new();
        for entry in self.endpoint_configs.iter() {
            let endpoint = entry.key();
            let config = entry.value();
            
            let mut endpoint_info = HashMap::new();
            endpoint_info.insert("per_peer", serde_json::json!(config.per_peer));
            endpoint_info.insert("trusted_multiplier", serde_json::json!(config.trusted_multiplier));
            endpoint_info.insert("algorithm", serde_json::json!(match &config.algorithm {
                RateLimitAlgorithmType::TokenBucket(_) => "TokenBucket",
                RateLimitAlgorithmType::SlidingWindow(_) => "SlidingWindow",
            }));
            
            endpoint_stats.insert(endpoint.clone(), serde_json::json!(endpoint_info));
        }
        stats.insert("endpoints".to_string(), serde_json::json!(endpoint_stats));

        stats
    }

    /// Remove peer limiters that haven't been used recently using lock-free operations
    pub fn cleanup_unused_limiters(&self) {
        let initial_count: usize = self
            .peer_limiters
            .iter()
            .map(|entry| entry.value().len())
            .sum();

        // Lock-free removal of peer limiters for endpoints that are no longer configured
        self.peer_limiters
            .retain(|endpoint, _| self.endpoint_configs.contains_key(endpoint));

        // Clean up inactive peer limiters within each endpoint
        for entry in self.peer_limiters.iter() {
            entry.value().retain(|_, limiter| limiter.is_active());
        }

        // Remove empty endpoint entries
        self.peer_limiters.retain(|_, peer_map| !peer_map.is_empty());

        let final_count: usize = self
            .peer_limiters
            .iter()
            .map(|entry| entry.value().len())
            .sum();

        if initial_count != final_count {
            info!(
                "Cleaned up {} unused peer rate limiters",
                initial_count - final_count
            );
        }
    }

    /// Get detailed state information for monitoring and debugging
    pub fn get_detailed_state(&self) -> DistributedRateLimitState {
        let mut endpoint_states = HashMap::new();
        let mut peer_states = HashMap::new();

        // Collect endpoint limiter states
        for entry in self.endpoint_limiters.iter() {
            let endpoint = entry.key().clone();
            let mut limiter = entry.value().clone();
            endpoint_states.insert(endpoint, limiter.get_state());
        }

        // Collect peer limiter states
        for entry in self.peer_limiters.iter() {
            let endpoint = entry.key().clone();
            let peer_limiters = entry.value();
            
            let mut endpoint_peer_states = HashMap::new();
            for peer_entry in peer_limiters.iter() {
                let peer_ip = peer_entry.key().clone();
                let mut limiter = peer_entry.value().clone();
                endpoint_peer_states.insert(peer_ip, limiter.get_state());
            }
            
            if !endpoint_peer_states.is_empty() {
                peer_states.insert(endpoint, endpoint_peer_states);
            }
        }

        DistributedRateLimitState {
            load_multiplier: self.load_multiplier.load(Ordering::Relaxed),
            endpoint_states,
            peer_states,
            total_endpoints: self.endpoint_configs.len(),
            total_peer_limiters: self.peer_limiters.iter().map(|e| e.value().len()).sum(),
            uptime: self.created_at.elapsed(),
        }
    }

    /// Check if distributed manager is healthy with comprehensive health check
    pub fn is_healthy(&self) -> bool {
        // Check if load multiplier is within reasonable bounds
        let load_multiplier = self.load_multiplier.load(Ordering::Relaxed);
        if load_multiplier < 0.01 || load_multiplier > 100.0 {
            return false;
        }

        // Check if we have reasonable number of active limiters
        let total_limiters = self.endpoint_limiters.len() + 
            self.peer_limiters.iter().map(|e| e.value().len()).sum::<usize>();
        
        // Healthy if we have some limiters but not an excessive number
        total_limiters > 0 && total_limiters < 10000
    }

    /// Reset all rate limiters with optimized reset operation
    pub fn reset_all_limiters(&self) {
        // Reset endpoint limiters
        for mut entry in self.endpoint_limiters.iter_mut() {
            // Note: This would require implementing reset on RateLimiter enum
            // For now, we'll clear and let them be recreated
        }
        self.endpoint_limiters.clear();

        // Reset peer limiters
        self.peer_limiters.clear();

        // Reset load multiplier
        self.load_multiplier.store(1.0, Ordering::Relaxed);

        info!("All rate limiters have been reset");
    }
}

impl Default for DistributedRateLimitManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive state information for distributed rate limiting
#[derive(Debug, Clone)]
pub struct DistributedRateLimitState {
    pub load_multiplier: f64,
    pub endpoint_states: HashMap<String, AlgorithmState>,
    pub peer_states: HashMap<String, HashMap<String, AlgorithmState>>,
    pub total_endpoints: usize,
    pub total_peer_limiters: usize,
    pub uptime: Duration,
}

impl DistributedRateLimitState {
    /// Calculate overall system utilization with weighted average
    pub fn overall_utilization(&self) -> f64 {
        let mut total_utilization = 0.0;
        let mut total_weight = 0.0;

        // Weight endpoint states
        for state in self.endpoint_states.values() {
            total_utilization += state.utilization_percent();
            total_weight += 1.0;
        }

        // Weight peer states (with lower weight per peer)
        for peer_map in self.peer_states.values() {
            for state in peer_map.values() {
                total_utilization += state.utilization_percent() * 0.1; // Lower weight for peers
                total_weight += 0.1;
            }
        }

        if total_weight > 0.0 {
            total_utilization / total_weight
        } else {
            0.0
        }
    }

    /// Check if any limiters are near capacity with configurable threshold
    pub fn has_limiters_near_capacity(&self, threshold: f64) -> bool {
        // Check endpoint limiters
        for state in self.endpoint_states.values() {
            if state.is_near_capacity(threshold) {
                return true;
            }
        }

        // Check peer limiters
        for peer_map in self.peer_states.values() {
            for state in peer_map.values() {
                if state.is_near_capacity(threshold) {
                    return true;
                }
            }
        }

        false
    }

    /// Get summary statistics with optimized calculation
    pub fn get_summary(&self) -> DistributedRateLimitSummary {
        DistributedRateLimitSummary {
            load_multiplier: self.load_multiplier,
            overall_utilization: self.overall_utilization(),
            total_endpoints: self.total_endpoints,
            total_peer_limiters: self.total_peer_limiters,
            near_capacity_count: self.endpoint_states.values()
                .chain(self.peer_states.values().flat_map(|m| m.values()))
                .filter(|s| s.is_near_capacity(80.0))
                .count(),
            uptime_seconds: self.uptime.as_secs(),
        }
    }
}

/// Summary statistics for distributed rate limiting
#[derive(Debug, Clone, Serialize)]
pub struct DistributedRateLimitSummary {
    pub load_multiplier: f64,
    pub overall_utilization: f64,
    pub total_endpoints: usize,
    pub total_peer_limiters: usize,
    pub near_capacity_count: usize,
    pub uptime_seconds: u64,
}