//! macOS platform implementation using osascript and launchd.

use crate::install::builder::CommandBuilder;
use crate::install::{InstallerBuilder, InstallerError};
use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use plist::Value;
use std::{collections::HashMap, path::PathBuf, process::Command};

pub(crate) struct PlatformExecutor;

// Global helper path - initialized once, used everywhere
static HELPER_PATH: OnceCell<PathBuf> = OnceCell::new();

// Embedded ZIP data for the signed helper app
// This is generated at build time by build.rs which creates a proper signed macOS helper
const APP_ZIP_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SweetMCPHelper.app.zip"));

impl PlatformExecutor {
    pub fn install(b: InstallerBuilder) -> Result<(), InstallerError> {
        // Initialize helper path if not already set
        Self::ensure_helper_path()?;

        // First, copy the binary to /tmp so elevated context can access it
        let temp_path = format!("/tmp/{}", b.label);
        std::fs::copy(&b.program, &temp_path)
            .map_err(|e| InstallerError::System(format!("Failed to copy binary to temp: {}", e)))?;

        let plist_content = Self::generate_plist(&b);

        // Build the installation commands using CommandBuilder
        let mkdir_cmd = CommandBuilder::new("mkdir").args([
            "-p",
            "/Library/LaunchDaemons",
            "/usr/local/bin",
            &format!("/var/log/{}", b.label),
        ]);

        let cp_cmd =
            CommandBuilder::new("cp").args([&temp_path, &format!("/usr/local/bin/{}", b.label)]);

        let chown_cmd = CommandBuilder::new("chown")
            .args(["root:wheel", &format!("/usr/local/bin/{}", b.label)]);

        let chmod_cmd =
            CommandBuilder::new("chmod").args(["755", &format!("/usr/local/bin/{}", b.label)]);

        let rm_cmd = CommandBuilder::new("rm").args(["-f", &temp_path]);

        // Write files to temp location first, then move them in elevated context
        let temp_plist = format!("/tmp/{}.plist", b.label);
        std::fs::write(&temp_plist, &plist_content)
            .map_err(|e| InstallerError::System(format!("Failed to write temp plist: {}", e)))?;

        let plist_file = format!("/Library/LaunchDaemons/{}.plist", b.label);

        let mut script = format!("set -e\n{}", Self::command_to_script(&mkdir_cmd));
        script.push_str(&format!(" && {}", Self::command_to_script(&cp_cmd)));
        script.push_str(&format!(" && {}", Self::command_to_script(&chown_cmd)));
        script.push_str(&format!(" && {}", Self::command_to_script(&chmod_cmd)));
        script.push_str(&format!(" && {}", Self::command_to_script(&rm_cmd)));
        script.push_str(&format!(" && mv {} {}", temp_plist, plist_file));

        // Set plist permissions
        let plist_perms_chown = CommandBuilder::new("chown").args(["root:wheel", &plist_file]);

        let plist_perms_chmod = CommandBuilder::new("chmod").args(["644", &plist_file]);

        script.push_str(&format!(
            " && {}",
            Self::command_to_script(&plist_perms_chown)
        ));
        script.push_str(&format!(
            " && {}",
            Self::command_to_script(&plist_perms_chmod)
        ));

        // Create services directory
        let services_dir = CommandBuilder::new("mkdir").args(["-p", "/etc/cyrupd/services"]);

        script.push_str(&format!(" && {}", Self::command_to_script(&services_dir)));

        // Add service definitions using CommandBuilder
        if !b.services.is_empty() {
            for service in &b.services {
                let service_toml = toml::to_string_pretty(service).map_err(|e| {
                    InstallerError::System(format!("Failed to serialize service: {}", e))
                })?;

                // Write service file to temp first
                let temp_service = format!("/tmp/{}.toml", service.name);
                std::fs::write(&temp_service, &service_toml).map_err(|e| {
                    InstallerError::System(format!("Failed to write temp service: {}", e))
                })?;

                let service_file = format!("/etc/cyrupd/services/{}.toml", service.name);
                script.push_str(&format!(" && mv {} {}", temp_service, service_file));

                // Set service file permissions using CommandBuilder
                let service_perms_chown =
                    CommandBuilder::new("chown").args(["root:wheel", &service_file]);

                let service_perms_chmod = CommandBuilder::new("chmod").args(["644", &service_file]);

                script.push_str(&format!(
                    " && {}",
                    Self::command_to_script(&service_perms_chown)
                ));
                script.push_str(&format!(
                    " && {}",
                    Self::command_to_script(&service_perms_chmod)
                ));
            }
        }

        // Load the daemon using CommandBuilder
        let load_daemon = CommandBuilder::new("launchctl").args([
            "load",
            "-w",
            &format!("/Library/LaunchDaemons/{}.plist", b.label),
        ]);

        script.push_str(&format!(" && {}", Self::command_to_script(&load_daemon)));

        Self::run_osascript(&script)
    }

    /// Ensure the helper path is initialized for secure privileged operations
    fn ensure_helper_path() -> Result<(), InstallerError> {
        if HELPER_PATH.get().is_none() {
            let helper_path = Self::extract_helper_app()?;
            HELPER_PATH
                .set(helper_path)
                .map_err(|_| InstallerError::System("Failed to set helper path".to_string()))?;
        }
        Ok(())
    }

    /// Extract the signed helper app from embedded data
    fn extract_helper_app() -> Result<PathBuf, InstallerError> {
        // Use a stable location based on app version to avoid re-extraction
        let version_hash = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            APP_ZIP_DATA.len().hash(&mut hasher);
            APP_ZIP_DATA
                .get(0..64)
                .unwrap_or(&APP_ZIP_DATA[0..APP_ZIP_DATA.len().min(64)])
                .hash(&mut hasher);
            hasher.finish()
        };

        let helper_dir = std::env::temp_dir()
            .join("sweetmcp_helper")
            .join(format!("v{:016x}", version_hash));

        std::fs::create_dir_all(&helper_dir).map_err(|e| {
            InstallerError::System(format!("Failed to create helper directory: {}", e))
        })?;

        let helper_path = helper_dir.join("SweetMCPHelper.app");

        // Check if helper already exists and is valid
        if helper_path.exists()
            && Self::validate_helper(&helper_path)?
            && Self::verify_code_signature(&helper_path)?
        {
            return Ok(helper_path);
        }

        // Extract from embedded data
        Self::extract_from_embedded_data(&helper_path)?;

        Ok(helper_path)
    }

    /// Extract helper from embedded APP_ZIP_DATA
    fn extract_from_embedded_data(helper_path: &PathBuf) -> Result<bool, InstallerError> {
        // Validate ZIP data size and header
        if APP_ZIP_DATA.len() < 22 {
            // Minimum ZIP file size
            return Err(InstallerError::System(
                "Embedded helper ZIP data is too small".to_string(),
            ));
        }

        // Check ZIP signature
        if !APP_ZIP_DATA.starts_with(&[0x50, 0x4B, 0x03, 0x04]) && // Local file header
           !APP_ZIP_DATA.starts_with(&[0x50, 0x4B, 0x05, 0x06]) && // Empty archive
           !APP_ZIP_DATA.starts_with(&[0x50, 0x4B, 0x07, 0x08])
        {
            // Spanned archive
            return Err(InstallerError::System(
                "Invalid ZIP signature in embedded data".to_string(),
            ));
        }

        // Extract the embedded ZIP data
        match Self::extract_zip_data(APP_ZIP_DATA, helper_path) {
            Ok(_) => {
                // Verify the extracted app is valid and properly signed
                if Self::validate_helper(helper_path)? && Self::verify_code_signature(helper_path)?
                {
                    Ok(true)
                } else {
                    // Cleanup invalid extraction
                    let _ = std::fs::remove_dir_all(helper_path);
                    Err(InstallerError::System(
                        "Extracted helper failed validation".to_string(),
                    ))?
                }
            }
            Err(e) => {
                // Extraction failed, cleanup
                let _ = std::fs::remove_dir_all(helper_path);
                Err(e)
            }
        }
    }

    /// Extract ZIP data to the specified path
    fn extract_zip_data(zip_data: &[u8], target_path: &PathBuf) -> Result<(), InstallerError> {
        use std::io::{Cursor, Read};
        use zip::ZipArchive;

        // Create a cursor for the ZIP data
        let cursor = Cursor::new(zip_data);

        // Create ZIP archive reader
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| InstallerError::System(format!("Failed to read ZIP archive: {}", e)))?;

        // Extract all files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                InstallerError::System(format!("Failed to access file in ZIP: {}", e))
            })?;

            let file_path = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => {
                    // Skip files with invalid paths
                    continue;
                }
            };

            let out_path = target_path.join(&file_path);

            // Create parent directories if needed
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    InstallerError::System(format!(
                        "Failed to create directory {}: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }

            if file.is_dir() {
                // Create directory
                std::fs::create_dir_all(&out_path).map_err(|e| {
                    InstallerError::System(format!(
                        "Failed to create directory {}: {}",
                        out_path.display(),
                        e
                    ))
                })?;
            } else {
                // Extract file
                let mut outfile = std::fs::File::create(&out_path).map_err(|e| {
                    InstallerError::System(format!(
                        "Failed to create file {}: {}",
                        out_path.display(),
                        e
                    ))
                })?;

                // Copy file contents with zero-copy optimization where possible
                let mut buffer = Vec::with_capacity(file.size() as usize);
                file.read_to_end(&mut buffer).map_err(|e| {
                    InstallerError::System(format!("Failed to read file from ZIP: {}", e))
                })?;

                use std::io::Write;
                outfile.write_all(&buffer).map_err(|e| {
                    InstallerError::System(format!(
                        "Failed to write file {}: {}",
                        out_path.display(),
                        e
                    ))
                })?;

                // Sync to ensure data is written
                outfile.sync_all().map_err(|e| {
                    InstallerError::System(format!(
                        "Failed to sync file {}: {}",
                        out_path.display(),
                        e
                    ))
                })?;

                // Set file permissions on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        let permissions = std::fs::Permissions::from_mode(mode);
                        std::fs::set_permissions(&out_path, permissions).map_err(|e| {
                            InstallerError::System(format!(
                                "Failed to set permissions on {}: {}",
                                out_path.display(),
                                e
                            ))
                        })?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate that the helper app is properly signed and functional
    fn validate_helper(helper_path: &PathBuf) -> Result<bool, InstallerError> {
        // Check if the helper exists and has the expected structure
        let contents = helper_path.join("Contents");
        let macos = contents.join("MacOS");
        let info_plist = contents.join("Info.plist");
        let executable = macos.join("SweetMCPHelper");

        // Verify all required components exist
        if !contents.exists() || !macos.exists() || !info_plist.exists() || !executable.exists() {
            return Ok(false);
        }

        // Verify Info.plist contains required keys
        let plist_data = std::fs::read(&info_plist)
            .map_err(|e| InstallerError::System(format!("Failed to read Info.plist: {}", e)))?;

        let plist_value = plist::from_bytes::<plist::Value>(&plist_data)
            .map_err(|e| InstallerError::System(format!("Failed to parse Info.plist: {}", e)))?;

        if let plist::Value::Dictionary(dict) = plist_value {
            // Check required keys
            let has_bundle_id = dict.contains_key("CFBundleIdentifier");
            let has_bundle_executable = dict.contains_key("CFBundleExecutable");
            let has_sm_authorized = dict.contains_key("SMAuthorizedClients");

            Ok(has_bundle_id && has_bundle_executable && has_sm_authorized)
        } else {
            Ok(false)
        }
    }

    /// Verify the code signature of the helper app
    fn verify_code_signature(helper_path: &PathBuf) -> Result<bool, InstallerError> {
        // Use codesign to verify the signature
        let output = Command::new("codesign")
            .args([
                "--verify",
                "--deep",
                "--strict",
                "--verbose=2",
                helper_path.to_str().unwrap_or(""),
            ])
            .output()
            .map_err(|e| InstallerError::System(format!("Failed to run codesign: {}", e)))?;

        if output.status.success() {
            // Get detailed signature information
            let verify_output = Command::new("codesign")
                .args(["-d", "--verbose=4", helper_path.to_str().unwrap_or("")])
                .output()
                .map_err(|e| InstallerError::System(format!("Failed to run codesign -d: {}", e)))?;

            let stderr = String::from_utf8_lossy(&verify_output.stderr);

            // For distributed deployment, accept both Developer ID signatures and ad-hoc signatures
            let has_developer_id = stderr.contains("Developer ID Application");
            let has_valid_timestamp = stderr.contains("Timestamp=");
            let has_hardened_runtime = stderr.contains("flags=0x10000(runtime)");
            let is_adhoc_signed = stderr.contains("Signature=adhoc");

            // Accept either a fully signed Developer ID app OR a properly ad-hoc signed app
            let is_valid = (has_developer_id && has_valid_timestamp && has_hardened_runtime) || is_adhoc_signed;

            Ok(is_valid)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(InstallerError::System(format!(
                "Code signature verification failed: {}",
                stderr
            )))
        }
    }

    pub fn uninstall(label: &str) -> Result<(), InstallerError> {
        let script = format!(
            r#"
            set -e
            # Unload daemon if running
            launchctl unload -w /Library/LaunchDaemons/{label}.plist 2>/dev/null || true
            
            # Remove files
            rm -f /Library/LaunchDaemons/{label}.plist
            rm -f /usr/local/bin/{label}
            rm -rf /var/log/{label}
        "#,
            label = label
        );

        Self::run_osascript(&script)
    }

    fn generate_plist(b: &InstallerBuilder) -> String {
        let mut plist = HashMap::new();

        // Basic properties
        plist.insert("Label".to_string(), Value::String(b.label.clone()));
        plist.insert("Disabled".to_string(), Value::Boolean(false));

        // Program and arguments
        let mut program_args = vec![Value::String(format!("/usr/local/bin/{}", b.label))];
        program_args.extend(b.args.iter().map(|a| Value::String(a.clone())));
        plist.insert("ProgramArguments".to_string(), Value::Array(program_args));

        // Environment variables
        if !b.env.is_empty() {
            let env_dict: HashMap<String, Value> = b
                .env
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect();
            plist.insert(
                "EnvironmentVariables".to_string(),
                Value::Dictionary(env_dict.into_iter().collect()),
            );
        }

        // User/Group
        plist.insert("UserName".to_string(), Value::String(b.run_as_user.clone()));
        if b.run_as_group != "wheel" && b.run_as_group != "staff" {
            plist.insert(
                "GroupName".to_string(),
                Value::String(b.run_as_group.clone()),
            );
        }

        // Auto-restart
        plist.insert(
            "KeepAlive".to_string(),
            if b.auto_restart {
                Value::Dictionary(
                    vec![("SuccessfulExit".to_string(), Value::Boolean(false))]
                        .into_iter()
                        .collect(),
                )
            } else {
                Value::Boolean(false)
            },
        );

        // Logging
        plist.insert(
            "StandardOutPath".to_string(),
            Value::String(format!("/var/log/{}/stdout.log", b.label)),
        );
        plist.insert(
            "StandardErrorPath".to_string(),
            Value::String(format!("/var/log/{}/stderr.log", b.label)),
        );

        // Run at load
        plist.insert("RunAtLoad".to_string(), Value::Boolean(true));

        // Network dependency
        if b.wants_network {
            plist.insert(
                "LimitLoadToSessionType".to_string(),
                Value::String("System".to_string()),
            );
        }

        // Generate XML
        let mut buf = Vec::new();
        plist::to_writer_xml(&mut buf, &Value::Dictionary(plist.into_iter().collect()))
            .expect("plist generation failed");
        String::from_utf8(buf).expect("valid utf8")
    }

    fn run_osascript(script: &str) -> Result<(), InstallerError> {
        // Escape the script for AppleScript
        let escaped_script = script.replace('\\', "\\\\").replace('"', "\\\"");

        let applescript = format!(
            r#"do shell script "{}" with administrator privileges"#,
            escaped_script
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&applescript)
            .output()
            .context("failed to invoke osascript")?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("User canceled") || stderr.contains("-128") {
                Err(InstallerError::Cancelled)
            } else if stderr.contains("authorization") || stderr.contains("privileges") {
                Err(InstallerError::PermissionDenied)
            } else {
                Err(InstallerError::System(stderr.into_owned()))
            }
        }
    }

    pub async fn install_async(b: InstallerBuilder) -> Result<(), InstallerError> {
        tokio::task::spawn_blocking(move || Self::install(b))
            .await
            .context("task join failed")?
    }

    fn command_to_script(cmd: &CommandBuilder) -> String {
        let mut parts = vec![cmd.program.to_string_lossy().to_string()];
        parts.extend(cmd.args.iter().cloned());
        parts.join(" ")
    }

    pub async fn uninstall_async(label: &str) -> Result<(), InstallerError> {
        let label = label.to_string();
        tokio::task::spawn_blocking(move || Self::uninstall(&label))
            .await
            .context("task join failed")?
    }
}
