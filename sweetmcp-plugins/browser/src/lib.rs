mod automation;
mod commands;
mod errors;
mod pdk;

use automation::*;
use commands::*;
use errors::*;
use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::json;

// MCP Protocol Functions

/// Called when the browser tool is invoked
pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    extism_pdk::log!(
        LogLevel::Info,
        "Browser plugin called with args: {:?}",
        input.params.arguments
    );

    let args = input.params.arguments.unwrap_or_default();

    match input.params.name.as_str() {
        "navigate" => handle_navigate(args),
        "screenshot" => handle_screenshot(args),
        "click" => handle_click(args),
        "type_text" => handle_type_text(args),
        "extract_text" => handle_extract_text(args),
        "scroll" => handle_scroll(args),
        "wait" => handle_wait(args),
        "run_automation" => handle_run_automation(args),
        _ => Err(Error::msg(format!(
            "Unknown browser action: {}",
            input.params.name
        ))),
    }
}

/// Handle browser navigation
fn handle_navigate(
    args: serde_json::Map<String, serde_json::Value>,
) -> Result<CallToolResult, Error> {
    let url = match args.get("url") {
        Some(v) => v
            .as_str()
            .ok_or_else(|| BrowserError::InvalidInput("url must be a string".to_string()))?,
        None => {
            return Err(browser_error_to_extism(BrowserError::InvalidInput(
                "url is required for navigate action".to_string(),
            )));
        }
    };

    // Validate URL
    validate_url(url).map_err(browser_error_to_extism)?;

    extism_pdk::log!(LogLevel::Debug, "Navigating to URL: {}", url);

    // Create the navigation command
    let command = BrowserCommand::Navigate(NavigateCommand {
        url: url.to_string(),
    });

    // Serialize the command for host execution
    let command_json = serde_json::to_string_pretty(&command).map_err(|e| {
        BrowserError::SerializationError(format!("Failed to serialize command: {e}"))
    })?;

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(command_json),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Handle taking screenshots
fn handle_screenshot(
    args: serde_json::Map<String, serde_json::Value>,
) -> Result<CallToolResult, Error> {
    let element_selector = args
        .get("element_selector")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let format = match args.get("format").and_then(|v| v.as_str()) {
        Some("png") => ScreenshotFormat::Png,
        Some("jpeg") => ScreenshotFormat::Jpeg,
        _ => ScreenshotFormat::Base64,
    };

    let command = BrowserCommand::Screenshot(ScreenshotCommand {
        element_selector,
        format,
    });

    let command_json = serde_json::to_string_pretty(&command)
        .map_err(|e| Error::msg(format!("Failed to serialize command: {e}")))?;

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(command_json),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Handle clicking elements
fn handle_click(args: serde_json::Map<String, serde_json::Value>) -> Result<CallToolResult, Error> {
    let selector = match args.get("selector") {
        Some(v) => v
            .as_str()
            .ok_or_else(|| BrowserError::InvalidInput("selector must be a string".to_string()))?,
        None => {
            return Err(browser_error_to_extism(BrowserError::InvalidInput(
                "selector is required for click action".to_string(),
            )));
        }
    };

    // Validate selector
    validate_selector(selector).map_err(browser_error_to_extism)?;

    extism_pdk::log!(LogLevel::Debug, "Clicking element: {}", selector);

    let command = BrowserCommand::Click(ClickCommand {
        selector: selector.to_string(),
    });

    let command_json = serde_json::to_string_pretty(&command).map_err(|e| {
        BrowserError::SerializationError(format!("Failed to serialize command: {e}"))
    })?;

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(command_json),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Handle typing text into elements
fn handle_type_text(
    args: serde_json::Map<String, serde_json::Value>,
) -> Result<CallToolResult, Error> {
    let selector = match args.get("selector") {
        Some(v) => v
            .as_str()
            .ok_or_else(|| Error::msg("selector must be a string"))?,
        None => return Err(Error::msg("selector is required for type_text action")),
    };

    let text = match args.get("text") {
        Some(v) => v
            .as_str()
            .ok_or_else(|| Error::msg("text must be a string"))?,
        None => return Err(Error::msg("text is required for type_text action")),
    };

    let command = BrowserCommand::TypeText(TypeTextCommand {
        selector: selector.to_string(),
        text: text.to_string(),
    });

    let command_json = serde_json::to_string_pretty(&command)
        .map_err(|e| Error::msg(format!("Failed to serialize command: {e}")))?;

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(command_json),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Handle text extraction from elements
fn handle_extract_text(
    args: serde_json::Map<String, serde_json::Value>,
) -> Result<CallToolResult, Error> {
    let selector = args
        .get("selector")
        .and_then(|v| v.as_str())
        .unwrap_or("body");

    let command = BrowserCommand::ExtractText(ExtractTextCommand {
        selector: selector.to_string(),
    });

    let command_json = serde_json::to_string_pretty(&command)
        .map_err(|e| Error::msg(format!("Failed to serialize command: {e}")))?;

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(command_json),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Handle scrolling
fn handle_scroll(
    args: serde_json::Map<String, serde_json::Value>,
) -> Result<CallToolResult, Error> {
    let direction = match args.get("direction").and_then(|v| v.as_str()) {
        Some("up") => ScrollDirection::Up,
        Some("left") => ScrollDirection::Left,
        Some("right") => ScrollDirection::Right,
        _ => ScrollDirection::Down,
    };

    let amount = args.get("amount").and_then(|v| v.as_i64()).unwrap_or(300);

    let command = BrowserCommand::Scroll(ScrollCommand { direction, amount });

    let command_json = serde_json::to_string_pretty(&command)
        .map_err(|e| Error::msg(format!("Failed to serialize command: {e}")))?;

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(command_json),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Handle waiting
fn handle_wait(args: serde_json::Map<String, serde_json::Value>) -> Result<CallToolResult, Error> {
    let duration = args
        .get("duration")
        .and_then(|v| v.as_i64())
        .unwrap_or(1000);

    let command = BrowserCommand::Wait(WaitCommand { duration });

    let command_json = serde_json::to_string_pretty(&command)
        .map_err(|e| Error::msg(format!("Failed to serialize command: {e}")))?;

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(command_json),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Handle running complex browser automation tasks
fn handle_run_automation(
    args: serde_json::Map<String, serde_json::Value>,
) -> Result<CallToolResult, Error> {
    let task = match args.get("task") {
        Some(v) => v
            .as_str()
            .ok_or_else(|| Error::msg("task must be a string"))?,
        None => {
            return Err(Error::msg(
                "task description is required for run_automation",
            ));
        }
    };

    let use_vision = args
        .get("use_vision")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let additional_info = args
        .get("additional_info")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Create automation context for advanced features
    let automation_context = AutomationContext {
        state: AutomationState {
            prev_action_evaluation: String::new(),
            important_contents: String::new(),
            task_progress: "Starting automation task".to_string(),
            future_plans: task.to_string(),
            thought: format!("Preparing to execute: {task}"),
            summary: format!("Automation task initialized: {task}"),
        },
        action_history: Vec::new(),
        browser_config: BrowserConfig::default(),
    };

    // Create agent message for LLM-driven automation
    let agent_message = AgentMessage {
        system_prompt: "You are an expert browser automation agent. Analyze the task and determine the necessary browser actions.".to_string(),
        user_task: task.to_string(),
        context: automation_context,
        use_vision,
    };

    // Package as enhanced automation command
    let command = BrowserCommand::RunAutomation(RunAutomationCommand {
        task: task.to_string(),
        use_vision,
        additional_info: additional_info.to_string(),
    });

    // Include both command and agent context
    let response = json!({
        "command": command,
        "agent_context": agent_message,
        "capabilities": {
            "vision": use_vision,
            "javascript_execution": true,
            "multi_step_automation": true,
            "element_interaction": true,
            "screenshot_analysis": true
        }
    });

    let response_json = serde_json::to_string_pretty(&response)
        .map_err(|e| Error::msg(format!("Failed to serialize automation response: {e}")))?;

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(response_json),
            mime_type: Some("application/json".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Called by MCP to understand how and why to use this browser automation tool
pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![
            ToolDescription {
                name: "navigate".into(),
                description: "Navigate the browser to a specific URL. Use this tool when you need to visit a website or web page.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to navigate to (must include protocol, e.g., https://)"
                        }
                    },
                    "required": ["url"]
                }).as_object().map(|obj| obj.clone()).unwrap_or_else(|| {
                    let mut map = serde_json::Map::new();
                    map.insert("type".to_string(), json!("object"));
                    map
                }),
            },
            ToolDescription {
                name: "screenshot".into(),
                description: "Take a screenshot of the current page or a specific element. Use this tool when you need to capture visual content for analysis or documentation.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "element_selector": {
                            "type": "string",
                            "description": "CSS selector for specific element to screenshot (optional, defaults to full page)"
                        },
                        "format": {
                            "type": "string",
                            "description": "Image format for the screenshot",
                            "enum": ["base64", "png", "jpeg"],
                            "default": "base64"
                        }
                    }
                }).as_object().map(|obj| obj.clone()).unwrap_or_else(|| {
                    let mut map = serde_json::Map::new();
                    map.insert("type".to_string(), json!("object"));
                    map
                }),
            },
            ToolDescription {
                name: "click".into(),
                description: "Click on an element on the page. Use this tool to interact with buttons, links, or other clickable elements.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "selector": {
                            "type": "string",
                            "description": "CSS selector or XPath to identify the element to click"
                        }
                    },
                    "required": ["selector"]
                }).as_object().map(|obj| obj.clone()).unwrap_or_else(|| {
                    let mut map = serde_json::Map::new();
                    map.insert("type".to_string(), json!("object"));
                    map
                }),
            },
            ToolDescription {
                name: "type_text".into(),
                description: "Type text into an input field or text area. Use this tool to fill out forms or enter search queries.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "selector": {
                            "type": "string",
                            "description": "CSS selector to identify the input element"
                        },
                        "text": {
                            "type": "string",
                            "description": "The text to type into the element"
                        }
                    },
                    "required": ["selector", "text"]
                }).as_object().map(|obj| obj.clone()).unwrap_or_else(|| {
                    let mut map = serde_json::Map::new();
                    map.insert("type".to_string(), json!("object"));
                    map
                }),
            },
            ToolDescription {
                name: "extract_text".into(),
                description: "Extract text content from the page or specific elements. Use this tool to gather information from web pages.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "selector": {
                            "type": "string",
                            "description": "CSS selector to extract text from (optional, defaults to entire page body)",
                            "default": "body"
                        }
                    }
                }).as_object().map(|obj| obj.clone()).unwrap_or_else(|| {
                    let mut map = serde_json::Map::new();
                    map.insert("type".to_string(), json!("object"));
                    map
                }),
            },
            ToolDescription {
                name: "scroll".into(),
                description: "Scroll the page in a specified direction. Use this tool to navigate through long pages or reach elements not currently visible.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "direction": {
                            "type": "string",
                            "description": "Direction to scroll",
                            "enum": ["up", "down", "left", "right"],
                            "default": "down"
                        },
                        "amount": {
                            "type": "integer",
                            "description": "Number of pixels to scroll",
                            "default": 300
                        }
                    }
                }).as_object().map(|obj| obj.clone()).unwrap_or_else(|| {
                    let mut map = serde_json::Map::new();
                    map.insert("type".to_string(), json!("object"));
                    map
                }),
            },
            ToolDescription {
                name: "wait".into(),
                description: "Wait for a specified duration. Use this tool when you need to pause for page loading or animations to complete.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "duration": {
                            "type": "integer",
                            "description": "Duration to wait in milliseconds",
                            "default": 1000
                        }
                    }
                }).as_object().map(|obj| obj.clone()).unwrap_or_else(|| {
                    let mut map = serde_json::Map::new();
                    map.insert("type".to_string(), json!("object"));
                    map
                }),
            },
            ToolDescription {
                name: "run_automation".into(),
                description: "Run complex browser automation tasks using AI agents. Use this tool for sophisticated workflows that require multiple steps, decision-making, or visual analysis of web pages. Perfect for tasks like 'fill out this form', 'find product information', or 'complete this checkout process'.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "task": {
                            "type": "string",
                            "description": "Detailed description of the automation task to perform"
                        },
                        "use_vision": {
                            "type": "boolean",
                            "description": "Whether to use computer vision capabilities for visual analysis",
                            "default": false
                        },
                        "additional_info": {
                            "type": "string",
                            "description": "Additional context or instructions for the automation task",
                            "default": ""
                        }
                    },
                    "required": ["task"]
                }).as_object().map(|obj| obj.clone()).unwrap_or_else(|| {
                    let mut map = serde_json::Map::new();
                    map.insert("type".to_string(), json!("object"));
                    map
                }),
            },
        ],
    })
}
