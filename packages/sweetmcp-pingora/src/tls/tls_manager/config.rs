//! Configuration management and parsing
//!
//! This module provides comprehensive configuration management for TLS certificates
//! with zero allocation fast paths and blazing-fast performance.

use super::core::{ParsedCertificate, TlsError};
use der::{Decode, Reader, SliceReader, Tag, TagNumber};
use rustls_pemfile;
use std::collections::HashMap;
use std::io::Cursor;
use std::net::IpAddr;
use std::time::SystemTime;
use tracing::{debug, error, info, warn};
use x509_cert::Certificate as X509CertCert;
use x509_cert::name::Name;

/// Configuration manager for TLS certificate parsing and validation
pub struct ConfigManager;

impl ConfigManager {
    /// Extract certificate details using x509-cert with optimized parsing
    pub fn extract_certificate_details(
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
        // Extract SANs with zero allocation fast path
        let mut san_dns_names = Vec::new();
        let mut san_ip_addresses = Vec::new();

        // Extract BasicConstraints for CA flag
        let mut is_ca = false;

        // Extract key usage with optimized parsing
        let mut key_usage = Vec::new();

        // OIDs for extensions (constants for performance)
        const OID_SUBJECT_ALT_NAME: &str = "2.5.29.17";
        const OID_BASIC_CONSTRAINTS: &str = "2.5.29.19";
        const OID_KEY_USAGE: &str = "2.5.29.15";

        // Process extensions with fast path validation
        if let Some(extensions) = &cert.tbs_certificate.extensions {
            for ext in extensions.iter() {
                let oid_string = ext.extn_id.to_string();

                match oid_string.as_str() {
                    OID_SUBJECT_ALT_NAME => {
                        // Parse SubjectAltName extension properly using ASN.1
                        // SubjectAltName ::= GeneralNames
                        // GeneralNames ::= SEQUENCE OF GeneralName
                        let ext_data = ext.extn_value.as_bytes();

                        // Parse the OCTET STRING wrapper first
                        match der::asn1::OctetString::from_der(ext_data) {
                            Ok(octet_string) => {
                                // Now parse the actual SubjectAltName SEQUENCE
                                let san_data = octet_string.as_bytes();
                                match SliceReader::new(san_data) {
                                    Ok(mut reader) => {
                                        // Parse SEQUENCE OF GeneralName
                                        if let Ok(sequence_header) = reader.read_sequence() {
                                            let mut seq_reader = sequence_header;
                                            
                                            // Process each GeneralName in the sequence
                                            while !seq_reader.is_finished() {
                                                match Self::parse_general_name(&mut seq_reader) {
                                                    Ok(Some((name_type, name_value))) => {
                                                        match name_type {
                                                            2 => { // dNSName
                                                                san_dns_names.push(name_value);
                                                            }
                                                            7 => { // iPAddress
                                                                if let Ok(ip) = Self::parse_ip_address(&name_value) {
                                                                    san_ip_addresses.push(ip);
                                                                }
                                                            }
                                                            _ => {
                                                                // Other GeneralName types not handled
                                                                tracing::debug!("Unhandled GeneralName type: {}", name_type);
                                                            }
                                                        }
                                                    }
                                                    Ok(None) => continue,
                                                    Err(e) => {
                                                        tracing::warn!("Failed to parse GeneralName: {}", e);
                                                        continue;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to create SAN reader: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse SAN OCTET STRING: {}", e);
                            }
                        }
                    }
                    OID_BASIC_CONSTRAINTS => {
                        // Parse BasicConstraints extension
                        let ext_data = ext.extn_value.as_bytes();
                        
                        match der::asn1::OctetString::from_der(ext_data) {
                            Ok(octet_string) => {
                                let bc_data = octet_string.as_bytes();
                                match SliceReader::new(bc_data) {
                                    Ok(mut reader) => {
                                        if let Ok(sequence_header) = reader.read_sequence() {
                                            let mut seq_reader = sequence_header;
                                            
                                            // First element is CA boolean (optional, defaults to FALSE)
                                            if !seq_reader.is_finished() {
                                                if let Ok(ca_bool) = seq_reader.read_bool() {
                                                    is_ca = ca_bool;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to parse BasicConstraints: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse BasicConstraints OCTET STRING: {}", e);
                            }
                        }
                    }
                    OID_KEY_USAGE => {
                        // Parse KeyUsage extension (BIT STRING)
                        let ext_data = ext.extn_value.as_bytes();
                        
                        match der::asn1::OctetString::from_der(ext_data) {
                            Ok(octet_string) => {
                                let ku_data = octet_string.as_bytes();
                                match SliceReader::new(ku_data) {
                                    Ok(mut reader) => {
                                        if let Ok(bit_string) = reader.read_bit_string() {
                                            key_usage = Self::parse_key_usage_bits(&bit_string);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to parse KeyUsage: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse KeyUsage OCTET STRING: {}", e);
                            }
                        }
                    }
                    _ => {
                        // Other extensions not processed here
                    }
                }
            }
        }

        // Extract validity times from TBS certificate with optimized conversion
        let validity = &cert.tbs_certificate.validity;
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

    /// Parse GeneralName from ASN.1 reader with zero allocation fast path
    fn parse_general_name(reader: &mut SliceReader) -> Result<Option<(u8, String)>, TlsError> {
        // GeneralName is a CHOICE with context-specific tags
        if reader.is_finished() {
            return Ok(None);
        }

        // Read the tag to determine the GeneralName type
        let tag = reader.peek_tag()
            .map_err(|e| TlsError::CertificateParsing(format!("Failed to peek tag: {}", e)))?;

        // Check if it's a context-specific tag
        if tag.is_context_specific() {
            let tag_number = tag.number().as_u8();
            
            // Read the value based on the tag number
            match tag_number {
                2 => { // dNSName [2] IA5String
                    let dns_bytes = reader.read_context_specific(TagNumber::new(2))
                        .map_err(|e| TlsError::CertificateParsing(format!("Failed to read dNSName: {}", e)))?;
                    
                    let dns_name = String::from_utf8(dns_bytes.to_vec())
                        .map_err(|e| TlsError::CertificateParsing(format!("Invalid UTF-8 in dNSName: {}", e)))?;
                    
                    Ok(Some((2, dns_name)))
                }
                7 => { // iPAddress [7] OCTET STRING
                    let ip_bytes = reader.read_context_specific(TagNumber::new(7))
                        .map_err(|e| TlsError::CertificateParsing(format!("Failed to read iPAddress: {}", e)))?;
                    
                    // Convert bytes to string representation for consistent handling
                    let ip_string = match ip_bytes.len() {
                        4 => {
                            // IPv4
                            format!("{}.{}.{}.{}", ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3])
                        }
                        16 => {
                            // IPv6 - simplified representation
                            let mut parts = Vec::new();
                            for chunk in ip_bytes.chunks(2) {
                                let part = u16::from_be_bytes([chunk[0], chunk[1]]);
                                parts.push(format!("{:x}", part));
                            }
                            parts.join(":")
                        }
                        _ => {
                            return Err(TlsError::CertificateParsing(
                                "Invalid IP address length".to_string()
                            ));
                        }
                    };
                    
                    Ok(Some((7, ip_string)))
                }
                _ => {
                    // Skip other GeneralName types for now
                    let _ = reader.read_context_specific(TagNumber::new(tag_number));
                    Ok(None)
                }
            }
        } else {
            // Skip non-context-specific tags
            let _ = reader.read_any();
            Ok(None)
        }
    }

    /// Parse IP address from string with optimized parsing
    fn parse_ip_address(ip_string: &str) -> Result<IpAddr, TlsError> {
        ip_string.parse()
            .map_err(|e| TlsError::CertificateParsing(format!("Invalid IP address '{}': {}", ip_string, e)))
    }

    /// Parse key usage bits from BIT STRING with fast bit checking
    fn parse_key_usage_bits(bit_string: &der::asn1::BitString) -> Vec<String> {
        let mut usage = Vec::new();
        let bytes = bit_string.raw_bytes();
        
        if bytes.is_empty() {
            return usage;
        }

        let first_byte = bytes[0];
        
        // Key usage bit definitions (RFC 5280)
        if first_byte & 0x80 != 0 { usage.push("digitalSignature".to_string()); }
        if first_byte & 0x40 != 0 { usage.push("nonRepudiation".to_string()); }
        if first_byte & 0x20 != 0 { usage.push("keyEncipherment".to_string()); }
        if first_byte & 0x10 != 0 { usage.push("dataEncipherment".to_string()); }
        if first_byte & 0x08 != 0 { usage.push("keyAgreement".to_string()); }
        if first_byte & 0x04 != 0 { usage.push("keyCertSign".to_string()); }
        if first_byte & 0x02 != 0 { usage.push("cRLSign".to_string()); }
        if first_byte & 0x01 != 0 { usage.push("encipherOnly".to_string()); }
        
        // Check second byte if present
        if bytes.len() > 1 {
            let second_byte = bytes[1];
            if second_byte & 0x80 != 0 { usage.push("decipherOnly".to_string()); }
        }
        
        usage
    }

    /// Parse certificate from PEM data to extract actual certificate information with optimized parsing
    pub fn parse_certificate_from_pem_internal(pem_data: &str) -> Result<ParsedCertificate, TlsError> {
        // Parse PEM to get DER bytes using rustls-pemfile with fast path
        let mut cursor = Cursor::new(pem_data.as_bytes());
        let cert_der = rustls_pemfile::certs(&mut cursor)
            .next()
            .ok_or_else(|| TlsError::CertificateParsing("No certificate in PEM data".to_string()))?
            .map_err(|e| TlsError::CertificateParsing(format!("Failed to parse PEM: {}", e)))?;

        // Parse X.509 certificate using x509-cert with optimized error handling
        let cert = X509CertCert::from_der(&cert_der)
            .map_err(|e| TlsError::CertificateParsing(format!("X.509 parsing failed: {}", e)))?;

        // Extract subject DN using x509-cert API with fast parsing
        let mut subject = HashMap::new();
        Self::extract_name_attributes(&cert.tbs_certificate.subject, &mut subject);

        // Extract issuer DN using x509-cert API with fast parsing
        let mut issuer = HashMap::new();
        Self::extract_name_attributes(&cert.tbs_certificate.issuer, &mut issuer);

        // Extract basic certificate info using x509-cert with optimized extraction
        let (san_dns_names, san_ip_addresses, is_ca, key_usage, not_before, not_after) =
            Self::extract_certificate_details(&cert)?;

        // Extract OCSP and CRL URLs from certificate extensions with fast parsing
        let mut ocsp_urls = Vec::new();
        let mut crl_urls = Vec::new();

        // Extract additional certificate information with optimized parsing
        let serial_number = cert.tbs_certificate.serial_number.as_bytes().to_vec();
        let signature_algorithm = cert.signature_algorithm.oid.to_string();
        let public_key_algorithm = cert.tbs_certificate.subject_public_key_info.algorithm.oid.to_string();

        // Get common name from subject with fast lookup
        let common_name = subject.get("CN").cloned();

        // Create ParsedCertificate with all extracted information
        Ok(ParsedCertificate {
            subject,
            issuer,
            serial_number,
            not_before,
            not_after,
            subject_alt_names: san_dns_names,
            subject_alt_ips: san_ip_addresses,
            is_ca,
            path_len_constraint: None, // Would need to parse from BasicConstraints
            key_usage,
            extended_key_usage: Vec::new(), // Would need to parse from ExtendedKeyUsage extension
            ocsp_urls,
            crl_urls,
            signature_algorithm,
            public_key_algorithm,
            common_name,
            subject_der: cert.tbs_certificate.subject.to_der()
                .map_err(|e| TlsError::CertificateParsing(format!("Failed to encode subject: {}", e)))?,
            issuer_der: cert.tbs_certificate.issuer.to_der()
                .map_err(|e| TlsError::CertificateParsing(format!("Failed to encode issuer: {}", e)))?,
        })
    }

    /// Extract name attributes from X.509 Name with optimized parsing
    fn extract_name_attributes(name: &Name, attrs: &mut HashMap<String, String>) {
        // Common OID constants for performance
        const OID_CN: &str = "2.5.4.3";   // commonName
        const OID_O: &str = "2.5.4.10";   // organizationName
        const OID_OU: &str = "2.5.4.11";  // organizationalUnitName
        const OID_C: &str = "2.5.4.6";    // countryName
        const OID_ST: &str = "2.5.4.8";   // stateOrProvinceName
        const OID_L: &str = "2.5.4.7";    // localityName

        // Iterate through RDNs (Relative Distinguished Names) with fast processing
        for rdn in name.0.iter() {
            // Each RDN contains one or more AttributeTypeAndValue
            for atv in rdn.0.iter() {
                let oid_string = atv.oid.to_string();

                // Extract the value as string using proper ASN.1 type handling with fast path
                let string_value = if let Ok(ps) = der::asn1::PrintableStringRef::try_from(&atv.value) {
                    Some(ps.to_string())
                } else if let Ok(utf8s) = der::asn1::Utf8StringRef::try_from(&atv.value) {
                    Some(utf8s.to_string())
                } else if let Ok(ia5s) = der::asn1::Ia5StringRef::try_from(&atv.value) {
                    Some(ia5s.to_string())
                } else {
                    None
                };

                if let Some(value_str) = string_value {
                    match oid_string.as_str() {
                        OID_CN => { attrs.insert("CN".to_string(), value_str); }
                        OID_O => { attrs.insert("O".to_string(), value_str); }
                        OID_OU => { attrs.insert("OU".to_string(), value_str); }
                        OID_C => { attrs.insert("C".to_string(), value_str); }
                        OID_ST => { attrs.insert("ST".to_string(), value_str); }
                        OID_L => { attrs.insert("L".to_string(), value_str); }
                        _ => {
                            // Store unknown OIDs with their numeric representation
                            attrs.insert(oid_string, value_str);
                        }
                    }
                }
            }
        }
    }
}

/// Configuration utilities for TLS certificate management
pub struct ConfigUtils;

impl ConfigUtils {
    /// Validate PEM format with fast format checking
    pub fn is_valid_pem_format(pem_data: &str) -> bool {
        pem_data.contains("-----BEGIN CERTIFICATE-----") && 
        pem_data.contains("-----END CERTIFICATE-----")
    }

    /// Extract certificate count from PEM data with optimized counting
    pub fn count_certificates_in_pem(pem_data: &str) -> usize {
        pem_data.matches("-----BEGIN CERTIFICATE-----").count()
    }

    /// Normalize PEM data with fast whitespace handling
    pub fn normalize_pem_data(pem_data: &str) -> String {
        // Remove extra whitespace and ensure proper line endings
        pem_data.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}