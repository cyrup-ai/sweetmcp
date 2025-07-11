//! Configuration management for SweetMCP Server

use anyhow::{Context, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc, time::Duration};

/// Main configuration structure for SweetMCP Server
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// JWT signing secret (32 bytes)
    #[serde(skip)]
    pub jwt_secret: Arc<[u8; 32]>,

    /// Maximum concurrent in-flight requests
    pub inflight_max: u64,

    /// List of upstream peer URLs for load balancing
    pub upstreams: Vec<String>,

    /// TCP bind address
    pub tcp_bind: String,

    /// MCP Streamable HTTP bind address
    pub mcp_bind: String,

    /// Unix domain socket path
    pub uds_path: String,

    /// Number of worker threads
    pub workers: usize,

    /// Metrics endpoint bind address
    pub metrics_bind: String,

    /// JWT token expiry duration
    pub jwt_expiry: Duration,

    /// Health check interval for peers
    pub health_check_interval: Duration,

    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,

    /// Request timeout duration
    pub request_timeout: Duration,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
}

/// Rate limiting configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per second per user
    pub per_user_rps: u32,

    /// Requests per second per IP
    pub per_ip_rps: u32,

    /// Burst capacity
    pub burst_capacity: u32,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        // JWT secret handling - auto-generate in dev mode if not provided
        let secret_b64 = match env::var("SWEETMCP_JWT_SECRET") {
            Ok(secret) => secret,
            Err(_) => {
                // Check if in development mode
                let is_dev_mode = cfg!(debug_assertions) || env::var("SWEETMCP_DEV_MODE").is_ok();

                if is_dev_mode {
                    // Generate random 32 bytes
                    let mut rng = rand::rng();
                    let mut secret_bytes = [0u8; 32];
                    rng.fill(&mut secret_bytes);
                    let generated_secret = base64_url::encode(&secret_bytes);

                    // Print warning in dev mode only
                    eprintln!();
                    eprintln!("⚠️  Development Mode: Auto-generated JWT secret");
                    eprintln!("   SWEETMCP_JWT_SECRET={}", generated_secret);
                    eprintln!("   For production, set this environment variable explicitly!");
                    eprintln!();

                    generated_secret
                } else {
                    return Err(anyhow::anyhow!(
                        "SWEETMCP_JWT_SECRET environment variable is required"
                    ));
                }
            }
        };

        let secret_vec = base64_url::decode(&secret_b64)
            .context("Invalid base64 encoding in SWEETMCP_JWT_SECRET")?;

        anyhow::ensure!(
            secret_vec.len() == 32,
            "JWT secret must be exactly 32 bytes, got {} bytes",
            secret_vec.len()
        );

        let mut secret = [0u8; 32];
        secret.copy_from_slice(&secret_vec);

        // Parse other configuration values with defaults
        let inflight_max = env::var("SWEETMCP_INFLIGHT_MAX")
            .unwrap_or_else(|_| "400".to_string())
            .parse()
            .context("Invalid SWEETMCP_INFLIGHT_MAX value")?;

        let upstreams: Vec<String> = env::var("SWEETMCP_UPSTREAMS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        let tcp_bind = env::var("SWEETMCP_TCP_BIND").unwrap_or_else(|_| "0.0.0.0:8443".to_string());

        let mcp_bind =
            env::var("SWEETMCP_MCP_BIND").unwrap_or_else(|_| "0.0.0.0:33399".to_string());

        let uds_path = env::var("SWEETMCP_UDS_PATH").unwrap_or_else(|_| {
            let xdg_config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
                let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                format!("{}/.config", home)
            });
            format!("{}/sweetmcp/sugora.sock", xdg_config)
        });

        let workers = env::var("SWEETMCP_WORKERS")
            .unwrap_or_else(|_| "4".to_string())
            .parse()
            .context("Invalid SWEETMCP_WORKERS value")?;

        let metrics_bind =
            env::var("SWEETMCP_METRICS_BIND").unwrap_or_else(|_| "127.0.0.1:9090".to_string());

        let jwt_expiry_str = env::var("SWEETMCP_JWT_EXPIRY").unwrap_or_else(|_| "1h".to_string());
        let jwt_expiry =
            parse_duration(&jwt_expiry_str).context("Invalid SWEETMCP_JWT_EXPIRY format")?;

        let health_check_interval_str =
            env::var("SWEETMCP_HEALTH_CHECK_INTERVAL").unwrap_or_else(|_| "5s".to_string());
        let health_check_interval = parse_duration(&health_check_interval_str)
            .context("Invalid SWEETMCP_HEALTH_CHECK_INTERVAL format")?;

        let circuit_breaker_threshold = env::var("SWEETMCP_CIRCUIT_BREAKER_THRESHOLD")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .context("Invalid SWEETMCP_CIRCUIT_BREAKER_THRESHOLD value")?;

        let request_timeout_str =
            env::var("SWEETMCP_REQUEST_TIMEOUT").unwrap_or_else(|_| "30s".to_string());
        let request_timeout = parse_duration(&request_timeout_str)
            .context("Invalid SWEETMCP_REQUEST_TIMEOUT format")?;

        // Rate limiting configuration
        let per_user_rps = env::var("SWEETMCP_RATE_LIMIT_USER_RPS")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .context("Invalid SWEETMCP_RATE_LIMIT_USER_RPS value")?;

        let per_ip_rps = env::var("SWEETMCP_RATE_LIMIT_IP_RPS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .context("Invalid SWEETMCP_RATE_LIMIT_IP_RPS value")?;

        let burst_capacity = env::var("SWEETMCP_RATE_LIMIT_BURST")
            .unwrap_or_else(|_| "50".to_string())
            .parse()
            .context("Invalid SWEETMCP_RATE_LIMIT_BURST value")?;

        let rate_limit = RateLimitConfig {
            per_user_rps,
            per_ip_rps,
            burst_capacity,
        };

        Ok(Self {
            jwt_secret: Arc::new(secret),
            inflight_max,
            upstreams,
            tcp_bind,
            mcp_bind,
            uds_path,
            workers,
            metrics_bind,
            jwt_expiry,
            health_check_interval,
            circuit_breaker_threshold,
            request_timeout,
            rate_limit,
        })
    }

    /// Validate the configuration
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<()> {
        if self.inflight_max == 0 {
            anyhow::bail!("inflight_max must be greater than 0");
        }

        if self.workers == 0 {
            anyhow::bail!("workers must be greater than 0");
        }

        if self.jwt_expiry.as_secs() == 0 {
            anyhow::bail!("jwt_expiry must be greater than 0");
        }

        if self.health_check_interval.as_secs() == 0 {
            anyhow::bail!("health_check_interval must be greater than 0");
        }

        if self.request_timeout.as_secs() == 0 {
            anyhow::bail!("request_timeout must be greater than 0");
        }

        // Validate upstream URLs
        for upstream in &self.upstreams {
            url::Url::parse(upstream)
                .with_context(|| format!("Invalid upstream URL: {}", upstream))?;
        }

        Ok(())
    }
}

/// Parse duration strings like "1h", "30m", "5s"
fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();

    if s.is_empty() {
        anyhow::bail!("Duration string cannot be empty");
    }

    let (number_part, unit_part) = if let Some(pos) = s.find(|c: char| c.is_alphabetic()) {
        (&s[..pos], &s[pos..])
    } else {
        anyhow::bail!("Duration must include a unit (s, m, h, d)");
    };

    let number: u64 = number_part.parse().context("Invalid number in duration")?;

    let duration = match unit_part {
        "s" | "sec" | "second" | "seconds" => Duration::from_secs(number),
        "m" | "min" | "minute" | "minutes" => Duration::from_secs(number * 60),
        "h" | "hr" | "hour" | "hours" => Duration::from_secs(number * 3600),
        "d" | "day" | "days" => Duration::from_secs(number * 86400),
        _ => anyhow::bail!("Unknown duration unit: {}", unit_part),
    };

    Ok(duration)
}
