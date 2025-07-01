//! Windows platform implementation using UAC and Service Control Manager.

use crate::install::{InstallerBuilder, InstallerError};
use anyhow::{Context, Result};
use std::process::Command;

pub(crate) struct PlatformExecutor;

impl PlatformExecutor {
    pub fn install(b: InstallerBuilder) -> Result<(), InstallerError> {
        // Build the service installation command
        let exe_path = b
            .program
            .to_str()
            .ok_or_else(|| InstallerError::System("Invalid executable path".into()))?;

        let mut cmd = format!(
            r#"sc.exe create "{}" binPath= "{}" start= auto DisplayName= "{}" "#,
            b.label, exe_path, b.description
        );

        // Add command line arguments
        if !b.args.is_empty() {
            let args_str = b.args.join(" ");
            cmd = format!(
                r#"sc.exe create "{}" binPath= "{} {}" start= auto DisplayName= "{}" "#,
                b.label, exe_path, args_str, b.description
            );
        }

        // Set failure actions if auto-restart is enabled
        let restart_cmd = if b.auto_restart {
            format!(
                r#" && sc.exe failure "{}" reset= 86400 actions= restart/5000/restart/10000/restart/30000"#,
                b.label
            )
        } else {
            String::new()
        };

        // Set service dependencies if network is required
        let deps_cmd = if b.wants_network {
            format!(r#" && sc.exe config "{}" depend= Tcpip/Afd"#, b.label)
        } else {
            String::new()
        };

        // Start the service
        let start_cmd = format!(r#" && sc.exe start "{}""#, b.label);

        let full_cmd = format!("{}{}{}{}", cmd, restart_cmd, deps_cmd, start_cmd);

        let result = Self::run_as_admin(&full_cmd);
        
        // If daemon installation succeeded, install service definitions
        if result.is_ok() && !b.services.is_empty() {
            Self::install_services(&b.services)?;
        }
        
        result
    }

    pub fn uninstall(label: &str) -> Result<(), InstallerError> {
        let cmd = format!(r#"sc.exe stop "{}" & sc.exe delete "{}""#, label, label);
        Self::run_as_admin(&cmd)
    }

    fn install_services(services: &[crate::config::ServiceDefinition]) -> Result<(), InstallerError> {
        for service in services {
            let service_toml = toml::to_string_pretty(service)
                .map_err(|e| InstallerError::System(format!("Failed to serialize service: {}", e)))?;
            
            // Windows paths use backslashes
            let services_dir = r"C:\ProgramData\cyrupd\services";
            let service_file = format!(r"{}\{}.toml", services_dir, service.name);
            
            let cmd = format!(
                r#"mkdir "{}" 2>nul & echo.{} > "{}""#,
                services_dir,
                service_toml.replace('\n', "^

& echo."),
                service_file
            );
            
            Self::run_as_admin(&cmd)?;
        }
        Ok(())
    }

    fn run_as_admin(cmd: &str) -> Result<(), InstallerError> {
        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use windows::core::{HSTRING, PCWSTR};
            use windows::Win32::UI::Shell::ShellExecuteW;
            use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

            unsafe {
                // Convert strings to wide strings
                let verb = HSTRING::from("runas");
                let file = HSTRING::from("cmd.exe");
                let params = HSTRING::from(format!("/C {}", cmd));

                let result = ShellExecuteW(
                    windows::Win32::Foundation::HWND(0),
                    &verb,
                    &file,
                    &params,
                    PCWSTR::null(),
                    SW_HIDE,
                );

                let code = result.0 as i32;
                if code > 32 {
                    Ok(())
                } else {
                    match code {
                        2 => Err(InstallerError::MissingExecutable("cmd.exe".into())),
                        5 => Err(InstallerError::PermissionDenied),
                        _ => Err(InstallerError::System(format!(
                            "ShellExecute error: {}",
                            code
                        ))),
                    }
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = cmd;
            Err(InstallerError::System("Not running on Windows".into()))
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
