//! Core memory safety structures and types
//!
//! This module provides the core memory safety verification functionality with zero
//! allocation patterns, blazing-fast performance, and comprehensive safety validation
//! for production environments.

use arrayvec::{ArrayString, ArrayVec};
use dashmap::DashMap;
use memchr::memmem;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

/// Maximum number of memory safety violations to track without heap allocation
pub const MAX_SAFETY_VIOLATIONS: usize = 64;

/// Maximum size for safety violation messages
pub const MAX_VIOLATION_MESSAGE_SIZE: usize = 512;

/// Maximum size for memory operation identifiers
pub const MAX_OPERATION_ID_SIZE: usize = 128;

/// Maximum number of memory allocations to track
pub const MAX_TRACKED_ALLOCATIONS: usize = 1024;

/// Maximum number of concurrent safety rules
pub const MAX_SAFETY_RULES: usize = 32;

/// Maximum size for memory pattern detection
pub const MAX_PATTERN_SIZE: usize = 256;

/// Memory safety violation types with comprehensive coverage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyViolationType {
    /// Buffer overflow or underflow
    BufferOverflow,
    /// Use after free
    UseAfterFree,
    /// Double free
    DoubleFree,
    /// Memory leak
    MemoryLeak,
    /// Integer overflow
    IntegerOverflow,
    /// Null pointer dereference
    NullPointerDereference,
    /// Uninitialized memory access
    UninitializedMemory,
    /// Data race condition
    DataRace,
    /// Stack overflow
    StackOverflow,
    /// Heap corruption
    HeapCorruption,
    /// Resource exhaustion
    ResourceExhaustion,
    /// Invalid memory access
    InvalidMemoryAccess,
}

/// Memory safety violation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SafetyViolationSeverity {
    /// Low severity - potential issue
    Low,
    /// Medium severity - likely issue
    Medium,
    /// High severity - confirmed issue
    High,
    /// Critical severity - immediate action required
    Critical,
}

/// Memory safety violation with zero allocation storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySafetyViolation {
    /// Type of violation
    pub violation_type: SafetyViolationType,
    /// Severity level
    pub severity: SafetyViolationSeverity,
    /// Violation message (stack-allocated)
    pub message: ArrayString<MAX_VIOLATION_MESSAGE_SIZE>,
    /// Operation identifier (stack-allocated)
    pub operation_id: ArrayString<MAX_OPERATION_ID_SIZE>,
    /// Memory address involved
    pub memory_address: usize,
    /// Size of memory operation
    pub operation_size: usize,
    /// Timestamp of violation
    pub timestamp: u64,
    /// Thread ID where violation occurred
    pub thread_id: u64,
    /// Stack trace depth
    pub stack_depth: u16,
}

impl MemorySafetyViolation {
    /// Create new memory safety violation with optimized initialization
    pub fn new(
        violation_type: SafetyViolationType,
        severity: SafetyViolationSeverity,
        message: &str,
        operation_id: &str,
        memory_address: usize,
        operation_size: usize,
    ) -> Option<Self> {
        let message_str = ArrayString::from(message).ok()?;
        let operation_id_str = ArrayString::from(operation_id).ok()?;
        
        Some(Self {
            violation_type,
            severity,
            message: message_str,
            operation_id: operation_id_str,
            memory_address,
            operation_size,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            thread_id: std::thread::current().id().as_u64().get(),
            stack_depth: 0, // Would be populated by stack trace analysis
        })
    }

    /// Check if violation is critical
    pub fn is_critical(&self) -> bool {
        matches!(self.severity, SafetyViolationSeverity::Critical)
    }

    /// Check if violation requires immediate action
    pub fn requires_immediate_action(&self) -> bool {
        matches!(
            self.severity,
            SafetyViolationSeverity::Critical | SafetyViolationSeverity::High
        )
    }

    /// Get violation priority score for sorting
    pub fn priority_score(&self) -> u32 {
        let severity_score = match self.severity {
            SafetyViolationSeverity::Low => 1,
            SafetyViolationSeverity::Medium => 2,
            SafetyViolationSeverity::High => 3,
            SafetyViolationSeverity::Critical => 4,
        };

        let type_score = match self.violation_type {
            SafetyViolationType::UseAfterFree => 10,
            SafetyViolationType::DoubleFree => 9,
            SafetyViolationType::BufferOverflow => 8,
            SafetyViolationType::HeapCorruption => 7,
            SafetyViolationType::DataRace => 6,
            SafetyViolationType::NullPointerDereference => 5,
            SafetyViolationType::StackOverflow => 4,
            SafetyViolationType::InvalidMemoryAccess => 3,
            SafetyViolationType::IntegerOverflow => 2,
            SafetyViolationType::UninitializedMemory => 2,
            SafetyViolationType::MemoryLeak => 1,
            SafetyViolationType::ResourceExhaustion => 1,
        };

        severity_score * 10 + type_score
    }
}

/// Memory operation types for validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryOperationType {
    /// Memory allocation
    Allocation,
    /// Memory deallocation
    Deallocation,
    /// Memory read
    Read,
    /// Memory write
    Write,
    /// Memory copy
    Copy,
    /// Memory move
    Move,
    /// Memory comparison
    Compare,
    /// Memory set/fill
    Set,
    /// Pointer arithmetic
    PointerArithmetic,
    /// Memory mapping
    MemoryMap,
    /// Memory protection change
    ProtectionChange,
}

/// Memory operation descriptor with zero allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryOperation {
    /// Operation type
    pub operation_type: MemoryOperationType,
    /// Operation identifier (stack-allocated)
    pub operation_id: ArrayString<MAX_OPERATION_ID_SIZE>,
    /// Memory address
    pub address: usize,
    /// Operation size in bytes
    pub size: usize,
    /// Source address for copy/move operations
    pub source_address: Option<usize>,
    /// Destination address for copy/move operations
    pub destination_address: Option<usize>,
    /// Thread ID performing operation
    pub thread_id: u64,
    /// Timestamp of operation
    pub timestamp: u64,
    /// Stack depth at operation
    pub stack_depth: u16,
}

impl MemoryOperation {
    /// Create new memory operation with optimized initialization
    pub fn new(
        operation_type: MemoryOperationType,
        operation_id: &str,
        address: usize,
        size: usize,
    ) -> Option<Self> {
        let operation_id_str = ArrayString::from(operation_id).ok()?;
        
        Some(Self {
            operation_type,
            operation_id: operation_id_str,
            address,
            size,
            source_address: None,
            destination_address: None,
            thread_id: std::thread::current().id().as_u64().get(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            stack_depth: 0,
        })
    }

    /// Set source and destination for copy/move operations
    pub fn with_copy_addresses(mut self, source: usize, destination: usize) -> Self {
        self.source_address = Some(source);
        self.destination_address = Some(destination);
        self
    }

    /// Check if operation involves memory allocation
    pub fn is_allocation(&self) -> bool {
        matches!(self.operation_type, MemoryOperationType::Allocation)
    }

    /// Check if operation involves memory deallocation
    pub fn is_deallocation(&self) -> bool {
        matches!(self.operation_type, MemoryOperationType::Deallocation)
    }

    /// Check if operation is potentially unsafe
    pub fn is_potentially_unsafe(&self) -> bool {
        matches!(
            self.operation_type,
            MemoryOperationType::PointerArithmetic
                | MemoryOperationType::MemoryMap
                | MemoryOperationType::ProtectionChange
        )
    }

    /// Get memory range for operation
    pub fn memory_range(&self) -> (usize, usize) {
        (self.address, self.address.saturating_add(self.size))
    }

    /// Check if operation overlaps with given range
    pub fn overlaps_with(&self, start: usize, end: usize) -> bool {
        let (op_start, op_end) = self.memory_range();
        !(op_end <= start || op_start >= end)
    }
}

/// Memory safety validation result with zero allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySafetyResult {
    /// Safety violations found (stack-allocated)
    pub violations: ArrayVec<MemorySafetyViolation, MAX_SAFETY_VIOLATIONS>,
    /// Validation duration in microseconds
    pub duration_us: u64,
    /// Overall safety status
    pub is_safe: bool,
    /// Number of safety rules checked
    pub rules_checked: u32,
    /// Memory operations validated
    pub operations_validated: u32,
    /// Validation timestamp
    pub timestamp: u64,
}

impl MemorySafetyResult {
    /// Create new memory safety result with optimized initialization
    pub fn new() -> Self {
        Self {
            violations: ArrayVec::new(),
            duration_us: 0,
            is_safe: true,
            rules_checked: 0,
            operations_validated: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Add safety violation with capacity checking
    pub fn add_violation(
        &mut self,
        violation: MemorySafetyViolation,
    ) -> Result<(), MemorySafetyViolation> {
        if self.violations.try_push(violation).is_err() {
            // If we can't add more violations, create a capacity violation
            if let Some(capacity_violation) = MemorySafetyViolation::new(
                SafetyViolationType::ResourceExhaustion,
                SafetyViolationSeverity::Critical,
                "Maximum safety violations exceeded",
                "capacity_limit",
                0,
                MAX_SAFETY_VIOLATIONS,
            ) {
                return Err(capacity_violation);
            }
        }
        self.is_safe = false;
        Ok(())
    }

    /// Check if memory operation is safe
    pub fn is_safe(&self) -> bool {
        self.is_safe && self.violations.is_empty()
    }

    /// Get violation count by severity
    pub fn count_by_severity(&self, severity: SafetyViolationSeverity) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .count()
    }

    /// Get violation count by type
    pub fn count_by_type(&self, violation_type: SafetyViolationType) -> usize {
        self.violations
            .iter()
            .filter(|v| v.violation_type == violation_type)
            .count()
    }

    /// Get highest severity violation
    pub fn highest_severity(&self) -> Option<SafetyViolationSeverity> {
        self.violations
            .iter()
            .map(|v| v.severity)
            .max()
    }

    /// Get most critical violation
    pub fn most_critical_violation(&self) -> Option<&MemorySafetyViolation> {
        self.violations
            .iter()
            .max_by_key(|v| v.priority_score())
    }

    /// Sort violations by priority
    pub fn sort_by_priority(&mut self) {
        self.violations.sort_by_key(|v| std::cmp::Reverse(v.priority_score()));
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> ValidationSummary {
        ValidationSummary {
            total_violations: self.violations.len(),
            critical_violations: self.count_by_severity(SafetyViolationSeverity::Critical),
            high_violations: self.count_by_severity(SafetyViolationSeverity::High),
            medium_violations: self.count_by_severity(SafetyViolationSeverity::Medium),
            low_violations: self.count_by_severity(SafetyViolationSeverity::Low),
            duration_us: self.duration_us,
            rules_checked: self.rules_checked,
            operations_validated: self.operations_validated,
            is_safe: self.is_safe(),
        }
    }
}

impl Default for MemorySafetyResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total number of violations
    pub total_violations: usize,
    /// Critical severity violations
    pub critical_violations: usize,
    /// High severity violations
    pub high_violations: usize,
    /// Medium severity violations
    pub medium_violations: usize,
    /// Low severity violations
    pub low_violations: usize,
    /// Validation duration in microseconds
    pub duration_us: u64,
    /// Number of rules checked
    pub rules_checked: u32,
    /// Number of operations validated
    pub operations_validated: u32,
    /// Overall safety status
    pub is_safe: bool,
}

/// Memory allocation tracking entry with zero allocation
#[derive(Debug, Clone)]
pub struct AllocationEntry {
    /// Memory address
    pub address: usize,
    /// Allocation size
    pub size: usize,
    /// Allocation timestamp
    pub timestamp: u64,
    /// Thread ID that allocated
    pub thread_id: u64,
    /// Stack depth at allocation
    pub stack_depth: u16,
    /// Whether allocation is still valid
    pub is_valid: bool,
}

impl AllocationEntry {
    /// Create new allocation entry with optimized initialization
    pub fn new(address: usize, size: usize) -> Self {
        Self {
            address,
            size,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            thread_id: std::thread::current().id().as_u64().get(),
            stack_depth: 0,
            is_valid: true,
        }
    }

    /// Mark allocation as freed
    pub fn mark_freed(&mut self) {
        self.is_valid = false;
    }

    /// Check if allocation contains address
    pub fn contains_address(&self, addr: usize) -> bool {
        addr >= self.address && addr < self.address.saturating_add(self.size)
    }

    /// Get memory range
    pub fn memory_range(&self) -> (usize, usize) {
        (self.address, self.address.saturating_add(self.size))
    }

    /// Check if allocation overlaps with range
    pub fn overlaps_with(&self, start: usize, end: usize) -> bool {
        let (alloc_start, alloc_end) = self.memory_range();
        !(alloc_end <= start || alloc_start >= end)
    }

    /// Get allocation age in seconds
    pub fn age_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
            .saturating_sub(self.timestamp)
    }
}

/// Memory safety configuration with optimized defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySafetyConfig {
    /// Enable buffer overflow detection
    pub enable_buffer_overflow_detection: bool,
    /// Enable use-after-free detection
    pub enable_use_after_free_detection: bool,
    /// Enable memory leak detection
    pub enable_memory_leak_detection: bool,
    /// Enable integer overflow detection
    pub enable_integer_overflow_detection: bool,
    /// Enable data race detection
    pub enable_data_race_detection: bool,
    /// Maximum validation timeout in milliseconds
    pub validation_timeout_ms: u64,
    /// Maximum memory operations to track
    pub max_tracked_operations: usize,
    /// Memory leak detection threshold in seconds
    pub memory_leak_threshold_seconds: u64,
    /// Enable SIMD acceleration where available
    pub enable_simd_acceleration: bool,
    /// Enable production monitoring
    pub enable_production_monitoring: bool,
}

impl Default for MemorySafetyConfig {
    fn default() -> Self {
        Self {
            enable_buffer_overflow_detection: true,
            enable_use_after_free_detection: true,
            enable_memory_leak_detection: true,
            enable_integer_overflow_detection: true,
            enable_data_race_detection: true,
            validation_timeout_ms: 1000,
            max_tracked_operations: MAX_TRACKED_ALLOCATIONS,
            memory_leak_threshold_seconds: 300, // 5 minutes
            enable_simd_acceleration: true,
            enable_production_monitoring: true,
        }
    }
}

/// Memory safety metrics with atomic counters
#[derive(Debug)]
pub struct MemorySafetyMetrics {
    /// Total validations performed
    pub total_validations: AtomicU64,
    /// Total violations detected
    pub total_violations: AtomicU64,
    /// Critical violations detected
    pub critical_violations: AtomicU64,
    /// Buffer overflow violations
    pub buffer_overflow_violations: AtomicU64,
    /// Use-after-free violations
    pub use_after_free_violations: AtomicU64,
    /// Memory leak violations
    pub memory_leak_violations: AtomicU64,
    /// Average validation time in microseconds
    pub avg_validation_time_us: AtomicU64,
    /// Currently tracked allocations
    pub tracked_allocations: AtomicUsize,
    /// Peak tracked allocations
    pub peak_tracked_allocations: AtomicUsize,
    /// Total memory operations validated
    pub total_operations_validated: AtomicU64,
}

impl MemorySafetyMetrics {
    /// Create new metrics with optimized initialization
    pub fn new() -> Self {
        Self {
            total_validations: AtomicU64::new(0),
            total_violations: AtomicU64::new(0),
            critical_violations: AtomicU64::new(0),
            buffer_overflow_violations: AtomicU64::new(0),
            use_after_free_violations: AtomicU64::new(0),
            memory_leak_violations: AtomicU64::new(0),
            avg_validation_time_us: AtomicU64::new(0),
            tracked_allocations: AtomicUsize::new(0),
            peak_tracked_allocations: AtomicUsize::new(0),
            total_operations_validated: AtomicU64::new(0),
        }
    }

    /// Record validation with atomic updates
    pub fn record_validation(&self, result: &MemorySafetyResult) {
        self.total_validations.fetch_add(1, Ordering::Relaxed);
        self.total_violations.fetch_add(result.violations.len() as u64, Ordering::Relaxed);
        self.total_operations_validated.fetch_add(result.operations_validated as u64, Ordering::Relaxed);

        // Update average validation time
        let current_avg = self.avg_validation_time_us.load(Ordering::Relaxed);
        let new_avg = (current_avg + result.duration_us) / 2;
        self.avg_validation_time_us.store(new_avg, Ordering::Relaxed);

        // Count violations by type
        for violation in &result.violations {
            match violation.violation_type {
                SafetyViolationType::BufferOverflow => {
                    self.buffer_overflow_violations.fetch_add(1, Ordering::Relaxed);
                }
                SafetyViolationType::UseAfterFree => {
                    self.use_after_free_violations.fetch_add(1, Ordering::Relaxed);
                }
                SafetyViolationType::MemoryLeak => {
                    self.memory_leak_violations.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }

            if violation.is_critical() {
                self.critical_violations.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Update allocation tracking metrics
    pub fn update_allocation_tracking(&self, current_count: usize) {
        self.tracked_allocations.store(current_count, Ordering::Relaxed);
        
        // Update peak if necessary
        let current_peak = self.peak_tracked_allocations.load(Ordering::Relaxed);
        if current_count > current_peak {
            self.peak_tracked_allocations.store(current_count, Ordering::Relaxed);
        }
    }

    /// Get metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_validations: self.total_validations.load(Ordering::Relaxed),
            total_violations: self.total_violations.load(Ordering::Relaxed),
            critical_violations: self.critical_violations.load(Ordering::Relaxed),
            buffer_overflow_violations: self.buffer_overflow_violations.load(Ordering::Relaxed),
            use_after_free_violations: self.use_after_free_violations.load(Ordering::Relaxed),
            memory_leak_violations: self.memory_leak_violations.load(Ordering::Relaxed),
            avg_validation_time_us: self.avg_validation_time_us.load(Ordering::Relaxed),
            tracked_allocations: self.tracked_allocations.load(Ordering::Relaxed),
            peak_tracked_allocations: self.peak_tracked_allocations.load(Ordering::Relaxed),
            total_operations_validated: self.total_operations_validated.load(Ordering::Relaxed),
        }
    }
}

impl Default for MemorySafetyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Total validations performed
    pub total_validations: u64,
    /// Total violations detected
    pub total_violations: u64,
    /// Critical violations detected
    pub critical_violations: u64,
    /// Buffer overflow violations
    pub buffer_overflow_violations: u64,
    /// Use-after-free violations
    pub use_after_free_violations: u64,
    /// Memory leak violations
    pub memory_leak_violations: u64,
    /// Average validation time in microseconds
    pub avg_validation_time_us: u64,
    /// Currently tracked allocations
    pub tracked_allocations: usize,
    /// Peak tracked allocations
    pub peak_tracked_allocations: usize,
    /// Total memory operations validated
    pub total_operations_validated: u64,
}