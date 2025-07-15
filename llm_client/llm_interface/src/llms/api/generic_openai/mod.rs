// Public modules
pub mod tool_use;

// Internal imports
use super::*;
use llm_models::ApiLlmModel;
use crate::requests::{CompletionError, CompletionRequest, CompletionResponse};
use std::pin::Pin;
use crate::llms::api::ClientError;
use futures_util::{Stream, StreamExt};
use llm_devices::LoggingConfig;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use secrecy::{SecretString as Secret, ExposeSecret};

// Use the OpenAI types from the crate
use crate::llms::api::openai::completion::req::OpenAiCompletionRequest;
use crate::requests::CompletionResponseChunk;

pub struct GenericApiBackend {
    pub(crate) client: ApiClient<GenericApiConfig>,
    pub model: ApiLlmModel,
}

impl GenericApiBackend {
    pub fn new(mut config: GenericApiConfig, model: ApiLlmModel) -> crate::Result<Self> {
        config.logging_config.load_logger()?;
        if let Ok(api_key) = config.api_config.load_api_key() {
            config.api_config.api_key = Some(api_key);
        }
        Ok(Self {
            client: ApiClient::new(config),
            model,
        })
    }
    pub(crate) async fn completion_request(
        &self,
        request: &CompletionRequest,
    ) -> crate::Result<CompletionResponse, CompletionError> {
        match self
            .client
            .post(
                &self.client.config.completion_path,
                OpenAiCompletionRequest::new(request)?,
            )
            .await
        {
            Err(e) => Err(CompletionError::ClientError(e)),
            Ok(res) => Ok(CompletionResponse::new_from_openai(request, res)?),
        }
    }
    
    /// Handles a streaming completion request for a GenericApi backend
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
        let api_request = OpenAiCompletionRequest::new(request)?;
        
        // Stream events from the GenericApi endpoint
        let event_stream_result = self
            .client
            .post_stream(&self.client.config.completion_path, api_request)
            .await;
        
        match event_stream_result {
            Err(client_error) => Err(CompletionError::ClientError(client_error)),
            Ok(event_stream) => {
                // Define state for stream processing
                #[derive(Default)]
                struct StreamState {
                    // Store ID once received
                    completion_id: String,
                }
                
                // Create a pinned stream upfront to address Unpin requirement
                let pinned_stream = Box::pin(event_stream);
                
                Ok(Box::pin(futures_util::stream::try_unfold(
                    (pinned_stream, StreamState::default()),
                    |(mut stream, mut state)| async move {
                        match stream.next().await {
                            Some(event_result) => {
                                match event_result {
                                    Ok(event) => {
                                        // Parse event as a CompletionResponseChunk
                                        let event_str = serde_json::to_string(&event)
                                            .unwrap_or_else(|_| format!("{:?}", event));
                                        let chunk: CompletionResponseChunk = serde_json::from_str(&event_str)
                                            .map_err(|e| CompletionError::ClientError(ClientError::StreamParseError(e.to_string())))?;
                                        if state.completion_id.is_empty() && !chunk.id.is_empty() {
                                            state.completion_id = chunk.id.clone();
                                        }
                                        Ok(Some((chunk, (stream, state))))
                                    }
                                    Err(e) => Err(CompletionError::ClientError(ClientError::StreamParseError(e.to_string()))),
                                }
                            }
                            None => Ok(None),
                        }
                    },
                )))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct GenericApiConfig {
    pub api_config: ApiConfig,
    pub logging_config: LoggingConfig,
    pub completion_path: String,
}

impl Default for GenericApiConfig {
    fn default() -> Self {
        Self {
            api_config: ApiConfig {
                host: Default::default(),
                port: None,
                api_key: None,
                api_key_env_var: Default::default(),
            },
            logging_config: LoggingConfig {
                logger_name: "generic".to_string(),
                ..Default::default()
            },
            completion_path: "/chat/completions".to_string(),
        }
    }
}

impl GenericApiConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn completion_path<S: Into<String>>(mut self, path: S) -> Self {
        self.completion_path = path.into();
        self
    }
}

impl ApiConfigTrait for GenericApiConfig {
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(api_key) = self.api_key() {
            if let Ok(header_value) =
                HeaderValue::from_str(&format!("Bearer {}", api_key.expose_secret()))
            {
                headers.insert(AUTHORIZATION, header_value);
            } else {
                crate::error!("Failed to create header value from authorization value");
            }
        }

        headers
    }

    fn url(&self, path: &str) -> String {
        if let Some(port) = &self.api_config.port {
            format!("https://{}:{}{}", self.api_config.host, port, path)
        } else {
            format!("https://{}{}", self.api_config.host, path)
        }
    }

    fn api_key(&self) -> &Option<Secret> {
        &self.api_config.api_key
    }
}
