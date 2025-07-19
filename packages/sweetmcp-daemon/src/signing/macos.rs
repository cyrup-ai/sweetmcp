//! macOS code signing implementation

use super::{PlatformConfig, SigningConfig};
use anyhow::{bail, Context, Result};
use std::env;
use std::path::Path;
use std::process::Command;

/// Sign a binary on macOS using codesign
pub fn sign(config: &SigningConfig) -> Result<()> {
    let PlatformConfig::MacOS {
        identity,
        team_id: _,
        apple_id,
        apple_password,
        entitlements,
    } = &config.platform
    else {
        bail!("Invalid platform config for macOS");
    };

    // Prepare codesign arguments
    let mut args = vec![
        "--force".to_string(),
        "--sign".to_string(),
        identity.clone(),
        "--options".to_string(),
        "runtime".to_string(),
        "--timestamp".to_string(),
        "--deep".to_string(),
        "--verbose".to_string(),
    ];

    // Add entitlements if provided
    if let Some(entitlements_path) = entitlements {
        args.push("--entitlements".to_string());
        args.push(entitlements_path.to_string_lossy().to_string());
    }

    // Add the binary path
    args.push(config.binary_path.to_string_lossy().to_string());

    // Execute codesign
    let output = Command::new("codesign")
        .args(&args)
        .output()
        .context("Failed to execute codesign")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Code signing failed: {}", stderr);
    }

    println!("Successfully signed {}", config.binary_path.display());

    // If Apple ID and password are provided, notarize the binary
    if apple_id.is_some() && apple_password.is_some() {
        notarize(config)?;
    }

    Ok(())
}

/// Verify a signed binary on macOS
pub fn verify(binary_path: &Path) -> Result<bool> {
    let output = Command::new("codesign")
        .args(&[
            "--verify",
            "--deep",
            "--strict",
            "--verbose=2",
            binary_path.to_str().unwrap(),
        ])
        .output()
        .context("Failed to execute codesign verify")?;

    if output.status.success() {
        // Additional check for valid signature
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(!stderr.contains("code object is not signed at all"))
    } else {
        Ok(false)
    }
}

/// Notarize a signed binary with Apple
fn notarize(config: &SigningConfig) -> Result<()> {
    let PlatformConfig::MacOS {
        team_id,
        apple_id,
        apple_password,
        ..
    } = &config.platform
    else {
        bail!("Invalid platform config for macOS");
    };

    let apple_id = apple_id
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Apple ID required for notarization"))?;

    let apple_password = apple_password
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Apple password required for notarization"))?;

    // Create a ZIP file for notarization
    let zip_path = config.binary_path.with_extension("zip");

    let output = Command::new("ditto")
        .args(&[
            "-c",
            "-k",
            "--keepParent",
            config.binary_path.to_str().unwrap(),
            zip_path.to_str().unwrap(),
        ])
        .output()
        .context("Failed to create ZIP for notarization")?;

    if !output.status.success() {
        bail!(
            "Failed to create ZIP: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Submit for notarization using notarytool
    let mut notarize_args = vec![
        "notarytool".to_string(),
        "submit".to_string(),
        zip_path.to_string_lossy().to_string(),
        "--apple-id".to_string(),
        apple_id.clone(),
        "--password".to_string(),
        apple_password.clone(),
        "--wait".to_string(),
    ];

    if let Some(tid) = team_id {
        notarize_args.push("--team-id".to_string());
        notarize_args.push(tid.clone());
    }

    let output = Command::new("xcrun")
        .args(&notarize_args)
        .output()
        .context("Failed to submit for notarization")?;

    // Clean up ZIP file
    let _ = std::fs::remove_file(&zip_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Notarization failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check if notarization was successful
    if stdout.contains("status: Accepted") {
        println!("Notarization successful!");

        // Staple the notarization ticket
        staple_ticket(&config.binary_path)?;
    } else {
        bail!("Notarization was not accepted: {}", stdout);
    }

    Ok(())
}

/// Staple the notarization ticket to the binary
fn staple_ticket(binary_path: &Path) -> Result<()> {
    let output = Command::new("xcrun")
        .args(&["stapler", "staple", binary_path.to_str().unwrap()])
        .output()
        .context("Failed to staple notarization ticket")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to staple ticket: {}", stderr);
    }

    println!("Successfully stapled notarization ticket");
    Ok(())
}

/// Import a certificate from file for signing
pub fn import_certificate(cert_path: &Path, password: Option<&str>) -> Result<String> {
    // Create temporary keychain
    let keychain_name = format!("sweetmcp-signing-{}.keychain", std::process::id());
    let keychain_password = "temporary-keychain-password";

    // Create keychain
    let output = Command::new("security")
        .args(&["create-keychain", "-p", keychain_password, &keychain_name])
        .output()
        .context("Failed to create temporary keychain")?;

    if !output.status.success() {
        bail!(
            "Failed to create keychain: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Set keychain settings
    Command::new("security")
        .args(&["set-keychain-settings", "-t", "3600", "-u", &keychain_name])
        .output()
        .context("Failed to set keychain settings")?;

    // Import certificate
    let mut import_args = vec![
        "import",
        cert_path.to_str().unwrap(),
        "-k",
        &keychain_name,
        "-T",
        "/usr/bin/codesign",
        "-T",
        "/usr/bin/security",
    ];

    if let Some(pwd) = password {
        import_args.push("-P");
        import_args.push(pwd);
    }

    let output = Command::new("security")
        .args(&import_args)
        .output()
        .context("Failed to import certificate")?;

    if !output.status.success() {
        // Clean up keychain
        let _ = Command::new("security")
            .args(&["delete-keychain", &keychain_name])
            .output();

        bail!(
            "Failed to import certificate: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Find the signing identity
    let output = Command::new("security")
        .args(&["find-identity", "-v", "-p", "codesigning", &keychain_name])
        .output()
        .context("Failed to find signing identity")?;

    if !output.status.success() {
        // Clean up keychain
        let _ = Command::new("security")
            .args(&["delete-keychain", &keychain_name])
            .output();

        bail!(
            "Failed to find identity: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Parse identity from output
    let identity = output_str
        .lines()
        .find(|line| {
            line.contains("Developer ID Application") || line.contains("Apple Development")
        })
        .and_then(|line| line.split('"').nth(1))
        .ok_or_else(|| anyhow::anyhow!("No valid signing identity found"))?
        .to_string();

    // Store keychain name for cleanup later
    env::set_var("SWEETMCP_TEMP_KEYCHAIN", &keychain_name);

    Ok(identity)
}

/// Clean up temporary keychain
pub fn cleanup_keychain() -> Result<()> {
    if let Ok(keychain_name) = env::var("SWEETMCP_TEMP_KEYCHAIN") {
        let _ = Command::new("security")
            .args(&["delete-keychain", &keychain_name])
            .output();

        env::remove_var("SWEETMCP_TEMP_KEYCHAIN");
    }
    Ok(())
}
