use sweetmcp::rate_limit::*;
use tokio::time::{sleep, Duration};

#[test]
fn test_token_bucket_basic() {
    let config = TokenBucketConfig {
        capacity: 10,
        refill_rate: 1.0,
        initial_tokens: 5,
    };
    let mut bucket = TokenBucket::new(&config);
    
    // Should allow consuming available tokens
    assert!(bucket.try_consume(3));
    assert!(bucket.try_consume(2));
    
    // Should reject when insufficient tokens
    assert!(!bucket.try_consume(1));
}

#[tokio::test]
async fn test_token_bucket_refill() {
    let config = TokenBucketConfig {
        capacity: 10,
        refill_rate: 10.0, // 10 tokens per second
        initial_tokens: 0,
    };
    let mut bucket = TokenBucket::new(&config);
    
    // Initially no tokens
    assert!(!bucket.try_consume(1));
    
    // Wait for refill
    sleep(Duration::from_millis(200)).await;
    
    // Should have approximately 2 tokens (10 * 0.2 seconds)
    assert!(bucket.try_consume(1));
}

#[test]
fn test_sliding_window_basic() {
    let config = SlidingWindowConfig {
        window_size: 60,
        max_requests: 5,
        sub_windows: 6,
    };
    let mut window = SlidingWindow::new(&config);
    
    // Should allow up to max_requests
    for _ in 0..5 {
        assert!(window.try_request());
    }
    
    // Should reject when limit exceeded
    assert!(!window.try_request());
}

#[test]
fn test_rate_limit_manager() {
    let manager = AdvancedRateLimitManager::new();
    
    // Should allow requests to configured endpoints
    assert!(manager.check_request("/api/peers", None, 1));
    assert!(manager.check_request("/api/register", Some("127.0.0.1"), 1));
    
    // Should allow requests to unconfigured endpoints
    assert!(manager.check_request("/unknown", None, 1));
}