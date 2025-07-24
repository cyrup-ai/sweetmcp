//! System resource estimation and compatibility checking for quantum MCTS
//!
//! This module provides blazing-fast system analysis with zero-allocation
//! resource estimation for optimal quantum MCTS configuration.

use crate::cognitive::quantum_mcts::config::core::QuantumMCTSConfig;
use std::sync::OnceLock;

/// System resource information cached for performance
#[derive(Debug, Clone)]
pub struct SystemResources {
    pub cpu_count: usize,
    pub estimated_memory: u64,
    pub cache_line_size: usize,
    pub page_size: usize,
    pub has_simd: bool,
    pub arch_type: ArchType,
}

/// CPU architecture type for optimization hints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchType {
    X86_64,
    Aarch64,
    Unknown,
}

/// System resource analyzer with caching for performance
pub struct SystemAnalyzer {
    resources: &'static SystemResources,
}

static SYSTEM_RESOURCES: OnceLock<SystemResources> = OnceLock::new();

impl SystemAnalyzer {
    /// Create new system analyzer with cached resource information
    pub fn new() -> Self {
        let resources = SYSTEM_RESOURCES.get_or_init(|| Self::detect_system_resources());
        Self { resources }
    }

    /// Get cached system resources
    pub fn resources(&self) -> &SystemResources {
        self.resources
    }

    /// Detect system resources once and cache the results
    fn detect_system_resources() -> SystemResources {
        let cpu_count = num_cpus::get();
        let estimated_memory = Self::estimate_system_memory(cpu_count);
        let arch_type = Self::detect_architecture();
        
        SystemResources {
            cpu_count,
            estimated_memory,
            cache_line_size: Self::detect_cache_line_size(),
            page_size: Self::detect_page_size(),
            has_simd: Self::detect_simd_support(),
            arch_type,
        }
    }

    /// Estimate system memory based on CPU count and common configurations
    fn estimate_system_memory(cpu_count: usize) -> u64 {
        // Conservative estimates based on typical system configurations
        match cpu_count {
            1 => 1_000_000_000,      // 1GB for single-core systems
            2 => 2_000_000_000,      // 2GB for dual-core systems
            3..=4 => 4_000_000_000,  // 4GB for quad-core systems
            5..=8 => 8_000_000_000,  // 8GB for 8-core systems
            9..=16 => 16_000_000_000, // 16GB for 16-core systems
            17..=32 => 32_000_000_000, // 32GB for 32-core systems
            _ => 64_000_000_000,     // 64GB+ for high-end systems
        }
    }

    /// Detect CPU architecture for optimization hints
    fn detect_architecture() -> ArchType {
        #[cfg(target_arch = "x86_64")]
        return ArchType::X86_64;
        
        #[cfg(target_arch = "aarch64")]
        return ArchType::Aarch64;
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        return ArchType::Unknown;
    }

    /// Detect cache line size for memory alignment optimization
    fn detect_cache_line_size() -> usize {
        // Most modern systems use 64-byte cache lines
        // This could be enhanced with runtime detection
        64
    }

    /// Detect page size for memory management optimization
    fn detect_page_size() -> usize {
        // Most systems use 4KB pages, but some use 16KB or 64KB
        // This could be enhanced with runtime detection
        4096
    }

    /// Detect SIMD support for vectorization optimization
    fn detect_simd_support() -> bool {
        // Conservative approach - assume SIMD is available on modern architectures
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        return true;
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        return false;
    }

    /// Generate system-optimized configuration
    pub fn system_optimized_config(&self) -> QuantumMCTSConfig {
        let resources = self.resources;
        
        // Scale parallelism based on CPU cores with architecture-specific tuning
        let max_parallel = match resources.arch_type {
            ArchType::X86_64 => {
                if resources.cpu_count <= 4 {
                    resources.cpu_count
                } else if resources.cpu_count <= 8 {
                    resources.cpu_count - 1 // Leave one core for OS
                } else {
                    (resources.cpu_count * 3) / 4 // Use 75% of cores
                }
            },
            ArchType::Aarch64 => {
                // ARM processors often have efficiency cores
                if resources.cpu_count <= 8 {
                    resources.cpu_count
                } else {
                    resources.cpu_count / 2 // Conservative for mixed core designs
                }
            },
            ArchType::Unknown => {
                // Very conservative for unknown architectures
                (resources.cpu_count / 2).max(1)
            },
        };

        // Scale tree size based on available memory
        let max_tree_size = self.calculate_optimal_tree_size();
        
        // Adjust timeout based on CPU performance
        let timeout_ms = if resources.cpu_count >= 8 { 
            60_000 
        } else if resources.cpu_count >= 4 { 
            45_000 
        } else { 
            30_000 
        };

        // Enable error correction only on systems that can handle the overhead
        let enable_error_correction = resources.cpu_count >= 4 && 
                                     resources.estimated_memory >= 4_000_000_000;

        // Adjust precision based on architecture capabilities
        let measurement_precision = match resources.arch_type {
            ArchType::X86_64 => 1e-10,  // High precision on x86_64
            ArchType::Aarch64 => 1e-8,  // Slightly lower precision on ARM
            ArchType::Unknown => 1e-6,  // Conservative precision
        };

        QuantumMCTSConfig {
            max_quantum_parallel: max_parallel,
            max_tree_size,
            simulation_timeout_ms: timeout_ms,
            enable_error_correction,
            measurement_precision,
            ..QuantumMCTSConfig::default()
        }
    }

    /// Calculate optimal tree size based on available memory
    fn calculate_optimal_tree_size(&self) -> usize {
        let available_memory = self.resources.estimated_memory;
        let node_size = 1024; // Approximate size per quantum node
        let safety_factor = 4; // Use only 1/4 of memory for tree
        
        let max_nodes = available_memory / (node_size * safety_factor);
        
        // Clamp to reasonable bounds
        max_nodes.min(1_000_000).max(1_000) as usize
    }

    /// Check if configuration is compatible with current system
    pub fn is_compatible(&self, config: &QuantumMCTSConfig) -> Result<(), String> {
        let resources = self.resources;
        
        // Check CPU oversubscription
        if config.max_quantum_parallel > resources.cpu_count * 4 {
            return Err(format!(
                "Parallelism {} exceeds reasonable limit for {} CPU cores",
                config.max_quantum_parallel, resources.cpu_count
            ));
        }

        // Check memory requirements
        let estimated_usage = config.estimate_memory_usage();
        let memory_threshold = resources.estimated_memory / 2; // Use max 50% of memory
        
        if estimated_usage > memory_threshold {
            return Err(format!(
                "Estimated memory usage {}MB exceeds threshold {}MB",
                estimated_usage / 1_000_000,
                memory_threshold / 1_000_000
            ));
        }

        // Check for unsupported features
        if config.enable_error_correction && resources.cpu_count < 2 {
            return Err(
                "Error correction requires at least 2 CPU cores".to_string()
            );
        }

        // Architecture-specific checks
        match resources.arch_type {
            ArchType::X86_64 => {
                // x86_64 generally supports all features well
            },
            ArchType::Aarch64 => {
                // ARM may have different performance characteristics
                if config.max_quantum_parallel > resources.cpu_count {
                    return Err(
                        "ARM systems may perform better with parallelism <= CPU count".to_string()
                    );
                }
            },
            ArchType::Unknown => {
                // Very conservative checks for unknown architectures
                if config.max_quantum_parallel > resources.cpu_count / 2 {
                    return Err(
                        "Unknown architecture: recommend conservative parallelism".to_string()
                    );
                }
            },
        }

        Ok(())
    }

    /// Get performance recommendations for current system
    pub fn performance_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let resources = self.resources;

        // CPU-based recommendations
        if resources.cpu_count == 1 {
            recommendations.push(
                "Single-core system: disable error correction for better performance".to_string()
            );
        } else if resources.cpu_count >= 16 {
            recommendations.push(
                "High-core system: consider using accuracy_optimized config".to_string()
            );
        }

        // Memory-based recommendations
        if resources.estimated_memory < 2_000_000_000 {
            recommendations.push(
                "Limited memory: use performance_optimized config with smaller tree size".to_string()
            );
        } else if resources.estimated_memory >= 16_000_000_000 {
            recommendations.push(
                "High memory system: can use larger tree sizes for better accuracy".to_string()
            );
        }

        // Architecture-specific recommendations
        match resources.arch_type {
            ArchType::X86_64 => {
                if resources.has_simd {
                    recommendations.push(
                        "x86_64 with SIMD: optimized for vectorized quantum calculations".to_string()
                    );
                }
            },
            ArchType::Aarch64 => {
                recommendations.push(
                    "ARM architecture: consider conservative parallelism settings".to_string()
                );
            },
            ArchType::Unknown => {
                recommendations.push(
                    "Unknown architecture: use default config with validation".to_string()
                );
            },
        }

        // Cache optimization recommendations
        if resources.cache_line_size != 64 {
            recommendations.push(format!(
                "Non-standard cache line size ({}): may affect performance",
                resources.cache_line_size
            ));
        }

        recommendations
    }

    /// Get system capability summary
    pub fn capability_summary(&self) -> SystemCapabilitySummary {
        let resources = self.resources;
        
        let performance_tier = if resources.cpu_count >= 16 && resources.estimated_memory >= 16_000_000_000 {
            PerformanceTier::HighEnd
        } else if resources.cpu_count >= 8 && resources.estimated_memory >= 8_000_000_000 {
            PerformanceTier::MidRange
        } else if resources.cpu_count >= 4 && resources.estimated_memory >= 4_000_000_000 {
            PerformanceTier::Standard
        } else {
            PerformanceTier::LowEnd
        };

        let recommended_config_type = match performance_tier {
            PerformanceTier::HighEnd => ConfigType::AccuracyOptimized,
            PerformanceTier::MidRange => ConfigType::SystemOptimized,
            PerformanceTier::Standard => ConfigType::PerformanceOptimized,
            PerformanceTier::LowEnd => ConfigType::Minimal,
        };

        SystemCapabilitySummary {
            performance_tier,
            recommended_config_type,
            cpu_count: resources.cpu_count,
            memory_gb: resources.estimated_memory / 1_000_000_000,
            arch_type: resources.arch_type,
            has_simd: resources.has_simd,
            max_recommended_parallel: self.calculate_max_recommended_parallel(),
            max_recommended_tree_size: self.calculate_optimal_tree_size(),
        }
    }

    /// Calculate maximum recommended parallelism for this system
    fn calculate_max_recommended_parallel(&self) -> usize {
        let cpu_count = self.resources.cpu_count;
        match self.resources.arch_type {
            ArchType::X86_64 => cpu_count * 2,      // x86_64 handles oversubscription well
            ArchType::Aarch64 => cpu_count,         // ARM prefers 1:1 mapping
            ArchType::Unknown => cpu_count / 2,     // Conservative for unknown
        }
    }
}

impl Default for SystemAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// System performance tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceTier {
    LowEnd,
    Standard,
    MidRange,
    HighEnd,
}

/// Recommended configuration type based on system capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    Minimal,
    PerformanceOptimized,
    SystemOptimized,
    AccuracyOptimized,
}

/// Summary of system capabilities and recommendations
#[derive(Debug, Clone)]
pub struct SystemCapabilitySummary {
    pub performance_tier: PerformanceTier,
    pub recommended_config_type: ConfigType,
    pub cpu_count: usize,
    pub memory_gb: u64,
    pub arch_type: ArchType,
    pub has_simd: bool,
    pub max_recommended_parallel: usize,
    pub max_recommended_tree_size: usize,
}

impl SystemCapabilitySummary {
    /// Get human-readable description of system capabilities
    pub fn description(&self) -> String {
        format!(
            "{:?} system: {} CPU cores, {}GB RAM, {:?} architecture{}",
            self.performance_tier,
            self.cpu_count,
            self.memory_gb,
            self.arch_type,
            if self.has_simd { " with SIMD" } else { "" }
        )
    }

    /// Get configuration recommendations as text
    pub fn recommendations(&self) -> Vec<String> {
        let mut recs = Vec::new();
        
        recs.push(format!("Recommended config: {:?}", self.recommended_config_type));
        recs.push(format!("Max parallelism: {}", self.max_recommended_parallel));
        recs.push(format!("Max tree size: {}", self.max_recommended_tree_size));
        
        match self.performance_tier {
            PerformanceTier::LowEnd => {
                recs.push("Consider disabling error correction for better performance".to_string());
                recs.push("Use shorter timeouts to prevent resource exhaustion".to_string());
            },
            PerformanceTier::HighEnd => {
                recs.push("System can handle accuracy-optimized configurations".to_string());
                recs.push("Consider larger tree sizes for better search quality".to_string());
            },
            _ => {
                recs.push("System supports balanced performance/accuracy configurations".to_string());
            },
        }
        
        recs
    }
}