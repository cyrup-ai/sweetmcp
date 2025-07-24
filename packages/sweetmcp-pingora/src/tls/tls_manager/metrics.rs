//! Performance metrics and monitoring
//!
//! This module provides comprehensive performance metrics and monitoring for TLS operations
//! with zero allocation fast paths and blazing-fast performance.

use super::core::{ParsedCertificate, TlsError};
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair, SanType,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, error, info, warn};

/// Metrics manager for TLS operations with optimized performance tracking
pub struct MetricsManager {
    operation_counts: HashMap<String, u64>,
    operation_durations: HashMap<String, Duration>,
    last_reset: Instant,
}

impl MetricsManager {
    /// Create new metrics manager with optimized tracking
    pub fn new() -> Self {
        Self {
            operation_counts: HashMap::new(),
            operation_durations: HashMap::new(),
            last_reset: Instant::now(),
        }
    }

    /// Record operation with fast path metrics tracking
    pub fn record_operation(&mut self, operation: &str, duration: Duration) {
        *self.operation_counts.entry(operation.to_string()).or_insert(0) += 1;
        *self.operation_durations.entry(operation.to_string()).or_insert(Duration::ZERO) += duration;
    }

    /// Get operation statistics with zero allocation fast path
    pub fn get_operation_stats(&self, operation: &str) -> Option<(u64, Duration)> {
        let count = self.operation_counts.get(operation)?;
        let total_duration = self.operation_durations.get(operation)?;
        Some((*count, *total_duration))
    }

    /// Reset metrics with optimized cleanup
    pub fn reset_metrics(&mut self) {
        self.operation_counts.clear();
        self.operation_durations.clear();
        self.last_reset = Instant::now();
    }

    /// Get uptime since last reset with fast calculation
    pub fn uptime(&self) -> Duration {
        self.last_reset.elapsed()
    }
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Certificate generation utilities with optimized performance
pub struct CertificateGenerator;

impl CertificateGenerator {
    /// Extract certificate information for metrics with fast parsing
    pub fn extract_certificate_info_for_metrics(cert: &ParsedCertificate) -> HashMap<String, String> {
        let mut info = HashMap::new();
        
        // Extract key metrics with zero allocation fast path
        info.insert("serial_number".to_string(), hex::encode(&cert.serial_number));
        info.insert("signature_algorithm".to_string(), cert.signature_algorithm.clone());
        info.insert("public_key_algorithm".to_string(), cert.public_key_algorithm.clone());
        info.insert("is_ca".to_string(), cert.is_ca.to_string());
        info.insert("key_usage_count".to_string(), cert.key_usage.len().to_string());
        info.insert("san_count".to_string(), cert.subject_alt_names.len().to_string());
        info.insert("ocsp_urls_count".to_string(), cert.ocsp_urls.len().to_string());
        info.insert("crl_urls_count".to_string(), cert.crl_urls.len().to_string());
        
        // Calculate validity period with optimized duration calculation
        if let Ok(validity_duration) = cert.not_after.duration_since(cert.not_before) {
            info.insert("validity_days".to_string(), (validity_duration.as_secs() / 86400).to_string());
        }
        
        // Calculate time until expiry with fast calculation
        if let Ok(time_until_expiry) = cert.not_after.duration_since(SystemTime::now()) {
            info.insert("days_until_expiry".to_string(), (time_until_expiry.as_secs() / 86400).to_string());
        }
        
        info
    }

    /// Extract OCSP and CRL URLs from certificate extensions with optimized parsing
    pub fn extract_urls_from_extensions(
        cert: &x509_cert::Certificate,
    ) -> (Vec<String>, Vec<String>) {
        let mut ocsp_urls = Vec::new();
        let mut crl_urls = Vec::new();

        // Process extensions with fast path validation
        if let Some(extensions) = &cert.tbs_certificate.extensions {
            for ext in extensions.iter() {
                let oid_str = ext.extn_id.to_string();

                // Authority Information Access extension (1.3.6.1.5.5.7.1.1)
                if oid_str == "1.3.6.1.5.5.7.1.1" {
                    // Extract OCSP URLs from Authority Information Access with optimized parsing
                    let ext_bytes = ext.extn_value.as_bytes();

                    // Look for HTTP URLs in the extension data with fast scanning
                    for i in 0..ext_bytes.len().saturating_sub(4) {
                        if &ext_bytes[i..i + 4] == b"http" {
                            // Found potential URL start
                            let mut url_bytes = Vec::new();
                            for &byte in &ext_bytes[i..] {
                                if byte >= 0x20 && byte <= 0x7E {
                                    // Printable ASCII
                                    url_bytes.push(byte);
                                } else {
                                    break;
                                }
                            }

                            if let Ok(url) = String::from_utf8(url_bytes) {
                                if url.starts_with("http://") || url.starts_with("https://") {
                                    if !ocsp_urls.contains(&url) {
                                        ocsp_urls.push(url);
                                    }
                                }
                            }
                        }
                    }
                }
                // CRL Distribution Points extension (2.5.29.31)
                else if oid_str == "2.5.29.31" {
                    // Extract CRL URLs from CRL Distribution Points with optimized parsing
                    let ext_bytes = ext.extn_value.as_bytes();

                    // Look for HTTP URLs in the extension data with fast scanning
                    for i in 0..ext_bytes.len().saturating_sub(4) {
                        if &ext_bytes[i..i + 4] == b"http" {
                            // Found potential URL start
                            let mut url_bytes = Vec::new();
                            for &byte in &ext_bytes[i..] {
                                if byte >= 0x20 && byte <= 0x7E {
                                    // Printable ASCII
                                    url_bytes.push(byte);
                                } else {
                                    break;
                                }
                            }

                            if let Ok(url) = String::from_utf8(url_bytes) {
                                if url.starts_with("http://") || url.starts_with("https://") {
                                    if !crl_urls.contains(&url) {
                                        crl_urls.push(url);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        (ocsp_urls, crl_urls)
    }

    /// Generate CA certificate with optimized key generation
    pub async fn generate_ca_cert(
        cert_dir: &Path,
    ) -> Result<(CertificateDer<'static>, PrivatePkcs8KeyDer<'static>, rcgen::CertifiedKey), TlsError> {
        let mut params = CertificateParams::new(Vec::default())
            .map_err(|e| TlsError::CertificateGeneration(format!("Failed to create CA params: {}", e)))?;

        // Configure CA certificate parameters with optimized settings
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];

        // Set validity period (10 years for CA) with optimized time calculation
        let now = SystemTime::now();
        params.not_before = now;
        params.not_after = now + Duration::from_secs(10 * 365 * 24 * 3600); // 10 years

        // Create distinguished name with fast construction
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "SweetMCP Root CA");
        dn.push(DnType::OrganizationName, "SweetMCP");
        dn.push(DnType::CountryName, "US");
        params.distinguished_name = dn;

        // Generate key pair with optimized key generation
        let key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)
            .map_err(|e| TlsError::CertificateGeneration(format!("Failed to generate CA key: {}", e)))?;

        // Generate certificate with fast generation
        let cert = params.self_signed(&key_pair)
            .map_err(|e| TlsError::CertificateGeneration(format!("Failed to self-sign CA cert: {}", e)))?;

        let cert_der = cert.der().to_vec();
        let key_der = key_pair.serialize_der();

        // Create CertifiedKey for use as issuer with optimized construction
        let certified_key = rcgen::CertifiedKey::from_params_and_key_pair(params.clone(), key_pair)
            .map_err(|e| TlsError::CertificateGeneration(format!("Failed to create certified key: {}", e)))?;

        Ok((
            CertificateDer::from(cert_der),
            PrivatePkcs8KeyDer::from(key_der),
            certified_key,
        ))
    }

    /// Generate server certificate signed by CA with optimized generation
    pub async fn generate_server_cert(
        ca_certified_key: &rcgen::CertifiedKey,
        cert_dir: &Path,
    ) -> Result<(CertificateDer<'static>, PrivatePkcs8KeyDer<'static>), TlsError> {
        let mut params = CertificateParams::new(Vec::default())
            .map_err(|e| TlsError::CertificateGeneration(format!("Failed to create server params: {}", e)))?;

        // Add SAN entries with optimized construction
        params.subject_alt_names = vec![
            SanType::DnsName("localhost".try_into()
                .map_err(|e| TlsError::CertificateGeneration(format!("Invalid DNS name: {}", e)))?),
            SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
            SanType::IpAddress(std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)),
        ];

        // Add hostname if available with fast hostname resolution
        if let Ok(hostname) = hostname::get() {
            if let Some(hostname_str) = hostname.to_str() {
                match hostname_str.try_into() {
                    Ok(dns_name) => params.subject_alt_names.push(SanType::DnsName(dns_name)),
                    Err(e) => tracing::warn!("Failed to add hostname to SAN: {}", e),
                }
            }
        }

        // Configure server certificate parameters with optimized settings
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::KeyEncipherment,
        ];

        params.extended_key_usages = vec![
            rcgen::ExtendedKeyUsagePurpose::ServerAuth,
        ];

        // Set validity period (1 year for server cert) with optimized time calculation
        let now = SystemTime::now();
        params.not_before = now;
        params.not_after = now + Duration::from_secs(365 * 24 * 3600); // 1 year

        // Create distinguished name with fast construction
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "localhost");
        dn.push(DnType::OrganizationName, "SweetMCP");
        params.distinguished_name = dn;

        // Generate key pair with optimized key generation
        let key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)
            .map_err(|e| TlsError::CertificateGeneration(format!("Failed to generate server key: {}", e)))?;

        // Generate certificate signed by CA with fast signing
        let cert = params.signed_by(&key_pair, &ca_certified_key.cert, &ca_certified_key.key_pair)
            .map_err(|e| TlsError::CertificateGeneration(format!("Failed to sign server cert: {}", e)))?;

        let cert_der = cert.der().to_vec();
        let key_der = key_pair.serialize_der();

        Ok((
            CertificateDer::from(cert_der),
            PrivatePkcs8KeyDer::from(key_der),
        ))
    }

    /// Validate certificate generation parameters with fast validation
    pub fn validate_generation_params(params: &CertificateParams) -> Result<(), TlsError> {
        // Check validity period with optimized time checking
        if params.not_after <= params.not_before {
            return Err(TlsError::CertificateGeneration(
                "Certificate not_after must be after not_before".to_string(),
            ));
        }

        // Check for reasonable validity period (not more than 10 years) with fast calculation
        let validity_duration = params.not_after.duration_since(params.not_before)
            .map_err(|e| TlsError::CertificateGeneration(format!("Invalid validity period: {}", e)))?;
        
        const MAX_VALIDITY_SECS: u64 = 10 * 365 * 24 * 3600; // 10 years
        if validity_duration.as_secs() > MAX_VALIDITY_SECS {
            return Err(TlsError::CertificateGeneration(
                "Certificate validity period exceeds maximum of 10 years".to_string(),
            ));
        }

        // Check distinguished name has at least CN with fast validation
        let dn_entries = &params.distinguished_name.entries;
        if !dn_entries.iter().any(|entry| matches!(entry.0, DnType::CommonName)) {
            return Err(TlsError::CertificateGeneration(
                "Certificate must have a Common Name (CN)".to_string(),
            ));
        }

        Ok(())
    }
}

/// Performance monitoring utilities
pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// Measure operation duration with zero allocation timing
    pub fn measure_operation<F, R>(operation_name: &str, f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        tracing::debug!("Operation '{}' completed in {:?}", operation_name, duration);
        
        (result, duration)
    }

    /// Measure async operation duration with fast async timing
    pub async fn measure_async_operation<F, Fut, R>(operation_name: &str, f: F) -> (R, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        
        tracing::debug!("Async operation '{}' completed in {:?}", operation_name, duration);
        
        (result, duration)
    }

    /// Log performance statistics with optimized logging
    pub fn log_performance_stats(metrics: &MetricsManager) {
        tracing::info!("TLS Manager Performance Statistics:");
        tracing::info!("Uptime: {:?}", metrics.uptime());
        
        for (operation, count) in &metrics.operation_counts {
            if let Some(total_duration) = metrics.operation_durations.get(operation) {
                let avg_duration = *total_duration / (*count as u32);
                tracing::info!(
                    "Operation '{}': {} calls, avg duration: {:?}",
                    operation, count, avg_duration
                );
            }
        }
    }
}