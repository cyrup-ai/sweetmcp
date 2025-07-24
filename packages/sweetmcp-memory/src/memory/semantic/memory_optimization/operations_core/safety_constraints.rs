//! Safety constraints for optimization operations
//!
//! This module provides blazing-fast safety validation with zero allocation
//! optimizations and elegant ergonomic interfaces for operation safety.

use std::time::Duration;

/// Safety constraints for optimization operations
#[derive(Debug, Clone)]
pub struct SafetyConstraints {
    /// Maximum memory usage allowed during optimization
    pub max_memory_usage: usize,
    /// Maximum CPU usage percentage
    pub max_cpu_usage: f64,
    /// Require backup before destructive operations
    pub require_backup: bool,
    /// Enable rollback on failure
    pub enable_rollback: bool,
    /// Maximum items to process in single operation
    pub max_items_per_operation: usize,
    /// Timeout for individual operations
    pub operation_timeout: Duration,
}

impl SafetyConstraints {
    /// Create strict safety constraints
    #[inline]
    pub fn strict() -> Self {
        Self {
            max_memory_usage: 1024 * 1024 * 100, // 100MB
            max_cpu_usage: 50.0,
            require_backup: true,
            enable_rollback: true,
            max_items_per_operation: 1000,
            operation_timeout: Duration::from_secs(30),
        }
    }

    /// Create relaxed safety constraints
    #[inline]
    pub fn relaxed() -> Self {
        Self {
            max_memory_usage: 1024 * 1024 * 500, // 500MB
            max_cpu_usage: 80.0,
            require_backup: false,
            enable_rollback: true,
            max_items_per_operation: 10000,
            operation_timeout: Duration::from_secs(120),
        }
    }

    /// Create balanced safety constraints
    #[inline]
    pub fn balanced() -> Self {
        Self {
            max_memory_usage: 1024 * 1024 * 250, // 250MB
            max_cpu_usage: 70.0,
            require_backup: true,
            enable_rollback: true,
            max_items_per_operation: 5000,
            operation_timeout: Duration::from_secs(60),
        }
    }

    /// Create production safety constraints
    #[inline]
    pub fn production() -> Self {
        Self {
            max_memory_usage: 1024 * 1024 * 200, // 200MB
            max_cpu_usage: 60.0,
            require_backup: true,
            enable_rollback: true,
            max_items_per_operation: 2000,
            operation_timeout: Duration::from_secs(45),
        }
    }

    /// Create development safety constraints
    #[inline]
    pub fn development() -> Self {
        Self {
            max_memory_usage: 1024 * 1024 * 1000, // 1GB
            max_cpu_usage: 90.0,
            require_backup: false,
            enable_rollback: false,
            max_items_per_operation: 50000,
            operation_timeout: Duration::from_secs(300),
        }
    }

    /// Check if constraints are satisfied
    #[inline]
    pub fn are_satisfied(&self) -> bool {
        self.max_memory_usage > 0 &&
        self.max_cpu_usage > 0.0 && self.max_cpu_usage <= 100.0 &&
        self.max_items_per_operation > 0 &&
        self.operation_timeout > Duration::from_secs(0)
    }

    /// Validate memory usage against constraints
    #[inline]
    pub fn validate_memory_usage(&self, current_usage: usize) -> bool {
        current_usage <= self.max_memory_usage
    }

    /// Validate CPU usage against constraints
    #[inline]
    pub fn validate_cpu_usage(&self, current_usage: f64) -> bool {
        current_usage <= self.max_cpu_usage
    }

    /// Validate operation item count
    #[inline]
    pub fn validate_item_count(&self, item_count: usize) -> bool {
        item_count <= self.max_items_per_operation
    }

    /// Check if backup is required
    #[inline]
    pub fn backup_required(&self) -> bool {
        self.require_backup
    }

    /// Check if rollback is enabled
    #[inline]
    pub fn rollback_enabled(&self) -> bool {
        self.enable_rollback
    }

    /// Get safety level description
    #[inline]
    pub fn safety_level(&self) -> String {
        if self == &Self::strict() {
            "Strict".to_string()
        } else if self == &Self::relaxed() {
            "Relaxed".to_string()
        } else if self == &Self::balanced() {
            "Balanced".to_string()
        } else if self == &Self::production() {
            "Production".to_string()
        } else if self == &Self::development() {
            "Development".to_string()
        } else {
            "Custom".to_string()
        }
    }

    /// Get constraint strictness score (0.0-1.0, higher is stricter)
    #[inline]
    pub fn strictness_score(&self) -> f64 {
        let memory_score = 1.0 - (self.max_memory_usage as f64 / (1024.0 * 1024.0 * 1000.0)).min(1.0);
        let cpu_score = 1.0 - (self.max_cpu_usage / 100.0);
        let items_score = 1.0 - (self.max_items_per_operation as f64 / 50000.0).min(1.0);
        let timeout_score = 1.0 - (self.operation_timeout.as_secs() as f64 / 300.0).min(1.0);
        let backup_score = if self.require_backup { 1.0 } else { 0.0 };
        let rollback_score = if self.enable_rollback { 0.8 } else { 0.0 };

        (memory_score + cpu_score + items_score + timeout_score + backup_score + rollback_score) / 6.0
    }

    /// Check if constraints are suitable for item count
    #[inline]
    pub fn is_suitable_for_item_count(&self, item_count: usize) -> bool {
        let operations_needed = (item_count / self.max_items_per_operation.max(1)) + 1;
        let estimated_time = self.operation_timeout * operations_needed as u32;
        
        // Check if estimated time is reasonable (less than 30 minutes)
        estimated_time < Duration::from_secs(1800)
    }

    /// Get recommended batch size for item count
    #[inline]
    pub fn recommended_batch_size(&self, total_items: usize) -> usize {
        if total_items <= self.max_items_per_operation {
            total_items
        } else {
            self.max_items_per_operation
        }
    }

    /// Calculate estimated operations for item count
    #[inline]
    pub fn estimated_operations(&self, item_count: usize) -> usize {
        (item_count / self.max_items_per_operation.max(1)) + 1
    }

    /// Calculate estimated total time for item count
    #[inline]
    pub fn estimated_total_time(&self, item_count: usize) -> Duration {
        let operations = self.estimated_operations(item_count);
        self.operation_timeout * operations as u32
    }

    /// Validate constraints configuration
    #[inline]
    pub fn validate_configuration(&self) -> Result<(), &'static str> {
        if self.max_memory_usage == 0 {
            return Err("Maximum memory usage must be greater than 0");
        }
        if self.max_cpu_usage <= 0.0 || self.max_cpu_usage > 100.0 {
            return Err("Maximum CPU usage must be between 0 and 100");
        }
        if self.max_items_per_operation == 0 {
            return Err("Maximum items per operation must be greater than 0");
        }
        if self.operation_timeout.as_secs() == 0 {
            return Err("Operation timeout must be greater than 0");
        }
        Ok(())
    }

    /// Create custom constraints with validation
    #[inline]
    pub fn custom(
        max_memory_usage: usize,
        max_cpu_usage: f64,
        require_backup: bool,
        enable_rollback: bool,
        max_items_per_operation: usize,
        operation_timeout: Duration,
    ) -> Result<Self, &'static str> {
        let constraints = Self {
            max_memory_usage,
            max_cpu_usage,
            require_backup,
            enable_rollback,
            max_items_per_operation,
            operation_timeout,
        };

        constraints.validate_configuration()?;
        Ok(constraints)
    }

    /// Builder pattern for creating custom constraints
    #[inline]
    pub fn builder() -> SafetyConstraintsBuilder {
        SafetyConstraintsBuilder::new()
    }

    /// Check if constraints allow aggressive optimization
    #[inline]
    pub fn allows_aggressive_optimization(&self) -> bool {
        self.max_memory_usage >= 1024 * 1024 * 300 && // 300MB+
        self.max_cpu_usage >= 70.0 &&
        self.max_items_per_operation >= 5000 &&
        self.operation_timeout >= Duration::from_secs(60)
    }

    /// Check if constraints are conservative
    #[inline]
    pub fn is_conservative(&self) -> bool {
        self.max_memory_usage <= 1024 * 1024 * 150 && // 150MB or less
        self.max_cpu_usage <= 60.0 &&
        self.max_items_per_operation <= 2000 &&
        self.operation_timeout <= Duration::from_secs(45) &&
        self.require_backup &&
        self.enable_rollback
    }

    /// Get constraint summary
    #[inline]
    pub fn summary(&self) -> ConstraintSummary {
        ConstraintSummary {
            safety_level: self.safety_level(),
            strictness_score: self.strictness_score(),
            max_memory_mb: self.max_memory_usage / (1024 * 1024),
            max_cpu_usage: self.max_cpu_usage,
            max_items_per_operation: self.max_items_per_operation,
            operation_timeout_secs: self.operation_timeout.as_secs(),
            backup_required: self.require_backup,
            rollback_enabled: self.enable_rollback,
            allows_aggressive: self.allows_aggressive_optimization(),
            is_conservative: self.is_conservative(),
        }
    }
}

impl Default for SafetyConstraints {
    fn default() -> Self {
        Self::balanced()
    }
}

impl PartialEq for SafetyConstraints {
    fn eq(&self, other: &Self) -> bool {
        self.max_memory_usage == other.max_memory_usage &&
        (self.max_cpu_usage - other.max_cpu_usage).abs() < f64::EPSILON &&
        self.require_backup == other.require_backup &&
        self.enable_rollback == other.enable_rollback &&
        self.max_items_per_operation == other.max_items_per_operation &&
        self.operation_timeout == other.operation_timeout
    }
}

/// Builder for creating custom safety constraints
pub struct SafetyConstraintsBuilder {
    max_memory_usage: usize,
    max_cpu_usage: f64,
    require_backup: bool,
    enable_rollback: bool,
    max_items_per_operation: usize,
    operation_timeout: Duration,
}

impl SafetyConstraintsBuilder {
    /// Create new constraints builder
    #[inline]
    pub fn new() -> Self {
        Self {
            max_memory_usage: 1024 * 1024 * 250, // 250MB
            max_cpu_usage: 70.0,
            require_backup: true,
            enable_rollback: true,
            max_items_per_operation: 5000,
            operation_timeout: Duration::from_secs(60),
        }
    }

    /// Set maximum memory usage
    #[inline]
    pub fn max_memory_usage(mut self, bytes: usize) -> Self {
        self.max_memory_usage = bytes;
        self
    }

    /// Set maximum memory usage in MB
    #[inline]
    pub fn max_memory_mb(mut self, mb: usize) -> Self {
        self.max_memory_usage = mb * 1024 * 1024;
        self
    }

    /// Set maximum CPU usage
    #[inline]
    pub fn max_cpu_usage(mut self, percentage: f64) -> Self {
        self.max_cpu_usage = percentage;
        self
    }

    /// Set backup requirement
    #[inline]
    pub fn require_backup(mut self, require: bool) -> Self {
        self.require_backup = require;
        self
    }

    /// Set rollback enablement
    #[inline]
    pub fn enable_rollback(mut self, enable: bool) -> Self {
        self.enable_rollback = enable;
        self
    }

    /// Set maximum items per operation
    #[inline]
    pub fn max_items_per_operation(mut self, count: usize) -> Self {
        self.max_items_per_operation = count;
        self
    }

    /// Set operation timeout
    #[inline]
    pub fn operation_timeout(mut self, timeout: Duration) -> Self {
        self.operation_timeout = timeout;
        self
    }

    /// Set operation timeout in seconds
    #[inline]
    pub fn operation_timeout_secs(mut self, seconds: u64) -> Self {
        self.operation_timeout = Duration::from_secs(seconds);
        self
    }

    /// Build the safety constraints
    #[inline]
    pub fn build(self) -> Result<SafetyConstraints, &'static str> {
        SafetyConstraints::custom(
            self.max_memory_usage,
            self.max_cpu_usage,
            self.require_backup,
            self.enable_rollback,
            self.max_items_per_operation,
            self.operation_timeout,
        )
    }

    /// Build with validation disabled (use with caution)
    #[inline]
    pub fn build_unchecked(self) -> SafetyConstraints {
        SafetyConstraints {
            max_memory_usage: self.max_memory_usage,
            max_cpu_usage: self.max_cpu_usage,
            require_backup: self.require_backup,
            enable_rollback: self.enable_rollback,
            max_items_per_operation: self.max_items_per_operation,
            operation_timeout: self.operation_timeout,
        }
    }
}

impl Default for SafetyConstraintsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Constraint summary for reporting
#[derive(Debug, Clone)]
pub struct ConstraintSummary {
    pub safety_level: String,
    pub strictness_score: f64,
    pub max_memory_mb: usize,
    pub max_cpu_usage: f64,
    pub max_items_per_operation: usize,
    pub operation_timeout_secs: u64,
    pub backup_required: bool,
    pub rollback_enabled: bool,
    pub allows_aggressive: bool,
    pub is_conservative: bool,
}

impl ConstraintSummary {
    /// Check if constraints are well-balanced
    #[inline]
    pub fn is_well_balanced(&self) -> bool {
        self.strictness_score > 0.3 && self.strictness_score < 0.8 &&
        self.max_memory_mb >= 100 && self.max_memory_mb <= 500 &&
        self.max_cpu_usage >= 50.0 && self.max_cpu_usage <= 80.0
    }

    /// Get configuration recommendations
    #[inline]
    pub fn recommendations(&self) -> Vec<&'static str> {
        let mut recommendations = Vec::new();

        if self.strictness_score > 0.9 {
            recommendations.push("Constraints are very strict, consider relaxing for better performance");
        } else if self.strictness_score < 0.2 {
            recommendations.push("Constraints are very relaxed, consider tightening for safety");
        }

        if self.max_memory_mb < 50 {
            recommendations.push("Memory limit is very low, may cause frequent failures");
        } else if self.max_memory_mb > 1000 {
            recommendations.push("Memory limit is very high, consider reducing for safety");
        }

        if self.max_cpu_usage < 30.0 {
            recommendations.push("CPU limit is very low, may cause slow performance");
        } else if self.max_cpu_usage > 90.0 {
            recommendations.push("CPU limit is very high, may impact system responsiveness");
        }

        if !self.backup_required && !self.rollback_enabled {
            recommendations.push("No backup or rollback enabled, consider enabling for safety");
        }

        recommendations
    }

    /// Get overall safety score (0.0-1.0)
    #[inline]
    pub fn safety_score(&self) -> f64 {
        let backup_score = if self.backup_required { 1.0 } else { 0.0 };
        let rollback_score = if self.rollback_enabled { 1.0 } else { 0.0 };
        let memory_score = if self.max_memory_mb <= 500 { 1.0 } else { 0.5 };
        let cpu_score = if self.max_cpu_usage <= 80.0 { 1.0 } else { 0.5 };
        let timeout_score = if self.operation_timeout_secs <= 120 { 1.0 } else { 0.7 };

        (backup_score + rollback_score + memory_score + cpu_score + timeout_score) / 5.0
    }
}
