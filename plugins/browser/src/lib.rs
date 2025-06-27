pub mod agent;
pub mod browser;
pub mod controller;
pub mod utils;

use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use kalosm::language::*;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info, Level};
use tracing_subscriber::FmtSubscriber;

// Import error utilities
pub use crate::utils::errors::UtilsError;

use crate::agent::{Agent, SystemPrompt, AgentMessagePrompt, AgentHistoryList};
use crate::browser::{BrowserContext, BrowserContextConfig, Browser};
use crate::controller::Controller;
use crate::utils::{AgentState, get_llm_model};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub headless: bool,
    pub disable_security: bool,
    pub window: WindowConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub name: String,
    pub temperature: f32,
    pub max_steps: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeysConfig {
    pub openai: Option<String>,
    pub anthropic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub browser: BrowserConfig,
    pub model: ModelConfig,
    #[serde(default)]
    pub api_keys: Option<ApiKeysConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            browser: BrowserConfig {
                headless: true,
                disable_security: false,
                window: WindowConfig {
                    width: 1280,
                    height: 720,
                },
            },
            model: ModelConfig {
                provider: "openai".to_string(),
                name: String::new(),
                temperature: 0.7,
                max_steps: 20,
            },
            api_keys: None,
        }
    }
}

/// Initialize tracing for the application
pub fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Error setting tracing default: {}", e);
    }
}

/// Load configuration from YAML file and environment variables
/// Environment variables take precedence over YAML config
pub fn load_config() -> (String, String, f32, usize) {
    // Load default config
    let mut config = load_yaml_config().unwrap_or_else(|_| {
        info!("No valid config.yaml found, using default configuration");
        Config {
            browser: BrowserConfig {
                headless: true,
                disable_security: false,
                window: WindowConfig {
                    width: 1280,
                    height: 720,
                },
            },
            model: ModelConfig {
                provider: "openai".to_string(),
                name: String::new(),
                temperature: 0.7,
                max_steps: 20,
            },
            api_keys: None,
        }
    });
    
    // Override with environment variables if present
    let model_provider = env::var("MCP_MODEL_PROVIDER")
        .unwrap_or(config.model.provider);
        
    let model_name = env::var("MCP_MODEL_NAME")
        .unwrap_or(config.model.name);
        
    let temperature = env::var("MCP_TEMPERATURE")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(config.model.temperature);
        
    let max_steps = env::var("MCP_MAX_STEPS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(config.model.max_steps);
        
    (model_provider, model_name, temperature, max_steps)
}

/// Load configuration from YAML file
fn load_yaml_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = Path::new("config.yaml");
    
    if !config_path.exists() {
        return Err("config.yaml not found".into());
    }
    
    let config_content = fs::read_to_string(config_path)?;
    let config: Config = serde_yaml::from_str(&config_content)?;
    
    Ok(config)
}

/// Initialize the browser
pub async fn init_browser() -> Result<Arc<BrowserContext>, UtilsError> {
    // Load default config
    let config = load_yaml_config().unwrap_or_else(|_| {
        info!("No valid config.yaml found, using default browser configuration");
        Config {
            browser: BrowserConfig {
                headless: true,
                disable_security: false,
                window: WindowConfig {
                    width: 1280,
                    height: 720,
                },
            },
            model: ModelConfig {
                provider: "openai".to_string(),
                name: String::new(),
                temperature: 0.7,
                max_steps: 20,
            },
            api_keys: None,
        }
    });
    
    // Override with environment variables if present
    let headless = env::var("BROWSER_HEADLESS")
        .ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(config.browser.headless);
        
    let disable_security = env::var("BROWSER_DISABLE_SECURITY")
        .ok()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(config.browser.disable_security);
        
    let window_width = env::var("BROWSER_WINDOW_WIDTH")
        .ok()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(config.browser.window.width);
        
    let window_height = env::var("BROWSER_WINDOW_HEIGHT")
        .ok()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(config.browser.window.height);
        
    let browser = Browser::new(headless, disable_security).await
        .map_err(|e| {
            error!("Failed to initialize browser: {}", e);
            UtilsError::BrowserError(format!("Failed to initialize browser: {}", e))
        })?;
        
    let context_config = BrowserContextConfig {
        browser_window_size: browser::BrowserWindowSize {
            width: window_width,
            height: window_height,
        },
        ..Default::default()
    };
    
    let context = browser.new_context(context_config).await
        .map_err(|e| {
            error!("Failed to create browser context: {}", e);
            UtilsError::BrowserError(format!("Failed to create browser context: {}", e))
        })?;
        
    Ok(Arc::new(context))
}

/// Initialize the agent
pub fn init_agent(
    task: &str,
    add_infos: &str,
    use_vision: bool,
    llm: Llama,
    browser: Arc<BrowserContext>,
) -> Result<Agent, UtilsError> {
    let controller = Controller::new();
    let system_prompt = SystemPrompt::new();
    let agent_prompt = AgentMessagePrompt::new();
    let max_actions_per_step = 3;
    let agent_state = Arc::new(Mutex::new(AgentState::new()));
    let tool_calling_method = "action".to_string();
    
    Agent::new(
        task,
        add_infos,
        use_vision,
        llm,
        browser,
        controller,
        system_prompt,
        agent_prompt,
        max_actions_per_step,
        agent_state,
        &tool_calling_method,
    )
    .map_err(|e| {
        error!("Failed to create agent: {}", e);
        UtilsError::UnexpectedError(format!("Failed to create agent: {}", e))
    })
}

/// Run the browser agent with the specified task
pub async fn run_browser_agent(
    task: &str,
    add_infos: &str,
    use_vision: bool,
) -> Result<AgentHistoryList, UtilsError> {
    let (model_provider, model_name, temperature, max_steps) = load_config();
    
    let llm = get_llm_model(
        &model_provider,
        Some(&model_name),
        temperature,
    )
    .map_err(|e| {
        error!("Failed to initialize LLM: {}", e);
        UtilsError::LlmError(format!("Failed to initialize LLM: {}", e))
    })?;
    
    let browser = init_browser().await?;
    
    let agent = init_agent(
        task,
        add_infos,
        use_vision,
        llm,
        browser.clone(),
    )?;
    
    info!("Running agent for task: {}", task);
    let history = agent.run(max_steps).await
        .map_err(|e| {
            error!("Agent execution failed: {}", e);
            UtilsError::UnexpectedError(format!("Agent execution failed: {}", e))
        })?;
    
    // Clean up browser
    if let Err(e) = browser.close().await {
        error!("Error closing browser: {}", e);
    }
    
    Ok(history)
}

