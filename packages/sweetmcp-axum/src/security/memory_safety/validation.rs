//! Memory safety validation engine with comprehensive rule-based validation
//!
//! This module provides the core validation engine for memory safety operations
//! with zero allocation patterns, blazing-fast performance, and comprehensive
//! safety rule enforcement for production environments.

use crate::security::memory_safety::core::*;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

/// Memory safety rule trait for validation logic
pub trait MemorySafetyRule: Send + Sync {
    /// Validate memory operation against this rule
    fn validate(&self, operation: &MemoryOperation) -> Result<(), MemorySafetyViolation>;

    /// Get rule name for identification
    fn rule_name(&self) -> &'static str;

    /// Get rule severity level
    fn severity(&self) -> SafetyViolationSeverity;

    /// Check if rule applies to operation type
    fn applies_to(&self, operation_type: MemoryOperationType) -> bool {
        // Default implementation applies to all operations
        true
    }

    /// Get rule priority for ordering
    fn priority(&self) -> u32 {
        match self.severity() {
            SafetyViolationSeverity::Critical => 4,
            SafetyViolationSeverity::High => 3,
            SafetyViolationSeverity::Medium => 2,
            SafetyViolationSeverity::Low => 1,
        }
    }
}

/// Buffer overflow detection rule
#[derive(Debug)]
pub struct BufferOverflowRule;

impl MemorySafetyRule for BufferOverflowRule {
    fn validate(&self, operation: &MemoryOperation) -> Result<(), MemorySafetyViolation> {
        // Check for buffer overflow conditions
        if matches!(
            operation.operation_type,
            MemoryOperationType::Write | MemoryOperationType::Copy | MemoryOperationType::Set
        ) {
            // Check for integer overflow in size calculation
            if operation.address.checked_add(operation.size).is_none() {
                return Err(MemorySafetyViolation::new(
                    SafetyViolationType::BufferOverflow,
                    SafetyViolationSeverity::Critical,
                    "Buffer overflow - address + size overflow",
                    operation.operation_id.as_str(),
                    operation.address,
                    operation.size,
                )
                .ok_or_else(|| {
                    MemorySafetyViolation::new(
                        SafetyViolationType::BufferOverflow,
                        SafetyViolationSeverity::Critical,
                        "Buffer overflow validation failed",
                        "unknown",
                        0,
                        0,
                    )
                    .unwrap()
                })?);
            }

            // Check for zero-size operations that might indicate issues
            if operation.size == 0 {
                return Err(MemorySafetyViolation::new(
                    SafetyViolationType::InvalidMemoryAccess,
                    SafetyViolationSeverity::Medium,
                    "Zero-size memory operation detected",
                    operation.operation_id.as_str(),
                    operation.address,
                    operation.size,
                )
                .ok_or_else(|| {
                    MemorySafetyViolation::new(
                        SafetyViolationType::InvalidMemoryAccess,
                        SafetyViolationSeverity::Medium,
                        "Zero-size validation failed",
                        "unknown",
                        0,
                        0,
                    )
                    .unwrap()
                })?);
            }

            // Check for extremely large operations that might indicate overflow
            if operation.size > (1usize << 30) {
                // 1GB limit
                return Err(MemorySafetyViolation::new(
                    SafetyViolationType::BufferOverflow,
                    SafetyViolationSeverity::High,
                    "Extremely large memory operation - potential overflow",
                    operation.operation_id.as_str(),
                    operation.address,
                    operation.size,
                )
                .ok_or_else(|| {
                    MemorySafetyViolation::new(
                        SafetyViolationType::BufferOverflow,
                        SafetyViolationSeverity::High,
                        "Large operation validation failed",
                        "unknown",
                        0,
                        0,
                    )
                    .unwrap()
                })?);
            }
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "buffer_overflow_detection"
    }

    fn severity(&self) -> SafetyViolationSeverity {
        SafetyViolationSeverity::Critical
    }
}

/// Use-after-free detection rule
#[derive(Debug)]
pub struct UseAfterFreeRule;

impl MemorySafetyRule for UseAfterFreeRule {
    fn validate(&self, operation: &MemoryOperation) -> Result<(), MemorySafetyViolation> {
        // Check for use-after-free conditions
        if matches!(
            operation.operation_type,
            MemoryOperationType::Read | MemoryOperationType::Write
        ) {
            // Check for null pointer dereference
            if operation.address == 0 {
                return Err(MemorySafetyViolation::new(
                    SafetyViolationType::NullPointerDereference,
                    SafetyViolationSeverity::Critical,
                    "Null pointer dereference detected",
                    operation.operation_id.as_str(),
                    operation.address,
                    operation.size,
                )
                .ok_or_else(|| {
                    MemorySafetyViolation::new(
                        SafetyViolationType::NullPointerDereference,
                        SafetyViolationSeverity::Critical,
                        "Null pointer validation failed",
                        "unknown",
                        0,
                        0,
                    )
                    .unwrap()
                })?);
            }

            // Check for suspicious address patterns (potential use-after-free)
            if operation.address < 0x1000 || operation.address > usize::MAX - 0x1000 {
                return Err(MemorySafetyViolation::new(
                    SafetyViolationType::UseAfterFree,
                    SafetyViolationSeverity::Critical,
                    "Suspicious memory address - potential use-after-free",
                    operation.operation_id.as_str(),
                    operation.address,
                    operation.size,
                )
                .ok_or_else(|| {
                    MemorySafetyViolation::new(
                        SafetyViolationType::UseAfterFree,
                        SafetyViolationSeverity::Critical,
                        "Use-after-free validation failed",
                        "unknown",
                        0,
                        0,
                    )
                    .unwrap()
                })?);
            }
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "use_after_free_detection"
    }

    fn severity(&self) -> SafetyViolationSeverity {
        SafetyViolationSeverity::Critical
    }
}

/// Integer overflow detection rule
#[derive(Debug)]
pub struct IntegerOverflowRule;

impl MemorySafetyRule for IntegerOverflowRule {
    fn validate(&self, operation: &MemoryOperation) -> Result<(), MemorySafetyViolation> {
        // Check for integer overflow in pointer arithmetic
        if operation.operation_type == MemoryOperationType::PointerArithmetic {
            // Check for address + size overflow
            if operation.address.checked_add(operation.size).is_none() {
                return Err(MemorySafetyViolation::new(
                    SafetyViolationType::IntegerOverflow,
                    SafetyViolationSeverity::Critical,
                    "Integer overflow in pointer arithmetic",
                    operation.operation_id.as_str(),
                    operation.address,
                    operation.size,
                )
                .ok_or_else(|| {
                    MemorySafetyViolation::new(
                        SafetyViolationType::IntegerOverflow,
                        SafetyViolationSeverity::Critical,
                        "Integer overflow validation failed",
                        "unknown",
                        0,
                        0,
                    )
                    .unwrap()
                })?);
            }
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "integer_overflow_detection"
    }

    fn severity(&self) -> SafetyViolationSeverity {
        SafetyViolationSeverity::Critical
    }
}

/// Memory safety validator with comprehensive rule-based validation
pub struct MemorySafetyValidator {
    /// Validation rules
    rules: Vec<Arc<dyn MemorySafetyRule>>,
    /// Configuration
    config: MemorySafetyConfig,
    /// Metrics
    metrics: Arc<MemorySafetyMetrics>,
    /// Allocation tracking
    allocations: DashMap<usize, AllocationEntry>,
    /// Validation cache for performance
    result_cache: DashMap<u64, MemorySafetyResult>,
    /// Timeout duration
    timeout_duration: Duration,
    /// Atomic counters for performance
    total_validations: AtomicU64,
    successful_validations: AtomicU64,
    failed_validations: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl MemorySafetyValidator {
    /// Create new memory safety validator with optimized initialization
    pub fn new() -> Self {
        let mut validator = Self {
            rules: Vec::new(),
            config: MemorySafetyConfig::default(),
            metrics: Arc::new(MemorySafetyMetrics::new()),
            allocations: DashMap::new(),
            result_cache: DashMap::new(),
            timeout_duration: Duration::from_millis(1000),
            total_validations: AtomicU64::new(0),
            successful_validations: AtomicU64::new(0),
            failed_validations: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        };

        // Add default rules
        validator.add_rule(Arc::new(BufferOverflowRule));
        validator.add_rule(Arc::new(UseAfterFreeRule));
        validator.add_rule(Arc::new(IntegerOverflowRule));

        validator
    }

    /// Create validator with custom configuration
    pub fn with_config(config: MemorySafetyConfig) -> Self {
        let mut validator = Self::new();
        validator.config = config;
        validator.timeout_duration = Duration::from_millis(validator.config.validation_timeout_ms);
        validator
    }

    /// Add validation rule
    pub fn add_rule(&mut self, rule: Arc<dyn MemorySafetyRule>) {
        self.rules.push(rule);
        // Sort rules by priority (highest first)
        self.rules.sort_by_key(|rule| std::cmp::Reverse(rule.priority()));
    }

    /// Remove validation rule by name
    pub fn remove_rule(&mut self, rule_name: &str) {
        self.rules.retain(|rule| rule.rule_name() != rule_name);
    }

    /// Validate memory operation with comprehensive checking
    pub async fn validate_memory_operation(
        &self,
        operation: &MemoryOperation,
    ) -> Result<MemorySafetyResult, MemorySafetyViolation> {
        let start_time = SystemTime::now();
        self.total_validations.fetch_add(1, Ordering::Relaxed);

        // Check cache first for performance
        let cache_key = self.compute_cache_key(operation);
        if let Some(cached_result) = self.result_cache.get(&cache_key) {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(cached_result.clone());
        }
        self.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Perform validation with timeout
        let validation_future = self.validate_operation_internal(operation);
        let result = match timeout(self.timeout_duration, validation_future).await {
            Ok(result) => result,
            Err(_) => {
                self.failed_validations.fetch_add(1, Ordering::Relaxed);
                return Err(MemorySafetyViolation::new(
                    SafetyViolationType::ResourceExhaustion,
                    SafetyViolationSeverity::High,
                    "Validation timeout exceeded",
                    operation.operation_id.as_str(),
                    operation.address,
                    operation.size,
                )
                .ok_or_else(|| {
                    MemorySafetyViolation::new(
                        SafetyViolationType::ResourceExhaustion,
                        SafetyViolationSeverity::High,
                        "Timeout validation failed",
                        "unknown",
                        0,
                        0,
                    )
                    .unwrap()
                })?);
            }
        };

        // Update metrics
        let duration = start_time.elapsed().map(|d| d.as_micros() as u64).unwrap_or(0);
        let mut final_result = result?;
        final_result.duration_us = duration;
        final_result.rules_checked = self.rules.len() as u32;
        final_result.operations_validated = 1;

        if final_result.is_safe() {
            self.successful_validations.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_validations.fetch_add(1, Ordering::Relaxed);
        }

        // Update allocation tracking
        self.update_allocation_tracking(operation);

        // Cache result for future use
        self.result_cache.insert(cache_key, final_result.clone());

        // Update metrics
        self.metrics.record_validation(&final_result);

        Ok(final_result)
    }

    /// Internal validation logic
    async fn validate_operation_internal(
        &self,
        operation: &MemoryOperation,
    ) -> Result<MemorySafetyResult, MemorySafetyViolation> {
        let mut result = MemorySafetyResult::new();

        // Apply all relevant rules
        for rule in &self.rules {
            if rule.applies_to(operation.operation_type) {
                if let Err(violation) = rule.validate(operation) {
                    if result.add_violation(violation).is_err() {
                        // Maximum violations reached
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Compute cache key for operation
    fn compute_cache_key(&self, operation: &MemoryOperation) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        operation.operation_type.hash(&mut hasher);
        operation.address.hash(&mut hasher);
        operation.size.hash(&mut hasher);
        operation.source_address.hash(&mut hasher);
        operation.destination_address.hash(&mut hasher);
        hasher.finish()
    }

    /// Update allocation tracking
    fn update_allocation_tracking(&self, operation: &MemoryOperation) {
        match operation.operation_type {
            MemoryOperationType::Allocation => {
                let entry = AllocationEntry::new(operation.address, operation.size);
                self.allocations.insert(operation.address, entry);
            }
            MemoryOperationType::Deallocation => {
                if let Some(mut entry) = self.allocations.get_mut(&operation.address) {
                    entry.mark_freed();
                }
            }
            _ => {}
        }
    }

    /// Scan for memory leaks
    pub async fn scan_for_leaks(&self) -> Result<MemorySafetyResult, MemorySafetyViolation> {
        let mut result = MemorySafetyResult::new();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        for entry in self.allocations.iter() {
            if entry.age_seconds() > self.config.memory_leak_threshold_seconds && entry.is_valid {
                if let Some(violation) = MemorySafetyViolation::new(
                    SafetyViolationType::MemoryLeak,
                    SafetyViolationSeverity::High,
                    "Memory leak detected - allocation not freed",
                    "leak_scan",
                    entry.address,
                    entry.size,
                ) {
                    if result.add_violation(violation).is_err() {
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Get validation metrics
    pub fn get_metrics(&self) -> ValidationMetrics {
        ValidationMetrics {
            total_validations: self.total_validations.load(Ordering::Relaxed),
            successful_validations: self.successful_validations.load(Ordering::Relaxed),
            failed_validations: self.failed_validations.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            cache_size: self.result_cache.len() as u64,
            rule_count: self.rules.len() as u32,
            tracked_allocations: self.allocations.len() as u64,
            metrics_snapshot: self.metrics.snapshot(),
        }
    }

    /// Clear validation cache
    pub fn clear_cache(&self) {
        self.result_cache.clear();
    }

    /// Clear allocation tracking
    pub fn clear_allocations(&self) {
        self.allocations.clear();
    }

    /// Set validation timeout
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }

    /// Get configuration
    pub fn get_config(&self) -> &MemorySafetyConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: MemorySafetyConfig) {
        self.config = config;
        self.timeout_duration = Duration::from_millis(self.config.validation_timeout_ms);
    }
}

impl Default for MemorySafetyValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Total validations performed
    pub total_validations: u64,
    /// Successful validations
    pub successful_validations: u64,
    /// Failed validations
    pub failed_validations: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Cache size
    pub cache_size: u64,
    /// Number of rules
    pub rule_count: u32,
    /// Tracked allocations
    pub tracked_allocations: u64,
    /// Metrics snapshot
    pub metrics_snapshot: MetricsSnapshot,
}

impl ValidationMetrics {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_validations == 0 {
            0.0
        } else {
            self.successful_validations as f64 / self.total_validations as f64
        }
    }

    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_requests as f64
        }
    }

    /// Check if performance is acceptable
    pub fn is_performance_acceptable(&self) -> bool {
        self.success_rate() > 0.95 && self.cache_hit_rate() > 0.8
    }
}