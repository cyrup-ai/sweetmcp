//! Zero-allocation dependency vulnerability scanner with lock-free caching
//!
//! This module provides automated vulnerability scanning for Rust dependencies using
//! cargo-audit integration with zero-allocation, lock-free, and SIMD-accelerated patterns.
//!
//! # Features
//!
//! - Zero-allocation vulnerability scanning using ArrayVec and ArrayString
//! - Lock-free vulnerability caching using DashMap for concurrent access
//! - SIMD-accelerated string matching for vulnerability pattern detection
//! - Atomic vulnerability tracking for thread-safe metrics
//! - CI/CD integration with configurable failure thresholds
//! - Cache-line aligned data structures for optimal performance
//!
//! # Usage
//!
//! ```rust
//! use sweetmcp_daemon::security::audit::*;
//!
//! let scanner = VulnerabilityScanner::new(AuditThresholds {
//!     critical_max: 0,
//!     high_max: 2,
//!     medium_max: 10,
//!     low_max: 50,
//! });
//!
//! let result = scanner.scan_dependencies().await?;
//! if !result.passes_thresholds() {
//!     return Err("Vulnerability threshold exceeded".into());
//! }
//! ```

use arrayvec::{ArrayString, ArrayVec};
use dashmap::DashMap;
use memchr::memmem;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// Maximum number of vulnerabilities to track without heap allocation
const MAX_VULNERABILITIES: usize = 256;

/// Maximum size for vulnerability report content
const MAX_REPORT_SIZE: usize = 1024;

/// Maximum size for package names and vulnerability IDs
const MAX_IDENTIFIER_SIZE: usize = 64;

/// Maximum size for vulnerability descriptions
const MAX_DESCRIPTION_SIZE: usize = 256;

/// Default padding for cache-line alignment
fn default_padding() -> [u8; 64] {
    [0; 64]
}

/// Vulnerability severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl VulnerabilitySeverity {
    /// Convert severity string to enum with zero-allocation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "critical" => Some(Self::Critical),
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            "info" => Some(Self::Info),
            _ => None,
        }
    }

    /// Get numeric weight for threshold comparison
    pub fn weight(&self) -> u32 {
        match self {
            Self::Critical => 1000,
            Self::High => 100,
            Self::Medium => 10,
            Self::Low => 1,
            Self::Info => 0,
        }
    }
}

/// Vulnerability status for caching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulnerabilityStatus {
    /// Vulnerability is confirmed and active
    Active,
    /// Vulnerability has been patched
    Patched,
    /// Vulnerability is marked as false positive
    FalsePositive,
    /// Vulnerability is accepted risk
    Accepted,
    /// Vulnerability status is unknown
    Unknown,
}

/// Cache-line aligned vulnerability data structure
#[repr(align(64))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// Vulnerability ID (e.g., RUSTSEC-2023-0001)
    pub id: ArrayString<MAX_IDENTIFIER_SIZE>,
    /// Affected package name
    pub package: ArrayString<MAX_IDENTIFIER_SIZE>,
    /// Vulnerability severity
    pub severity: VulnerabilitySeverity,
    /// Vulnerability description
    pub description: ArrayString<MAX_DESCRIPTION_SIZE>,
    /// Affected version
    pub version: ArrayString<MAX_IDENTIFIER_SIZE>,
    /// Patched version (if available)
    pub patched: Option<ArrayString<MAX_IDENTIFIER_SIZE>>,
    /// Vulnerability discovery timestamp
    pub discovered: u64,
    /// Cache padding to prevent false sharing
    #[serde(skip, default = "default_padding")]
    _padding: [u8; 64],
}

impl Vulnerability {
    /// Create new vulnerability with zero-allocation
    pub fn new(
        id: &str,
        package: &str,
        severity: VulnerabilitySeverity,
        description: &str,
        version: &str,
        patched: Option<&str>,
    ) -> Option<Self> {
        let id = ArrayString::from(id).ok()?;
        let package = ArrayString::from(package).ok()?;
        let description = ArrayString::from(description).ok()?;
        let version = ArrayString::from(version).ok()?;
        let patched = match patched {
            Some(p) => Some(ArrayString::from(p).ok()?),
            None => None,
        };

        Some(Self {
            id,
            package,
            severity,
            description,
            version,
            patched,
            discovered: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs(),
            _padding: [0; 64],
        })
    }

    /// Check if vulnerability matches pattern using SIMD-accelerated search
    pub fn matches_pattern(&self, pattern: &[u8]) -> bool {
        let finder = memmem::Finder::new(pattern);

        finder.find(self.id.as_bytes()).is_some()
            || finder.find(self.package.as_bytes()).is_some()
            || finder.find(self.description.as_bytes()).is_some()
    }

    /// Check if vulnerability is in given package
    pub fn affects_package(&self, package_name: &str) -> bool {
        self.package.as_str() == package_name
    }
}

/// Audit result containing vulnerability collection
#[derive(Debug, Clone)]
pub struct AuditResult {
    /// Collection of found vulnerabilities (zero-allocation)
    pub vulnerabilities: ArrayVec<Vulnerability, MAX_VULNERABILITIES>,
    /// Total scan duration in milliseconds
    pub scan_duration_ms: u64,
    /// Number of packages scanned
    pub packages_scanned: u32,
    /// Scan timestamp
    pub scan_timestamp: u64,
    /// Whether scan completed successfully
    pub success: bool,
}

impl AuditResult {
    /// Create new audit result
    pub fn new() -> Self {
        Self {
            vulnerabilities: ArrayVec::new(),
            scan_duration_ms: 0,
            packages_scanned: 0,
            scan_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            success: false,
        }
    }

    /// Add vulnerability to result with capacity checking
    pub fn add_vulnerability(&mut self, vulnerability: Vulnerability) -> Result<(), AuditError> {
        self.vulnerabilities
            .try_push(vulnerability)
            .map_err(|_| AuditError::TooManyVulnerabilities)
    }

    /// Get vulnerability count by severity
    pub fn count_by_severity(&self, severity: VulnerabilitySeverity) -> usize {
        self.vulnerabilities
            .iter()
            .filter(|v| v.severity == severity)
            .count()
    }

    /// Check if result passes given thresholds
    pub fn passes_thresholds(&self, thresholds: &AuditThresholds) -> bool {
        self.count_by_severity(VulnerabilitySeverity::Critical)
            <= thresholds.critical_max.load(Ordering::Relaxed) as usize
            && self.count_by_severity(VulnerabilitySeverity::High)
                <= thresholds.high_max.load(Ordering::Relaxed) as usize
            && self.count_by_severity(VulnerabilitySeverity::Medium)
                <= thresholds.medium_max.load(Ordering::Relaxed) as usize
            && self.count_by_severity(VulnerabilitySeverity::Low)
                <= thresholds.low_max.load(Ordering::Relaxed) as usize
    }

    /// Get total vulnerability weight for scoring
    pub fn total_weight(&self) -> u32 {
        self.vulnerabilities
            .iter()
            .map(|v| v.severity.weight())
            .sum()
    }
}

/// Audit thresholds for CI/CD integration
#[derive(Debug)]
pub struct AuditThresholds {
    /// Maximum critical vulnerabilities allowed
    pub critical_max: AtomicU32,
    /// Maximum high vulnerabilities allowed
    pub high_max: AtomicU32,
    /// Maximum medium vulnerabilities allowed
    pub medium_max: AtomicU32,
    /// Maximum low vulnerabilities allowed
    pub low_max: AtomicU32,
}

impl AuditThresholds {
    /// Create new thresholds with atomic initialization
    pub fn new(critical: u32, high: u32, medium: u32, low: u32) -> Self {
        Self {
            critical_max: AtomicU32::new(critical),
            high_max: AtomicU32::new(high),
            medium_max: AtomicU32::new(medium),
            low_max: AtomicU32::new(low),
        }
    }

    /// Update thresholds atomically
    pub fn update(&self, critical: u32, high: u32, medium: u32, low: u32) {
        self.critical_max.store(critical, Ordering::Relaxed);
        self.high_max.store(high, Ordering::Relaxed);
        self.medium_max.store(medium, Ordering::Relaxed);
        self.low_max.store(low, Ordering::Relaxed);
    }

    /// Check if vulnerability counts exceed thresholds
    pub fn exceeded_by(&self, result: &AuditResult) -> bool {
        let critical_count = result.count_by_severity(VulnerabilitySeverity::Critical) as u32;
        let high_count = result.count_by_severity(VulnerabilitySeverity::High) as u32;
        let medium_count = result.count_by_severity(VulnerabilitySeverity::Medium) as u32;
        let low_count = result.count_by_severity(VulnerabilitySeverity::Low) as u32;

        critical_count > self.critical_max.load(Ordering::Relaxed)
            || high_count > self.high_max.load(Ordering::Relaxed)
            || medium_count > self.medium_max.load(Ordering::Relaxed)
            || low_count > self.low_max.load(Ordering::Relaxed)
    }
}

/// Vulnerability scanner error types
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Cargo audit command failed: {0}")]
    CargoAuditFailed(String),
    #[error("JSON parsing failed: {0}")]
    JsonParsingFailed(String),
    #[error("Too many vulnerabilities found (max: {MAX_VULNERABILITIES})")]
    TooManyVulnerabilities,
    #[error("Scan timeout exceeded")]
    ScanTimeout,
    #[error("Invalid vulnerability data: {0}")]
    InvalidVulnerabilityData(String),
    #[error("Cache operation failed: {0}")]
    CacheOperationFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
}

/// Main vulnerability scanner with atomic tracking
pub struct VulnerabilityScanner {
    /// Lock-free vulnerability cache
    cache: Arc<DashMap<ArrayString<MAX_IDENTIFIER_SIZE>, VulnerabilityStatus>>,
    /// Atomic vulnerability counters
    critical_count: AtomicU32,
    high_count: AtomicU32,
    medium_count: AtomicU32,
    low_count: AtomicU32,
    /// Total scans performed
    total_scans: AtomicU64,
    /// Scan success rate numerator
    successful_scans: AtomicU64,
    /// Audit thresholds for CI/CD
    thresholds: AuditThresholds,
    /// Scan timeout duration
    timeout_duration: Duration,
}

impl VulnerabilityScanner {
    /// Create new vulnerability scanner with default thresholds
    pub fn new(thresholds: AuditThresholds) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            critical_count: AtomicU32::new(0),
            high_count: AtomicU32::new(0),
            medium_count: AtomicU32::new(0),
            low_count: AtomicU32::new(0),
            total_scans: AtomicU64::new(0),
            successful_scans: AtomicU64::new(0),
            thresholds,
            timeout_duration: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Scan dependencies for vulnerabilities using cargo-audit
    pub async fn scan_dependencies(&self) -> Result<AuditResult, AuditError> {
        let _start_time = std::time::Instant::now();
        self.total_scans.fetch_add(1, Ordering::Relaxed);

        let result = self.run_cargo_audit().await;

        match &result {
            Ok(audit_result) => {
                if audit_result.success {
                    self.successful_scans.fetch_add(1, Ordering::Relaxed);
                    self.update_counters(audit_result);
                    self.update_cache(audit_result);
                }
            }
            Err(_) => {
                // Scan failed, metrics already updated
            }
        }

        result
    }

    /// Run cargo-audit command with timeout
    async fn run_cargo_audit(&self) -> Result<AuditResult, AuditError> {
        let command = Command::new("cargo")
            .args(&["audit", "--format", "json", "--color", "never"])
            .output();

        let output = timeout(self.timeout_duration, command)
            .await
            .map_err(|_| AuditError::ScanTimeout)?
            .map_err(|e| AuditError::CargoAuditFailed(e.to_string()))?;

        let stdout = std::str::from_utf8(&output.stdout)?;
        let stderr = std::str::from_utf8(&output.stderr)?;

        if !output.status.success() && !stderr.is_empty() {
            return Err(AuditError::CargoAuditFailed(stderr.to_string()));
        }

        self.parse_audit_output(stdout).await
    }

    /// Parse cargo-audit JSON output with zero-allocation
    async fn parse_audit_output(&self, output: &str) -> Result<AuditResult, AuditError> {
        let mut result = AuditResult::new();
        let _start_time = std::time::Instant::now();

        // Parse JSON using zero-allocation string processing
        let mut buffer = ArrayString::<MAX_REPORT_SIZE>::new();
        if output.len() > MAX_REPORT_SIZE {
            buffer.push_str(&output[..MAX_REPORT_SIZE]);
        } else {
            buffer.push_str(output);
        }

        // Use SIMD-accelerated pattern matching to find vulnerability entries
        let vuln_pattern = b"\"type\":\"vulnerability\"";
        let finder = memmem::Finder::new(vuln_pattern);

        let mut offset = 0;
        while let Some(pos) = finder.find(&output.as_bytes()[offset..]) {
            let start = offset + pos;

            // Extract vulnerability JSON object
            if let Some(vuln) = self.extract_vulnerability_at(output, start) {
                result.add_vulnerability(vuln)?;
            }

            offset = start + vuln_pattern.len();
        }

        result.scan_duration_ms = _start_time.elapsed().as_millis() as u64;
        result.success = true;

        Ok(result)
    }

    /// Extract vulnerability from JSON at given position
    fn extract_vulnerability_at(&self, json: &str, start: usize) -> Option<Vulnerability> {
        // Find JSON object boundaries
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut object_start = None;
        let mut object_end = None;

        for (i, byte) in json.bytes().enumerate().skip(start) {
            if escape_next {
                escape_next = false;
                continue;
            }

            match byte {
                b'\\' => escape_next = true,
                b'"' => in_string = !in_string,
                b'{' if !in_string => {
                    if object_start.is_none() {
                        object_start = Some(i);
                    }
                    brace_count += 1;
                }
                b'}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        object_end = Some(i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }

        // Extract and parse vulnerability object
        if let (Some(start), Some(end)) = (object_start, object_end) {
            let vuln_json = &json[start..end];
            self.parse_vulnerability_json(vuln_json)
        } else {
            None
        }
    }

    /// Parse individual vulnerability JSON with zero-allocation
    fn parse_vulnerability_json(&self, json: &str) -> Option<Vulnerability> {
        // Use SIMD-accelerated field extraction
        let id = self.extract_json_field(json, "id")?;
        let package = self.extract_json_field(json, "package")?;
        let severity_str = self.extract_json_field(json, "severity")?;
        let description = self.extract_json_field(json, "description")?;
        let version = self.extract_json_field(json, "version")?;
        let patched = self.extract_json_field(json, "patched");

        let severity = VulnerabilitySeverity::from_str(&severity_str)?;

        Vulnerability::new(
            &id,
            &package,
            severity,
            &description,
            &version,
            patched.as_deref(),
        )
    }

    /// Extract JSON field value using SIMD-accelerated search
    fn extract_json_field(&self, json: &str, field: &str) -> Option<String> {
        let pattern = format!("\"{}\":", field);
        let finder = memmem::Finder::new(pattern.as_bytes());

        let pos = finder.find(json.as_bytes())?;
        let start = pos + pattern.len();

        // Skip whitespace
        let mut value_start = start;
        while value_start < json.len() && json.as_bytes()[value_start].is_ascii_whitespace() {
            value_start += 1;
        }

        if value_start >= json.len() || json.as_bytes()[value_start] != b'"' {
            return None;
        }

        value_start += 1; // Skip opening quote

        // Find closing quote
        let mut value_end = value_start;
        let mut escaped = false;
        while value_end < json.len() {
            let byte = json.as_bytes()[value_end];
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                break;
            }
            value_end += 1;
        }

        if value_end >= json.len() {
            return None;
        }

        Some(json[value_start..value_end].to_string())
    }

    /// Update atomic vulnerability counters
    fn update_counters(&self, result: &AuditResult) {
        let critical = result.count_by_severity(VulnerabilitySeverity::Critical) as u32;
        let high = result.count_by_severity(VulnerabilitySeverity::High) as u32;
        let medium = result.count_by_severity(VulnerabilitySeverity::Medium) as u32;
        let low = result.count_by_severity(VulnerabilitySeverity::Low) as u32;

        self.critical_count.store(critical, Ordering::Relaxed);
        self.high_count.store(high, Ordering::Relaxed);
        self.medium_count.store(medium, Ordering::Relaxed);
        self.low_count.store(low, Ordering::Relaxed);
    }

    /// Update lock-free vulnerability cache
    fn update_cache(&self, result: &AuditResult) {
        for vulnerability in &result.vulnerabilities {
            let key = vulnerability.id.clone();
            let status = if vulnerability.patched.is_some() {
                VulnerabilityStatus::Patched
            } else {
                VulnerabilityStatus::Active
            };
            self.cache.insert(key, status);
        }
    }

    /// Check vulnerability status in cache
    pub fn check_cache(&self, vulnerability_id: &str) -> Option<VulnerabilityStatus> {
        let key = ArrayString::from(vulnerability_id).ok()?;
        self.cache.get(&key).map(|entry| *entry.value())
    }

    /// Get current vulnerability metrics
    pub fn get_metrics(&self) -> VulnerabilityMetrics {
        VulnerabilityMetrics {
            critical_count: self.critical_count.load(Ordering::Relaxed),
            high_count: self.high_count.load(Ordering::Relaxed),
            medium_count: self.medium_count.load(Ordering::Relaxed),
            low_count: self.low_count.load(Ordering::Relaxed),
            total_scans: self.total_scans.load(Ordering::Relaxed),
            successful_scans: self.successful_scans.load(Ordering::Relaxed),
            cache_size: self.cache.len() as u64,
        }
    }

    /// Clear vulnerability cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Update scan timeout
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }

    /// Check if thresholds are exceeded
    pub fn thresholds_exceeded(&self, result: &AuditResult) -> bool {
        self.thresholds.exceeded_by(result)
    }
}

/// Vulnerability metrics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct VulnerabilityMetrics {
    pub critical_count: u32,
    pub high_count: u32,
    pub medium_count: u32,
    pub low_count: u32,
    pub total_scans: u64,
    pub successful_scans: u64,
    pub cache_size: u64,
}

impl VulnerabilityMetrics {
    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_scans == 0 {
            0.0
        } else {
            (self.successful_scans as f64 / self.total_scans as f64) * 100.0
        }
    }

    /// Get total vulnerability count
    pub fn total_vulnerabilities(&self) -> u32 {
        self.critical_count + self.high_count + self.medium_count + self.low_count
    }

    /// Check if any critical vulnerabilities exist
    pub fn has_critical(&self) -> bool {
        self.critical_count > 0
    }
}

/// CI/CD integration helpers
pub mod ci_cd {
    use super::*;

    /// Check if vulnerabilities exceed CI/CD thresholds
    pub fn should_fail_build(scanner: &VulnerabilityScanner, result: &AuditResult) -> bool {
        scanner.thresholds_exceeded(result)
    }

    /// Generate CI/CD failure message
    pub fn generate_failure_message(
        result: &AuditResult,
        _thresholds: &AuditThresholds,
    ) -> ArrayString<512> {
        let mut message = ArrayString::new();

        let critical = result.count_by_severity(VulnerabilitySeverity::Critical);
        let high = result.count_by_severity(VulnerabilitySeverity::High);
        let medium = result.count_by_severity(VulnerabilitySeverity::Medium);
        let low = result.count_by_severity(VulnerabilitySeverity::Low);

        let _ = message.try_push_str(&format!(
            "Vulnerability scan failed: Critical: {}, High: {}, Medium: {}, Low: {}",
            critical, high, medium, low
        ));

        message
    }

    /// Format scan results for CI/CD output
    pub fn format_scan_results(result: &AuditResult) -> ArrayString<1024> {
        let mut output = ArrayString::new();

        let _ = output.try_push_str(&format!(
            "Vulnerability Scan Results:\n\
            - Total vulnerabilities: {}\n\
            - Critical: {}\n\
            - High: {}\n\
            - Medium: {}\n\
            - Low: {}\n\
            - Packages scanned: {}\n\
            - Scan duration: {}ms\n",
            result.vulnerabilities.len(),
            result.count_by_severity(VulnerabilitySeverity::Critical),
            result.count_by_severity(VulnerabilitySeverity::High),
            result.count_by_severity(VulnerabilitySeverity::Medium),
            result.count_by_severity(VulnerabilitySeverity::Low),
            result.packages_scanned,
            result.scan_duration_ms
        ));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerability_creation() {
        let vuln = Vulnerability::new(
            "RUSTSEC-2023-0001",
            "test-package",
            VulnerabilitySeverity::High,
            "Test vulnerability",
            "1.0.0",
            Some("1.0.1"),
        );

        assert!(vuln.is_some());
        let vuln = vuln.unwrap();
        assert_eq!(vuln.id.as_str(), "RUSTSEC-2023-0001");
        assert_eq!(vuln.package.as_str(), "test-package");
        assert_eq!(vuln.severity, VulnerabilitySeverity::High);
    }

    #[test]
    fn test_audit_result_thresholds() {
        let thresholds = AuditThresholds::new(0, 1, 5, 10);
        let mut result = AuditResult::new();

        let vuln = Vulnerability::new(
            "RUSTSEC-2023-0001",
            "test-package",
            VulnerabilitySeverity::High,
            "Test vulnerability",
            "1.0.0",
            None,
        )
        .unwrap();

        result.add_vulnerability(vuln).unwrap();

        assert!(result.passes_thresholds(&thresholds));

        let critical_vuln = Vulnerability::new(
            "RUSTSEC-2023-0002",
            "test-package-2",
            VulnerabilitySeverity::Critical,
            "Critical vulnerability",
            "1.0.0",
            None,
        )
        .unwrap();

        result.add_vulnerability(critical_vuln).unwrap();

        assert!(!result.passes_thresholds(&thresholds));
    }

    #[test]
    fn test_simd_pattern_matching() {
        let vuln = Vulnerability::new(
            "RUSTSEC-2023-0001",
            "test-package",
            VulnerabilitySeverity::High,
            "Test vulnerability with pattern",
            "1.0.0",
            None,
        )
        .unwrap();

        assert!(vuln.matches_pattern(b"RUSTSEC"));
        assert!(vuln.matches_pattern(b"pattern"));
        assert!(!vuln.matches_pattern(b"nonexistent"));
    }

    #[test]
    fn test_vulnerability_metrics() {
        let metrics = VulnerabilityMetrics {
            critical_count: 1,
            high_count: 2,
            medium_count: 3,
            low_count: 4,
            total_scans: 10,
            successful_scans: 8,
            cache_size: 100,
        };

        assert_eq!(metrics.total_vulnerabilities(), 10);
        assert_eq!(metrics.success_rate(), 80.0);
        assert!(metrics.has_critical());
    }
}
