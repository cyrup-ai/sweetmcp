//! Linux code signing implementation using GPG

use super::{PlatformConfig, SigningConfig};
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;
use which::which;

/// Sign a binary on Linux using GPG
pub fn sign(config: &SigningConfig) -> Result<()> {
    let PlatformConfig::Linux { key_id, detached } = &config.platform else {
        bail!("Invalid platform config for Linux");
    };

    // Find gpg command
    let gpg = find_gpg()?;

    // Build GPG arguments
    let mut args = vec![];

    if *detached {
        args.push("--detach-sign".to_string());
        args.push("--armor".to_string()); // ASCII armored output
    } else {
        args.push("--sign".to_string());
    }

    // Add key ID if specified
    if let Some(key) = key_id {
        args.push("--local-user".to_string());
        args.push(key.clone());
    }

    // Output file
    let sig_path = if *detached {
        config.binary_path.with_extension("sig")
    } else {
        config.output_path.with_extension("gpg")
    };

    args.push("--output".to_string());
    args.push(sig_path.to_string_lossy().to_string());

    // Input file
    args.push(config.binary_path.to_string_lossy().to_string());

    // Execute GPG
    let output = Command::new(&gpg)
        .args(&args)
        .output()
        .context("Failed to execute GPG")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("GPG signing failed: {}", stderr);
    }

    if *detached {
        println!(
            "Successfully created detached signature: {}",
            sig_path.display()
        );
    } else {
        println!("Successfully signed binary: {}", sig_path.display());
    }

    Ok(())
}

/// Verify a signed binary on Linux
pub fn verify(binary_path: &Path) -> Result<bool> {
    let gpg = match find_gpg() {
        Ok(cmd) => cmd,
        Err(_) => return Ok(false), // Can't verify without GPG
    };

    // Check for detached signature first
    let sig_path = binary_path.with_extension("sig");
    if sig_path.exists() {
        // Verify detached signature
        let output = Command::new(&gpg)
            .args(&[
                "--verify",
                sig_path.to_str().unwrap(),
                binary_path.to_str().unwrap(),
            ])
            .output()
            .context("Failed to execute GPG verify")?;

        return Ok(output.status.success());
    }

    // Check for inline signed file
    let gpg_path = binary_path.with_extension("gpg");
    if gpg_path.exists() {
        let output = Command::new(&gpg)
            .args(&["--verify", gpg_path.to_str().unwrap()])
            .output()
            .context("Failed to execute GPG verify")?;

        return Ok(output.status.success());
    }

    // No signature found
    Ok(false)
}

/// Find GPG command on the system
fn find_gpg() -> Result<String> {
    // Try gpg2 first, then gpg
    if let Ok(gpg2) = which("gpg2") {
        return Ok(gpg2.to_string_lossy().to_string());
    }

    if let Ok(gpg) = which("gpg") {
        return Ok(gpg.to_string_lossy().to_string());
    }

    bail!("GPG not found. Please install gpg or gpg2");
}

/// Create AppImage signature
pub fn sign_appimage(appimage_path: &Path, key_id: Option<&str>) -> Result<()> {
    // Set environment variables for AppImage signing
    std::env::set_var("SIGN", "1");

    if let Some(key) = key_id {
        std::env::set_var("SIGN_KEY", key);
    }

    // The actual signing happens during AppImage creation
    // This is just a helper to set up the environment

    println!(
        "AppImage signing configured. The signature will be embedded during AppImage creation."
    );
    Ok(())
}

/// Generate a new GPG key for signing
pub fn generate_signing_key(name: &str, email: &str) -> Result<String> {
    let gpg = find_gpg()?;

    // Create key generation script
    let key_script = format!(
        r#"
Key-Type: RSA
Key-Length: 4096
Subkey-Type: RSA
Subkey-Length: 4096
Name-Real: {}
Name-Email: {}
Expire-Date: 2y
%no-protection
%commit
"#,
        name, email
    );

    // Execute GPG with batch mode
    let output = Command::new(&gpg)
        .args(&["--batch", "--generate-key"])
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn GPG")?;

    // Write key script to stdin
    use std::io::Write;
    if let Some(mut stdin) = output.stdin {
        stdin
            .write_all(key_script.as_bytes())
            .context("Failed to write key generation script")?;
    }

    let output = output
        .wait_with_output()
        .context("Failed to generate GPG key")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to generate GPG key: {}", stderr);
    }

    // Get the key ID
    let list_output = Command::new(&gpg)
        .args(&["--list-secret-keys", "--keyid-format", "LONG", email])
        .output()
        .context("Failed to list GPG keys")?;

    let stdout = String::from_utf8_lossy(&list_output.stdout);

    // Parse key ID from output
    let key_id = stdout
        .lines()
        .find(|line| line.contains("sec"))
        .and_then(|line| line.split('/').nth(1))
        .and_then(|s| s.split_whitespace().next())
        .ok_or_else(|| anyhow::anyhow!("Failed to parse generated key ID"))?
        .to_string();

    println!("Generated GPG key: {}", key_id);
    Ok(key_id)
}

/// Export GPG public key for distribution
pub fn export_public_key(key_id: &str, output_path: &Path) -> Result<()> {
    let gpg = find_gpg()?;

    let output = Command::new(&gpg)
        .args(&[
            "--armor",
            "--export",
            key_id,
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .context("Failed to export GPG public key")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to export public key: {}", stderr);
    }

    println!("Exported public key to: {}", output_path.display());
    Ok(())
}
