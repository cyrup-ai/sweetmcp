// use futures_util::StreamExt; // Temporarily unused
use arrayvec::ArrayString;
use log::{self, error};
use rpc_router::HandlerResult;
use smallvec::SmallVec;
use std::env;
use tokio::sync::{mpsc, oneshot};
// use fluent_ai::{FluentAi, Providers, Models}; // Temporarily disabled due to dependency issues

use super::model::*;
// use crate::auth::JwtAuth; // Auth module not available
use crate::sampling::notifications::SamplingProgressNotification;

/// Select the best LLM model using fluent-ai based on model preferences
async fn select_llm_model(
    preferences: &Option<McpModelPreferences>,
) -> Result<(String, String), String> {
    // Default priorities if not specified
    let mut cost_priority = 0.5;
    let mut speed_priority = 0.5;
    let mut intelligence_priority = 0.5;

    // Use ArrayString for zero-allocation string handling (up to 64 chars)
    let mut model_hint: ArrayString<64> = ArrayString::new();

    if let Some(prefs) = preferences {
        cost_priority = prefs.cost_priority.unwrap_or(0.5);
        speed_priority = prefs.speed_priority.unwrap_or(0.5);
        intelligence_priority = prefs.intelligence_priority.unwrap_or(0.5);

        // Get first hint if available
        if let Some(hints) = &prefs.hints {
            if let Some(first_hint) = hints.first() {
                let hint_lowercase = first_hint.name.to_lowercase();
                if let Err(_) = model_hint.try_push_str(&hint_lowercase) {
                    // If hint is too long, just use the first 63 chars
                    let truncated = &hint_lowercase[..63.min(hint_lowercase.len())];
                    model_hint.clear();
                    let _ = model_hint.try_push_str(truncated);
                }
            }
        }
    }

    // Check environment for API keys
    let has_anthropic = env::var("ANTHROPIC_API_KEY").is_ok();
    let has_openai = env::var("OPENAI_API_KEY").is_ok();

    // Model selection based on hints and priorities - using zero-allocation const strings
    let (provider, model) = if model_hint.contains("claude") && has_anthropic {
        // Claude models - prioritize based on needs
        if intelligence_priority > 0.7 {
            ("claude", "claude-3-opus-20240229")
        } else if speed_priority > 0.7 {
            ("claude", "claude-3-haiku-20240307")
        } else {
            ("claude", "claude-3-sonnet-20240229")
        }
    } else if (model_hint.contains("gpt") || model_hint.contains("openai")) && has_openai {
        // OpenAI models
        if intelligence_priority > 0.7 {
            ("openai", "gpt-4-turbo")
        } else if cost_priority > 0.7 {
            ("openai", "gpt-3.5-turbo")
        } else {
            ("openai", "gpt-4")
        }
    } else if has_anthropic {
        // Default to Claude Sonnet if Anthropic is available
        ("claude", "claude-3-sonnet-20240229")
    } else if has_openai {
        // Default to GPT-4 if OpenAI is available
        ("openai", "gpt-4")
    } else {
        // Fallback to a simple model that might work locally
        ("openai", "gpt-3.5-turbo")
    };

    Ok((provider.to_string(), model.to_string()))
}

/// Handler for the sampling/createMessage method (returns AsyncSamplingResult).
pub fn sampling_create_message_pending(request: CreateMessageRequest) -> AsyncSamplingResult {
    let (tx_result, rx_result) = oneshot::channel();
    // Channel for streaming results (if needed in the future)
    let (_tx_stream, rx_stream) = mpsc::channel::<HandlerResult<CreateMessageResult>>(16);

    tokio::spawn(async move {
        log::info!("Received sampling/createMessage request: {:?}", request);

        // Mock implementation: Replace with real LLM calls via MCP client requests.

        // Extract the last user message for demonstration
        let last_message = request
            .messages
            .last()
            .ok_or_else(|| rpc_router::HandlerError::new("No messages provided"));

        let result = match last_message {
            Ok(last_message) => {
                // Get the text from the last message (if it's a text message)
                let prompt_text = match &last_message.content {
                    McpMessageContent { type_, text, .. } if type_ == "text" && text.is_some() => {
                        text.as_ref().unwrap()
                    }
                    _ => {
                        return {
                            let _ = tx_result.send(Err(rpc_router::HandlerError::new(
                                "Last message must be text",
                            )));
                            ()
                        };
                    }
                };

                // Report initial progress if request has meta params
                if let Some(meta) = &request.meta {
                    // Create a progress channel
                    let (tx_progress, _rx_progress) =
                        mpsc::channel::<HandlerResult<SamplingProgressNotification>>(16);
                    report_sampling_progress(&tx_progress, meta.progress_token.clone(), 0, 150);
                }

                // Use fluent-ai to generate actual response
                let (provider, model) = match select_llm_model(&request.model_preferences).await {
                    Ok((provider, model)) => (provider, model),
                    Err(e) => {
                        error!("Failed to select LLM model: {}", e);
                        return {
                            let _ = tx_result.send(Err(rpc_router::HandlerError::new(
                                "Failed to select LLM model",
                            )));
                            ()
                        };
                    }
                };

                // For now, create a simple response since the full API isn't available yet
                // TODO: Replace with actual fluent-ai completion when API is ready

                // Use SmallVec for zero-allocation response building for typical response sizes
                let mut response_parts: SmallVec<[&str; 8]> = SmallVec::new();
                response_parts.push("Echo (fluent-ai ");
                response_parts.push(&provider);
                response_parts.push(":");
                response_parts.push(&model);
                response_parts.push("): ");
                response_parts.push(prompt_text);

                let response_text = response_parts.join("");
                let model_name = model.clone();

                // Create the result
                let result = CreateMessageResult {
                    role: "assistant".to_string(),
                    content: McpMessageContent {
                        type_: "text".to_string(),
                        text: Some(response_text),
                        data: None,
                        mime_type: None,
                    },
                    model: model_name,
                    stop_reason: Some("endTurn".to_string()),
                    usage: Some(CompletionUsage {
                        completion_tokens: 150, // Estimate - could be improved with actual token counting
                        prompt_tokens: prompt_text.len() as u32 / 4, // Rough estimate
                        total_tokens: 150 + (prompt_text.len() as u32 / 4),
                    }),
                };

                log::info!("Returning sampling result: {:?}", result);
                Ok(result)
            }
            Err(e) => Err(e),
        };

        match result {
            Ok(value) => {
                // Assuming `value` here is the CreateMessageResult
                // We need to send CompletionUsage
                let usage = match value.usage.clone() {
                    Some(usage) => usage,
                    None => {
                        error!("Sampling result missing usage data");
                        let _ = tx_result.send(Err(rpc_router::HandlerError::new(
                            "Internal error: Missing usage data",
                        )));
                        return;
                    }
                };
                let _ = tx_result.send(Ok(usage));

                // Commenting out the previous incorrect logic
                /*
                match serde_json::from_str::<CreateMessageResult>(&value) {
                    Ok(parsed_result) => {
                        // Simulate work and potential usage calculation
                        tokio::time::sleep(Duration::from_millis(200)).await;
                        let usage = CompletionUsage {
                            prompt_tokens: 50,  // Example value
                            completion_tokens: 150, // Example value
                            total_tokens: 200, // Example value
                        };
                        // Send CompletionUsage, not CreateMessageResult
                        let _ = tx_result.send(Ok(usage));
                    }
                    Err(e) => {
                        error!("Failed to parse sampling result: {}", e);
                        // Ensure error type matches receiver expectation if needed
                        let _ = tx_result.send(Err(e.into_handler_error()));
                    }
                }
                */
            }
            Err(e) => {
                error!("Sampling message creation failed: {}", e);
                // Ensure error type matches receiver expectation if needed
                let _ = tx_result.send(Err(e)); // Send the original HandlerError
            }
        }
    });

    // Return AsyncSamplingResult which expects Result<CompletionUsage, ...>
    AsyncSamplingResult { rx: rx_result }
}

pub fn sampling_create_message(request: CreateMessageRequest) -> AsyncSamplingResult {
    sampling_create_message_pending(request)
}

/// Create a streaming sampling result (for future use with streaming LLMs)
pub fn sampling_create_message_stream(_request: CreateMessageRequest) -> SamplingStream {
    let (tx_stream, rx_stream) = mpsc::channel::<HandlerResult<CreateMessageResult>>(16);

    // In the future, this would stream tokens as they're generated
    tokio::spawn(async move {
        // Placeholder - would integrate with streaming LLM APIs
        drop(tx_stream);
    });

    SamplingStream::new(rx_stream)
}

// Restore unused function - signature updated
fn report_sampling_progress(
    tx_progress: &mpsc::Sender<HandlerResult<SamplingProgressNotification>>,
    request_id: String, // Added request_id
    tokens: u32,        // Renamed progress to tokens for clarity?
    total_tokens: u32,  // Renamed total to total_tokens for clarity?
) {
    // Correctly initialize SamplingProgressNotification
    let progress_notification = SamplingProgressNotification {
        request_id,
        progress: tokens,    // Map tokens to progress field
        total: total_tokens, // Map total_tokens to total field
        message: None,       // No message for now
    };
    // Removed incorrect SamplingProgressData usage
    // let progress_notification = SamplingProgressNotification {
    // progress: SamplingProgressData { // Now resolved
    // tokens,
    // total_tokens,
    // estimated_completion_time: None, // Not implemented
    // },
    // };

    // Try to send, but ignore error if receiver is closed
    let _ = tx_progress.try_send(Ok(progress_notification));
}
