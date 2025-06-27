use std::sync::Arc;
use std::time::Duration;
use sweetmcp::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerManager, CircuitState};

#[tokio::test]
async fn test_circuit_breaker_opens_on_threshold() {
    let config = CircuitBreakerConfig {
        error_threshold_percentage: 50,
        request_volume_threshold: 4,
        ..Default::default()
    };
    
    let breaker = CircuitBreaker::new("test-peer".to_string(), config);
    
    // Record some successes and failures
    breaker.record_success().await;
    breaker.record_failure().await;
    breaker.record_failure().await;
    breaker.record_failure().await;
    
    // Should be open now (3 failures out of 4 = 75% > 50%)
    assert_eq!(breaker.get_state().await, CircuitState::Open);
    assert!(!breaker.should_allow_request().await);
}

#[tokio::test]
async fn test_circuit_breaker_half_open() {
    let config = CircuitBreakerConfig {
        error_threshold_percentage: 50,
        request_volume_threshold: 2,
        sleep_window: Duration::from_millis(100),
        half_open_requests: 3,
        ..Default::default()
    };
    
    let breaker = CircuitBreaker::new("test-peer".to_string(), config);
    
    // Open the circuit
    breaker.record_failure().await;
    breaker.record_failure().await;
    assert_eq!(breaker.get_state().await, CircuitState::Open);
    
    // Wait for sleep window
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Should transition to half-open
    assert!(breaker.should_allow_request().await);
    assert_eq!(breaker.get_state().await, CircuitState::HalfOpen);
    
    // Should allow limited requests
    assert!(breaker.should_allow_request().await);
    assert!(breaker.should_allow_request().await);
    assert!(!breaker.should_allow_request().await); // Exhausted
}

#[tokio::test]
async fn test_circuit_breaker_closes_after_recovery() {
    let config = CircuitBreakerConfig {
        error_threshold_percentage: 50,
        request_volume_threshold: 2,
        sleep_window: Duration::from_millis(100),
        half_open_requests: 2,
        ..Default::default()
    };
    
    let breaker = CircuitBreaker::new("test-peer".to_string(), config);
    
    // Open the circuit
    breaker.record_failure().await;
    breaker.record_failure().await;
    
    // Wait and transition to half-open
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(breaker.should_allow_request().await);
    
    // Successful requests in half-open should close circuit
    breaker.record_success().await;
    breaker.record_success().await;
    
    assert_eq!(breaker.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_manager() {
    let manager = CircuitBreakerManager::new(Default::default());
    
    // Get breaker for peer1
    let breaker1 = manager.get_breaker("peer1").await;
    let breaker1_again = manager.get_breaker("peer1").await;
    
    // Should be the same instance
    assert!(Arc::ptr_eq(&breaker1, &breaker1_again));
    
    // Different peer should get different breaker
    let breaker2 = manager.get_breaker("peer2").await;
    assert!(!Arc::ptr_eq(&breaker1, &breaker2));
    
    // Test removal
    manager.remove_breaker("peer1").await;
    let breaker1_new = manager.get_breaker("peer1").await;
    assert!(!Arc::ptr_eq(&breaker1, &breaker1_new));
}

#[tokio::test]
async fn test_window_reset() {
    let config = CircuitBreakerConfig {
        error_threshold_percentage: 50,
        request_volume_threshold: 2,
        metrics_window: Duration::from_millis(100),
        ..Default::default()
    };
    
    let breaker = CircuitBreaker::new("test-peer".to_string(), config);
    
    // Record failures but not enough to trip
    breaker.record_failure().await;
    let (total, failed) = breaker.get_metrics();
    assert_eq!(total, 1);
    assert_eq!(failed, 1);
    
    // Wait for window to reset
    tokio::time::sleep(Duration::from_millis(150)).await;
    breaker.should_allow_request().await; // Triggers window check
    
    // Metrics should be reset
    let (total, failed) = breaker.get_metrics();
    assert_eq!(total, 0);
    assert_eq!(failed, 0);
}