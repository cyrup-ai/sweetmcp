//! Rate limiting module decomposition
//!
//! This module provides the decomposed rate limiting functionality split into
//! logical modules for better maintainability and adherence to the 300-line limit.

pub mod limiter;
pub mod algorithms;
pub mod distributed;

// Re-export key types and functions for backward compatibility
pub use limiter::{
    AdvancedRateLimitManager, TokenBucketConfig, SlidingWindowConfig, 
    RateLimitConfig, RateLimitAlgorithmType, RateLimitStats, RateLimitStatsSnapshot
};
pub use algorithms::{
    RateLimitAlgorithm, TokenBucket, SlidingWindow, RateLimiter, 
    AlgorithmState, HybridAlgorithm
};
pub use distributed::{
    DistributedRateLimitManager, EndpointRateConfig, DistributedRateLimitState,
    DistributedRateLimitSummary
};