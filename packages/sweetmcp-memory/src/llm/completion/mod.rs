//! LLM completion service for generating text completions

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::llm::{LLMProvider, LLMError};

/// Completion service for generating text completions
pub struct CompletionService {
    provider: Arc<dyn LLMProvider>,
}

impl CompletionService {
    /// Create a new completion service
    pub fn new(provider: Arc<dyn LLMProvider>) -> Self {
        Self { provider }
    }

    /// Get the provider
    pub fn provider(&self) -> &Arc<dyn LLMProvider> {
        &self.provider
    }

    /// Generate a completion
    pub async fn generate(
        &self,
        prompt: &str,
        max_tokens: Option<usize>,
        temperature: Option<f32>,
    ) -> Result<String, LLMError> {
        self.provider.complete(prompt, max_tokens, temperature).await
    }

    /// Generate a completion with messages
    pub async fn generate_with_messages(
        &self,
        messages: Vec<HashMap<String, String>>,
        max_tokens: Option<usize>,
        temperature: Option<f32>,
    ) -> Result<String, LLMError> {
        self.provider
            .complete_with_messages(messages, max_tokens, temperature)
            .await
    }

    /// Generate a JSON completion
    pub async fn generate_json<T: for<'de> Deserialize<'de>>(
        &self,
        prompt: &str,
        max_tokens: Option<usize>,
        temperature: Option<f32>,
    ) -> Result<T, LLMError> {
        let response = self.generate(prompt, max_tokens, temperature).await?;
        serde_json::from_str(&response).map_err(|e| LLMError::DeserializationError(e.to_string()))
    }

    /// Generate a completion with tools
    pub async fn generate_with_tools(
        &self,
        messages: Vec<HashMap<String, String>>,
        tools: Vec<HashMap<String, String>>,
        max_tokens: Option<usize>,
        temperature: Option<f32>,
    ) -> Result<HashMap<String, String>, LLMError> {
        self.provider
            .complete_with_tools(messages, tools, max_tokens, temperature)
            .await
    }

    /// Generate completion from messages (SDK compatibility method)
    pub async fn generate_completion(
        &self,
        messages: Vec<HashMap<String, String>>,
    ) -> Result<String, LLMError> {
        self.provider
            .complete_with_messages(messages, None, None)
            .await
    }

    /// Generate completion with tools (SDK compatibility method)
    pub async fn generate_completion_with_tools(
        &self,
        messages: Vec<HashMap<String, String>>,
        tools: Vec<HashMap<String, String>>,
    ) -> Result<HashMap<String, String>, LLMError> {
        self.provider
            .complete_with_tools(messages, tools, None, None)
            .await
    }

    /// Generate JSON completion (SDK compatibility method)
    pub async fn generate_json_completion(
        &self,
        messages: Vec<HashMap<String, String>>,
    ) -> Result<String, LLMError> {
        self.provider
            .complete_with_messages(messages, None, Some(0.1))
            .await
    }
}