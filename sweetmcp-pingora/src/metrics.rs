//! Metrics for SweetMCP discovery and operations

use once_cell::sync::Lazy;
use prometheus::{
    register_counter_vec, register_histogram_vec, register_int_gauge, register_int_gauge_vec,
    CounterVec, HistogramVec, IntGauge, IntGaugeVec,
};

/// Discovery operation counter
pub static DISCOVERY_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "sweetmcp_discovery_operations_total",
        "Total number of discovery operations",
        &["operation", "status"]
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register discovery counter: {}", e);
        std::process::exit(1)
    })
});

/// Discovery operation latency
pub static DISCOVERY_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "sweetmcp_discovery_duration_seconds",
        "Discovery operation duration in seconds",
        &["operation"],
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register discovery latency: {}", e);
        std::process::exit(1)
    })
});

/// Active peer count
pub static PEER_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("sweetmcp_active_peers", "Number of active peers").unwrap_or_else(|e| {
        tracing::error!("Failed to register peer count gauge: {}", e);
        std::process::exit(1)
    })
});

/// Rate limit rejections
pub static RATE_LIMIT_REJECTIONS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "sweetmcp_rate_limit_rejections_total",
        "Total number of rate limit rejections",
        &["endpoint"]
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register rate limit counter: {}", e);
        std::process::exit(1)
    })
});

/// Record a discovery operation
pub fn record_discovery(operation: &str, success: bool, duration_secs: f64) {
    let status = if success { "success" } else { "failure" };
    DISCOVERY_COUNTER
        .with_label_values(&[operation, status])
        .inc();
    DISCOVERY_LATENCY
        .with_label_values(&[operation])
        .observe(duration_secs);
}

/// Update peer count
pub fn update_peer_count(count: i64) {
    PEER_COUNT.set(count);
}

/// Record rate limit rejection
pub fn record_rate_limit_rejection(endpoint: &str) {
    RATE_LIMIT_REJECTIONS.with_label_values(&[endpoint]).inc();
}

/// Circuit breaker state counter
pub static CIRCUIT_BREAKER_STATE: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "sweetmcp_circuit_breaker_state_changes_total",
        "Total number of circuit breaker state changes",
        &["peer", "state"]
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register circuit breaker state counter: {}", e);
        std::process::exit(1)
    })
});

/// Record circuit breaker state change
pub fn record_circuit_breaker_state(peer: &str, state: &str) {
    CIRCUIT_BREAKER_STATE
        .with_label_values(&[peer, state])
        .inc();
}

// ============================================================================
// HTTP Request/Response Metrics for Enterprise Observability
// ============================================================================

/// HTTP request duration histogram per endpoint
pub static HTTP_REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "sweetmcp_http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint", "status_code"],
        vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register HTTP request duration histogram: {}", e);
        std::process::exit(1)
    })
});

/// HTTP response status code counter
pub static HTTP_RESPONSE_STATUS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "sweetmcp_http_responses_total",
        "Total number of HTTP responses by status code",
        &["method", "endpoint", "status_code"]
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register HTTP response status counter: {}", e);
        std::process::exit(1)
    })
});

/// HTTP request size histogram
pub static HTTP_REQUEST_SIZE: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "sweetmcp_http_request_size_bytes",
        "HTTP request size in bytes",
        &["method", "endpoint"],
        vec![64.0, 256.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0]
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register HTTP request size histogram: {}", e);
        std::process::exit(1)
    })
});

/// HTTP response size histogram
pub static HTTP_RESPONSE_SIZE: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "sweetmcp_http_response_size_bytes",
        "HTTP response size in bytes",
        &["method", "endpoint", "status_code"],
        vec![64.0, 256.0, 1024.0, 4096.0, 16384.0, 65536.0, 262144.0, 1048576.0]
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register HTTP response size histogram: {}", e);
        std::process::exit(1)
    })
});

/// Active HTTP connections gauge
#[allow(dead_code)]
pub static HTTP_CONNECTIONS_ACTIVE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "sweetmcp_http_connections_active",
        "Number of active HTTP connections"
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register active HTTP connections gauge: {}", e);
        std::process::exit(1)
    })
});

/// Active HTTP requests per endpoint gauge
pub static HTTP_REQUESTS_ACTIVE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "sweetmcp_http_requests_active",
        "Number of active HTTP requests per endpoint",
        &["method", "endpoint"]
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register active HTTP requests gauge: {}", e);
        std::process::exit(1)
    })
});

/// Concurrent HTTP requests total gauge
pub static HTTP_REQUESTS_CONCURRENT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "sweetmcp_http_requests_concurrent_total",
        "Total number of concurrent HTTP requests across all endpoints"
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to register concurrent HTTP requests gauge: {}", e);
        std::process::exit(1)
    })
});

// ============================================================================
// HTTP Metrics Recording Functions
// ============================================================================

/// Record HTTP request completion with comprehensive metrics
pub fn record_http_request(
    method: &str,
    endpoint: &str,
    status_code: u16,
    duration_secs: f64,
    request_size_bytes: usize,
    response_size_bytes: usize,
) {
    let status_str = &status_code.to_string();

    // Record request duration
    HTTP_REQUEST_DURATION
        .with_label_values(&[method, endpoint, status_str])
        .observe(duration_secs);

    // Record response status
    HTTP_RESPONSE_STATUS
        .with_label_values(&[method, endpoint, status_str])
        .inc();

    // Record request size
    HTTP_REQUEST_SIZE
        .with_label_values(&[method, endpoint])
        .observe(request_size_bytes as f64);

    // Record response size
    HTTP_RESPONSE_SIZE
        .with_label_values(&[method, endpoint, status_str])
        .observe(response_size_bytes as f64);
}

/// Track active HTTP connection count
#[allow(dead_code)]
pub fn update_http_connections_active(count: i64) {
    HTTP_CONNECTIONS_ACTIVE.set(count);
}

/// Increment active requests for endpoint (call at request start)
pub fn increment_active_requests(method: &str, endpoint: &str) {
    HTTP_REQUESTS_ACTIVE
        .with_label_values(&[method, endpoint])
        .inc();
    HTTP_REQUESTS_CONCURRENT.inc();
}

/// Decrement active requests for endpoint (call at request end)
pub fn decrement_active_requests(method: &str, endpoint: &str) {
    HTTP_REQUESTS_ACTIVE
        .with_label_values(&[method, endpoint])
        .dec();
    HTTP_REQUESTS_CONCURRENT.dec();
}
