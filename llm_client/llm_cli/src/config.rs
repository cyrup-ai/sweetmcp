use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default provider to use
    pub default_provider: Option<String>,
    
    /// Default model for each provider
    pub default_models: HashMap<String, String>,
    
    /// API keys for providers
    pub api_keys: HashMap<String, String>,
    
    /// API base URLs for providers
    pub api_base_urls: HashMap<String, String>,
    
    /// Default generation parameters
    pub defaults: GenerationDefaults,
    
    /// Chat-specific settings
    pub chat: ChatConfig,
    
    /// Path to the config file
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationDefaults {
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    /// Show token usage after each response
    pub show_token_usage: bool,
    
    /// Show generation time
    pub show_timing: bool,
    
    /// History file location
    pub history_file: Option<PathBuf>,
    
    /// Maximum history entries to keep
    pub max_history: usize,
    
    /// System prompt for chat mode
    pub system_prompt: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_provider: None,
            default_models: HashMap::from([
                ("openai".to_string(), "gpt-4-turbo-preview".to_string()),
                ("anthropic".to_string(), "claude-3-sonnet-20240229".to_string()),
                ("llama_cpp".to_string(), "llama-3.2-3b-instruct".to_string()),
                ("mistral_rs".to_string(), "mistral-7b-instruct-v0.3".to_string()),
            ]),
            api_keys: HashMap::new(),
            api_base_urls: HashMap::new(),
            defaults: GenerationDefaults {
                temperature: 0.7,
                max_tokens: Some(2048),
                top_p: None,
                frequency_penalty: Some(0.0),
                presence_penalty: Some(0.0),
            },
            chat: ChatConfig {
                show_token_usage: true,
                show_timing: true,
                history_file: dirs::data_dir().map(|d| d.join("llm-cli").join("history.json")),
                max_history: 1000,
                system_prompt: None,
            },
            config_path: None,
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub async fn load(path: Option<&Path>) -> Result<Self> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path()?,
        };
        
        let mut config = if config_path.exists() {
            let contents = fs::read_to_string(&config_path).await
                .context("Failed to read config file")?;
            toml::from_str::<Config>(&contents)
                .context("Failed to parse config file")?
        } else {
            Config::default()
        };
        
        config.config_path = Some(config_path);
        
        // Load API keys from environment if not in config
        config.load_env_vars();
        
        Ok(config)
    }
    
    /// Save configuration to file
    pub async fn save(&self) -> Result<()> {
        let path = self.config_path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Config path not set"))?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .context("Failed to create config directory")?;
        }
        
        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(path, contents).await
            .context("Failed to write config file")?;
        
        Ok(())
    }
    
    /// Initialize a new config file
    pub async fn init_config_file(path: Option<&Path>) -> Result<PathBuf> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path()?,
        };
        
        if config_path.exists() {
            anyhow::bail!("Config file already exists at: {}", config_path.display());
        }
        
        let mut config = Config::default();
        config.config_path = Some(config_path.clone());
        config.save().await?;
        
        Ok(config_path)
    }
    
    /// Set a configuration value by key
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "default_provider" => self.default_provider = Some(value.to_string()),
            k if k.starts_with("default_models.") => {
                let provider = k.strip_prefix("default_models.")
                    .ok_or_else(|| anyhow::anyhow!("Invalid key"))?;
                self.default_models.insert(provider.to_string(), value.to_string());
            }
            k if k.starts_with("api_keys.") => {
                let provider = k.strip_prefix("api_keys.")
                    .ok_or_else(|| anyhow::anyhow!("Invalid key"))?;
                self.api_keys.insert(provider.to_string(), value.to_string());
            }
            "defaults.temperature" => self.defaults.temperature = value.parse()?,
            "defaults.max_tokens" => self.defaults.max_tokens = Some(value.parse()?),
            "chat.show_token_usage" => self.chat.show_token_usage = value.parse()?,
            "chat.show_timing" => self.chat.show_timing = value.parse()?,
            _ => anyhow::bail!("Unknown configuration key: {}", key),
        }
        Ok(())
    }
    
    /// Get the default config path
    fn default_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("llm-cli").join("config.toml"))
    }
    
    /// Load API keys from environment variables
    fn load_env_vars(&mut self) {
        dotenvy::dotenv().ok();
        
        // OpenAI
        if self.api_keys.get("openai").is_none() {
            if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                self.api_keys.insert("openai".to_string(), key);
            }
        }
        
        // Anthropic
        if self.api_keys.get("anthropic").is_none() {
            if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
                self.api_keys.insert("anthropic".to_string(), key);
            }
        }
        
        // Generic providers might use custom env vars
    }
}