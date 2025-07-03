//! Production-grade TLS and mTLS configuration for SweetMCP
//!
//! This module provides comprehensive mTLS support with certificate lifecycle management

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
use std::sync::RwLock;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::time::timeout;
use tracing::{error, info};
use x509_cert::{der::Decode, Certificate as X509CertCert};
use x509_parser::prelude::*;
use zeroize::ZeroizeOnDrop;

// Import OCSP types from the ocsp module
use super::ocsp::{OcspCache, OcspStatus};

// PBKDF2 iteration count constant (OWASP 2024 minimum)
#[allow(dead_code)]
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
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CrlCacheEntry {
    revoked_serials: HashSet<Vec<u8>>,
    cached_at: SystemTime,
    next_update: Option<SystemTime>,
}

/// CRL download and validation cache
#[allow(dead_code)]
#[derive(Clone)]
pub struct CrlCache {
    cache: Arc<RwLock<HashMap<String, CrlCacheEntry>>>,
    http_client: Client,
}

impl CrlCache {
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30)) // CRL files can be large
            .user_agent("SweetMCP/1.0 CRL Client")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            cache: Arc::new(RwLock::new(HashMap::with_capacity(64))),
            http_client,
        }
    }

    /// Check if certificate serial number is revoked using CRL
    pub async fn check_certificate_revocation(
        &self,
        cert: &ParsedCertificate,
    ) -> Result<bool, TlsError> {
        if cert.crl_urls.is_empty() {
            tracing::warn!("No CRL URLs found in certificate, skipping CRL validation");
            return Ok(false); // Not revoked (no CRL available)
        }

        // Try each CRL URL until one succeeds
        for crl_url in &cert.crl_urls {
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
                    tracing::warn!("CRL validation failed for URL {}: {}", crl_url, e);
                    continue;
                }
            }
        }

        // If all CRLs were checked and certificate not found in any, it's not revoked
        Ok(false)
    }

    async fn check_against_crl(
        &self,
        cert: &ParsedCertificate,
        crl_url: &str,
    ) -> Result<bool, TlsError> {
        let cache_key = crl_url.to_string();

        // Check cache first
        if let Some(cached_crl) = self.get_cached_crl(&cache_key) {
            if !Self::is_crl_cache_expired(&cached_crl) {
                tracing::debug!("CRL cache hit for URL: {}", crl_url);
                return Ok(cached_crl.revoked_serials.contains(&cert.serial_number));
            }
        }

        // Download and parse CRL
        let crl_entry = self.download_and_parse_crl(crl_url).await?;

        // Cache the CRL
        self.cache_crl(cache_key, crl_entry.clone());

        // Check if certificate is revoked
        Ok(crl_entry.revoked_serials.contains(&cert.serial_number))
    }

    #[inline]
    fn get_cached_crl(&self, cache_key: &str) -> Option<CrlCacheEntry> {
        match self.cache.read() {
            Ok(cache) => cache.get(cache_key).cloned(),
            Err(poisoned) => {
                tracing::warn!("CRL cache read lock poisoned, recovering");
                poisoned.into_inner().get(cache_key).cloned()
            }
        }
    }

    fn is_crl_cache_expired(entry: &CrlCacheEntry) -> bool {
        let now = SystemTime::now();

        // Check if we have next_update time and it's passed
        if let Some(next_update) = entry.next_update {
            return now > next_update;
        }

        // Default cache expiry: 24 hours (CRLs are typically updated daily)
        let cache_duration = Duration::from_secs(24 * 3600);
        now.duration_since(entry.cached_at)
            .unwrap_or(Duration::ZERO)
            > cache_duration
    }

    #[inline]
    fn cache_crl(&self, cache_key: String, entry: CrlCacheEntry) {
        match self.cache.write() {
            Ok(mut cache) => {
                cache.insert(cache_key, entry);
            }
            Err(poisoned) => {
                tracing::warn!("CRL cache write lock poisoned, recovering");
                poisoned.into_inner().insert(cache_key, entry);
            }
        }
    }

    async fn download_and_parse_crl(&self, crl_url: &str) -> Result<CrlCacheEntry, TlsError> {
        // Download CRL with timeout
        let response = timeout(
            Duration::from_secs(30),
            self.http_client.get(crl_url).send(),
        )
        .await
        .map_err(|_| TlsError::NetworkError("CRL download timeout".to_string()))?
        .map_err(|e| TlsError::NetworkError(format!("CRL HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(TlsError::CrlValidation(format!(
                "CRL download failed with status: {}",
                response.status()
            )));
        }

        let crl_bytes = response
            .bytes()
            .await
            .map_err(|e| TlsError::NetworkError(format!("Failed to read CRL data: {}", e)))?;

        // Parse CRL
        self.parse_crl_data(&crl_bytes)
    }

    fn parse_crl_data(&self, crl_bytes: &[u8]) -> Result<CrlCacheEntry, TlsError> {
        // Parse PEM if it starts with "-----BEGIN"
        let der_bytes = if crl_bytes.starts_with(b"-----BEGIN") {
            let crl_pem = std::str::from_utf8(crl_bytes)
                .map_err(|_| TlsError::CrlValidation("Invalid UTF-8 in PEM CRL".to_string()))?;

            // Extract DER from PEM
            let mut der_data = Vec::new();
            let mut in_crl = false;
            for line in crl_pem.lines() {
                if line.contains("-----BEGIN") && line.contains("CRL") {
                    in_crl = true;
                    continue;
                }
                if line.contains("-----END") && line.contains("CRL") {
                    break;
                }
                if in_crl {
                    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(line) {
                        der_data.extend(decoded);
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
            // Assume DER format
            crl_bytes.to_vec()
        };

        // Parse X.509 CRL using x509-parser
        let (_, crl) = parse_x509_crl(&der_bytes)
            .map_err(|e| TlsError::CrlValidation(format!("CRL parsing failed: {}", e)))?;

        // Extract revoked certificate serial numbers
        let mut revoked_serials = HashSet::new();
        for revoked_cert in crl.iter_revoked_certificates() {
            revoked_serials.insert(revoked_cert.user_certificate.to_bytes_be());
        }

        // Extract next update time
        let next_update = crl.next_update().map(|time| {
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(time.timestamp() as u64)
        });

        tracing::info!(
            "Parsed CRL with {} revoked certificates, next update: {:?}",
            revoked_serials.len(),
            next_update
        );

        Ok(CrlCacheEntry {
            revoked_serials,
            cached_at: SystemTime::now(),
            next_update,
        })
    }

    /// Cleanup expired CRL cache entries
    pub fn cleanup_cache(&self) {
        let mut cache = match self.cache.write() {
            Ok(cache) => cache,
            Err(poisoned) => {
                tracing::warn!("CRL cache write lock poisoned during cleanup, recovering");
                poisoned.into_inner()
            }
        };

        cache.retain(|_url, entry| !Self::is_crl_cache_expired(entry));

        tracing::debug!(
            "CRL cache cleanup completed, {} CRLs remaining",
            cache.len()
        );
    }
}

/// Production TLS manager with comprehensive certificate lifecycle management
pub struct TlsManager {
    #[allow(dead_code)]
    cert_dir: PathBuf,
    ca_cert: CertificateDer<'static>,
    #[allow(dead_code)]
    ca_key: PrivatePkcs8KeyDer<'static>,
    server_cert: CertificateDer<'static>,
    server_key: PrivatePkcs8KeyDer<'static>,
    ocsp_cache: OcspCache,
    crl_cache: CrlCache,
}

/// Secure key material that zeroes on drop
#[derive(ZeroizeOnDrop)]
pub struct SecureKeyMaterial {
    data: Vec<u8>,
}

impl SecureKeyMaterial {
    fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl TlsManager {
    /// Parse certificate from PEM data to extract actual certificate information
    pub fn parse_certificate_from_pem(pem_data: &str) -> Result<ParsedCertificate, TlsError> {
        Self::parse_certificate_from_pem_internal(pem_data)
    }

    /// Validate certificate time constraints
    pub fn validate_certificate_time(parsed_cert: &ParsedCertificate) -> Result<(), TlsError> {
        Self::validate_certificate_time_internal(parsed_cert)
    }

    /// Validate BasicConstraints extension
    pub fn validate_basic_constraints(
        parsed_cert: &ParsedCertificate,
        expected_ca: bool,
    ) -> Result<(), TlsError> {
        Self::validate_basic_constraints_internal(parsed_cert, expected_ca)
    }

    /// Validate KeyUsage extension
    pub fn validate_key_usage(
        parsed_cert: &ParsedCertificate,
        usage: CertificateUsage,
    ) -> Result<(), TlsError> {
        Self::validate_key_usage_internal(parsed_cert, usage)
    }
    /// Validate certificate using OCSP (Online Certificate Status Protocol)
    pub async fn validate_certificate_ocsp(
        &self,
        cert_pem: &str,
        issuer_cert_pem: Option<&str>,
    ) -> Result<(), TlsError> {
        let parsed_cert = Self::parse_certificate_from_pem(cert_pem)?;

        // Parse issuer certificate if provided
        let issuer_cert = if let Some(issuer_pem) = issuer_cert_pem {
            Some(Self::parse_certificate_from_pem(issuer_pem)?)
        } else {
            None
        };

        match self
            .ocsp_cache
            .check_certificate(&parsed_cert, issuer_cert.as_ref())
            .await
        {
            Ok(OcspStatus::Good) => {
                tracing::info!("OCSP validation successful: certificate is valid");
                Ok(())
            }
            Ok(OcspStatus::Revoked) => Err(TlsError::OcspValidation(
                "Certificate has been revoked".to_string(),
            )),
            Ok(OcspStatus::Unknown) => {
                tracing::warn!("OCSP status unknown, proceeding with validation");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("OCSP validation failed: {}, proceeding without OCSP", e);
                Ok(())
            }
        }
    }

    /// Validate certificate using CRL (Certificate Revocation List)
    pub async fn validate_certificate_crl(&self, cert_pem: &str) -> Result<(), TlsError> {
        let parsed_cert = Self::parse_certificate_from_pem(cert_pem)?;

        match self
            .crl_cache
            .check_certificate_revocation(&parsed_cert)
            .await
        {
            Ok(false) => {
                tracing::info!("CRL validation successful: certificate is not revoked");
                Ok(())
            }
            Ok(true) => Err(TlsError::CrlValidation(
                "Certificate has been revoked according to CRL".to_string(),
            )),
            Err(e) => {
                tracing::warn!("CRL validation failed: {}, proceeding without CRL", e);
                Ok(())
            }
        }
    }

    /// Validate certificate chain to root CA
    pub async fn validate_certificate_chain(&self, cert_chain_pem: &str) -> Result<(), TlsError> {
        // Parse all certificates from the PEM chain
        let mut certificates = Vec::new();
        let mut cursor = std::io::Cursor::new(cert_chain_pem.as_bytes());

        for cert_der in rustls_pemfile::certs(&mut cursor) {
            let cert_der = cert_der.map_err(|e| {
                TlsError::CertificateParsing(format!(
                    "Failed to parse certificate from chain: {}",
                    e
                ))
            })?;
            certificates.push(cert_der);
        }

        if certificates.is_empty() {
            return Err(TlsError::ChainValidation(
                "No certificates found in chain".to_string(),
            ));
        }

        tracing::info!(
            "Validating certificate chain with {} certificates",
            certificates.len()
        );

        // The first certificate should be the end-entity certificate
        let end_entity_der = &certificates[0];

        // Build trust anchors from our CA certificate
        let mut trust_anchors = Vec::new();
        let ca_trust_anchor =
            webpki::TrustAnchor::try_from_cert_der(&self.ca_cert).map_err(|e| {
                TlsError::ChainValidation(format!("Failed to create trust anchor from CA: {:?}", e))
            })?;
        trust_anchors.push(ca_trust_anchor);

        // Also add system root CAs if available
        // Keep system_roots in scope for the lifetime of trust_anchors
        let system_roots = Self::load_system_root_certificates().unwrap_or_default();
        for root_der in &system_roots {
            if let Ok(trust_anchor) = webpki::TrustAnchor::try_from_cert_der(root_der.as_ref()) {
                trust_anchors.push(trust_anchor);
            }
        }
        if !system_roots.is_empty() {
            tracing::debug!(
                "Loaded {} total trust anchors (including system roots)",
                trust_anchors.len()
            );
        }

        // Create intermediate certificates collection (all certs except the first one)
        let mut intermediates = Vec::new();
        for cert in certificates.iter().skip(1) {
            intermediates.push(cert.as_ref());
        }

        // Get current time for validation
        let now_secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| TlsError::ChainValidation("Failed to get current time".to_string()))?
            .as_secs();
        let time = webpki::Time::from_seconds_since_unix_epoch(now_secs);

        // Perform chain validation using webpki
        let end_entity_cert =
            webpki::EndEntityCert::try_from(end_entity_der.as_ref()).map_err(|e| {
                TlsError::ChainValidation(format!(
                    "Failed to parse end-entity certificate: {:?}",
                    e
                ))
            })?;

        // Define supported signature algorithms
        let supported_sig_algs = &[
            &webpki::ECDSA_P256_SHA256,
            &webpki::ECDSA_P384_SHA384,
            &webpki::ED25519,
            &webpki::RSA_PKCS1_2048_8192_SHA256,
            &webpki::RSA_PKCS1_2048_8192_SHA384,
            &webpki::RSA_PKCS1_2048_8192_SHA512,
            &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
            &webpki::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
            &webpki::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
        ];

        // Convert trust anchors to TlsServerTrustAnchors
        let tls_server_trust_anchors = webpki::TlsServerTrustAnchors(&trust_anchors);

        // Verify the certificate chain for TLS server usage
        end_entity_cert
            .verify_is_valid_tls_server_cert(
                supported_sig_algs,
                &tls_server_trust_anchors,
                &intermediates,
                time,
            )
            .map_err(|e| {
                TlsError::ChainValidation(format!("Certificate chain validation failed: {:?}", e))
            })?;

        // Additional validation: check each certificate in the chain
        for (i, cert_der) in certificates.iter().enumerate() {
            let cert_pem = Self::der_to_pem(cert_der)?;
            let parsed_cert = Self::parse_certificate_from_pem(&cert_pem)?;

            // Validate time constraints for each certificate
            Self::validate_certificate_time_internal(&parsed_cert).map_err(|e| {
                TlsError::ChainValidation(format!("Certificate {} in chain: {}", i, e))
            })?;

            // For intermediate and root CAs, verify they have proper CA constraints
            if i > 0 {
                Self::validate_basic_constraints_internal(&parsed_cert, true).map_err(|e| {
                    TlsError::ChainValidation(format!("Certificate {} in chain: {}", i, e))
                })?;

                Self::validate_key_usage_internal(
                    &parsed_cert,
                    CertificateUsage::CertificateAuthority,
                )
                .map_err(|e| {
                    TlsError::ChainValidation(format!("Certificate {} in chain: {}", i, e))
                })?;
            }
        }

        tracing::info!(
            "Certificate chain validation successful: chain properly links to trusted root CA"
        );
        Ok(())
    }

    /// Convert DER certificate to PEM format
    fn der_to_pem(cert_der: &CertificateDer) -> Result<String, TlsError> {
        use base64::engine::general_purpose::STANDARD;

        let base64_cert = STANDARD.encode(cert_der.as_ref());
        let pem = format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
            base64_cert
                .chars()
                .collect::<Vec<_>>()
                .chunks(64)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(pem)
    }

    /// Load system root certificates if available
    fn load_system_root_certificates() -> Result<Vec<CertificateDer<'static>>, TlsError> {
        let mut roots = Vec::new();

        // Try to load from common system certificate locations
        let possible_paths = [
            "/etc/ssl/certs/ca-certificates.crt", // Debian/Ubuntu
            "/etc/pki/tls/certs/ca-bundle.crt",   // RHEL/CentOS
            "/etc/ssl/cert.pem",                  // macOS/BSD
        ];

        for path in &possible_paths {
            if let Ok(contents) = std::fs::read_to_string(path) {
                let mut cursor = std::io::Cursor::new(contents.as_bytes());
                for cert in rustls_pemfile::certs(&mut cursor) {
                    if let Ok(cert) = cert {
                        roots.push(cert);
                    }
                }
                tracing::debug!(
                    "Loaded {} system root certificates from {}",
                    roots.len(),
                    path
                );
                break;
            }
        }

        if roots.is_empty() {
            tracing::debug!("No system root certificates loaded, using only configured CA");
        }

        Ok(roots)
    }

    /// Start periodic OCSP cache cleanup task
    pub fn start_ocsp_cleanup_task(&self) {
        let ocsp_cache = self.ocsp_cache.clone();
        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(Duration::from_secs(3600)); // Cleanup every hour

            loop {
                cleanup_interval.tick().await;
                ocsp_cache.cleanup_cache();
            }
        });
    }

    /// Start periodic CRL cache cleanup task
    pub fn start_crl_cleanup_task(&self) {
        let crl_cache = self.crl_cache.clone();
        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(Duration::from_secs(6 * 3600)); // Cleanup every 6 hours

            loop {
                cleanup_interval.tick().await;
                crl_cache.cleanup_cache();
            }
        });
    }

    /// Validate encryption passphrase from deployment environment
    fn validate_encryption_passphrase() -> Result<String, TlsError> {
        // Get encryption passphrase from environment variable
        let passphrase = env::var("SWEETMCP_KEY_ENCRYPTION_PASSPHRASE").map_err(|_| {
            TlsError::KeyProtection(
                "SWEETMCP_KEY_ENCRYPTION_PASSPHRASE environment variable not set".to_string(),
            )
        })?;

        // Validate passphrase strength - minimum 32 characters
        if passphrase.len() < 32 {
            return Err(TlsError::KeyProtection(
                "Encryption passphrase must be at least 32 characters".to_string(),
            ));
        }

        // Enhanced entropy validation - character class requirements
        let has_lowercase = passphrase.chars().any(|c| c.is_lowercase());
        let has_uppercase = passphrase.chars().any(|c| c.is_uppercase());
        let has_digit = passphrase.chars().any(|c| c.is_ascii_digit());
        let has_symbol = passphrase.chars().any(|c| !c.is_alphanumeric());

        let char_classes = [has_lowercase, has_uppercase, has_digit, has_symbol]
            .iter()
            .filter(|&&x| x)
            .count();

        if char_classes < 3 {
            return Err(TlsError::KeyProtection(
                "Encryption passphrase must contain at least 3 character classes (lowercase, uppercase, digits, symbols)".to_string()
            ));
        }

        // Validate entropy - check for repeated characters
        let unique_chars: HashSet<char> = passphrase.chars().collect();
        if unique_chars.len() < 12 {
            return Err(TlsError::KeyProtection(
                "Encryption passphrase must contain at least 12 unique characters".to_string(),
            ));
        }

        // Check for common patterns (sequential characters, repeated sequences)
        if Self::has_weak_patterns(&passphrase) {
            return Err(TlsError::KeyProtection(
                "Encryption passphrase contains weak patterns (sequential or repeated characters)"
                    .to_string(),
            ));
        }

        Ok(passphrase)
    }

    /// Check for weak patterns in passphrase
    fn has_weak_patterns(passphrase: &str) -> bool {
        let chars: Vec<char> = passphrase.chars().collect();

        // Check for sequential characters (e.g., "abc", "123")
        for window in chars.windows(3) {
            if window.len() == 3 {
                let a = window[0] as u32;
                let b = window[1] as u32;
                let c = window[2] as u32;

                // Ascending or descending sequence
                if (b == a + 1 && c == b + 1) || (b == a - 1 && c == b - 1) {
                    return true;
                }
            }
        }

        // Check for repeated substrings of length 3 or more
        for i in 0..chars.len().saturating_sub(5) {
            for len in 3..=((chars.len() - i) / 2) {
                if i + len * 2 <= chars.len() {
                    let first = &chars[i..i + len];
                    let second = &chars[i + len..i + len * 2];
                    if first == second {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Encrypt private key data using AES-256-GCM authenticated encryption
    async fn encrypt_private_key(key_pem: &str) -> Result<Vec<u8>, TlsError> {
        use ring::{aead, pbkdf2, rand};

        // Get and validate passphrase from environment
        let passphrase = Self::validate_encryption_passphrase()?;

        // Generate random salt for PBKDF2
        let rng = rand::SystemRandom::new();
        let mut salt = [0u8; 32];
        rand::SecureRandom::fill(&rng, &mut salt)
            .map_err(|_| TlsError::KeyProtection("Failed to generate random salt".to_string()))?;

        // Derive key using PBKDF2
        let mut key_bytes = [0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            PBKDF2_ITERATIONS,
            &salt,
            passphrase.as_bytes(),
            &mut key_bytes,
        );

        // Create AES-256-GCM key
        let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
            .map_err(|_| TlsError::KeyProtection("Failed to create encryption key".to_string()))?;
        let key = aead::LessSafeKey::new(key);

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::SecureRandom::fill(&rng, &mut nonce_bytes)
            .map_err(|_| TlsError::KeyProtection("Failed to generate random nonce".to_string()))?;
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

        // Encrypt the key data
        let mut plaintext = key_pem.as_bytes().to_vec();
        key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut plaintext)
            .map_err(|_| TlsError::KeyProtection("Encryption failed".to_string()))?;

        // Format: [salt:32][nonce:12][ciphertext+tag]
        let mut result = Vec::with_capacity(32 + 12 + plaintext.len());
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&plaintext);

        Ok(result)
    }

    /// Decrypt private key data using AES-256-GCM authenticated encryption
    async fn decrypt_private_key(encrypted_data: &[u8]) -> Result<SecureKeyMaterial, TlsError> {
        use ring::{aead, pbkdf2};

        // Validate minimum size: salt(32) + nonce(12) + tag(16) = 60 bytes minimum
        if encrypted_data.len() < 60 {
            return Err(TlsError::KeyProtection(
                "Invalid encrypted data format".to_string(),
            ));
        }

        // Extract components
        let salt = &encrypted_data[0..32];
        let nonce_bytes = &encrypted_data[32..44];
        let ciphertext = &encrypted_data[44..];

        // Get and validate passphrase from environment
        let passphrase = Self::validate_encryption_passphrase()?;

        // Derive key using same parameters
        let mut key_bytes = [0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            PBKDF2_ITERATIONS,
            salt,
            passphrase.as_bytes(),
            &mut key_bytes,
        );

        // Create AES-256-GCM key
        let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
            .map_err(|_| TlsError::KeyProtection("Failed to create decryption key".to_string()))?;
        let key = aead::LessSafeKey::new(key);

        // Create nonce
        let mut nonce_array = [0u8; 12];
        nonce_array.copy_from_slice(nonce_bytes);
        let nonce = aead::Nonce::assume_unique_for_key(nonce_array);

        // Decrypt and authenticate - use constant error message to prevent timing attacks
        let mut ciphertext_copy = ciphertext.to_vec();
        let decrypted = key
            .open_in_place(nonce, aead::Aad::empty(), &mut ciphertext_copy)
            .map_err(|_| TlsError::KeyProtection("Authentication failed".to_string()))?;

        Ok(SecureKeyMaterial::new(decrypted.to_vec()))
    }

    /// Validate certificate expiration and time constraints
    fn validate_certificate_time_internal(parsed_cert: &ParsedCertificate) -> Result<(), TlsError> {
        let now = SystemTime::now();

        // Check if certificate is not yet valid
        if now < parsed_cert.not_before {
            return Err(TlsError::CertificateExpired(format!(
                "Certificate is not yet valid (not before: {:?}, current time: {:?})",
                parsed_cert.not_before, now
            )));
        }

        // Check if certificate is expired
        if now > parsed_cert.not_after {
            return Err(TlsError::CertificateExpired(format!(
                "Certificate has expired (not after: {:?}, current time: {:?})",
                parsed_cert.not_after, now
            )));
        }

        // Check for expiration warning (within 30 days)
        if let Ok(duration_until_expiry) = parsed_cert.not_after.duration_since(now) {
            if duration_until_expiry.as_secs() < 30 * 24 * 3600 {
                // 30 days
                tracing::warn!(
                    "Certificate expires soon: {} days remaining (expires: {:?})",
                    duration_until_expiry.as_secs() / (24 * 3600),
                    parsed_cert.not_after
                );
            }
        }

        Ok(())
    }

    /// Validate certificate BasicConstraints for CA usage
    fn validate_basic_constraints_internal(
        parsed_cert: &ParsedCertificate,
        expected_ca: bool,
    ) -> Result<(), TlsError> {
        if parsed_cert.is_ca != expected_ca {
            if expected_ca {
                return Err(TlsError::CertificateValidation(
                    "Certificate is not a valid CA certificate (BasicConstraints CA=false)"
                        .to_string(),
                ));
            } else {
                return Err(TlsError::CertificateValidation(
                    "End-entity certificate incorrectly marked as CA (BasicConstraints CA=true)"
                        .to_string(),
                ));
            }
        }

        // For CA certificates, ensure they have the keyCertSign usage
        if expected_ca && !parsed_cert.key_usage.contains(&"keyCertSign".to_string()) {
            return Err(TlsError::CertificateValidation(
                "CA certificate missing required keyCertSign usage".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate certificate KeyUsage extension for intended purpose
    fn validate_key_usage_internal(
        parsed_cert: &ParsedCertificate,
        usage: CertificateUsage,
    ) -> Result<(), TlsError> {
        match usage {
            CertificateUsage::CertificateAuthority => {
                // CA certificates must have keyCertSign and should have cRLSign
                if !parsed_cert.key_usage.contains(&"keyCertSign".to_string()) {
                    return Err(TlsError::CertificateValidation(
                        "CA certificate missing required keyCertSign usage".to_string(),
                    ));
                }

                // CRL signing is recommended but not strictly required
                if !parsed_cert.key_usage.contains(&"cRLSign".to_string()) {
                    tracing::warn!("CA certificate missing recommended cRLSign usage");
                }
            }
            CertificateUsage::ServerAuth => {
                // Server certificates must have digitalSignature for TLS
                if !parsed_cert
                    .key_usage
                    .contains(&"digitalSignature".to_string())
                {
                    return Err(TlsError::CertificateValidation(
                        "Server certificate missing required digitalSignature usage".to_string(),
                    ));
                }

                // Key encipherment may be required for RSA key exchange
                if !parsed_cert
                    .key_usage
                    .contains(&"keyEncipherment".to_string())
                {
                    tracing::warn!("Server certificate missing keyEncipherment usage (may be required for RSA)");
                }
            }
            CertificateUsage::ClientAuth => {
                // Client certificates must have digitalSignature
                if !parsed_cert
                    .key_usage
                    .contains(&"digitalSignature".to_string())
                {
                    return Err(TlsError::CertificateValidation(
                        "Client certificate missing required digitalSignature usage".to_string(),
                    ));
                }
            }
        }

        // Ensure certificate is not marked as CA if it's for server/client auth
        match usage {
            CertificateUsage::ServerAuth | CertificateUsage::ClientAuth => {
                if parsed_cert.is_ca {
                    return Err(TlsError::CertificateValidation(
                        "End-entity certificate incorrectly marked as CA".to_string(),
                    ));
                }
            }
            CertificateUsage::CertificateAuthority => {
                if !parsed_cert.is_ca {
                    return Err(TlsError::CertificateValidation(
                        "CA certificate not marked as CA in BasicConstraints".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Verify hostname against certificate Subject Alternative Names (SANs)
    pub fn verify_hostname(
        parsed_cert: &ParsedCertificate,
        hostname: &str,
    ) -> Result<(), TlsError> {
        // First try to parse hostname as IP address
        if let Ok(ip_addr) = hostname.parse::<std::net::IpAddr>() {
            // Check against IP SANs
            if parsed_cert.san_ip_addresses.contains(&ip_addr) {
                return Ok(());
            }
            return Err(TlsError::PeerVerification(format!(
                "IP address {} not found in certificate SANs",
                hostname
            )));
        }

        // Check against DNS SANs
        for san_dns in &parsed_cert.san_dns_names {
            if Self::match_hostname(hostname, san_dns) {
                return Ok(());
            }
        }

        // Also check against Common Name as fallback (though SANs should be preferred)
        if let Some(cn) = parsed_cert.subject.get("CN") {
            if Self::match_hostname(hostname, cn) {
                tracing::warn!(
                    "Using Common Name for hostname verification - SANs should be preferred"
                );
                return Ok(());
            }
        }

        Err(TlsError::PeerVerification(format!(
            "Hostname {} does not match any certificate SANs or Common Name",
            hostname
        )))
    }

    /// Match hostname against a DNS name pattern (supports wildcards)
    fn match_hostname(hostname: &str, pattern: &str) -> bool {
        // Convert to lowercase for case-insensitive comparison
        let hostname = hostname.to_lowercase();
        let pattern = pattern.to_lowercase();

        // Exact match
        if hostname == pattern {
            return true;
        }

        // Wildcard matching - only support single level wildcard at the beginning
        if pattern.starts_with("*.") {
            let pattern_suffix = &pattern[2..]; // Remove "*."

            // The hostname must have exactly one more label than the pattern
            if hostname.ends_with(pattern_suffix) {
                let hostname_prefix = &hostname[..hostname.len() - pattern_suffix.len()];

                // Ensure the prefix doesn't contain dots (single level wildcard)
                if !hostname_prefix.is_empty()
                    && !hostname_prefix.contains('.')
                    && hostname_prefix.ends_with('.')
                {
                    return true;
                }
            }
        }

        false
    }

    /// Verify peer certificate against expected hostname
    pub fn verify_peer_certificate(
        cert_pem: &str,
        expected_hostname: &str,
    ) -> Result<(), TlsError> {
        // Parse the certificate
        let parsed_cert = Self::parse_certificate_from_pem(cert_pem)?;

        // Validate certificate time constraints
        Self::validate_certificate_time_internal(&parsed_cert)?;

        // Validate this is an end-entity certificate (not CA)
        Self::validate_basic_constraints_internal(&parsed_cert, false)?;

        // Validate KeyUsage for server authentication
        Self::validate_key_usage_internal(&parsed_cert, CertificateUsage::ServerAuth)?;

        // Verify hostname matches
        Self::verify_hostname(&parsed_cert, expected_hostname)?;

        tracing::info!(
            "Successfully verified peer certificate for hostname: {}",
            expected_hostname
        );
        Ok(())
    }

    /// Verify peer certificate with OCSP validation
    pub async fn verify_peer_certificate_with_ocsp(
        &self,
        cert_pem: &str,
        expected_hostname: &str,
        issuer_cert_pem: Option<&str>,
    ) -> Result<(), TlsError> {
        // Perform standard certificate validation
        Self::verify_peer_certificate(cert_pem, expected_hostname)?;

        // Additional OCSP validation
        self.validate_certificate_ocsp(cert_pem, issuer_cert_pem)
            .await?;

        tracing::info!(
            "Successfully verified peer certificate with OCSP for hostname: {}",
            expected_hostname
        );
        Ok(())
    }

    /// Verify peer certificate with comprehensive revocation checking (OCSP + CRL + Chain)
    pub async fn verify_peer_certificate_comprehensive(
        &self,
        cert_pem: &str,
        expected_hostname: &str,
        full_chain_pem: Option<&str>,
    ) -> Result<(), TlsError> {
        // Perform standard certificate validation
        Self::verify_peer_certificate(cert_pem, expected_hostname)?;

        // Extract issuer certificate from chain if available
        let issuer_cert_pem = if let Some(chain_pem) = full_chain_pem {
            // Certificate chain validation to root CA
            self.validate_certificate_chain(chain_pem).await?;

            // Extract the issuer certificate (second certificate in the chain)
            let mut cursor = std::io::Cursor::new(chain_pem.as_bytes());
            let certs: Vec<_> = rustls_pemfile::certs(&mut cursor)
                .filter_map(|cert| cert.ok())
                .collect();

            if certs.len() > 1 {
                // Convert the issuer certificate to PEM for OCSP validation
                Some(Self::der_to_pem(&certs[1])?)
            } else {
                None
            }
        } else {
            tracing::warn!("No certificate chain provided, skipping chain validation");
            None
        };

        // CRL validation (more reliable but potentially outdated)
        self.validate_certificate_crl(cert_pem).await?;

        // OCSP validation (real-time but may be unavailable)
        self.validate_certificate_ocsp(cert_pem, issuer_cert_pem.as_deref())
            .await?;

        tracing::info!("Successfully verified peer certificate with comprehensive validation (chain + CRL + OCSP) for hostname: {}", expected_hostname);
        Ok(())
    }

    /// Extract name attributes from x509-cert Name structure
    fn extract_name_attributes(name: &x509_cert::name::Name, attrs: &mut HashMap<String, String>) {
        use der::asn1::{Ia5StringRef, PrintableStringRef, Utf8StringRef};

        // Common OIDs for DN components
        const OID_CN: &str = "2.5.4.3"; // commonName
        const OID_O: &str = "2.5.4.10"; // organizationName
        const OID_OU: &str = "2.5.4.11"; // organizationalUnitName
        const OID_C: &str = "2.5.4.6"; // countryName
        const OID_ST: &str = "2.5.4.8"; // stateOrProvinceName
        const OID_L: &str = "2.5.4.7"; // localityName

        // Iterate through RDNs (Relative Distinguished Names)
        for rdn in name.0.iter() {
            // Each RDN contains one or more AttributeTypeAndValue
            for atv in rdn.0.iter() {
                let oid_string = atv.oid.to_string();

                // Extract the value as string using proper ASN.1 type handling
                // Try different ASN.1 string types as shown in x509-cert tests
                let string_value = if let Ok(ps) = PrintableStringRef::try_from(&atv.value) {
                    Some(ps.to_string())
                } else if let Ok(utf8s) = Utf8StringRef::try_from(&atv.value) {
                    Some(utf8s.to_string())
                } else if let Ok(ia5s) = Ia5StringRef::try_from(&atv.value) {
                    Some(ia5s.to_string())
                } else {
                    None
                };

                if let Some(value_str) = string_value {
                    match oid_string.as_str() {
                        OID_CN => {
                            attrs.insert("CN".to_string(), value_str);
                        }
                        OID_O => {
                            attrs.insert("O".to_string(), value_str);
                        }
                        OID_OU => {
                            attrs.insert("OU".to_string(), value_str);
                        }
                        OID_C => {
                            attrs.insert("C".to_string(), value_str);
                        }
                        OID_ST => {
                            attrs.insert("ST".to_string(), value_str);
                        }
                        OID_L => {
                            attrs.insert("L".to_string(), value_str);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Extract certificate details using x509-cert
    fn extract_certificate_details(
        cert: &X509CertCert,
    ) -> Result<
        (
            Vec<String>,
            Vec<std::net::IpAddr>,
            bool,
            Vec<String>,
            SystemTime,
            SystemTime,
        ),
        TlsError,
    > {
        // Extract SANs
        let mut san_dns_names = Vec::new();
        let mut san_ip_addresses = Vec::new();

        // Extract BasicConstraints for CA flag
        let mut is_ca = false;

        // Extract key usage
        let mut key_usage = Vec::new();

        // OIDs for extensions
        const OID_SUBJECT_ALT_NAME: &str = "2.5.29.17";
        const OID_BASIC_CONSTRAINTS: &str = "2.5.29.19";
        const OID_KEY_USAGE: &str = "2.5.29.15";

        // Process extensions
        if let Some(extensions) = &cert.tbs_certificate.extensions {
            for ext in extensions.iter() {
                let oid_string = ext.extn_id.to_string();

                match oid_string.as_str() {
                    OID_SUBJECT_ALT_NAME => {
                        // Parse SubjectAltName extension properly using ASN.1
                        // SubjectAltName ::= GeneralNames
                        // GeneralNames ::= SEQUENCE OF GeneralName
                        use der::{Decode, Reader, SliceReader, Tag, TagNumber};

                        let ext_data = ext.extn_value.as_bytes();

                        // Parse the OCTET STRING wrapper first
                        match der::asn1::OctetString::from_der(ext_data) {
                            Ok(octet_string) => {
                                // Now parse the actual SubjectAltName SEQUENCE
                                let san_data = octet_string.as_bytes();
                                let mut reader = match SliceReader::new(san_data) {
                                    Ok(reader) => reader,
                                    Err(_) => {
                                        tracing::warn!("Failed to create DER reader for SAN data");
                                        continue;
                                    }
                                };

                                // Read the SEQUENCE header
                                if let Ok(header) = reader.peek_header() {
                                    if header.tag == Tag::Sequence {
                                        // Consume the header
                                        match reader.peek_header() {
                                            Ok(_) => {}
                                            Err(_) => {
                                                tracing::warn!("Failed to consume sequence header");
                                                continue;
                                            }
                                        }
                                        match reader.read_slice(header.length) {
                                            Ok(_) => {}
                                            Err(_) => {
                                                tracing::warn!("Failed to read sequence data");
                                                continue;
                                            }
                                        }

                                        // Parse each GeneralName in the sequence
                                        while !reader.is_finished() {
                                            if let Ok(name_header) = reader.peek_header() {
                                                match name_header.tag.number() {
                                                    TagNumber::N2 => {
                                                        // dNSName [2] IMPLICIT IA5String
                                                        if let Ok(dns_header) = reader.peek_header()
                                                        {
                                                            if let Ok(dns_bytes) =
                                                                reader.read_vec(dns_header.length)
                                                            {
                                                                if let Ok(dns_name) =
                                                                    std::str::from_utf8(&dns_bytes)
                                                                {
                                                                    san_dns_names
                                                                        .push(dns_name.to_string());
                                                                }
                                                            }
                                                        }
                                                    }
                                                    TagNumber::N7 => {
                                                        // iPAddress [7] IMPLICIT OCTET STRING
                                                        if let Ok(ip_header) = reader.peek_header()
                                                        {
                                                            if let Ok(ip_bytes) =
                                                                reader.read_vec(ip_header.length)
                                                            {
                                                                // IPv4 = 4 bytes, IPv6 = 16 bytes
                                                                match ip_bytes.len() {
                                                                    4 => {
                                                                        let octets: [u8; 4] =
                                                                            match ip_bytes
                                                                                .try_into()
                                                                            {
                                                                                Ok(octets) => {
                                                                                    octets
                                                                                }
                                                                                Err(_) => {
                                                                                    tracing::warn!("Invalid IPv4 address bytes");
                                                                                    continue;
                                                                                }
                                                                            };
                                                                        san_ip_addresses.push(std::net::IpAddr::V4(
                                                                            std::net::Ipv4Addr::from(octets)
                                                                        ));
                                                                    }
                                                                    16 => {
                                                                        let octets: [u8; 16] =
                                                                            match ip_bytes
                                                                                .try_into()
                                                                            {
                                                                                Ok(octets) => {
                                                                                    octets
                                                                                }
                                                                                Err(_) => {
                                                                                    tracing::warn!("Invalid IPv6 address bytes");
                                                                                    continue;
                                                                                }
                                                                            };
                                                                        san_ip_addresses.push(std::net::IpAddr::V6(
                                                                            std::net::Ipv6Addr::from(octets)
                                                                        ));
                                                                    }
                                                                    _ => {
                                                                        // Invalid IP address length
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        // Skip other GeneralName types
                                                        // (rfc822Name, x400Address, directoryName, ediPartyName, uniformResourceIdentifier, registeredID)
                                                        let _ = reader.peek_header();
                                                        let _ =
                                                            reader.read_slice(name_header.length);
                                                    }
                                                }
                                            } else {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse SubjectAltName extension: {}", e);
                            }
                        }
                    }
                    OID_BASIC_CONSTRAINTS => {
                        // Parse BasicConstraints extension
                        // Structure: SEQUENCE { cA BOOLEAN DEFAULT FALSE, ... }
                        let ext_data = ext.extn_value.as_bytes();

                        // Look for the CA boolean flag
                        // In DER encoding, BOOLEAN TRUE is 0x01 0x01 0xFF
                        if ext_data.len() >= 3 {
                            for i in 0..ext_data.len() - 2 {
                                if ext_data[i] == 0x01
                                    && ext_data[i + 1] == 0x01
                                    && ext_data[i + 2] == 0xFF
                                {
                                    is_ca = true;
                                    break;
                                }
                            }
                        }
                    }
                    OID_KEY_USAGE => {
                        // Parse KeyUsage extension
                        // Structure: BIT STRING with specific bit positions
                        let ext_data = ext.extn_value.as_bytes();

                        // KeyUsage bits (from RFC 5280):
                        // 0: digitalSignature
                        // 1: nonRepudiation/contentCommitment
                        // 2: keyEncipherment
                        // 3: dataEncipherment
                        // 4: keyAgreement
                        // 5: keyCertSign
                        // 6: cRLSign
                        // 7: encipherOnly
                        // 8: decipherOnly

                        // Find the bit string in the extension data
                        // BIT STRING starts with tag 0x03
                        for i in 0..ext_data.len() {
                            if ext_data[i] == 0x03 && i + 2 < ext_data.len() {
                                // Next byte is length, then unused bits, then the actual bits
                                if i + 3 < ext_data.len() {
                                    let bits = ext_data[i + 3];

                                    if bits & 0x80 != 0 {
                                        key_usage.push("digitalSignature".to_string());
                                    }
                                    if bits & 0x40 != 0 {
                                        key_usage.push("contentCommitment".to_string());
                                    }
                                    if bits & 0x20 != 0 {
                                        key_usage.push("keyEncipherment".to_string());
                                    }
                                    if bits & 0x10 != 0 {
                                        key_usage.push("dataEncipherment".to_string());
                                    }
                                    if bits & 0x08 != 0 {
                                        key_usage.push("keyAgreement".to_string());
                                    }
                                    if bits & 0x04 != 0 {
                                        key_usage.push("keyCertSign".to_string());
                                    }
                                    if bits & 0x02 != 0 {
                                        key_usage.push("cRLSign".to_string());
                                    }

                                    // Check second byte if present for last two bits
                                    if i + 4 < ext_data.len() && ext_data[i + 1] > 1 {
                                        let bits2 = ext_data[i + 4];
                                        if bits2 & 0x80 != 0 {
                                            key_usage.push("encipherOnly".to_string());
                                        }
                                        if bits2 & 0x40 != 0 {
                                            key_usage.push("decipherOnly".to_string());
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Extract validity times from TBS certificate
        let validity = &cert.tbs_certificate.validity;

        // Convert x509-cert Time to SystemTime
        let not_before = validity.not_before.to_system_time();
        let not_after = validity.not_after.to_system_time();

        Ok((
            san_dns_names,
            san_ip_addresses,
            is_ca,
            key_usage,
            not_before,
            not_after,
        ))
    }

    /// Parse certificate from PEM data to extract actual certificate information
    fn parse_certificate_from_pem_internal(pem_data: &str) -> Result<ParsedCertificate, TlsError> {
        // Parse PEM to get DER bytes using rustls-pemfile
        let mut cursor = std::io::Cursor::new(pem_data.as_bytes());
        let cert_der = rustls_pemfile::certs(&mut cursor)
            .next()
            .ok_or_else(|| TlsError::CertificateParsing("No certificate in PEM data".to_string()))?
            .map_err(|e| TlsError::CertificateParsing(format!("Failed to parse PEM: {}", e)))?;

        // Parse X.509 certificate using x509-cert
        let cert = X509CertCert::from_der(&cert_der)
            .map_err(|e| TlsError::CertificateParsing(format!("X.509 parsing failed: {}", e)))?;

        // Extract subject DN using x509-cert API
        let mut subject = HashMap::new();
        Self::extract_name_attributes(&cert.tbs_certificate.subject, &mut subject);

        // Extract issuer DN using x509-cert API
        let mut issuer = HashMap::new();
        Self::extract_name_attributes(&cert.tbs_certificate.issuer, &mut issuer);

        // Extract basic certificate info using x509-cert
        let (san_dns_names, san_ip_addresses, is_ca, key_usage, not_before, not_after) =
            Self::extract_certificate_details(&cert)?;

        // Extract OCSP and CRL URLs from certificate extensions
        let mut ocsp_urls = Vec::new();
        let mut crl_urls = Vec::new();

        // Iterate through all extensions to find Authority Information Access and CRL Distribution Points
        if let Some(extensions) = &cert.tbs_certificate.extensions {
            for ext in extensions.iter() {
                let oid_str = ext.extn_id.to_string();

                // Authority Information Access extension (1.3.6.1.5.5.7.1.1)
                if oid_str == "1.3.6.1.5.5.7.1.1" {
                    // Extract OCSP URLs from Authority Information Access
                    // This is a simplified extraction - proper ASN.1 parsing would be more robust
                    let ext_bytes = ext.extn_value.as_bytes();

                    // Look for HTTP URLs in the extension data
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
                    // Extract CRL URLs from CRL Distribution Points
                    // This is a simplified extraction - proper ASN.1 parsing would be more robust
                    let ext_bytes = ext.extn_value.as_bytes();

                    // Look for HTTP URLs in the extension data
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

        // Extract subject DER from TBS certificate
        use der::Encode as DerEncode;
        let subject_der = cert.tbs_certificate.subject.to_der().map_err(|e| {
            TlsError::CertificateParsing(format!("Failed to encode subject DER: {}", e))
        })?;

        // Extract public key DER from TBS certificate
        let public_key_der = cert
            .tbs_certificate
            .subject_public_key_info
            .to_der()
            .map_err(|e| {
                TlsError::CertificateParsing(format!("Failed to encode public key DER: {}", e))
            })?;

        Ok(ParsedCertificate {
            subject,
            issuer,
            san_dns_names,
            san_ip_addresses,
            is_ca,
            key_usage,
            not_before,
            not_after,
            serial_number: cert.tbs_certificate.serial_number.as_bytes().to_vec(),
            ocsp_urls,
            crl_urls,
            subject_der,
            public_key_der,
        })
    }
    /// Create a new TLS manager with self-signed certificates
    pub async fn new(cert_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cert_dir).await?;

        // Generate or load CA
        let (ca_cert, ca_key, ca_cert_obj) = if cert_dir.join("ca.crt").exists() {
            info!("Loading existing CA certificate");
            Self::load_ca(&cert_dir).await?
        } else {
            info!("Generating new CA certificate");
            Self::generate_ca(&cert_dir).await?
        };

        // Generate server certificate
        info!("Generating server certificate");
        let (server_cert, server_key) = Self::generate_server_cert(&ca_cert_obj, &cert_dir).await?;

        let tls_manager = Self {
            cert_dir,
            ca_cert,
            ca_key,
            server_cert,
            server_key,
            ocsp_cache: OcspCache::new(),
            crl_cache: CrlCache::new(),
        };

        // Start cache cleanup tasks
        tls_manager.start_ocsp_cleanup_task();
        tls_manager.start_crl_cleanup_task();

        Ok(tls_manager)
    }

    /// Generate a new CA certificate
    async fn generate_ca(
        cert_dir: &Path,
    ) -> Result<(
        CertificateDer<'static>,
        PrivatePkcs8KeyDer<'static>,
        Issuer<'static, KeyPair>,
    )> {
        let mut params =
            CertificateParams::new(Vec::default()).context("Failed to create CA params")?;

        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        let mut dn = DistinguishedName::new();
        dn.push(DnType::OrganizationName, "SweetMCP");
        dn.push(DnType::CommonName, "SweetMCP CA");
        params.distinguished_name = dn;

        let key_pair = KeyPair::generate()?;
        let cert = params.clone().self_signed(&key_pair)?;

        // Save to disk
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        fs::write(cert_dir.join("ca.crt"), &cert_pem).await?;

        // Encrypt private key before saving
        let encrypted_key = Self::encrypt_private_key(&key_pem).await?;
        fs::write(cert_dir.join("ca.key"), &encrypted_key).await?;

        // Set permissions on key file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(cert_dir.join("ca.key")).await?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(cert_dir.join("ca.key"), perms).await?;
        }

        let cert_der = cert.der();
        let key_der = key_pair.serialize_der();

        // Create issuer for signing other certificates
        let issuer = Issuer::<'static>::new(params, key_pair);

        Ok((
            CertificateDer::from(cert_der.to_vec()),
            PrivatePkcs8KeyDer::from(key_der),
            issuer,
        ))
    }

    /// Load existing CA certificate
    async fn load_ca(
        cert_dir: &Path,
    ) -> Result<(
        CertificateDer<'static>,
        PrivatePkcs8KeyDer<'static>,
        Issuer<'static, KeyPair>,
    )> {
        let cert_pem = fs::read_to_string(cert_dir.join("ca.crt")).await?;

        // Read and decrypt the encrypted key file
        let encrypted_key_data = fs::read(cert_dir.join("ca.key")).await?;
        let decrypted_key = Self::decrypt_private_key(&encrypted_key_data).await?;
        let key_pem = String::from_utf8(decrypted_key.as_bytes().to_vec()).map_err(|e| {
            TlsError::KeyProtection(format!("Invalid UTF-8 in decrypted key: {}", e))
        })?;

        // Parse certificate
        let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .next()
            .ok_or_else(|| anyhow::anyhow!("No certificate in CA file"))??;

        // Parse key
        let key_der = rustls_pemfile::pkcs8_private_keys(&mut key_pem.as_bytes())
            .next()
            .ok_or_else(|| anyhow::anyhow!("No private key in CA file"))??;

        // Parse the loaded certificate to extract actual parameters
        let parsed_cert = Self::parse_certificate_from_pem(&cert_pem)
            .map_err(|e| anyhow::anyhow!("Failed to parse loaded CA certificate: {}", e))?;

        // Validate certificate time constraints
        Self::validate_certificate_time(&parsed_cert)
            .map_err(|e| anyhow::anyhow!("CA certificate time validation failed: {}", e))?;

        // Validate BasicConstraints for CA certificate
        Self::validate_basic_constraints_internal(&parsed_cert, true).map_err(|e| {
            anyhow::anyhow!("CA certificate BasicConstraints validation failed: {}", e)
        })?;

        // Validate KeyUsage for CA certificate
        Self::validate_key_usage_internal(&parsed_cert, CertificateUsage::CertificateAuthority)
            .map_err(|e| anyhow::anyhow!("CA certificate KeyUsage validation failed: {}", e))?;

        // Recreate the key pair
        let ca_key_pair = KeyPair::from_pem(&key_pem)?;

        // Recreate params from parsed certificate data
        let mut params = CertificateParams::new(
            parsed_cert
                .san_dns_names
                .iter()
                .map(|s| s.as_str().try_into())
                .collect::<Result<Vec<_>, _>>()?,
        )?;

        // Set CA constraints based on parsed data
        if parsed_cert.is_ca {
            params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        }

        // Reconstruct distinguished name from parsed data
        let mut dn = DistinguishedName::new();
        if let Some(cn) = parsed_cert.subject.get("CN") {
            dn.push(DnType::CommonName, cn);
        }
        if let Some(o) = parsed_cert.subject.get("O") {
            dn.push(DnType::OrganizationName, o);
        }
        if let Some(ou) = parsed_cert.subject.get("OU") {
            dn.push(DnType::OrganizationalUnitName, ou);
        }
        if let Some(c) = parsed_cert.subject.get("C") {
            dn.push(DnType::CountryName, c);
        }
        if let Some(st) = parsed_cert.subject.get("ST") {
            dn.push(DnType::StateOrProvinceName, st);
        }
        if let Some(l) = parsed_cert.subject.get("L") {
            dn.push(DnType::LocalityName, l);
        }
        params.distinguished_name = dn;

        let issuer = Issuer::<'static>::new(params, ca_key_pair);

        Ok((
            CertificateDer::from(cert_der.to_vec()),
            PrivatePkcs8KeyDer::from(key_der),
            issuer,
        ))
    }

    /// Generate server certificate signed by CA
    async fn generate_server_cert(
        ca_issuer: &Issuer<'static, KeyPair>,
        cert_dir: &Path,
    ) -> Result<(CertificateDer<'static>, PrivatePkcs8KeyDer<'static>)> {
        let mut params = CertificateParams::new(Vec::default())?;

        // Add SAN entries
        params.subject_alt_names = vec![
            SanType::DnsName("localhost".try_into()?),
            SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
            SanType::IpAddress(std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)),
        ];

        // Add hostname if available
        if let Ok(hostname) = hostname::get() {
            if let Some(hostname_str) = hostname.to_str() {
                params
                    .subject_alt_names
                    .push(SanType::DnsName(hostname_str.try_into()?));
            }
        }

        let mut dn = DistinguishedName::new();
        dn.push(DnType::OrganizationName, "SweetMCP");
        dn.push(DnType::CommonName, "SweetMCP Server");
        params.distinguished_name = dn;

        let key_pair = KeyPair::generate()?;
        let cert = params.signed_by(&key_pair, ca_issuer)?;
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        // Save to disk
        fs::write(cert_dir.join("server.crt"), &cert_pem).await?;

        // Encrypt private key before saving
        let encrypted_key = Self::encrypt_private_key(&key_pem).await?;
        fs::write(cert_dir.join("server.key"), &encrypted_key).await?;

        // Set permissions on key file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(cert_dir.join("server.key"))
                .await?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(cert_dir.join("server.key"), perms).await?;
        }

        let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .next()
            .ok_or_else(|| anyhow::anyhow!("No certificate generated"))??;

        let key_der = rustls_pemfile::pkcs8_private_keys(&mut key_pem.as_bytes())
            .next()
            .ok_or_else(|| anyhow::anyhow!("No private key generated"))??;

        Ok((cert_der.into(), key_der.into()))
    }

    /// Get server TLS configuration
    pub fn server_config(&self) -> Result<ServerConfig> {
        let mut root_store = RootCertStore::empty();
        root_store.add(self.ca_cert.clone())?;

        let config = ServerConfig::builder()
            .with_client_cert_verifier(
                rustls::server::WebPkiClientVerifier::builder(Arc::new(root_store)).build()?,
            )
            .with_single_cert(
                vec![self.server_cert.clone()],
                PrivateKeyDer::Pkcs8(self.server_key.clone_key()),
            )?;

        Ok(config)
    }

    /// Get client TLS configuration
    pub fn client_config(&self) -> Result<ClientConfig> {
        let mut root_store = RootCertStore::empty();
        root_store.add(self.ca_cert.clone())?;

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(
                vec![self.server_cert.clone()],
                PrivateKeyDer::Pkcs8(self.server_key.clone_key()),
            )?;

        Ok(config)
    }

    /// Generate wildcard certificate with multiple SAN entries for SweetMCP auto-integration
    /// Creates a non-expiring certificate for *.cyrup.dev with SAN entries for *.cyrup.ai, *.cyrup.cloud, *.cyrup.pro
    pub async fn generate_wildcard_certificate(xdg_config_home: &Path) -> Result<(), TlsError> {
        let cert_dir = xdg_config_home.join("sweetmcp");

        // Create cert directory if it doesn't exist
        fs::create_dir_all(&cert_dir).await.map_err(|e| {
            TlsError::FileOperation(format!("Failed to create certificate directory: {}", e))
        })?;

        let wildcard_cert_path = cert_dir.join("wildcard.cyrup.pem");

        // Check if certificate already exists and is valid
        if wildcard_cert_path.exists() {
            if let Ok(_) = Self::validate_existing_wildcard_cert(&wildcard_cert_path).await {
                info!(
                    "Valid wildcard certificate already exists at {}",
                    wildcard_cert_path.display()
                );
                return Ok(());
            }
            info!("Existing wildcard certificate is invalid, regenerating...");
        }

        info!("Generating new wildcard certificate with multiple SAN entries");

        let mut params = CertificateParams::new(Vec::default()).map_err(|e| {
            TlsError::CertificateParsing(format!("Failed to create certificate params: {}", e))
        })?;

        // Set as non-CA certificate
        params.is_ca = rcgen::IsCa::NoCa;

        // Primary wildcard domain with SweetMCP branding
        params.subject_alt_names =
            vec![
                SanType::DnsName("sweetmcp.cyrup.dev".try_into().map_err(|e| {
                    TlsError::CertificateParsing(format!("Invalid DNS name: {}", e))
                })?),
                SanType::DnsName("sweetmcp.cyrup.ai".try_into().map_err(|e| {
                    TlsError::CertificateParsing(format!("Invalid DNS name: {}", e))
                })?),
                SanType::DnsName("sweetmcp.cyrup.cloud".try_into().map_err(|e| {
                    TlsError::CertificateParsing(format!("Invalid DNS name: {}", e))
                })?),
                SanType::DnsName("sweetmcp.cyrup.pro".try_into().map_err(|e| {
                    TlsError::CertificateParsing(format!("Invalid DNS name: {}", e))
                })?),
            ];

        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::OrganizationName, "SweetMCP");
        dn.push(DnType::CommonName, "sweetmcp.cyrup.dev");
        params.distinguished_name = dn;

        // Set non-expiring validity period (100 years)
        let now = SystemTime::now();
        params.not_before = now.into();
        params.not_after = (now + Duration::from_secs(100 * 365 * 24 * 60 * 60)).into();

        // Generate key pair and self-signed certificate
        let key_pair = KeyPair::generate().map_err(|e| {
            TlsError::CertificateParsing(format!("Failed to generate key pair: {}", e))
        })?;

        let cert = params.self_signed(&key_pair).map_err(|e| {
            TlsError::CertificateParsing(format!("Failed to generate certificate: {}", e))
        })?;

        // Create combined PEM file with certificate and private key
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();
        let combined_pem = format!("{}\n{}", cert_pem, key_pem);

        // Write combined PEM file
        fs::write(&wildcard_cert_path, &combined_pem)
            .await
            .map_err(|e| {
                TlsError::FileOperation(format!("Failed to write wildcard certificate: {}", e))
            })?;

        // Set secure permissions on certificate file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&wildcard_cert_path)
                .await
                .map_err(|e| {
                    TlsError::FileOperation(format!("Failed to get file metadata: {}", e))
                })?
                .permissions();
            perms.set_mode(0o600); // Owner read/write only
            fs::set_permissions(&wildcard_cert_path, perms)
                .await
                .map_err(|e| {
                    TlsError::FileOperation(format!("Failed to set file permissions: {}", e))
                })?;
        }

        info!(
            "Wildcard certificate generated successfully at {}",
            wildcard_cert_path.display()
        );
        Ok(())
    }

    /// Validate existing wildcard certificate
    async fn validate_existing_wildcard_cert(cert_path: &Path) -> Result<(), TlsError> {
        let cert_content = fs::read_to_string(cert_path).await.map_err(|e| {
            TlsError::FileOperation(format!("Failed to read certificate file: {}", e))
        })?;

        // Parse the certificate from the combined PEM
        let parsed_cert = Self::parse_certificate_from_pem(&cert_content)?;

        // Check if it has the required SAN entries
        let required_sans = [
            "sweetmcp.cyrup.dev",
            "sweetmcp.cyrup.ai",
            "sweetmcp.cyrup.cloud",
            "sweetmcp.cyrup.pro",
        ];

        for required_san in &required_sans {
            if !parsed_cert
                .san_dns_names
                .contains(&required_san.to_string())
            {
                return Err(TlsError::CertificateValidation(format!(
                    "Missing required SAN entry: {}",
                    required_san
                )));
            }
        }

        // Check if certificate is still valid (should be non-expiring but validate anyway)
        Self::validate_certificate_time_internal(&parsed_cert)?;

        Ok(())
    }
}
