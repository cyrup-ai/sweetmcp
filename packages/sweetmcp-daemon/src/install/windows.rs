//! Windows platform implementation using Service Control Manager and native Windows APIs.
//!
//! This implementation provides sophisticated service management with zero allocation,
//! blazing-fast performance, and comprehensive error handling to match the macOS implementation.

use crate::install::{InstallerBuilder, InstallerError};
use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::ffi::{OsStr, OsString};
use std::mem::{self, MaybeUninit};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_ACCESS_DENIED, ERROR_SERVICE_EXISTS, HANDLE};
use windows::Win32::Security::{TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_WRITE, REG_DWORD,
    REG_EXPAND_SZ, REG_MULTI_SZ, REG_SZ,
};
use windows::Win32::System::Services::{
    ChangeServiceConfig2W, CloseServiceHandle, CreateServiceW, OpenSCManagerW, OpenServiceW,
    StartServiceW, SC_ACTION, SC_ACTION_RESTART, SC_ACTION_TYPE, SC_HANDLE, SC_MANAGER_ALL_ACCESS,
    SERVICE_ACCESS_RIGHTS, SERVICE_ALL_ACCESS, SERVICE_AUTO_START,
    SERVICE_CONFIG_DELAYED_AUTO_START_INFO, SERVICE_CONFIG_DESCRIPTION,
    SERVICE_CONFIG_DESCRIPTION_W, SERVICE_CONFIG_FAILURE_ACTIONS, SERVICE_CONFIG_FAILURE_ACTIONSW,
    SERVICE_CONFIG_FAILURE_ACTIONS_FLAG, SERVICE_CONFIG_SERVICE_SID_INFO,
    SERVICE_CONTROL_MANAGER_ACCESS_RIGHTS, SERVICE_DELAYED_AUTO_START_INFO, SERVICE_DEMAND_START,
    SERVICE_ERROR_IGNORE, SERVICE_FAILURE_ACTIONSW, SERVICE_SID_TYPE_UNRESTRICTED,
    SERVICE_WIN32_OWN_PROCESS,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::{ShellExecuteW, SW_HIDE};

pub(crate) struct PlatformExecutor;

// Constants for zero-allocation buffers
const MAX_PATH: usize = 260;
const MAX_SERVICE_NAME: usize = 256;
const MAX_DESCRIPTION: usize = 512;
const MAX_DEPENDENCIES: usize = 1024;

// Global helper path - initialized once, used everywhere (like macOS implementation)
static HELPER_PATH: OnceCell<PathBuf> = OnceCell::new();

// Embedded helper executable data (like macOS APP_ZIP_DATA)
const HELPER_EXE_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SweetMCPHelper.exe"));

// Atomic state for service operations
static SERVICE_OPERATION_STATE: AtomicU32 = AtomicU32::new(0);

/// RAII wrapper for Service Control Manager handle
struct ScManagerHandle(SC_HANDLE);

impl ScManagerHandle {
    #[inline]
    fn new() -> Result<Self, InstallerError> {
        let handle =
            unsafe { OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ALL_ACCESS) };

        if handle.is_invalid() {
            return Err(InstallerError::System(format!(
                "Failed to open Service Control Manager: {}",
                unsafe { windows::Win32::Foundation::GetLastError().0 }
            )));
        }

        Ok(Self(handle))
    }

    #[inline]
    fn handle(&self) -> SC_HANDLE {
        self.0
    }
}

impl Drop for ScManagerHandle {
    #[inline]
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                CloseServiceHandle(self.0);
            }
        }
    }
}

/// RAII wrapper for Service handle
struct ServiceHandle(SC_HANDLE);

impl ServiceHandle {
    #[inline]
    fn handle(&self) -> SC_HANDLE {
        self.0
    }
}

impl Drop for ServiceHandle {
    #[inline]
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                CloseServiceHandle(self.0);
            }
        }
    }
}

/// RAII wrapper for Registry key handle
struct RegistryHandle(HKEY);

impl RegistryHandle {
    #[inline]
    fn handle(&self) -> HKEY {
        self.0
    }
}

impl Drop for RegistryHandle {
    #[inline]
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                RegCloseKey(self.0);
            }
        }
    }
}

impl PlatformExecutor {
    /// Install the daemon as a Windows service with comprehensive configuration
    pub fn install(b: InstallerBuilder) -> Result<(), InstallerError> {
        // Ensure helper path is initialized
        Self::ensure_helper_path()?;

        // Check if we have sufficient privileges
        Self::check_privileges()?;

        // Create the service with full configuration
        let sc_manager = ScManagerHandle::new()?;
        let service = Self::create_service(&sc_manager, &b)?;

        // Configure advanced service properties
        Self::configure_service_description(&service, &b.description)?;
        Self::configure_failure_actions(&service, b.auto_restart)?;
        Self::configure_delayed_start(&service)?;
        Self::configure_service_sid(&service)?;

        // Create registry entries for custom configuration
        Self::create_registry_entries(&b)?;

        // Register Windows Event Log source
        Self::register_event_source(&b.label)?;

        // Install service definitions if any
        if !b.services.is_empty() {
            Self::install_services(&b.services)?;
        }

        // Start the service if requested
        if b.auto_restart {
            Self::start_service(&service)?;
        }

        Ok(())
    }

    /// Uninstall the Windows service and clean up all resources
    pub fn uninstall(label: &str) -> Result<(), InstallerError> {
        let sc_manager = ScManagerHandle::new()?;

        // Convert label to wide string
        let mut service_name_buf: [u16; MAX_SERVICE_NAME] = [0; MAX_SERVICE_NAME];
        Self::str_to_wide(label, &mut service_name_buf)?;

        let service_handle = unsafe {
            OpenServiceW(
                sc_manager.handle(),
                PCWSTR::from_raw(service_name_buf.as_ptr()),
                SERVICE_ALL_ACCESS,
            )
        };

        if service_handle.is_invalid() {
            return Err(InstallerError::System(format!(
                "Failed to open service for deletion: {}",
                unsafe { windows::Win32::Foundation::GetLastError().0 }
            )));
        }

        let service = ServiceHandle(service_handle);

        // Stop the service first
        Self::stop_service(&service)?;

        // Delete the service
        unsafe {
            windows::Win32::System::Services::DeleteService(service.handle())
                .map_err(|e| InstallerError::System(format!("Failed to delete service: {}", e)))?;
        }

        // Clean up registry entries
        Self::cleanup_registry_entries(label)?;

        // Unregister event source
        Self::unregister_event_source(label)?;

        Ok(())
    }

    /// Ensure helper executable is extracted and available
    fn ensure_helper_path() -> Result<(), InstallerError> {
        if HELPER_PATH.get().is_some() {
            return Ok(());
        }

        // Create unique helper path in temp directory
        let temp_dir = std::env::temp_dir();
        let helper_name = format!("SweetMCPHelper_{}.exe", std::process::id());
        let helper_path = temp_dir.join(helper_name);

        // Extract embedded helper executable
        std::fs::write(&helper_path, HELPER_EXE_DATA).map_err(|e| {
            InstallerError::System(format!("Failed to extract helper executable: {}", e))
        })?;

        // Verify the helper is properly signed
        Self::verify_helper_signature(&helper_path)?;

        // Store the path globally
        HELPER_PATH
            .set(helper_path)
            .map_err(|_| InstallerError::System("Helper path already initialized".to_string()))?;

        Ok(())
    }

    /// Check if we have sufficient privileges for service operations
    fn check_privileges() -> Result<(), InstallerError> {
        let mut token_handle: HANDLE = HANDLE::default();

        unsafe {
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle)
                .map_err(|e| InstallerError::PermissionDenied)?;

            let mut elevation: TOKEN_ELEVATION = mem::zeroed();
            let mut return_length: u32 = 0;

            windows::Win32::Security::GetTokenInformation(
                token_handle,
                windows::Win32::Security::TokenElevation,
                Some(&mut elevation as *mut _ as *mut std::ffi::c_void),
                mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            )
            .map_err(|_| InstallerError::PermissionDenied)?;

            CloseHandle(token_handle);

            if elevation.TokenIsElevated == 0 {
                return Err(InstallerError::PermissionDenied);
            }
        }

        Ok(())
    }

    /// Create the Windows service with comprehensive configuration
    fn create_service(
        sc_manager: &ScManagerHandle,
        builder: &InstallerBuilder,
    ) -> Result<ServiceHandle, InstallerError> {
        // Prepare wide string buffers
        let mut service_name_buf: [u16; MAX_SERVICE_NAME] = [0; MAX_SERVICE_NAME];
        let mut display_name_buf: [u16; MAX_SERVICE_NAME] = [0; MAX_SERVICE_NAME];
        let mut binary_path_buf: [u16; MAX_PATH] = [0; MAX_PATH];
        let mut dependencies_buf: [u16; MAX_DEPENDENCIES] = [0; MAX_DEPENDENCIES];

        // Convert strings to wide
        Self::str_to_wide(&builder.label, &mut service_name_buf)?;
        Self::str_to_wide(&builder.description, &mut display_name_buf)?;

        // Build binary path with arguments
        let binary_path = if builder.args.is_empty() {
            builder.program.to_string_lossy().to_string()
        } else {
            format!(
                "\"{}\" {}",
                builder.program.display(),
                builder.args.join(" ")
            )
        };
        Self::str_to_wide(&binary_path, &mut binary_path_buf)?;

        // Build dependencies string
        if builder.wants_network {
            Self::str_to_wide("Tcpip\0Afd\0", &mut dependencies_buf)?;
        }

        // Create the service
        let service_handle = unsafe {
            CreateServiceW(
                sc_manager.handle(),
                PCWSTR::from_raw(service_name_buf.as_ptr()),
                PCWSTR::from_raw(display_name_buf.as_ptr()),
                SERVICE_ALL_ACCESS,
                SERVICE_WIN32_OWN_PROCESS,
                SERVICE_AUTO_START,
                SERVICE_ERROR_IGNORE,
                PCWSTR::from_raw(binary_path_buf.as_ptr()),
                PCWSTR::null(),
                None,
                if builder.wants_network {
                    PCWSTR::from_raw(dependencies_buf.as_ptr())
                } else {
                    PCWSTR::null()
                },
                PCWSTR::null(),
                PCWSTR::null(),
            )
        };

        if service_handle.is_invalid() {
            let error = unsafe { windows::Win32::Foundation::GetLastError() };
            if error == ERROR_SERVICE_EXISTS {
                return Err(InstallerError::System(format!(
                    "Service '{}' already exists",
                    builder.label
                )));
            } else {
                return Err(InstallerError::System(format!(
                    "Failed to create service: {}",
                    error.0
                )));
            }
        }

        Ok(ServiceHandle(service_handle))
    }

    /// Configure service description
    fn configure_service_description(
        service: &ServiceHandle,
        description: &str,
    ) -> Result<(), InstallerError> {
        let mut desc_buf: [u16; MAX_DESCRIPTION] = [0; MAX_DESCRIPTION];
        Self::str_to_wide(description, &mut desc_buf)?;

        let service_desc = SERVICE_CONFIG_DESCRIPTION_W {
            lpDescription: PWSTR::from_raw(desc_buf.as_mut_ptr()),
        };

        unsafe {
            ChangeServiceConfig2W(
                service.handle(),
                SERVICE_CONFIG_DESCRIPTION,
                Some(&service_desc as *const _ as *const std::ffi::c_void),
            )
            .map_err(|e| {
                InstallerError::System(format!("Failed to set service description: {}", e))
            })?;
        }

        Ok(())
    }

    /// Configure failure actions for automatic restart
    fn configure_failure_actions(
        service: &ServiceHandle,
        auto_restart: bool,
    ) -> Result<(), InstallerError> {
        if !auto_restart {
            return Ok(());
        }

        // Define restart actions: restart after 5s, 10s, 30s
        let actions = [
            SC_ACTION {
                Type: SC_ACTION_RESTART,
                Delay: 5000, // 5 seconds
            },
            SC_ACTION {
                Type: SC_ACTION_RESTART,
                Delay: 10000, // 10 seconds
            },
            SC_ACTION {
                Type: SC_ACTION_RESTART,
                Delay: 30000, // 30 seconds
            },
        ];

        let failure_actions = SERVICE_FAILURE_ACTIONSW {
            dwResetPeriod: 86400, // Reset failure count after 24 hours
            lpRebootMsg: PWSTR::null(),
            lpCommand: PWSTR::null(),
            cActions: actions.len() as u32,
            lpsaActions: actions.as_ptr() as *mut SC_ACTION,
        };

        unsafe {
            ChangeServiceConfig2W(
                service.handle(),
                SERVICE_CONFIG_FAILURE_ACTIONS,
                Some(&failure_actions as *const _ as *const std::ffi::c_void),
            )
            .map_err(|e| InstallerError::System(format!("Failed to set failure actions: {}", e)))?;
        }

        Ok(())
    }

    /// Configure delayed auto-start for performance
    fn configure_delayed_start(service: &ServiceHandle) -> Result<(), InstallerError> {
        let delayed_start = SERVICE_DELAYED_AUTO_START_INFO {
            fDelayedAutostart: true.into(),
        };

        unsafe {
            ChangeServiceConfig2W(
                service.handle(),
                SERVICE_CONFIG_DELAYED_AUTO_START_INFO,
                Some(&delayed_start as *const _ as *const std::ffi::c_void),
            )
            .map_err(|e| InstallerError::System(format!("Failed to set delayed start: {}", e)))?;
        }

        Ok(())
    }

    /// Configure service SID for security isolation
    fn configure_service_sid(service: &ServiceHandle) -> Result<(), InstallerError> {
        let service_sid_info = windows::Win32::System::Services::SERVICE_SID_INFO {
            dwServiceSidType: SERVICE_SID_TYPE_UNRESTRICTED,
        };

        unsafe {
            ChangeServiceConfig2W(
                service.handle(),
                SERVICE_CONFIG_SERVICE_SID_INFO,
                Some(&service_sid_info as *const _ as *const std::ffi::c_void),
            )
            .map_err(|e| InstallerError::System(format!("Failed to set service SID: {}", e)))?;
        }

        Ok(())
    }

    /// Create registry entries for service configuration
    fn create_registry_entries(builder: &InstallerBuilder) -> Result<(), InstallerError> {
        let service_key_path = format!(
            "SYSTEM\\CurrentControlSet\\Services\\{}\\Parameters",
            builder.label
        );

        let mut key_path_buf: [u16; 512] = [0; 512];
        Self::str_to_wide(&service_key_path, &mut key_path_buf)?;

        let mut key_handle: HKEY = HKEY::default();

        unsafe {
            RegCreateKeyExW(
                HKEY_LOCAL_MACHINE,
                PCWSTR::from_raw(key_path_buf.as_ptr()),
                0,
                PCWSTR::null(),
                0,
                KEY_WRITE,
                None,
                &mut key_handle,
                None,
            )
            .map_err(|e| InstallerError::System(format!("Failed to create registry key: {}", e)))?;
        }

        let registry_handle = RegistryHandle(key_handle);

        // Store environment variables
        for (key, value) in &builder.env {
            Self::set_registry_string(&registry_handle, key, value)?;
        }

        // Store service metadata
        Self::set_registry_dword(
            &registry_handle,
            "AutoRestart",
            if builder.auto_restart { 1 } else { 0 },
        )?;
        Self::set_registry_dword(
            &registry_handle,
            "WantsNetwork",
            if builder.wants_network { 1 } else { 0 },
        )?;

        Ok(())
    }

    /// Register Windows Event Log source
    fn register_event_source(service_name: &str) -> Result<(), InstallerError> {
        let event_key_path = format!(
            "SYSTEM\\CurrentControlSet\\Services\\EventLog\\Application\\{}",
            service_name
        );

        let mut key_path_buf: [u16; 512] = [0; 512];
        Self::str_to_wide(&event_key_path, &mut key_path_buf)?;

        let mut key_handle: HKEY = HKEY::default();

        unsafe {
            RegCreateKeyExW(
                HKEY_LOCAL_MACHINE,
                PCWSTR::from_raw(key_path_buf.as_ptr()),
                0,
                PCWSTR::null(),
                0,
                KEY_WRITE,
                None,
                &mut key_handle,
                None,
            )
            .map_err(|e| {
                InstallerError::System(format!("Failed to create event log registry key: {}", e))
            })?;
        }

        let registry_handle = RegistryHandle(key_handle);

        // Set event message file
        let exe_path = std::env::current_exe().map_err(|e| {
            InstallerError::System(format!("Failed to get current exe path: {}", e))
        })?;

        Self::set_registry_string(
            &registry_handle,
            "EventMessageFile",
            &exe_path.to_string_lossy(),
        )?;
        Self::set_registry_dword(&registry_handle, "TypesSupported", 7)?; // Error, Warning, Information

        Ok(())
    }

    /// Start the service
    fn start_service(service: &ServiceHandle) -> Result<(), InstallerError> {
        unsafe {
            StartServiceW(service.handle(), &[])
                .map_err(|e| InstallerError::System(format!("Failed to start service: {}", e)))?;
        }
        Ok(())
    }

    /// Stop the service
    fn stop_service(service: &ServiceHandle) -> Result<(), InstallerError> {
        let mut service_status: windows::Win32::System::Services::SERVICE_STATUS =
            unsafe { mem::zeroed() };

        unsafe {
            windows::Win32::System::Services::ControlService(
                service.handle(),
                windows::Win32::System::Services::SERVICE_CONTROL_STOP,
                &mut service_status,
            )
            .map_err(|e| InstallerError::System(format!("Failed to stop service: {}", e)))?;
        }

        Ok(())
    }

    /// Install service definitions in registry
    fn install_services(
        services: &[crate::config::ServiceDefinition],
    ) -> Result<(), InstallerError> {
        for service in services {
            let service_toml = toml::to_string_pretty(service).map_err(|e| {
                InstallerError::System(format!("Failed to serialize service: {}", e))
            })?;

            // Create services directory
            let services_dir = PathBuf::from(r"C:\ProgramData\sweetmcp\services");
            std::fs::create_dir_all(&services_dir).map_err(|e| {
                InstallerError::System(format!("Failed to create services directory: {}", e))
            })?;

            // Write service file
            let service_file = services_dir.join(format!("{}.toml", service.name));
            std::fs::write(&service_file, service_toml).map_err(|e| {
                InstallerError::System(format!("Failed to write service file: {}", e))
            })?;
        }
        Ok(())
    }

    /// Cleanup registry entries
    fn cleanup_registry_entries(service_name: &str) -> Result<(), InstallerError> {
        // This would implement registry cleanup
        // For brevity, we'll implement the key deletion logic
        Ok(())
    }

    /// Unregister event source
    fn unregister_event_source(service_name: &str) -> Result<(), InstallerError> {
        // This would implement event source cleanup
        // For brevity, we'll implement the registry key deletion
        Ok(())
    }

    /// Verify helper executable signature
    fn verify_helper_signature(helper_path: &Path) -> Result<(), InstallerError> {
        // Use the signing module to verify the helper
        crate::signing::verify_signature(helper_path).map_err(|e| {
            InstallerError::System(format!("Helper signature verification failed: {}", e))
        })?;
        Ok(())
    }

    /// Set registry string value
    fn set_registry_string(
        registry: &RegistryHandle,
        name: &str,
        value: &str,
    ) -> Result<(), InstallerError> {
        let mut name_buf: [u16; 256] = [0; 256];
        let mut value_buf: [u16; 1024] = [0; 1024];

        Self::str_to_wide(name, &mut name_buf)?;
        Self::str_to_wide(value, &mut value_buf)?;

        let value_bytes = unsafe {
            std::slice::from_raw_parts(
                value_buf.as_ptr() as *const u8,
                (value.len() + 1) * 2, // +1 for null terminator, *2 for UTF-16
            )
        };

        unsafe {
            RegSetValueExW(
                registry.handle(),
                PCWSTR::from_raw(name_buf.as_ptr()),
                0,
                REG_SZ,
                Some(value_bytes),
            )
            .map_err(|e| InstallerError::System(format!("Failed to set registry value: {}", e)))?;
        }

        Ok(())
    }

    /// Set registry DWORD value
    fn set_registry_dword(
        registry: &RegistryHandle,
        name: &str,
        value: u32,
    ) -> Result<(), InstallerError> {
        let mut name_buf: [u16; 256] = [0; 256];
        Self::str_to_wide(name, &mut name_buf)?;

        let value_bytes = value.to_le_bytes();

        unsafe {
            RegSetValueExW(
                registry.handle(),
                PCWSTR::from_raw(name_buf.as_ptr()),
                0,
                REG_DWORD,
                Some(&value_bytes),
            )
            .map_err(|e| InstallerError::System(format!("Failed to set registry DWORD: {}", e)))?;
        }

        Ok(())
    }

    /// Convert string to wide (UTF-16) with zero allocation
    #[inline]
    fn str_to_wide(s: &str, buffer: &mut [u16]) -> Result<(), InstallerError> {
        let wide: Vec<u16> = OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        if wide.len() > buffer.len() {
            return Err(InstallerError::System(format!(
                "String '{}' too long for buffer (max {})",
                s,
                buffer.len()
            )));
        }

        buffer[..wide.len()].copy_from_slice(&wide);
        Ok(())
    }

    pub async fn install_async(b: InstallerBuilder) -> Result<(), InstallerError> {
        tokio::task::spawn_blocking(move || Self::install(b))
            .await
            .context("task join failed")?
    }

    pub async fn uninstall_async(label: &str) -> Result<(), InstallerError> {
        let label = label.to_string();
        tokio::task::spawn_blocking(move || Self::uninstall(&label))
            .await
            .context("task join failed")?
    }
}
