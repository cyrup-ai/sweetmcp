//! macOS code signing and notarization implementation
//! 
//! High-performance, zero-allocation macOS bundle signing with:
//! - Developer ID certificate management
//! - Hardened runtime and entitlements
//! - Automatic notarization with Apple
//! - Bundle structure validation
//! - Keychain integration

#[cfg(target_os = "macos")]
mod implementation {
    use super::super::{
        BundleError, MacOSError, PlatformError, PlatformSigner, SignedBundle, SigningConfig,
        SigningError, ValidationError, SigningMetrics, path_utils
    };
    use std::path::{Path, PathBuf};
    use std::process::Stdio;
    use std::time::{Duration, Instant, SystemTime};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::process::Command;
    use tracing::{debug, error, info, instrument, warn};
    use tauri_macos_sign::Keychain;
    use security_framework::certificate::SecCertificate;
    use security_framework::identity::SecIdentity;
    use security_framework::keychain::SecKeychain;
    use core_foundation::string::CFString;
    
    /// macOS-specific signing configuration
    #[derive(Debug, Clone)]
    pub struct MacOSConfig {
        pub identity: String,
        pub keychain_path: Option<PathBuf>,
        pub entitlements_path: Option<PathBuf>,
        pub hardened_runtime: bool,
        pub notarize: bool,
        pub apple_id: Option<String>,
        pub app_password: Option<String>,
        pub team_id: Option<String>,
        pub timestamp_url: String,
    }
    
    impl Default for MacOSConfig {
        fn default() -> Self {
            Self {
                identity: String::from("-"), // Ad-hoc signing by default
                keychain_path: None,
                entitlements_path: None,
                hardened_runtime: true,
                notarize: false,
                apple_id: None,
                app_password: None,
                team_id: None,
                timestamp_url: String::from("http://timestamp.apple.com/ts01"),
            }
        }
    }
    
    /// High-performance macOS bundle signer
    #[derive(Debug)]
    pub struct MacOSSigner {
        keychain: Option<Keychain>,
        metrics: std::sync::atomic::AtomicU64,
    }
    
    impl MacOSSigner {
        /// Create new macOS signer with optional keychain
        #[inline]
        pub fn new() -> Self {
            Self {
                keychain: None,
                metrics: std::sync::atomic::AtomicU64::new(0),
            }
        }
        
        /// Create signer with specific identity
        #[inline]
        pub fn with_identity(identity: &str) -> Result<Self, BundleError> {
            let keychain = if identity == "-" {
                Keychain::with_signing_identity("-")
            } else {
                Keychain::with_signing_identity(identity)
            };
            
            Ok(Self {
                keychain: Some(keychain),
                metrics: std::sync::atomic::AtomicU64::new(0),
            })
        }
        
        /// Create signer with certificate from keychain
        pub async fn with_certificate_from_keychain(
            certificate_name: &str,
            keychain_path: Option<&Path>
        ) -> Result<Self, BundleError> {
            let identity = Self::find_identity_in_keychain(certificate_name, keychain_path).await?;
            Self::with_identity(&identity)
        }
        
        /// Find signing identity in keychain
        async fn find_identity_in_keychain(
            certificate_name: &str,
            keychain_path: Option<&Path>
        ) -> Result<String, BundleError> {
            let keychain = if let Some(path) = keychain_path {
                SecKeychain::open(path)
                    .map_err(|_| BundleError::Platform(PlatformError::MacOS(
                        MacOSError::KeychainError("Failed to open keychain")
                    )))?
            } else {
                SecKeychain::default()
                    .map_err(|_| BundleError::Platform(PlatformError::MacOS(
                        MacOSError::KeychainError("Failed to access default keychain")
                    )))?
            };
            
            // Search for certificate by name
            let cf_name = CFString::new(certificate_name);
            let identities = keychain.find_identity(Some(&cf_name))
                .map_err(|_| BundleError::Platform(PlatformError::MacOS(
                    MacOSError::KeychainError("Failed to find identity")
                )))?;
            
            if identities.is_empty() {
                return Err(BundleError::Signing(SigningError::CertificateNotFound(
                    "Certificate not found in keychain"
                )));
            }
            
            // Use the first matching identity
            let identity = &identities[0];
            let certificate = identity.certificate()
                .map_err(|_| BundleError::Platform(PlatformError::MacOS(
                    MacOSError::SecurityFrameworkError("Failed to get certificate from identity")
                )))?;
            
            let subject = certificate.subject_summary()
                .ok_or_else(|| BundleError::Platform(PlatformError::MacOS(
                    MacOSError::SecurityFrameworkError("Failed to get certificate subject")
                )))?;
            
            Ok(subject)
        }
        
        /// Validate macOS app bundle structure
        #[instrument(skip(self), fields(bundle = %bundle_path.display()))]
        async fn validate_bundle_structure(&self, bundle_path: &Path) -> Result<(), BundleError> {
            debug!("Validating bundle structure");
            
            if !bundle_path.exists() {
                return Err(BundleError::Validation(ValidationError::InvalidBundleStructure(
                    "Bundle path does not exist"
                )));
            }
            
            if !path_utils::is_bundle_dir(bundle_path) {
                return Err(BundleError::Validation(ValidationError::InvalidBundleStructure(
                    "Path is not a valid bundle directory"
                )));
            }
            
            // Check required bundle structure
            let contents_dir = bundle_path.join("Contents");
            let macos_dir = contents_dir.join("MacOS");
            let info_plist = contents_dir.join("Info.plist");
            
            if !contents_dir.exists() {
                return Err(BundleError::Validation(ValidationError::MissingFile("Contents directory")));
            }
            
            if !macos_dir.exists() {
                return Err(BundleError::Validation(ValidationError::MissingFile("MacOS directory")));
            }
            
            if !info_plist.exists() {
                return Err(BundleError::Validation(ValidationError::MissingFile("Info.plist")));
            }
            
            // Validate Info.plist
            self.validate_info_plist(&info_plist).await?;
            
            // Check for executable
            let executable_name = self.get_executable_name(&info_plist).await?;
            let executable_path = macos_dir.join(&executable_name);
            
            if !executable_path.exists() {
                return Err(BundleError::Validation(ValidationError::MissingFile("Main executable")));
            }
            
            // Verify executable permissions
            let metadata = tokio::fs::metadata(&executable_path).await
                .map_err(|_| BundleError::Validation(ValidationError::MissingFile("Cannot read executable metadata")))?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                if (mode & 0o111) == 0 {
                    return Err(BundleError::Validation(ValidationError::InvalidBundleStructure(
                        "Executable does not have execute permissions"
                    )));
                }
            }
            
            debug!("Bundle structure validation completed");
            Ok(())
        }
        
        /// Validate Info.plist structure and required keys
        async fn validate_info_plist(&self, plist_path: &Path) -> Result<(), BundleError> {
            let plist_data = tokio::fs::read(plist_path).await
                .map_err(|_| BundleError::Validation(ValidationError::MissingFile("Info.plist")))?;
            
            let plist_value: plist::Value = plist::from_bytes(&plist_data)
                .map_err(|_| BundleError::Validation(ValidationError::InvalidPlist("Malformed Info.plist")))?;
            
            if let plist::Value::Dictionary(dict) = plist_value {
                // Check required keys
                let required_keys = [
                    "CFBundleIdentifier",
                    "CFBundleName", 
                    "CFBundleExecutable",
                    "CFBundleVersion",
                ];
                
                for key in &required_keys {
                    if !dict.contains_key(*key) {
                        return Err(BundleError::Validation(ValidationError::InvalidPlist(
                            "Missing required Info.plist key"
                        )));
                    }
                }
            } else {
                return Err(BundleError::Validation(ValidationError::InvalidPlist(
                    "Info.plist is not a dictionary"
                )));
            }
            
            Ok(())
        }
        
        /// Get executable name from Info.plist
        async fn get_executable_name(&self, plist_path: &Path) -> Result<String, BundleError> {
            let plist_data = tokio::fs::read(plist_path).await
                .map_err(|_| BundleError::Validation(ValidationError::MissingFile("Info.plist")))?;
            
            let plist_value: plist::Value = plist::from_bytes(&plist_data)
                .map_err(|_| BundleError::Validation(ValidationError::InvalidPlist("Malformed Info.plist")))?;
            
            if let plist::Value::Dictionary(dict) = plist_value {
                if let Some(plist::Value::String(executable)) = dict.get("CFBundleExecutable") {
                    return Ok(executable.clone());
                }
            }
            
            Err(BundleError::Validation(ValidationError::InvalidPlist(
                "CFBundleExecutable not found in Info.plist"
            )))
        }
        
        /// Apply code signature to bundle
        #[instrument(skip(self), fields(bundle = %bundle_path.display()))]
        async fn apply_code_signature(
            &self,
            bundle_path: &Path,
            config: &MacOSConfig,
        ) -> Result<[u8; 32], BundleError> {
            let start_time = Instant::now();
            
            let keychain = if let Some(ref kc) = self.keychain {
                kc
            } else {
                return Err(BundleError::Signing(SigningError::SigningFailed(
                    "No keychain configured"
                )));
            };
            
            info!("Starting code signing process");
            
            // Sign the bundle
            keychain.sign(
                bundle_path,
                config.entitlements_path.as_deref(),
                config.hardened_runtime,
            ).map_err(|e| {
                error!("Code signing failed: {}", e);
                BundleError::Platform(PlatformError::MacOS(
                    MacOSError::CodeSignError("Failed to apply code signature")
                ))
            })?;
            
            // Verify the signature immediately
            self.verify_code_signature(bundle_path).await?;
            
            // Calculate signature hash
            let signature_hash = self.calculate_signature_hash(bundle_path).await?;
            
            let duration = start_time.elapsed();
            info!("Code signing completed in {:.2}ms", duration.as_millis());
            
            Ok(signature_hash)
        }
        
        /// Verify code signature using codesign tool
        async fn verify_code_signature(&self, bundle_path: &Path) -> Result<(), BundleError> {
            let output = Command::new("codesign")
                .args(["--verify", "--deep", "--strict"])
                .arg(bundle_path)
                .output()
                .await
                .map_err(|_| BundleError::Signing(SigningError::SigningFailed(
                    "Failed to run codesign verification"
                )))?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Code signature verification failed: {}", stderr);
                return Err(BundleError::Signing(SigningError::InvalidSignature(
                    "Code signature verification failed"
                )));
            }
            
            Ok(())
        }
        
        /// Calculate hash of the code signature
        async fn calculate_signature_hash(&self, bundle_path: &Path) -> Result<[u8; 32], BundleError> {
            use sha2::{Digest, Sha256};
            
            let code_resources_path = bundle_path.join("Contents/_CodeSignature/CodeResources");
            if !code_resources_path.exists() {
                return Err(BundleError::Signing(SigningError::InvalidSignature(
                    "CodeResources file not found"
                )));
            }
            
            let signature_data = tokio::fs::read(&code_resources_path).await
                .map_err(|_| BundleError::Signing(SigningError::InvalidSignature(
                    "Failed to read CodeResources"
                )))?;
            
            let mut hasher = Sha256::new();
            hasher.update(&signature_data);
            Ok(hasher.finalize().into())
        }
        
        /// Submit bundle for notarization
        #[instrument(skip(self), fields(bundle = %bundle_path.display()))]
        async fn submit_for_notarization(
            &self,
            bundle_path: &Path,
            config: &MacOSConfig,
        ) -> Result<(), BundleError> {
            let apple_id = config.apple_id.as_ref()
                .ok_or_else(|| BundleError::Signing(SigningError::NotarizationFailed(
                    "Apple ID required for notarization"
                )))?;
            
            let app_password = config.app_password.as_ref()
                .ok_or_else(|| BundleError::Signing(SigningError::NotarizationFailed(
                    "App-specific password required for notarization"
                )))?;
            
            info!("Starting notarization process");
            
            // Create ZIP archive for submission
            let zip_path = self.create_notarization_zip(bundle_path).await?;
            
            // Submit for notarization
            let submission_id = self.submit_notarization_request(&zip_path, apple_id, app_password).await?;
            
            // Wait for completion
            self.wait_for_notarization_completion(&submission_id, apple_id, app_password).await?;
            
            // Staple the notarization
            self.staple_notarization(bundle_path).await?;
            
            // Clean up ZIP file
            let _ = tokio::fs::remove_file(&zip_path).await;
            
            info!("Notarization completed successfully");
            Ok(())
        }
        
        /// Create ZIP archive for notarization
        async fn create_notarization_zip(&self, bundle_path: &Path) -> Result<PathBuf, BundleError> {
            let zip_name = format!("{}.zip", path_utils::file_stem_str(bundle_path).unwrap_or("bundle"));
            let zip_path = bundle_path.parent().unwrap().join(zip_name);
            
            let output = Command::new("ditto")
                .args(["-c", "-k", "--keepParent"])
                .arg(bundle_path)
                .arg(&zip_path)
                .output()
                .await
                .map_err(|_| BundleError::Signing(SigningError::NotarizationFailed(
                    "Failed to create notarization ZIP"
                )))?;
            
            if !output.status.success() {
                return Err(BundleError::Signing(SigningError::NotarizationFailed(
                    "Failed to create notarization ZIP"
                )));
            }
            
            Ok(zip_path)
        }
        
        /// Submit notarization request
        async fn submit_notarization_request(
            &self,
            zip_path: &Path,
            apple_id: &str,
            app_password: &str,
        ) -> Result<String, BundleError> {
            let output = Command::new("xcrun")
                .args([
                    "notarytool", "submit",
                    "--apple-id", apple_id,
                    "--password", app_password,
                    "--wait",
                    "--output-format", "json"
                ])
                .arg(zip_path)
                .output()
                .await
                .map_err(|_| BundleError::Signing(SigningError::NotarizationFailed(
                    "Failed to submit for notarization"
                )))?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Notarization submission failed: {}", stderr);
                return Err(BundleError::Signing(SigningError::NotarizationFailed(
                    "Notarization submission failed"
                )));
            }
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let json: serde_json::Value = serde_json::from_str(&stdout)
                .map_err(|_| BundleError::Signing(SigningError::NotarizationFailed(
                    "Invalid notarization response"
                )))?;
            
            json.get("id")
                .and_then(|id| id.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| BundleError::Signing(SigningError::NotarizationFailed(
                    "No submission ID in response"
                )))
        }
        
        /// Wait for notarization completion with exponential backoff
        async fn wait_for_notarization_completion(
            &self,
            submission_id: &str,
            apple_id: &str,
            app_password: &str,
        ) -> Result<(), BundleError> {
            let mut delay = Duration::from_secs(30);
            let max_delay = Duration::from_secs(300);
            let max_attempts = 20;
            
            for attempt in 0..max_attempts {
                debug!("Checking notarization status (attempt {})", attempt + 1);
                
                let output = Command::new("xcrun")
                    .args([
                        "notarytool", "info",
                        "--apple-id", apple_id,
                        "--password", app_password,
                        "--output-format", "json",
                        submission_id
                    ])
                    .output()
                    .await
                    .map_err(|_| BundleError::Signing(SigningError::NotarizationFailed(
                        "Failed to check notarization status"
                    )))?;
                
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let json: serde_json::Value = serde_json::from_str(&stdout)
                        .map_err(|_| BundleError::Signing(SigningError::NotarizationFailed(
                            "Invalid status response"
                        )))?;
                    
                    if let Some(status) = json.get("status").and_then(|s| s.as_str()) {
                        match status {
                            "Accepted" => {
                                info!("Notarization accepted");
                                return Ok(());
                            }
                            "Rejected" => {
                                error!("Notarization rejected");
                                return Err(BundleError::Signing(SigningError::NotarizationFailed(
                                    "Notarization was rejected"
                                )));
                            }
                            "In Progress" => {
                                debug!("Notarization still in progress");
                                // Continue waiting
                            }
                            _ => {
                                warn!("Unknown notarization status: {}", status);
                            }
                        }
                    }
                }
                
                if attempt < max_attempts - 1 {
                    tokio::time::sleep(delay).await;
                    delay = core::cmp::min(delay * 2, max_delay);
                }
            }
            
            Err(BundleError::Signing(SigningError::NotarizationFailed(
                "Notarization timed out"
            )))
        }
        
        /// Staple notarization to bundle
        async fn staple_notarization(&self, bundle_path: &Path) -> Result<(), BundleError> {
            let output = Command::new("xcrun")
                .args(["stapler", "staple"])
                .arg(bundle_path)
                .output()
                .await
                .map_err(|_| BundleError::Signing(SigningError::NotarizationFailed(
                    "Failed to staple notarization"
                )))?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Stapling failed: {}", stderr);
                return Err(BundleError::Signing(SigningError::NotarizationFailed(
                    "Failed to staple notarization"
                )));
            }
            
            Ok(())
        }
        
        /// Get current signing metrics
        #[inline]
        pub fn metrics(&self) -> u64 {
            self.metrics.load(std::sync::atomic::Ordering::Relaxed)
        }
        
        /// Reset metrics counter
        #[inline]
        pub fn reset_metrics(&self) {
            self.metrics.store(0, std::sync::atomic::Ordering::Relaxed);
        }
    }
    
    impl Default for MacOSSigner {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl PlatformSigner for MacOSSigner {
        type Config = MacOSConfig;
        type Output = SignedBundle;
        
        #[instrument(skip(self), fields(bundle = %bundle_path.display()))]
        async fn sign_bundle(
            &self,
            bundle_path: &Path,
            config: &Self::Config,
        ) -> Result<Self::Output, BundleError> {
            let start_time = Instant::now();
            
            // Increment metrics
            self.metrics.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            
            // Validate bundle structure
            self.validate_bundle_structure(bundle_path).await?;
            
            // Apply code signature
            let signature_hash = self.apply_code_signature(bundle_path, config).await?;
            
            // Notarize if requested
            if config.notarize {
                self.submit_for_notarization(bundle_path, config).await?;
            }
            
            // Get bundle metadata
            let metadata = tokio::fs::metadata(bundle_path).await
                .map_err(|_| BundleError::Validation(ValidationError::InvalidBundleStructure(
                    "Cannot read bundle metadata"
                )))?;
            
            let version = self.extract_bundle_version(bundle_path).await
                .unwrap_or_else(|_| semver::Version::new(1, 0, 0));
            
            let result = SignedBundle {
                path: bundle_path.to_path_buf(),
                platform: crate::bundle::PlatformTarget::current(),
                signature_hash,
                timestamp: SystemTime::now(),
                size: metadata.len(),
                version,
            };
            
            let duration = start_time.elapsed();
            info!("Bundle signing completed in {:.2}ms", duration.as_millis());
            
            Ok(result)
        }
        
        async fn validate_signature(&self, bundle_path: &Path) -> Result<bool, BundleError> {
            match self.verify_code_signature(bundle_path).await {
                Ok(()) => Ok(true),
                Err(BundleError::Signing(SigningError::InvalidSignature(_))) => Ok(false),
                Err(e) => Err(e),
            }
        }
        
        #[inline]
        fn supported_formats(&self) -> &'static [&'static str] {
            &["app", "framework", "bundle", "dylib", "kext"]
        }
        
        #[inline]
        fn file_extensions(&self) -> &'static [&'static str] {
            &[".app", ".framework", ".bundle", ".dylib", ".kext"]
        }
    }
    
    impl MacOSSigner {
        /// Extract version from bundle Info.plist
        async fn extract_bundle_version(&self, bundle_path: &Path) -> Result<semver::Version, BundleError> {
            let info_plist = bundle_path.join("Contents/Info.plist");
            let plist_data = tokio::fs::read(&info_plist).await
                .map_err(|_| BundleError::Validation(ValidationError::MissingFile("Info.plist")))?;
            
            let plist_value: plist::Value = plist::from_bytes(&plist_data)
                .map_err(|_| BundleError::Validation(ValidationError::InvalidPlist("Malformed Info.plist")))?;
            
            if let plist::Value::Dictionary(dict) = plist_value {
                if let Some(plist::Value::String(version_str)) = dict.get("CFBundleShortVersionString")
                    .or_else(|| dict.get("CFBundleVersion")) {
                    return semver::Version::parse(version_str)
                        .map_err(|_| BundleError::Validation(ValidationError::InvalidPlist(
                            "Invalid version format"
                        )));
                }
            }
            
            Err(BundleError::Validation(ValidationError::InvalidPlist(
                "Version not found in Info.plist"
            )))
        }
    }
}

#[cfg(target_os = "macos")]
pub use implementation::*;

#[cfg(not(target_os = "macos"))]
mod stub {
    use super::super::{BundleError, PlatformError, PlatformSigner, SignedBundle};
    use std::path::Path;
    
    #[derive(Debug, Clone, Default)]
    pub struct MacOSConfig;
    
    #[derive(Debug, Default)]
    pub struct MacOSSigner;
    
    impl MacOSSigner {
        pub fn new() -> Self { Self }
        pub fn with_identity(_: &str) -> Result<Self, BundleError> { 
            Err(BundleError::Platform(PlatformError::Unsupported))
        }
    }
    
    impl PlatformSigner for MacOSSigner {
        type Config = MacOSConfig;
        type Output = SignedBundle;
        
        async fn sign_bundle(&self, _: &Path, _: &Self::Config) -> Result<Self::Output, BundleError> {
            Err(BundleError::Platform(PlatformError::Unsupported))
        }
        
        async fn validate_signature(&self, _: &Path) -> Result<bool, BundleError> {
            Err(BundleError::Platform(PlatformError::Unsupported))
        }
        
        fn supported_formats(&self) -> &'static [&'static str] { &[] }
        fn file_extensions(&self) -> &'static [&'static str] { &[] }
    }
}