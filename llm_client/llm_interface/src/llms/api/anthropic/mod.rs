// Public modules
pub mod builder;
pub mod completion;
pub mod tool_use;

// Internal imports
use std::pin::Pin;
use crate::llms::api::{ClientError, ApiClient, ApiConfig, ApiConfigTrait};
use crate::{
    llms::api::anthropic::completion::res::{
        AnthropicStreamEvent, ContentBlockStart, Delta, StopReason,
    },
    requests::{
        common::tools::ToolCallSummary,
        CompletionError,
        CompletionFinishReason, CompletionRequest, CompletionResponse,
        CompletionResponseChunk,
    },
};
use completion::req::AnthropicCompletionRequest;
use futures_util::{Stream, StreamExt};
use llm_devices::LoggingConfig;
use llm_models::api_models::ApiLlmModel;
use reqwest::header::{HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString as Secret};
use serde_json::Value as JsonValue;

/// Default v1 API base url
pub const ANTHROPIC_API_HOST: &str = "https://api.anthropic.com/v1"; // Use https
/// Reguired version header
pub const ANTHROPIC_VERSION_HEADER: &str = "anthropic-version";
/// Optional beta header
pub const ANTHROPIC_BETA_HEADER: &str = "anthropic-beta";

pub struct AnthropicBackend {
    pub(crate) client: ApiClient<AnthropicConfig>,
    pub model: ApiLlmModel,
}

impl AnthropicBackend {
    pub fn new(mut config: AnthropicConfig, model: ApiLlmModel) -> crate::Result<Self> {
        config.logging_config.load_logger()?;
        config.api_config.api_key = Some(config.api_config.load_api_key()?);
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
            .post("/messages", AnthropicCompletionRequest::new(request)?)
            .await
        {
            Err(e) => Err(CompletionError::ClientError(e)),
            Ok(res) => Ok(CompletionResponse::new_from_anthropic(request, res)?),
        }
    }

    /// Handles a streaming completion request.
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

        let api_request = AnthropicCompletionRequest::new(request)?;

        let event_stream_result = self.client.post_stream("/messages", api_request).await;

        match event_stream_result {
            Err(client_error) => Err(CompletionError::ClientError(client_error)),
            Ok(event_stream) => {
                // Use try_unfold for stateful processing of the stream
                #[derive(Default)]
                struct StreamState {
                    message_id: String,
                    // Store (id, name, accumulated_input_json_string) for active tool calls by index
                    active_tool_calls: std::collections::HashMap<usize, (String, String, String)>,
                    // Store completed tool call summaries to include in the final finish reason
                    completed_tool_calls: Vec<ToolCallSummary>,
                    // Store delta from last MessageDelta to attach to the *next* content chunk
                    pending_output_tokens_delta: Option<usize>,
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
                                    match event {
                                        AnthropicStreamEvent::MessageStart { message } => {
                                            state.message_id = message.id; // Capture message ID
                                        }
                                        AnthropicStreamEvent::ContentBlockStart {
                                            index,
                                            content_block,
                                        } => {
                                            if let ContentBlockStart::ToolUse { id, name, .. } =
                                                content_block
                                            {
                                                // Store active tool call info, initialize input buffer
                                                state.active_tool_calls.insert(
                                                    index,
                                                    (id.clone(), name.clone(), String::new()),
                                                );
                                                // Yield a chunk indicating tool call start (optional, could be empty)
                                                let chunk = CompletionResponseChunk {
                                                    id: state.message_id.clone(),
                                                    index: 0, // Anthropic doesn't index choices
                                                    text_delta: None,
                                                    tool_call_id: Some(id),
                                                    tool_call_name: Some(name), // Indicate start
                                                    tool_call_input_delta: None,
                                                    finish_reason: None,
                                                };
                                                return Ok(Some((chunk, (stream, state))));
                                            }
                                            // Ignore Text ContentBlockStart (delta will follow)
                                        }
                                        AnthropicStreamEvent::ContentBlockDelta {
                                            index,
                                            delta,
                                        } => {
                                            match delta {
                                                Delta::TextDelta { text } => {
                                                    if !text.is_empty() {
                                                        let chunk = CompletionResponseChunk {
                                                            id: state.message_id.clone(),
                                                            index: 0,
                                                            text_delta: Some(text),
                                                            tool_call_id: None,
                                                            tool_call_name: None,
                                                            tool_call_input_delta: None,
                                                            finish_reason: None,
                                                        };
                                                        return Ok(Some((chunk, (stream, state))));
                                                    }
                                                }
                                                Delta::InputJsonDelta { partial_json } => {
                                                    // Append partial JSON to the buffer for the active tool call
                                                    if let Some((
                                                        tool_id,
                                                        _tool_name,
                                                        input_buffer,
                                                    )) = state.active_tool_calls.get_mut(&index)
                                                    {
                                                        input_buffer.push_str(&partial_json); // Accumulate
                                                        let chunk = CompletionResponseChunk {
                                                            id: state.message_id.clone(),
                                                            index: 0,
                                                            text_delta: None,
                                                            tool_call_id: Some(tool_id.clone()),
                                                            tool_call_name: None, // Name was sent at start
                                                            tool_call_input_delta: Some(
                                                                partial_json,
                                                            ), // Send the delta
                                                            finish_reason: None,
                                                        };
                                                        return Ok(Some((chunk, (stream, state))));
                                                    } else {
                                                        tracing::warn!("Received InputJsonDelta for unknown/stopped tool call index: {}", index);
                                                    }
                                                }
                                            }
                                        }
                                        AnthropicStreamEvent::ContentBlockStop { index } => {
                                            // If this index corresponds to an active tool call, finalize it
                                            if let Some((id, name, accumulated_input)) =
                                                state.active_tool_calls.remove(&index)
                                            {
                                                // Attempt to parse the accumulated input JSON
                                                let parsed_input = match serde_json::from_str::<
                                                    JsonValue,
                                                >(
                                                    &accumulated_input
                                                ) {
                                                    Ok(json_value) => json_value,
                                                    Err(e) => {
                                                        tracing::error!(
                                                            "Failed to parse accumulated tool input JSON for tool_id {}: {}. Input: '{}'",
                                                            id, e, accumulated_input
                                                        );
                                                        // Represent parse failure as Null or an error string? Let's use Null.
                                                        JsonValue::Null
                                                    }
                                                };
                                                state.completed_tool_calls.push(ToolCallSummary {
                                                    name,
                                                    input: parsed_input,
                                                });
                                                // Note: Tool ID is tracked internally but not part of the ToolCallSummary
                                                // Optionally yield a chunk indicating tool call end (can be empty)
                                                // let chunk = CompletionResponseChunk { ... finish_reason: None ... };
                                                // return Ok(Some((chunk, (stream, state))));
                                            }
                                            // Ignore stop for text blocks
                                        }
                                        AnthropicStreamEvent::MessageDelta { delta, .. } => {
                                            // MessageDelta contains the final stop reason and usage.
                                            // We don't need to store usage delta here, it's handled by the final MessageStop.
                                            // The stop_reason from MessageDelta is often the *final* reason.
                                            // We'll use the reason from MessageStop for consistency.
                                            tracing::trace!("Received MessageDelta: {:?}", delta);
                                            // Store the usage delta to be included in the *next* content chunk or final chunk
                                            if let Some(usage) = delta.usage {
                                                state.pending_output_tokens_delta = Some(usage.output_tokens);
                                            }
                                        }
                                        AnthropicStreamEvent::MessageStop { stop_reason } => {
                                            // This is the definitive end signal. Yield the final chunk with the stop reason.
                                            // Use the stop_reason provided in this event if available, otherwise default.
                                            let final_reason = map_anthropic_stop_reason(
                                                stop_reason.unwrap_or(StopReason::EndTurn), // Use StopReason from res module
                                                &state.completed_tool_calls,
                                            );
                                            let final_chunk = CompletionResponseChunk {
                                                id: state.message_id.clone(),
                                                index: 0,
                                                text_delta: None,
                                                tool_call_id: None,
                                                tool_call_name: None,
                                                tool_call_input_delta: None,
                                                finish_reason: Some(final_reason),
                                            };
                                            // Yield the final chunk.
                                            return Ok(Some((final_chunk, (stream, state))));
                                        }
                                        AnthropicStreamEvent::Error { error } => {
                                            // Propagate errors received within the stream
                                            tracing::error!(
                                                "Error received in Anthropic stream: {:?}",
                                                error
                                            );
                                            return Err(CompletionError::ClientError(
                                                ClientError::ApiError(error.error),
                                            ));
                                        }
                                        AnthropicStreamEvent::Ping => {
                                            // Ignore Ping events
                                            tracing::trace!("Received Anthropic Ping");
                                        }
                                    }
                                }
                                Err(e) => {
                                    // Propagate client errors (e.g., connection issues, SSE parsing errors from ApiClient)
                                    tracing::error!("Error reading from Anthropic stream: {:?}", e);
                                    return Err(CompletionError::ClientError(e));
                                }
                            }
                        }
                        // Stream ended without a MessageStop event? This shouldn't happen with a valid stream.
                        tracing::warn!("Anthropic stream ended without a MessageStop event.");
                        Ok(None) // Terminate the unfold
                    },
                )))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnthropicConfig {
    pub api_config: ApiConfig,
    pub logging_config: LoggingConfig,
    pub anthropic_version: String,
    pub anthropic_beta: Option<String>,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_config: ApiConfig {
                host: ANTHROPIC_API_HOST.to_string(),
                port: None,
                api_key: None,
                api_key_env_var: "ANTHROPIC_API_KEY".to_string(),
            },
            logging_config: LoggingConfig {
                logger_name: "anthropic".to_string(),
                ..Default::default()
            },
            anthropic_version: "2023-06-01".to_string(),
            anthropic_beta: None,
        }
    }
}

impl AnthropicConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_anthropic_version<S: Into<String>>(mut self, version: S) -> Self {
        self.anthropic_version = version.into();
        self
    }

    pub fn with_anthropic_beta<S: Into<String>>(mut self, beta: S) -> Self {
        self.anthropic_beta = Some(beta.into());
        self
    }

    /// Enables the token-efficient tool use beta feature.
    /// See: https://docs.anthropic.com/en/docs/build-with-claude/tool-use/token-efficient-tool-use
    pub fn with_token_efficient_tool_use(mut self) -> Self {
        self.anthropic_beta = Some("tools-2024-05-16".to_string());
        self
    }
}

impl ApiConfigTrait for AnthropicConfig {
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        // Version Header (Required)
        match HeaderValue::from_str(&self.anthropic_version) {
            Ok(v) => {
                headers.insert(ANTHROPIC_VERSION_HEADER, v);
            }
            Err(e) => tracing::error!("Invalid Anthropic version header value: {}", e),
        }

        // Beta Header (Optional)
        if let Some(beta_version) = &self.anthropic_beta {
            match HeaderValue::from_str(beta_version) {
                Ok(v) => {
                    headers.insert(ANTHROPIC_BETA_HEADER, v);
                }
                Err(e) => tracing::error!("Invalid Anthropic beta header value: {}", e),
            }
        }

        // API Key Header (Required)
        if let Some(api_key) = self.api_key() {
            match HeaderValue::from_str(api_key.expose_secret()) {
                Ok(mut v) => {
                    v.set_sensitive(true); // Mark API key header as sensitive
                    headers.insert(reqwest::header::HeaderName::from_static("x-api-key"), v);
                }
                Err(e) => tracing::error!("Invalid API key header value: {}", e),
            }
        } else {
            tracing::error!("Anthropic API key is missing");
            // Potentially return an error here instead of just logging
        }

        // Content-Type is added by the ApiClient post/post_stream methods

        headers
    }

    fn url(&self, path: &str) -> String {
        // Ensure the host starts with https:// and path starts with /
        let host = &self.api_config.host;
        // Use url crate for robust joining
        match url::Url::parse(host) {
            Ok(mut base_url) => {
                // Ensure scheme is https if not present
                if base_url.scheme() != "https" && base_url.scheme() != "http" {
                    let _ = base_url.set_scheme("https"); // Ignore error if scheme cannot be set
                }
                // Join path segments carefully
                base_url
                    .path_segments_mut()
                    .unwrap()
                    .extend(path.split('/').filter(|s| !s.is_empty()));
                base_url.to_string()
            }
            Err(_) => {
                // Fallback for invalid base URLs, try simple concatenation
                tracing::warn!("Invalid base URL provided: {}", host);
                let base_url = if !host.starts_with("http://") && !host.starts_with("https://") {
                    format!("https://{}", host)
                } else {
                    host.to_string()
                };
                format!(
                    "{}{}",
                    base_url.trim_end_matches('/'),
                    if path.starts_with('/') { path } else { "/" }
                )
            }
        }
    }

    fn api_key(&self) -> &Option<Secret> {
        &self.api_config.api_key
    }
}

// Helper function to map Anthropic stop reasons to generic ones
fn map_anthropic_stop_reason(
    reason: completion::res::StopReason,
    completed_calls: &[ToolCallSummary], // Pass completed calls for ToolUse reason
) -> CompletionFinishReason {
    match reason {
        completion::res::StopReason::EndTurn => CompletionFinishReason::Eos,
        completion::res::StopReason::MaxTokens => CompletionFinishReason::StopLimit,
        completion::res::StopReason::StopSequence => {
            // We don't get the specific sequence in the stream delta/stop event easily
            CompletionFinishReason::NonMatchingStoppingSequence(None) // Indicate stop sequence, but specific one unknown here
        }
        completion::res::StopReason::ToolUse => {
            let calls = if completed_calls.is_empty() {
                None
            } else {
                Some(completed_calls.to_vec())
            };
            CompletionFinishReason::ToolUse(calls) // Include summaries if available
        }
    }
}
