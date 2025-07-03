//! Tests for metrics module

#[cfg(test)]
mod tests {
    use crate::monitoring::metrics::*;
    
    #[test]
    fn test_counter_metric() {
        let counter = CounterMetric::new("test_counter", "Test counter");
        counter.record(1.0);
        counter.record(2.0);
        
        if let MetricValue::Counter(value) = counter.value() {
            assert_eq!(value, 3.0);
        } else {
            panic!("Expected counter value");
        }
    }
    
    #[test]
    fn test_gauge_metric() {
        let gauge = GaugeMetric::new("test_gauge", "Test gauge");
        gauge.record(10.0);
        gauge.record(20.0);
        
        if let MetricValue::Gauge(value) = gauge.value() {
            assert_eq!(value, 20.0);
        } else {
            panic!("Expected gauge value");
        }
    }
    
    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();
        
        collector.register(
            "requests".to_string(),
            Box::new(CounterMetric::new("requests", "Total requests")),
        );
        
        collector.record("requests", 1.0);
        collector.record("requests", 1.0);
        
        let metrics = collector.collect();
        assert!(metrics.contains_key("requests"));
    }
}