//! Core installer structures and async task management
//!
//! This module provides the core installer functionality with async task handling,
//! certificate generation, and service configuration with zero allocation fast paths
//! and blazing-fast performance.

use crate::install::fluent_voice;
use crate::install::{
    install_daemon_async, uninstall_daemon_async, InstallerBuilder, InstallerError,
};
use crate::signing;
use anyhow::{Context, Result};
use futures::{Future, Stream, StreamExt};
use log::{info, warn};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use std::fs;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Command;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// Async task that can be either a future or a stream
pub enum AsyncTask<T> {
    FutureVariant(Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>),
    StreamVariant(ReceiverStream<T>),
}

impl<T> AsyncTask<T> {
    /// Construct from a future with optimized boxing
    pub fn from_future<F>(fut: F) -> Self
    where
        F: std::future::Future<Output = T> + Send + 'static,
    {
        AsyncTask::FutureVariant(Box::pin(fut))
    }

    /// Construct from a receiver with zero allocation stream wrapping
    pub fn from_receiver(receiver: mpsc::Receiver<T>) -> Self {
        AsyncTask::StreamVariant(ReceiverStream::new(receiver))
    }
}

impl<T> std::future::Future for AsyncTask<T> {
    type Output = T;

    /// Poll the async task with optimized polling
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match &mut *self {
            AsyncTask::FutureVariant(fut) => fut.as_mut().poll(cx),
            AsyncTask::StreamVariant(stream) => {
                // For streams, we expect only one item
                match Pin::new(stream).poll_next(cx) {
                    std::task::Poll::Ready(Some(item)) => std::task::Poll::Ready(item),
                    std::task::Poll::Ready(None) => std::task::Poll::Pending,
                    std::task::Poll::Pending => std::task::Poll::Pending,
                }
            }
        }
    }
}

impl<T, E> AsyncTask<Result<T, E>> {
    /// Convert this async task into a Result after completion with optimized error handling
    pub async fn into_result(self) -> Result<T, E> {
        self.await
    }

    /// Map the success value with fast mapping
    pub fn map<U, F>(self, f: F) -> AsyncTask<Result<U, E>>
    where
        F: FnOnce(T) -> U + Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
        U: Send + 'static,
    {
        match self {
            AsyncTask::FutureVariant(fut) => {
                AsyncTask::from_future(async move {
                    match fut.await {
                        Ok(value) => Ok(f(value)),
                        Err(err) => Err(err),
                    }
                })
            }
            AsyncTask::StreamVariant(stream) => {
                AsyncTask::from_future(async move {
                    let mut stream = stream;
                    match stream.next().await {
                        Some(Ok(value)) => Ok(f(value)),
                        Some(Err(err)) => Err(err),
                        None => panic!("Stream ended without producing a value"),
                    }
                })
            }
        }
    }

    /// Map the error value with fast error mapping
    pub fn map_err<F, G>(self, f: F) -> AsyncTask<Result<T, G>>
    where
        F: FnOnce(E) -> G + Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
        G: Send + 'static,
    {
        match self {
            AsyncTask::FutureVariant(fut) => {
                AsyncTask::from_future(async move {
                    match fut.await {
                        Ok(value) => Ok(value),
                        Err(err) => Err(f(err)),
                    }
                })
            }
            AsyncTask::StreamVariant(stream) => {
                AsyncTask::from_future(async move {
                    let mut stream = stream;
                    match stream.next().await {
                        Some(Ok(value)) => Ok(value),
                        Some(Err(err)) => Err(f(err)),
                        None => panic!("Stream ended without producing a value"),
                    }
                })
            }
        }
    }

    /// Chain another async operation with optimized chaining
    pub fn and_then<U, F, Fut>(self, f: F) -> AsyncTask<Result<U, E>>
    where
        F: FnOnce(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<U, E>> + Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
        U: Send + 'static,
    {
        match self {
            AsyncTask::FutureVariant(fut) => {
                AsyncTask::from_future(async move {
                    match fut.await {
                        Ok(value) => f(value).await,
                        Err(err) => Err(err),
                    }
                })
            }
            AsyncTask::StreamVariant(stream) => {
                AsyncTask::from_future(async move {
                    let mut stream = stream;
                    match stream.next().await {
                        Some(Ok(value)) => f(value).await,
                        Some(Err(err)) => Err(err),
                        None => panic!("Stream ended without producing a value"),
                    }
                })
            }
        }
    }
}

/// Installation progress tracking
#[derive(Debug, Clone)]
pub struct InstallProgress {
    pub step: String,
    pub progress: f32, // 0.0 to 1.0
    pub message: String,
    pub is_error: bool,
}

impl InstallProgress {
    /// Create new progress update with optimized initialization
    pub fn new(step: String, progress: f32, message: String) -> Self {
        Self {
            step,
            progress: progress.clamp(0.0, 1.0),
            message,
            is_error: false,
        }
    }

    /// Create error progress update
    pub fn error(step: String, message: String) -> Self {
        Self {
            step,
            progress: 0.0,
            message,
            is_error: true,
        }
    }

    /// Create completion progress update
    pub fn complete(step: String, message: String) -> Self {
        Self {
            step,
            progress: 1.0,
            message,
            is_error: false,
        }
    }
}

/// Certificate generation configuration
#[derive(Debug, Clone)]
pub struct CertificateConfig {
    pub common_name: String,
    pub organization: String,
    pub country: String,
    pub validity_days: u32,
    pub key_size: usize,
    pub san_entries: Vec<String>,
}

impl Default for CertificateConfig {
    fn default() -> Self {
        Self {
            common_name: "SweetMCP Local CA".to_string(),
            organization: "SweetMCP".to_string(),
            country: "US".to_string(),
            validity_days: 365,
            key_size: 2048,
            san_entries: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "::1".to_string(),
            ],
        }
    }
}

impl CertificateConfig {
    /// Create new certificate config with optimized defaults
    pub fn new(common_name: String) -> Self {
        Self {
            common_name,
            ..Default::default()
        }
    }

    /// Add SAN entry with zero allocation
    pub fn add_san(mut self, san: String) -> Self {
        self.san_entries.push(san);
        self
    }

    /// Set validity period
    pub fn validity_days(mut self, days: u32) -> Self {
        self.validity_days = days;
        self
    }

    /// Set organization
    pub fn organization(mut self, org: String) -> Self {
        self.organization = org;
        self
    }

    /// Set country
    pub fn country(mut self, country: String) -> Self {
        self.country = country;
        self
    }

    /// Set key size
    pub fn key_size(mut self, size: usize) -> Self {
        self.key_size = size;
        self
    }
}

/// Service configuration for installer
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub description: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub env_vars: std::collections::HashMap<String, String>,
    pub auto_restart: bool,
    pub user: Option<String>,
    pub group: Option<String>,
    pub dependencies: Vec<String>,
}

impl ServiceConfig {
    /// Create new service config with optimized initialization
    pub fn new(name: String, command: String) -> Self {
        Self {
            name,
            description: String::new(),
            command,
            args: Vec::new(),
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
            auto_restart: true,
            user: None,
            group: None,
            dependencies: Vec::new(),
        }
    }

    /// Set description
    pub fn description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    /// Add argument
    pub fn arg(mut self, arg: String) -> Self {
        self.args.push(arg);
        self
    }

    /// Add multiple arguments
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args.extend(args);
        self
    }

    /// Set working directory
    pub fn working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Add environment variable
    pub fn env(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }

    /// Set auto restart
    pub fn auto_restart(mut self, restart: bool) -> Self {
        self.auto_restart = restart;
        self
    }

    /// Set user
    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    /// Set group
    pub fn group(mut self, group: String) -> Self {
        self.group = Some(group);
        self
    }

    /// Add dependency
    pub fn depends_on(mut self, service: String) -> Self {
        self.dependencies.push(service);
        self
    }
}

/// Installation context
#[derive(Debug)]
pub struct InstallContext {
    pub exe_path: PathBuf,
    pub config_path: PathBuf,
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
    pub cert_dir: PathBuf,
    pub services: Vec<ServiceConfig>,
    pub certificate_config: CertificateConfig,
    pub progress_tx: Option<mpsc::UnboundedSender<InstallProgress>>,
}

impl InstallContext {
    /// Create new install context with optimized initialization
    pub fn new(exe_path: PathBuf) -> Self {
        let data_dir = Self::get_data_dir();
        let config_path = data_dir.join("config.toml");
        let log_dir = data_dir.join("logs");
        let cert_dir = data_dir.join("certs");

        Self {
            exe_path,
            config_path,
            data_dir,
            log_dir,
            cert_dir,
            services: Vec::new(),
            certificate_config: CertificateConfig::default(),
            progress_tx: None,
        }
    }

    /// Get platform-specific data directory
    fn get_data_dir() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            PathBuf::from("/usr/local/var/sweetmcp")
        }
        #[cfg(target_os = "linux")]
        {
            PathBuf::from("/var/lib/sweetmcp")
        }
        #[cfg(target_os = "windows")]
        {
            PathBuf::from("C:\\ProgramData\\SweetMCP")
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            PathBuf::from("/tmp/sweetmcp")
        }
    }

    /// Add service to installation
    pub fn add_service(&mut self, service: ServiceConfig) {
        self.services.push(service);
    }

    /// Set certificate configuration
    pub fn set_certificate_config(&mut self, config: CertificateConfig) {
        self.certificate_config = config;
    }

    /// Set progress channel
    pub fn set_progress_channel(&mut self, tx: mpsc::UnboundedSender<InstallProgress>) {
        self.progress_tx = Some(tx);
    }

    /// Send progress update with fast progress reporting
    pub fn send_progress(&self, progress: InstallProgress) {
        if let Some(ref tx) = self.progress_tx {
            let _ = tx.send(progress);
        }
    }

    /// Create necessary directories with optimized directory creation
    pub fn create_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.data_dir)
            .with_context(|| format!("Failed to create data directory: {:?}", self.data_dir))?;
        
        fs::create_dir_all(&self.log_dir)
            .with_context(|| format!("Failed to create log directory: {:?}", self.log_dir))?;
        
        fs::create_dir_all(&self.cert_dir)
            .with_context(|| format!("Failed to create cert directory: {:?}", self.cert_dir))?;

        self.send_progress(InstallProgress::new(
            "directories".to_string(),
            0.2,
            "Created installation directories".to_string(),
        ));

        Ok(())
    }

    /// Generate certificates with optimized certificate generation
    pub fn generate_certificates(&self) -> Result<()> {
        let config = &self.certificate_config;
        
        // Generate CA key pair
        let ca_key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)
            .with_context(|| "Failed to generate CA key pair")?;

        // Create CA certificate parameters
        let mut ca_params = CertificateParams::new(vec![config.common_name.clone()]);
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_params.key_pair = Some(ca_key_pair);
        
        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, &config.common_name);
        dn.push(DnType::OrganizationName, &config.organization);
        dn.push(DnType::CountryName, &config.country);
        ca_params.distinguished_name = dn;

        // Set validity period
        let not_before = SystemTime::now();
        let not_after = not_before + Duration::from_secs(config.validity_days as u64 * 24 * 3600);
        ca_params.not_before = not_before;
        ca_params.not_after = not_after;

        // Generate CA certificate
        let ca_cert = rcgen::Certificate::from_params(ca_params)
            .with_context(|| "Failed to generate CA certificate")?;

        // Save CA certificate and key
        let ca_cert_path = self.cert_dir.join("ca.crt");
        let ca_key_path = self.cert_dir.join("ca.key");
        
        fs::write(&ca_cert_path, ca_cert.serialize_pem()?)
            .with_context(|| format!("Failed to write CA certificate to {:?}", ca_cert_path))?;
        
        fs::write(&ca_key_path, ca_cert.serialize_private_key_pem())
            .with_context(|| format!("Failed to write CA key to {:?}", ca_key_path))?;

        // Generate server certificate
        self.generate_server_certificate(&ca_cert)?;

        self.send_progress(InstallProgress::new(
            "certificates".to_string(),
            0.4,
            "Generated SSL certificates".to_string(),
        ));

        Ok(())
    }

    /// Generate server certificate with optimized server cert generation
    fn generate_server_certificate(&self, ca_cert: &rcgen::Certificate) -> Result<()> {
        let config = &self.certificate_config;
        
        // Generate server key pair
        let server_key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)
            .with_context(|| "Failed to generate server key pair")?;

        // Create server certificate parameters
        let mut server_params = CertificateParams::new(vec!["localhost".to_string()]);
        server_params.key_pair = Some(server_key_pair);
        
        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "localhost");
        dn.push(DnType::OrganizationName, &config.organization);
        dn.push(DnType::CountryName, &config.country);
        server_params.distinguished_name = dn;

        // Add SAN entries
        for san in &config.san_entries {
            if san.parse::<std::net::IpAddr>().is_ok() {
                server_params.subject_alt_names.push(SanType::IpAddress(san.parse()?));
            } else {
                server_params.subject_alt_names.push(SanType::DnsName(san.clone()));
            }
        }

        // Set validity period
        let not_before = SystemTime::now();
        let not_after = not_before + Duration::from_secs(config.validity_days as u64 * 24 * 3600);
        server_params.not_before = not_before;
        server_params.not_after = not_after;

        // Generate server certificate
        let server_cert = rcgen::Certificate::from_params(server_params)
            .with_context(|| "Failed to generate server certificate")?;

        // Sign with CA
        let server_cert_pem = server_cert.serialize_pem_with_signer(ca_cert)
            .with_context(|| "Failed to sign server certificate")?;

        // Save server certificate and key
        let server_cert_path = self.cert_dir.join("server.crt");
        let server_key_path = self.cert_dir.join("server.key");
        
        fs::write(&server_cert_path, server_cert_pem)
            .with_context(|| format!("Failed to write server certificate to {:?}", server_cert_path))?;
        
        fs::write(&server_key_path, server_cert.serialize_private_key_pem())
            .with_context(|| format!("Failed to write server key to {:?}", server_key_path))?;

        Ok(())
    }

    /// Validate installation prerequisites with fast validation
    pub fn validate_prerequisites(&self) -> Result<()> {
        // Check if running as appropriate user
        #[cfg(unix)]
        {
            let uid = unsafe { libc::getuid() };
            if uid != 0 {
                return Err(anyhow::anyhow!(
                    "Installation requires root privileges. Please run with sudo."
                ));
            }
        }

        // Check if executable exists and is executable
        if !self.exe_path.exists() {
            return Err(anyhow::anyhow!(
                "Executable not found: {:?}",
                self.exe_path
            ));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&self.exe_path)
                .with_context(|| format!("Failed to read metadata for {:?}", self.exe_path))?;
            
            if metadata.permissions().mode() & 0o111 == 0 {
                return Err(anyhow::anyhow!(
                    "Executable is not executable: {:?}",
                    self.exe_path
                ));
            }
        }

        self.send_progress(InstallProgress::new(
            "validation".to_string(),
            0.1,
            "Validated installation prerequisites".to_string(),
        ));

        Ok(())
    }
}