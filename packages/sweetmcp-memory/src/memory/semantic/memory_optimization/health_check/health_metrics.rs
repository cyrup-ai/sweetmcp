//! Health check metrics and resource utilization
//!
//! This module provides blazing-fast performance and resource monitoring with zero allocation
//! optimizations and elegant ergonomic interfaces for system metrics assessment.

use serde::{Deserialize, Serialize};

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average response time in milliseconds
    pub response_time_ms: f64,
    /// Throughput in operations per second
    pub throughput_ops_per_sec: f64,
    /// Error rate percentage
    pub error_rate_percent: f64,
    /// Memory allocation rate
    pub allocation_rate_mb_per_sec: f64,
    /// Garbage collection frequency
    pub gc_frequency_per_hour: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            response_time_ms: 0.0,
            throughput_ops_per_sec: 0.0,
            error_rate_percent: 0.0,
            allocation_rate_mb_per_sec: 0.0,
            gc_frequency_per_hour: 0.0,
        }
    }
}

impl PerformanceMetrics {
    /// Create new performance metrics
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific values
    #[inline]
    pub fn with_values(
        response_time_ms: f64,
        throughput_ops_per_sec: f64,
        error_rate_percent: f64,
        allocation_rate_mb_per_sec: f64,
        gc_frequency_per_hour: f64,
    ) -> Self {
        Self {
            response_time_ms,
            throughput_ops_per_sec,
            error_rate_percent,
            allocation_rate_mb_per_sec,
            gc_frequency_per_hour,
        }
    }

    /// Check if performance is acceptable
    #[inline]
    pub fn is_performance_acceptable(&self) -> bool {
        self.response_time_ms < 500.0 &&
        self.throughput_ops_per_sec > 200.0 &&
        self.error_rate_percent < 1.0
    }

    /// Get performance score (0.0-1.0)
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let response_score = (1000.0 - self.response_time_ms.min(1000.0)) / 1000.0;
        let throughput_score = (self.throughput_ops_per_sec / 1000.0).min(1.0);
        let error_score = (100.0 - self.error_rate_percent.min(100.0)) / 100.0;
        
        (response_score + throughput_score + error_score) / 3.0
    }

    /// Check if response time is acceptable
    #[inline]
    pub fn is_response_time_acceptable(&self) -> bool {
        self.response_time_ms < 500.0
    }

    /// Check if throughput is acceptable
    #[inline]
    pub fn is_throughput_acceptable(&self) -> bool {
        self.throughput_ops_per_sec > 200.0
    }

    /// Check if error rate is acceptable
    #[inline]
    pub fn is_error_rate_acceptable(&self) -> bool {
        self.error_rate_percent < 1.0
    }

    /// Get performance bottleneck
    #[inline]
    pub fn get_bottleneck(&self) -> Option<&'static str> {
        if self.response_time_ms > 1000.0 {
            Some("response_time")
        } else if self.throughput_ops_per_sec < 100.0 {
            Some("throughput")
        } else if self.error_rate_percent > 5.0 {
            Some("error_rate")
        } else if self.allocation_rate_mb_per_sec > 100.0 {
            Some("allocation_rate")
        } else if self.gc_frequency_per_hour > 60.0 {
            Some("gc_frequency")
        } else {
            None
        }
    }

    /// Update metrics with new values
    #[inline]
    pub fn update(&mut self, 
                  response_time_ms: Option<f64>,
                  throughput_ops_per_sec: Option<f64>,
                  error_rate_percent: Option<f64>,
                  allocation_rate_mb_per_sec: Option<f64>,
                  gc_frequency_per_hour: Option<f64>) {
        if let Some(rt) = response_time_ms {
            self.response_time_ms = rt;
        }
        if let Some(tp) = throughput_ops_per_sec {
            self.throughput_ops_per_sec = tp;
        }
        if let Some(er) = error_rate_percent {
            self.error_rate_percent = er;
        }
        if let Some(ar) = allocation_rate_mb_per_sec {
            self.allocation_rate_mb_per_sec = ar;
        }
        if let Some(gc) = gc_frequency_per_hour {
            self.gc_frequency_per_hour = gc;
        }
    }
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Disk I/O usage percentage
    pub disk_io_percent: f64,
    /// Network usage percentage
    pub network_usage_percent: f64,
    /// File descriptor usage
    pub file_descriptor_usage: usize,
    /// Thread count
    pub thread_count: usize,
}

impl Default for ResourceUtilization {
    fn default() -> Self {
        Self {
            memory_usage_percent: 0.0,
            cpu_usage_percent: 0.0,
            disk_io_percent: 0.0,
            network_usage_percent: 0.0,
            file_descriptor_usage: 0,
            thread_count: 0,
        }
    }
}

impl ResourceUtilization {
    /// Create new resource utilization metrics
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific values
    #[inline]
    pub fn with_values(
        memory_usage_percent: f64,
        cpu_usage_percent: f64,
        disk_io_percent: f64,
        network_usage_percent: f64,
        file_descriptor_usage: usize,
        thread_count: usize,
    ) -> Self {
        Self {
            memory_usage_percent,
            cpu_usage_percent,
            disk_io_percent,
            network_usage_percent,
            file_descriptor_usage,
            thread_count,
        }
    }

    /// Check if resource usage is healthy
    #[inline]
    pub fn is_resource_usage_healthy(&self) -> bool {
        self.memory_usage_percent < 80.0 &&
        self.cpu_usage_percent < 80.0 &&
        self.disk_io_percent < 80.0
    }

    /// Get resource utilization score (0.0-1.0)
    #[inline]
    pub fn utilization_score(&self) -> f64 {
        let memory_score = (100.0 - self.memory_usage_percent.min(100.0)) / 100.0;
        let cpu_score = (100.0 - self.cpu_usage_percent.min(100.0)) / 100.0;
        let disk_score = (100.0 - self.disk_io_percent.min(100.0)) / 100.0;
        
        (memory_score + cpu_score + disk_score) / 3.0
    }

    /// Get highest usage component
    #[inline]
    pub fn highest_usage_component(&self) -> (&'static str, f64) {
        let components = [
            ("memory", self.memory_usage_percent),
            ("cpu", self.cpu_usage_percent),
            ("disk_io", self.disk_io_percent),
            ("network", self.network_usage_percent),
        ];

        components.iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|&(name, usage)| (name, usage))
            .unwrap_or(("unknown", 0.0))
    }

    /// Check if memory usage is critical
    #[inline]
    pub fn is_memory_usage_critical(&self) -> bool {
        self.memory_usage_percent > 90.0
    }

    /// Check if CPU usage is critical
    #[inline]
    pub fn is_cpu_usage_critical(&self) -> bool {
        self.cpu_usage_percent > 90.0
    }

    /// Check if disk I/O is critical
    #[inline]
    pub fn is_disk_io_critical(&self) -> bool {
        self.disk_io_percent > 90.0
    }

    /// Check if file descriptor usage is high
    #[inline]
    pub fn is_file_descriptor_usage_high(&self) -> bool {
        self.file_descriptor_usage > 1000
    }

    /// Check if thread count is high
    #[inline]
    pub fn is_thread_count_high(&self) -> bool {
        self.thread_count > 100
    }

    /// Get resource warnings
    #[inline]
    pub fn get_resource_warnings(&self) -> Vec<&'static str> {
        let mut warnings = Vec::new();

        if self.is_memory_usage_critical() {
            warnings.push("Critical memory usage");
        }
        if self.is_cpu_usage_critical() {
            warnings.push("Critical CPU usage");
        }
        if self.is_disk_io_critical() {
            warnings.push("Critical disk I/O usage");
        }
        if self.is_file_descriptor_usage_high() {
            warnings.push("High file descriptor usage");
        }
        if self.is_thread_count_high() {
            warnings.push("High thread count");
        }

        warnings
    }

    /// Update resource utilization with new values
    #[inline]
    pub fn update(&mut self,
                  memory_usage_percent: Option<f64>,
                  cpu_usage_percent: Option<f64>,
                  disk_io_percent: Option<f64>,
                  network_usage_percent: Option<f64>,
                  file_descriptor_usage: Option<usize>,
                  thread_count: Option<usize>) {
        if let Some(mem) = memory_usage_percent {
            self.memory_usage_percent = mem;
        }
        if let Some(cpu) = cpu_usage_percent {
            self.cpu_usage_percent = cpu;
        }
        if let Some(disk) = disk_io_percent {
            self.disk_io_percent = disk;
        }
        if let Some(net) = network_usage_percent {
            self.network_usage_percent = net;
        }
        if let Some(fd) = file_descriptor_usage {
            self.file_descriptor_usage = fd;
        }
        if let Some(tc) = thread_count {
            self.thread_count = tc;
        }
    }
}
