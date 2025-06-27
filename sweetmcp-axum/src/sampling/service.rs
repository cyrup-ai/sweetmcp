use log::{self, error};
use rpc_router::HandlerResult;
use tokio::sync::{mpsc, oneshot};

// Remove the entire unused pub use crate::db block
use super::model::*;
// Import notification types
use crate::sampling::notifications::{
    SamplingProgressNotification, // , SamplingProgressData Removed
};

/// Handler for the sampling/createMessage method (returns AsyncSamplingResult).
pub fn sampling_create_message_pending(request: CreateMessageRequest) -> AsyncSamplingResult {
    let (tx_result, rx_result) = oneshot::channel();
    let (_tx_progress, _rx_progress) = mpsc::channel::<SamplingProgressNotification>(16); // Progress channel seems unused

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

                // For demonstration, we'll create a simple echo response
                let response = format!("Echo: {}", prompt_text);

                // Create the result
                let result = CreateMessageResult {
                    role: "assistant".to_string(),
                    content: McpMessageContent {
                        type_: "text".to_string(),
                        text: Some(response),
                        data: None,
                        mime_type: None,
                    },
                    model: "mock-model-v1".to_string(),
                    stop_reason: Some("endTurn".to_string()),
                    usage: Some(CompletionUsage {
                        completion_tokens: 50,
                        prompt_tokens: 100,
                        total_tokens: 150,
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
