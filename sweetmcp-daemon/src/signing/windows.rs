//! Windows code signing implementation

use super::{PlatformConfig, SigningConfig};
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use std::path::Path;
use std::process::Command;
use which::which;

/// Sign a binary on Windows using signtool
pub fn sign(config: &SigningConfig) -> Result<()> {
    let PlatformConfig::Windows {
        certificate,
        password,
        timestamp_url,
        digest_algorithm,
    } = &config.platform
    else {
        bail!("Invalid platform config for Windows");
    };

    // Find signtool.exe
    let signtool = find_signtool()?;

    // Build signtool arguments
    let mut args = vec!["sign".to_string()];

    // Certificate handling
    if certificate.ends_with(".pfx") || certificate.ends_with(".p12") {
        // File-based certificate
        args.push("/f".to_string());
        args.push(certificate.clone());

        if let Some(pwd) = password {
            args.push("/p".to_string());
            args.push(pwd.clone());
        }
    } else if !certificate.is_empty() {
        // Thumbprint
        args.push("/sha1".to_string());
        args.push(certificate.clone());
    } else {
        // Auto-select certificate
        args.push("/a".to_string());
    }

    // Timestamp server
    args.push("/tr".to_string());
    args.push(timestamp_url.clone());

    // Digest algorithms
    args.push("/td".to_string());
    args.push(digest_algorithm.clone());
    args.push("/fd".to_string());
    args.push(digest_algorithm.clone());

    // Verbose output
    args.push("/v".to_string());

    // Binary to sign
    args.push(config.binary_path.to_string_lossy().to_string());

    // Execute signtool
    let output = Command::new(&signtool)
        .args(&args)
        .output()
        .context("Failed to execute signtool")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!(
            "Code signing failed:\nSTDOUT: {}\nSTDERR: {}",
            stdout,
            stderr
        );
    }

    println!("Successfully signed {}", config.binary_path.display());
    Ok(())
}

/// Verify a signed binary on Windows
pub fn verify(binary_path: &Path) -> Result<bool> {
    let signtool = match find_signtool() {
        Ok(tool) => tool,
        Err(_) => return Ok(false), // Can't verify without signtool
    };

    let output = Command::new(&signtool)
        .args(&[
            "verify",
            "/pa", // Default Authenticode verification
            "/v",  // Verbose
            binary_path.to_str().unwrap(),
        ])
        .output()
        .context("Failed to execute signtool verify")?;

    Ok(output.status.success())
}

/// Find signtool.exe on the system
fn find_signtool() -> Result<String> {
    // First, check if signtool is in PATH
    if let Ok(signtool) = which("signtool.exe") {
        return Ok(signtool.to_string_lossy().to_string());
    }

    // Search in Windows SDK locations
    let sdk_paths = vec![
        r"C:\Program Files (x86)\Windows Kits\10\bin",
        r"C:\Program Files\Windows Kits\10\bin",
        r"C:\Program Files (x86)\Windows Kits\8.1\bin",
        r"C:\Program Files\Windows Kits\8.1\bin",
    ];

    for sdk_path in sdk_paths {
        let sdk_dir = Path::new(sdk_path);
        if sdk_dir.exists() {
            // Look for versioned directories
            if let Ok(entries) = std::fs::read_dir(sdk_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Check x64 and x86 subdirectories
                        for arch in &["x64", "x86"] {
                            let signtool_path = path.join(arch).join("signtool.exe");
                            if signtool_path.exists() {
                                return Ok(signtool_path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }

            // Also check direct paths
            for arch in &["x64", "x86"] {
                let signtool_path = sdk_dir.join(arch).join("signtool.exe");
                if signtool_path.exists() {
                    return Ok(signtool_path.to_string_lossy().to_string());
                }
            }
        }
    }

    bail!("signtool.exe not found. Please install Windows SDK or add signtool to PATH");
}

/// Sign using Azure Code Signing (Trusted Signing)
pub fn sign_with_azure(
    config: &SigningConfig,
    endpoint: &str,
    account: &str,
    profile: &str,
) -> Result<()> {
    // Check if trusted-signing-cli is available
    let tsc = which("trusted-signing-cli")
        .or_else(|_| which("trusted-signing-cli.exe"))
        .context(
            "trusted-signing-cli not found. Install with: cargo install trusted-signing-cli",
        )?;

    let description = format!("SweetMCP Daemon v{}", env!("CARGO_PKG_VERSION"));

    let output = Command::new(tsc)
        .args(&[
            "-e",
            endpoint,
            "-a",
            account,
            "-c",
            profile,
            "-d",
            &description,
            config.binary_path.to_str().unwrap(),
        ])
        .output()
        .context("Failed to execute trusted-signing-cli")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Azure code signing failed: {}", stderr);
    }

    println!("Successfully signed with Azure Code Signing");
    Ok(())
}

/// Import a certificate from base64 string (for CI/CD)
pub fn import_certificate_from_base64(base64_cert: &str, password: &str) -> Result<String> {
    use std::io::Write;

    // Decode base64
    let cert_data = general_purpose::STANDARD
        .decode(base64_cert)
        .context("Failed to decode base64 certificate")?;

    // Write to temporary file
    let temp_dir = std::env::temp_dir();
    let cert_path = temp_dir.join("temp_cert.pfx");

    let mut file =
        std::fs::File::create(&cert_path).context("Failed to create temporary certificate file")?;

    file.write_all(&cert_data)
        .context("Failed to write certificate data")?;

    // Import to certificate store
    let output = Command::new("certutil")
        .args(&[
            "-f",
            "-p",
            password,
            "-importpfx",
            cert_path.to_str().unwrap(),
        ])
        .output()
        .context("Failed to import certificate")?;

    // Clean up temp file
    let _ = std::fs::remove_file(&cert_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to import certificate: {}", stderr);
    }

    // Get thumbprint from output
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse thumbprint from certutil output
    let thumbprint = stdout
        .lines()
        .find(|line| line.contains("Cert Hash(sha1):"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim().replace(' ', ""))
        .ok_or_else(|| anyhow::anyhow!("Failed to parse certificate thumbprint"))?;

    Ok(thumbprint)
}
