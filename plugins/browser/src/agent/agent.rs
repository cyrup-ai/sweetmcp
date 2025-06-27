use std::sync::Arc;

use kalosm::language::*;
use kalosm_llama::Llama;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};

// For vision/segmentation
use image;
use kalosm::vision;

use crate::agent::{
    prompts::{AgentMessagePrompt, SystemPrompt},
    AgentError, AgentHistory, AgentHistoryList, AgentOutput, AgentResult, ActionModel, ActionResult,
};
use crate::browser::{BrowserContext, BrowserError};
use crate::controller::Controller;
use crate::utils::AgentState;

/// Agent implementation that manages browser automation
#[derive(Clone)]
pub struct Agent {
    task: String,
    add_infos: String,
    use_vision: bool,
    llm: Llama,
    browser: Arc<BrowserContext>,
    controller: Controller,
    system_prompt: SystemPrompt,
    agent_prompt: AgentMessagePrompt,
    max_actions_per_step: usize,
    agent_state: Arc<Mutex<AgentState>>,
    tool_calling_method: String,
    command_channel: mpsc::Sender<AgentCommand>,
    response_channel: mpsc::Receiver<AgentResponse>,
}

/// Agent command enum for internal message passing
enum AgentCommand {
    RunStep,
    Stop,
}

/// Agent response enum for internal message passing
enum AgentResponse {
    StepComplete(AgentOutput),
    Error(String),
    Stopped,
}

///  agent implementation
impl Agent {
    /// Create a new agent instance
    pub fn new(
        task: &str,
        add_infos: &str,
        use_vision: bool,
        llm: Llama,
        browser: Arc<BrowserContext>,
        controller: Controller,
        system_prompt: SystemPrompt,
        agent_prompt: AgentMessagePrompt,
        max_actions_per_step: usize,
        agent_state: Arc<Mutex<AgentState>>,
        tool_calling_method: &str,
    ) -> AgentResult<Self> {
        // Create channels for command passing
        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (resp_tx, resp_rx) = mpsc::channel(32);

        let agent = Self {
            task: task.to_string(),
            add_infos: add_infos.to_string(),
            use_vision: use_vision,
            llm,
            browser,
            controller,
            system_prompt,
            agent_prompt,
            max_actions_per_step,
            agent_state,
            tool_calling_method: tool_calling_method.to_string(),
            command_channel: cmd_tx,
            response_channel: resp_rx,
        };

        // Spawn agent processing task
        Self::spawn_agent_processor(
            agent.clone(),
            cmd_rx,
            resp_tx
        );

        Ok(agent)
    }
    
    /// Run the agent to perform a task with a maximum number of steps
    pub async fn run(&self, max_steps: usize) -> AgentResult<AgentHistoryList> {
        let mut history = AgentHistoryList::new();

        for step in 0..max_steps {
            debug!("Running agent step {}/{}", step + 1, max_steps);

            // Check if stop was requested
            if self.is_stop_requested().await {
                info!("Agent run stopped as requested");
                break;
            }

            // Run a single step
            match self.run_step().await {
                Ok(output) => {
                    // Record step output
                    let is_done = output.action.iter().any(|a| a.action.eq_ignore_ascii_case("done"));
                    history.add_step_with_completion(output.clone(), is_done);

                    // Check if agent considers task complete
                    // Protocol: done if any action is "done" or "Done"
                    if is_done {
                        info!("Agent completed task in {} steps", step + 1);
                        break;
                    }
                }
                Err(e) => {
                    error!("Agent step error: {}", e);
                    return Err(e);
                }
            }
        }

        Ok(history)
    }
    
    /// Run a single agent step
    async fn run_step(&self) -> AgentResult<AgentOutput> {
        // Send command to agent processor
        self.command_channel
            .send(AgentCommand::RunStep)
            .await
            .map_err(|_| AgentError::ChannelClosed("Command channel closed".into()))?;
        
        // Wait for response
        match self.response_channel.recv().await {
            Some(AgentResponse::StepComplete(output)) => Ok(output),
            Some(AgentResponse::Error(msg)) => Err(AgentError::StepFailed(msg)),
            Some(AgentResponse::Stopped) => Err(AgentError::Stopped),
            None => Err(AgentError::ChannelClosed("Response channel closed".into())),
        }
    }
    
    /// Check if agent stop was requested
    async fn is_stop_requested(&self) -> bool {
        let agent_state = self.agent_state.lock().await;
        agent_state.is_stop_requested()
    }
    
    /// Spawn the agent processor task
    fn spawn_agent_processor(
        agent: Self,
        mut cmd_rx: mpsc::Receiver<AgentCommand>,
        resp_tx: mpsc::Sender<AgentResponse>,
    ) {
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    AgentCommand::RunStep => {
                        // Process agent step
                        let result = agent.process_step().await;
                        
                        // Send result back through response channel
                        match result {
                            Ok(output) => {
                                if let Err(e) = resp_tx.send(AgentResponse::StepComplete(output)).await {
                                    error!("Failed to send step complete response: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                if let Err(e) = resp_tx.send(AgentResponse::Error(e.to_string())).await {
                                    error!("Failed to send error response: {}", e);
                                }
                                break;
                            }
                        }
                    }
                    AgentCommand::Stop => {
                        if let Err(e) = resp_tx.send(AgentResponse::Stopped).await {
                            error!("Failed to send stopped response: {}", e);
                        }
                        break;
                    }
                }
            }
        });
    }
    
    /// Process a single agent step internally
    async fn process_step(&self) -> AgentResult<AgentOutput> {
        // Check if stop requested
        let agent_state = self.agent_state.lock().await;
        if agent_state.is_stop_requested() {
            return Err(AgentError::Stopped);
        }
        drop(agent_state);

        // Get current browser state (with screenshot)
        let browser_state = self.get_browser_state().await?;

        // Prepare context for LLM with system prompt, task description, and browser state
        let messages = self.build_step_messages(&browser_state).await?;

        // Generate agent actions using LLM
        let actions = self.generate_actions(messages).await?;

        // NOTE: The protocol requires the LLM to return both current_state and action.
        // Here, we must get the full AgentLLMResponse from the LLM, not just actions.
        use crate::agent::AgentLLMResponse;

        let mut chat = self.llm.chat();

        let mut has_system_prompt = false;
        for message in &messages {
            if let Some(content) = message.get_system_content() {
                chat = chat.with_system_prompt(content);
                has_system_prompt = true;
                break;
            }
        }
        if !has_system_prompt {
            chat = chat.with_system_prompt("You are a helpful browser automation assistant.");
        }
        for message in &messages {
            if let Some(content) = message.get_user_content() {
                chat = chat.with_user_prompt(content);
            }
        }

        let agent_response: AgentLLMResponse = chat
            .constrained()
            .await
            .map_err(|e| AgentError::LlmError(e.to_string()))?;

        // Limit the number of actions
        let mut limited_actions = agent_response.action;
        if limited_actions.len() > self.max_actions_per_step {
            limited_actions = limited_actions[0..self.max_actions_per_step].to_vec();
        }

        let output = AgentOutput {
            current_state: agent_response.current_state,
            action: limited_actions,
        };

        Ok(output)
    }
    
    /// Get current browser state, including screenshot (base64)
    async fn get_browser_state(&self) -> AgentResult<BrowserStateWithScreenshot> {
        // Get page screenshot and contents
        match self.browser.get_current_page().await {
            Ok(page) => {
                // (Optional) Mask sensitive elements before screenshot
                // Blur all password fields
                let _ = page.evaluate(
                    "document.querySelectorAll('input[type=password]').forEach(e => e.style.filter='blur(8px)')"
                ).await;

                // Take screenshot
                let screenshot_data = self.browser.screenshot().await.ok();
                let screenshot_base64 = screenshot_data
                    .as_ref()
                    .map(|bytes| base64::encode(bytes));

                // === SEGMENT ANYTHING INTEGRATION ===
                // If we have screenshot data, run segmentation using Kalosm vision API
                let (segmentation_result, bounding_boxes) = if let Some(ref bytes) = screenshot_data {
                    // Load image from bytes using the image crate
                    let img = match image::load_from_memory(bytes) {
                        Ok(img) => img,
                        Err(e) => {
                            error!("Failed to load screenshot as image: {}", e);
                            // Continue without segmentation
                            return Err(AgentError::UnexpectedError(format!("Failed to load screenshot as image: {}", e)));
                        }
                    };

                    // Initialize the Segment Anything model
                    let model = match kalosm::vision::SegmentAnything::builder().build() {
                        Ok(model) => model,
                        Err(e) => {
                            error!("Failed to initialize Segment Anything model: {}", e);
                            return Err(AgentError::UnexpectedError(format!("Failed to initialize Segment Anything model: {}", e)));
                        }
                    };

                    // Segment the center of the image as a demonstration
                    let x = img.width() / 2;
                    let y = img.height() / 4;
                    let settings = kalosm::vision::SegmentAnythingInferenceSettings::new(img.clone()).add_goal_point(x, y);

                    match model.segment_from_points(settings) {
                        Ok(segmentation) => {
                            // Extract bounding boxes and labels for each region
                            let mut boxes = Vec::new();
                            for (i, region) in segmentation.regions().enumerate() {
                                let bbox = region.bounding_box();
                                // bounding_box() returns (x, y, width, height)
                                boxes.push(VisualElementBox {
                                    label: format!("region_{}", i + 1),
                                    x: bbox.0,
                                    y: bbox.1,
                                    width: bbox.2,
                                    height: bbox.3,
                                });
                            }
                            (Some(segmentation), Some(boxes))
                        }
                        Err(e) => {
                            error!("Segmentation failed: {}", e);
                            (None, None)
                        }
                    }
                } else {
                    (None, None)
                };
                // === END SEGMENT ANYTHING INTEGRATION ===

                // Store the segmentation_result and bounding_boxes for use in subsequent steps (OCR, prompt construction, etc.)

                // The bounding_boxes variable now contains a Vec<VisualElementBox> for all detected regions.
                // This will be used in subsequent steps (OCR, prompt construction, etc.).

                // Get page title and URL
                let title = page.title().await.unwrap_or_else(|_| "Untitled".into());
                let url = page.url().await.unwrap_or_else(|_| "about:blank".into());

                // Get page content and structure
                let content = page.content().await
                    .map_err(|e| AgentError::BrowserError(e.to_string()))?;

                // Combine information into a structured state representation
                let state = format!(
                    "URL: {}\nTitle: {}\nContent Length: {}\nContent Sample: {}...",
                    url,
                    title,
                    content.len(),
                    &content[0..content.len().min(500)]
                );

                // Store state in agent state for recovery if needed
                let mut agent_state = self.agent_state.lock().await;
                agent_state.set_last_valid_state(state.clone());

                // The segmentation_result and bounding_boxes are now available for use in subsequent steps.

                Ok(BrowserStateWithScreenshot {
                    state,
                    screenshot_base64,
                    bounding_boxes,
                })
            },
            Err(e) => {
                // If failed to get current page, try to recover from last valid state
                let agent_state = self.agent_state.lock().await;
                if let Some(last_state) = agent_state.get_last_valid_state() {
                    warn!("Using last valid browser state due to error: {}", e);
                    Ok(BrowserStateWithScreenshot {
                        state: last_state,
                        screenshot_base64: None,
                    })
                } else {
                    Err(AgentError::BrowserError(e.to_string()))
                }
            }
        }
    }
    
    /// Build messages for the LLM for this step
    async fn build_step_messages(&self, browser_state: &BrowserStateWithScreenshot) -> AgentResult<Vec<ChatMessage>> {
        let mut messages = Vec::new();

        // Add system prompt
        let system_prompt = self.system_prompt.build_prompt();
        messages.push(ChatMessage::system(&system_prompt));

        // Add task description
        let task_description = format!(
            "Task: {}\nAdditional Information: {}",
            self.task,
            self.add_infos
        );
        messages.push(ChatMessage::user(&task_description));

        // Add browser state
        let mut browser_state_msg = format!("Current browser state:\n{}", browser_state.state);
        if let Some(ref screenshot_b64) = browser_state.screenshot_base64 {
            browser_state_msg.push_str(&format!("\n[IMAGE: Base64-encoded screenshot]\n{}", screenshot_b64));
        }
        messages.push(ChatMessage::user(&browser_state_msg));

        // Add available actions
        let actions_description = self.controller.list_available_actions();
        messages.push(ChatMessage::system(&actions_description));

        Ok(messages)
    }
    
    /// Generate actions using the LLM (now using constrained generation)
    async fn generate_actions(&self, messages: Vec<ChatMessage>) -> AgentResult<Vec<ActionModel>> {
        use crate::agent::AgentLLMResponse;

        // Create a chat session from the Llama model
        let mut chat = self.llm.chat();

        // Extract system message and user messages
        let mut has_system_prompt = false;

        // First process system messages
        for message in &messages {
            if let Some(content) = message.get_system_content() {
                chat = chat.with_system_prompt(content);
                has_system_prompt = true;
                break; // Use the first system message as the system prompt
            }
        }

        // If no system message was found, use a default
        if !has_system_prompt {
            chat = chat.with_system_prompt("You are a helpful browser automation assistant.");
        }

        // Then add all user messages
        for message in &messages {
            if let Some(content) = message.get_user_content() {
                chat = chat.with_user_prompt(content);
            }
        }

        // Use constrained generation for protocol-compliant output
        let agent_response: AgentLLMResponse = chat
            .constrained()
            .await
            .map_err(|e| AgentError::LlmError(e.to_string()))?;

        // Limit the number of actions
        let actions = agent_response.action;
        let limited_actions = if actions.len() > self.max_actions_per_step {
            warn!(
                "Agent generated {} actions, limiting to {}",
                actions.len(),
                self.max_actions_per_step
            );
            actions[0..self.max_actions_per_step].to_vec()
        } else {
            actions
        };

        Ok(limited_actions)
    }
    
    /// Execute actions and collect results
    async fn execute_actions(
        &self, 
        actions: Vec<ActionModel>
    ) -> AgentResult<(Vec<ActionResult>, Vec<String>)> {
        let mut results = Vec::new();
        let mut errors = Vec::new();

        for action in actions {
            match self.controller.execute_action(&action, &self.browser).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    let error_msg = format!("Action '{}' failed: {}", action.action, e);
                    errors.push(error_msg);
                }
            }
        }

        Ok((results, errors))
    }
}
#[derive(Debug, Clone)]
struct VisualElementBox {
    label: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

/// Struct to hold browser state, screenshot (base64), and visual element bounding boxes
#[derive(Debug, Clone)]
struct BrowserStateWithScreenshot {
    state: String,
    screenshot_base64: Option<String>,
    bounding_boxes: Option<Vec<VisualElementBox>>,
}
