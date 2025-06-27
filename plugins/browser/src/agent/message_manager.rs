use std::sync::Arc;

use kalosm::llm::{ChatMessage, Llm};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::agent::{AgentError, AgentResult};
use crate::browser::BrowserContext;

/// Manager for handling agent message history and context
#[derive(Clone)]
pub struct MessageManager {
    messages: Arc<Mutex<Vec<ChatMessage>>>,
    max_history_length: usize,
    llm: Arc<dyn Llm>,
}

impl MessageManager {
    /// Create a new message manager
    pub fn new(llm: Arc<dyn Llm>, max_history_length: usize) -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
            max_history_length,
            llm,
        }
    }
    
    /// Initialize the message context with system prompt
    pub async fn init_context(&self, system_prompt: &str) -> AgentResult<()> {
        let mut messages = self.messages.lock().await;
        messages.clear();
        messages.push(ChatMessage::system(system_prompt));
        Ok(())
    }
    
    /// Add a user message to the history
    pub async fn add_user_message(&self, content: &str) -> AgentResult<()> {
        let mut messages = self.messages.lock().await;
        messages.push(ChatMessage::user(content));
        self.trim_history(&mut messages);
        Ok(())
    }
    
    /// Add an assistant message to the history
    pub async fn add_assistant_message(&self, content: &str) -> AgentResult<()> {
        let mut messages = self.messages.lock().await;
        messages.push(ChatMessage::assistant(content));
        self.trim_history(&mut messages);
        Ok(())
    }
    
    /// Get all messages in the history
    pub async fn get_messages(&self) -> AgentResult<Vec<ChatMessage>> {
        let messages = self.messages.lock().await;
        Ok(messages.clone())
    }
    
    /// Clear all messages except the system prompt
    pub async fn clear_messages(&self) -> AgentResult<()> {
        let mut messages = self.messages.lock().await;
        if messages.is_empty() {
            return Ok(());
        }
        
        // Keep the system prompt if present
        let system_prompt = if !messages.is_empty() && messages[0].role == "system" {
            Some(messages[0].clone())
        } else {
            None
        };
        
        messages.clear();
        
        if let Some(prompt) = system_prompt {
            messages.push(prompt);
        }
        
        Ok(())
    }
    
    /// Summarize the current conversation for context management
    pub async fn summarize_conversation(&self) -> AgentResult<String> {
        let messages = self.messages.lock().await;
        
        // If we have fewer messages than the history limit, no need to summarize
        if messages.len() <= self.max_history_length {
            return Ok("No summary needed, conversation is within history limits.".to_string());
        }
        
        // Create a summarization prompt
        let mut summary_messages = Vec::new();
        summary_messages.push(ChatMessage::system(
            "You are a helpful assistant tasked with summarizing a conversation. \
            Create a concise summary that captures the key information, \
            decisions, and context needed to continue the conversation."
        ));
        
        // Add all messages except the system prompt
        let conversation_messages = if !messages.is_empty() && messages[0].role == "system" {
            messages.iter().skip(1).cloned().collect::<Vec<_>>()
        } else {
            messages.clone()
        };
        
        // Join conversation messages into a single content
        let conversation_content = conversation_messages.iter()
            .map(|msg| format!("{}:\n{}", msg.role, msg.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        summary_messages.push(ChatMessage::user(&format!(
            "Please summarize the following conversation:\n\n{}",
            conversation_content
        )));
        
        // Generate summary using LLM
        let summary = self.llm.chat(summary_messages).await
            .map_err(|e| AgentError::LlmError(e.to_string()))?;
        
        Ok(summary)
    }
    
    /// Add browser state information to the context
    pub async fn add_browser_state(&self, browser: &BrowserContext) -> AgentResult<()> {
        // Get current page information
        let page = browser.get_current_page().await
            .map_err(|e| AgentError::BrowserError(e.to_string()))?;
            
        let url = page.url().await
            .map_err(|e| AgentError::BrowserError(e.to_string()))?;
            
        let title = page.title().await
            .unwrap_or_else(|_| "Unknown Title".to_string());
            
        // Create browser state message
        let state_message = format!(
            "Current Browser State:\nURL: {}\nTitle: {}\n",
            url, title
        );
        
        self.add_user_message(&state_message).await
    }
    
    /// Trim history to max length, preserving system prompt and recent messages
    fn trim_history(&self, messages: &mut Vec<ChatMessage>) {
        if messages.len() <= self.max_history_length {
            return;
        }
        
        // Always keep the system prompt if present
        let has_system_prompt = !messages.is_empty() && messages[0].role == "system";
        let system_prompt = if has_system_prompt {
            Some(messages[0].clone())
        } else {
            None
        };
        
        // Calculate how many messages to keep after trimming
        let keep_count = self.max_history_length - if has_system_prompt { 1 } else { 0 };
        
        // Keep the most recent messages
        let recent_messages = messages.iter()
            .skip(messages.len() - keep_count)
            .cloned()
            .collect::<Vec<_>>();
        
        messages.clear();
        
        // Re-add system prompt if it existed
        if let Some(prompt) = system_prompt {
            messages.push(prompt);
        }
        
        // Add recent messages
        messages.extend(recent_messages);
    }
}
