use std::collections::HashMap;
use std::sync::Arc;

use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::browser::{BrowserContext, BrowserError, BrowserResult};
use crate::controller::{ControllerError, ControllerResult, ActionExecutor};

/// Represents available browser actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionModel {
    pub action: String,
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}

/// Result of an executed action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action: String,
    pub success: bool,
    pub extracted_content: Option<String>,
    pub error: Option<String>,
}

///  controller for browser automation
#[derive(Clone)]
pub struct Controller {
    actions: Arc<Mutex<HashMap<String, Box<dyn ActionExecutor + Send + Sync>>>>,
    exclude_actions: Vec<String>,
}

impl Controller {
    /// Create a new controller
    pub fn new() -> Self {
        let actions: Arc<Mutex<HashMap<String, Box<dyn ActionExecutor + Send + Sync>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let exclude_actions = Vec::new();

        // Register all protocol-compliant actions synchronously
        {
            let mut actions_map = futures::executor::block_on(actions.lock());

            // go_to_url
            actions_map.insert(
                "go_to_url".to_string(),
                Box::new(|params: &ActionModel, browser: &BrowserContext| {
                    Box::pin(async move {
                        let url = params.parameters.get("url")
                            .ok_or_else(|| ControllerError::MissingParameter("url".into()))?;
                        let page = browser.get_current_page().await?;
                        page.navigate(url).await?;
                        Ok(ActionResult {
                            action: "go_to_url".into(),
                            success: true,
                            extracted_content: Some(format!("Navigated to {}", url)),
                            error: None,
                        })
                    })
                }),
            );

            // click_element
            actions_map.insert(
                "click_element".to_string(),
                Box::new(|params: &ActionModel, browser: &BrowserContext| {
                    Box::pin(async move {
                        let index = params.parameters.get("index")
                            .ok_or_else(|| ControllerError::MissingParameter("index".into()))?;
                        let selector = format!("[data-mcp-index=\"{}\"]", index);
                        let page = browser.get_current_page().await?;
                        page.click(&selector).await?;
                        Ok(ActionResult {
                            action: "click_element".into(),
                            success: true,
                            extracted_content: Some(format!("Clicked element with index: {}", index)),
                            error: None,
                        })
                    })
                }),
            );

            // input_text
            actions_map.insert(
                "input_text".to_string(),
                Box::new(|params: &ActionModel, browser: &BrowserContext| {
                    Box::pin(async move {
                        let index = params.parameters.get("index")
                            .ok_or_else(|| ControllerError::MissingParameter("index".into()))?;
                        let text = params.parameters.get("text")
                            .ok_or_else(|| ControllerError::MissingParameter("text".into()))?;
                        let selector = format!("[data-mcp-index=\"{}\"]", index);
                        let page = browser.get_current_page().await?;
                        page.type_text(&selector, text).await?;
                        Ok(ActionResult {
                            action: "input_text".into(),
                            success: true,
                            extracted_content: Some(format!("Typed text into index: {}", index)),
                            error: None,
                        })
                    })
                }),
            );

            // send_keys
            actions_map.insert(
                "send_keys".to_string(),
                Box::new(|params: &ActionModel, browser: &BrowserContext| {
                    Box::pin(async move {
                        let keys = params.parameters.get("keys")
                            .ok_or_else(|| ControllerError::MissingParameter("keys".into()))?;
                        let page = browser.get_current_page().await?;
                        page.keyboard_type(keys).await?;
                        Ok(ActionResult {
                            action: "send_keys".into(),
                            success: true,
                            extracted_content: Some(format!("Sent keys: {}", keys)),
                            error: None,
                        })
                    })
                }),
            );

            // scroll
            actions_map.insert(
                "scroll".to_string(),
                Box::new(|params: &ActionModel, browser: &BrowserContext| {
                    Box::pin(async move {
                        let direction = params.parameters.get("direction")
                            .map(|s| s.as_str())
                            .unwrap_or("down");
                        let amount = params.parameters.get("amount")
                            .and_then(|a| a.parse::<i32>().ok())
                            .unwrap_or(500);
                        let element_index = params.parameters.get("element_index");
                        let scroll_script = if let Some(idx) = element_index {
                            format!("document.querySelector('[data-mcp-index=\"{}\"]').scrollIntoView();", idx)
                        } else {
                            match direction {
                                "up" => format!("window.scrollBy(0, -{})", amount),
                                "down" => format!("window.scrollBy(0, {})", amount),
                                "to_element" => "/* element_index required */".to_string(),
                                _ => format!("window.scrollBy(0, {})", amount),
                            }
                        };
                        let page = browser.get_current_page().await?;
                        page.evaluate(&scroll_script).await?;
                        Ok(ActionResult {
                            action: "scroll".into(),
                            success: true,
                            extracted_content: Some(format!("Scrolled {}", direction)),
                            error: None,
                        })
                    })
                }),
            );

            // extract_page_content
            actions_map.insert(
                "extract_page_content".to_string(),
                Box::new(|params: &ActionModel, browser: &BrowserContext| {
                    Box::pin(async move {
                        let include_links = params.parameters.get("include_links")
                            .map(|v| v == "true")
                            .unwrap_or(false);
                        let page = browser.get_current_page().await?;
                        let extract_script = r#"
                            function extractContent() {
                                // Remove script tags, style tags, and non-visible elements
                                const clone = document.cloneNode(true);
                                const scripts = clone.getElementsByTagName('script');
                                const styles = clone.getElementsByTagName('style');
                                while (scripts.length > 0) {
                                    scripts[0].parentNode.removeChild(scripts[0]);
                                }
                                while (styles.length > 0) {
                                    styles[0].parentNode.removeChild(styles[0]);
                                }
                                // Try to find main content
                                const mainContent =
                                    document.querySelector('main') ||
                                    document.querySelector('article') ||
                                    document.querySelector('.content') ||
                                    document.querySelector('#content') ||
                                    document.body;
                                let text = mainContent.innerText;
                                let html = mainContent.innerHTML;
                                return { text, html };
                            }
                            extractContent();
                        "#;
                        let content_result = page.evaluate(extract_script).await?;
                        let content = serde_json::from_str::<serde_json::Value>(&content_result)
                            .map_err(|e| ControllerError::JsonParseError(e.to_string()))?;
                        let extracted = if include_links {
                            content["html"].as_str().unwrap_or("").to_string()
                        } else {
                            content["text"].as_str().unwrap_or("").to_string()
                        };
                        let format_type = if include_links { "markdown" } else { "text" };
                        let message = format!("📄 Extracted page content as {}:\n{}", format_type, extracted);
                        Ok(ActionResult {
                            action: "extract_page_content".into(),
                            success: true,
                            extracted_content: Some(message),
                            error: None,
                        })
                    })
                }),
            );

            // copy_to_clipboard
            actions_map.insert(
                "copy_to_clipboard".to_string(),
                Box::new(|params: &ActionModel, _browser: &BrowserContext| {
                    Box::pin(async move {
                        let text = params.parameters.get("text")
                            .ok_or_else(|| ControllerError::MissingParameter("text".into()))?;
                        let mut clipboard = Clipboard::new()
                            .map_err(|e| ControllerError::ClipboardError(e.to_string()))?;
                        clipboard.set_text(text)
                            .map_err(|e| ControllerError::ClipboardError(e.to_string()))?;
                        Ok(ActionResult {
                            action: "copy_to_clipboard".into(),
                            success: true,
                            extracted_content: Some(format!("Copied to clipboard: {}", text)),
                            error: None,
                        })
                    })
                }),
            );

            // paste_from_clipboard
            actions_map.insert(
                "paste_from_clipboard".to_string(),
                Box::new(|_params: &ActionModel, browser: &BrowserContext| {
                    Box::pin(async move {
                        let mut clipboard = Clipboard::new()
                            .map_err(|e| ControllerError::ClipboardError(e.to_string()))?;
                        let text = clipboard.get_text()
                            .map_err(|e| ControllerError::ClipboardError(e.to_string()))?;
                        let page = browser.get_current_page().await?;
                        // Paste into the currently focused element
                        page.evaluate("document.activeElement.focus()").await?;
                        page.keyboard_type(&text).await?;
                        Ok(ActionResult {
                            action: "paste_from_clipboard".into(),
                            success: true,
                            extracted_content: Some(format!("Pasted from clipboard: {}", text)),
                            error: None,
                        })
                    })
                }),
            );

            // done
            actions_map.insert(
                "done".to_string(),
                Box::new(|params: &ActionModel, _browser: &BrowserContext| {
                    Box::pin(async move {
                        let result = params.parameters.get("result")
                            .unwrap_or(&"Task completed".to_string());
                        Ok(ActionResult {
                            action: "done".into(),
                            success: true,
                            extracted_content: Some(format!("Task completed: {}", result)),
                            error: None,
                        })
                    })
                }),
            );
        }

        Self {
            actions,
            exclude_actions,
        }
    }
    
    /// List all available actions for reference
    pub fn list_available_actions(&self) -> String {
        let actions_future = self.actions.clone();
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let actions = actions_future.lock().await;
                let mut action_list = Vec::new();
                
                for (name, _) in actions.iter() {
                    if !self.exclude_actions.contains(name) {
                        action_list.push(name.clone());
                    }
                }
                
                let mut description = String::from("Available Actions:\n");
                for action in action_list {
                    description.push_str(&format!("- {}\n", action));
                }
                
                description
            })
        })
    }
    
    /// Execute an action with the given parameters
    pub async fn execute_action(
        &self,
        action: &ActionModel,
        browser: &BrowserContext,
    ) -> ControllerResult<ActionResult> {
        let actions = self.actions.lock().await;
        
        if let Some(executor) = actions.get(&action.action) {
            // Execute the action using the new interface
            let result = executor.execute(action, browser).await?;
            Ok(result)
        } else {
            Err(ControllerError::UnknownAction(action.action.clone()))
        }
    }
}
