use std::env;
use std::sync::{Arc, Mutex};

use rmcp::{
    server::{ServerHandler, ServiceExt},
    tool,
    transport::stdio,
};
use tokio::sync::oneshot;
use tracing::{error, info};

use crate::agent::{Agent, AgentMessagePrompt, SystemPrompt};
use crate::browser::{Browser, BrowserConfig, BrowserContextConfig, BrowserContextWindowSize, BrowserContext};
use crate::controller::Controller;
use crate::utils::{AgentState, utils};

// Global references for single "running agent" approach using thread-safe wrappers
lazy_static::lazy_static! {
    static ref GLOBAL_AGENT: Mutex<Option<Agent>> = Mutex::new(None);
    static ref GLOBAL_BROWSER: Mutex<Option<Browser>> = Mutex::new(None);
    static ref GLOBAL_BROWSER_CONTEXT: Mutex<Option<Arc<BrowserContext>>> = Mutex::new(None);
    static ref GLOBAL_AGENT_STATE: Arc<tokio::sync::Mutex<AgentState>> = Arc::new(tokio::sync::Mutex::new(AgentState::new()));
}

// Helper function to get boolean from environment variable
fn get_env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .to_lowercase()
        .as_str()
        .matches(&["true", "1", "yes"][..])
        .next()
        .is_some()
}

// Safe cleanup function to release browser resources
async fn safe_cleanup() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let agent_state = GLOBAL_AGENT_STATE.clone();
    let mut agent_state = agent_state.lock().await;
    
    // Request agent stop
    if let Err(e) = agent_state.request_stop() {
        error!("Error requesting agent stop: {}", e);
    }
    
    // Close browser context
    if let Ok(guard) = GLOBAL_BROWSER_CONTEXT.lock() {
        if let Some(ctx) = guard.clone() {
        let mut ctx = ctx.lock().await;
        if let Err(e) = ctx.close().await {
            error!("Error closing browser context: {}", e);
        }
    }
    
    // Close browser
    if let Ok(mut guard) = GLOBAL_BROWSER.lock() {
        if let Some(browser) = &mut *guard {
        if let Err(e) = browser.close().await {
            error!("Error closing browser: {}", e);
        }
    }
    
    // Reset global variables
    if let Ok(mut guard) = GLOBAL_BROWSER.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = GLOBAL_BROWSER_CONTEXT.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = GLOBAL_AGENT.lock() {
        *guard = None;
    }
    *agent_state = AgentState::new();
    
    Ok(())
}

pub struct BrowserAgentServer;

#[tool(tool_box)]
impl BrowserAgentServer {
    #[tool(description = "Run browser automation agent with specified task")]
    async fn run_browser_agent(
        &self,
        #[tool(param)] task: String,
        #[tool(param)] add_infos: Option<String>,
    ) -> Result<String, String> {
        // Clone AgentState for async block
        let agent_state = GLOBAL_AGENT_STATE.clone();
        let mut agent_state_guard = agent_state.lock().await;
        
        // Clear any previous agent stop signals
        agent_state_guard.clear_stop();
        
        // Get browser configuration from environment
        let headless = get_env_bool("BROWSER_HEADLESS", true);
        let disable_security = get_env_bool("BROWSER_DISABLE_SECURITY", false);
        let chrome_instance_path = env::var("BROWSER_CHROME_INSTANCE_PATH").ok();
        let window_w = env::var("BROWSER_WINDOW_WIDTH")
            .unwrap_or_else(|_| "1280".to_string())
            .parse::<i32>()
            .unwrap_or(1280);
        let window_h = env::var("BROWSER_WINDOW_HEIGHT")
            .unwrap_or_else(|_| "720".to_string())
            .parse::<i32>()
            .unwrap_or(720);
        
        // Get agent configuration
        let model_provider = env::var("MCP_MODEL_PROVIDER").unwrap_or_else(|_| "anthropic".to_string());
        let model_name = env::var("MCP_MODEL_NAME").unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());
        let temperature = env::var("MCP_TEMPERATURE")
            .unwrap_or_else(|_| "0.7".to_string())
            .parse::<f32>()
            .unwrap_or(0.7);
        let max_steps = env::var("MCP_MAX_STEPS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .unwrap_or(100);
        let use_vision = get_env_bool("MCP_USE_VISION", true);
        let max_actions_per_step = env::var("MCP_MAX_ACTIONS_PER_STEP")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<usize>()
            .unwrap_or(5);
        let tool_calling_method = env::var("MCP_TOOL_CALLING_METHOD").unwrap_or_else(|_| "auto".to_string());
        
        // Configure browser window size
        let extra_chromium_args = vec![format!("--window-size={},{}", window_w, window_h)];
        
        // Initialize browser if needed
        let mut browser_guard = GLOBAL_BROWSER.lock().map_err(|e| {
            error!("Failed to acquire browser lock: {}", e);
            UtilsError::UnexpectedError(format!("Failed to acquire browser lock: {}", e))
        })?;
        if browser_guard.is_none() {
            match Browser::new(BrowserConfig {
                headless,
                disable_security,
                chrome_instance_path,
                extra_chromium_args,
            }).await {
                Ok(browser) => *browser_guard = Some(browser),
                Err(e) => return Err(format!("Failed to initialize browser: {}", e)),
            }
        }
        let browser = match browser_guard.as_ref() {
            Some(b) => b.clone(),
            None => {
                return Err(UtilsError::UnexpectedError("Browser not initialized".into()))
            }
        };
        drop(browser_guard);
        
        // Initialize browser context if needed
        let mut context_guard = GLOBAL_BROWSER_CONTEXT.lock().map_err(|e| {
            error!("Failed to acquire browser context lock: {}", e);
            UtilsError::UnexpectedError(format!("Failed to acquire browser context lock: {}", e))
        })?;
        if context_guard.is_none() {
            let browser_context_config = BrowserContextConfig {
                trace_path: env::var("BROWSER_TRACE_PATH").ok(),
                save_recording_path: env::var("BROWSER_RECORDING_PATH").ok(),
                no_viewport: false,
                browser_window_size: BrowserContextWindowSize {
                    width: window_w,
                    height: window_h,
                },
            };
            
            match browser.new_context(browser_context_config).await {
                Ok(context) => {
                    *context_guard = Some(Arc::new(tokio::sync::Mutex::new(context)));
                },
                Err(e) => return Err(format!("Failed to create browser context: {}", e)),
            }
        }
        let context = match context_guard.clone() {
            Some(c) => c,
            None => {
                return Err(UtilsError::UnexpectedError("Browser context not initialized".into()))
            }
        };
        drop(context_guard);
        
        // Prepare LLM
        let llm = match utils::utils::llama() {
            Ok(model) => model,
            Err(e) => return Err(format!("Failed to initialize LLM: {}", e)),
        };
        
        // Create controller and agent
        let controller = Controller::new();
        let add_infos_str = add_infos.unwrap_or_default();
        
        let agent = match Agent::new(
            &task,
            &add_infos_str,
            use_vision,
            llm,
            browser.clone(),
            context.clone(),
            controller,
            SystemPrompt::new(),
            AgentMessagePrompt::new(),
            max_actions_per_step,
            agent_state.clone(),
            &tool_calling_method,
        ) {
            Ok(agent) => agent,
            Err(e) => return Err(format!("Failed to create agent: {}", e)),
        };
        
        let mut agent_guard = GLOBAL_AGENT.lock().map_err(|e| {
            error!("Failed to acquire agent lock: {}", e);
            UtilsError::UnexpectedError(format!("Failed to acquire agent lock: {}", e))
        })?;
        *agent_guard = Some(agent.clone());
        drop(agent_guard);
        
        // Run agent
        let result = match agent.run(max_steps).await {
            Ok(history) => {
                if let Some(final_result) = history.final_result() {
                    final_result
                } else {
                    format!("No final result. Possibly incomplete. {:?}", history)
                }
            },
            Err(e) => {
                error!("Agent run error: {}", e);
                format!("Error during task execution: {}", e)
            }
        };
        
        // Schedule cleanup
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            if let Err(e) = safe_cleanup().await {
                error!("Error during cleanup: {}", e);
            }
            let _ = tx.send(());
        });
        
        // Return result without waiting for cleanup
        Ok(result)
    }
}

impl ServerHandler for BrowserAgentServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo {
            name: "mcp_server_browser_use".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            capabilities: rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = BrowserAgentServer;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

pub fn main() {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
        
    // Run the async server in the Tokio runtime
    if let Err(e) = tokio::runtime::Runtime::new()
        .map_err(|e| {
            error!("Failed to register tool: {}", e);
            UtilsError::ServerError(format!("Failed to register tool: {}", e))
        })?
        .block_on(run_server())
    {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}
    
