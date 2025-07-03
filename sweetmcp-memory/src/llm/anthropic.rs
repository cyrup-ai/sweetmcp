//! Anthropic Claude LLM provider implementation

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::oneshot;

use crate::llm::{LLMError, LLMProvider, PendingCompletion, PendingEmbedding};

/// Anthropic provider
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
    api_base: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            model: model.unwrap_or_else(|| "claude-3-opus-20240229".to_string()),
            api_base: "https://api.anthropic.com/v1".to_string(),
        }
    }

    /// Set custom API base URL
    pub fn with_api_base(mut self, api_base: String) -> Self {
        self.api_base = api_base;
        self
    }
}

impl LLMProvider for AnthropicProvider {
    fn complete(&self, prompt: &str) -> PendingCompletion {
        let (tx, rx) = oneshot::channel();

        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let api_base = self.api_base.clone();
        let prompt = prompt.to_string();

        tokio::spawn(async move {
            let request = CompletionRequest {
                model,
                messages: vec![Message {
                    role: "user".to_string(),
                    content: prompt,
                }],
                max_tokens: 1024,
                temperature: 0.7,
            };

            let result = async {
                let response = client
                    .post(format!("{}/messages", api_base))
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| LLMError::NetworkError(e))?;

                match response.status() {
                    StatusCode::OK => {
                        let completion: CompletionResponse = response
                            .json()
                            .await
                            .map_err(|e| LLMError::NetworkError(e))?;

                        completion
                            .content
                            .first()
                            .and_then(|content| {
                                if content.content_type == "text" {
                                    Some(content.text.clone())
                                } else {
                                    None
                                }
                            })
                            .ok_or_else(|| {
                                LLMError::InvalidResponse("No text content in response".to_string())
                            })
                    }
                    StatusCode::UNAUTHORIZED => Err(LLMError::AuthenticationFailed(
                        "Invalid API key".to_string(),
                    )),
                    StatusCode::TOO_MANY_REQUESTS => Err(LLMError::RateLimitExceeded),
                    _ => {
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        Err(LLMError::ApiError(error_text))
                    }
                }
            }
            .await;

            let _ = tx.send(result);
        });

        PendingCompletion::new(rx)
    }

    fn embed(&self, _text: &str) -> PendingEmbedding {
        let (tx, rx) = oneshot::channel();

        // Anthropic doesn't provide embeddings API
        tokio::spawn(async move {
            let _ = tx.send(Err(LLMError::ApiError(
                "Anthropic does not provide embedding API".to_string(),
            )));
        });

        PendingEmbedding::new(rx)
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

// Request/Response types

#[derive(Serialize)]
struct CompletionRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct CompletionResponse {
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}
