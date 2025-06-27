//! Linux platform implementation using PolicyKit (pkexec) and systemd.

use crate::{InstallerBuilder, InstallerError};
use anyhow::{Context, Result};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

pub(crate) struct PlatformExecutor;

impl PlatformExecutor {
    pub fn install(builder: InstallerBuilder) -> Result<(), InstallerError> {
        // First check if systemd is available
        if !Self::has_systemd() {
            return Err(InstallerError::System(
                "systemd is required but not available".into()
            ));
        }
        
        // Check if pkexec is available
        if !Self::has_pkexec() {
            return Err(InstallerError::MissingExecutable("pkexec".into()));
        }
        
        Self::run_pkexec(Self::install_script(&builder)).map_err(Into::into)
    }

    pub fn uninstall(label: &str) -> Result<(), InstallerError> {
        if !Self::has_systemd() {
            return Err(InstallerError::System(
                "systemd is required but not available".into()
            ));
        }
        
        let script = format!(r#"
            set -e
            systemctl stop {label}.service 2>/dev/null || true
            systemctl disable {label}.service 2>/dev/null || true
            rm -f /etc/systemd/system/{label}.service
            rm -f /usr/local/bin/{label}
            systemctl daemon-reload
        "#, label = label);
        
        Self::run_pkexec(script).map_err(Into::into)
    }

    fn install_script(b: &InstallerBuilder) -> String {
        let unit = Self::unit_file(b);
        format!(r#"
            set -e
            # Create user and group if they don't exist
            if ! id -u {user} >/dev/null 2>&1; then
                useradd -r -s /sbin/nologin {user} || true
            fi
            if ! getent group {group} >/dev/null 2>&1; then
                groupadd -r {group} || true
            fi
            
            # Install binary
            install -m755 {prog} /usr/local/bin/{label}
            
            # Create systemd unit
            cat >/etc/systemd/system/{label}.service <<'UNIT'
{unit}
UNIT
            
            # Reload and start service
            systemctl daemon-reload
            systemctl enable --now {label}.service
        "#,
            prog   = b.program.display(),
            label  = b.label,
            user   = b.run_as_user,
            group  = b.run_as_group,
            unit   = unit,
        )
    }

    fn unit_file(b: &InstallerBuilder) -> String {
        let env_lines = b.env.iter()
            .map(|(k, v)| format!("Environment=\"{}={}\"", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        
        let restart_policy = if b.auto_restart { 
            "Restart=on-failure\nRestartSec=5" 
        } else { 
            "" 
        };
        
        let network_deps = if b.wants_network {
            "After=network-online.target\nWants=network-online.target"
        } else {
            ""
        };
        
        format!(r#"[Unit]
Description={desc}
After=multi-user.target
{network}

[Service]
Type=simple
ExecStart=/usr/local/bin/{label} {args}
User={user}
Group={group}
{env}
{restart}

# Security hardening
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
NoNewPrivileges=true
ReadWritePaths=/var/log /var/lib/{label}

[Install]
WantedBy=multi-user.target
"#,
            desc    = b.description,
            network = network_deps,
            label   = b.label,
            args    = shell_escape::escape(b.args.join(" ")),
            user    = b.run_as_user,
            group   = b.run_as_group,
            env     = env_lines,
            restart = restart_policy,
        )
    }

    fn run_pkexec(script: String) -> Result<(), InstallerError> {
        let status = Command::new("pkexec")
            .arg("bash")
            .arg("-c")
            .arg(&script)
            .status()
            .context("failed to spawn pkexec")?;
            
        if status.success() {
            Ok(())
        } else {
            match status.code() {
                Some(126) => Err(InstallerError::MissingExecutable("bash".into())),
                Some(127) => Err(InstallerError::MissingExecutable("pkexec".into())),
                Some(1) | Some(256) => Err(InstallerError::Cancelled),
                _ => Err(InstallerError::PermissionDenied),
            }
        }
    }
    
    fn has_systemd() -> bool {
        PathBuf::from("/run/systemd/system").exists()
    }
    
    fn has_pkexec() -> bool {
        Command::new("which")
            .arg("pkexec")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(feature = "runtime")]
    pub async fn install_async(builder: InstallerBuilder) -> Result<(), InstallerError> {
        tokio::task::spawn_blocking(move || Self::install(builder))
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

// Helper module for shell escaping
mod shell_escape {
    pub fn escape(s: impl AsRef<str>) -> String {
        let s = s.as_ref();
        if s.is_empty() {
            return "''".to_string();
        }
        
        if s.chars().all(|c| c.is_ascii_alphanumeric() || "-_/.=".contains(c)) {
            s.to_string()
        } else {
            format!("'{}'", s.replace('\'', "'\\''"))
        }
    }
}