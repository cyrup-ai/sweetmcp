// Public modules
pub mod builder;
pub mod completion;
pub mod server;
pub mod tool_use;

// Internal imports
use super::*;
use crate::llms::api::{ApiClient, ApiConfig, ApiConfigTrait};
use crate::requests::{CompletionError, CompletionRequest, CompletionResponseChunk, CompletionFinishReason, StoppingSequence};
use std::pin::Pin;
use crate::llms::local::llama_cpp::server::LlamaCppServer;
use crate::llms::local::llama_cpp::completion::LlamaCppCompletionRequest;
use llm_devices::LoggingConfig;

use llm_models::LocalLlmModel;
use secrecy::{SecretString as Secret, ExposeSecret};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use futures_util::{Stream, StreamExt};
use serde::Deserialize;

/// Streaming chunk response format from LlamaCpp
#[derive(Debug, Deserialize)]
struct LlamaCppStreamChunk {
    id: String,
    model: String,
    created: u64,
    content: Option<String>,
    stop: Option<bool>,
    stop_reason: Option<String>,
}

pub const LLAMA_CPP_API_HOST: &str = "localhost";
pub const LLAMA_CPP_API_PORT: &str = "3333";

pub struct LlamaCppBackend {
    pub model: LocalLlmModel,
    pub server: LlamaCppServer,
    pub(crate) client: ApiClient<LlamaCppConfig>,
}

impl LlamaCppBackend {
    pub async fn new(
        mut config: LlamaCppConfig,
        mut local_config: LocalLlmConfig,
        llm_loader: GgufLoader,
    ) -> crate::Result<Self> {
        config.logging_config.load_logger()?;
        if let Ok(api_key) = config.api_config.load_api_key() {
            config.api_config.api_key = Some(api_key);
        }
        local_config.device_config.initialize()?;
        let model = local_config.load_model(llm_loader)?;

        let mut server = LlamaCppServer::new(
            local_config.device_config,
            &config.api_config.host,
            &config.api_config.port,
            local_config.inference_ctx_size,
        )?;
        let client: ApiClient<LlamaCppConfig> = ApiClient::new(config);
        server.start_server(&client).await?;
        println!(
            "{} with model: {}",
            colorful::Colorful::bold(colorful::Colorful::color(
                "LlamaCppBackend Initialized",
                colorful::RGB::new(220, 0, 115)
            )),
            model.model_base.model_id
        );
        Ok(Self {
            client,
            server,
            model,
        })
    }

    pub(crate) async fn completion_request(
        &self,
        request: &CompletionRequest,
    ) -> crate::Result<CompletionResponse, CompletionError> {
        match self
            .client
            .post("/completion", LlamaCppCompletionRequest::new(request)?)
            .await
        {
            Err(e) => Err(CompletionError::ClientError(e)),
            Ok(res) => Ok(CompletionResponse::new_from_llama(request, res)?),
        }
    }
    
    /// Handles a streaming completion request for LlamaCpp backend
    pub(crate) async fn completion_stream(
        &self,
        request: &CompletionRequest,
    ) -> crate::Result<
        Pin<Box<dyn Stream<Item = Result<CompletionResponseChunk, CompletionError>> + Send>>,
        CompletionError,
    > {
        // Ensure stream flag is set in the underlying request config
        if !request.config.stream_response {
            return Err(CompletionError::RequestBuilderError(
                "Streaming requested via completion_stream, but config.stream_response is false."
                    .to_string(),
            ));
        }
        
        // Create the API request with stream flag set
        let mut api_request = LlamaCppCompletionRequest::new(request)?;
        api_request.stream = Some(true);
        
        // Stream events from LlamaCpp API
        let event_stream_result = self.client.post_stream("/completion", api_request).await;
        
        match event_stream_result {
            Err(client_error) => Err(CompletionError::ClientError(client_error)),
            Ok(event_stream) => {
                // Define state for stream processing
                #[derive(Default)]
                struct StreamState {
                    // Store completion metadata once received
                    completion_id: String,
                    model_name: String,
                    created_timestamp: Option<u64>,
                }
                
                // Create a pinned stream up front to address Unpin requirement
                let pinned_stream = Box::pin(event_stream);
                
                Ok(Box::pin(futures_util::stream::try_unfold(
                    (pinned_stream, StreamState::default()),
                    |(mut stream, mut state)| async move {
                        // Stream is already pinned
                        while let Some(event_result) = stream.as_mut().next().await {
                            match event_result {
                                Ok(event) => {
                                    // Parse the SSE event as a LlamaCpp streaming response
                                    let event_str = serde_json::to_string(&event)
                                        .unwrap_or_else(|_| format!("{:?}", event));
                                    if let Ok(delta) = serde_json::from_str::<LlamaCppStreamChunk>(&event_str) {
                                        // Update completion metadata if needed
                                        if state.completion_id.is_empty() && !delta.id.is_empty() {
                                            // Store all relevant metadata from the first chunk
                                            state.completion_id = delta.id.clone();
                                            state.model_name = delta.model.clone();
                                            state.created_timestamp = Some(delta.created);
                                            
                                            // Log model and creation information when we first get the ID
                                            tracing::debug!(
                                                "LlamaCpp stream started: id={}, model={}, created={}",
                                                delta.id,
                                                delta.model,
                                                delta.created
                                            );
                                        }
                                        
                                        // Check if this is a content delta
                                        if let Some(content) = &delta.content {
                                            if !content.is_empty() {
                                                // Return content delta
                                                let chunk = CompletionResponseChunk {
                                                    id: state.completion_id.clone(),
                                                    index: 0,
                                                    text_delta: Some(content.clone()),
                                                    tool_call_id: None,
                                                    tool_call_name: None,
                                                    tool_call_input_delta: None,
                                                    finish_reason: None,
                                                };
                                                return Ok(Some((chunk, (stream, state))));
                                            }
                                        }
                                        
                                        // Check if this is a stop signal
                                        if delta.stop.unwrap_or(false) {
                                            let finish_reason = match delta.stop_reason.as_deref() {
                                                Some("stop") => CompletionFinishReason::MatchingStoppingSequence(StoppingSequence::InferenceDone("stop".to_string())),
                                                Some("length") => CompletionFinishReason::Eos,
                                                _ => CompletionFinishReason::Eos,
                                            };
                                            
                                            // Log completion details
                                            if let Some(timestamp) = state.created_timestamp {
                                                let elapsed = std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs()
                                                    .saturating_sub(timestamp);
                                                
                                                tracing::debug!(
                                                    "LlamaCpp stream completed: id={}, model={}, reason={:?}, duration={}s",
                                                    state.completion_id,
                                                    state.model_name,
                                                    finish_reason,
                                                    elapsed
                                                );
                                            }
                                            
                                            // Return completion finish chunk
                                            let chunk = CompletionResponseChunk {
                                                id: state.completion_id.clone(),
                                                index: 0,
                                                text_delta: None,
                                                tool_call_id: None,
                                                tool_call_name: None,
                                                tool_call_input_delta: None,
                                                finish_reason: Some(finish_reason),
                                            };
                                            return Ok(Some((chunk, (stream, state))));
                                        }
                                        
                                        // If we reached here, we don't have any useful data in this event
                                        continue;
                                    }
                                }
                                Err(e) => return Err(CompletionError::ClientError(e)),
                            }
                        }
                        Ok(None)
                    },
                )))
            }
        }
    }

    pub(crate) fn shutdown(&self) {
        match self.server.shutdown() {
            Ok(_) => (),
            Err(e) => crate::error!("Failed to shutdown server: {}", e),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LlamaCppConfig {
    pub api_config: ApiConfig,
    pub logging_config: LoggingConfig,
}

impl Default for LlamaCppConfig {
    fn default() -> Self {
        Self {
            api_config: ApiConfig {
                host: LLAMA_CPP_API_HOST.to_string(),
                port: Some(LLAMA_CPP_API_PORT.to_string()),
                api_key: None,
                api_key_env_var: "LLAMA_API_KEY".to_string(),
            },
            logging_config: LoggingConfig {
                logger_name: "llama_cpp".to_string(),
                ..Default::default()
            },
        }
    }
}

impl LlamaCppConfig {
    pub fn new() -> Self {
        Default::default()
    }
}

impl ApiConfigTrait for LlamaCppConfig {
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        if let Some(api_key) = self.api_key() {
            headers.insert(
                AUTHORIZATION,
                format!("Bearer {}", api_key.expose_secret())
                    .as_str()
                    .parse()
                    .unwrap(),
            );
        }

        headers
    }

    fn url(&self, path: &str) -> String {
        if let Some(port) = &self.api_config.port {
            format!("http://{}:{}{}", self.api_config.host, port, path)
        } else {
            format!("http://{}:{}", self.api_config.host, path)
        }
    }

    fn api_key(&self) -> &Option<Secret> {
        &self.api_config.api_key
    }
}
