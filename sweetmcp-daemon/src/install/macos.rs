//! macOS platform implementation using osascript and launchd.

use crate::install::{InstallerBuilder, InstallerError};
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
        let plist_content = Self::generate_plist(&b);
        let script = format!(
            r#"
            set -e
            # Create directories
            mkdir -p /Library/LaunchDaemons
            mkdir -p /usr/local/bin
            mkdir -p /var/log/{label}
            
            # Install binary
            cp {prog} /usr/local/bin/{label}
            chown root:wheel /usr/local/bin/{label}
            chmod 755 /usr/local/bin/{label}
            
            # Create launch daemon plist
            cat > /Library/LaunchDaemons/{label}.plist << 'EOF'
{plist}
EOF
            
            # Set permissions
            chown root:wheel /Library/LaunchDaemons/{label}.plist
            chmod 644 /Library/LaunchDaemons/{label}.plist
            
            # Load the daemon
            launchctl load -w /Library/LaunchDaemons/{label}.plist
        "#,
            prog = b.program.display(),
            label = b.label,
            plist = plist_content,
        );

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

    #[cfg(feature = "runtime")]
    pub async fn uninstall_async(label: &str) -> Result<(), InstallerError> {
        let label = label.to_string();
        tokio::task::spawn_blocking(move || Self::uninstall(&label))
            .await
            .context("task join failed")?
    }
}
