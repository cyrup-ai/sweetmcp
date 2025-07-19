#[cfg(test)]
mod certificate_parsing_tests {
    use rcgen::{CertificateParams, DistinguishedName, DnType, SanType};
    use std::time::SystemTime;
    use sweetmcp::tls::{CertificateUsage, TlsManager};

    // Generate a valid test certificate that mimics Let's Encrypt structure
    fn generate_test_certificate() -> String {
        let mut params = CertificateParams::default();
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(DnType::CommonName, "test.example.com");
        params
            .distinguished_name
            .push(DnType::OrganizationName, "Test Organization");
        params.distinguished_name.push(DnType::CountryName, "US");

        params.subject_alt_names = vec![SanType::DnsName("test.example.com".try_into().unwrap())];

        let cert = params
            .self_signed(&rcgen::KeyPair::generate().unwrap())
            .unwrap();
        cert.pem()
    }

    // Create a real test certificate using rcgen that we know is valid
    fn get_test_certificate() -> String {
        let mut params = CertificateParams::default();
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(DnType::CommonName, "test.example.com");
        params
            .distinguished_name
            .push(DnType::OrganizationName, "Test Organization");
        params.distinguished_name.push(DnType::CountryName, "US");

        params.subject_alt_names = vec![
            SanType::DnsName("test.example.com".try_into().unwrap()),
            SanType::DnsName("www.test.example.com".try_into().unwrap()),
        ];

        let cert = params
            .self_signed(&rcgen::KeyPair::generate().unwrap())
            .unwrap();
        cert.pem()
    }

    // Self-signed certificate for testing
    const SELF_SIGNED_CERT: &str = r#"-----BEGIN CERTIFICATE-----
MIIDazCCAlOgAwIBAgIUJL+6TCAquuXkOqN0SV8MkYR3mIwwDQYJKoZIhvcNAQEL
BQAwRTELMAkGA1UEBhMCVVMxEzARBgNVBAgMClNvbWUtU3RhdGUxITAfBgNVBAoM
GEludGVybmV0IFdpZGdpdHMgUHR5IEx0ZDAeFw0yNDAzMjYxMzAwMDBaFw0yNTAz
MjYxMzAwMDBaMEUxCzAJBgNVBAYTAlVTMRMwEQYDVQQIDApTb21lLVN0YXRlMSEw
HwYDVQQKDBhJbnRlcm5ldCBXaWRnaXRzIFB0eSBMdGQwggEiMA0GCSqGSIb3DQEB
AQUAA4IBDwAwggEKAoIBAQDL9P6YMJqzhL2x3qE8p7Hu4JHubmPSJmBpMRmDm7UD
F9XULRhYz8iNMn4U3yY1L6CI7XRz3kQGVeDBKVvYAGABqUiJREPJ4pW/6KkLQZ8q
vKpU6XkdqZ0Gz4eFqXPQq1Y1V5fJ6f6VqokuGK1W7SEH5zcqDHY6Dp4PpAJKdS4o
aFYJYzFWP8IjAS7rpLyYN9q6dXEJsf8YBBpiT6q8hGqJLzMojhVgKKUmDwOjZw4P
vH1Y7Y0L1c8KkKE7V4oFRlbCq8zYGV9xGEaHWYYOY1jzlB6TH+Yqr0B1ldvM0Lqk
BYiJEtgUM6BDbqvHhvG4xRf6GP0SjPXPZxBG2LvdFNMDAgMBAAGjUzBRMB0GA1Ud
DgQWBBSSk4qXSv4o5CqHw7FKlh6HnH4bQDAfBgNVHSMEGDAWgBSSk4qXSv4o5CqH
w7FKlh6HnH4bQDAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUAA4IBAQCl
q7T5F8Y8bK8rZKbHv9FS3gFXbl8R5zF9HpvCkQGqZ0y8qDC6oD3J9aQ3xV8NSq7m
FgPl6Q9QB7U3bQFgqkK7PbKQEeqoPHpCPUW4vVj2qrPxJFyGQQP1c8Y6UQXSfQcU
FrMWJQWBBrkCL8hGYYtN8pm8zqR+LMJR4rKVqvJNXML2koSJY3F4fWXd9x3VRKvL
bLQQXFSv6jMdsk1FgMWXk1t3gF3J5TgI8lthRJLjBqwvAwQbBvUlUE0PJqHV3KQU
0LckbQEz7HBqeRXUbN8lM3Q7jcD8RqV0YKmFrEBrDOKqJL1hNJnPv1BKXG0H6IFt
MjqUcBq5zfPYcQqLKgkj
-----END CERTIFICATE-----"#;

    // Valid example.com certificate from x509-cert test examples
    const EXAMPLE_CERT: &str = r#"-----BEGIN CERTIFICATE-----
MIIDnDCCAoSgAwIBAgIJAKQzLo3paeO7MA0GCSqGSIb3DQEBCwUAMGQxFDASBgNV
BAMMC2V4YW1wbGUuY29tMRQwEgYDVQQHDAtMb3MgQW5nZWxlczETMBEGA1UECAwK
Q2FsaWZvcm5pYTEUMBIGA1UECgwLRXhhbXBsZSBJbmMxCzAJBgNVBAYTAlVTMB4X
DTIyMDEwODE4NDA1N1oXDTIzMDEwODE4NDA1N1owZDEUMBIGA1UEAwwLZXhhbXBs
ZS5jb20xFDASBgNVBAcMC0xvcyBBbmdlbGVzMRMwEQYDVQQIDApDYWxpZm9ybmlh
MRQwEgYDVQQKDAtFeGFtcGxlIEluYzELMAkGA1UEBhMCVVMwggEiMA0GCSqGSIb3
DQEBAQUAA4IBDwAwggEKAoIBAQC/Wff+cW3eR8c1ecqEbvqNMKs2EuDWpSQgSnLK
jlDJ9FlRPfDXMzG+09ei2no2Jxnkce5qnYeCfRAk7URgWrm0jzuAjF4XO58+xAA9
V/FxhIn1x6BCHEb71SekCrS6a52xalRdHs9uKlYzvYBZTrpK/ucfY+HTV8ZOmj/2
uDdGqIXDc/NSeYfkwrSvf+TU6hZAXl4VKF3ZOII6oY4mNLr+hHp2HK+rsEAdP6A6
B6nQl8uwx3FWzP42Ex2t8cEJwoI5cvCvIaNfNY54gwTAx4uVFznZH6v/0HqozU9p
dGs9DrRYdGn5059PvcdhIA37J9r2lWIxHYsZG37vquL41vjrAgMBAAGjUTBPMAkG
A1UdEwQCMAAwCwYDVR0PBAQDAgWgMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEF
BQcDAjAWBgNVHREEDzANggtleGFtcGxlLmNvbTANBgkqhkiG9w0BAQsFAAOCAQEA
kqvA9M0WRVffA9Eb5h813vio3ceQ8JItVHWyvh9vNGOz3d3eywXIOAKMmzQRQUfY
7WMbjCM9ppTKRmfoFbMnDQb1aa93isuCoo5QRSpX6DmN/p4v3uz79p8m8in+xhKQ
1m6et1iwR9cbQxLsmsaVaVTn16xdsL+gq7V4IZXf8CVyxL0mH5FdRmj/nqiWTv6S
I9tIFiEhCqq1P5XGi6TJAg59M8Dlnd/j5eJHTIlADjG0O1LLvAcuc3rq+dYj0mOU
RX4MzusreyKRGdvr2IN2gYCDPOgOiqp3YKkOnXV8/pya1KSGrT51fEYTdUrjJ6dr
430thqsUED++/t+K76IRMw==
-----END CERTIFICATE-----"#;

    // CA certificate for testing
    #[test]
    fn test_simple_parse() {
        // Minimal test to check if parse_certificate_from_pem is working
        let simple_cert = SELF_SIGNED_CERT;
        let result = TlsManager::parse_certificate_from_pem(simple_cert);
        result.expect("Failed to parse simple certificate");
    }

    #[test]
    fn test_debug_example_cert() {
        let result = TlsManager::parse_certificate_from_pem(EXAMPLE_CERT);
        match result {
            Ok(_parsed) => {
                // Certificate parsed successfully
            }
            Err(e) => {
                assert!(false, "Failed to parse EXAMPLE_CERT: {:?}", e);
            }
        }
    }

    const CA_CERT: &str = r#"-----BEGIN CERTIFICATE-----
MIIFFjCCAv6gAwIBAgIRAJErCErPDBinU/bWLiWnX1owDQYJKoZIhvcNAQELBQAw
TzELMAkGA1UEBhMCVVMxKTAnBgNVBAoTIEludGVybmV0IFNlY3VyaXR5IFJlc2Vh
cmNoIEdyb3VwMRUwEwYDVQQDEwxJU1JHIFJvb3QgWDEwHhcNMjAwOTA0MDAwMDAw
WhcNMjUwOTE1MTYwMDAwWjAyMQswCQYDVQQGEwJVUzEWMBQGA1UEChMNTGV0J3Mg
RW5jcnlwdDELMAkGA1UEAxMCUjMwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEK
AoIBAQC7AhUozPaglNMPEuyNVZLD+ILxmaZ6QoinXSaqtSu5xUyxr45r+XXIo9cP
R5QUVTVXjJ6oojkZ9YI8QqlObvU7wy7bjcCwXPNZOOftz2nwWgsbvsCUJCWH+jdx
sxPnHKzhm+/b5DtFUkWWqcFTzjTIUu61ru2P3mBw4qVUq7ZtDpelQDRrK9O8Zutm
NHz6a4uPVymZ+DAXXbpyb/uBxa3Shlg9F8fnCbvxK/eG3MHacV3URuPMrSXBiLxg
Z3Vms/EY96Jc5lP/Ooi2R6X/ExjqmAl3P51T+c8B5fWmcBcUr2Ok/5mzk53cU6cG
/kiFHaFpriV1uxPMUgP17VGhi9sVAgMBAAGjggEIMIIBBDAOBgNVHQ8BAf8EBAMC
AYYwHQYDVR0lBBYwFAYIKwYBBQUHAwIGCCsGAQUFBwMBMBIGA1UdEwEB/wQIMAYB
Af8CAQAwHQYDVR0OBBYEFBQusxe3WFbLrlAJQOYfr52LFMLGMB8GA1UdIwQYMBaA
FHm0WeZ7tuXkAXOACIjIGlj26ZtuMDIGCCsGAQUFBwEBBCYwJDAiBggrBgEFBQcw
AoYWaHR0cDovL3gxLmkubGVuY3Iub3JnLzAnBgNVHR8EIDAeMBygGqAYhhZodHRw
Oi8veDEuYy5sZW5jci5vcmcvMCIGA1UdIAQbMBkwCAYGZ4EMAQIBMA0GCysGAQQB
gt8TAQEBMA0GCSqGSIb3DQEBCwUAA4ICAQCFyk5HPqP3hUSFvNVneLKYY611TR6W
PTNlclQtgaDqw+34IL9fzLdwALduO/ZelN7kIJ+m74uyA+eitRY8kc607TkC53wl
ikfmZW4/RvTZ8M6UK+5UzhK8jCdLuMGYL6KvzXGRSgi3yLgjewQtCPkIVz6D2QQz
CkcheAmCJ8MqyJu5zlzyZMjAvnnAT45tRAxekrsu94sQ4egdRCnbWSDtY7kh+BIm
lJNXoB1lBMEKIq4QDUOXoRgffuDghje1WrG9ML+Hbisq/yFOGwXD9RiX8F6sw6W4
avAuvDszue5L3sz85K+EC4Y/wFVDNvZo4TYXao6Z0f+lQKc0t8DQYzk1OXVu8rp2
yJMC6alLbBfODALZvYH7n7do1AZls4I9d1P4jnkDrQoxB3UqQ9hVl3LEKQ73xF1O
yK5GhDDX8oVfGKF5u+decIsH4YaTw7mP3GFxJSqv3+0lUFJoi5Lc5da149p90Ids
hCExroL1+7mryIkXPeFM5TgO9r0rvZaBFOvV2z0gp35Z0+L4WPlbuEjN/lxPFin+
HlUjr8gRsI3qfJOQFy/9rKIJR0Y/8Omwt/8oTWgy1mdeHmmjk7j1nYsvC9JSQ6Zv
MldlTTKB3zhThV1+XWYp6rjd5JW1zbVWEkLNxE7GJThEUG3szgBVGP7pSWTUTsqX
nLRbwHOoq7hHwg==
-----END CERTIFICATE-----"#;

    #[test]
    fn test_parse_example_certificate() {
        // Test with the EXAMPLE_CERT we know is valid
        let parsed = TlsManager::parse_certificate_from_pem(EXAMPLE_CERT)
            .expect("Failed to parse EXAMPLE_CERT");

        // Verify basic fields are populated
        assert!(!parsed.serial_number.is_empty());
        assert!(!parsed.subject_der.is_empty());
        assert!(!parsed.public_key_der.is_empty());

        // Verify subject attributes (from our debug output)
        assert_eq!(parsed.subject.get("CN"), Some(&"example.com".to_string()));
        assert_eq!(parsed.subject.get("O"), Some(&"Example Inc".to_string()));
        assert_eq!(parsed.subject.get("C"), Some(&"US".to_string()));
        assert_eq!(parsed.subject.get("ST"), Some(&"California".to_string()));
        assert_eq!(parsed.subject.get("L"), Some(&"Los Angeles".to_string()));

        // This certificate is self-signed, so subject equals issuer
        assert_eq!(parsed.subject, parsed.issuer);
    }

    #[test]
    fn test_parse_self_signed_certificate() {
        let parsed = TlsManager::parse_certificate_from_pem(SELF_SIGNED_CERT)
            .expect("Failed to parse self-signed certificate");

        // Verify subject equals issuer (self-signed)
        assert_eq!(parsed.subject.get("O"), parsed.issuer.get("O"));
        assert_eq!(
            parsed.subject.get("O"),
            Some(&"Internet Widgits Pty Ltd".to_string())
        );
        assert_eq!(parsed.subject.get("C"), Some(&"US".to_string()));
        assert_eq!(parsed.subject.get("ST"), Some(&"Some-State".to_string()));

        // Verify it's a CA (self-signed root)
        assert!(parsed.is_ca);

        // Self-signed certs typically don't have OCSP URLs
        assert!(parsed.ocsp_urls.is_empty());

        // Verify basic certificate properties
        assert!(!parsed.serial_number.is_empty());
        assert!(!parsed.subject_der.is_empty());
        assert!(!parsed.public_key_der.is_empty());
    }

    #[test]
    fn test_parse_ca_certificate() {
        let parsed = TlsManager::parse_certificate_from_pem(CA_CERT)
            .expect("Failed to parse CA certificate");

        // Verify it's a CA
        assert!(parsed.is_ca);

        // Verify subject
        assert_eq!(parsed.subject.get("CN"), Some(&"R3".to_string()));
        assert_eq!(parsed.subject.get("O"), Some(&"Let's Encrypt".to_string()));

        // Verify issuer (ISRG Root X1)
        assert_eq!(parsed.issuer.get("CN"), Some(&"ISRG Root X1".to_string()));

        // Verify CA key usage
        assert!(parsed.key_usage.contains(&"keyCertSign".to_string()));
        assert!(parsed.key_usage.contains(&"digitalSignature".to_string()));

        // CA should have CRL distribution points
        assert!(!parsed.crl_urls.is_empty());
        assert!(parsed.crl_urls[0].starts_with("http://x1.c.lencr.org"));
    }

    #[test]
    fn test_parse_malformed_certificate() {
        let malformed =
            "-----BEGIN CERTIFICATE-----\nINVALID BASE64 DATA\n-----END CERTIFICATE-----";

        let result = TlsManager::parse_certificate_from_pem(malformed);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(
                matches!(e, sweetmcp::tls::TlsError::CertificateParsing(_)),
                "Expected CertificateParsing error, got: {:?}",
                e
            );
        }
    }

    #[test]
    fn test_parse_empty_pem() {
        let empty = "";
        let result = TlsManager::parse_certificate_from_pem(empty);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_private_key_instead_of_cert() {
        let private_key = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC7AhUozPaglNMP
-----END PRIVATE KEY-----"#;

        let result = TlsManager::parse_certificate_from_pem(private_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_certificate_time_validation() {
        let parsed = TlsManager::parse_certificate_from_pem(EXAMPLE_CERT)
            .expect("Failed to parse certificate");

        // This cert was valid in 2022-2023
        let jan_2022 = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1641636000); // Jan 8, 2022
        let feb_2023 = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1675900800); // Feb 8, 2023
        assert!(jan_2022 < parsed.not_before);
        assert!(feb_2023 > parsed.not_after);
    }

    #[test]
    fn test_basic_constraints_validation() {
        // Test CA certificate
        let ca_parsed = TlsManager::parse_certificate_from_pem(CA_CERT)
            .expect("Failed to parse CA certificate");
        assert!(TlsManager::validate_basic_constraints(&ca_parsed, true).is_ok());
        assert!(TlsManager::validate_basic_constraints(&ca_parsed, false).is_err());

        // Test end-entity certificate
        let ee_parsed = TlsManager::parse_certificate_from_pem(EXAMPLE_CERT)
            .expect("Failed to parse end-entity certificate");
        assert!(TlsManager::validate_basic_constraints(&ee_parsed, false).is_ok());
        assert!(TlsManager::validate_basic_constraints(&ee_parsed, true).is_err());
    }

    #[test]
    fn test_key_usage_validation() {
        // Test server certificate
        let server_parsed = TlsManager::parse_certificate_from_pem(EXAMPLE_CERT)
            .expect("Failed to parse server certificate");
        assert!(
            TlsManager::validate_key_usage(&server_parsed, CertificateUsage::ServerAuth).is_ok()
        );
        assert!(TlsManager::validate_key_usage(
            &server_parsed,
            CertificateUsage::CertificateAuthority
        )
        .is_err());

        // Test CA certificate
        let ca_parsed = TlsManager::parse_certificate_from_pem(CA_CERT)
            .expect("Failed to parse CA certificate");
        assert!(
            TlsManager::validate_key_usage(&ca_parsed, CertificateUsage::CertificateAuthority)
                .is_ok()
        );
    }

    #[test]
    fn test_san_extraction() {
        let parsed = TlsManager::parse_certificate_from_pem(EXAMPLE_CERT)
            .expect("Failed to parse certificate");

        // Verify that the certificate has a Common Name
        assert_eq!(parsed.subject.get("CN"), Some(&"example.com".to_string()));

        // This cert doesn't have IP SANs
        assert!(parsed.san_ip_addresses.is_empty());
    }

    #[test]
    fn test_extension_url_extraction() {
        let parsed = TlsManager::parse_certificate_from_pem(EXAMPLE_CERT)
            .expect("Failed to parse certificate");

        // The example cert is self-signed and may not have OCSP URLs
        // This test will verify our parsing works without errors
        // OCSP URLs may be empty for self-signed certs

        // Let's Encrypt CA cert should have CRL URLs
        let ca_parsed = TlsManager::parse_certificate_from_pem(CA_CERT)
            .expect("Failed to parse CA certificate");
        assert!(!ca_parsed.crl_urls.is_empty());
        assert!(ca_parsed
            .crl_urls
            .iter()
            .any(|url| url.contains("lencr.org")));
    }
}
