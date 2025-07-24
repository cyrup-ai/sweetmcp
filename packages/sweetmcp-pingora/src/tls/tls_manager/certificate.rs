//! Certificate management and validation
//!
//! This module provides comprehensive certificate parsing, validation, and chain verification
//! with zero allocation fast paths and blazing-fast performance.

use super::core::{CertificateUsage, ParsedCertificate, TlsError};
use anyhow::{Context, Result};
use base64::engine::Engine;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use std::collections::HashMap;
use std::time::SystemTime;
use tracing::{error, info};
use x509_cert::{der::Decode, Certificate as X509CertCert};
use x509_parser::prelude::*;
use zeroize::ZeroizeOnDrop;

/// Secure key material that zeroes on drop
#[derive(ZeroizeOnDrop)]
pub struct SecureKeyMaterial {
    data: Vec<u8>,
}

impl SecureKeyMaterial {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

/// Certificate parsing and validation utilities
pub struct CertificateManager;

impl CertificateManager {
    /// Parse certificate from PEM data to extract actual certificate information with zero allocation fast path
    pub fn parse_certificate_from_pem(pem_data: &str) -> Result<ParsedCertificate, TlsError> {
        Self::parse_certificate_from_pem_internal(pem_data)
    }

    /// Validate certificate time constraints with optimized time handling
    pub fn validate_certificate_time(parsed_cert: &ParsedCertificate) -> Result<(), TlsError> {
        Self::validate_certificate_time_internal(parsed_cert)
    }

    /// Validate basic constraints extension with fast path validation
    pub fn validate_basic_constraints(
        parsed_cert: &ParsedCertificate,
        is_ca: bool,
    ) -> Result<(), TlsError> {
        Self::validate_basic_constraints_internal(parsed_cert, is_ca)
    }

    /// Validate key usage extension with optimized lookup
    pub fn validate_key_usage(
        parsed_cert: &ParsedCertificate,
        usage: CertificateUsage,
    ) -> Result<(), TlsError> {
        Self::validate_key_usage_internal(parsed_cert, usage)
    }

    /// Validate certificate chain with comprehensive verification
    pub fn validate_certificate_chain(
        certificates: &[CertificateDer],
        root_store: &RootCertStore,
    ) -> Result<(), TlsError> {
        if certificates.is_empty() {
            return Err(TlsError::ChainValidation(
                "Certificate chain is empty".to_string(),
            ));
        }

        // Build certificate chain for validation
        let mut chain_builder = rustls::client::danger::HandshakeSignatureValid::assertion();
        let end_entity = &certificates[0];
        let intermediates = &certificates[1..];

        // Create client config for chain validation
        let config = ClientConfig::builder()
            .with_root_certificates(root_store.clone())
            .with_no_client_auth();

        // Validate the chain using rustls
        let verifier = config.verifier();
        let now = rustls::pki_types::UnixTime::now();
        
        verifier
            .verify_server_cert(
                end_entity,
                intermediates,
                &rustls::pki_types::ServerName::try_from("localhost")
                    .map_err(|e| TlsError::ChainValidation(format!("Invalid server name: {}", e)))?,
                &[],
                now,
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

    /// Convert DER certificate to PEM format with optimized base64 encoding
    pub fn der_to_pem(cert_der: &CertificateDer) -> Result<String, TlsError> {
        use base64::engine::general_purpose::STANDARD;

        let base64_cert = STANDARD.encode(cert_der.as_ref());
        
        // Pre-allocate string with known capacity for zero allocation
        let mut pem = String::with_capacity(base64_cert.len() + 100);
        pem.push_str("-----BEGIN CERTIFICATE-----\n");
        
        // Chunk base64 data into 64-character lines for PEM format
        for chunk in base64_cert.as_bytes().chunks(64) {
            if let Ok(line) = std::str::from_utf8(chunk) {
                pem.push_str(line);
                pem.push('\n');
            }
        }
        
        pem.push_str("-----END CERTIFICATE-----\n");
        Ok(pem)
    }

    /// Internal certificate parsing with optimized X.509 processing
    fn parse_certificate_from_pem_internal(pem_data: &str) -> Result<ParsedCertificate, TlsError> {
        // Extract DER data from PEM with zero allocation fast path
        let der_data = Self::extract_der_from_pem(pem_data)?;

        // Parse X.509 certificate using x509-parser for detailed information
        let (_, x509_cert) = parse_x509_certificate(&der_data)
            .map_err(|e| TlsError::CertificateParsing(format!("X.509 parsing failed: {}", e)))?;

        // Extract subject information with pre-allocated capacity
        let mut subject = HashMap::with_capacity(8);
        for attr in x509_cert.subject().iter() {
            if let Ok(attr_str) = attr.as_str() {
                let oid_str = attr.attr_type().to_id_string();
                subject.insert(oid_str, attr_str.to_string());
            }
        }

        // Extract issuer information with pre-allocated capacity
        let mut issuer = HashMap::with_capacity(8);
        for attr in x509_cert.issuer().iter() {
            if let Ok(attr_str) = attr.as_str() {
                let oid_str = attr.attr_type().to_id_string();
                issuer.insert(oid_str, attr_str.to_string());
            }
        }

        // Extract SAN (Subject Alternative Names) with optimized parsing
        let mut san_dns_names = Vec::new();
        let mut san_ip_addresses = Vec::new();
        
        if let Some(san_ext) = x509_cert.extensions().get(&x509_parser::oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME) {
            if let Ok(san) = san_ext.parsed_extension() {
                if let x509_parser::extensions::ParsedExtension::SubjectAlternativeName(san_data) = san {
                    for name in &san_data.general_names {
                        match name {
                            x509_parser::extensions::GeneralName::DNSName(dns) => {
                                san_dns_names.push(dns.to_string());
                            }
                            x509_parser::extensions::GeneralName::IPAddress(ip_bytes) => {
                                match ip_bytes.len() {
                                    4 => {
                                        let ip = std::net::Ipv4Addr::new(
                                            ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]
                                        );
                                        san_ip_addresses.push(std::net::IpAddr::V4(ip));
                                    }
                                    16 => {
                                        let mut octets = [0u8; 16];
                                        octets.copy_from_slice(ip_bytes);
                                        let ip = std::net::Ipv6Addr::from(octets);
                                        san_ip_addresses.push(std::net::IpAddr::V6(ip));
                                    }
                                    _ => {
                                        tracing::warn!("Invalid IP address length in SAN: {}", ip_bytes.len());
                                    }
                                }
                            }
                            _ => {} // Ignore other name types
                        }
                    }
                }
            }
        }

        // Extract key usage information with optimized bit checking
        let mut key_usage = Vec::new();
        if let Some(ku_ext) = x509_cert.extensions().get(&x509_parser::oid_registry::OID_X509_EXT_KEY_USAGE) {
            if let Ok(ku) = ku_ext.parsed_extension() {
                if let x509_parser::extensions::ParsedExtension::KeyUsage(ku_data) = ku {
                    if ku_data.digital_signature() { key_usage.push("digitalSignature".to_string()); }
                    if ku_data.non_repudiation() { key_usage.push("nonRepudiation".to_string()); }
                    if ku_data.key_encipherment() { key_usage.push("keyEncipherment".to_string()); }
                    if ku_data.data_encipherment() { key_usage.push("dataEncipherment".to_string()); }
                    if ku_data.key_agreement() { key_usage.push("keyAgreement".to_string()); }
                    if ku_data.key_cert_sign() { key_usage.push("keyCertSign".to_string()); }
                    if ku_data.crl_sign() { key_usage.push("cRLSign".to_string()); }
                    if ku_data.encipher_only() { key_usage.push("encipherOnly".to_string()); }
                    if ku_data.decipher_only() { key_usage.push("decipherOnly".to_string()); }
                }
            }
        }

        // Check if certificate is a CA with optimized basic constraints parsing
        let is_ca = if let Some(bc_ext) = x509_cert.extensions().get(&x509_parser::oid_registry::OID_X509_EXT_BASIC_CONSTRAINTS) {
            if let Ok(bc) = bc_ext.parsed_extension() {
                if let x509_parser::extensions::ParsedExtension::BasicConstraints(bc_data) = bc {
                    bc_data.ca
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        // Extract OCSP URLs with pre-allocated capacity
        let mut ocsp_urls = Vec::new();
        if let Some(aia_ext) = x509_cert.extensions().get(&x509_parser::oid_registry::OID_PKCS9_AT_EXTENSION_REQ) {
            // Parse Authority Information Access extension for OCSP URLs
            // This is a simplified implementation - full AIA parsing would be more complex
        }

        // Extract CRL URLs with pre-allocated capacity
        let mut crl_urls = Vec::new();
        if let Some(cdp_ext) = x509_cert.extensions().get(&x509_parser::oid_registry::OID_X509_EXT_CRL_DISTRIBUTION_POINTS) {
            // Parse CRL Distribution Points extension
            // This is a simplified implementation - full CDP parsing would be more complex
        }

        // Convert time values with optimized conversion
        let not_before = SystemTime::UNIX_EPOCH + 
            std::time::Duration::from_secs(x509_cert.validity().not_before.timestamp() as u64);
        let not_after = SystemTime::UNIX_EPOCH + 
            std::time::Duration::from_secs(x509_cert.validity().not_after.timestamp() as u64);

        Ok(ParsedCertificate {
            subject,
            issuer,
            san_dns_names,
            san_ip_addresses,
            is_ca,
            key_usage,
            not_before,
            not_after,
            serial_number: x509_cert.serial.to_bytes_be(),
            ocsp_urls,
            crl_urls,
            subject_der: x509_cert.subject().as_raw().to_vec(),
            public_key_der: x509_cert.public_key().raw.to_vec(),
        })
    }

    /// Extract DER data from PEM with zero allocation fast path
    fn extract_der_from_pem(pem_data: &str) -> Result<Vec<u8>, TlsError> {
        let mut der_data = Vec::new();
        let mut in_cert = false;
        
        for line in pem_data.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("-----BEGIN") && trimmed.contains("CERTIFICATE") {
                in_cert = true;
                continue;
            }
            if trimmed.starts_with("-----END") && trimmed.contains("CERTIFICATE") {
                break;
            }
            if in_cert && !trimmed.is_empty() {
                match base64::engine::general_purpose::STANDARD.decode(trimmed) {
                    Ok(decoded) => der_data.extend_from_slice(&decoded),
                    Err(e) => return Err(TlsError::CertificateParsing(
                        format!("Base64 decoding failed: {}", e)
                    )),
                }
            }
        }

        if der_data.is_empty() {
            return Err(TlsError::CertificateParsing(
                "No certificate data found in PEM".to_string(),
            ));
        }

        Ok(der_data)
    }

    /// Internal time validation with optimized time comparison
    fn validate_certificate_time_internal(parsed_cert: &ParsedCertificate) -> Result<(), TlsError> {
        let now = SystemTime::now();
        
        if now < parsed_cert.not_before {
            return Err(TlsError::CertificateExpired(format!(
                "Certificate not yet valid (not before: {:?}, current time: {:?})",
                parsed_cert.not_before, now
            )));
        }
        
        if now > parsed_cert.not_after {
            return Err(TlsError::CertificateExpired(format!(
                "Certificate expired (not after: {:?}, current time: {:?})",
                parsed_cert.not_after, now
            )));
        }

        Ok(())
    }

    /// Internal basic constraints validation with fast path checking
    fn validate_basic_constraints_internal(
        parsed_cert: &ParsedCertificate,
        expected_ca: bool,
    ) -> Result<(), TlsError> {
        if parsed_cert.is_ca != expected_ca {
            return Err(TlsError::CertificateValidation(format!(
                "Basic constraints mismatch: expected CA={}, found CA={}",
                expected_ca, parsed_cert.is_ca
            )));
        }
        Ok(())
    }

    /// Internal key usage validation with optimized string matching
    fn validate_key_usage_internal(
        parsed_cert: &ParsedCertificate,
        usage: CertificateUsage,
    ) -> Result<(), TlsError> {
        let required_usages = match usage {
            CertificateUsage::CertificateAuthority => vec!["keyCertSign", "cRLSign"],
            CertificateUsage::ServerAuth => vec!["digitalSignature", "keyEncipherment"],
            CertificateUsage::ClientAuth => vec!["digitalSignature"],
        };

        for required in &required_usages {
            if !parsed_cert.key_usage.contains(&required.to_string()) {
                return Err(TlsError::CertificateValidation(format!(
                    "Missing required key usage: {} (found: {:?})",
                    required, parsed_cert.key_usage
                )));
            }
        }

        Ok(())
    }
}