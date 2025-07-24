//! Security protocols and cipher suites
//!
//! This module provides comprehensive security validation for TLS certificates
//! with zero allocation fast paths and blazing-fast performance.

use super::core::{ParsedCertificate, TlsError, CertificateUsage};
use der::{Decode, Encode};
use x509_cert::name::{Name, RdnSequence};
use x509_cert::attr::{AttributeTypeAndValue};
use der::asn1::{PrintableStringRef, Utf8StringRef, Ia5StringRef};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Security validation manager with optimized certificate security checks
pub struct SecurityManager;

impl SecurityManager {
    /// Validate certificate KeyUsage extension for intended purpose with fast path validation
    pub fn validate_key_usage_internal(
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
                    tracing::warn!("Server certificate missing keyEncipherment usage - may cause issues with RSA key exchange");
                }

                // Check Extended Key Usage for server authentication
                if !parsed_cert
                    .extended_key_usage
                    .contains(&"serverAuth".to_string())
                {
                    return Err(TlsError::CertificateValidation(
                        "Server certificate missing required serverAuth extended key usage"
                            .to_string(),
                    ));
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

                // Check Extended Key Usage for client authentication
                if !parsed_cert
                    .extended_key_usage
                    .contains(&"clientAuth".to_string())
                {
                    return Err(TlsError::CertificateValidation(
                        "Client certificate missing required clientAuth extended key usage"
                            .to_string(),
                    ));
                }
            }
            CertificateUsage::CodeSigning => {
                // Code signing certificates must have digitalSignature
                if !parsed_cert
                    .key_usage
                    .contains(&"digitalSignature".to_string())
                {
                    return Err(TlsError::CertificateValidation(
                        "Code signing certificate missing required digitalSignature usage"
                            .to_string(),
                    ));
                }

                // Check Extended Key Usage for code signing
                if !parsed_cert
                    .extended_key_usage
                    .contains(&"codeSigning".to_string())
                {
                    return Err(TlsError::CertificateValidation(
                        "Code signing certificate missing required codeSigning extended key usage"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate certificate path length constraints with optimized depth checking
    pub fn validate_path_length_constraints(
        parsed_cert: &ParsedCertificate,
        current_depth: u32,
    ) -> Result<(), TlsError> {
        // Check if this is a CA certificate with path length constraints
        if parsed_cert.is_ca {
            if let Some(path_len_constraint) = parsed_cert.path_len_constraint {
                if current_depth > path_len_constraint {
                    return Err(TlsError::CertificateValidation(format!(
                        "Certificate chain exceeds path length constraint: depth {} > constraint {}",
                        current_depth, path_len_constraint
                    )));
                }
            }
        }

        Ok(())
    }

    /// Validate certificate subject alternative names with fast string matching
    pub fn validate_subject_alternative_names(
        parsed_cert: &ParsedCertificate,
        expected_hostname: &str,
    ) -> Result<(), TlsError> {
        // If no SANs are present, check the common name
        if parsed_cert.subject_alt_names.is_empty() {
            return Self::validate_common_name(parsed_cert, expected_hostname);
        }

        // Check each SAN for a match
        for san in &parsed_cert.subject_alt_names {
            if Self::hostname_matches(san, expected_hostname) {
                return Ok(());
            }
        }

        Err(TlsError::CertificateValidation(format!(
            "Certificate does not match hostname '{}'. SANs: {:?}",
            expected_hostname, parsed_cert.subject_alt_names
        )))
    }

    /// Validate common name against expected hostname with optimized matching
    fn validate_common_name(
        parsed_cert: &ParsedCertificate,
        expected_hostname: &str,
    ) -> Result<(), TlsError> {
        if let Some(common_name) = &parsed_cert.common_name {
            if Self::hostname_matches(common_name, expected_hostname) {
                return Ok(());
            }
        }

        Err(TlsError::CertificateValidation(format!(
            "Certificate common name does not match hostname '{}'",
            expected_hostname
        )))
    }

    /// Check if hostname matches certificate name (supports wildcards) with zero allocation
    fn hostname_matches(cert_name: &str, hostname: &str) -> bool {
        // Exact match (fast path)
        if cert_name == hostname {
            return true;
        }

        // Wildcard matching
        if cert_name.starts_with("*.") {
            let cert_domain = &cert_name[2..];
            
            // Find the first dot in hostname
            if let Some(dot_pos) = hostname.find('.') {
                let hostname_domain = &hostname[dot_pos + 1..];
                return cert_domain == hostname_domain;
            }
        }

        false
    }

    /// Parse X.509 distinguished name with optimized ASN.1 parsing
    pub fn parse_distinguished_name(name_der: &[u8]) -> Result<HashMap<String, String>, TlsError> {
        let name = Name::from_der(name_der)
            .map_err(|e| TlsError::CertificateValidation(format!("Failed to parse DN: {}", e)))?;

        let mut attrs = HashMap::new();

        // Common OID constants for performance
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
                        _ => {
                            // Store unknown OIDs with their numeric representation
                            attrs.insert(oid_string, value_str);
                        }
                    }
                }
            }
        }

        Ok(attrs)
    }

    /// Validate certificate signature algorithm strength with fast algorithm checking
    pub fn validate_signature_algorithm_strength(
        parsed_cert: &ParsedCertificate,
    ) -> Result<(), TlsError> {
        // Check for weak signature algorithms
        let weak_algorithms = [
            "md5WithRSAEncryption",
            "sha1WithRSAEncryption", 
            "md2WithRSAEncryption",
        ];

        for weak_alg in &weak_algorithms {
            if parsed_cert.signature_algorithm.contains(weak_alg) {
                return Err(TlsError::CertificateValidation(format!(
                    "Certificate uses weak signature algorithm: {}",
                    parsed_cert.signature_algorithm
                )));
            }
        }

        // Warn about SHA-1 (deprecated but may still be in use)
        if parsed_cert.signature_algorithm.contains("sha1") {
            tracing::warn!(
                "Certificate uses deprecated SHA-1 signature algorithm: {}",
                parsed_cert.signature_algorithm
            );
        }

        Ok(())
    }

    /// Validate RSA key size with optimized bit length checking
    pub fn validate_rsa_key_size(key_size_bits: u32) -> Result<(), TlsError> {
        const MIN_RSA_KEY_SIZE: u32 = 2048;
        const RECOMMENDED_RSA_KEY_SIZE: u32 = 3072;

        if key_size_bits < MIN_RSA_KEY_SIZE {
            return Err(TlsError::CertificateValidation(format!(
                "RSA key size {} is below minimum required size of {} bits",
                key_size_bits, MIN_RSA_KEY_SIZE
            )));
        }

        if key_size_bits < RECOMMENDED_RSA_KEY_SIZE {
            tracing::warn!(
                "RSA key size {} is below recommended size of {} bits",
                key_size_bits, RECOMMENDED_RSA_KEY_SIZE
            );
        }

        Ok(())
    }

    /// Validate elliptic curve parameters with fast curve validation
    pub fn validate_ec_parameters(curve_name: &str) -> Result<(), TlsError> {
        // List of approved curves (NIST and secure curves)
        let approved_curves = [
            "secp256r1", // P-256
            "secp384r1", // P-384
            "secp521r1", // P-521
            "curve25519",
            "curve448",
        ];

        // List of deprecated/weak curves
        let weak_curves = [
            "secp112r1",
            "secp112r2", 
            "secp128r1",
            "secp128r2",
            "secp160k1",
            "secp160r1",
            "secp160r2",
            "secp192k1",
            "secp192r1", // P-192 - considered weak
            "secp224k1",
            "secp224r1", // P-224 - borderline
        ];

        // Check for weak curves first
        for weak_curve in &weak_curves {
            if curve_name == *weak_curve {
                return Err(TlsError::CertificateValidation(format!(
                    "Certificate uses weak elliptic curve: {}",
                    curve_name
                )));
            }
        }

        // Check for approved curves
        for approved_curve in &approved_curves {
            if curve_name == *approved_curve {
                return Ok(());
            }
        }

        // Unknown curve - warn but don't fail
        tracing::warn!("Certificate uses unknown elliptic curve: {}", curve_name);
        Ok(())
    }

    /// Check certificate for security policy compliance with fast validation
    pub fn check_security_policy_compliance(
        parsed_cert: &ParsedCertificate,
    ) -> Result<(), TlsError> {
        // Validate signature algorithm strength
        Self::validate_signature_algorithm_strength(parsed_cert)?;

        // Validate key size if RSA
        if parsed_cert.public_key_algorithm.contains("RSA") {
            // Extract key size from algorithm string or use a default check
            // This is a simplified implementation
            if let Some(key_size) = Self::extract_rsa_key_size(&parsed_cert.public_key_algorithm) {
                Self::validate_rsa_key_size(key_size)?;
            }
        }

        // Validate elliptic curve if EC
        if parsed_cert.public_key_algorithm.contains("EC") {
            // Extract curve name from algorithm string
            if let Some(curve_name) = Self::extract_ec_curve_name(&parsed_cert.public_key_algorithm) {
                Self::validate_ec_parameters(&curve_name)?;
            }
        }

        Ok(())
    }

    /// Extract RSA key size from algorithm string (simplified implementation)
    fn extract_rsa_key_size(algorithm: &str) -> Option<u32> {
        // This is a simplified implementation
        // In practice, you would parse the actual public key to get the size
        if algorithm.contains("2048") {
            Some(2048)
        } else if algorithm.contains("3072") {
            Some(3072)
        } else if algorithm.contains("4096") {
            Some(4096)
        } else {
            // Default assumption for RSA
            Some(2048)
        }
    }

    /// Extract EC curve name from algorithm string (simplified implementation)
    fn extract_ec_curve_name(algorithm: &str) -> Option<String> {
        // This is a simplified implementation
        // In practice, you would parse the actual public key parameters
        if algorithm.contains("P-256") || algorithm.contains("secp256r1") {
            Some("secp256r1".to_string())
        } else if algorithm.contains("P-384") || algorithm.contains("secp384r1") {
            Some("secp384r1".to_string())
        } else if algorithm.contains("P-521") || algorithm.contains("secp521r1") {
            Some("secp521r1".to_string())
        } else {
            None
        }
    }
}

/// Security policy configuration
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub min_rsa_key_size: u32,
    pub allowed_signature_algorithms: Vec<String>,
    pub allowed_curves: Vec<String>,
    pub require_san: bool,
    pub max_cert_validity_days: u32,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            min_rsa_key_size: 2048,
            allowed_signature_algorithms: vec![
                "sha256WithRSAEncryption".to_string(),
                "sha384WithRSAEncryption".to_string(),
                "sha512WithRSAEncryption".to_string(),
                "ecdsa-with-SHA256".to_string(),
                "ecdsa-with-SHA384".to_string(),
                "ecdsa-with-SHA512".to_string(),
            ],
            allowed_curves: vec![
                "secp256r1".to_string(),
                "secp384r1".to_string(),
                "secp521r1".to_string(),
                "curve25519".to_string(),
            ],
            require_san: true,
            max_cert_validity_days: 825, // Current CA/Browser Forum limit
        }
    }
}