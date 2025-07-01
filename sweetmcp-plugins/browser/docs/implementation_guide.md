# Browser Automation MCP: Implementation Guide

This guide outlines the core components needed to implement the browser automation MCP in Rust, following project conventions and patterns exhibited in the sample code.

## Architecture Overview

```
┌────────────────┐     ┌────────────────┐     ┌────────────────┐
│                │     │                │     │                │
│  MCP Server    │────▶│  Agent Brain   │────▶│  Controller    │
│                │     │                │     │                │
└────────────────┘     └────────────────┘     └────────────────┘
        │                     │                      │
        │                     │                      │
        ▼                     ▼                      ▼
┌────────────────┐     ┌────────────────┐     ┌────────────────┐
│                │     │                │     │                │
│  Transport     │     │  Memory Store  │     │  Browser       │
│                │     │                │     │                │
└────────────────┘     └────────────────┘     └────────────────┘
```

## 1. Core Interfaces and Types

### Domain-Specific Types

```rust
// src/types.rs
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub step_number: usize,
    pub max_steps: usize,
    pub task: String,
    pub add_infos: Option<String>,
    pub memory: Vec<String>,
    pub task_progress: String,
    pub future_plans: String,
    pub stop_requested: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserState {
    pub url: String,
    pub tabs: Vec<TabInfo>,
    pub element_tree: String,
    pub screenshot: Option<String>,
    pub pixels_above: usize,
    pub pixels_below: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserAction {
    pub action_type: String,
    pub element_index: Option<usize>,
    pub text_input: Option<String>,
    pub url: Option<String>,
    pub keys: Option<String>,
    pub direction: Option<String>,
    pub amount: Option<usize>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub current_state: AgentBrain,
    pub actions: Vec<BrowserAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBrain {
    pub prev_action_evaluation: String,
    pub important_contents: String,
    pub task_progress: String,
    pub future_plans: String,
    pub thought: String,
    pub summary: String,
}

#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("Browser connection error: {0}")]
    ConnectionError(String),
    
    #[error("Navigation error: {0}")]
    NavigationError(String),
    
    #[error("Element interaction error: {0}")]
    ElementError(String),
    
    #[error("Screenshot error: {0}")]
    ScreenshotError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Model error: {0}")]
    ModelError(String),
    
    #[error("Browser error: {0}")]
    BrowserError(#[from] BrowserError),
    
    #[error("Parser error: {0}")]
    ParserError(String),
    
    #[error("Maximum steps exceeded")]
    MaxStepsExceeded,
    
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),
}
```

## 2. Custom Domain-Specific Return Types

```rust
// src/streams.rs
use crate::types::{AgentOutput, BrowserAction, BrowserError, AgentError};
use tokio::sync::mpsc::{Receiver, Sender};

// Domain-specific return type for agent operations
pub struct AgentStream {
    receiver: Receiver<Result<AgentOutput, AgentError>>,
}

impl AgentStream {
    pub fn new(receiver: Receiver<Result<AgentOutput, AgentError>>) -> Self {
        Self { receiver }
    }
    
    // Synchronous interface that awaits internally
    pub fn next(&mut self) -> Option<Result<AgentOutput, AgentError>> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async { self.receiver.recv().await })
    }
    
    // Additional utility methods that hide async complexity
    pub fn collect_all(&mut self) -> Vec<Result<AgentOutput, AgentError>> {
        let mut results = Vec::new();
        while let Some(item) = self.next() {
            results.push(item);
        }
        results
    }
}

// Domain-specific return type for browser operations
pub struct ActionResultStream {
    receiver: Receiver<Result<BrowserAction, BrowserError>>,
}

impl ActionResultStream {
    pub fn new(receiver: Receiver<Result<BrowserAction, BrowserError>>) -> Self {
        Self { receiver }
    }
    
    // Synchronous interface that awaits internally
    pub fn next(&mut self) -> Option<Result<BrowserAction, BrowserError>> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async { self.receiver.recv().await })
    }
}
```

## 3. Browser Component Implementation

```rust
// src/browser/browser.rs
use chromiumoxide::{Browser as ChromeBrowser, BrowserConfig, Page};
use tokio::sync::mpsc;

use crate::types::{BrowserState, BrowserError, TabInfo, BrowserAction};
use crate::streams::ActionResultStream;

pub struct Browser {
    inner: ChromeBrowser,
}

impl Browser {
    // No async in the public interface
    pub fn new(headless: bool) -> Self {
        let rt = tokio::runtime::Handle::current();
        
        // Configure browser
        let config = BrowserConfig::builder()
            .with_head(!headless)
            .build()
            .unwrap();
            
        // Launch browser - async happens internally
        let (browser, _) = rt.block_on(async {
            ChromeBrowser::launch(config).await.unwrap()
        });
        
        Self { inner: browser }
    }
    
    // Return domain-specific type, not a future
    pub fn execute_action(&self, action: BrowserAction) -> ActionResultStream {
        let (tx, rx) = mpsc::channel(1);
        let browser = self.inner.clone();
        let action_clone = action.clone();
        
        // Spawn task to handle async work
        tokio::spawn(async move {
            let result = match action_clone.action_type.as_str() {
                "go_to_url" => execute_go_to_url(&browser, action_clone).await,
                "click_element" => execute_click_element(&browser, action_clone).await,
                "input_text" => execute_input_text(&browser, action_clone).await,
                // ... other action types
                _ => Err(BrowserError::InvalidArgument(format!(
                    "Unknown action type: {}", action_clone.action_type
                ))),
            };
            
            let _ = tx.send(result).await;
        });
        
        // Return domain-specific stream, not a future
        ActionResultStream::new(rx)
    }
    
    pub fn capture_state(&self) -> Result<BrowserState, BrowserError> {
        let (tx, rx) = mpsc::channel(1);
        let browser = self.inner.clone();
        
        tokio::spawn(async move {
            let result = capture_browser_state(&browser).await;
            let _ = tx.send(result).await;
        });
        
        // Synchronous interface that awaits internally
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async { 
            rx.recv()
                .await
                .unwrap_or(Err(BrowserError::TimeoutError(
                    "Failed to receive browser state".into()
                )))
        })
    }
}

// Helper functions implementing the actual async work
async fn execute_go_to_url(
    browser: &ChromeBrowser, 
    action: BrowserAction
) -> Result<BrowserAction, BrowserError> {
    let url = action.url.clone().ok_or_else(|| 
        BrowserError::InvalidArgument("URL is required for go_to_url action".into())
    )?;
    
    let page = browser.pages().await.get(0).cloned().ok_or_else(|| 
        BrowserError::NavigationError("No pages available".into())
    )?;
    
    page.goto(url).await.map_err(|e| 
        BrowserError::NavigationError(format!("Failed to navigate: {}", e))
    )?;
    
    Ok(action)
}

// ... other action implementation functions

async fn capture_browser_state(browser: &ChromeBrowser) -> Result<BrowserState, BrowserError> {
    let pages = browser.pages().await;
    let active_page = pages.get(0).cloned().ok_or_else(|| 
        BrowserError::ConnectionError("No pages available".into())
    )?;
    
    // Get current URL
    let url = active_page.url().await.map_err(|e| 
        BrowserError::NavigationError(format!("Failed to get URL: {}", e))
    )?;
    
    // Capture tabs
    let tabs = pages.iter().map(|page| {
        let id = page.target_id().to_string();
        let title = "".to_string(); // Would need to fetch asynchronously
        let url = "".to_string();   // Would need to fetch asynchronously
        let is_active = page.target_id() == active_page.target_id();
        
        TabInfo { id, title, url, is_active }
    }).collect();
    
    // Capture screenshot
    let screenshot = active_page.screenshot(false).await.ok().map(|data| {
        base64::encode(data)
    });
    
    // Capture DOM structure (simplified)
    let element_tree = active_page.evaluate("document.body.innerHTML")
        .await
        .map(|res| res.to_string())
        .unwrap_or_else(|_| "".to_string());
    
    Ok(BrowserState {
        url: url.to_string(),
        tabs,
        element_tree,
        screenshot,
        pixels_above: 0,  // Would compute from scroll position
        pixels_below: 0,  // Would compute from scroll position
    })
}
```

## 4. Agent Implementation

```rust
// src/agent/agent.rs
use kalosm::language::*;
use tokio::sync::mpsc;

use crate::browser::Browser;
use crate::types::{AgentState, BrowserState, AgentOutput, AgentError, BrowserAction};
use crate::streams::AgentStream;

pub struct Agent {
    browser: Browser,
    model: LlamaModel,
    state: AgentState,
}

impl Agent {
    pub fn new(browser: Browser, task: String, add_infos: Option<String>) -> Self {
        // Synchronous interface that awaits internally
        let rt = tokio::runtime::Handle::current();
        let model = rt.block_on(async {
            Llama::builder()
                .with_source(LlamaSource::phi_3())
                .build()
                .await
                .unwrap()
        });
        
        let state = AgentState {
            step_number: 0,
            max_steps: 50,
            task,
            add_infos,
            memory: Vec::new(),
            task_progress: String::new(),
            future_plans: String::new(),
            stop_requested: false,
        };
        
        Self { browser, model, state }
    }
    
    // Return domain-specific stream, not a future
    pub fn run(&mut self) -> AgentStream {
        let (tx, rx) = mpsc::channel(100);
        let browser = self.browser;
        let mut state = self.state.clone();
        let model = Llama::builder()
            .with_source(LlamaSource::phi_3())
            .build();
            
        // Spawn task to handle async work
        tokio::spawn(async move {
            let model = match model.await {
                Ok(m) => m,
                Err(e) => {
                    let _ = tx.send(Err(AgentError::ModelError(format!(
                        "Failed to initialize model: {}", e
                    )))).await;
                    return;
                }
            };
            
            let mut chat = model.chat().with_system_prompt(SYSTEM_PROMPT);
            
            while state.step_number < state.max_steps && !state.stop_requested {
                // Get browser state
                let browser_state = match browser.capture_state() {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.send(Err(AgentError::BrowserError(e))).await;
                        continue;
                    }
                };
                
                // Create user message
                let user_message = create_user_message(&state, &browser_state);
                
                // Generate response using Kalosm
                let response = chat(&user_message).typed::<AgentOutput>();
                
                // Parse and process result
                match response.await {
                    Ok(output) => {
                        // Update state
                        state.step_number += 1;
                        state.task_progress = output.current_state.task_progress.clone();
                        state.future_plans = output.current_state.future_plans.clone();
                        
                        // Check for done action
                        if output.actions.iter().any(|a| a.action_type == "done") {
                            state.stop_requested = true;
                        }
                        
                        // Send output
                        let _ = tx.send(Ok(output)).await;
                        
                        // Execute actions
                        for action in &output.actions {
                            if action.action_type == "done" {
                                break;
                            }
                            
                            // Execute action and collect result
                            let mut result_stream = browser.execute_action(action.clone());
                            let _ = result_stream.next(); // Discard result for simplicity
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(AgentError::ModelError(format!(
                            "Failed to generate response: {}", e
                        )))).await;
                    }
                }
            }
        });
        
        AgentStream::new(rx)
    }
}

const SYSTEM_PROMPT: &str = "You are a precise browser automation agent..."; // Full prompt from schema

fn create_user_message(state: &AgentState, browser_state: &BrowserState) -> String {
    format!(
        "Current step: {}/{}\n\
        Current date and time: {}\n\
        1. Task: {}.\n\
        2. Hints(Optional):\n{}\n\
        3. Memory:\n{}\n\
        4. Current url: {}\n\
        5. Available tabs:\n{}\n\
        6. Interactive elements:\n{}",
        state.step_number,
        state.max_steps,
        chrono::Local::now().to_rfc3339(),
        state.task,
        state.add_infos.as_deref().unwrap_or(""),
        state.memory.join("\n"),
        browser_state.url,
        browser_state.tabs.iter()
            .map(|t| format!("- {} ({})", t.title, t.url))
            .collect::<Vec<_>>()
            .join("\n"),
        browser_state.element_tree,
    )
}
```

## 5. MCP Server Implementation

```rust
// src/server.rs
use mcpr::{schema::*, server::{Server, ToolRegistry}};
use serde_json::{json, Value};

use crate::agent::Agent;
use crate::browser::Browser;
use crate::types::AgentError;

pub struct BrowserMCPServer {
    // Configuration options
}

impl BrowserMCPServer {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn start(&self) {
        let mut server = Server::new();
        self.register_tools(&mut server.tool_registry);
        server.listen();
    }
    
    fn register_tools(&self, registry: &mut ToolRegistry) {
        registry.register(
            "run_browser_agent",
            "Execute a browser automation task",
            json!({
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": "The task to perform"
                    },
                    "add_infos": {
                        "type": "string",
                        "description": "Additional information to help with the task"
                    }
                },
                "required": ["task"]
            }),
            self.handle_run_browser_agent,
        );
    }
    
    fn handle_run_browser_agent(&self, params: Value) -> Value {
        // Extract parameters
        let task = params["task"].as_str().unwrap_or("").to_string();
        let add_infos = params.get("add_infos").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        // Create browser and agent
        let browser = Browser::new(false);
        let mut agent = Agent::new(browser, task, add_infos);
        
        // Run agent and collect results
        let mut agent_stream = agent.run();
        let results = agent_stream.collect_all();
        
        // Format results as JSON
        let output = results.into_iter()
            .map(|result| match result {
                Ok(output) => json!({
                    "success": true,
                    "step": output.current_state.task_progress,
                    "result": output.current_state.important_contents,
                }),
                Err(e) => json!({
                    "success": false,
                    "error": format!("{}", e),
                }),
            })
            .collect::<Vec<_>>();
            
        json!({ "results": output })
    }
}
```

## 6. Main Application Entry Point

```rust
// src/main.rs
mod agent;
mod browser;
mod streams;
mod types;
mod server;

use log::info;
use server::BrowserMCPServer;

fn main() {
    // Initialize logging
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    
    info!("Starting Browser Automation MCP Server");
    
    // Create and start server
    let server = BrowserMCPServer::new();
    server.start();
}
```

## Missing Implementation Elements

The above code provides the skeleton for all major components, but several critical implementation details are still needed:

1. **Structured Type Parsing with Kalosm**: Implement the Parse and Schema traits for all data structures
2. **OCR Integration**: Add capabilities for extracting text from screenshots using Kalosm's OCR
3. **Visual Segmentation**: Add capabilities for visually identifying UI elements
4. **Memory Management**: Implement SurrealDB integration for persistence
5. **Error Recovery**: Add retry logic and more robust error handling
6. **Testing Infrastructure**: Create comprehensive test suite

## Next Implementation Steps

1. Set up the project structure with proper Cargo configuration (via `cargo` commands)
2. Implement the core types and domain-specific return types
3. Create the Browser implementation with chromiumoxide
4. Build the Agent implementation with Kalosm
5. Implement the MCP server
6. Write comprehensive tests
7. Add OCR and visual element detection capabilities

Each component should be thoroughly tested individually before integration testing the entire system.
