//! OpenAI LLM provider implementation

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::oneshot;

use crate::llm::{LLMError, LLMProvider, PendingCompletion, PendingEmbedding};

/// OpenAI provider
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
    api_base: String,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            model: model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            api_base: "https://api.openai.com/v1".to_string(),
        }
    }

    /// Set custom API base URL
    pub fn with_api_base(mut self, api_base: String) -> Self {
        self.api_base = api_base;
        self
    }
}

impl LLMProvider for OpenAIProvider {
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
                temperature: 0.7,
                max_tokens: None,
            };

            let result = async {
                let response = client
                    .post(format!("{}/chat/completions", api_base))
                    .header("Authorization", format!("Bearer {}", api_key))
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
                            .choices
                            .first()
                            .map(|choice| choice.message.content.clone())
                            .ok_or_else(|| {
                                LLMError::InvalidResponse("No completion choices".to_string())
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

    fn embed(&self, text: &str) -> PendingEmbedding {
        let (tx, rx) = oneshot::channel();

        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let api_base = self.api_base.clone();
        let text = text.to_string();

        tokio::spawn(async move {
            let request = EmbeddingRequest {
                model: "text-embedding-ada-002".to_string(),
                input: text,
            };

            let result = async {
                let response = client
                    .post(format!("{}/embeddings", api_base))
                    .header("Authorization", format!("Bearer {}", api_key))
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| LLMError::NetworkError(e))?;

                match response.status() {
                    StatusCode::OK => {
                        let embedding_response: EmbeddingResponse = response
                            .json()
                            .await
                            .map_err(|e| LLMError::NetworkError(e))?;

                        embedding_response
                            .data
                            .first()
                            .map(|data| data.embedding.clone())
                            .ok_or_else(|| {
                                LLMError::InvalidResponse("No embedding data".to_string())
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
    temperature: f32,
    max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct CompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}
