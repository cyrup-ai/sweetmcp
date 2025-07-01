//! macOS platform implementation using osascript and launchd.

use crate::install::{InstallerBuilder, InstallerError};
use crate::install::builder::CommandBuilder;
use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use plist::Value;
use std::{collections::HashMap, path::PathBuf, process::Command};

pub(crate) struct PlatformExecutor;

// For now, we'll use a placeholder for the signed helper
// In production, this would contain the actual signed applet
static HELPER_PATH: OnceCell<PathBuf> = OnceCell::new();
const APP_ZIP_PLACEHOLDER: &[u8] = b"PLACEHOLDER_FOR_SIGNED_HELPER";

impl PlatformExecutor {
    pub fn install(b: InstallerBuilder) -> Result<(), InstallerError> {
        // First, copy the binary to /tmp so elevated context can access it
        let temp_path = format!("/tmp/{}", b.label);
        std::fs::copy(&b.program, &temp_path)
            .map_err(|e| InstallerError::System(format!("Failed to copy binary to temp: {}", e)))?;

        let plist_content = Self::generate_plist(&b);
        
        // Build the installation commands using CommandBuilder
        let mkdir_cmd = CommandBuilder::new("mkdir")
            .arg("-p")
            .arg("/Library/LaunchDaemons")
            .arg("/usr/local/bin")
            .arg(&format!("/var/log/{}", b.label));
        
        let cp_cmd = CommandBuilder::new("cp")
            .arg(&temp_path)
            .arg(&format!("/usr/local/bin/{}", b.label));
            
        let chown_cmd = CommandBuilder::new("chown")
            .arg("root:wheel")
            .arg(&format!("/usr/local/bin/{}", b.label));
            
        let chmod_cmd = CommandBuilder::new("chmod")
            .arg("755")
            .arg(&format!("/usr/local/bin/{}", b.label));
            
        let rm_cmd = CommandBuilder::new("rm")
            .arg("-f")
            .arg(&temp_path);
        
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
        let plist_perms_chown = CommandBuilder::new("chown")
            .arg("root:wheel")
            .arg(&plist_file);
        
        let plist_perms_chmod = CommandBuilder::new("chmod")
            .arg("644")
            .arg(&plist_file);
        
        script.push_str(&format!(" && {}", Self::command_to_script(&plist_perms_chown)));
        script.push_str(&format!(" && {}", Self::command_to_script(&plist_perms_chmod)));
        
        // Create services directory
        let services_dir = CommandBuilder::new("mkdir")
            .arg("-p")
            .arg("/etc/cyrupd/services");
        
        script.push_str(&format!(" && {}", Self::command_to_script(&services_dir)));

        // Add service definitions using CommandBuilder
        if !b.services.is_empty() {
            for service in &b.services {
                let service_toml = toml::to_string_pretty(service)
                    .map_err(|e| InstallerError::System(format!("Failed to serialize service: {}", e)))?;
                
                // Write service file to temp first
                let temp_service = format!("/tmp/{}.toml", service.name);
                std::fs::write(&temp_service, &service_toml)
                    .map_err(|e| InstallerError::System(format!("Failed to write temp service: {}", e)))?;
                
                let service_file = format!("/etc/cyrupd/services/{}.toml", service.name);
                script.push_str(&format!(" && mv {} {}", temp_service, service_file));
                
                // Set service file permissions using CommandBuilder
                let service_perms_chown = CommandBuilder::new("chown")
                    .arg("root:wheel")
                    .arg(&service_file);
                
                let service_perms_chmod = CommandBuilder::new("chmod")
                    .arg("644")
                    .arg(&service_file);
                
                script.push_str(&format!(" && {}", Self::command_to_script(&service_perms_chown)));
                script.push_str(&format!(" && {}", Self::command_to_script(&service_perms_chmod)));
            }
        }

        // Load the daemon using CommandBuilder
        let load_daemon = CommandBuilder::new("launchctl")
            .arg("load")
            .arg("-w")
            .arg(&format!("/Library/LaunchDaemons/{}.plist", b.label));
        
        script.push_str(&format!(" && {}", Self::command_to_script(&load_daemon)));

        Self::run_osascript(&script)
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

    #[cfg(feature = "runtime")]
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

    #[cfg(feature = "runtime")]
    pub async fn uninstall_async(label: &str) -> Result<(), InstallerError> {
        let label = label.to_string();
        tokio::task::spawn_blocking(move || Self::uninstall(&label))
            .await
            .context("task join failed")?
    }
}
