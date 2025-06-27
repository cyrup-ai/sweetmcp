use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agent::{AgentHistoryList, ActionResult};

/// Represents the browser state for rendering in the view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserStateView {
    pub url: String,
    pub title: String,
    pub content_sample: String,
    pub screenshot: Option<String>,
}

/// Represents an action for rendering in the view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionView {
    pub action_type: String,
    pub parameters: HashMap<String, String>,
    pub result: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

/// Represents a step in the agent history for rendering in the view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepView {
    pub step_number: usize,
    pub browser_state: BrowserStateView,
    pub actions: Vec<ActionView>,
    pub reasoning: String,
    pub is_complete: bool,
    pub timestamp: String,
}

/// Represents the entire agent history for rendering in the view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryView {
    pub task: String,
    pub steps: Vec<StepView>,
}

impl HistoryView {
    /// Create a new history view from an agent history list
    pub fn from_history(history: &AgentHistoryList) -> Self {
        let mut steps = Vec::new();
        let mut task = String::new();

        for (idx, step) in history.steps.iter().enumerate() {
            // Extract task from the first step
            if idx == 0 {
                task = step.output.current_state.summary.clone();
            }

            // Try to extract screenshot from the important_contents field if present as base64
            let screenshot = extract_screenshot_from_state(&step.output.current_state.important_contents);

            // Extract browser state using real parsing
            let browser_state = BrowserStateView {
                url: extract_url_from_state(&step.output.current_state.important_contents),
                title: extract_title_from_state(&step.output.current_state.important_contents),
                content_sample: extract_content_sample(&step.output.current_state.important_contents),
                screenshot,
            };

            // Extract actions
            let actions = step.output.action.iter()
                .map(|a| ActionView {
                    action_type: a.action.clone(),
                    parameters: a.parameters.clone(),
                    result: None,
                    success: false,
                    error: None,
                })
                .collect();

            // Create step view
            let step_view = StepView {
                step_number: idx,
                browser_state,
                actions,
                reasoning: step.output.current_state.thought.clone(),
                is_complete: step.output.action.iter().any(|a| a.action.eq_ignore_ascii_case("done")),
                timestamp: step.timestamp.to_rfc3339(),
            };

            steps.push(step_view);
        }

        Self {
            task,
            steps,
        }
    }
    
    /// Convert the history view to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Extract URL from browser state string
fn extract_url_from_state(state: &str) -> String {
    for line in state.lines() {
        if line.starts_with("URL:") {
            return line.trim_start_matches("URL:").trim().to_string();
        }
    }
    "about:blank".to_string()
}

/// Extract title from browser state string
fn extract_title_from_state(state: &str) -> String {
    for line in state.lines() {
        if line.starts_with("Title:") {
            return line.trim_start_matches("Title:").trim().to_string();
        }
    }
    "Untitled".to_string()
}

/// Extract content sample from browser state string
fn extract_content_sample(state: &str) -> String {
    for line in state.lines() {
        if line.starts_with("Content Sample:") {
            return line.trim_start_matches("Content Sample:").trim().to_string();
        }
    }

    // If we can't find the content sample prefix, return the whole state
    // but limit it to a reasonable size
    let max_length = 500;
    if state.len() > max_length {
        format!("{}...", &state[0..max_length])
    } else {
        state.to_string()
    }
}

/// Extract screenshot (base64) from browser state string, if present
fn extract_screenshot_from_state(state: &str) -> Option<String> {
    // Look for a line starting with [IMAGE: Base64-encoded screenshot]
    let mut found = false;
    let mut b64 = String::new();
    for line in state.lines() {
        if found {
            b64.push_str(line.trim());
            break;
        }
        if line.trim().starts_with("[IMAGE: Base64-encoded screenshot]") {
            found = true;
        }
    }
    if !b64.is_empty() {
        Some(b64)
    } else {
        None
    }
}
