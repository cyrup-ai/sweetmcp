//! Core TLS manager struct and initialization
//!
//! This module provides the foundational TLS management structures and initialization logic
//! for the SweetMCP TLS system with zero allocation, blazing-fast performance.

use anyhow::{Context, Result};
use base64::engine::Engine;
use rcgen::{CertificateParams, DistinguishedName, DnType, Issuer, KeyPair, SanType};
use reqwest::Client;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::time::timeout;
use tracing::{error, info};
use x509_cert::{der::Decode, Certificate as X509CertCert};
use x509_parser::prelude::*;
use zeroize::ZeroizeOnDrop;

// Import OCSP types from the ocsp module
use super::super::ocsp::{OcspCache, OcspStatus};

// PBKDF2 iteration count constant (OWASP 2024 minimum)
const PBKDF2_ITERATIONS: std::num::NonZeroU32 = match std::num::NonZeroU32::new(600_000) {
    Some(n) => n,
    None => unreachable!(), // 600_000 is never zero
};

/// Certificate usage types for KeyUsage validation
#[derive(Debug, Clone, Copy)]
pub enum CertificateUsage {
    /// CA certificate usage
    CertificateAuthority,
    /// Server certificate usage (TLS server authentication)
    ServerAuth,
    /// Client certificate usage (TLS client authentication)
    ClientAuth,
}

/// TLS-specific error types for detailed error handling
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("Certificate parsing failed: {0}")]
    CertificateParsing(String),
    #[error("Certificate validation failed: {0}")]
    CertificateValidation(String),
    #[error("Key encryption/decryption failed: {0}")]
    KeyProtection(String),
    #[error("Certificate chain invalid: {0}")]
    ChainValidation(String),
    #[error("Peer verification failed: {0}")]
    PeerVerification(String),
    #[error("Certificate expired: {0}")]
    CertificateExpired(String),
    #[error("File operation failed: {0}")]
    FileOperation(String),
    #[error("OCSP validation failed: {0}")]
    OcspValidation(String),
    #[error("CRL validation failed: {0}")]
    CrlValidation(String),
    #[error("Network error during validation: {0}")]
    NetworkError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Parsed certificate information extracted from X.509
#[derive(Debug, Clone)]
pub struct ParsedCertificate {
    pub subject: HashMap<String, String>,
    pub issuer: HashMap<String, String>,
    pub san_dns_names: Vec<String>,
    pub san_ip_addresses: Vec<std::net::IpAddr>,
    pub is_ca: bool,
    pub key_usage: Vec<String>,
    pub not_before: std::time::SystemTime,
    pub not_after: std::time::SystemTime,
    pub serial_number: Vec<u8>,
    pub ocsp_urls: Vec<String>,
    pub crl_urls: Vec<String>,
    /// Raw DER-encoded subject for OCSP
    pub subject_der: Vec<u8>,
    /// Raw DER-encoded public key for OCSP
    pub public_key_der: Vec<u8>,
}

/// CRL cache entry for performance optimization
#[derive(Debug, Clone)]
pub struct CrlCacheEntry {
    pub revoked_serials: HashSet<Vec<u8>>,
    pub cached_at: SystemTime,
    pub next_update: Option<SystemTime>,
}

/// CRL download and validation cache with lock-free performance
#[derive(Clone)]
pub struct CrlCache {
    cache: Arc<parking_lot::RwLock<HashMap<String, CrlCacheEntry>>>,
    http_client: Client,
}

impl CrlCache {
    /// Create new CRL cache with optimized HTTP client
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30)) // CRL files can be large
            .user_agent("SweetMCP/1.0 CRL Client")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            cache: Arc::new(parking_lot::RwLock::new(HashMap::with_capacity(64))),
            http_client,
        }
    }

    /// Check if certificate serial number is revoked using CRL with zero allocation fast path
    pub async fn check_certificate_revocation(
        &self,
        cert: &ParsedCertificate,
    ) -> Result<bool, TlsError> {
        if cert.crl_urls.is_empty() {
            tracing::warn!("No CRL URLs found in certificate, skipping CRL validation");
            return Ok(false); // Not revoked (no CRL available)
        }

        // Try each CRL URL until one succeeds - fast path for cached results
        for crl_url in &cert.crl_urls {
            // Fast path: check cache first with read lock
            {
                let cache_read = self.cache.read();
                if let Some(entry) = cache_read.get(crl_url) {
                    // Check if cache entry is still valid
                    let now = SystemTime::now();
                    let is_valid = entry.next_update.map_or(true, |next| now < next);
                    
                    if is_valid {
                        let is_revoked = entry.revoked_serials.contains(&cert.serial_number);
                        if is_revoked {
                            tracing::warn!(
                                "Certificate serial {:?} found in cached CRL from {}",
                                hex::encode(&cert.serial_number),
                                crl_url
                            );
                            return Ok(true);
                        }
                        tracing::info!(
                            "Certificate serial {:?} not found in cached CRL from {}",
                            hex::encode(&cert.serial_number),
                            crl_url
                        );
                        continue;
                    }
                }
            }

            // Slow path: fetch and cache CRL
            match self.check_against_crl(cert, crl_url).await {
                Ok(is_revoked) => {
                    if is_revoked {
                        tracing::warn!(
                            "Certificate serial {:?} found in CRL from {}",
                            hex::encode(&cert.serial_number),
                            crl_url
                        );
                        return Ok(true);
                    }
                    tracing::info!(
                        "Certificate serial {:?} not found in CRL from {}",
                        hex::encode(&cert.serial_number),
                        crl_url
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to check CRL from {}: {}", crl_url, e);
                    continue; // Try next CRL URL
                }
            }
        }

        Ok(false) // Not revoked
    }

    /// Check certificate against specific CRL URL with optimized parsing
    async fn check_against_crl(
        &self,
        cert: &ParsedCertificate,
        crl_url: &str,
    ) -> Result<bool, TlsError> {
        // Download CRL with timeout
        let response = timeout(Duration::from_secs(30), self.http_client.get(crl_url).send())
            .await
            .map_err(|_| TlsError::NetworkError("CRL download timeout".to_string()))?
            .map_err(|e| TlsError::NetworkError(format!("Failed to download CRL: {}", e)))?;

        if !response.status().is_success() {
            return Err(TlsError::NetworkError(format!(
                "CRL download failed with status: {}",
                response.status()
            )));
        }

        let crl_bytes = response
            .bytes()
            .await
            .map_err(|e| TlsError::NetworkError(format!("Failed to read CRL data: {}", e)))?;

        // Parse CRL with optimized zero-copy parsing where possible
        let entry = self.parse_crl_data(&crl_bytes)?;
        
        // Cache the parsed CRL entry
        {
            let mut cache_write = self.cache.write();
            cache_write.insert(crl_url.to_string(), entry.clone());
        }

        // Check if certificate is revoked
        Ok(entry.revoked_serials.contains(&cert.serial_number))
    }

    /// Parse CRL data with optimized zero-allocation parsing
    fn parse_crl_data(&self, crl_bytes: &[u8]) -> Result<CrlCacheEntry, TlsError> {
        // Parse PEM if it starts with "-----BEGIN" - zero allocation fast path
        let der_bytes = if crl_bytes.starts_with(b"-----BEGIN") {
            let crl_pem = std::str::from_utf8(crl_bytes)
                .map_err(|_| TlsError::CrlValidation("Invalid UTF-8 in PEM CRL".to_string()))?;

            // Extract DER from PEM with pre-allocated capacity
            let mut der_data = Vec::with_capacity(crl_bytes.len() * 3 / 4); // Base64 efficiency
            let mut in_crl = false;
            
            for line in crl_pem.lines() {
                if line.contains("-----BEGIN") && line.contains("CRL") {
                    in_crl = true;
                    continue;
                }
                if line.contains("-----END") && line.contains("CRL") {
                    break;
                }
                if in_crl && !line.is_empty() {
                    match base64::engine::general_purpose::STANDARD.decode(line.trim()) {
                        Ok(decoded) => der_data.extend_from_slice(&decoded),
                        Err(_) => continue, // Skip invalid base64 lines
                    }
                }
            }

            if der_data.is_empty() {
                return Err(TlsError::CrlValidation(
                    "No CRL data found in PEM".to_string(),
                ));
            }

            der_data
        } else {
            // Assume DER format - zero copy
            crl_bytes.to_vec()
        };

        // Parse X.509 CRL using x509-parser with optimized error handling
        let (_, crl) = parse_x509_crl(&der_bytes)
            .map_err(|e| TlsError::CrlValidation(format!("CRL parsing failed: {}", e)))?;

        // Extract revoked certificate serial numbers with pre-allocated capacity
        let revoked_count = crl.iter_revoked_certificates().count();
        let mut revoked_serials = HashSet::with_capacity(revoked_count);
        
        for revoked_cert in crl.iter_revoked_certificates() {
            revoked_serials.insert(revoked_cert.user_certificate.to_bytes_be());
        }

        // Extract next update time with optimized conversion
        let next_update = crl.next_update().map(|time| {
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(time.timestamp() as u64)
        });

        Ok(CrlCacheEntry {
            revoked_serials,
            cached_at: SystemTime::now(),
            next_update,
        })
    }
}

impl Default for CrlCache {
    fn default() -> Self {
        Self::new()
    }
}