use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top‑level daemon configuration (mirrors original defaults).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub services_dir: Option<String>,
    pub log_dir:      Option<String>,
    pub default_user: Option<String>,
    pub default_group: Option<String>,
    pub auto_restart: Option<bool>,
    pub services:     Vec<ServiceDefinition>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            services_dir:  Some("/etc/cyrupd/services".into()),
            log_dir:       Some("/var/log/cyrupd".into()),
            default_user:  Some("cyrupd".into()),
            default_group: Some("cyops".into()),
            auto_restart:  Some(true),
            services:      vec![],
        }
    }
}

/// On‑disk TOML description of a single service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    pub name:            String,
    pub description:     Option<String>,
    pub command:         String,
    pub working_dir:     Option<String>,
    #[serde(default)]
    pub env_vars:        HashMap<String, String>,
    #[serde(default)]
    pub auto_restart:    bool,
    pub user:            Option<String>,
    pub group:           Option<String>,
    pub restart_delay_s: Option<u64>,
    #[serde(default)]
    pub depends_on:      Vec<String>,
    #[serde(default)]
    pub health_check:    Option<HealthCheckConfig>,
    #[serde(default)]
    pub log_rotation:    Option<LogRotationConfig>,
    #[serde(default)]
    pub watch_dirs:      Vec<String>,
    pub ephemeral_dir:   Option<String>,
    pub memfs:           Option<MemoryFsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFsConfig {
    pub size_mb:    u32,            // clamped at 2048 elsewhere
    pub mount_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub check_type:       String,   // http | tcp | script
    pub target:           String,
    pub interval_secs:    u64,
    pub timeout_secs:     u64,
    pub retries:          u32,
    pub expected_response: Option<String>,
    #[serde(default)]
    pub on_failure:       Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    pub max_size_mb:  u64,
    pub max_files:    u32,
    pub interval_days:u32,
    pub compress:     bool,
    pub timestamp:    bool,
}