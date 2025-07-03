//! Metrics collection and export

use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Opts};
use std::collections::HashMap;

/// Metric types
#[derive(Debug, Clone)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Metric value
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(f64),
    Gauge(f64),
    Histogram(f64),
}

/// Metrics collector
pub struct MetricsCollector {
    /// Registered metrics
    metrics: HashMap<String, Box<dyn Metric>>,
}

impl MetricsCollector {
    /// Create a new collector
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }
    
    /// Register a metric
    pub fn register(&mut self, name: String, metric: Box<dyn Metric>) {
        self.metrics.insert(name, metric);
    }
    
    /// Record a value
    pub fn record(&self, name: &str, value: f64) {
        if let Some(metric) = self.metrics.get(name) {
            metric.record(value);
        }
    }
    
    /// Get all metrics
    pub fn collect(&self) -> HashMap<String, MetricValue> {
        self.metrics
            .iter()
            .map(|(name, metric)| (name.clone(), metric.value()))
            .collect()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric trait
pub trait Metric: Send + Sync {
    /// Record a value
    fn record(&self, value: f64);
    
    /// Get current value
    fn value(&self) -> MetricValue;
    
    /// Get metric type
    fn metric_type(&self) -> MetricType;
}

/// Counter metric wrapper
pub struct CounterMetric {
    counter: Counter,
}

impl CounterMetric {
    pub fn new(name: &str, help: &str) -> Self {
        let opts = Opts::new(name, help);
        let counter = Counter::with_opts(opts).unwrap();
        Self { counter }
    }
}

impl Metric for CounterMetric {
    fn record(&self, value: f64) {
        self.counter.inc_by(value);
    }
    
    fn value(&self) -> MetricValue {
        MetricValue::Counter(self.counter.get())
    }
    
    fn metric_type(&self) -> MetricType {
        MetricType::Counter
    }
}

/// Gauge metric wrapper
pub struct GaugeMetric {
    gauge: Gauge,
}

impl GaugeMetric {
    pub fn new(name: &str, help: &str) -> Self {
        let opts = Opts::new(name, help);
        let gauge = Gauge::with_opts(opts).unwrap();
        Self { gauge }
    }
}

impl Metric for GaugeMetric {
    fn record(&self, value: f64) {
        self.gauge.set(value);
    }
    
    fn value(&self) -> MetricValue {
        MetricValue::Gauge(self.gauge.get())
    }
    
    fn metric_type(&self) -> MetricType {
        MetricType::Gauge
    }
}

/// Histogram metric wrapper
pub struct HistogramMetric {
    histogram: Histogram,
}

impl HistogramMetric {
    pub fn new(name: &str, help: &str) -> Self {
        let opts = HistogramOpts::new(name, help);
        let histogram = Histogram::with_opts(opts).unwrap();
        Self { histogram }
    }
}

impl Metric for HistogramMetric {
    fn record(&self, value: f64) {
        self.histogram.observe(value);
    }
    
    fn value(&self) -> MetricValue {
        // Return the sum for simplicity
        MetricValue::Histogram(self.histogram.get_sample_sum())
    }
    
    fn metric_type(&self) -> MetricType {
        MetricType::Histogram
    }
}