use serde::{Deserialize, Serialize};

/// Advanced automation structures for AI-driven browser control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationContext {
    /// Current state of the automation task
    pub state: AutomationState,
    /// History of actions taken
    pub action_history: Vec<ActionHistory>,
    /// Browser context configuration
    pub browser_config: BrowserConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationState {
    /// Previous action evaluation
    pub prev_action_evaluation: String,
    /// Important page contents
    pub important_contents: String,
    /// Current task progress
    pub task_progress: String,
    /// Future planned steps
    pub future_plans: String,
    /// Current thought process
    pub thought: String,
    /// Summary of current state
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionHistory {
    /// Action that was performed
    pub action: ActionModel,
    /// Result of the action
    pub result: ActionResult,
    /// Timestamp when action was performed
    pub timestamp: i64,
    /// Screenshot data if captured
    pub screenshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionModel {
    /// Navigate to a URL
    Navigate { url: String },
    /// Click on an element
    Click { selector: String },
    /// Type text into an element
    Type { selector: String, text: String },
    /// Take a screenshot
    Screenshot { full_page: bool },
    /// Extract text from elements
    ExtractText { selector: String },
    /// Scroll the page
    Scroll { direction: String, amount: i32 },
    /// Wait for a duration or element
    Wait {
        duration: Option<i64>,
        selector: Option<String>,
    },
    /// Execute custom JavaScript
    ExecuteScript { script: String },
    /// AI-based visual analysis
    AnalyzeVisual { prompt: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    /// Whether the action succeeded
    pub success: bool,
    /// Result message
    pub message: String,
    /// Any data returned by the action
    pub data: Option<serde_json::Value>,
    /// Error details if failed
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrowserConfig {
    /// Browser window size
    pub window_size: WindowSize,
    /// Whether to disable viewport
    pub no_viewport: bool,
    /// Path to save recordings
    pub save_recording_path: Option<String>,
    /// Path to save traces
    pub trace_path: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Proxy configuration
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: i32,
    pub height: i32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
        }
    }
}

/// Vision capabilities for analyzing page content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionAnalysis {
    /// The prompt for vision analysis
    pub prompt: String,
    /// Screenshot to analyze
    pub screenshot_base64: String,
    /// Region of interest (optional)
    pub region: Option<Region>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Agent message for LLM communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// System prompt defining agent behavior
    pub system_prompt: String,
    /// User task description
    pub user_task: String,
    /// Current automation context
    pub context: AutomationContext,
    /// Whether to use vision capabilities
    pub use_vision: bool,
}

/// LLM response for next actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// Updated automation state
    pub state: AutomationState,
    /// Next actions to perform
    pub actions: Vec<ActionModel>,
    /// Whether the task is complete
    pub task_complete: bool,
    /// Confidence in completion (0-1)
    pub confidence: f32,
}
