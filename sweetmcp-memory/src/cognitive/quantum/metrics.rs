//! Quantum system metrics and performance tracking

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Comprehensive metrics collection for quantum operations
#[derive(Debug, Default)]
pub struct QuantumMetrics {
    pub total_routing_requests: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
    pub average_coherence_time: Duration,
    pub entanglement_creation_rate: f64,
    pub decoherence_events: u64,
    pub error_correction_activations: u64,
    pub fidelity_measurements: Vec<f64>,
    pub performance_indicators: PerformanceIndicators,
    pub history: MetricsHistory,
}

/// Performance indicators
#[derive(Debug, Default)]
pub struct PerformanceIndicators {
    pub throughput: f64,
    pub latency_percentiles: LatencyPercentiles,
    pub resource_utilization: ResourceUtilization,
    pub error_rates: ErrorRates,
}

/// Latency percentiles
#[derive(Debug, Default)]
pub struct LatencyPercentiles {
    pub p50: Duration,
    pub p90: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub p999: Duration,
}

/// Resource utilization metrics
#[derive(Debug, Default)]
pub struct ResourceUtilization {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub quantum_register_usage: f64,
    pub entanglement_capacity_usage: f64,
}

/// Error rate tracking
#[derive(Debug, Default)]
pub struct ErrorRates {
    pub gate_error_rate: f64,
    pub readout_error_rate: f64,
    pub coherence_error_rate: f64,
    pub entanglement_error_rate: f64,
}

/// Historical metrics tracking
#[derive(Debug)]
pub struct MetricsHistory {
    pub routing_history: VecDeque<RoutingMetric>,
    pub fidelity_history: VecDeque<FidelityMetric>,
    pub resource_history: VecDeque<ResourceMetric>,
    pub max_history_size: usize,
}

/// Individual routing metric
#[derive(Debug, Clone)]
pub struct RoutingMetric {
    pub timestamp: Instant,
    pub duration: Duration,
    pub success: bool,
    pub strategy: String,
    pub confidence: f64,
}

/// Fidelity metric over time
#[derive(Debug, Clone)]
pub struct FidelityMetric {
    pub timestamp: Instant,
    pub fidelity: f64,
    pub measurement_type: String,
    pub error_corrected: bool,
}

/// Resource usage metric
#[derive(Debug, Clone)]
pub struct ResourceMetric {
    pub timestamp: Instant,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub quantum_states: usize,
    pub entanglement_links: usize,
}

impl QuantumMetrics {
    /// Create new metrics tracker
    pub fn new() -> Self {
        Self {
            history: MetricsHistory::new(10000),
            ..Default::default()
        }
    }

    /// Record a routing event
    pub fn record_routing(
        &mut self,
        duration: Duration,
        success: bool,
        strategy: &str,
        confidence: f64,
    ) {
        self.total_routing_requests += 1;

        if success {
            self.successful_routes += 1;
        } else {
            self.failed_routes += 1;
        }

        // Update throughput
        if self.total_routing_requests > 0 {
            self.performance_indicators.throughput =
                self.successful_routes as f64 / self.total_routing_requests as f64;
        }

        // Add to history
        self.history.add_routing_metric(RoutingMetric {
            timestamp: Instant::now(),
            duration,
            success,
            strategy: strategy.to_string(),
            confidence,
        });

        // Update latency percentiles
        self.update_latency_percentiles();
    }

    /// Record a fidelity measurement
    pub fn record_fidelity(
        &mut self,
        fidelity: f64,
        measurement_type: &str,
        error_corrected: bool,
    ) {
        self.fidelity_measurements.push(fidelity);

        // Keep only recent measurements
        if self.fidelity_measurements.len() > 1000 {
            self.fidelity_measurements.remove(0);
        }

        self.history.add_fidelity_metric(FidelityMetric {
            timestamp: Instant::now(),
            fidelity,
            measurement_type: measurement_type.to_string(),
            error_corrected,
        });
    }

    /// Record resource usage
    pub fn record_resource_usage(
        &mut self,
        cpu: f64,
        memory: f64,
        quantum_states: usize,
        entanglement_links: usize,
    ) {
        self.performance_indicators.resource_utilization.cpu_usage = cpu;
        self.performance_indicators
            .resource_utilization
            .memory_usage = memory;

        self.history.add_resource_metric(ResourceMetric {
            timestamp: Instant::now(),
            cpu_usage: cpu,
            memory_usage: memory,
            quantum_states,
            entanglement_links,
        });
    }

    /// Record a decoherence event
    pub fn record_decoherence(&mut self) {
        self.decoherence_events += 1;
    }

    /// Record error correction activation
    pub fn record_error_correction(&mut self) {
        self.error_correction_activations += 1;
    }

    /// Update error rates
    pub fn update_error_rates(
        &mut self,
        gate_error: f64,
        readout_error: f64,
        coherence_error: f64,
        entanglement_error: f64,
    ) {
        let rates = &mut self.performance_indicators.error_rates;
        rates.gate_error_rate = gate_error;
        rates.readout_error_rate = readout_error;
        rates.coherence_error_rate = coherence_error;
        rates.entanglement_error_rate = entanglement_error;
    }

    /// Update latency percentiles from history
    fn update_latency_percentiles(&mut self) {
        let mut latencies: Vec<Duration> = self
            .history
            .routing_history
            .iter()
            .map(|m| m.duration)
            .collect();

        if latencies.is_empty() {
            return;
        }

        latencies.sort();

        let len = latencies.len();
        self.performance_indicators.latency_percentiles = LatencyPercentiles {
            p50: latencies[len * 50 / 100],
            p90: latencies[len * 90 / 100],
            p95: latencies[len * 95 / 100],
            p99: latencies[len * 99 / 100],
            p999: latencies[(len * 999 / 1000).min(len - 1)],
        };
    }

    /// Get average fidelity
    pub fn average_fidelity(&self) -> f64 {
        if self.fidelity_measurements.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.fidelity_measurements.iter().sum();
        sum / self.fidelity_measurements.len() as f64
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_routing_requests == 0 {
            return 0.0;
        }

        self.successful_routes as f64 / self.total_routing_requests as f64
    }

    /// Export metrics as JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct ExportableMetrics {
            total_requests: u64,
            success_rate: f64,
            average_fidelity: f64,
            decoherence_events: u64,
            error_corrections: u64,
            latency_p50_ms: f64,
            latency_p99_ms: f64,
            cpu_usage: f64,
            memory_usage: f64,
        }

        let export = ExportableMetrics {
            total_requests: self.total_routing_requests,
            success_rate: self.success_rate(),
            average_fidelity: self.average_fidelity(),
            decoherence_events: self.decoherence_events,
            error_corrections: self.error_correction_activations,
            latency_p50_ms: self
                .performance_indicators
                .latency_percentiles
                .p50
                .as_secs_f64()
                * 1000.0,
            latency_p99_ms: self
                .performance_indicators
                .latency_percentiles
                .p99
                .as_secs_f64()
                * 1000.0,
            cpu_usage: self.performance_indicators.resource_utilization.cpu_usage,
            memory_usage: self
                .performance_indicators
                .resource_utilization
                .memory_usage,
        };

        serde_json::to_string_pretty(&export)
    }
}

impl Default for MetricsHistory {
    fn default() -> Self {
        Self::new(10000)
    }
}

impl MetricsHistory {
    /// Create new metrics history
    pub fn new(max_size: usize) -> Self {
        Self {
            routing_history: VecDeque::with_capacity(max_size),
            fidelity_history: VecDeque::with_capacity(max_size),
            resource_history: VecDeque::with_capacity(max_size),
            max_history_size: max_size,
        }
    }

    /// Add routing metric
    pub fn add_routing_metric(&mut self, metric: RoutingMetric) {
        if self.routing_history.len() >= self.max_history_size {
            self.routing_history.pop_front();
        }
        self.routing_history.push_back(metric);
    }

    /// Add fidelity metric
    pub fn add_fidelity_metric(&mut self, metric: FidelityMetric) {
        if self.fidelity_history.len() >= self.max_history_size {
            self.fidelity_history.pop_front();
        }
        self.fidelity_history.push_back(metric);
    }

    /// Add resource metric
    pub fn add_resource_metric(&mut self, metric: ResourceMetric) {
        if self.resource_history.len() >= self.max_history_size {
            self.resource_history.pop_front();
        }
        self.resource_history.push_back(metric);
    }

    /// Get metrics within time window
    pub fn get_metrics_in_window(&self, window: Duration) -> MetricsWindow {
        let cutoff = Instant::now() - window;

        let routing: Vec<_> = self
            .routing_history
            .iter()
            .filter(|m| m.timestamp > cutoff)
            .cloned()
            .collect();

        let fidelity: Vec<_> = self
            .fidelity_history
            .iter()
            .filter(|m| m.timestamp > cutoff)
            .cloned()
            .collect();

        let resource: Vec<_> = self
            .resource_history
            .iter()
            .filter(|m| m.timestamp > cutoff)
            .cloned()
            .collect();

        MetricsWindow {
            routing,
            fidelity,
            resource,
        }
    }
}

/// Metrics within a time window
#[derive(Debug)]
pub struct MetricsWindow {
    pub routing: Vec<RoutingMetric>,
    pub fidelity: Vec<FidelityMetric>,
    pub resource: Vec<ResourceMetric>,
}

impl MetricsWindow {
    /// Calculate average metrics in window
    pub fn averages(&self) -> WindowAverages {
        let avg_latency = if !self.routing.is_empty() {
            let sum: Duration = self.routing.iter().map(|m| m.duration).sum();
            sum / self.routing.len() as u32
        } else {
            Duration::ZERO
        };

        let avg_fidelity = if !self.fidelity.is_empty() {
            let sum: f64 = self.fidelity.iter().map(|m| m.fidelity).sum();
            sum / self.fidelity.len() as f64
        } else {
            0.0
        };

        let avg_cpu = if !self.resource.is_empty() {
            let sum: f64 = self.resource.iter().map(|m| m.cpu_usage).sum();
            sum / self.resource.len() as f64
        } else {
            0.0
        };

        let avg_memory = if !self.resource.is_empty() {
            let sum: f64 = self.resource.iter().map(|m| m.memory_usage).sum();
            sum / self.resource.len() as f64
        } else {
            0.0
        };

        WindowAverages {
            latency: avg_latency,
            fidelity: avg_fidelity,
            cpu_usage: avg_cpu,
            memory_usage: avg_memory,
        }
    }
}

/// Average metrics in a window
#[derive(Debug)]
pub struct WindowAverages {
    pub latency: Duration,
    pub fidelity: f64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let mut metrics = QuantumMetrics::new();

        // Record some routing events
        metrics.record_routing(Duration::from_millis(10), true, "quantum", 0.95);
        metrics.record_routing(Duration::from_millis(15), true, "quantum", 0.90);
        metrics.record_routing(Duration::from_millis(20), false, "quantum", 0.70);

        assert_eq!(metrics.total_routing_requests, 3);
        assert_eq!(metrics.successful_routes, 2);
        assert_eq!(metrics.failed_routes, 1);
        assert!((metrics.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_fidelity_tracking() {
        let mut metrics = QuantumMetrics::new();

        metrics.record_fidelity(0.95, "bell_state", false);
        metrics.record_fidelity(0.98, "bell_state", true);
        metrics.record_fidelity(0.92, "ghz_state", false);

        assert_eq!(metrics.fidelity_measurements.len(), 3);
        assert!((metrics.average_fidelity() - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_history_window() {
        let mut history = MetricsHistory::new(100);

        // Add metrics
        for i in 0..5 {
            history.add_routing_metric(RoutingMetric {
                timestamp: Instant::now(),
                duration: Duration::from_millis(10 + i),
                success: true,
                strategy: "quantum".to_string(),
                confidence: 0.9,
            });

            std::thread::sleep(Duration::from_millis(10));
        }

        // Get recent window
        let window = history.get_metrics_in_window(Duration::from_millis(30));
        assert!(window.routing.len() <= 3); // Should only include recent metrics

        let averages = window.averages();
        assert!(averages.latency.as_millis() >= 10);
    }

    #[test]
    fn test_export_json() {
        let mut metrics = QuantumMetrics::new();

        metrics.record_routing(Duration::from_millis(10), true, "quantum", 0.95);
        metrics.record_fidelity(0.95, "test", false);
        metrics.record_resource_usage(0.5, 0.6, 100, 50);

        let json = metrics.export_json().unwrap();
        assert!(json.contains("\"total_requests\": 1"));
        assert!(json.contains("\"success_rate\": 1.0"));
        assert!(json.contains("\"average_fidelity\": 0.95"));
    }
}
